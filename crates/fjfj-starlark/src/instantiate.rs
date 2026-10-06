//! Instantiating a rule from a BUILD file (buildfiji-mum.3.5): the call
//! `filegroup(name = "a", srcs = [...])` or `my_rule(name = "a", ...)`,
//! checked against the [`RuleSchema`] of the rule class and recorded as a
//! target of the package.
//!
//! A native rule and a `rule()` from a `.bzl` take this one path; all that
//! differs is where the schema came from. What Bazel 9.2.0 does, all of it
//! read off probes:
//!
//! - A call takes keyword arguments only. `name` is required and is a string;
//!   any other attribute the schema does not have, or that is private, is an
//!   *event* (`no such attribute 'x' in 'r' rule`, with a did-you-mean), as
//!   is a value of the wrong type, and the file goes on. A `None` is as if
//!   the attribute were not set.
//! - A list-typed attribute takes any list, tuple, range, dict (its keys),
//!   set or depset, and a string is refused; a label is a string read in
//!   the BUILD file's package and repo, or a `Label`.
//! - Then, in order, the events for mandatory attributes left out, values not
//!   in `values`, a label given twice, a label that reaches into a
//!   subpackage, and a test's size and timeout; then the target is added,
//!   which is fatal if its name is not a name, or conflicts with another
//!   target, and last its output files.
//! - A rule with `outputs`, and a rule with `attr.output` attributes, create
//!   generated files in the package. A file that cannot be a target name or
//!   reaches into a subpackage is an event; one that is another target's
//!   name, or made twice by the rule, is fatal.

use crate::args::{describe, fatal};
use crate::depset::{depset_to_list, is_depset};
use crate::label::{StarlarkLabel, display_label, label_of_value};
use crate::native::{BuildContext, call_frames, call_name_argument, context_for, location};
use crate::select;
use fjfj_graph::package::{PackageError, check_subpackage_crossing};
use fjfj_graph::rule::{
    AttrType, AttrValue, RuleClass, Selector, SelectorList, default_condition, label_relative_to,
};
use fjfj_graph::schema::{RuleSchema, SchemaAttr, TEST_SIZES, TEST_TIMEOUTS};
use fjfj_graph::visibility::Visibility;
use fjfj_graph::{Label, LabelParseError};
use starlark::collections::SmallMap;
use starlark::eval::{Arguments, Evaluator};
use starlark::values::dict::{AllocDict, DictRef};
use starlark::values::none::NoneType;
use starlark::values::tuple::AllocTuple;
use starlark::values::{Heap, StringValue, Value};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

/// The schema of a native rule, built once.
pub(crate) fn native_schema(class: &'static RuleClass) -> Arc<RuleSchema> {
    static SCHEMAS: LazyLock<Mutex<HashMap<&'static str, Arc<RuleSchema>>>> =
        LazyLock::new(|| Mutex::new(HashMap::new()));
    SCHEMAS
        .lock()
        .unwrap()
        .entry(class.name)
        .or_insert_with(|| Arc::new(RuleSchema::native(class)))
        .clone()
}

/// Which words a rule call's errors are in: the natives' or a `rule()`'s.
fn context<'a, 'e>(
    eval: &Evaluator<'_, 'a, 'e>,
    class: &str,
    schema: &RuleSchema,
) -> starlark::Result<&'a BuildContext<'e>> {
    if schema.starlark {
        return eval
            .extra
            .and_then(|extra| extra.downcast_ref::<BuildContext>())
            .ok_or_else(|| {
                fatal(
                    "a rule can only be instantiated while evaluating a BUILD file or a legacy \
                     or symbolic macro",
                )
            });
    }
    context_for(eval, class)
}

/// Instantiate the rule `class`, whose schema is `schema`, from the keyword
/// arguments of a call. `outputs` is the function of a `rule(outputs = f)`.
pub(crate) fn call_rule<'v>(
    schema: &Arc<RuleSchema>,
    class: &str,
    outputs: Option<Value<'v>>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<NoneType> {
    if args.positions(eval.heap())?.next().is_some() {
        return Err(fatal(if schema.starlark {
            "Unexpected positional arguments"
        } else {
            "unexpected positional arguments"
        }));
    }
    let named: SmallMap<StringValue<'v>, Value<'v>> = args.names_map()?;
    instantiate(schema, class, outputs, named, eval)
}

