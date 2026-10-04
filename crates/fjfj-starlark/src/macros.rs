//! Symbolic macros (buildfiji-mum.3.8): `macro(implementation = ...)`, and
//! instantiating one from a BUILD file, a legacy macro or another symbolic
//! macro.
//!
//! What Bazel 9.2.0 does, all of it read off probes:
//!
//! - **Declaring.** `macro(*, implementation, attrs, inherit_attrs,
//!   finalizer, doc)` is only callable while a `.bzl` initializes. `attrs` is
//!   a dict of attribute descriptors (or `None`, which removes an inherited
//!   one); `name` and `visibility` cannot be declared (they are the macro's),
//!   and a computed or late-bound default is refused. `inherit_attrs` is a
//!   rule, a macro, or `"common"`: the attributes of the rule (public ones,
//!   without `name` and the `generator_*` ones), of the macro, or the ones every
//!   rule has. `finalizer = True` makes a macro that runs when the BUILD file
//!   is done. A macro is named like a rule, by the top-level name it is bound
//!   to; one that has none cannot be instantiated.
//! - **Instantiating** takes keywords only, `name` first required and a string;
//!   an attribute the macro does not have, and a value of the wrong type, are
//!   *fatal* (where a rule's are events), and so is a `select()` in an
//!   attribute that is not configurable. A missing mandatory attribute and a
//!   value outside `values` are events (without a location). The
//!   implementation is then called with `name` and every attribute by name:
//!   `visibility` is a list of `Label`s that always holds the package of the
//!   call, and any other configurable attribute is a `select`, so a plain
//!   value `v` is `select({"//conditions:default": v})`; an attribute
//!   inherited and not given is `None`. The implementation returns `None`.
//! - **Names.** A macro's name is one of the package's targets: it may not be
//!   a target's, or another macro's, and a target may not take the name of a
//!   macro that did not make it. A macro that calls itself is an event.
//! - **Inside a macro** `glob()`, `package()`, and (unless the macro is a
//!   finalizer) `existing_rule()` and `existing_rules()` are errors.
//! - **Finalizers** run after the BUILD file, in the order they were
//!   instantiated, and then see every target; one may not be instantiated by
//!   a macro that is not a finalizer.

use crate::args::fatal;
use crate::attr::view as attribute_view;
use crate::decl::{
    P, bind_checked, is_bool, is_dict, is_function, is_identifier, is_string_or_none, p,
};
use crate::exports::{Kind, Named, name_at_assignment, next_id, resolve_name};
use crate::instantiate::{check_values, convert};
use crate::label::{StarlarkLabel, display_label, evaluating_bzl};
use crate::native::BuildContext;
use crate::select;
use allocative::Allocative;
use fjfj_graph::Label;
use fjfj_graph::rule::{AttrDef, AttrType, AttrValue, SelectorList};
use fjfj_graph::schema::{RuleSchema, SchemaAttr};
use starlark::collections::StarlarkHasher;
use starlark::environment::GlobalsBuilder;
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_complex_value;
use starlark::starlark_module;
use starlark::values::dict::{AllocDict, DictRef};
use starlark::values::list::AllocList;
use starlark::values::{
    Coerce, Freeze, FreezeResult, Freezer, FrozenValue, Heap, NoSerialize, ProvidesStaticType,
    StarlarkPagablePanic, StarlarkValue, Trace, Value, ValueLike,
};
use starlark_derive::starlark_value;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::hash::Hash;
use std::sync::{Arc, OnceLock};

// ---- the schema -------------------------------------------------------------------

/// One attribute of a macro.
#[derive(Debug, Clone)]
pub(crate) struct MacroAttr {
    pub(crate) attr: SchemaAttr,
    /// Came from `inherit_attrs`, and so is `None` when not given.
    pub(crate) inherited: bool,
}

/// What a macro takes.
#[derive(Debug)]
pub(crate) struct MacroSchema {
    /// `visibility` first, then the inherited and declared attributes.
    pub(crate) attrs: Vec<MacroAttr>,
    pub(crate) finalizer: bool,
}

