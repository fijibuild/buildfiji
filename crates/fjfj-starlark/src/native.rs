//! `native.*` and the rules a BUILD file calls directly: `glob`, `package`,
//! `package_group`, `exports_files`, `existing_rule(s)`, `filegroup`,
//! `alias`, `package_name`, `repository_name` and `repo_name`
//! (buildfiji-mum.4).
//!
//! Where Bazel puts them:
//!
//! - a BUILD file sees them as globals and has no `native`;
//! - a `.bzl` sees them only as `native.x`, and calling one while the `.bzl`
//!   loads, rather than from a macro a BUILD file calls, is an error.
//!
//! What they record is a [`fjfj_graph::package::Package`], built through
//! [`PackageBuilder`] so its conflict, subpackage and visibility rules apply.
//!
//! Every message below was checked against Bazel 9.2.0. Two kinds of error
//! exist and are kept apart: a *fatal* one stops the file (`Error in glob:
//! ...`), and an *event* is reported, recorded, and the file goes on, so one
//! run can list several. Events fail the package at the end.

use fjfj_graph::package::{Package, PackageBuilder, PackageSettings, Target, TargetKind};
use fjfj_graph::rule::{
    self, AttrSpec, AttrType, AttrValue, RuleClass, label_relative_to, native_rule,
};
use fjfj_graph::visibility::{PackageGroup, PackageSpec, Visibility};
use fjfj_graph::{Label, LabelContext};
use fjfj_loading::{GlobOptions, PackageLookup};
use starlark::collections::SmallMap;
use starlark::environment::{Globals, GlobalsBuilder, LibraryExtension, Module};
use starlark::eval::{Arguments, Evaluator, FileLoader};
use starlark::starlark_module;
use starlark::values::dict::AllocDict;
use starlark::values::none::NoneType;
use starlark::values::tuple::AllocTuple;
use starlark::values::{ProvidesStaticType, StringValue, Value};
use std::cell::RefCell;

use crate::args::{Wording, bind, describe, fatal, param, sequence, want_sequence};
use crate::depset::depset_globals;
use crate::{FileKind, parse};

/// Everything one BUILD file evaluation hands back.
#[derive(Debug)]
pub struct BuildFileOutput {
    pub package: Package,
    /// What `print()` wrote, one string per call.
    pub printed: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum BuildFileError {
    /// The file did not parse, or a fatal error stopped it.
    #[error("{0:#}")]
    Eval(anyhow::Error),
    /// The file ran to the end but reported errors.
    #[error("package contains errors:\n{}", .0.join("\n"))]
    Package(Vec<String>),
}

/// What evaluating a BUILD file needs.
pub struct BuildFile<'a> {
    /// Canonical repo name; empty for the main repository.
    pub repo: &'a str,
    pub package: &'a str,
    pub lookup: &'a PackageLookup,
    /// How locations and the parser name this file.
    pub path: &'a str,
    pub source: &'a str,
    /// Resolves the file's `load()`s. Each `.bzl` it evaluates should use
    /// [`bzl_globals`].
    pub loader: &'a dyn FileLoader,
}

/// The globals of a BUILD file: the standard ones, `print`, and the native
/// functions.
pub fn build_globals() -> Globals {
    GlobalsBuilder::extended_by(&[LibraryExtension::Print])
        .with(native_functions)
        .with(depset_globals)
        .build()
}

/// The globals of a `.bzl` file: the same, with the native functions under
/// `native`.
pub fn bzl_globals() -> Globals {
    let mut builder = GlobalsBuilder::extended_by(&[LibraryExtension::Print]);
    builder.namespace("native", native_functions);
    builder.with(depset_globals).build()
}

/// Evaluate a BUILD file into the package it declares.
pub fn evaluate_build_file(input: &BuildFile<'_>) -> Result<BuildFileOutput, BuildFileError> {
    let _span = tracing::debug_span!("evaluate_build_file", package = input.package).entered();
    let ast = parse(input.path, input.source, FileKind::Build).map_err(BuildFileError::Eval)?;
    let is_package = |p: &str| input.lookup.is_package(p);
    let ctx = BuildContext {
        repo: input.repo,
        package: input.package,
        lookup: input.lookup,
        state: RefCell::new(BuildState {
            builder: PackageBuilder::new(input.repo, input.package, &is_package),
            errors: Vec::new(),
        }),
        printed: RefCell::new(Vec::new()),
    };
    let globals = build_globals();
    Module::with_temp_heap(|module| {
        let mut eval = Evaluator::new(&module);
        eval.extra = Some(&ctx);
        eval.set_loader(input.loader);
        eval.set_print_handler(&ctx);
        eval.eval_module(ast, &globals)
            .map(|_| ())
            .map_err(|e| BuildFileError::Eval(e.into_anyhow()))
    })?;
    let BuildContext { state, printed, .. } = ctx;
    let state = state.into_inner();
    if !state.errors.is_empty() {
        return Err(BuildFileError::Package(state.errors));
    }
    Ok(BuildFileOutput {
        package: state.builder.build(),
        printed: printed.into_inner(),
    })
}

/// The state a BUILD file's native calls accumulate into. `Evaluator::extra`
/// hands them a shared reference, hence the `RefCell`; nothing escapes it
/// but the finished [`Package`].
#[derive(ProvidesStaticType)]
struct BuildContext<'a> {
    repo: &'a str,
    package: &'a str,
    lookup: &'a PackageLookup,
    state: RefCell<BuildState<'a>>,
    printed: RefCell<Vec<String>>,
}

struct BuildState<'a> {
    builder: PackageBuilder<'a>,
    /// Events, as `location: message`.
    errors: Vec<String>,
}

impl starlark::PrintHandler for BuildContext<'_> {
    fn println(&self, text: &str) -> starlark::Result<()> {
        self.printed.borrow_mut().push(text.to_owned());
        Ok(())
    }
}

impl BuildContext<'_> {
    fn label_context(&self) -> LabelContext<'_> {
        LabelContext {
            repo: self.repo,
            package: self.package,
        }
    }

    fn event(&self, location: &str, message: impl AsRef<str>) {
        self.state
            .borrow_mut()
            .errors
            .push(format!("{location}: {}", message.as_ref()));
    }
}

/// The context of the BUILD file being evaluated, or the error a native
/// function gives when there is none: a `.bzl` loading, not a macro called
/// from a BUILD file.
fn context<'a, 'e>(
    eval: &Evaluator<'_, 'a, 'e>,
    function: &str,
) -> starlark::Result<&'a BuildContext<'e>> {
    eval.extra
        .and_then(|extra| extra.downcast_ref::<BuildContext>())
        .ok_or_else(|| {
            fatal(format!(
                "{function}() can only be used while evaluating a BUILD file or a legacy macro"
            ))
        })
}

/// `file:line:col` of the call being made, at its `(` the way Bazel reports
/// it.
fn location(eval: &Evaluator<'_, '_, '_>) -> String {
    let Some(span) = eval.call_stack_top_location() else {
        return "<unknown>".to_owned();
    };
    let resolved = span.resolve();
    let begin = resolved.span.begin;
    // The span covers the whole call; Bazel points at its `(`. It is on the
    // first line for every callee that is a name or a dotted name.
    let text = span.source_span();
    let column = text
        .split('\n')
        .next()
        .and_then(|line| line.find('('))
        .map_or(0, |bytes| text[..bytes].chars().count());
    format!(
        "{}:{}:{}",
        resolved.file,
        begin.line + 1,
        begin.column + column + 1
    )
}