/// [`call_rule`] for the attributes already collected.
pub(crate) fn instantiate<'v>(
    schema: &Arc<RuleSchema>,
    class: &str,
    outputs: Option<Value<'v>>,
    named: SmallMap<StringValue<'v>, Value<'v>>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<NoneType> {
    let ctx = context(eval, class, schema)?;
    let at = location(eval);
    let name_value = named
        .iter()
        .find(|(k, _)| k.as_str() == "name")
        .map(|(_, v)| *v)
        .ok_or_else(|| fatal(format!("{class} rule has no 'name' attribute")))?;
    let name = name_value
        .unpack_str()
        .ok_or_else(|| fatal(format!("{class} 'name' attribute must be a string")))?
        .to_owned();
    let me = Label {
        repo: ctx.repo.to_owned(),
        package: ctx.package.to_owned(),
        name: name.clone(),
    };
    let event = |message: String| ctx.event(&at, format!("{me}: {message}"));
    let heap = eval.heap();

    let mut attrs: Vec<(String, AttrValue)> = Vec::new();
    let mut provided: Vec<&str> = Vec::new();
    for (key, value) in named.iter() {
        let key = key.as_str();
        if key == "name" {
            continue;
        }
        let Some(attr) = schema.attr(key).filter(|a| a.settable()) else {
            let hint = schema
                .suggest(key)
                .map(|s| format!(" (did you mean '{s}'?)"))
                .unwrap_or_default();
            event(format!("no such attribute '{key}' in '{class}' rule{hint}"));
            continue;
        };
        match convert(ctx, class, attr, *value, heap) {
            Ok(Some(v)) => {
                for message in check_values(attr, &v) {
                    ctx.event(&at, format!("{me}: {message}"));
                }
                provided.push(&attr.name);
                attrs.push((key.to_owned(), v));
            }
            // `None` says "as if unset", which a mandatory attribute does not
            // forgive.
            Ok(None) => {}
            Err(message) => event(message),
        }
    }
    normalize(schema, &mut attrs);
    // A rule a macro makes says which: the macro's name, its function (the
    // one the BUILD file called) and where it was called.
    let frames = call_frames(eval);
    if frames.len() > 1 && !ctx.macros.borrow().inside() {
        let function = eval
            .call_stack()
            .frames
            .first()
            .map(|f| f.name.clone())
            .unwrap_or_default();
        let call_text = eval
            .call_stack()
            .frames
            .first()
            .and_then(|f| f.location.as_ref().map(|l| l.source_span().to_owned()))
            .unwrap_or_default();
        let macro_name = call_name_argument(&call_text).unwrap_or_else(|| name.clone());
        // Bazel writes the path of the BUILD file as the root package's
        // absolute path and others relative to the repository.
        let location = if ctx.repo.is_empty() && ctx.package.is_empty() {
            format!("{}/{}", ctx.lookup.package_dir("").display(), at)
        } else {
            at.clone()
        };
        for (key, value) in [
            ("generator_name", macro_name),
            ("generator_function", function),
            ("generator_location", location),
        ] {
            if !attrs.iter().any(|(k, _)| k == key) {
                attrs.push((key.to_owned(), AttrValue::String(value)));
            }
        }
    }
    for attr in &schema.attrs {
        if attr.name != "name" && attr.def.mandatory() && !provided.contains(&attr.name.as_str()) {
            event(format!(
                "missing value for mandatory attribute '{}' in '{class}' rule",
                attr.name
            ));
        }
    }
    // Labels given twice, then labels that reach into a subpackage.
    let is_output = |attr: &str| {
        schema
            .attr(attr)
            .is_some_and(|a| matches!(a.def.ty, AttrType::Output | AttrType::OutputList))
    };
    for (attr, value) in &attrs {
        if attr == "visibility" || attr == "transitive_configs" || is_output(attr) {
            continue;
        }
        for group in duplicate_groups(value) {
            let mut seen: std::collections::HashSet<&Label> = std::collections::HashSet::new();
            if let Some(again) = group.iter().find(|l| !seen.insert(*l)) {
                ctx.event(
                    &at,
                    format!(
                        "Label '{}' is duplicated in the '{attr}' attribute of rule '{name}'",
                        display_label_short(again)
                    ),
                );
                break;
            }
        }
    }
    // Whether a directory is a package is asked once per package.
    let is_package = |p: &str| {
        if let Some(known) = ctx.package_cache.borrow().get(p) {
            return *known;
        }
        let answer = ctx.lookup.is_package(p);
        ctx.package_cache.borrow_mut().insert(p.to_owned(), answer);
        answer
    };
    for (attr, value) in &attrs {
        if attr == "visibility" || attr == "transitive_configs" {
            continue;
        }
        let mut checked: std::collections::HashSet<Label> = std::collections::HashSet::new();
        for label in label_groups(value).into_iter().flatten() {
            if label.repo != ctx.repo
                || label.package != ctx.package
                || !checked.insert(label.clone())
            {
                continue;
            }
            // A macro's labels are not looked at (what Bazel does).
            if !ctx.macros.borrow().inside()
                && let Err(e) =
                    check_subpackage_crossing(ctx.repo, ctx.package, &label.name, &is_package)
            {
                ctx.late_event(&at, e.to_string());
            }
        }
    }
    if schema.test {
        test_events(ctx, &at, &name, &attrs, schema);
    }

    // A target may not take the name of a macro that did not make it.
    if !ctx.state.borrow().builder.has_target(&name) {
        let macros = ctx.macros.borrow();
        if let Some(owner) = macros.instances.get(&name)
            && !macros.stack.iter().any(|f| f.instance == *owner)
        {
            return Err(fatal(format!(
                "target '{name}' conflicts with an existing macro (and was not created by it)"
            )));
        }
    }
    let visibility = attrs.iter().find_map(|(k, v)| match (k.as_str(), v) {
        ("visibility", AttrValue::LabelList(labels)) => Some(visibility_of(ctx, labels)),
        _ => None,
    });
    let crossing = ctx
        .state
        .borrow_mut()
        .builder
        .add_rule_with(
            &name,
            class,
            schema.defined_in.clone(),
            attrs.clone(),
            visibility,
            &at,
        )
        .map_err(|e| fatal(e.to_string()))?;
    if frames.len() > 1 {
        ctx.state.borrow_mut().builder.set_stack(&name, frames);
    }
    if let Some(crossing) = crossing {
        ctx.event(&at, crossing.to_string());
    }
    ctx.schemas
        .borrow_mut()
        .insert(name.clone(), schema.clone());

    // The files the rule creates.
    let mut made: Vec<String> = Vec::new();
    let value_of = |attr: &SchemaAttr| -> Option<AttrValue> {
        attrs
            .iter()
            .find(|(k, _)| *k == attr.name)
            .map(|(_, v)| v.clone())
            .or_else(|| attr.def.default_value())
    };
    for attr in &schema.attrs {
        match (attr.def.ty, attrs.iter().find(|(k, _)| *k == attr.name)) {
            (AttrType::Output, Some((_, AttrValue::Label(l)))) => made.push(l.name.clone()),
            (AttrType::OutputList, Some((_, AttrValue::LabelList(ls)))) => {
                made.extend(ls.iter().map(|l| l.name.clone()))
            }
            _ => {}
        }
    }
    for (key, template) in &schema.outputs {
        match expand_template(&name, schema, template, &value_of) {
            Ok(files) => made.extend(files),
            // A placeholder that names an attribute that cannot be used is
            // reported under that attribute, and a template that is not one
            // under the output it makes.
            Err(TemplateError::Placeholder) => ctx.event(
                &at,
                format!(
                    "In rule {me}: For attribute '{key}' in outputs: Invalid placeholder(s) in \
                     template"
                ),
            ),
            Err(TemplateError::Type { attr, ty }) => ctx.event(
                &at,
                format!(
                    "In rule {me}: For attribute '{attr}' in outputs: Attributes of type {ty} \
                     cannot be used in an outputs substitution template"
                ),
            ),
        }
    }
    if let Some(function) = outputs {
        for file in call_outputs(ctx, &at, &me, &name, schema, &value_of, function, eval) {
            made.push(file);
        }
    }
    for (n, file) in made.iter().enumerate() {
        if made[..n].contains(file) {
            return Err(fatal(format!(
                "rule '{name}' has more than one generated file named '{file}'"
            )));
        }
    }
    for file in made {
        let added =
            ctx.state
                .borrow_mut()
                .builder
                .add_generated_file(&file, &name, &me.to_string(), &at);
        match added {
            Ok(()) => {}
            Err(e @ (PackageError::IllegalOutputName { .. } | PackageError::Subpackage(_))) => {
                ctx.event(&at, e.to_string());
            }
            Err(e) => return Err(fatal(e.to_string())),
        }
    }
    Ok(NoneType)
}

