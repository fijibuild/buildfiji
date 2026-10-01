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
    /// What `.bzl` files looked up, shared by every copy of the mappings.
    lookups: std::sync::Arc<LookupLog>,
}

/// An apparent repo name a `.bzl` file looked up through its repo's mapping:
/// the file's repo, the name, and the canonical repo it named (`\0` if none).
pub type RepoLookup = (String, String, String);

/// The repo-name lookups each `.bzl` file made while it loaded (a `load()`
/// label) or ran (`Label("@dep//...")`), and which files each one loaded. A
/// module extension's `recordedInputs` has `REPO_MAPPING:` for those of the
/// files it was built from.
#[derive(Debug, Default)]
pub struct LookupLog {
    by_file: std::sync::Mutex<BTreeMap<String, std::collections::BTreeSet<RepoLookup>>>,
    loads: std::sync::Mutex<BTreeMap<String, std::collections::BTreeSet<String>>>,
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

    /// The mappings of a module graph: `fjfj_bzlmod::Resolution::repo_mappings`
    /// gives them as a canonical repo name and its `(apparent, canonical)`
    /// rows.
    pub fn from_repos(repos: impl IntoIterator<Item = (String, Vec<(String, String)>)>) -> Self {
        let mut mappings = RepoMappings::new();
        for (repo, rows) in repos {
            mappings.insert(repo, rows);
        }
        mappings
    }

    /// Note that the file `file` (as [`bzl_name`] writes it) wrote the repo name
    /// `apparent` in a label.
    pub(crate) fn note_lookup(&self, file: &Label, apparent: &str) {
        let canonical = self
            .find_apparent(&file.repo, apparent)
            .unwrap_or_else(|| "\\0".to_owned());
        self.lookups
            .by_file
            .lock()
            .unwrap()
            .entry(bzl_name(file))
            .or_default()
            .insert((file.repo.clone(), apparent.to_owned(), canonical));
    }

    /// Note that the file `importer` loads `loaded`.
    pub(crate) fn note_load(&self, importer: &Label, loaded: &Label) {
        self.lookups
            .loads
            .lock()
            .unwrap()
            .entry(bzl_name(importer))
            .or_default()
            .insert(bzl_name(loaded));
    }

    /// The lookups made by `root` and every file it loads, directly or not.
    pub fn lookups_under(&self, root: &Label) -> std::collections::BTreeSet<RepoLookup> {
        let loads = self.lookups.loads.lock().unwrap();
        let by_file = self.lookups.by_file.lock().unwrap();
        let mut seen = std::collections::BTreeSet::new();
        let mut todo = vec![bzl_name(root)];
        let mut found = std::collections::BTreeSet::new();
        while let Some(file) = todo.pop() {
            if !seen.insert(file.clone()) {
                continue;
            }
            found.extend(by_file.get(&file).into_iter().flatten().cloned());
            todo.extend(loads.get(&file).into_iter().flatten().cloned());
        }
        found
    }

    /// The canonical repo `apparent` names in `from`, if it names one.
    pub fn find_apparent(&self, from: &str, apparent: &str) -> Option<String> {
        self.by_repo.get(from)?.get(apparent).cloned()
    }