/// A macro, before or after freezing.
#[derive(
    Debug, Trace, Coerce, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative,
)]
#[repr(C)]
pub(crate) struct MacroGen<V> {
    /// What a macro is, across freezing.
    #[trace(static)]
    id: u64,
    #[trace(static)]
    #[allocative(skip)]
    schema: Arc<MacroSchema>,
    implementation: V,
    /// The descriptors of the attributes it declares.
    #[allow(dead_code)]
    attrs: Vec<V>,
    #[trace(static)]
    #[allocative(skip)]
    #[allow(dead_code)]
    doc: Option<String>,
    #[trace(static)]
    #[allocative(skip)]
    name: OnceLock<String>,
}

starlark_complex_value!(pub(crate) Macro);

impl<'v> Freeze for Macro<'v> {
    type Frozen = FrozenMacro;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenMacro> {
        Ok(MacroGen {
            id: self.id,
            schema: self.schema,
            implementation: self.implementation.freeze(freezer)?,
            attrs: self
                .attrs
                .into_iter()
                .map(|v| v.freeze(freezer))
                .collect::<FreezeResult<Vec<FrozenValue>>>()?,
            doc: self.doc,
            name: self.name,
        })
    }
}

impl<V> fmt::Display for MacroGen<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name.get() {
            Some(name) => write!(f, "<macro {name}>"),
            None => write!(f, "<macro>"),
        }
    }
}

fn identity<'v>(value: Value<'v>) -> Option<(u64, &'v OnceLock<String>, Arc<MacroSchema>)> {
    fn of<'v, V: ValueLike<'v>>(
        m: &'v MacroGen<V>,
    ) -> (u64, &'v OnceLock<String>, Arc<MacroSchema>) {
        (m.id, &m.name, m.schema.clone())
    }
    if let Some(live) = value.downcast_ref::<Macro<'v>>() {
        Some(of(live))
    } else {
        value.downcast_ref::<FrozenMacro>().map(of)
    }
}

/// A macro, for [`crate::exports`] to name.
pub(crate) fn named<'v>(value: Value<'v>) -> Option<Named<'v>> {
    identity(value).map(|(id, name, _)| Named {
        id,
        name,
        kind: Kind::Macro,
    })
}

/// The schema of the macro `value`.
fn schema_of(value: Value<'_>) -> Option<Arc<MacroSchema>> {
    identity(value).map(|(_, _, schema)| schema)
}

#[starlark_value(type = "macro")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for MacroGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn invoke(
        &self,
        me: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        instantiate(
            self.id,
            &self.schema,
            self.implementation.to_value(),
            me,
            args,
            eval,
        )?;
        Ok(Value::new_none())
    }

    fn write_hash(&self, hasher: &mut StarlarkHasher) -> starlark::Result<()> {
        self.id.hash(hasher);
        Ok(())
    }

    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(identity(other).is_some_and(|(id, _, _)| id == self.id))
    }
}

// ---- macro() ----------------------------------------------------------------------

const MACRO_PARAMS: &[P] = &[
    p("implementation", false, true, "function", is_function),
    p("attrs", false, false, "dict", is_dict),
    p(
        "inherit_attrs",
        false,
        false,
        "rule, macro, string, or NoneType",
        inheritable,
    ),
    p("finalizer", false, false, "bool", is_bool),
    p("doc", false, false, "string or NoneType", is_string_or_none),
];

fn inheritable(v: Value<'_>) -> bool {
    v.is_none() || v.unpack_str().is_some() || matches!(v.get_type(), "rule" | "macro")
}

/// The attributes every macro has and `"common"` inherits: what a rule with
/// nothing of its own has, less what no macro has.
fn common_attrs() -> Vec<SchemaAttr> {
    let plain = RuleSchema::starlark(Vec::new(), false, false, Vec::new()).expect("no attributes");
    inherited_from(&plain)
}

fn inherited_from(schema: &RuleSchema) -> Vec<SchemaAttr> {
    schema
        .attrs
        .iter()
        .filter(|a| {
            a.settable()
                && !a.hidden
                && !matches!(
                    a.name.as_str(),
                    "name" | "generator_name" | "generator_function" | "generator_location"
                )
        })
        .cloned()
        .collect()
}