/// What Bazel does to a few attributes once they are set: `transitive_configs`
/// is a sorted set, and a rule that sets `applicable_licenses` and not
/// `package_metadata` has set the latter.
fn normalize(schema: &RuleSchema, attrs: &mut [(String, AttrValue)]) {
    for (name, value) in attrs.iter_mut() {
        if name == "transitive_configs"
            && let AttrValue::LabelList(labels) = value
        {
            labels.sort_by(|a, b| {
                (&a.repo, &a.package, &a.name).cmp(&(&b.repo, &b.package, &b.name))
            });
            labels.dedup();
        }
    }
    let hidden = schema.attr("applicable_licenses").is_some_and(|a| a.hidden);
    if hidden && !attrs.iter().any(|(k, _)| k == "package_metadata") {
        for (name, _) in attrs.iter_mut() {
            if name == "applicable_licenses" {
                *name = "package_metadata".to_owned();
            }
        }
    }
}

/// `//:a` as Bazel writes a label in a message about duplicates: a label in
/// the main repo has no repo part.
fn display_label_short(label: &Label) -> String {
    display_label(label)
}

fn visibility_of(ctx: &BuildContext<'_>, labels: &[Label]) -> Visibility {
    let strings: Vec<String> = labels.iter().map(ToString::to_string).collect();
    Visibility::parse(strings.iter().map(String::as_str), ctx.label_context())
        .unwrap_or_else(|_| Visibility::private())
}

/// The lists of labels in a value: none for what holds no labels, one for a
/// list or a single label, and one per entry of a dict of lists.
fn label_groups(value: &AttrValue) -> Vec<Vec<Label>> {
    match value {
        AttrValue::Label(l) => vec![vec![l.clone()]],
        AttrValue::LabelList(ls) => vec![ls.clone()],
        AttrValue::LabelKeyedStringDict(entries) => {
            vec![entries.iter().map(|(l, _)| l.clone()).collect()]
        }
        AttrValue::StringKeyedLabelDict(entries) => {
            vec![entries.iter().map(|(_, l)| l.clone()).collect()]
        }
        AttrValue::LabelListDict(entries) => entries.iter().map(|(_, ls)| ls.clone()).collect(),
        AttrValue::Select(list) => {
            let mut groups = conditions(list);
            groups.extend(in_branches(list, label_groups));
            groups
        }
        _ => Vec::new(),
    }
}

/// `f` of every value a select may give, in order.
fn in_branches(list: &SelectorList, f: fn(&AttrValue) -> Vec<Vec<Label>>) -> Vec<Vec<Label>> {
    list.elements
        .iter()
        .flat_map(|e| &e.branches)
        .filter_map(|(_, v)| v.as_ref())
        .flat_map(f)
        .collect()
}

/// The conditions a select names, which reaching into a subpackage is as much
/// an error for as a value is.
fn conditions(list: &SelectorList) -> Vec<Vec<Label>> {
    list.elements
        .iter()
        .map(|e| e.branches.iter().map(|(l, _)| l.clone()).collect())
        .collect()
}

/// The lists of labels in which a label given twice is an event: a list, and
/// each list of a dict of lists. (A dict's keys are checked when it is
/// converted, and its values may repeat.)
fn duplicate_groups(value: &AttrValue) -> Vec<Vec<Label>> {
    match value {
        AttrValue::LabelList(ls) => vec![ls.clone()],
        AttrValue::LabelListDict(entries) => entries.iter().map(|(_, ls)| ls.clone()).collect(),
        AttrValue::Select(list) => in_branches(list, duplicate_groups),
        _ => Vec::new(),
    }
}