    /// The canonical repo `apparent` names in `from`, or the placeholder
    /// Bazel puts in a label whose repo `from` cannot name.
    pub fn resolve_apparent(&self, from: &str, apparent: &str) -> String {
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
pub(crate) struct BzlEval<'a> {
    mappings: &'a RepoMappings,
    /// The names the file assigns at its top level, in order.
    assigned: Vec<String>,
    /// The file's own code is being run, as opposed to a function of it
    /// called later (a repository rule's implementation).
    loading: bool,
    /// A module extension is running, and what it has made so far.
    pub(crate) extension: Option<std::cell::RefCell<crate::module_ctx::ExtensionState>>,
    /// Where a running module extension's inputs are kept: a `Label` it makes
    /// of `@dep//...` is one.
    pub(crate) recorder: Option<crate::repo_ctx::Recorder>,
    /// The file being evaluated, when it is one's code that runs.
    file: Option<Label>,
}

impl<'a> BzlEval<'a> {
    /// The state for running code of a `.bzl` file that has already been
    /// evaluated (a repository rule's implementation), which can make
    /// `Label`s.
    pub(crate) fn running(mappings: &'a RepoMappings) -> BzlEval<'a> {
        BzlEval {
            mappings,
            assigned: Vec::new(),
            loading: false,
            extension: None,
            recorder: None,
            file: None,
        }
    }
}

/// fjfj's own `.bzl`: the providers every file has as globals.
const BUILTINS_SOURCE: &str = include_str!("builtins.bzl");

/// The loads of the builtins: there are none.
struct NoLoads;

impl FileLoader for NoLoads {
    fn load(&self, path: &str) -> starlark::Result<FrozenModule> {
        Err(starlark::Error::new_other(anyhow::anyhow!(
            "the builtins load nothing, not {path}"
        )))
    }
}

/// The frozen module of [`BUILTINS_SOURCE`], made once.
fn builtins_module() -> &'static FrozenModule {
    static MODULE: std::sync::OnceLock<FrozenModule> = std::sync::OnceLock::new();
    MODULE.get_or_init(|| {
        let globals = crate::native::bzl_globals();
        let mappings = RepoMappings::new();
        let label = Label {
            repo: "_builtins".to_owned(),
            package: String::new(),
            name: "providers.bzl".to_owned(),
        };
        evaluate_bzl_with(
            &BzlFile {
                file: &label,
                source: BUILTINS_SOURCE,
                globals: &globals,
                mappings: &mappings,
                loader: &NoLoads,
                print: None,
            },
            false,
        )
        .expect("the builtins evaluate")
    })
}

/// Evaluate a `.bzl` file and freeze what it defines.
pub fn evaluate_bzl(input: &BzlFile<'_>) -> starlark::Result<FrozenModule> {
    evaluate_bzl_with(input, true)
}

fn evaluate_bzl_with(input: &BzlFile<'_>, builtins: bool) -> starlark::Result<FrozenModule> {
    let _span = tracing::debug_span!("evaluate_bzl", file = %bzl_name(input.file)).entered();
    let ast = {
        let _span = tracing::debug_span!("parse", file = %bzl_name(input.file)).entered();
        parse(&bzl_name(input.file), input.source, FileKind::Bzl)
            .map_err(starlark::Error::new_other)?
    };
    let env = BzlEval {
        mappings: input.mappings,
        assigned: assigned_names(&ast),
        loading: true,
        extension: None,
        recorder: None,
        file: Some(input.file.clone()),
    };
    Module::with_temp_heap(|module| {
        if builtins {
            let provided = builtins_module();
            for name in provided.names() {
                if let Ok(owned) = provided.get(&name)
                    && let Some(value) = owned.value().unpack_frozen()
                {
                    module.frozen_heap().add_reference(owned.owner());
                    module.set(&name, value.to_value());
                }
            }
        }
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
        .and_then(|e| e.downcast_ref::<BzlEval>())
        .is_some_and(|e| e.loading)
}

/// The `.bzl` file whose code `eval` is running at its top level.
pub(crate) fn evaluating_file(eval: &Evaluator<'_, '_, '_>) -> Option<Label> {
    eval.extra
        .and_then(|e| e.downcast_ref::<BzlEval>())
        .and_then(|e| e.file.clone())
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

/// The repo name a label written as `text` leaves to the mapping: `dep` in
/// `@dep//a:b` or `@dep`.
pub(crate) fn apparent_repo(text: &str) -> Option<&str> {
    let rest = text.strip_prefix('@')?;
    if rest.starts_with('@') {
        return None;
    }
    Some(rest.split_once("//").map_or(rest, |(repo, _)| repo))
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

impl StarlarkLabel {
    /// The label as `@@repo//package:name`, which names it from anywhere.
    pub(crate) fn canonical(&self) -> String {
        format!("@@{}//{}:{}", self.repo, self.package, self.name)
    }

    pub(crate) fn into_label(self) -> Label {
        Label {
            repo: self.repo,
            package: self.package,
            name: self.name,
        }
    }
}

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
        if let Some(apparent) = bound[0]
            .and_then(|v| v.unpack_str())
            .and_then(apparent_repo)
        {
            let recorder = eval
                .extra
                .and_then(|e| e.downcast_ref::<BzlEval>())
                .and_then(|env| env.recorder.as_ref());
            match recorder {
                Some(recorder) => recorder.push(crate::repo_ctx::RecordedInput::RepoMapping {
                    repo: file.repo.clone(),
                    apparent: apparent.to_owned(),
                    canonical: mappings
                        .find_apparent(&file.repo, apparent)
                        .unwrap_or_else(|| "\\0".to_owned()),
                }),
                None => mappings.note_lookup(&file, apparent),
            }
        }
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