// ---- argument handling ----------------------------------------------------

/// The strings of a sequence, each element checked. `noun` is how Bazel
/// names the argument in the message: `'glob' argument`.
fn strings(items: &[Value<'_>], noun: &str) -> starlark::Result<Vec<String>> {
    items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            item.unpack_str().map(str::to_owned).ok_or_else(|| {
                fatal(format!(
                    "expected value of type 'string' for element {i} of {noun}, but got {}",
                    describe(*item)
                ))
            })
        })
        .collect()
}

/// Labels written in the package being loaded, each element checked.
fn labels(ctx: &BuildContext<'_>, items: &[Value<'_>], noun: &str) -> starlark::Result<Vec<Label>> {
    strings(items, noun)?
        .iter()
        .enumerate()
        .map(|(i, s)| {
            Label::parse(s, ctx.label_context())
                .map_err(|e| fatal(format!("invalid label '{s}' in element {i} of {noun}: {e}")))
        })
        .collect()
}

fn visibility_of(ctx: &BuildContext<'_>, labels: &[Label]) -> Visibility {
    let strings: Vec<String> = labels.iter().map(ToString::to_string).collect();
    Visibility::parse(strings.iter().map(String::as_str), ctx.label_context())
        .unwrap_or_else(|_| Visibility::private())
}

// ---- the native functions ---------------------------------------------------

#[starlark_module]
fn native_functions(builder: &mut GlobalsBuilder) {
    fn glob<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let ctx = context(eval, "glob")?;
        let bound = bind(
            "glob",
            Wording::Signature,
            &[
                param("include", true, false),
                param("exclude", true, false),
                param("exclude_directories", true, false),
                param("allow_empty", true, false),
            ],
            args,
            eval,
        )?;
        let noun = "'glob' argument";
        let patterns = |slot: usize, name: &str| -> starlark::Result<Vec<String>> {
            match bound[slot] {
                Some(v) => strings(
                    &want_sequence("glob", name, v, false)?.unwrap_or_default(),
                    noun,
                ),
                None => Ok(Vec::new()),
            }
        };
        let include = patterns(0, "include")?;
        let exclude = patterns(1, "exclude")?;
        let exclude_directories = match bound[2] {
            Some(v) => match v.unpack_i32() {
                Some(n) => n != 0,
                None => {
                    return Err(fatal(format!(
                        "in call to glob(), parameter 'exclude_directories' got value of type \
                         '{}', want 'int'",
                        v.get_type()
                    )));
                }
            },
            None => true,
        };
        let allow_empty = match bound[3] {
            Some(v) => v.unpack_bool().ok_or_else(|| {
                fatal(format!(
                    "expected boolean for argument `allow_empty`, got `{}`",
                    v.to_str()
                ))
            })?,
            None => false,
        };
        let found = fjfj_loading::glob(
            ctx.lookup,
            ctx.package,
            &include,
            &exclude,
            GlobOptions {
                exclude_directories,
                allow_empty,
            },
        )
        .map_err(|e| fatal(e.to_string()))?;
        Ok(eval.heap().alloc(found))
    }

    fn package<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let ctx = context(eval, "package")?;
        let bound = bind(
            "package",
            Wording::Package,
            &[
                param("default_visibility", false, false),
                param("default_testonly", false, false),
                param("default_deprecation", false, false),
                param("default_compatible_with", false, false),
                param("default_restricted_to", false, false),
                param("default_hdrs_check", false, false),
                param("licenses", false, false),
                param("default_applicable_licenses", false, false),
                param("default_package_metadata", false, false),
                param("features", false, false),
            ],
            args,
            eval,
        )?;
        let noun = |name: &str| format!("package() argument '{name}'");
        let label_list = |slot: usize, name: &str| -> starlark::Result<Option<Vec<Label>>> {
            let Some(v) = bound[slot] else {
                return Ok(None);
            };
            let noun = noun(name);
            let items = package_list(v, "list(label)", &noun)?;
            labels(ctx, &items, &noun).map(Some)
        };
        let string_list = |slot: usize, name: &str| -> starlark::Result<Vec<String>> {
            let Some(v) = bound[slot] else {
                return Ok(Vec::new());
            };
            let noun = noun(name);
            strings(&package_list(v, "list(string)", &noun)?, &noun)
        };
        let string = |slot: usize, name: &str| -> starlark::Result<Option<String>> {
            bound[slot]
                .map(|v| package_string(v, &noun(name)))
                .transpose()
        };

        let mut settings = PackageSettings {
            default_visibility: label_list(0, "default_visibility")?
                .map(|parsed| visibility_of(ctx, &parsed)),
            ..PackageSettings::default()
        };
        if let Some(v) = bound[1] {
            settings.defaults.testonly = match (v.unpack_bool(), v.unpack_i32()) {
                (Some(b), _) => b,
                (None, Some(0)) => false,
                (None, Some(1)) => true,
                _ => {
                    return Err(fatal(format!(
                        "expected one of [False, True, 0, 1] for {}, but got {}",
                        noun("default_testonly"),
                        describe(v)
                    )));
                }
            };
        }
        settings.defaults.deprecation = string(2, "default_deprecation")?;
        settings.defaults.compatible_with =
            label_list(3, "default_compatible_with")?.unwrap_or_default();
        settings.defaults.restricted_to =
            label_list(4, "default_restricted_to")?.unwrap_or_default();
        settings.defaults.hdrs_check = string(5, "default_hdrs_check")?;
        settings.defaults.licenses = string_list(6, "licenses")?;
        let licenses = label_list(7, "default_applicable_licenses")?;
        let metadata = label_list(8, "default_package_metadata")?;
        if licenses.is_some() && metadata.is_some() {
            return Err(fatal(
                "Can not set both default_package_metadata and default_applicable_licenses. \
                 Move all declarations to default_package_metadata.",
            ));
        }
        settings.defaults.package_metadata = metadata.or(licenses).unwrap_or_default();
        settings.defaults.features = string_list(9, "features")?;
        ctx.state
            .borrow_mut()
            .builder
            .call_package(settings)
            .map_err(|e| fatal(e.to_string()))?;
        Ok(NoneType)
    }

    fn package_group<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let ctx = context(eval, "package_group")?;
        let at = location(eval);
        let bound = bind(
            "package_group",
            Wording::Group,
            &[
                param("name", false, true),
                param("packages", false, false),
                param("includes", false, false),
            ],
            args,
            eval,
        )?;
        let name_value = bound[0].expect("required");
        let name = name_value.unpack_str().ok_or_else(|| {
            fatal(format!(
                "in call to package_group(), parameter 'name' got value of type '{}', want \
                 'string'",
                name_value.get_type()
            ))
        })?;
        let mut group = PackageGroup::default();
        if let Some(v) = bound[1] {
            let items = want_sequence("package_group", "packages", v, false)?.unwrap_or_default();
            for spec in strings(&items, "'package_group.packages argument'")? {
                match PackageSpec::parse(&spec, ctx.repo) {
                    Ok(Some(spec)) => group.specs.push(spec),
                    Ok(None) => {}
                    Err(e) => ctx.event(&at, e.to_string()),
                }
            }
        }
        if let Some(v) = bound[2] {
            let items = want_sequence("package_group", "includes", v, false)?.unwrap_or_default();
            group.includes = labels(ctx, &items, "'package_group.includes argument'")?;
        }
        ctx.state
            .borrow_mut()
            .builder
            .add_package_group(name, group, &at)
            .map_err(|e| fatal(e.to_string()))?;
        Ok(NoneType)
    }

    fn exports_files<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let ctx = context(eval, "exports_files")?;
        let at = location(eval);
        let bound = bind(
            "exports_files",
            Wording::Signature,
            &[
                param("srcs", true, true),
                param("visibility", true, false),
                param("licenses", true, false),
            ],
            args,
            eval,
        )?;
        let noun = "'exports_files' operand";
        let srcs = want_sequence("exports_files", "srcs", bound[0].expect("required"), false)?
            .unwrap_or_default();
        let srcs = strings(&srcs, noun)?;
        let visibility = match bound[1] {
            Some(v) => match want_sequence("exports_files", "visibility", v, true)? {
                Some(items) => Some(visibility_of(ctx, &labels(ctx, &items, noun)?)),
                None => None,
            },
            None => None,
        };
        if let Some(v) = bound[2] {
            want_sequence("exports_files", "licenses", v, true)?;
        }
        for src in srcs {
            ctx.state
                .borrow_mut()
                .builder
                .export_file(&src, visibility.clone(), &at)
                .map_err(|e| fatal(e.to_string()))?;
        }
        Ok(NoneType)
    }

    fn filegroup<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        call_native_rule(&rule::FILEGROUP, args, eval)
    }

    fn alias<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        call_native_rule(&rule::ALIAS, args, eval)
    }

    fn existing_rule<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let ctx = context(eval, "existing_rule")?;
        let bound = bind(
            "existing_rule",
            Wording::Signature,
            &[param("name", true, true)],
            args,
            eval,
        )?;
        let value = bound[0].expect("required");
        let name = value.unpack_str().ok_or_else(|| {
            fatal(format!(
                "in call to existing_rule(), parameter 'name' got value of type '{}', want \
                 'string'",
                value.get_type()
            ))
        })?;
        let state = ctx.state.borrow();
        Ok(match state.builder.rule(name) {
            Some(target) => rule_view(ctx, target, eval),
            None => Value::new_none(),
        })
    }

    fn existing_rules<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let ctx = context(eval, "existing_rules")?;
        bind("existing_rules", Wording::Signature, &[], args, eval)?;
        let state = ctx.state.borrow();
        let entries: Vec<(String, Value<'v>)> = state
            .builder
            .rules()
            .map(|target| (target.name.clone(), rule_view(ctx, target, eval)))
            .collect();
        Ok(eval.heap().alloc(AllocDict(entries)))
    }

    fn package_name<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<String> {
        let ctx = context(eval, "package_name")?;
        bind("package_name", Wording::Signature, &[], args, eval)?;
        Ok(ctx.package.to_owned())
    }

    fn repository_name<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<String> {
        let ctx = context(eval, "repository_name")?;
        bind("repository_name", Wording::Signature, &[], args, eval)?;
        Ok(format!("@{}", ctx.repo))
    }

    fn repo_name<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<String> {
        let ctx = context(eval, "repo_name")?;
        bind("repo_name", Wording::Signature, &[], args, eval)?;
        Ok(ctx.repo.to_owned())
    }
}