/// `values = [...]`: the event for a value the attribute may not take.
pub(crate) fn check_values(attr: &SchemaAttr, value: &AttrValue) -> Vec<String> {
    if let AttrValue::Select(list) = value {
        return list
            .elements
            .iter()
            .flat_map(|e| &e.branches)
            .filter_map(|(_, v)| v.as_ref())
            .flat_map(|v| check_values(attr, v))
            .collect();
    }
    check_value(attr, value).into_iter().collect()
}

fn check_value(attr: &SchemaAttr, value: &AttrValue) -> Option<String> {
    if attr.values.is_empty() {
        return None;
    }
    let text = match value {
        AttrValue::String(s) => s.clone(),
        AttrValue::Int(i) => i.to_string(),
        _ => return None,
    };
    if attr.values.contains(&text) {
        return None;
    }
    let quoted: Vec<String> = attr.values.iter().map(|v| format!("'{v}'")).collect();
    let (last, rest) = quoted.split_last().expect("not empty");
    let list = if rest.is_empty() {
        last.clone()
    } else {
        format!("{} or {last}", rest.join(", "))
    };
    Some(format!(
        "invalid value in '{}' attribute: has to be one of {list} instead of '{text}'",
        attr.name
    ))
}

/// A test's `size` and `timeout` must be ones Bazel knows. A timeout not
/// given comes from the size, which is why an unknown size makes two events.
fn test_events(
    ctx: &BuildContext<'_>,
    at: &str,
    name: &str,
    attrs: &[(String, AttrValue)],
    schema: &RuleSchema,
) {
    let string_of = |attr: &str| {
        attrs
            .iter()
            .find(|(k, _)| k == attr)
            .and_then(|(_, v)| match v {
                AttrValue::String(s) => Some(s.clone()),
                _ => None,
            })
    };
    let default_size = schema
        .attr("size")
        .and_then(|a| a.def.default_value())
        .and_then(|v| match v {
            AttrValue::String(s) => Some(s),
            _ => None,
        })
        .unwrap_or_default();
    let size = string_of("size").unwrap_or(default_size);
    let size_ok = TEST_SIZES.contains(&size.as_str());
    if !size_ok {
        ctx.event(
            at,
            format!("In rule '{name}', size '{size}' is not a valid size."),
        );
    }
    match string_of("timeout") {
        Some(timeout) if !TEST_TIMEOUTS.contains(&timeout.as_str()) => ctx.event(
            at,
            format!("In rule '{name}', timeout '{timeout}' is not a valid timeout."),
        ),
        None if !size_ok => ctx.event(
            at,
            format!("In rule '{name}', timeout 'illegal' is not a valid timeout."),
        ),
        _ => {}
    }
}

// ---- attribute values -----------------------------------------------------------

/// Read `text` as a label written in the BUILD file being loaded.
fn parse_label(ctx: &BuildContext<'_>, text: &str) -> Result<Label, LabelParseError> {
    Label::parse_mapped(text, ctx.label_context(), &mut |apparent| {
        ctx.mappings.resolve_apparent(ctx.repo, apparent)
    })
}

/// A label written as a string or given as a `Label`. The error is `None`
/// for a value that is neither, and `Some(message)` for a string that is not
/// a label.
fn label_from<'v>(
    ctx: &BuildContext<'_>,
    value: Value<'v>,
    where_: &str,
) -> Result<Label, Option<String>> {
    if let Some(label) = label_of_value(value) {
        return Ok(label);
    }
    let Some(text) = value.unpack_str() else {
        return Err(None);
    };
    parse_label(ctx, text).map_err(|e| Some(format!("invalid label '{text}' in {where_}: {e}")))
}

/// The items of what a list-typed attribute takes: a list, tuple, range,
/// dict (its keys), set or depset.
fn items_of<'v>(value: Value<'v>, heap: Heap<'v>) -> Option<Vec<Value<'v>>> {
    if is_depset(value) {
        return depset_to_list(value).and_then(Result::ok);
    }
    if matches!(
        value.get_type(),
        "list" | "tuple" | "range" | "dict" | "set"
    ) {
        return value.iterate(heap).ok().map(|items| items.collect());
    }
    None
}

enum IntFail {
    NotInt,
    Range(String),
}

fn int32(value: Value<'_>) -> Result<i32, IntFail> {
    if let Some(i) = value.unpack_i32() {
        return Ok(i);
    }
    if value.get_type() == "int" {
        return Err(IntFail::Range(value.to_repr()));
    }
    Err(IntFail::NotInt)
}

/// Convert one attribute value to what the attribute holds, or say in
/// Bazel's words why not. `Ok(None)` is `None`: as if the attribute were not
/// set.
pub(crate) fn convert<'v>(
    ctx: &BuildContext<'_>,
    rule: &str,
    attr: &SchemaAttr,
    value: Value<'v>,
    heap: Heap<'v>,
) -> Result<Option<AttrValue>, String> {
    let at = format!("attribute '{}' of '{rule}'", attr.name);
    if let Some((elements, pipe)) = select::view(value) {
        return convert_select(ctx, attr, &at, &elements, pipe, heap);
    }
    convert_at(ctx, attr, &at, value, heap)
}

/// The key of a `select()`'s branch as a label: the default condition when it
/// is written as that, else read in the BUILD file's package and repo.
fn condition<'v>(ctx: &BuildContext<'_>, key: Value<'v>, at: &str) -> Result<Label, String> {
    if key.unpack_str() == Some("//conditions:default") {
        return Ok(default_condition());
    }
    let label = label_from(ctx, key, at).map_err(|e| e.unwrap_or_else(String::new))?;
    Ok(if label == default_condition() {
        default_condition()
    } else {
        label
    })
}