/// The string and int `values` of a descriptor, as a message lists them.
fn value_strings(ty: AttrType, values: &[Value<'_>]) -> Vec<String> {
    values
        .iter()
        .filter_map(|v| match ty {
            AttrType::String => v.unpack_str().map(str::to_owned),
            AttrType::Int => v.unpack_i32().map(|i| i.to_string()),
            _ => None,
        })
        .collect()
}

fn visibility_attr() -> SchemaAttr {
    let mut attr = SchemaAttr::new("visibility", AttrDef::new(AttrType::LabelList));
    attr.configurable = false;
    attr
}

fn make_macro<'v>(
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Value<'v>> {
    if !evaluating_bzl(eval) {
        return Err(fatal(
            "macro() can only be used during .bzl initialization (top-level evaluation)",
        ));
    }
    let bound = bind_checked("macro", MACRO_PARAMS, args, eval)?;
    let heap = eval.heap();
    let arg = |name: &str| {
        let i = MACRO_PARAMS
            .iter()
            .position(|p| p.name == name)
            .expect("known");
        bound[i].filter(|v| !v.is_none())
    };
    let implementation = arg("implementation").expect("required");

    // What is inherited.
    let mut attrs: Vec<MacroAttr> = vec![MacroAttr {
        attr: visibility_attr(),
        inherited: false,
    }];
    let from = |list: Vec<SchemaAttr>| {
        list.into_iter()
            .map(|attr| MacroAttr {
                attr,
                inherited: true,
            })
            .collect::<Vec<_>>()
    };
    let mut inherited: Vec<MacroAttr> = Vec::new();
    if let Some(parent) = arg("inherit_attrs") {
        if let Some(text) = parent.unpack_str() {
            if text != "common" {
                return Err(fatal(format!(
                    "Invalid 'inherit_attrs' value {}; expected a rule, a macro, or \"common\"",
                    parent.to_repr()
                )));
            }
            inherited = from(common_attrs());
        } else {
            if resolve_name(parent, eval)?.is_none() {
                return Err(fatal(
                    "Invalid 'inherit_attrs' value: a rule or macro callable must be assigned to \
                     a global variable in a .bzl file before it can be inherited from",
                ));
            }
            if let Some(schema) = crate::rule::schema_of(parent) {
                inherited = from(inherited_from(&schema));
            } else if let Some(schema) = schema_of(parent) {
                inherited = schema
                    .attrs
                    .iter()
                    .filter(|a| a.attr.name != "visibility")
                    .map(|a| MacroAttr {
                        attr: a.attr.clone(),
                        inherited: true,
                    })
                    .collect();
            }
        }
    }
    // `visibility` comes first, and the rest of what is inherited after it.
    attrs.extend(
        inherited
            .into_iter()
            .filter(|a| a.attr.name != "visibility"),
    );

    // What is declared.
    let mut descriptors: Vec<Value<'v>> = Vec::new();
    if let Some(dict) = arg("attrs").and_then(DictRef::from_value) {
        if let Some((k, v)) = dict.iter().find(|(k, v)| {
            k.unpack_str().is_none() || !(v.is_none() || attribute_view(*v).is_some())
        }) {
            return Err(fatal(format!(
                "got dict<{}, {}> for 'attrs', want dict<string, Attribute|None>",
                k.get_type(),
                v.get_type()
            )));
        }
        for (name, descriptor) in dict.iter() {
            let name = name.unpack_str().expect("checked");
            if !is_identifier(name) {
                return Err(fatal(format!(
                    "attribute name `{name}` is not a valid identifier."
                )));
            }
            if name == "name" || name == "visibility" {
                return Err(fatal(format!(
                    "Cannot declare a macro attribute named '{name}'"
                )));
            }
            if descriptor.is_none() {
                attrs.retain(|a| a.attr.name != name);
                continue;
            }
            let view = attribute_view(descriptor).expect("checked");
            if view.def.computed_default || view.computed.is_some() {
                return Err(fatal(format!(
                    "In macro attribute '{name}': Macros do not support computed defaults or \
                     late-bound defaults"
                )));
            }
            let own = MacroAttr {
                attr: SchemaAttr {
                    info: Default::default(),
                    name: name.to_owned(),
                    def: view.def.clone(),
                    values: value_strings(view.def.ty, &view.values),
                    hidden: false,
                    configurable: view.def.configurable.unwrap_or(!matches!(
                        view.def.ty,
                        AttrType::Output | AttrType::OutputList
                    )),
                    set: false,
                },
                inherited: false,
            };
            match attrs.iter_mut().find(|a| a.attr.name == name) {
                Some(at) => *at = own,
                None => attrs.push(own),
            }
            descriptors.push(descriptor);
        }
    }

    let finalizer = arg("finalizer")
        .and_then(|v| v.unpack_bool())
        .unwrap_or(false);
    let doc = arg("doc").and_then(|d| d.unpack_str()).map(str::to_owned);
    let name = OnceLock::new();
    name_at_assignment(eval, Kind::Macro, &name)?;
    Ok(heap.alloc_complex(MacroGen {
        id: next_id(),
        schema: Arc::new(MacroSchema { attrs, finalizer }),
        implementation,
        attrs: descriptors,
        doc,
        name,
    }))
}