/// A `package()` list argument: a sequence, or the type error.
fn package_list<'v>(
    value: Value<'v>,
    expected: &str,
    noun: &str,
) -> starlark::Result<Vec<Value<'v>>> {
    sequence(value).ok_or_else(|| {
        fatal(format!(
            "expected value of type '{expected}' for {noun}, but got {}",
            describe(value)
        ))
    })
}

fn package_string(value: Value<'_>, noun: &str) -> starlark::Result<String> {
    value.unpack_str().map(str::to_owned).ok_or_else(|| {
        fatal(format!(
            "expected value of type 'string' for {noun}, but got {}",
            describe(value)
        ))
    })
}

// ---- rules -------------------------------------------------------------------

/// Instantiate a native rule from the keyword arguments of its call.
fn call_native_rule<'v>(
    class: &'static RuleClass,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<NoneType> {
    let ctx = context(eval, class.name)?;
    let at = location(eval);
    if args.positions(eval.heap())?.next().is_some() {
        return Err(fatal("unexpected positional arguments"));
    }
    let named: SmallMap<StringValue<'v>, Value<'v>> = args.names_map()?;
    let name_value = named
        .iter()
        .find(|(k, _)| k.as_str() == "name")
        .map(|(_, v)| *v)
        .ok_or_else(|| fatal(format!("{} rule has no 'name' attribute", class.name)))?;
    let name = name_value
        .unpack_str()
        .ok_or_else(|| fatal(format!("{} 'name' attribute must be a string", class.name)))?
        .to_owned();
    let me = Label {
        repo: ctx.repo.to_owned(),
        package: ctx.package.to_owned(),
        name: name.clone(),
    };
    let event = |message: String| ctx.event(&at, format!("{me}: {message}"));

    let mut attrs: Vec<(String, AttrValue)> = Vec::new();
    let mut provided: Vec<&str> = Vec::new();
    for (key, value) in named.iter() {
        let key = key.as_str();
        if key == "name" {
            continue;
        }
        let Some(spec) = class.attr(key) else {
            let hint = class
                .suggest(key)
                .map(|s| format!(" (did you mean '{s}'?)"))
                .unwrap_or_default();
            event(format!(
                "no such attribute '{key}' in '{}' rule{hint}",
                class.name
            ));
            continue;
        };
        match attr_value(ctx, class, spec, *value) {
            Ok(Some(v)) => {
                provided.push(spec.name);
                attrs.push((key.to_owned(), v));
            }
            // `None` says "as if unset", and counts as having been given.
            Ok(None) => provided.push(spec.name),
            Err(message) => event(message),
        }
    }
    for spec in class.attrs.iter().filter(|s| s.mandatory) {
        if !provided.contains(&spec.name) {
            event(format!(
                "missing value for mandatory attribute '{}' in '{}' rule",
                spec.name, class.name
            ));
        }
    }
    for (attr, value) in &attrs {
        if attr == "visibility" {
            continue;
        }
        if let AttrValue::LabelList(labels) = value {
            let mut seen: Vec<&Label> = Vec::new();
            for label in labels {
                if seen.contains(&label) {
                    ctx.event(
                        &at,
                        format!(
                            "Label '{label}' is duplicated in the '{attr}' attribute of rule \
                             '{name}'"
                        ),
                    );
                    break;
                }
                seen.push(label);
            }
        }
    }
    let visibility = attrs.iter().find_map(|(k, v)| match (k.as_str(), v) {
        ("visibility", AttrValue::LabelList(labels)) => Some(visibility_of(ctx, labels)),
        _ => None,
    });
    ctx.state
        .borrow_mut()
        .builder
        .add_rule_with(&name, class.name, attrs, visibility, &at)
        .map_err(|e| fatal(e.to_string()))?;
    Ok(NoneType)
}