/// A `select()`, or plain values joined with some: each branch is converted
/// as the attribute's type, and what no configuration can change is the value
/// itself.
fn convert_select<'v>(
    ctx: &BuildContext<'_>,
    attr: &SchemaAttr,
    at: &str,
    elements: &[select::Element<'v>],
    pipe: bool,
    heap: Heap<'v>,
) -> Result<Option<AttrValue>, String> {
    if !attr.configurable {
        // The alias of `package_metadata` goes by that name.
        let name = if attr.hidden && attr.name == "applicable_licenses" {
            "package_metadata"
        } else {
            &attr.name
        };
        return Err(format!("attribute \"{name}\" is not configurable"));
    }
    let ty = attr.def.ty;
    if elements.len() > 1 && matches!(ty, AttrType::Bool | AttrType::Label) {
        let name = if ty == AttrType::Bool {
            "boolean"
        } else {
            "label"
        };
        return Err(format!(
            "type '{name}' doesn't support select concatenation"
        ));
    }
    let mut selectors: Vec<Selector> = Vec::new();
    for element in elements {
        if !element.select {
            let value = convert_at(ctx, attr, at, element.value, heap)?.or_else(|| ty.zero());
            selectors.push(Selector {
                branches: vec![(default_condition(), value)],
                no_match_error: String::new(),
                unconditional: true,
            });
            continue;
        }
        let dict = DictRef::from_value(element.value).expect("a select holds a dict");
        let mut branches: Vec<(Label, Option<AttrValue>)> = Vec::new();
        for (key, value) in dict.iter() {
            let label = condition(ctx, key, at)?;
            let within = format!(
                "each branch in select expression of {at} (including '{}')",
                display_label(&label)
            );
            let converted = convert_at(ctx, attr, &within, value, heap)?.or_else(|| ty.zero());
            match branches.iter_mut().find(|(l, _)| *l == label) {
                Some(branch) => branch.1 = converted,
                None => branches.push((label, converted)),
            }
        }
        // Whether it is unconditional is as written, before two spellings of
        // a label become one.
        let unconditional = dict.len() == 1
            && branches
                .first()
                .is_some_and(|(l, _)| *l == default_condition());
        selectors.push(Selector {
            branches,
            no_match_error: element.no_match_error.clone(),
            unconditional,
        });
    }
    let list = SelectorList {
        elements: selectors,
        pipe,
    };
    if list.elements.iter().all(|s| s.unconditional) {
        return Ok(list.flatten());
    }
    Ok(Some(AttrValue::Select(list)))
}

/// [`convert`] of a value that is not a select; `at` says where it is, for
/// the errors (`attribute 'a' of 'r'`, or a branch of a select in it).
fn convert_at<'v>(
    ctx: &BuildContext<'_>,
    attr: &SchemaAttr,
    at: &str,
    value: Value<'v>,
    heap: Heap<'v>,
) -> Result<Option<AttrValue>, String> {
    if value.is_none() {
        return Ok(None);
    }
    if attr.set && value.get_type() != "set" {
        return Err(format!(
            "expected value of type 'set(string)' for {at}, but got {}",
            describe(value)
        ));
    }
    let wrong_type = |expected: &str| {
        format!(
            "expected value of type '{expected}' for {at}, but got {}",
            describe(value)
        )
    };
    let ty = attr.def.ty;
    let string_at = |i: usize, item: Value<'_>| {
        item.unpack_str().map(str::to_owned).ok_or_else(|| {
            format!(
                "expected value of type 'string' for element {i} of {at}, but got {}",
                describe(item)
            )
        })
    };
    let in_current_package = |label: &Label, written: &str| -> Result<(), String> {
        if label.repo == ctx.repo && label.package == ctx.package {
            Ok(())
        } else {
            Err(format!("label '{written}' is not in the current package"))
        }
    };
    Ok(Some(match ty {
        AttrType::Bool => match (value.unpack_bool(), value.unpack_i32()) {
            (Some(b), _) => AttrValue::Bool(b),
            (None, Some(0)) => AttrValue::Bool(false),
            (None, Some(1)) => AttrValue::Bool(true),
            _ => {
                return Err(format!(
                    "expected one of [False, True, 0, 1] for {at}, but got {}",
                    describe(value)
                ));
            }
        },
        AttrType::Tristate => match (value.unpack_bool(), int32(value)) {
            (Some(b), _) => AttrValue::Int(i32::from(b)),
            (None, Ok(i @ -1..=1)) => AttrValue::Int(i),
            (None, Ok(i)) => {
                return Err(format!(
                    "expected value of type 'tristate' for TriState values is not one of \
                     [-1, 0, 1], but got {i} (int)"
                ));
            }
            (None, Err(IntFail::NotInt)) => return Err(wrong_type("int")),
            (None, Err(IntFail::Range(n))) => {
                return Err(format!(
                    "for {at}, got {n}, want value in signed 32-bit range"
                ));
            }
        },
        AttrType::Int => match int32(value) {
            Ok(i) => AttrValue::Int(i),
            Err(IntFail::NotInt) => return Err(wrong_type("int")),
            Err(IntFail::Range(n)) => {
                return Err(format!(
                    "for {at}, got {n}, want value in signed 32-bit range"
                ));
            }
        },
        AttrType::String => AttrValue::String(
            value
                .unpack_str()
                .map(str::to_owned)
                .ok_or_else(|| wrong_type("string"))?,
        ),
        AttrType::Label | AttrType::Output => {
            let label = label_from(ctx, value, at)
                .map_err(|e| e.unwrap_or_else(|| wrong_type("string")))?;
            if ty == AttrType::Output {
                let written = value
                    .unpack_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| display_label(&label));
                in_current_package(&label, &written)?;
            }
            AttrValue::Label(label)
        }
        AttrType::StringList => {
            let items = items_of(value, heap).ok_or_else(|| wrong_type("list(string)"))?;
            AttrValue::StringList(
                items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| string_at(i, *item))
                    .collect::<Result<_, _>>()?,
            )
        }
        AttrType::IntList => {
            let items = items_of(value, heap).ok_or_else(|| wrong_type("list(int)"))?;
            AttrValue::IntList(
                items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        int32(*item).map_err(|e| match e {
                            IntFail::NotInt => format!(
                                "expected value of type 'int' for element {i} of {at}, but got {}",
                                describe(*item)
                            ),
                            IntFail::Range(n) => format!(
                                "for element {i} of {at}, got {n}, want value in signed 32-bit \
                                 range"
                            ),
                        })
                    })
                    .collect::<Result<_, _>>()?,
            )
        }
        AttrType::LabelList | AttrType::OutputList => {
            let items = items_of(value, heap).ok_or_else(|| wrong_type(ty.name()))?;
            let mut labels = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                let label = label_from(ctx, *item, &format!("element {i} of {at}"))
                    .map_err(|e| e.unwrap_or_else(|| string_at(i, *item).unwrap_err()))?;
                if ty == AttrType::OutputList {
                    let written = item
                        .unpack_str()
                        .map(str::to_owned)
                        .unwrap_or_else(|| display_label(&label));
                    in_current_package(&label, &written)?;
                }
                labels.push(label);
            }
            AttrValue::LabelList(labels)
        }
        AttrType::StringDict
        | AttrType::StringListDict
        | AttrType::LabelKeyedStringDict
        | AttrType::StringKeyedLabelDict
        | AttrType::LabelListDict => {
            let dict = DictRef::from_value(value).ok_or_else(|| wrong_type(ty.name()))?;
            convert_dict(ctx, ty, at, &dict, heap)?
        }
    }))
}