// ---- state of one BUILD file --------------------------------------------------------

/// A macro that is running.
#[derive(Debug, Clone)]
pub(crate) struct MacroFrame {
    /// The macro (class) it is an instance of.
    pub(crate) class: u64,
    /// This instantiation.
    pub(crate) instance: u64,
    pub(crate) name: String,
    pub(crate) finalizer: bool,
}

/// What a value passed to an implementation is, before there is a heap to
/// make it in.
#[derive(Debug, Clone)]
pub(crate) enum Passed {
    /// `None`: an inherited attribute that was not given, or a label with no
    /// value.
    Nothing,
    /// A plain value, in a `select` of only the default when the attribute
    /// is configurable.
    Value {
        value: AttrValue,
        configurable: bool,
    },
    /// Branches: a `select()` or a list of them.
    Select(SelectorList),
    /// The labels of `visibility`.
    Labels(Vec<Label>),
}

/// A finalizer instantiated, to run when the BUILD file is done.
pub(crate) struct PendingFinalizer {
    pub(crate) macro_value: FrozenValue,
    pub(crate) class: u64,
    pub(crate) name: String,
    pub(crate) values: Vec<(String, Passed)>,
}

/// What a BUILD file's macros share.
#[derive(Default)]
pub(crate) struct MacroState {
    pub(crate) stack: Vec<MacroFrame>,
    /// Instance ids by the name each was given: macro names are target
    /// names.
    pub(crate) instances: HashMap<String, u64>,
    pub(crate) pending: VecDeque<PendingFinalizer>,
    /// The rules there were when the finalizers began, which is all
    /// `existing_rules()` shows them.
    pub(crate) visible_to_finalizers: Option<Vec<String>>,
    next_instance: u64,
}

impl MacroState {
    fn fresh(&mut self) -> u64 {
        self.next_instance += 1;
        self.next_instance
    }

    /// Whether code is running in a symbolic macro (a finalizer included).
    pub(crate) fn inside(&self) -> bool {
        !self.stack.is_empty()
    }

    /// Whether the macro running now is a finalizer.
    pub(crate) fn in_finalizer(&self) -> bool {
        self.stack.last().is_some_and(|f| f.finalizer)
    }
}

// ---- instantiating -------------------------------------------------------------------

/// The visibility a macro gets: what was given and the package that made the
/// call, sorted; `//visibility:public` alone if it was given.
fn visibility_labels(ctx: &BuildContext<'_>, given: Vec<Label>) -> starlark::Result<Vec<Label>> {
    let visibility = |name: &str| Label {
        repo: String::new(),
        package: "visibility".to_owned(),
        name: name.to_owned(),
    };
    for label in &given {
        if label.package == "visibility"
            && label.repo.is_empty()
            && label.name != "public"
            && label.name != "private"
        {
            return Err(fatal(format!(
                "Invalid visibility label '{}'; did you mean //visibility:public or \
                 //visibility:private?",
                display_label(label)
            )));
        }
    }
    if given.contains(&visibility("public")) {
        return Ok(vec![visibility("public")]);
    }
    let own = Label {
        repo: ctx.repo.to_owned(),
        package: ctx.package.to_owned(),
        name: "__pkg__".to_owned(),
    };
    let mut labels: Vec<Label> = given
        .into_iter()
        .filter(|l| *l != visibility("private"))
        .collect();
    if !labels.contains(&own) {
        labels.push(own);
    }
    labels.sort_by_key(display_label);
    Ok(labels)
}