/// Check one attribute value against its type. `Ok(None)` is `None`, taken
/// as unset; `Err` is the event message.
fn attr_value(
    ctx: &BuildContext<'_>,
    class: &RuleClass,
    spec: &AttrSpec,
    value: Value<'_>,
) -> Result<Option<AttrValue>, String> {
    if value.is_none() {
        return Ok(None);
    }
    let attr = spec.name;
    let rule = class.name;
    let wrong_type = |expected: &str| {
        format!(
            "expected value of type '{expected}' for attribute '{attr}' of '{rule}', but got {}",
            describe(value)
        )
    };
    let string_at = |i: usize, item: Value<'_>| {
        item.unpack_str().map(str::to_owned).ok_or_else(|| {
            format!(
                "expected value of type 'string' for element {i} of attribute '{attr}' of \
                 '{rule}', but got {}",
                describe(item)
            )
        })
    };
    let label_at = |i: usize, item: Value<'_>| -> Result<Label, String> {
        let s = string_at(i, item)?;
        Label::parse(&s, ctx.label_context()).map_err(|e| {
            format!("invalid label '{s}' in element {i} of attribute '{attr}' of '{rule}': {e}")
        })
    };
    Ok(Some(match spec.ty {
        AttrType::Bool => match (value.unpack_bool(), value.unpack_i32()) {
            (Some(b), _) => AttrValue::Bool(b),
            (None, Some(0)) => AttrValue::Bool(false),
            (None, Some(1)) => AttrValue::Bool(true),
            _ => {
                return Err(format!(
                    "expected one of [False, True, 0, 1] for attribute '{attr}' of '{rule}', but \
                     got {}",
                    describe(value)
                ));
            }
        },
        AttrType::String => AttrValue::String(
            value
                .unpack_str()
                .map(str::to_owned)
                .ok_or_else(|| wrong_type("string"))?,
        ),
        AttrType::Label => {
            let s = value.unpack_str().ok_or_else(|| wrong_type("string"))?;
            AttrValue::Label(Label::parse(s, ctx.label_context()).map_err(|e| {
                format!("invalid label '{s}' in attribute '{attr}' of '{rule}': {e}")
            })?)
        }
        AttrType::StringList => {
            let items = sequence(value).ok_or_else(|| wrong_type("list(string)"))?;
            AttrValue::StringList(
                items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| string_at(i, *item))
                    .collect::<Result<_, _>>()?,
            )
        }
        AttrType::LabelList => {
            let items = sequence(value).ok_or_else(|| wrong_type("list(label)"))?;
            AttrValue::LabelList(
                items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| label_at(i, *item))
                    .collect::<Result<_, _>>()?,
            )
        }
    }))
}

/// What `native.existing_rule` returns for a rule: a dict of its attributes
/// as Bazel lists them, with lists as tuples and labels written relative to
/// this package. It shows what the rule set and its class's defaults, and
/// nothing of `package()`'s defaults.
///
/// Bazel returns a live read-only view; this is a snapshot, a `dict`.
fn rule_view<'v>(
    ctx: &BuildContext<'_>,
    target: &Target,
    eval: &Evaluator<'v, '_, '_>,
) -> Value<'v> {
    let heap = eval.heap();
    let TargetKind::Rule { rule_class, attrs } = &target.kind else {
        return Value::new_none();
    };
    let mut entries: Vec<(&str, Value<'v>)> = vec![
        ("name", heap.alloc(target.name.as_str())),
        ("kind", heap.alloc(rule_class.as_str())),
    ];
    let class = native_rule(rule_class);
    for spec in class.map_or(&[][..], |c| c.attrs) {
        let set = attrs.iter().find(|(k, _)| k == spec.name).map(|(_, v)| v);
        let default;
        let value = match set {
            Some(v) => v,
            None => match spec.default.value(spec.ty) {
                Some(v) => {
                    default = v;
                    &default
                }
                None => continue,
            },
        };
        entries.push((spec.name, attr_to_value(ctx, value, eval)));
    }
    heap.alloc(AllocDict(entries))
}