/// A dict-typed attribute's entries. Bazel does not name the attribute in
/// what is wrong with one.
fn convert_dict<'v>(
    ctx: &BuildContext<'_>,
    ty: AttrType,
    at: &str,
    dict: &DictRef<'v>,
    heap: Heap<'v>,
) -> Result<AttrValue, String> {
    let bad = |what: &str, item: Value<'_>| {
        format!(
            "expected value of type '{what}' for dict {{}} element, but got {}",
            describe(item)
        )
    };
    let string_key = |key: Value<'_>| {
        key.unpack_str()
            .map(str::to_owned)
            .ok_or_else(|| bad("string", key).replace("{}", "key"))
    };
    let string_value = |value: Value<'_>| {
        value
            .unpack_str()
            .map(str::to_owned)
            .ok_or_else(|| bad("string", value).replace("{}", "value"))
    };
    let label_at = |item: Value<'v>, where_: &str, what: &str| -> Result<Label, String> {
        label_from(ctx, item, where_).map_err(|e| {
            e.unwrap_or_else(|| {
                format!(
                    "expected value of type 'string' for {what}, but got {}",
                    describe(item)
                )
            })
        })
    };
    let list_of = |value: Value<'v>, want: &str| -> Result<Vec<Value<'v>>, String> {
        items_of(value, heap).ok_or_else(|| bad(want, value).replace("{}", "value"))
    };
    Ok(match ty {
        AttrType::StringDict => AttrValue::StringDict(
            dict.iter()
                .map(|(k, v)| Ok((string_key(k)?, string_value(v)?)))
                .collect::<Result<_, String>>()?,
        ),
        AttrType::StringListDict => AttrValue::StringListDict(
            dict.iter()
                .map(|(k, v)| {
                    let key = string_key(k)?;
                    let items = list_of(v, "list(string)")?
                        .iter()
                        .enumerate()
                        .map(|(i, item)| {
                            item.unpack_str().map(str::to_owned).ok_or_else(|| {
                                format!(
                                    "expected value of type 'string' for element {i} of dict \
                                     value element, but got {}",
                                    describe(*item)
                                )
                            })
                        })
                        .collect::<Result<_, String>>()?;
                    Ok((key, items))
                })
                .collect::<Result<_, String>>()?,
        ),
        AttrType::LabelKeyedStringDict => {
            let mut entries: Vec<(Label, String)> = Vec::new();
            let mut written: Vec<String> = Vec::new();
            for (k, v) in dict.iter() {
                let label = label_at(k, "dict key element", "dict key element")?;
                entries.push((label, string_value(v)?));
                written.push(k.to_repr());
            }
            for (i, (label, _)) in entries.iter().enumerate() {
                if entries[..i].iter().any(|(l, _)| l == label) {
                    let all: Vec<&str> = entries
                        .iter()
                        .zip(&written)
                        .filter(|((l, _), _)| l == label)
                        .map(|(_, w)| w.as_str())
                        .collect();
                    return Err(format!(
                        "duplicate labels in {at}: {} (as [{}])",
                        display_label(label),
                        all.join(", ")
                    ));
                }
            }
            AttrValue::LabelKeyedStringDict(entries)
        }
        AttrType::StringKeyedLabelDict => AttrValue::StringKeyedLabelDict(
            dict.iter()
                .map(|(k, v)| {
                    Ok((
                        string_key(k)?,
                        label_at(v, "dict value element", "dict value element")?,
                    ))
                })
                .collect::<Result<_, String>>()?,
        ),
        AttrType::LabelListDict => AttrValue::LabelListDict(
            dict.iter()
                .map(|(k, v)| {
                    let key = string_key(k)?;
                    let labels = list_of(v, "list(label)")?
                        .iter()
                        .enumerate()
                        .map(|(i, item)| {
                            let where_ = format!("element {i} of dict value element");
                            label_at(*item, &where_, &where_)
                        })
                        .collect::<Result<_, String>>()?;
                    Ok((key, labels))
                })
                .collect::<Result<_, String>>()?,
        ),
        _ => unreachable!("not a dict type"),
    })
}