/// The value of `AttrValue` as a Starlark value of the form a macro sees:
/// labels are `Label`s and lists are lists.
fn value_of<'v>(value: &AttrValue, heap: Heap<'v>) -> Value<'v> {
    let label = |l: &Label| heap.alloc(StarlarkLabel::from(l.clone()));
    let list = |items: Vec<Value<'v>>| heap.alloc(AllocList(items));
    match value {
        AttrValue::Bool(b) => Value::new_bool(*b),
        AttrValue::Int(i) => heap.alloc(*i),
        AttrValue::String(s) => heap.alloc(s.as_str()),
        AttrValue::Label(l) => label(l),
        AttrValue::StringList(items) => {
            list(items.iter().map(|s| heap.alloc(s.as_str())).collect())
        }
        AttrValue::IntList(items) => list(items.iter().map(|i| heap.alloc(*i)).collect()),
        AttrValue::LabelList(items) => list(items.iter().map(label).collect()),
        AttrValue::StringDict(entries) => heap.alloc(AllocDict(
            entries
                .iter()
                .map(|(k, v)| (heap.alloc(k.as_str()), heap.alloc(v.as_str()))),
        )),
        AttrValue::StringListDict(entries) => {
            heap.alloc(AllocDict(entries.iter().map(|(k, v)| {
                (
                    heap.alloc(k.as_str()),
                    list(v.iter().map(|s| heap.alloc(s.as_str())).collect()),
                )
            })))
        }
        AttrValue::LabelKeyedStringDict(entries) => heap.alloc(AllocDict(
            entries
                .iter()
                .map(|(k, v)| (label(k), heap.alloc(v.as_str()))),
        )),
        AttrValue::StringKeyedLabelDict(entries) => heap.alloc(AllocDict(
            entries
                .iter()
                .map(|(k, v)| (heap.alloc(k.as_str()), label(v))),
        )),
        AttrValue::LabelListDict(entries) => {
            heap.alloc(AllocDict(entries.iter().map(|(k, v)| {
                (heap.alloc(k.as_str()), list(v.iter().map(label).collect()))
            })))
        }
        AttrValue::Select(selectors) => select_value(selectors, heap),
    }
}

