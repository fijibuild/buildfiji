//! `Label` (buildfiji-mum.3.2): the label type of `.bzl` files, `Label()`,
//! and what it needs to know about where it is called from.
//!
//! What Bazel 9.2.0 does:
//!
//! - `Label(s)` resolves `s` against the `.bzl` file the call is written in,
//!   not the BUILD file being loaded: `:x` is in that file's package and
//!   `//p:x` in its repository, and `@r` is an *apparent* repo name mapped
//!   through that repository's mapping. A name the mapping lacks is not an
//!   error: it makes a label in a repo called `[unknown repo 'r' requested
//!   from @@]`, which fails only when its `repo_name`, `workspace_name` or
//!   `workspace_root` is read. `Label(label)` is the label itself.
//! - `str` is `@@repo//pkg:name` (`@@//pkg:name` in the main repo) and
//!   `repr` is `Label("//pkg:name")` for the main repo and
//!   `Label("@@repo//pkg:name")` for another. Labels are equal, hashable and
//!   ordered by (repo, package, name), and the members are `name`,
//!   `package`, `repo_name`, `workspace_name`, `workspace_root`, `relative`
//!   and `same_package_label`.
//! - `label.relative(s)` reads `s` like `Label(s)`, but against the label's
//!   own package, and through the mapping of the `.bzl` that *calls* it.
//!
//! # Where the call comes from
//!
//! A native function cannot ask which module defined the function it was
//! called from, so a `.bzl` is evaluated under a name that says: the
//! canonical label of the file, `@@repo//pkg:file.bzl`. The frame that calls
//! `Label` carries that name, and the [`RepoMappings`] of an evaluation give
//! the mapping of the repo it names. [`evaluate_bzl`] sets that up, and a
//! loader should use it.
//!
//! Repository mapping is buildfiji-mum.15's: it fills [`RepoMappings`] from
//! the module graph, and nothing here changes when it does.
//!
//! `print(label)` differs from Bazel, which prints a label the way the main
//! repo's mapping would write it (`//a:b`, `@dep//a:b`) and not as `str`
//! does. The crate's `print` calls `str`; see buildfiji-xq5.

use crate::args::{Wording, bind, fatal, positional_only};
use crate::dialect::assigned_names;
use crate::exports::export_all;
use crate::native::BuildContext;
use crate::{FileKind, parse};
use allocative::Allocative;
use fjfj_graph::label::validate_target_name;
use fjfj_graph::rule::suggest;
use fjfj_graph::{Label, LabelContext, LabelParseError};
use starlark::PrintHandler;
use starlark::collections::StarlarkHasher;
use starlark::environment::{
    FrozenModule, Globals, Methods, MethodsBuilder, MethodsStatic, Module,
};
use starlark::eval::{Arguments, Evaluator, FileLoader};
use starlark::starlark_module;
use starlark::starlark_simple_value;
use starlark::values::{Heap, NoSerialize, ProvidesStaticType, StarlarkValue, Value, ValueLike};
use starlark_derive::starlark_value;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::hash::Hash;

/// Apparent to canonical repo names, for every repository: what `@r` means
/// in a label written in each.
#[derive(Debug, Clone, Default)]
pub struct RepoMappings {
    by_repo: HashMap<String, BTreeMap<String, String>>,
}

impl RepoMappings {
    pub fn new() -> Self {
        RepoMappings::default()
    }

    /// Record what the repo `repo` (canonical; empty for the main one) calls
    /// each repo it can name: `(apparent, canonical)` pairs. A repo names
    /// itself, and the main repo's `""` is the main repo: `@//p:q`.
    pub fn insert(
        &mut self,
        repo: impl Into<String>,
        entries: impl IntoIterator<Item = (String, String)>,
    ) {
        self.by_repo
            .insert(repo.into(), entries.into_iter().collect());
    }

    /// The canonical repo `apparent` names in `from`, or the placeholder
    /// Bazel puts in a label whose repo `from` cannot name.
    pub(crate) fn resolve_apparent(&self, from: &str, apparent: &str) -> String {
        let entries = self.by_repo.get(from);
        if let Some(canonical) = entries.and_then(|e| e.get(apparent)) {
            return canonical.clone();
        }
        let hint = entries
            .and_then(|e| suggest(apparent, e.keys().map(String::as_str)))
            .map(|s| format!(" (did you mean '{s}'?)"))
            .unwrap_or_default();
        format!("[unknown repo '{apparent}' requested from @@{from}{hint}]")
    }
}