// ---- implicit outputs ------------------------------------------------------------

/// The name of the label `name` with its extension removed, which is what
/// an outputs template writes for a label attribute.
fn without_extension(name: &str) -> &str {
    let base = name.rsplit('/').next().unwrap_or(name);
    match base.rfind('.') {
        Some(dot) if dot > 0 => &name[..name.len() - (base.len() - dot)],
        _ => name,
    }
}

/// What is wrong with an outputs template.
enum TemplateError {
    /// A `%{}` that names nothing the rule has.
    Placeholder,
    /// A `%{attr}` of an attribute whose value cannot be written in a file
    /// name.
    Type { attr: String, ty: &'static str },
}

/// `%{name}.txt` for the rule `name`: each `%{attr}` is the rule's name, or
/// the value of a string attribute, or the name of a label (without its
/// extension), or the one element of a list of those.
fn expand_template(
    rule: &str,
    schema: &RuleSchema,
    template: &str,
    value_of: &dyn Fn(&SchemaAttr) -> Option<AttrValue>,
) -> Result<Vec<String>, TemplateError> {
    // Each placeholder stands for one piece of text, or for several when the
    // attribute is a list: one output per element.
    let mut outs: Vec<String> = vec![String::new()];
    let push = |outs: &mut Vec<String>, pieces: &[String]| {
        *outs = outs
            .iter()
            .flat_map(|out| pieces.iter().map(move |piece| format!("{out}{piece}")))
            .collect();
    };
    let mut rest = template;
    while let Some(start) = rest.find("%{") {
        push(&mut outs, &[rest[..start].to_owned()]);
        let after = &rest[start + 2..];
        let Some(end) = after.find('}') else {
            push(&mut outs, &[rest[start..].to_owned()]);
            rest = "";
            break;
        };
        let key = &after[..end];
        rest = &after[end + 1..];
        if key == "name" {
            push(&mut outs, &[rule.to_owned()]);
            continue;
        }
        let Some(attr) = schema.attr(key).filter(|_| !key.is_empty()) else {
            return Err(TemplateError::Placeholder);
        };
        let pieces: Vec<String> = match value_of(attr) {
            // `visibility` is a list of labels that a template cannot use.
            _ if attr.name == "visibility" => {
                return Err(TemplateError::Type {
                    attr: attr.name.clone(),
                    ty: attr.def.ty.name(),
                });
            }
            Some(AttrValue::String(s)) => vec![s],
            None if attr.def.ty == AttrType::String => vec![String::new()],
            Some(AttrValue::Label(l)) => vec![without_extension(&l.name).to_owned()],
            Some(AttrValue::StringList(items)) => items,
            Some(AttrValue::LabelList(items)) => items
                .iter()
                .map(|l| without_extension(&l.name).to_owned())
                .collect(),
            _ => {
                let ty = match attr.def.ty {
                    AttrType::Bool => "boolean",
                    other => other.name(),
                };
                return Err(TemplateError::Type {
                    attr: attr.name.clone(),
                    ty,
                });
            }
        };
        push(&mut outs, &pieces);
    }
    push(&mut outs, &[rest.to_owned()]);
    Ok(outs)
}

/// The files a rule's `outputs` function makes: it is called with the rule's
/// `name` and the attributes its parameters name.
#[allow(clippy::too_many_arguments)]
fn call_outputs<'v>(
    ctx: &BuildContext<'_>,
    at: &str,
    me: &Label,
    name: &str,
    schema: &RuleSchema,
    value_of: &dyn Fn(&SchemaAttr) -> Option<AttrValue>,
    function: Value<'v>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> Vec<String> {
    let mut wanted = parameter_names(function);
    if wanted.iter().any(|w| w == "**") {
        wanted.retain(|w| w != "**");
        wanted.extend(
            schema
                .attrs
                .iter()
                .filter(|a| a.settable())
                .map(|a| a.name.clone()),
        );
    }
    let heap = eval.heap();
    let mut named: Vec<(String, Value<'v>)> = Vec::new();
    for key in &wanted {
        if key == "name" {
            named.push((key.clone(), heap.alloc(name)));
        } else if let Some(attr) = schema.attr(key)
            && let Some(v) = value_of(attr)
        {
            named.push((key.clone(), attr_to_value(ctx, &v, heap)));
        }
    }
    let named: Vec<(&str, Value<'v>)> = named.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    let result = match eval.eval_function(function, &[], &named) {
        Ok(v) => v,
        Err(e) => {
            ctx.event(at, format!("In rule {me}: {e}"));
            return Vec::new();
        }
    };
    let Some(dict) = DictRef::from_value(result) else {
        ctx.event(
            at,
            format!(
                "In rule {me}: got {} for 'implicit outputs function return value', want dict",
                result.get_type()
            ),
        );
        return Vec::new();
    };
    let mut files = Vec::new();
    for (k, v) in dict.iter() {
        match (k.unpack_str(), v.unpack_str()) {
            (Some(_), Some(file)) => files.push(file.to_owned()),
            _ => {
                ctx.event(
                    at,
                    format!(
                        "In rule {me}: got dict<{}, {}> for 'implicit outputs function return \
                         value', want dict<string, string>",
                        k.get_type(),
                        v.get_type()
                    ),
                );
                return Vec::new();
            }
        }
    }
    files
}

/// The names of the parameters `function` takes.
fn parameter_names(function: Value<'_>) -> Vec<String> {
    use starlark::docs::{DocItem, DocMember};
    match function.documentation() {
        DocItem::Member(DocMember::Function(f)) => {
            let mut names: Vec<String> =
                f.params.regular_params().map(|p| p.name.clone()).collect();
            // A function that takes `**kwargs` is given every attribute.
            if f.params.kwargs.is_some() {
                names.push("**".to_owned());
            }
            names
        }
        _ => Vec::new(),
    }
}

// ---- what a rule shows ---------------------------------------------------------------

/// What `native.existing_rule` returns for a rule: a dict of its attributes
/// as Bazel lists them, with lists as tuples and labels written relative to
/// this package. It shows what the rule set and its class's defaults, and
/// nothing of `package()`'s defaults.
///
/// Bazel returns a live read-only view; this is a snapshot, a `dict`.
pub(crate) fn rule_view<'v>(
    ctx: &BuildContext<'_>,
    target: &fjfj_graph::package::Target,
    heap: Heap<'v>,
) -> Value<'v> {
    let fjfj_graph::package::TargetKind::Rule {
        rule_class, attrs, ..
    } = &target.kind
    else {
        return Value::new_none();
    };
    let mut entries: Vec<(&str, Value<'v>)> = vec![
        ("name", heap.alloc(target.name.as_str())),
        ("kind", heap.alloc(rule_class.as_str())),
    ];
    let schema = ctx.schemas.borrow().get(&target.name).cloned();
    for attr in schema.iter().flat_map(|s| &s.attrs) {
        if attr.name == "name" || attr.name == "kind" || !attr.shown() {
            continue;
        }
        let set = attrs.iter().find(|(k, _)| *k == attr.name).map(|(_, v)| v);
        let default;
        let value = match set {
            Some(v) => v,
            None => match attr.def.default_value() {
                Some(v) => {
                    default = v;
                    &default
                }
                None => continue,
            },
        };
        entries.push((&attr.name, attr_to_value(ctx, value, heap)));
    }
    crate::map_view::map_view(
        heap,
        heap.alloc(AllocDict(entries)),
        format!("<native.ExistingRuleView for target '{}'>", target.name),
    )
}