fn select_value<'v>(list: &SelectorList, heap: Heap<'v>) -> Value<'v> {
    select::alloc(
        heap,
        list.elements
            .iter()
            .map(|selector| {
                let branches = selector.branches.iter().map(|(label, value)| {
                    (
                        heap.alloc(StarlarkLabel::from(label.clone())),
                        value
                            .as_ref()
                            .map_or(Value::new_none(), |v| value_of(v, heap)),
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
    )
}

fn passed_value<'v>(passed: &Passed, heap: Heap<'v>) -> Value<'v> {
    match passed {
        Passed::Nothing => Value::new_none(),
        Passed::Labels(labels) => heap.alloc(AllocList(
            labels
                .iter()
                .map(|l| heap.alloc(StarlarkLabel::from(l.clone()))),
        )),
        Passed::Select(list) => select_value(list, heap),
        Passed::Value {
            value,
            configurable: false,
        } => value_of(value, heap),
        Passed::Value {
            value,
            configurable: true,
        } => {
            // A plain value stands in a `select` of only the default, whose
            // key is the string.
            let plain = value_of(value, heap);
            let key = heap.alloc(default_condition_text());
            select::alloc(
                heap,
                vec![select::Element {
                    select: true,
                    value: heap.alloc(AllocDict([(key, plain)])),
                    no_match_error: String::new(),
                }],
                false,
            )
        }
    }
}

fn default_condition_text() -> &'static str {
    "//conditions:default"
}

/// Call `implementation` with the arguments of one instantiation.
fn run<'v>(
    ctx: &BuildContext<'_>,
    frame: MacroFrame,
    implementation: Value<'v>,
    values: &[(String, Passed)],
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<()> {
    let heap = eval.heap();
    let name = frame.name.clone();
    let mut named: Vec<(String, Value<'v>)> = vec![("name".to_owned(), heap.alloc(name.as_str()))];
    for (key, passed) in values {
        named.push((key.clone(), passed_value(passed, heap)));
    }
    let named: Vec<(&str, Value<'v>)> = named.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    ctx.macros.borrow_mut().stack.push(frame);
    let result = eval.eval_function(implementation, &[], &named);
    ctx.macros.borrow_mut().stack.pop();
    let result = result?;
    if !result.is_none() {
        return Err(fatal(format!(
            "macro '{name}' may not return a non-None value (got {})",
            result.to_repr()
        )));
    }
    Ok(())
}

fn instantiate<'v>(
    class: u64,
    schema: &Arc<MacroSchema>,
    implementation: Value<'v>,
    me: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<()> {
    let Some(macro_name) = resolve_name(me, eval)? else {
        return Err(fatal(
            "Cannot instantiate a macro that has not been exported (assign it to a global \
             variable in the .bzl where it's defined)",
        ));
    };
    let ctx = match eval
        .extra
        .and_then(|extra| extra.downcast_ref::<BuildContext>())
    {
        Some(ctx) if !evaluating_bzl(eval) => ctx,
        _ => {
            return Err(fatal(
                "a symbolic macro can only be instantiated while evaluating a BUILD file or a \
                 legacy or symbolic macro",
            ));
        }
    };
    if schema.finalizer && ctx.macros.borrow().inside() && !ctx.macros.borrow().in_finalizer() {
        return Err(fatal(
            "Cannot instantiate a rule finalizer within a non-finalizer symbolic macro. Rule \
             finalizers may only be instantiated while evaluating a BUILD file, a legacy macro \
             called from a BUILD file, or another rule finalizer.",
        ));
    }
    let heap = eval.heap();
    if args.positions(heap)?.next().is_some() {
        return Err(fatal("unexpected positional arguments"));
    }
    let named = args.names_map()?;
    let Some(name_value) = named
        .iter()
        .find(|(k, _)| k.as_str() == "name")
        .map(|(_, v)| *v)
    else {
        return Err(fatal(format!(
            "missing value for mandatory attribute 'name' in '{macro_name}' macro"
        )));
    };
    let Some(name) = name_value.unpack_str().map(str::to_owned) else {
        return Err(fatal(format!(
            "expected value of type 'string' for attribute 'name' of '{macro_name}', but got {}",
            crate::args::describe(name_value)
        )));
    };
    if name.is_empty() {
        return Err(fatal("invalid target name '': empty target name"));
    }

    // Each argument, as the attribute it is.
    let mut given: HashMap<&str, Option<AttrValue>> = HashMap::new();
    for (key, value) in named.iter() {
        let key = key.as_str();
        if key == "name" {
            continue;
        }
        let Some(entry) = schema
            .attrs
            .iter()
            .find(|a| a.attr.name == key && a.attr.settable())
        else {
            let hint = fjfj_graph::rule::suggest(
                key,
                schema
                    .attrs
                    .iter()
                    .filter(|a| a.attr.settable())
                    .map(|a| a.attr.name.as_str()),
            )
            .map(|s| format!(" (did you mean '{s}'?)"))
            .unwrap_or_default();
            return Err(fatal(format!(
                "no such attribute '{key}' in '{macro_name}' macro{hint}"
            )));
        };
        let converted = convert(ctx, &macro_name, &entry.attr, *value, heap).map_err(fatal)?;
        given.insert(key, converted);
    }

    // What the implementation is given, and what is wrong that is not fatal.
    let me_label = Label {
        repo: ctx.repo.to_owned(),
        package: ctx.package.to_owned(),
        name: name.clone(),
    };
    let mut values: Vec<(String, Passed)> = Vec::new();
    let mut events: Vec<String> = Vec::new();
    for entry in &schema.attrs {
        let attr = &entry.attr;
        let value = given.get(attr.name.as_str()).cloned().flatten();
        if attr.name == "visibility" {
            let labels = match value {
                Some(AttrValue::LabelList(labels)) => labels,
                _ => Vec::new(),
            };
            values.push((
                attr.name.clone(),
                Passed::Labels(visibility_labels(ctx, labels)?),
            ));
            continue;
        }
        if attr.def.mandatory() && value.is_none() {
            events.push(format!(
                "{me_label}: missing value for mandatory attribute '{}' in '{macro_name}' macro",
                attr.name
            ));
        }
        let passed = match value {
            Some(AttrValue::Select(list)) => Passed::Select(list),
            Some(value) => {
                for message in check_values(attr, &value) {
                    events.push(format!("{me_label}: {message}"));
                }
                Passed::Value {
                    value,
                    configurable: attr.configurable,
                }
            }
            // A mandatory attribute that was left out is passed its default.
            None if entry.inherited && !attr.def.mandatory() => Passed::Nothing,
            None => match attr.def.default_value() {
                Some(value) => Passed::Value {
                    value,
                    configurable: attr.configurable,
                },
                None => Passed::Nothing,
            },
        };
        values.push((attr.name.clone(), passed));
    }

    // Names, and recursion.
    {
        let state = ctx.macros.borrow();
        if let Some(outer) = state.stack.iter().find(|f| f.class == class) {
            let direct = state.stack.last().is_some_and(|f| f.class == class);
            let text = format!(
                "macro '{name}' is {} recursive call of '{}'. Macro instantiation traceback \
                 (most recent call last):",
                if direct { "a direct" } else { "an indirect" },
                outer.name
            );
            drop(state);
            ctx.event_plain(text);
            return Ok(());
        }
        if state.instances.contains_key(&name) {
            return Err(fatal(format!(
                "macro '{name}' conflicts with an existing macro (and was not created by it)"
            )));
        }
    }
    if ctx.state.borrow().builder.has_target(&name) {
        return Err(fatal(format!(
            "macro '{name}' conflicts with an existing target."
        )));
    }
    for event in events {
        ctx.event_plain(event);
    }
    let instance = {
        let mut state = ctx.macros.borrow_mut();
        let instance = state.fresh();
        state.instances.insert(name.clone(), instance);
        instance
    };
    let frame = MacroFrame {
        class,
        instance,
        name: name.clone(),
        finalizer: schema.finalizer,
    };
    if schema.finalizer {
        let Some(macro_value) = me.unpack_frozen() else {
            return Err(fatal("a macro is frozen before it is instantiated"));
        };
        ctx.macros.borrow_mut().pending.push_back(PendingFinalizer {
            macro_value,
            class,
            name,
            values,
        });
        return Ok(());
    }
    run(ctx, frame, implementation, &values, eval)
}

/// Run the finalizers the BUILD file instantiated, now that it is done; one
/// may instantiate another, which runs after.
pub(crate) fn run_finalizers<'v>(
    ctx: &BuildContext<'_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<()> {
    if ctx.macros.borrow().pending.is_empty() {
        return Ok(());
    }
    let rules: Vec<String> = ctx
        .state
        .borrow()
        .builder
        .rules()
        .map(|t| t.name.clone())
        .collect();
    ctx.macros.borrow_mut().visible_to_finalizers = Some(rules);
    loop {
        let Some(pending) = ctx.macros.borrow_mut().pending.pop_front() else {
            return Ok(());
        };
        let macro_value = pending.macro_value.to_value();
        let Some((_, _, _)) = identity(macro_value) else {
            continue;
        };
        let implementation = implementation_of(macro_value).expect("a macro");
        let instance = ctx
            .macros
            .borrow()
            .instances
            .get(&pending.name)
            .copied()
            .unwrap_or(0);
        let frame = MacroFrame {
            class: pending.class,
            instance,
            name: pending.name.clone(),
            finalizer: true,
        };
        run(ctx, frame, implementation, &pending.values, eval)?;
    }
}

fn implementation_of<'v>(value: Value<'v>) -> Option<Value<'v>> {
    if let Some(live) = value.downcast_ref::<Macro<'v>>() {
        Some(live.implementation)
    } else {
        value
            .downcast_ref::<FrozenMacro>()
            .map(|m| m.implementation.to_value())
    }
}

#[starlark_module]
pub(crate) fn macro_globals(builder: &mut GlobalsBuilder) {
    /// `macro(*, implementation, attrs, inherit_attrs, finalizer, doc)`.
    fn r#macro<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        make_macro(args, eval)
    }
}