/// The name a `.bzl` file is evaluated and parsed under: its canonical
/// label.
pub fn bzl_name(file: &Label) -> String {
    format!("@@{}//{}:{}", file.repo, file.package, file.name)
}

/// What evaluating a `.bzl` file needs.
pub struct BzlFile<'a> {
    /// The file, canonically: its repo, its package, its name.
    pub file: &'a Label,
    pub source: &'a str,
    /// [`bzl_globals`](crate::bzl_globals), built once for all files.
    pub globals: &'a Globals,
    pub mappings: &'a RepoMappings,
    /// Resolves the file's `load()`s, each through [`evaluate_bzl`].
    pub loader: &'a dyn FileLoader,
    /// Where `print` goes; standard error if unset.
    pub print: Option<&'a dyn PrintHandler>,
}

/// The state a `.bzl` evaluation hands `Label`.
#[derive(ProvidesStaticType)]
struct BzlEval<'a> {
    mappings: &'a RepoMappings,
    /// The names the file assigns at its top level, in order.
    assigned: Vec<String>,
}

/// Evaluate a `.bzl` file and freeze what it defines.
pub fn evaluate_bzl(input: &BzlFile<'_>) -> starlark::Result<FrozenModule> {
    let _span = tracing::debug_span!("evaluate_bzl", file = %bzl_name(input.file)).entered();
    let ast = parse(&bzl_name(input.file), input.source, FileKind::Bzl)
        .map_err(starlark::Error::new_other)?;
    let env = BzlEval {
        mappings: input.mappings,
        assigned: assigned_names(&ast),
    };
    Module::with_temp_heap(|module| {
        {
            let mut eval = Evaluator::new(&module);
            eval.extra = Some(&env);
            eval.set_loader(input.loader);
            if let Some(print) = input.print {
                eval.set_print_handler(print);
            }
            eval.eval_module(ast, input.globals)?;
        }
        let frozen = module.freeze()?;
        export_all(&frozen, &env.assigned)?;
        Ok(frozen)
    })
}

/// Whether `eval` is running a `.bzl` file (and not a BUILD file, in which
/// the module it looks at is not the one that made a value).
pub(crate) fn evaluating_bzl(eval: &Evaluator<'_, '_, '_>) -> bool {
    eval.extra
        .is_some_and(|e| e.downcast_ref::<BzlEval>().is_some())
}

/// Whether the `.bzl` being evaluated assigns a top-level name that starts
/// with `_`, which the `starlark` crate will not let a native function read.
pub(crate) fn assigns_private_names(eval: &Evaluator<'_, '_, '_>) -> bool {
    eval.extra
        .and_then(|e| e.downcast_ref::<BzlEval>())
        .is_some_and(|env| env.assigned.iter().any(|n| n.starts_with('_')))
}

/// The mappings of the evaluation `eval` is running, if it has any.
fn mappings_of<'a>(eval: &Evaluator<'_, 'a, '_>) -> Option<&'a RepoMappings> {
    let extra = eval.extra?;
    if let Some(build) = extra.downcast_ref::<BuildContext>() {
        Some(build.mappings)
    } else {
        extra.downcast_ref::<BzlEval>().map(|env| env.mappings)
    }
}

/// The `.bzl` file whose code is making the current call, and the mappings.
pub(crate) fn caller<'a>(
    eval: &Evaluator<'_, 'a, '_>,
    function: &str,
) -> starlark::Result<(Label, &'a RepoMappings)> {
    let mappings = mappings_of(eval).ok_or_else(|| {
        fatal(format!(
            "{function}() can only be called while a .bzl file is evaluated"
        ))
    })?;
    let file = eval
        .call_stack_top_location()
        .map(|span| span.resolve().file)
        .unwrap_or_default();
    let label = Label::parse(
        &file,
        LabelContext {
            repo: "",
            package: "",
        },
    )
    .map_err(|_| {
        fatal(format!(
            "{function}() cannot tell which .bzl called it: '{file}'"
        ))
    })?;
    Ok((label, mappings))
}

/// Read `text` as a label written in `at`, naming repos as `written_in`
/// does.
pub(crate) fn resolve(
    text: &str,
    at: LabelContext<'_>,
    written_in: &str,
    mappings: &RepoMappings,
) -> Result<StarlarkLabel, LabelParseError> {
    Label::parse_mapped(text, at, &mut |apparent| {
        mappings.resolve_apparent(written_in, apparent)
    })
    .map(StarlarkLabel::from)
}