fn attr_to_value<'v>(
    ctx: &BuildContext<'_>,
    value: &AttrValue,
    eval: &Evaluator<'v, '_, '_>,
) -> Value<'v> {
    let heap = eval.heap();
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;

    /// Every expectation is what Bazel 9.2.0 printed or reported for the
    /// same BUILD file (`print` shows up as its `DEBUG:` lines).
    const FILES: &[&str] = &[
        "f1.txt",
        "fa.txt",
        "sub/x.txt",
        "sub/deep/y.txt",
        "pk/BUILD",
        "pk/z.txt",
    ];

    fn repo(files: &[&str]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for file in files {
            let path = dir.path().join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "").unwrap();
        }
        dir
    }

    /// Resolves `load()` paths to in-memory `.bzl` sources evaluated with
    /// the `.bzl` globals, which is what a real loader would do.
    struct MapLoader(HashMap<&'static str, &'static str>);

    impl FileLoader for MapLoader {
        fn load(&self, path: &str) -> starlark::Result<starlark::environment::FrozenModule> {
            let src = self
                .0
                .get(path)
                .ok_or_else(|| fatal(format!("no such file {path}")))?;
            let ast = parse(path, src, FileKind::Bzl).map_err(starlark::Error::new_other)?;
            Module::with_temp_heap(|module| {
                {
                    let mut eval = Evaluator::new(&module);
                    eval.set_loader(self);
                    eval.eval_module(ast, &bzl_globals())?;
                }
                Ok(module.freeze()?)
            })
        }
    }

    fn load_in(
        root: &Path,
        package: &str,
        bzl: &[(&'static str, &'static str)],
        build: &str,
    ) -> Result<BuildFileOutput, BuildFileError> {
        let lookup = PackageLookup::new(root).unwrap();
        let loader = MapLoader(bzl.iter().copied().collect());
        evaluate_build_file(&BuildFile {
            repo: "",
            package,
            lookup: &lookup,
            path: "BUILD.bazel",
            source: build,
            loader: &loader,
        })
    }

    fn load(build: &str) -> Result<BuildFileOutput, BuildFileError> {
        load_with(&[], build)
    }

    fn load_with(
        bzl: &[(&'static str, &'static str)],
        build: &str,
    ) -> Result<BuildFileOutput, BuildFileError> {
        let dir = repo(FILES);
        load_in(dir.path(), "", bzl, build)
    }

    fn printed(build: &str) -> Vec<String> {
        load(build).unwrap_or_else(|e| panic!("{e}")).printed
    }

    /// The message of the fatal error, or of the events joined by newlines.
    fn failure(build: &str) -> String {
        match load(build) {
            Ok(_) => panic!("accepted:\n{build}"),
            Err(BuildFileError::Eval(e)) => format!("{e:#}"),
            Err(BuildFileError::Package(events)) => events.join("\n"),
        }
    }

    fn events(build: &str) -> Vec<String> {
        match load(build) {
            Err(BuildFileError::Package(events)) => events,
            Err(BuildFileError::Eval(e)) => panic!("fatal: {e:#}"),
            Ok(_) => panic!("accepted:\n{build}"),
        }
    }

    fn package(build: &str) -> Package {
        load(build).unwrap_or_else(|e| panic!("{e}")).package
    }

    #[test]
    fn glob_returns_a_sorted_list_of_strings() {
        assert_eq!(
            printed("print(glob([\"*.txt\"]))\nprint(glob([\"**/*.txt\"]))"),
            [
                r#"["f1.txt", "fa.txt"]"#,
                r#"["f1.txt", "fa.txt", "sub/deep/y.txt", "sub/x.txt"]"#
            ]
        );
        assert_eq!(
            printed("print(glob(include = [\"f1.txt\"], exclude = [\"nope\"]))"),
            [r#"["f1.txt"]"#]
        );
        assert_eq!(
            printed("print(glob((\"*.txt\",), exclude = (\"f*\",), allow_empty = True))"),
            ["[]"]
        );
        // Positional: include, exclude, exclude_directories, allow_empty.
        assert_eq!(
            printed("print(glob([\"*.txt\"], [\"fa*\"], 0, False))"),
            [r#"["f1.txt"]"#]
        );
        assert_eq!(
            printed("print(glob([\"sub/*\"], exclude_directories = 0))"),
            [r#"["sub/deep", "sub/x.txt"]"#]
        );
        // Results are ordinary lists.
        assert_eq!(
            printed(
                "print(type(glob([\"*.txt\"])), glob([\"*.txt\"])[0], glob([\"f1.txt\"]) + glob([\"fa.txt\"]))"
            ),
            [r#"list f1.txt ["f1.txt", "fa.txt"]"#]
        );
    }

    #[test]
    fn glob_errors_are_bazels() {
        for (build, want) in [
            (
                "glob(\"*.txt\")",
                "in call to glob(), parameter 'include' got value of type 'string', want 'sequence'",
            ),
            (
                "glob([\"*.txt\"], \"f*\")",
                "in call to glob(), parameter 'exclude' got value of type 'string', want 'sequence'",
            ),
            (
                "glob([\"*.txt\"], exclude_directories = True)",
                "in call to glob(), parameter 'exclude_directories' got value of type 'bool', want 'int'",
            ),
            (
                "glob([\"*.txt\"], exclude_directories = \"x\")",
                "in call to glob(), parameter 'exclude_directories' got value of type 'string', want 'int'",
            ),
            (
                "glob([\"*.txt\"], allow_empty = 1)",
                "expected boolean for argument `allow_empty`, got `1`",
            ),
            (
                "glob([\"*.txt\"], allow_empty = None)",
                "expected boolean for argument `allow_empty`, got `None`",
            ),
            (
                "glob([1])",
                "expected value of type 'string' for element 0 of 'glob' argument, but got 1 (int)",
            ),
            (
                "glob([\"*.txt\"], exclude = [1])",
                "expected value of type 'string' for element 0 of 'glob' argument, but got 1 (int)",
            ),
            (
                "glob([\"*.txt\"], bogus = 1)",
                "glob() got unexpected keyword argument 'bogus'",
            ),
            (
                "glob([\"*.txt\"], [\"f*\"], 1, True, 5)",
                "glob() accepts no more than 4 positional arguments but got 5",
            ),
            (
                "glob([\"?.txt\"])",
                "invalid glob pattern '?.txt': wildcard ? forbidden",
            ),
            (
                "glob([\"nope\"])",
                "glob pattern 'nope' didn't match anything, but allow_empty is set to False",
            ),
            (
                "glob()",
                "all files in the glob have been excluded, but allow_empty is set to False",
            ),
            (
                "glob([\"pk/z.txt\"])",
                "glob pattern 'pk/z.txt' didn't match anything",
            ),
        ] {
            let got = failure(build);
            assert!(got.contains(want), "{build}\n  want: {want}\n  got: {got}");
        }
    }

    #[test]
    fn a_bzl_reaches_the_natives_through_native_and_only_from_a_macro() {
        let bzl = [(
            ":m.bzl",
            "def mac(name):\n    native.filegroup(name = name, srcs = native.glob([\"*.txt\"]))\n    print(native.package_name(), native.repository_name(), native.existing_rules().keys())\n\ndef files():\n    return native.glob([\"*.txt\"])\n\nname = native.package_name\n",
        )];
        let out = load_with(
            &bzl,
            "load(\":m.bzl\", \"mac\", \"files\", \"name\")\nmac(\"a\")\nprint(files(), repr(name()))",
        )
        .unwrap();
        assert_eq!(
            out.printed,
            [
                r#" @ ["a"]"#.to_owned(),
                r#"["f1.txt", "fa.txt"] """#.to_owned()
            ]
        );
        let target = out.package.target("a").unwrap();
        assert!(matches!(target.kind, TargetKind::Rule { .. }));

        // Loading is not evaluating a BUILD file.
        let bzl = [(":n.bzl", "Y = native.glob([\"*.txt\"])\n")];
        let err = load_with(&bzl, "load(\":n.bzl\", \"Y\")").unwrap_err();
        let shown = err.to_string();
        assert!(
            shown.contains(
                "glob() can only be used while evaluating a BUILD file or a legacy macro"
            ),
            "{shown}"
        );
    }

    #[test]
    fn a_build_file_has_no_native() {
        assert!(failure("print(native)").contains("native"));
        assert!(!bzl_globals().names().any(|n| n.as_str() == "glob"));
        assert!(bzl_globals().names().any(|n| n.as_str() == "native"));
    }

    #[test]
    fn package_name_and_friends() {
        let dir = repo(FILES);
        let out = load_in(
            dir.path(),
            "sub",
            &[],
            "print(repr(package_name()), repr(repository_name()), repr(repo_name()))",
        )
        .unwrap();
        assert_eq!(out.printed, [r#""sub" "@" """#]);
    }

    #[test]
    fn package_records_its_defaults_wherever_it_stands() {
        let p = package(
            "filegroup(name = \"a\")\npackage(default_visibility = [\"//visibility:public\"], default_testonly = True, default_deprecation = \"dd\", default_compatible_with = [\"//c:x\"], default_restricted_to = [\":r\"], default_hdrs_check = \"loose\", licenses = [\"notice\"], features = [\"pf\"], default_package_metadata = [\":m\"])",
        );
        assert_eq!(
            p.default_visibility,
            fjfj_graph::visibility::Visibility::public()
        );
        let d = &p.defaults;
        assert!(d.testonly);
        assert_eq!(d.deprecation.as_deref(), Some("dd"));
        assert_eq!(d.compatible_with.len(), 1);
        assert_eq!(d.restricted_to[0].to_string(), "//:r");
        assert_eq!(d.hdrs_check.as_deref(), Some("loose"));
        assert_eq!(d.licenses, ["notice"]);
        assert_eq!(d.features, ["pf"]);
        assert_eq!(d.package_metadata[0].to_string(), "//:m");
        // `default_applicable_licenses` is the old name of the same setting.
        let p = package("package(default_applicable_licenses = [\":m\"])");
        assert_eq!(p.defaults.package_metadata[0].to_string(), "//:m");
        // default_testonly takes 0 and 1 too.
        assert!(package("package(default_testonly = 1)").defaults.testonly);
    }

    #[test]
    fn package_errors_are_bazels() {
        for (build, want) in [
            (
                "package(\"x\")",
                "package() got unexpected positional argument",
            ),
            ("package(bogus = 1)", "unexpected keyword argument: bogus"),
            (
                "package(default_license = [\"notice\"])",
                "unexpected keyword argument: default_license",
            ),
            (
                "package(default_visibility = \"//visibility:public\")",
                "expected value of type 'list(label)' for package() argument 'default_visibility', but got \"//visibility:public\" (string)",
            ),
            (
                "package(default_testonly = \"x\")",
                "expected one of [False, True, 0, 1] for package() argument 'default_testonly', but got \"x\" (string)",
            ),
            (
                "package(default_deprecation = 1)",
                "expected value of type 'string' for package() argument 'default_deprecation', but got 1 (int)",
            ),
            (
                "package(features = \"x\")",
                "expected value of type 'list(string)' for package() argument 'features', but got \"x\" (string)",
            ),
            (
                "package(default_applicable_licenses = \"x\")",
                "expected value of type 'list(label)' for package() argument 'default_applicable_licenses', but got \"x\" (string)",
            ),
            (
                "package(default_visibility = [\"bad//x\"])",
                "invalid label 'bad//x' in element 0 of package() argument 'default_visibility': invalid target name 'bad//x': target names may not contain '//' path separators",
            ),
            (
                "package(default_applicable_licenses = [], default_package_metadata = [])",
                "Can not set both default_package_metadata and default_applicable_licenses. Move all declarations to default_package_metadata.",
            ),
            (
                "package()\npackage()",
                "'package' can only be used once per BUILD file",
            ),
        ] {
            let got = failure(build);
            assert!(got.contains(want), "{build}\n  want: {want}\n  got: {got}");
        }
    }

    #[test]
    fn package_group_and_exports_files_declare_targets() {
        let p = package(
            "package_group(name = \"pg\", packages = [\"//a/...\", \"-//a/b\", \"//c\", \"private\"], includes = [\":pg2\"])\npackage_group(name = \"pg2\")\nexports_files([\"f1.txt\", \"sub/x.txt\"], visibility = [\"//visibility:private\"], licenses = [\"notice\"])\nexports_files(srcs = [\"fa.txt\"])\nexports_files([\"f1.txt\"])",
        );
        let TargetKind::PackageGroup(group) = &p.target("pg").unwrap().kind else {
            panic!()
        };
        assert_eq!(group.specs.len(), 3);
        assert_eq!(group.includes[0].to_string(), "//:pg2");
        assert_eq!(p.target("f1.txt").unwrap().kind, TargetKind::SourceFile);
        assert_eq!(
            p.target("f1.txt").unwrap().visibility,
            Some(fjfj_graph::visibility::Visibility::private())
        );
        assert_eq!(
            p.target("fa.txt").unwrap().visibility,
            Some(fjfj_graph::visibility::Visibility::public())
        );
    }

    #[test]
    fn package_group_and_exports_files_errors_are_bazels() {
        for (build, want) in [
            (
                "package_group(name = \"pg\", packages = \"//a\")",
                "in call to package_group(), parameter 'packages' got value of type 'string', want 'sequence'",
            ),
            (
                "package_group(packages = [])",
                "package_group() missing 1 required named argument: name",
            ),
            (
                "package_group(name = \"pg\", bogus = 1)",
                "package_group() got unexpected keyword argument 'bogus'",
            ),
            (
                "package_group(\"pg\")",
                "package_group() got unexpected positional argument",
            ),
            (
                "package_group(name = \"pg\", visibility = [\"//visibility:public\"])",
                "package_group() got unexpected keyword argument 'visibility'",
            ),
            (
                "package_group(name = \"pg\", includes = [\"bad//x\"])",
                "invalid label 'bad//x' in element 0 of 'package_group.includes argument': invalid target name 'bad//x': target names may not contain '//' path separators",
            ),
            (
                "package_group(name = \"pg\", includes = [1])",
                "expected value of type 'string' for element 0 of 'package_group.includes argument', but got 1 (int)",
            ),
            (
                "package_group(name = \"pg\", packages = [1])",
                "expected value of type 'string' for element 0 of 'package_group.packages argument', but got 1 (int)",
            ),
            (
                "exports_files(\"f1.txt\")",
                "in call to exports_files(), parameter 'srcs' got value of type 'string', want 'sequence'",
            ),
            (
                "exports_files([1])",
                "expected value of type 'string' for element 0 of 'exports_files' operand, but got 1 (int)",
            ),
            (
                "exports_files()",
                "exports_files() missing 1 required positional argument: srcs",
            ),
            (
                "exports_files([\"f1.txt\"], visibility = \"//visibility:public\")",
                "in call to exports_files(), parameter 'visibility' got value of type 'string', want 'sequence or NoneType'",
            ),
            (
                "exports_files([\"f1.txt\"], visibility = [\"bad//x\"])",
                "invalid label 'bad//x' in element 0 of 'exports_files' operand: invalid target name 'bad//x': target names may not contain '//' path separators",
            ),
            (
                "exports_files([\"/x\"])",
                "target names may not start with '/'",
            ),
            (
                "exports_files([\"..\"])",
                "target names may not contain up-level references '..'",
            ),
            ("exports_files([\"\"])", "empty target name"),
            (
                "exports_files([\"f1.txt\"], visibility = [\"//visibility:public\"])\nexports_files([\"f1.txt\"], visibility = [\"//visibility:public\"])",
                "visibility for exported file 'f1.txt' declared twice",
            ),
            (
                "exports_files([\"pk/z.txt\"])",
                "Label '//:pk/z.txt' is invalid because 'pk' is a subpackage; perhaps you meant to put the colon here: '//pk:z.txt'?",
            ),
        ] {
            let got = failure(build);
            assert!(got.contains(want), "{build}\n  want: {want}\n  got: {got}");
        }
    }

    #[test]
    fn a_bad_package_spec_is_an_event_not_a_stop() {
        let got = events(
            "package_group(name = \"pg\", packages = [\"x\", \"//a:b:c\", \"-//a:b\"])\nprint(\"still running\")",
        );
        assert_eq!(
            got,
            [
                "BUILD.bazel:1:14: invalid package name 'x': must start with '//', '@', or be 'public' or 'private'",
                "BUILD.bazel:1:14: invalid package name '//a:b:c': invalid target name 'b:c:__pkg__': target names may not contain ':'",
                "BUILD.bazel:1:14: invalid package name '//a:b': invalid target name 'b:__pkg__': target names may not contain ':'",
            ]
        );
    }

    #[test]
    fn filegroup_records_what_it_was_given() {
        let p = package(
            "filegroup(name = \"a\", srcs = [\"f1.txt\", \":b\", \"//sub:x.txt\"], data = (\"f1.txt\",), output_group = \"og\", testonly = 1, tags = [\"t\"], visibility = [\"//visibility:public\"], deprecation = \"old\", features = [\"f\"], compatible_with = [], licenses = [])\nfilegroup(name = \"b\")",
        );
        let a = p.target("a").unwrap();
        assert_eq!(
            a.visibility,
            Some(fjfj_graph::visibility::Visibility::public())
        );
        let TargetKind::Rule { rule_class, attrs } = &a.kind else {
            panic!()
        };
        assert_eq!(rule_class, "filegroup");
        let srcs = attrs.iter().find(|(k, _)| k == "srcs").unwrap();
        let AttrValue::LabelList(srcs) = &srcs.1 else {
            panic!()
        };
        assert_eq!(
            srcs.iter().map(ToString::to_string).collect::<Vec<_>>(),
            ["//:f1.txt", "//:b", "//sub:x.txt"]
        );
        assert!(attrs.contains(&("testonly".into(), AttrValue::Bool(true))));
        assert!(attrs.contains(&("output_group".into(), AttrValue::String("og".into()))));
        // A rule that names no source that exists is fine at load time.
        package("filegroup(name = \"a\", srcs = [\"nope.txt\"])");
        // `None` is "as if unset", for any attribute.
        package("filegroup(name = \"a\", srcs = None, data = None, output_group = None)");
    }

    #[test]
    fn alias_takes_a_label() {
        let p = package(
            "alias(name = \"al\", actual = \":a\", visibility = [\"//visibility:public\"], tags = [\"x\"], testonly = True, deprecation = \"d\")\nfilegroup(name = \"a\")",
        );
        let TargetKind::Rule { rule_class, attrs } = &p.target("al").unwrap().kind else {
            panic!()
        };
        assert_eq!(rule_class, "alias");
        assert!(attrs.contains(&(
            "actual".into(),
            AttrValue::Label(Label {
                repo: String::new(),
                package: String::new(),
                name: "a".into()
            })
        )));
        // `actual = None` is not "missing", oddly.
        package("alias(name = \"al\", actual = None)");
    }

    #[test]
    fn rule_attribute_errors_are_events_and_the_file_goes_on() {
        for (build, want) in [
            (
                "alias(name = \"al\")",
                vec![
                    "BUILD.bazel:1:6: //:al: missing value for mandatory attribute 'actual' in 'alias' rule",
                ],
            ),
            (
                "alias(name = \"al\", actual = [\":a\"])",
                vec![
                    "BUILD.bazel:1:6: //:al: expected value of type 'string' for attribute 'actual' of 'alias', but got [\":a\"] (list)",
                    "BUILD.bazel:1:6: //:al: missing value for mandatory attribute 'actual' in 'alias' rule",
                ],
            ),
            (
                "alias(name = \"al\", actual = \"bad//x\")",
                vec![
                    "BUILD.bazel:1:6: //:al: invalid label 'bad//x' in attribute 'actual' of 'alias': invalid target name 'bad//x': target names may not contain '//' path separators",
                    "BUILD.bazel:1:6: //:al: missing value for mandatory attribute 'actual' in 'alias' rule",
                ],
            ),
            (
                "alias(name = \"al\", actual = \"\")",
                vec![
                    "BUILD.bazel:1:6: //:al: invalid label '' in attribute 'actual' of 'alias': invalid target name '': empty target name",
                    "BUILD.bazel:1:6: //:al: missing value for mandatory attribute 'actual' in 'alias' rule",
                ],
            ),
            (
                "filegroup(name = \"a\", bogus = 1, bogus2 = 2)",
                vec![
                    "BUILD.bazel:1:10: //:a: no such attribute 'bogus' in 'filegroup' rule",
                    "BUILD.bazel:1:10: //:a: no such attribute 'bogus2' in 'filegroup' rule",
                ],
            ),
            (
                "filegroup(name = \"a\", src = [])",
                vec![
                    "BUILD.bazel:1:10: //:a: no such attribute 'src' in 'filegroup' rule (did you mean 'srcs'?)",
                ],
            ),
            (
                "alias(name = \"a\", actual = \":b\", src = [])",
                vec!["BUILD.bazel:1:6: //:a: no such attribute 'src' in 'alias' rule"],
            ),
            (
                "filegroup(name = \"a\", exec_properties = {})",
                vec![
                    "BUILD.bazel:1:10: //:a: no such attribute 'exec_properties' in 'filegroup' rule",
                ],
            ),
            (
                "filegroup(name = \"a\", srcs = \"f1.txt\")",
                vec![
                    "BUILD.bazel:1:10: //:a: expected value of type 'list(label)' for attribute 'srcs' of 'filegroup', but got \"f1.txt\" (string)",
                ],
            ),
            (
                "filegroup(name = \"a\", srcs = [1])",
                vec![
                    "BUILD.bazel:1:10: //:a: expected value of type 'string' for element 0 of attribute 'srcs' of 'filegroup', but got 1 (int)",
                ],
            ),
            (
                "filegroup(name = \"a\", srcs = [None])",
                vec![
                    "BUILD.bazel:1:10: //:a: expected value of type 'string' for element 0 of attribute 'srcs' of 'filegroup', but got None (NoneType)",
                ],
            ),
            (
                "filegroup(name = \"a\", srcs = [\"//bad//x\"])",
                vec![
                    "BUILD.bazel:1:10: //:a: invalid label '//bad//x' in element 0 of attribute 'srcs' of 'filegroup': invalid package name 'bad//x': package names may not contain '//' path separators (perhaps you meant \":x\"?)",
                ],
            ),
            (
                "filegroup(name = \"a\", output_group = 1)",
                vec![
                    "BUILD.bazel:1:10: //:a: expected value of type 'string' for attribute 'output_group' of 'filegroup', but got 1 (int)",
                ],
            ),
            (
                "filegroup(name = \"a\", testonly = \"x\")",
                vec![
                    "BUILD.bazel:1:10: //:a: expected one of [False, True, 0, 1] for attribute 'testonly' of 'filegroup', but got \"x\" (string)",
                ],
            ),
            (
                "filegroup(name = \"a\", visibility = \"//visibility:public\")",
                vec![
                    "BUILD.bazel:1:10: //:a: expected value of type 'list(label)' for attribute 'visibility' of 'filegroup', but got \"//visibility:public\" (string)",
                ],
            ),
            (
                "filegroup(name = \"a\", srcs = [\"f1.txt\", \":f1.txt\"])",
                vec![
                    "BUILD.bazel:1:10: Label '//:f1.txt' is duplicated in the 'srcs' attribute of rule 'a'",
                ],
            ),
            (
                "filegroup(name = \"a\", data = [\"f1.txt\", \"f1.txt\"])",
                vec![
                    "BUILD.bazel:1:10: Label '//:f1.txt' is duplicated in the 'data' attribute of rule 'a'",
                ],
            ),
            (
                "filegroup(name = \"a\", compatible_with = [\"//x:y\", \"//x:y\"])",
                vec![
                    "BUILD.bazel:1:10: Label '//x:y' is duplicated in the 'compatible_with' attribute of rule 'a'",
                ],
            ),
        ] {
            assert_eq!(events(build), want, "{build}");
        }
        // Repeats that are fine: tags, and visibility.
        package(
            "filegroup(name = \"a\", tags = [\"x\", \"x\"], visibility = [\"//visibility:public\", \"//visibility:public\"])",
        );
    }

    #[test]
    fn rule_call_errors_stop_the_file() {
        for (build, want) in [
            (
                "filegroup(srcs = [])",
                "filegroup rule has no 'name' attribute",
            ),
            (
                "filegroup(name = 1)",
                "filegroup 'name' attribute must be a string",
            ),
            ("filegroup(\"a\")", "unexpected positional arguments"),
            (
                "filegroup(name = \"../x\")",
                "illegal rule name: ../x: invalid target name '../x': target names may not contain up-level references '..'",
            ),
            (
                "filegroup(name = \"\")",
                "illegal rule name: : invalid target name '': empty target name",
            ),
            (
                "filegroup(name = \"a\")\nfilegroup(name = \"a\")",
                "filegroup rule 'a' conflicts with existing filegroup rule, defined at BUILD.bazel:1:10",
            ),
            (
                "filegroup(name = \"a\", srcs = 1)\nfilegroup(name = \"a\")",
                "filegroup rule 'a' conflicts with existing filegroup rule, defined at BUILD.bazel:1:10",
            ),
            (
                "filegroup(name = \"pk/x\")",
                "Label '//:pk/x' is invalid because 'pk' is a subpackage; perhaps you meant to put the colon here: '//pk:x'?",
            ),
        ] {
            let got = failure(build);
            assert!(got.contains(want), "{build}\n  want: {want}\n  got: {got}");
        }
    }

    #[test]
    fn a_rule_with_a_returned_value_is_none() {
        assert_eq!(
            printed(
                "print(filegroup(name = \"a\"))\nprint(package(default_visibility = [\"//visibility:public\"]))\nprint(package_group(name = \"pg\"))\nprint(exports_files([\"f1.txt\"]))"
            ),
            ["None", "None", "None", "None"]
        );
    }

    /// The whole dict, as Bazel printed it.
    #[test]
    fn existing_rule_shows_the_attributes_bazel_shows() {
        assert_eq!(
            printed("filegroup(name = \"a\")\nprint(dict(existing_rule(\"a\")))"),
            [
                r#"{"name": "a", "kind": "filegroup", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "distribs": (), "target_compatible_with": (), "srcs": (), "output_group": "", "data": (), "output_licenses": ()}"#
            ]
        );
        assert_eq!(
            printed(
                "filegroup(name = \"a\", srcs = [\"f1.txt\", \":b\", \"//sub:x.txt\"], tags = [\"x\"], visibility = [\"//visibility:public\"], testonly = 1, output_group = \"og\")\nprint(dict(existing_rule(\"a\")))"
            ),
            [
                r#"{"name": "a", "kind": "filegroup", "visibility": ("//visibility:public",), "transitive_configs": (), "tags": ("x",), "generator_name": "", "generator_function": "", "generator_location": "", "testonly": True, "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "distribs": (), "target_compatible_with": (), "srcs": (":f1.txt", ":b", "//sub:x.txt"), "output_group": "og", "data": (), "output_licenses": ()}"#
            ]
        );
        assert_eq!(
            printed(
                "alias(name = \"al\", actual = \":a\", visibility = [\"//visibility:public\"])\nfilegroup(name = \"a\")\nprint(dict(existing_rule(\"al\")))"
            ),
            [
                r#"{"name": "al", "kind": "alias", "visibility": ("//visibility:public",), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "target_compatible_with": (), "actual": ":a"}"#
            ]
        );
        // testonly=0 and deprecation appear once set, and `package()`'s
        // defaults never do.
        assert_eq!(
            printed(
                "package(default_testonly = True, default_deprecation = \"dd\", features = [\"pf\"])\nfilegroup(name = \"a\", features = [\"rf\"])\nfilegroup(name = \"b\", testonly = 0, deprecation = \"e\")\nprint(existing_rule(\"a\").get(\"testonly\"), existing_rule(\"a\").get(\"deprecation\"), existing_rule(\"a\")[\"features\"])\nprint(existing_rule(\"b\")[\"testonly\"], existing_rule(\"b\")[\"deprecation\"])"
            ),
            [r#"None None ("rf",)"#, "False e"]
        );
    }

    #[test]
    fn existing_rule_is_a_lookup() {
        assert_eq!(
            printed(
                "print(existing_rule(\"a\"), existing_rules().keys())\nfilegroup(name = \"a\", srcs = [\"f1.txt\"], tags = [\"x\"])\nalias(name = \"al\", actual = \":a\")\nprint(existing_rules().keys(), len(existing_rules()), \"a\" in existing_rules())\nr = existing_rule(\"a\")\nprint(r[\"srcs\"], type(r[\"srcs\"]), r[\"kind\"], r[\"name\"], r[\"tags\"], r.get(\"nope\", \"dflt\"), \"srcs\" in r, \"nope\" in r)\nprint(existing_rule(\"nope\"))\nprint(existing_rules()[\"al\"][\"actual\"])"
            ),
            [
                "None []",
                r#"["a", "al"] 2 True"#,
                r#"(":f1.txt",) tuple filegroup a ("x",) dflt True False"#,
                "None",
                ":a",
            ]
        );
        // Only rules: not exported files, not package groups.
        assert_eq!(
            printed(
                "exports_files([\"f1.txt\"])\npackage_group(name = \"pg\")\nprint(existing_rules().keys(), existing_rule(\"f1.txt\"), existing_rule(\"pg\"))"
            ),
            ["[] None None"]
        );
        for (build, want) in [
            (
                "existing_rule(1)",
                "in call to existing_rule(), parameter 'name' got value of type 'int', want 'string'",
            ),
            (
                "existing_rule()",
                "existing_rule() missing 1 required positional argument: name",
            ),
        ] {
            assert!(failure(build).contains(want), "{build}");
        }
    }

    #[test]
    fn labels_in_a_rule_read_in_the_repo_they_are_written_in() {
        let dir = repo(FILES);
        let lookup = PackageLookup::new(dir.path()).unwrap();
        let loader = MapLoader(HashMap::new());
        let out = evaluate_build_file(&BuildFile {
            repo: "dep+",
            package: "p",
            lookup: &lookup,
            path: "BUILD.bazel",
            source: "filegroup(name = \"a\", srcs = [\":x\", \"//q:y\", \"@//m:z\", \"@r//s:t\"])\nprint(existing_rule(\"a\")[\"srcs\"], repr(repository_name()), repr(repo_name()))",
            loader: &loader,
        })
        .unwrap();
        assert_eq!(
            out.printed,
            [r#"(":x", "//q:y", "@@//m:z", "@@r//s:t") "@dep+" "dep+""#]
        );
    }

    #[test]
    fn a_subpackage_directory_is_not_a_target_name() {
        // `pk` has a BUILD file in the fixture.
        let got =
            failure("filegroup(name = \"a\", srcs = [\"pk/z.txt\"])\nfilegroup(name = \"pk/b\")");
        assert!(got.contains("'pk' is a subpackage"), "{got}");
    }

    #[test]
    fn a_load_error_is_not_swallowed() {
        let err = load("load(\":missing.bzl\", \"x\")").unwrap_err();
        assert!(matches!(err, BuildFileError::Eval(_)));
    }

    #[test]
    fn a_syntax_error_names_the_file() {
        let err = load("def f():\n    pass\n").unwrap_err();
        assert!(err.to_string().contains("BUILD.bazel"), "{err}");
    }
}