pub(crate) fn attr_to_value<'v>(
    ctx: &BuildContext<'_>,
    value: &AttrValue,
    heap: Heap<'v>,
) -> Value<'v> {
    let relative = |l: &Label| label_relative_to(l, ctx.repo, ctx.package);
    match value {
        AttrValue::Bool(b) => Value::new_bool(*b),
        AttrValue::String(s) => heap.alloc(s.as_str()),
        AttrValue::Label(l) => heap.alloc(relative(l)),
        AttrValue::StringList(items) => {
            heap.alloc(AllocTuple(items.iter().map(|s| heap.alloc(s.as_str()))))
        }
        AttrValue::LabelList(items) => {
            heap.alloc(AllocTuple(items.iter().map(|l| heap.alloc(relative(l)))))
        }
        AttrValue::Int(i) => heap.alloc(*i),
        AttrValue::IntList(items) => heap.alloc(AllocTuple(items.iter().map(|i| heap.alloc(*i)))),
        AttrValue::StringDict(entries) => heap.alloc(AllocDict(
            entries
                .iter()
                .map(|(k, v)| (heap.alloc(k.as_str()), heap.alloc(v.as_str()))),
        )),
        AttrValue::StringListDict(entries) => {
            heap.alloc(AllocDict(entries.iter().map(|(k, v)| {
                (
                    heap.alloc(k.as_str()),
                    heap.alloc(AllocTuple(v.iter().map(|s| heap.alloc(s.as_str())))),
                )
            })))
        }
        AttrValue::LabelKeyedStringDict(entries) => heap.alloc(AllocDict(
            entries
                .iter()
                .map(|(k, v)| (heap.alloc(relative(k)), heap.alloc(v.as_str()))),
        )),
        AttrValue::StringKeyedLabelDict(entries) => heap.alloc(AllocDict(
            entries
                .iter()
                .map(|(k, v)| (heap.alloc(k.as_str()), heap.alloc(relative(v)))),
        )),
        AttrValue::LabelListDict(entries) => heap.alloc(AllocDict(entries.iter().map(|(k, v)| {
            (
                heap.alloc(k.as_str()),
                heap.alloc(AllocTuple(v.iter().map(|l| heap.alloc(relative(l))))),
            )
        }))),
        // Each selector is a `select()` of its branches, labelled as the
        // conditions are, and a value that was written plain is one with
        // only the default.
        AttrValue::Select(list) => select::alloc(
            heap,
            list.elements
                .iter()
                .map(|selector| {
                    let branches = selector.branches.iter().map(|(label, value)| {
                        (
                            heap.alloc(StarlarkLabel::from(label.clone())),
                            value
                                .as_ref()
                                .map_or(Value::new_none(), |v| attr_to_value(ctx, v, heap)),
                        )
                    });
                    select::Element {
                        select: true,
                        value: heap.alloc(AllocDict(branches)),
                        no_match_error: selector.no_match_error.clone(),
                    }
                })
                .collect(),
            list.pipe,
        ),
    }
}