/// `text` as a label written in the `.bzl` whose code is making the call
/// `function`. The outer error is that no `.bzl` is; the inner one is Bazel's
/// text for a label that does not parse.
pub(crate) fn parse_in_caller(
    eval: &Evaluator<'_, '_, '_>,
    function: &str,
    text: &str,
) -> starlark::Result<Result<Label, LabelParseError>> {
    let (file, mappings) = caller(eval, function)?;
    Ok(Label::parse_mapped(
        text,
        LabelContext {
            repo: &file.repo,
            package: &file.package,
        },
        &mut |apparent| mappings.resolve_apparent(&file.repo, apparent),
    ))
}

/// The label a `Label` value holds.
pub(crate) fn label_of_value(value: Value<'_>) -> Option<Label> {
    value.downcast_ref::<StarlarkLabel>().map(|l| Label {
        repo: l.repo.clone(),
        package: l.package.clone(),
        name: l.name.clone(),
    })
}

/// How a label is written to say which one it is: `//pkg:name` in the main
/// repo, `@@repo//pkg:name` elsewhere.
pub(crate) fn display_label(label: &Label) -> String {
    format!("@@{}//{}:{}", label.repo, label.package, label.name).replacen("@@//", "//", 1)
}

/// A label, as a Starlark value.
#[derive(Debug, Clone, PartialEq, Eq, ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct StarlarkLabel {
    repo: String,
    package: String,
    name: String,
}

starlark_simple_value!(StarlarkLabel);

impl From<Label> for StarlarkLabel {
    fn from(label: Label) -> Self {
        StarlarkLabel {
            repo: label.repo,
            package: label.package,
            name: label.name,
        }
    }
}

impl StarlarkLabel {
    /// A repo the mapping could not name is a placeholder, not a name.
    fn is_invalid(&self) -> bool {
        self.repo.starts_with('[')
    }

    /// The repo, or the error Bazel gives for reading it from a label that
    /// has none.
    fn valid_repo(&self, member: &str) -> starlark::Result<&str> {
        if self.is_invalid() {
            Err(fatal(format!(
                "'{member}' is not allowed on invalid Label {}",
                self.unambiguous()
            )))
        } else {
            Ok(&self.repo)
        }
    }

    /// `str`: `@@repo//pkg:name`, `@@//pkg:name` in the main repo.
    fn unambiguous(&self) -> String {
        format!("@@{}//{}:{}", self.repo, self.package, self.name)
    }

    fn context(&self) -> LabelContext<'_> {
        LabelContext {
            repo: &self.repo,
            package: &self.package,
        }
    }
}

/// The `repr` form, which is what the crate means by `Display`, and how a
/// label shows inside a list, tuple or struct.
impl fmt::Display for StarlarkLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let form = if self.repo.is_empty() {
            format!("//{}:{}", self.package, self.name)
        } else {
            self.unambiguous()
        };
        write!(f, "Label(\"{}\")", form.replace('"', "\\\""))
    }
}

#[starlark_value(type = "Label")]
impl<'v> StarlarkValue<'v> for StarlarkLabel {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("Label", label_members);
        Some(RES.methods())
    }

    fn collect_str(&self, collector: &mut String) {
        collector.push_str(&self.unambiguous());
    }

    fn write_hash(&self, hasher: &mut StarlarkHasher) -> starlark::Result<()> {
        self.repo.hash(hasher);
        self.package.hash(hasher);
        self.name.hash(hasher);
        Ok(())
    }

    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(other
            .downcast_ref::<StarlarkLabel>()
            .is_some_and(|other| other == self))
    }

    fn compare(&self, other: Value<'v>) -> starlark::Result<Ordering> {
        let Some(other) = other.downcast_ref::<StarlarkLabel>() else {
            return Err(fatal(format!(
                "unsupported comparison: Label <=> {}",
                other.get_type()
            )));
        };
        Ok(
            (&self.repo, &self.package, &self.name).cmp(&(
                &other.repo,
                &other.package,
                &other.name,
            )),
        )
    }
}

fn this_label<'v>(this: Value<'v>) -> &'v StarlarkLabel {
    this.downcast_ref::<StarlarkLabel>()
        .expect("a Label method is called on a Label")
}

#[starlark_module]
fn label_members(builder: &mut MethodsBuilder) {
    /// The target's name.
    #[starlark(attribute)]
    fn name<'v>(this: Value<'v>) -> starlark::Result<String> {
        Ok(this_label(this).name.clone())
    }

    /// The package, `""` for the root package.
    #[starlark(attribute)]
    fn package<'v>(this: Value<'v>) -> starlark::Result<String> {
        Ok(this_label(this).package.clone())
    }

    /// The canonical repo name, `""` for the main repository.
    #[starlark(attribute)]
    fn repo_name<'v>(this: Value<'v>) -> starlark::Result<String> {
        Ok(this_label(this).valid_repo("repo_name")?.to_owned())
    }

    /// The same as `repo_name`.
    #[starlark(attribute)]
    fn workspace_name<'v>(this: Value<'v>) -> starlark::Result<String> {
        Ok(this_label(this).valid_repo("workspace_name")?.to_owned())
    }

    /// Where the repo's files are, relative to the execution root: empty
    /// for the main repository.
    #[starlark(attribute)]
    fn workspace_root<'v>(this: Value<'v>) -> starlark::Result<String> {
        let repo = this_label(this).valid_repo("workspace_root")?;
        Ok(if repo.is_empty() {
            String::new()
        } else {
            format!("external/{repo}")
        })
    }

    /// `label.relative(relName)`.
    fn relative<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<StarlarkLabel> {
        let bound = bind(
            "relative",
            Wording::Signature,
            &[positional_only("relName", true)],
            args,
            eval,
        )?;
        let rel = string_of("relative", "relName", bound[0].expect("required"))?;
        let (file, mappings) = caller(eval, "relative")?;
        resolve(rel, this_label(this).context(), &file.repo, mappings)
            .map_err(|e| fatal(e.to_string()))
    }

    /// `label.same_package_label(target_name)`.
    fn same_package_label<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<StarlarkLabel> {
        let bound = bind(
            "same_package_label",
            Wording::Signature,
            &[positional_only("target_name", true)],
            args,
            eval,
        )?;
        let name = string_of(
            "same_package_label",
            "target_name",
            bound[0].expect("required"),
        )?;
        validate_target_name(name).map_err(|source| {
            fatal(
                LabelParseError::Target {
                    name: name.to_owned(),
                    source,
                }
                .to_string(),
            )
        })?;
        let mut label = this_label(this).clone();
        label.name = name.to_owned();
        Ok(label)
    }
}

fn string_of<'v>(function: &str, param: &str, value: Value<'v>) -> starlark::Result<&'v str> {
    value.unpack_str().ok_or_else(|| {
        fatal(format!(
            "in call to {function}(), parameter '{param}' got value of type '{}', want 'string'",
            value.get_type()
        ))
    })
}

/// `input` as a label written in `at`: a string is resolved, a label is
/// itself. `who` names the caller in the error for a bad string.
fn label_of<'v>(
    input: Value<'v>,
    function: &str,
    who: &str,
    at: LabelContext<'_>,
    written_in: &str,
    mappings: &RepoMappings,
    heap: Heap<'v>,
) -> starlark::Result<Value<'v>> {
    if input.downcast_ref::<StarlarkLabel>().is_some() {
        return Ok(input);
    }
    let Some(text) = input.unpack_str() else {
        return Err(fatal(format!(
            "in call to {function}(), parameter 'input' got value of type '{}', want 'string or \
             Label'",
            input.get_type()
        )));
    };
    let label = resolve(text, at, written_in, mappings)
        .map_err(|e| fatal(format!("invalid label in {who}: {e}")))?;
    Ok(heap.alloc(label))
}

#[starlark_module]
pub(crate) fn label_globals(builder: &mut starlark::environment::GlobalsBuilder) {
    /// `Label(input)`.
    #[allow(non_snake_case)]
    fn Label<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let bound = bind(
            "Label",
            Wording::Signature,
            &[positional_only("input", true)],
            args,
            eval,
        )?;
        let (file, mappings) = caller(eval, "Label")?;
        label_of(
            bound[0].expect("required"),
            "Label",
            "Label()",
            LabelContext {
                repo: &file.repo,
                package: &file.package,
            },
            &file.repo,
            mappings,
            eval.heap(),
        )
    }
}

/// `native.package_relative_label(input)`: `input` as written in the package
/// being loaded, through its repo's mapping.
pub(crate) fn relative_to_package<'v>(
    input: Value<'v>,
    at: LabelContext<'_>,
    mappings: &RepoMappings,
    heap: Heap<'v>,
) -> starlark::Result<Value<'v>> {
    label_of(
        input,
        "package_relative_label",
        "native.package_relative_label",
        at,
        at.repo,
        mappings,
        heap,
    )
}
