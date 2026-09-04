//! Evaluating a `MODULE.bazel` file.
//!
//! A module file is Starlark, but a very restricted dialect of it: no
//! `def`, no `load()`, no top-level `if`/`for`, no `lambda`. Bazel
//! enforces this with `DotBazelFileSyntaxChecker` before compiling
//! (`CompiledModuleFile.parseAndCompile`); fjfj gets the same restriction
//! from the parser by turning those features off in the [`Dialect`], which
//! means a rejected file is rejected at parse time with a location, as in
//! Bazel.
//!
//! The directives do not compute anything — they *record*. Evaluation
//! produces a [`ModuleFile`]: one [`Module`], plus the overrides declared
//! in it. Whether those overrides are honoured is not decided here (see
//! [`crate::overrides`]); whether the deps exist is not decided here
//! either (see [`crate::discovery`]).

use std::cell::RefCell;
use std::rc::Rc;

use allocative::Allocative;
use starlark::collections::SmallMap;
use starlark::environment::{GlobalsBuilder, LibraryExtension, Module as StarlarkModuleEnv};
use starlark::eval::{Arguments, Evaluator};
use starlark::syntax::{AstModule, Dialect};
use starlark::values::list_or_tuple::UnpackListOrTuple;
use starlark::values::none::NoneType;
use starlark::values::tuple::UnpackTuple;
use starlark::values::{
    Heap, NoSerialize, ProvidesStaticType, StarlarkPagable, StarlarkValue, UnpackValue, Value,
    ValueLike,
};
use starlark::{starlark_module, starlark_simple_value};
use starlark_derive::starlark_value;

use crate::attrs::{AttrValue, Attrs};
use crate::error::{BzlmodError, Result};
use crate::module::{
    Dep, DepSpec, ExtensionUsage, Module, ModuleKey, RepoOverride, Tag, validate_module_name,
};
use crate::overrides::{
    ModuleOverride, MultipleVersionOverride, NonRegistryOverride, RepoRule, RepoSpec,
    SingleVersionOverride,
};
use crate::version::Version;

/// Bazel's dialect for `MODULE.bazel`.
///
/// Every switch here is a restriction Bazel imposes, not a preference:
/// `def`/`lambda` because a module file must be static enough to be
/// analysed without running arbitrary code; `load` because a module file
/// may only pull in other files through `include()`, which is checked
/// syntactically; top-level `if`/`for` because Bazel's `.bazel` files have
/// never allowed control flow outside a function.
pub fn module_dialect() -> Dialect {
    Dialect {
        enable_def: false,
        enable_lambda: false,
        enable_load: false,
        enable_top_level_stmt: false,
        ..Dialect::Standard
    }
}

/// The globals a module file evaluates against — built fresh each time
/// rather than shared, since an `include()`d file needs the exact same set
/// for its own nested `eval_module` call and `Evaluator` doesn't hand back
/// the `Globals` it was started with.
fn module_file_globals_instance() -> starlark::environment::Globals {
    GlobalsBuilder::extended_by(&[LibraryExtension::Print])
        .with(module_file_globals)
        .build()
}

/// The result of evaluating one module file.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleFile {
    pub module: Module,
    /// Overrides declared in this file, in declaration order. Only the
    /// root module's are ever applied.
    pub overrides: Vec<(String, ModuleOverride)>,
    /// `include()` labels found in the file. Resolving them is the
    /// caller's job, since it needs the filesystem.
    pub includes: Vec<String>,
    /// Non-fatal diagnostics, e.g. a no-op `compatibility_level`.
    pub warnings: Vec<String>,
}

/// Supplies the source text `include()` names, so eval.rs never has to know
/// about the filesystem: the caller resolves a label to a file however it
/// resolves any other module file (a workspace path for the root, an
/// extracted repo's path for a non-registry override).
pub trait IncludeSource: std::fmt::Debug {
    /// `label` has already passed [`validate_include_label`]: it starts
    /// with `//`, and its basename ends in `.MODULE.bazel` without
    /// starting with a dot.
    fn read(&self, label: &str) -> Result<String>;
}

/// How to evaluate a module file.
#[derive(Debug, Clone)]
pub struct EvalOptions {
    /// The key this file is expected to describe. The root module's key is
    /// [`ModuleKey::root`]; a registry module's key is the one that was
    /// requested, and a mismatch with the file's own `module()` call is an
    /// error.
    pub key: ModuleKey,
    /// Drop `dev_dependency = True` deps and ignore every override.
    ///
    /// Bazel sets this for **every non-root module**, and it is the single
    /// mechanism behind two separately documented rules: dev deps of a
    /// dependency don't affect your build, and a dependency's overrides
    /// are ignored. Both fall out of the same flag
    /// (`ModuleThreadContext.addOverride` returns early when it is set).
    pub ignore_dev_deps: bool,
    /// Modules every module implicitly depends on. Bazel's is
    /// `{bazel_tools}`; a module never gets an implicit dep on itself.
    pub builtin_modules: Vec<String>,
    /// Whether `include()` is allowed in this file at all — Bazel allows it
    /// only in the root module and in modules with a non-registry override;
    /// a registry module using it is an error (buildfiji-mum.22).
    pub allow_include: bool,
    /// How to fetch an `include()`d file's text. `None` with
    /// `allow_include: true` still lets the file *validate* its `include()`
    /// calls, but resolving one is then a "not configured" error rather
    /// than an unresolved-label refusal.
    pub include_source: Option<Rc<dyn IncludeSource>>,
}

impl EvalOptions {
    /// Options for the root module: its own dev deps and overrides count,
    /// and it may use `include()`.
    pub fn root() -> EvalOptions {
        EvalOptions {
            key: ModuleKey::root(),
            ignore_dev_deps: false,
            builtin_modules: vec!["bazel_tools".to_owned()],
            allow_include: true,
            include_source: None,
        }
    }

    /// Options for a module fetched from a registry. `include()` is
    /// refused there regardless of `include_source` — call
    /// [`Self::with_include_source`] for a non-registry override instead.
    pub fn dependency(key: ModuleKey) -> EvalOptions {
        EvalOptions {
            key,
            ignore_dev_deps: true,
            builtin_modules: vec!["bazel_tools".to_owned()],
            allow_include: false,
            include_source: None,
        }
    }

    /// Lets this file use `include()`, resolved through `source`. For
    /// [`Self::dependency`], only a module with a non-registry override
    /// should get this — a registry module using `include()` is an error.
    pub fn with_include_source(mut self, source: Rc<dyn IncludeSource>) -> EvalOptions {
        self.allow_include = true;
        self.include_source = Some(source);
        self
    }
}

/// Bazel's rule for an `include()` argument: repo-relative (starts with
/// `//`), and the basename is a real `.MODULE.bazel` segment file — not a
/// hidden file, and not the root `MODULE.bazel` itself (which would make
/// the include recursive by definition, though the cycle check catches
/// that too).
pub fn validate_include_label(label: &str) -> starlark::Result<()> {
    let Some(rest) = label.strip_prefix("//") else {
        return Err(err(format!(
            "include() label must be repo-relative (start with '//'): {label}"
        )));
    };
    let (package, target) = rest.split_once(':').unwrap_or((rest, ""));
    if target.is_empty() {
        return Err(err(format!(
            "include() label must name a target after ':': {label}"
        )));
    }
    if !package.is_empty() {
        fjfj_graph::label::validate_package_name(package)
            .map_err(|e| err(format!("include() label {label}: {e}")))?;
    }
    fjfj_graph::label::validate_target_name(target)
        .map_err(|e| err(format!("include() label {label}: {e}")))?;
    // "basename" here is the target's own last path segment: a target name
    // may itself contain '/' (a file nested under the package directory).
    let basename = target.rsplit('/').next().unwrap_or(target);
    if basename.starts_with('.') {
        return Err(err(format!(
            "the file referenced by include() must not start with '.': {label}"
        )));
    }
    if !basename.ends_with(".MODULE.bazel") {
        return Err(err(format!(
            "the file referenced by include() must be named *.MODULE.bazel: {label}"
        )));
    }
    Ok(())
}

/// Parses and evaluates a module file.
pub fn eval_module_file(filename: &str, source: &str, options: &EvalOptions) -> Result<ModuleFile> {
    let ast = AstModule::parse(filename, source.to_owned(), &module_dialect()).map_err(|e| {
        BzlmodError::BadModule {
            key: options.key.to_string(),
            message: e.into_anyhow().to_string(),
        }
    })?;

    let ctx = ModuleContext::new(options.clone());
    // `print` is a Bazel builtin but a starlark-crate *extension*, and
    // module files in the wild use it (bazel_gazelle prints a warning from
    // its MODULE.bazel). Its output goes nowhere for a dependency; see
    // below.
    let globals = module_file_globals_instance();
    StarlarkModuleEnv::with_temp_heap(|env| {
        let mut eval = Evaluator::new(&env);
        eval.extra = Some(&ctx);
        // A dependency's module file must not be able to write to the
        // console during resolution; Bazel makes `print` a no-op for
        // exactly this reason (`printIsNoop` in ModuleFileFunction).
        if options.ignore_dev_deps {
            eval.set_print_handler(&DISCARD_PRINT);
        }
        eval.eval_module(ast, &globals)
            .map_err(|e| BzlmodError::BadModule {
                key: options.key.to_string(),
                message: e.into_anyhow().to_string(),
            })?;
        Ok::<(), BzlmodError>(())
    })?;

    ctx.finish()
}

struct DiscardPrint;

impl starlark::PrintHandler for DiscardPrint {
    fn println(&self, _text: &str) -> starlark::Result<()> {
        Ok(())
    }
}

static DISCARD_PRINT: DiscardPrint = DiscardPrint;

/// The state a module file's directives accumulate into.
///
/// The `RefCell` is not a design choice: `Evaluator::extra` hands the
/// directives a shared reference, so the only way to record anything is
/// interior mutability. Nothing escapes — [`ModuleContext::finish`]
/// consumes it into plain immutable data.
#[derive(Debug, ProvidesStaticType)]
struct ModuleContext {
    options: EvalOptions,
    state: RefCell<ModuleState>,
    /// Labels currently being `include()`d, innermost last — Bazel's own
    /// `ModuleFileFunction` doesn't need this (Skyframe's cycle detector
    /// catches it for them), but a naive recursive `eval_module` call here
    /// would just overflow the stack on a self-include.
    include_stack: RefCell<Vec<String>>,
}

#[derive(Debug, Default)]
struct ModuleState {
    module_called: bool,
    had_non_module_call: bool,
    name: String,
    version: Version,
    repo_name: Option<String>,
    bazel_compatibility: Vec<String>,
    deps: Vec<Dep>,
    nodep_deps: Vec<DepSpec>,
    toolchains: Vec<String>,
    execution_platforms: Vec<String>,
    extension_usages: Vec<ExtensionUsage>,
    flag_aliases: Vec<(String, String)>,
    overrides: Vec<(String, ModuleOverride)>,
    includes: Vec<String>,
    warnings: Vec<String>,
    /// Every repo name this file has claimed, and how, so a collision can
    /// name both sides — Bazel's `repoNameUsages`.
    repo_name_usages: Vec<(String, String)>,
}

impl ModuleContext {
    fn new(options: EvalOptions) -> ModuleContext {
        let version = options.key.version.clone();
        ModuleContext {
            options,
            state: RefCell::new(ModuleState {
                version,
                ..ModuleState::default()
            }),
            include_stack: RefCell::new(Vec::new()),
        }
    }

    fn from_eval<'a>(eval: &Evaluator<'_, 'a, '_>) -> Result<&'a ModuleContext> {
        eval.extra
            .and_then(|extra| extra.downcast_ref::<ModuleContext>())
            .ok_or(BzlmodError::NotAModuleFile)
    }

    fn set_non_module_called(&self) {
        self.state.borrow_mut().had_non_module_call = true;
    }

    fn add_repo_name_usage(&self, repo_name: &str, how: &str) -> starlark::Result<()> {
        let mut state = self.state.borrow_mut();
        if let Some((_, existing_how)) = state
            .repo_name_usages
            .iter()
            .find(|(name, _)| name == repo_name)
        {
            return Err(err(format!(
                "The repo name '{repo_name}' cannot be defined {how} as it is already defined \
                 {existing_how}"
            )));
        }
        state
            .repo_name_usages
            .push((repo_name.to_owned(), how.to_owned()));
        Ok(())
    }

    /// Records an override, unless this module's overrides are ignored.
    fn add_override(&self, module_name: &str, value: ModuleOverride) -> starlark::Result<()> {
        if self.options.ignore_dev_deps {
            return Ok(());
        }
        let mut state = self.state.borrow_mut();
        if state.overrides.iter().any(|(name, _)| name == module_name) {
            return Err(err(format!(
                "multiple overrides for dep {module_name} found"
            )));
        }
        state.overrides.push((module_name.to_owned(), value));
        Ok(())
    }

    /// Finds the non-isolated usage of an extension, creating it if this
    /// is the first mention. Isolated usages never share, so they are
    /// created directly by `use_extension`.
    fn get_or_create_extension_usage(&self, bzl_file: &str, extension_name: &str) -> usize {
        let mut state = self.state.borrow_mut();
        if let Some(index) = state.extension_usages.iter().position(|u| {
            u.bzl_file == bzl_file && u.extension_name == extension_name && !u.isolate
        }) {
            return index;
        }
        state.extension_usages.push(ExtensionUsage {
            bzl_file: bzl_file.to_owned(),
            extension_name: extension_name.to_owned(),
            isolate: false,
            dev_dependency: false,
            imports: Vec::new(),
            tags: Vec::new(),
            repo_overrides: Vec::new(),
        });
        state.extension_usages.len() - 1
    }

    fn finish(self) -> Result<ModuleFile> {
        let mut state = self.state.into_inner();

        // Every module implicitly depends on the built-in modules, at the
        // empty version — they are supplied by fjfj, not by a registry.
        for builtin in &self.options.builtin_modules {
            if self.options.key.name == *builtin {
                continue;
            }
            if state
                .repo_name_usages
                .iter()
                .any(|(name, _)| name == builtin)
            {
                return Err(BzlmodError::BadModule {
                    key: self.options.key.to_string(),
                    message: format!(
                        "the repo name '{builtin}' is a built-in dependency and cannot be used by \
                         any 'bazel_dep' or 'use_repo' directive"
                    ),
                });
            }
            state.deps.push(Dep {
                repo_name: builtin.clone(),
                spec: DepSpec::new(builtin.clone(), Version::EMPTY),
            });
        }

        let repo_name = state.repo_name.unwrap_or_else(|| state.name.clone());
        let module = Module {
            key: self.options.key.clone(),
            name: state.name,
            version: state.version,
            repo_name,
            deps: state.deps,
            nodep_deps: state.nodep_deps,
            registry: None,
            bazel_compatibility: state.bazel_compatibility,
            toolchains_to_register: state.toolchains,
            execution_platforms_to_register: state.execution_platforms,
            extension_usages: state.extension_usages,
            flag_aliases: state.flag_aliases,
        };

        // A registry hands out a module file under a (name, version) it
        // chose; if the file disagrees, the registry is lying about what
        // it served and every downstream repo name would be wrong.
        if !self.options.key.is_root() {
            if module.name != self.options.key.name {
                return Err(BzlmodError::BadModule {
                    key: self.options.key.to_string(),
                    message: format!("declares a different name ({})", module.name),
                });
            }
            if !self.options.key.version.is_empty() && module.version != self.options.key.version {
                return Err(BzlmodError::BadModule {
                    key: self.options.key.to_string(),
                    message: format!("declares a different version ({})", module.version),
                });
            }
        }

        Ok(ModuleFile {
            module,
            overrides: state.overrides,
            includes: state.includes,
            warnings: state.warnings,
        })
    }
}

fn err(message: impl Into<String>) -> starlark::Error {
    starlark::Error::new_other(anyhow::anyhow!(message.into()))
}

fn parse_version(where_: &str, version: &str) -> starlark::Result<Version> {
    Version::parse(version).map_err(|e| err(format!("Invalid version in {where_}(): {e}")))
}

fn check_module_name(name: &str) -> starlark::Result<()> {
    validate_module_name(name).map_err(|e| err(e.to_string()))
}

fn check_user_repo_name(name: &str) -> starlark::Result<()> {
    fjfj_graph::label::validate_user_provided_repo_name(name).map_err(|e| err(e.to_string()))
}

/// `register_toolchains` and `register_execution_platforms` take target
/// patterns, and Bazel insists they be absolute: a relative pattern would
/// resolve against whichever module happened to be resolving, which is
/// never what the author meant.
fn check_absolute_patterns(where_: &str, patterns: &[String]) -> starlark::Result<()> {
    for pattern in patterns {
        if !pattern.starts_with("//") && !pattern.starts_with('@') {
            return Err(err(format!(
                "Expected absolute target patterns (must begin with '//' or '@') for '{where_}' \
                 argument, but got '{pattern}' as an argument"
            )));
        }
    }
    Ok(())
}

/// Bazel's `VALID_BAZEL_COMPATIBILITY_VERSION`: a comparison operator
/// followed by an `X.Y.Z` release.
fn check_bazel_compatibility(values: &[String]) -> starlark::Result<()> {
    for value in values {
        let rest = value
            .strip_prefix("<=")
            .or_else(|| value.strip_prefix(">="))
            .or_else(|| value.strip_prefix('<'))
            .or_else(|| value.strip_prefix('>'))
            .or_else(|| value.strip_prefix('-'));
        let ok = rest.is_some_and(|rest| {
            let segments: Vec<&str> = rest.split('.').collect();
            segments.len() == 3
                && segments
                    .iter()
                    .all(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        });
        if !ok {
            return Err(err(format!(
                "invalid version argument '{value}': valid argument must 1) start with \
                 (<,<=,>,>=,-); 2) contain a version number in form of X.X.X where X is a number"
            )));
        }
    }
    Ok(())
}

fn attrs_from_kwargs(kwargs: SmallMap<String, Value<'_>>) -> starlark::Result<Attrs> {
    kwargs
        .into_iter()
        .map(|(k, v)| Ok((k, AttrValue::from_value(v).map_err(|e| err(e.to_string()))?)))
        .collect()
}

/// The proxy `use_extension` returns.
///
/// It carries only an index into the context's usage list: the tags it
/// collects have to end up in the module, not in the Starlark heap, and an
/// index is the one thing that can cross that boundary without borrowing
/// the evaluator.
#[derive(Debug, PartialEq, ProvidesStaticType, NoSerialize, StarlarkPagable, Allocative)]
struct ExtensionProxy {
    usage_index: usize,
    dev_dependency: bool,
}
starlark_simple_value!(ExtensionProxy);

impl std::fmt::Display for ExtensionProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<module extension proxy>")
    }
}

#[starlark_value(type = "module_extension_proxy")]
impl<'v> StarlarkValue<'v> for ExtensionProxy {
    fn get_attr(&self, attribute: &str, heap: Heap<'v>) -> Option<Value<'v>> {
        // Any attribute is a tag class: the extension defines them and
        // fjfj has not loaded the extension yet, so the name is taken on
        // trust here and checked when the extension runs
        // (buildfiji-mum.8).
        Some(heap.alloc(TagCallable {
            usage_index: self.usage_index,
            tag_class: attribute.to_owned(),
            dev_dependency: self.dev_dependency,
        }))
    }

    fn has_attr(&self, _attribute: &str, _heap: Heap<'v>) -> bool {
        true
    }
}

/// `ext.tag_class` — calling it records one tag.
#[derive(Debug, PartialEq, ProvidesStaticType, NoSerialize, StarlarkPagable, Allocative)]
struct TagCallable {
    usage_index: usize,
    tag_class: String,
    dev_dependency: bool,
}
starlark_simple_value!(TagCallable);

impl std::fmt::Display for TagCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<tag class {}>", self.tag_class)
    }
}

#[starlark_value(type = "tag_callable")]
impl<'v> StarlarkValue<'v> for TagCallable {
    fn invoke(
        &self,
        _me: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        args.no_positional_args(eval.heap())?;
        let mut attrs = Attrs::new();
        for (name, value) in args.names_map()? {
            attrs.push((
                name.as_str().to_owned(),
                AttrValue::from_value(value).map_err(|e| err(e.to_string()))?,
            ));
        }
        let ctx = ModuleContext::from_eval(eval).map_err(|e| err(e.to_string()))?;
        ctx.set_non_module_called();
        ctx.state.borrow_mut().extension_usages[self.usage_index]
            .tags
            .push(Tag {
                tag_class: self.tag_class.clone(),
                attrs,
                dev_dependency: self.dev_dependency,
            });
        Ok(Value::new_none())
    }
}

/// The file a `use_repo_rule` repo is attributed to. Bazel uses the module
/// file itself, since the rule call is written there rather than in an
/// extension.
const INNATE_EXTENSION_FILE: &str = "//:MODULE.bazel";

/// The callable `use_repo_rule` returns.
#[derive(Debug, PartialEq, ProvidesStaticType, NoSerialize, StarlarkPagable, Allocative)]
struct RepoRuleProxy {
    usage_index: usize,
}
starlark_simple_value!(RepoRuleProxy);

impl std::fmt::Display for RepoRuleProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<repo rule proxy>")
    }
}

#[starlark_value(type = "repo_rule_proxy")]
impl<'v> StarlarkValue<'v> for RepoRuleProxy {
    fn invoke(
        &self,
        _me: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        args.no_positional_args(eval.heap())?;
        let mut attrs = Attrs::new();
        let mut repo_name = None;
        let mut dev_dependency = false;
        for (name, value) in args.names_map()? {
            match name.as_str() {
                "name" => {
                    let name = <&str>::unpack_value(value)
                        .map_err(|_| err("use_repo_rule() name must be a string"))?
                        .ok_or_else(|| err("use_repo_rule() name must be a string"))?;
                    check_user_repo_name(name)?;
                    repo_name = Some(name.to_owned());
                    attrs.push(("name".to_owned(), AttrValue::String(name.to_owned())));
                }
                "dev_dependency" => {
                    dev_dependency = value.to_bool();
                }
                _ => attrs.push((
                    name.as_str().to_owned(),
                    AttrValue::from_value(value).map_err(|e| err(e.to_string()))?,
                )),
            }
        }
        let repo_name = repo_name.ok_or_else(|| err("use_repo_rule() requires a name"))?;

        let ctx = ModuleContext::from_eval(eval).map_err(|e| err(e.to_string()))?;
        ctx.set_non_module_called();
        if ctx.options.ignore_dev_deps && dev_dependency {
            return Ok(Value::new_none());
        }
        ctx.add_repo_name_usage(&repo_name, "by a repo rule")?;
        let mut state = ctx.state.borrow_mut();
        let usage = &mut state.extension_usages[self.usage_index];
        usage.tags.push(Tag {
            tag_class: "repo".to_owned(),
            attrs,
            dev_dependency,
        });
        usage.imports.push((repo_name.clone(), repo_name));
        Ok(Value::new_none())
    }
}

/// Shared by `override_repo` and `inject_repo`, which differ only in
/// whether the extension must already generate the repo being replaced.
fn add_repo_overrides<'v>(
    extension_proxy: Value<'v>,
    args: UnpackTuple<String>,
    kwargs: SmallMap<String, String>,
    must_exist: bool,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<NoneType> {
    let ctx = ModuleContext::from_eval(eval).map_err(|e| err(e.to_string()))?;
    ctx.set_non_module_called();
    // These name repos of the calling module, which for a dependency may
    // be dev-only, so Bazel drops the call rather than resolving it.
    if ctx.options.ignore_dev_deps {
        return Ok(NoneType);
    }
    let proxy = extension_proxy
        .downcast_ref::<ExtensionProxy>()
        .ok_or_else(|| err("first argument must be a module extension proxy"))?;
    let overrides = args
        .items
        .into_iter()
        .map(|name| (name.clone(), name))
        .chain(kwargs)
        .map(|(overridden, overriding)| RepoOverride {
            overridden_repo_name: overridden,
            overriding_repo_name: overriding,
            must_exist,
        });
    ctx.state.borrow_mut().extension_usages[proxy.usage_index]
        .repo_overrides
        .extend(overrides);
    Ok(NoneType)
}

#[starlark_module]
fn module_file_globals(builder: &mut GlobalsBuilder) {
    /// Declares this module's own identity.
    fn module(
        #[starlark(require = named, default = "")] name: &str,
        #[starlark(require = named, default = "")] version: &str,
        #[starlark(require = named, default = -1)] compatibility_level: i32,
        #[starlark(require = named, default = "")] repo_name: &str,
        #[starlark(require = named)] bazel_compatibility: Option<UnpackListOrTuple<String>>,
        eval: &mut Evaluator,
    ) -> starlark::Result<NoneType> {
        let ctx = ModuleContext::from_eval(eval).map_err(|e| err(e.to_string()))?;
        {
            let state = ctx.state.borrow();
            if state.module_called {
                return Err(err("the module() directive can only be called once"));
            }
            if state.had_non_module_call {
                return Err(err(
                    "if module() is called, it must be called before any other functions",
                ));
            }
        }
        if !name.is_empty() {
            check_module_name(name)?;
        }
        let bazel_compatibility: Vec<String> =
            bazel_compatibility.map(|v| v.items).unwrap_or_default();
        check_bazel_compatibility(&bazel_compatibility)?;
        let version = parse_version("module", version)?;

        let repo_name = if repo_name.is_empty() {
            ctx.add_repo_name_usage(name, "as the current module name")?;
            None
        } else {
            check_user_repo_name(repo_name)?;
            ctx.add_repo_name_usage(repo_name, "as the module's own repo name")?;
            Some(repo_name.to_owned())
        };

        let mut state = ctx.state.borrow_mut();
        state.module_called = true;
        state.name = name.to_owned();
        state.version = version;
        state.repo_name = repo_name;
        state.bazel_compatibility = bazel_compatibility;
        if compatibility_level != -1 && ctx.options.key.is_root() {
            state.warnings.push(
                "The attribute 'compatibility_level' in module() is a no-op and will be removed \
                 in a future Bazel release. Please remove it from your MODULE.bazel file."
                    .to_owned(),
            );
        }
        Ok(NoneType)
    }

    /// Declares a direct dependency on another Bazel module.
    fn bazel_dep<'v>(
        #[starlark(require = named)] name: &str,
        #[starlark(require = named, default = "")] version: &str,
        #[starlark(require = named, default = -1)] max_compatibility_level: i32,
        #[starlark(require = named)] repo_name: Option<Value<'v>>,
        #[starlark(require = named, default = false)] dev_dependency: bool,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let ctx = ModuleContext::from_eval(eval).map_err(|e| err(e.to_string()))?;
        ctx.set_non_module_called();
        check_module_name(name)?;
        let version = parse_version("bazel_dep", version)?;
        if max_compatibility_level != -1 && ctx.options.key.is_root() {
            ctx.state.borrow_mut().warnings.push(
                "The attribute 'max_compatibility_level' in bazel_dep() is a no-op and will be \
                 removed in a future Bazel release. Please remove it from your MODULE.bazel file."
                    .to_owned(),
            );
        }

        // `repo_name = None` is not "use the default"; it makes this a
        // *nodep* edge, honoured only if the module is in the graph
        // anyway. The default is the empty string, which does mean
        // "use the module name".
        let repo_name = match repo_name {
            None => Some(name.to_owned()),
            Some(v) if v.is_none() => None,
            Some(v) => {
                let s = <&str>::unpack_value(v)
                    .map_err(|_| err("bazel_dep() repo_name must be a string or None"))?
                    .ok_or_else(|| err("bazel_dep() repo_name must be a string or None"))?;
                if s.is_empty() {
                    Some(name.to_owned())
                } else {
                    check_user_repo_name(s)?;
                    Some(s.to_owned())
                }
            }
        };

        if !(ctx.options.ignore_dev_deps && dev_dependency) {
            let spec = DepSpec::new(name, version);
            let mut state = ctx.state.borrow_mut();
            match &repo_name {
                Some(repo_name) => state.deps.push(Dep {
                    repo_name: repo_name.clone(),
                    spec,
                }),
                None => state.nodep_deps.push(spec),
            }
        }
        if let Some(repo_name) = &repo_name {
            ctx.add_repo_name_usage(repo_name, "by a bazel_dep")?;
        }
        Ok(NoneType)
    }

    /// Registers toolchains defined by this module or its deps.
    fn register_toolchains(
        #[starlark(args)] toolchain_labels: UnpackTuple<String>,
        #[starlark(require = named, default = false)] dev_dependency: bool,
        eval: &mut Evaluator,
    ) -> starlark::Result<NoneType> {
        let ctx = ModuleContext::from_eval(eval).map_err(|e| err(e.to_string()))?;
        ctx.set_non_module_called();
        let labels: Vec<String> = toolchain_labels.items;
        check_absolute_patterns("register_toolchains", &labels)?;
        if !(ctx.options.ignore_dev_deps && dev_dependency) {
            ctx.state.borrow_mut().toolchains.extend(labels);
        }
        Ok(NoneType)
    }

    /// Registers execution platforms defined by this module or its deps.
    fn register_execution_platforms(
        #[starlark(args)] platform_labels: UnpackTuple<String>,
        #[starlark(require = named, default = false)] dev_dependency: bool,
        eval: &mut Evaluator,
    ) -> starlark::Result<NoneType> {
        let ctx = ModuleContext::from_eval(eval).map_err(|e| err(e.to_string()))?;
        ctx.set_non_module_called();
        let labels: Vec<String> = platform_labels.items;
        check_absolute_patterns("register_execution_platforms", &labels)?;
        if !(ctx.options.ignore_dev_deps && dev_dependency) {
            ctx.state.borrow_mut().execution_platforms.extend(labels);
        }
        Ok(NoneType)
    }

    /// Returns a proxy for a module extension, to hang tags off and to
    /// import repos from with `use_repo`.
    fn use_extension<'v>(
        #[starlark(require = pos)] extension_bzl_file: &str,
        #[starlark(require = pos)] extension_name: &str,
        #[starlark(require = named, default = false)] dev_dependency: bool,
        #[starlark(require = named, default = false)] isolate: bool,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let ctx = ModuleContext::from_eval(eval).map_err(|e| err(e.to_string()))?;
        ctx.set_non_module_called();
        if !is_identifier(extension_name) {
            return Err(err(format!(
                "extension name is not a valid identifier: {extension_name}"
            )));
        }
        let usage_index = {
            let mut state = ctx.state.borrow_mut();
            // Non-isolated usages of the same extension share one row, so
            // that a module's tags reach the extension in one batch; an
            // isolated usage always gets its own.
            let existing = if isolate {
                None
            } else {
                state.extension_usages.iter().position(|u| {
                    u.bzl_file == extension_bzl_file
                        && u.extension_name == extension_name
                        && !u.isolate
                        && u.dev_dependency == dev_dependency
                })
            };
            match existing {
                Some(index) => index,
                None => {
                    state.extension_usages.push(ExtensionUsage {
                        bzl_file: extension_bzl_file.to_owned(),
                        extension_name: extension_name.to_owned(),
                        isolate,
                        dev_dependency,
                        imports: Vec::new(),
                        tags: Vec::new(),
                        repo_overrides: Vec::new(),
                    });
                    state.extension_usages.len() - 1
                }
            }
        };
        Ok(eval.heap().alloc(ExtensionProxy {
            usage_index,
            dev_dependency,
        }))
    }

    /// Imports repos generated by an extension into this module's scope.
    fn use_repo<'v>(
        #[starlark(require = pos)] extension_proxy: Value<'v>,
        #[starlark(args)] args: UnpackTuple<String>,
        #[starlark(kwargs)] kwargs: SmallMap<String, String>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let ctx = ModuleContext::from_eval(eval).map_err(|e| err(e.to_string()))?;
        ctx.set_non_module_called();
        let proxy = extension_proxy
            .downcast_ref::<ExtensionProxy>()
            .ok_or_else(|| err("use_repo() first argument must be a module extension proxy"))?;

        // `use_repo(ext, "a")` imports `a` as `a`; `use_repo(ext, b = "a")`
        // imports the extension's `a` under the local name `b`.
        // A kwarg's value may template the importing module's own
        // identity, so that a module can import a repo an extension named
        // after it (`use_repo(ext, deps = "{name}_{version}_deps")`).
        let (module_name, module_version) = {
            let state = ctx.state.borrow();
            (state.name.clone(), state.version.to_string())
        };
        let imports: Vec<(String, String)> = args
            .items
            .into_iter()
            .map(|name| (name.clone(), name))
            .chain(kwargs.into_iter().map(|(local, exported)| {
                let exported = exported
                    .replace("{name}", &module_name)
                    .replace("{version}", &module_version);
                (local, exported)
            }))
            .collect();
        for (local_name, _) in &imports {
            check_user_repo_name(local_name)?;
            ctx.add_repo_name_usage(local_name, "by a use_repo() call")?;
        }
        ctx.state.borrow_mut().extension_usages[proxy.usage_index]
            .imports
            .extend(imports);
        Ok(NoneType)
    }

    /// Replaces a repo an extension generates with one this module
    /// already has. The extension must generate it.
    fn override_repo<'v>(
        #[starlark(require = pos)] extension_proxy: Value<'v>,
        #[starlark(args)] args: UnpackTuple<String>,
        #[starlark(kwargs)] kwargs: SmallMap<String, String>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        add_repo_overrides(extension_proxy, args, kwargs, true, eval)
    }

    /// Adds a repo to what an extension sees, without requiring the
    /// extension to generate one by that name.
    fn inject_repo<'v>(
        #[starlark(require = pos)] extension_proxy: Value<'v>,
        #[starlark(args)] args: UnpackTuple<String>,
        #[starlark(kwargs)] kwargs: SmallMap<String, String>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        add_repo_overrides(extension_proxy, args, kwargs, false, eval)
    }

    /// Returns a callable that instantiates a repository rule directly,
    /// without writing an extension for it.
    fn use_repo_rule<'v>(
        #[starlark(require = pos)] repo_rule_bzl_file: &str,
        #[starlark(require = pos)] repo_rule_name: &str,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let ctx = ModuleContext::from_eval(eval).map_err(|e| err(e.to_string()))?;
        ctx.set_non_module_called();
        // Bazel models this as an "innate" extension of the module file
        // itself, named so that it cannot collide with a real one: the
        // name is not a valid Starlark identifier.
        let extension_name = format!("{repo_rule_bzl_file} {repo_rule_name}");
        let usage_index = ctx.get_or_create_extension_usage(INNATE_EXTENSION_FILE, &extension_name);
        Ok(eval.heap().alloc(RepoRuleProxy { usage_index }))
    }

    /// Gives a Starlark build setting a short command-line name.
    fn flag_alias(
        #[starlark(require = named)] name: &str,
        #[starlark(require = named)] starlark_flag: &str,
        eval: &mut Evaluator,
    ) -> starlark::Result<NoneType> {
        let ctx = ModuleContext::from_eval(eval).map_err(|e| err(e.to_string()))?;
        ctx.set_non_module_called();
        ctx.state
            .borrow_mut()
            .flag_aliases
            .push((name.to_owned(), starlark_flag.to_owned()));
        Ok(NoneType)
    }

    /// Executes another module file segment inline, in this same
    /// evaluation (Bazel's `ModuleFileFunction.execModuleFile`): the
    /// included file's directives land in the same accumulating
    /// [`ModuleState`] as if they had been written at the call site, not
    /// merged in afterwards. Allowed only in the root module and in a
    /// module with a non-registry override — [`EvalOptions::allow_include`]
    /// is what the caller decided that to be.
    fn include(
        #[starlark(require = pos)] label: &str,
        eval: &mut Evaluator,
    ) -> starlark::Result<NoneType> {
        let ctx = ModuleContext::from_eval(eval).map_err(|e| err(e.to_string()))?;
        ctx.set_non_module_called();
        validate_include_label(label)?;
        if !ctx.options.allow_include {
            return Err(err(format!(
                "include() is only allowed in the root module or a module with a non-registry \
                 override, not in {}: {label}",
                ctx.options.key
            )));
        }
        let source = ctx.options.include_source.as_ref().ok_or_else(|| {
            err(format!(
                "include(\"{label}\") could not be resolved: no include source is configured"
            ))
        })?;

        {
            let mut stack = ctx.include_stack.borrow_mut();
            if stack.iter().any(|l| l == label) {
                let mut cycle = stack.clone();
                cycle.push(label.to_owned());
                return Err(err(format!("include() cycle: {}", cycle.join(" -> "))));
            }
            stack.push(label.to_owned());
        }
        // Recorded before resolving, so a failed include still shows up in
        // an error report of what was reached.
        ctx.state.borrow_mut().includes.push(label.to_owned());

        let result = source
            .read(label)
            .map_err(|e| err(e.to_string()))
            .and_then(|text| {
                let ast = AstModule::parse(label, text, &module_dialect())
                    .map_err(|e| err(e.into_anyhow().to_string()))?;
                eval.eval_module(ast, &module_file_globals_instance())
                    .map(|_| ())
            });

        ctx.include_stack.borrow_mut().pop();
        result.map(|()| NoneType)
    }

    /// Pins a dep to one version, and/or redirects its registry, and/or
    /// patches it. Root module only.
    fn single_version_override(
        #[starlark(require = named)] module_name: &str,
        #[starlark(require = named, default = "")] version: &str,
        #[starlark(require = named, default = "")] registry: &str,
        #[starlark(require = named)] patches: Option<UnpackListOrTuple<String>>,
        #[starlark(require = named)] patch_cmds: Option<UnpackListOrTuple<String>>,
        #[starlark(require = named, default = 0)] patch_strip: i32,
        eval: &mut Evaluator,
    ) -> starlark::Result<NoneType> {
        let ctx = ModuleContext::from_eval(eval).map_err(|e| err(e.to_string()))?;
        ctx.set_non_module_called();
        check_module_name(module_name)?;
        let version = parse_version("single_version_override", version)?;
        ctx.add_override(
            module_name,
            ModuleOverride::SingleVersion(SingleVersionOverride {
                version,
                registry: non_empty(registry),
                patches: patches.map(|v| v.items).unwrap_or_default(),
                patch_cmds: patch_cmds.map(|v| v.items).unwrap_or_default(),
                patch_strip,
            }),
        )?;
        Ok(NoneType)
    }

    /// Lets several versions of a dep coexist. Root module only.
    fn multiple_version_override(
        #[starlark(require = named)] module_name: &str,
        #[starlark(require = named)] versions: UnpackListOrTuple<String>,
        #[starlark(require = named, default = "")] registry: &str,
        eval: &mut Evaluator,
    ) -> starlark::Result<NoneType> {
        let ctx = ModuleContext::from_eval(eval).map_err(|e| err(e.to_string()))?;
        ctx.set_non_module_called();
        check_module_name(module_name)?;
        let versions = versions
            .items
            .iter()
            .map(|v| parse_version("multiple_version_override", v))
            .collect::<starlark::Result<Vec<_>>>()?;
        if versions.len() < 2 {
            return Err(err(
                "multiple_version_override() must specify at least 2 versions",
            ));
        }
        ctx.add_override(
            module_name,
            ModuleOverride::MultipleVersion(MultipleVersionOverride {
                versions,
                registry: non_empty(registry),
            }),
        )?;
        Ok(NoneType)
    }

    /// Takes a dep out of the registry and backs it with `http_archive`.
    fn archive_override<'v>(
        #[starlark(require = named)] module_name: &str,
        #[starlark(kwargs)] kwargs: SmallMap<String, Value<'v>>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let ctx = ModuleContext::from_eval(eval).map_err(|e| err(e.to_string()))?;
        ctx.set_non_module_called();
        check_module_name(module_name)?;
        ctx.add_override(
            module_name,
            ModuleOverride::NonRegistry(NonRegistryOverride {
                repo_spec: RepoSpec {
                    rule: RepoRule::HttpArchive,
                    attrs: attrs_from_kwargs(kwargs)?,
                },
            }),
        )?;
        Ok(NoneType)
    }

    /// Takes a dep out of the registry and backs it with `git_repository`.
    fn git_override<'v>(
        #[starlark(require = named)] module_name: &str,
        #[starlark(kwargs)] kwargs: SmallMap<String, Value<'v>>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let ctx = ModuleContext::from_eval(eval).map_err(|e| err(e.to_string()))?;
        ctx.set_non_module_called();
        check_module_name(module_name)?;
        ctx.add_override(
            module_name,
            ModuleOverride::NonRegistry(NonRegistryOverride {
                repo_spec: RepoSpec {
                    rule: RepoRule::GitRepository,
                    attrs: attrs_from_kwargs(kwargs)?,
                },
            }),
        )?;
        Ok(NoneType)
    }

    /// Takes a dep out of the registry and backs it with a local directory.
    fn local_path_override(
        #[starlark(require = named)] module_name: &str,
        #[starlark(require = named)] path: &str,
        eval: &mut Evaluator,
    ) -> starlark::Result<NoneType> {
        let ctx = ModuleContext::from_eval(eval).map_err(|e| err(e.to_string()))?;
        ctx.set_non_module_called();
        check_module_name(module_name)?;
        ctx.add_override(
            module_name,
            ModuleOverride::NonRegistry(NonRegistryOverride {
                repo_spec: RepoSpec {
                    rule: RepoRule::LocalRepository,
                    attrs: vec![("path".to_owned(), AttrValue::String(path.to_owned()))],
                },
            }),
        )?;
        Ok(NoneType)
    }
}

fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_owned())
    }
}

/// Whether a string is a Starlark identifier — what Bazel requires of an
/// extension name, since the extension is a symbol in its `.bzl` file.
fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_root(source: &str) -> Result<ModuleFile> {
        eval_module_file("MODULE.bazel", source, &EvalOptions::root())
    }

    fn eval_root_no_builtins(source: &str) -> Result<ModuleFile> {
        let mut options = EvalOptions::root();
        options.builtin_modules = Vec::new();
        eval_module_file("MODULE.bazel", source, &options)
    }

    #[test]
    fn records_module_and_deps() {
        let file = eval_root_no_builtins(
            r#"
module(name = "my_module", version = "1.0", repo_name = "mine")
bazel_dep(name = "rules_foo", version = "1.2.3")
bazel_dep(name = "rules_bar", version = "0.1", repo_name = "bar")
"#,
        )
        .unwrap();
        assert_eq!(file.module.name, "my_module");
        assert_eq!(file.module.version.as_str(), "1.0");
        assert_eq!(file.module.repo_name, "mine");
        assert_eq!(
            file.module
                .deps
                .iter()
                .map(|d| (d.repo_name.as_str(), d.spec.name.as_str()))
                .collect::<Vec<_>>(),
            [("rules_foo", "rules_foo"), ("bar", "rules_bar")]
        );
    }

    #[test]
    fn every_module_implicitly_depends_on_bazel_tools() {
        let file = eval_root("module(name = 'a', version = '1')").unwrap();
        assert!(file.module.dep("bazel_tools").is_some());
        // ...except bazel_tools itself.
        let options = EvalOptions::dependency(ModuleKey::new("bazel_tools", Version::EMPTY));
        let file =
            eval_module_file("MODULE.bazel", "module(name = 'bazel_tools')", &options).unwrap();
        assert!(file.module.deps.is_empty());
    }

    #[test]
    fn module_must_come_first_and_only_once() {
        assert!(eval_root("module(name='a')\nmodule(name='b')").is_err());
        assert!(eval_root("bazel_dep(name='b', version='1')\nmodule(name='a')").is_err());
        // No module() call at all is legal for the root module.
        eval_root("bazel_dep(name = 'b', version = '1')").unwrap();
    }

    #[test]
    fn repo_names_must_not_collide() {
        let err = eval_root_no_builtins(
            "module(name = 'a')\nbazel_dep(name = 'b', version = '1', repo_name = 'a')",
        )
        .unwrap_err();
        assert!(err.to_string().contains("cannot be defined"), "{err}");
    }

    #[test]
    fn nodep_edges_come_from_repo_name_none() {
        let file = eval_root_no_builtins(
            r#"
bazel_dep(name = "regular", version = "1")
bazel_dep(name = "optional", version = "1", repo_name = None)
"#,
        )
        .unwrap();
        assert_eq!(file.module.deps.len(), 1);
        assert_eq!(file.module.nodep_deps.len(), 1);
        assert_eq!(file.module.nodep_deps[0].name, "optional");
    }

    #[test]
    fn dev_dependencies_are_dropped_for_non_root_modules() {
        let source = r#"
module(name = "dep", version = "1")
bazel_dep(name = "prod", version = "1")
bazel_dep(name = "test_only", version = "1", dev_dependency = True)
"#;
        let root = eval_root_no_builtins(source).unwrap();
        assert_eq!(root.module.deps.len(), 2);

        let mut options =
            EvalOptions::dependency(ModuleKey::new("dep", Version::parse("1").unwrap()));
        options.builtin_modules = Vec::new();
        let dep = eval_module_file("MODULE.bazel", source, &options).unwrap();
        assert_eq!(dep.module.deps.len(), 1);
        assert_eq!(dep.module.deps[0].spec.name, "prod");
    }

    #[test]
    fn overrides_of_a_dependency_are_ignored() {
        let source = r#"
module(name = "dep", version = "1")
single_version_override(module_name = "other", version = "2")
"#;
        let root = eval_root_no_builtins(source).unwrap();
        assert_eq!(root.overrides.len(), 1);

        let mut options =
            EvalOptions::dependency(ModuleKey::new("dep", Version::parse("1").unwrap()));
        options.builtin_modules = Vec::new();
        let dep = eval_module_file("MODULE.bazel", source, &options).unwrap();
        assert!(dep.overrides.is_empty());
    }

    #[test]
    fn overrides_are_recorded_with_their_arguments() {
        let file = eval_root_no_builtins(
            r#"
single_version_override(
    module_name = "pinned",
    version = "1.2.3",
    registry = "https://example.com/registry",
    patches = ["//:fix.patch"],
    patch_strip = 1,
)
multiple_version_override(module_name = "multi", versions = ["1.0", "2.0"])
archive_override(module_name = "arch", urls = ["https://e/a.zip"], strip_prefix = "a-1")
git_override(module_name = "git", remote = "https://e/g.git", commit = "abc")
local_path_override(module_name = "local", path = "../local")
"#,
        )
        .unwrap();
        let by_name: std::collections::BTreeMap<_, _> = file
            .overrides
            .iter()
            .map(|(n, o)| (n.as_str(), o))
            .collect();

        let ModuleOverride::SingleVersion(svo) = by_name["pinned"] else {
            panic!("expected single_version_override")
        };
        assert_eq!(svo.version.as_str(), "1.2.3");
        assert_eq!(
            svo.registry.as_deref(),
            Some("https://example.com/registry")
        );
        assert_eq!(svo.patches, ["//:fix.patch"]);
        assert_eq!(svo.patch_strip, 1);

        let ModuleOverride::MultipleVersion(mvo) = by_name["multi"] else {
            panic!("expected multiple_version_override")
        };
        assert_eq!(mvo.versions.len(), 2);

        for (name, rule) in [
            ("arch", RepoRule::HttpArchive),
            ("git", RepoRule::GitRepository),
            ("local", RepoRule::LocalRepository),
        ] {
            let ModuleOverride::NonRegistry(non_registry) = by_name[name] else {
                panic!("expected a non-registry override for {name}")
            };
            assert_eq!(non_registry.repo_spec.rule, rule);
        }
    }

    #[test]
    fn only_one_override_per_module() {
        let err = eval_root_no_builtins(
            r#"
single_version_override(module_name = "a", version = "1")
local_path_override(module_name = "a", path = "../a")
"#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("multiple overrides"), "{err}");
    }

    #[test]
    fn multiple_version_override_needs_two_versions() {
        assert!(
            eval_root_no_builtins(
                "multiple_version_override(module_name = 'a', versions = ['1.0'])"
            )
            .is_err()
        );
    }

    #[test]
    fn extension_usages_record_tags_and_imports() {
        let file = eval_root_no_builtins(
            r#"
rust = use_extension("@rules_rust//rust:extensions.bzl", "rust")
rust.toolchain(edition = "2024", versions = ["1.97.1"])
use_repo(rust, "rust_toolchains")
use_repo(rust, aliased = "rust_analyzer")
"#,
        )
        .unwrap();
        assert_eq!(file.module.extension_usages.len(), 1);
        let usage = &file.module.extension_usages[0];
        assert_eq!(usage.bzl_file, "@rules_rust//rust:extensions.bzl");
        assert_eq!(usage.extension_name, "rust");
        assert_eq!(usage.tags.len(), 1);
        assert_eq!(usage.tags[0].tag_class, "toolchain");
        assert_eq!(
            usage.tags[0].attrs,
            vec![
                ("edition".to_owned(), AttrValue::String("2024".to_owned())),
                (
                    "versions".to_owned(),
                    AttrValue::List(vec![AttrValue::String("1.97.1".to_owned())])
                ),
            ]
        );
        assert_eq!(
            usage.imports,
            [
                ("rust_toolchains".to_owned(), "rust_toolchains".to_owned()),
                ("aliased".to_owned(), "rust_analyzer".to_owned()),
            ]
        );
    }

    #[test]
    fn repeated_use_extension_shares_one_usage() {
        let file = eval_root_no_builtins(
            r#"
a = use_extension("//:ext.bzl", "ext")
b = use_extension("//:ext.bzl", "ext")
a.tag(x = 1)
b.tag(x = 2)
c = use_extension("//:ext.bzl", "ext", isolate = True)
"#,
        )
        .unwrap();
        assert_eq!(file.module.extension_usages.len(), 2);
        assert_eq!(file.module.extension_usages[0].tags.len(), 2);
        assert!(file.module.extension_usages[1].isolate);
    }

    #[test]
    fn rejects_control_flow_and_loads() {
        // MODULE.bazel is a declaration, not a program: Bazel's
        // DotBazelFileSyntaxChecker rejects all of these.
        for source in [
            "def f():\n    pass\n",
            "if True:\n    bazel_dep(name = 'a', version = '1')\n",
            "for x in []:\n    pass\n",
            "load('//:a.bzl', 'b')\n",
        ] {
            let err = eval_root(source).unwrap_err();
            assert!(
                matches!(err, BzlmodError::BadModule { .. }),
                "expected {source:?} to be rejected, got {err}"
            );
        }
    }

    #[test]
    fn registry_modules_must_declare_the_name_they_were_served_as() {
        let key = ModuleKey::new("expected", Version::parse("1.0").unwrap());
        let err = eval_module_file(
            "MODULE.bazel",
            "module(name = 'something_else', version = '1.0')",
            &EvalOptions::dependency(key.clone()),
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("declares a different name"),
            "{err}"
        );

        let err = eval_module_file(
            "MODULE.bazel",
            "module(name = 'expected', version = '9.9')",
            &EvalOptions::dependency(key),
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("declares a different version"),
            "{err}"
        );
    }

    #[test]
    fn register_directives_require_absolute_patterns() {
        let file = eval_root_no_builtins(
            "register_toolchains('//:tc', '@other//:tc')\nregister_execution_platforms('//:p')",
        )
        .unwrap();
        assert_eq!(file.module.toolchains_to_register, ["//:tc", "@other//:tc"]);
        assert_eq!(file.module.execution_platforms_to_register, ["//:p"]);
        assert!(eval_root_no_builtins("register_toolchains('relative:tc')").is_err());
    }

    #[test]
    fn compatibility_level_is_accepted_and_warned_about() {
        // Bazel 9.2 parses it, warns, and ignores it.
        let file =
            eval_root_no_builtins("module(name = 'a', version = '1', compatibility_level = 2)")
                .unwrap();
        assert_eq!(file.warnings.len(), 1);
        assert!(file.warnings[0].contains("no-op"));
    }

    #[test]
    fn bazel_compatibility_values_are_validated() {
        eval_root_no_builtins("module(name = 'a', bazel_compatibility = ['>=6.0.0', '-7.1.0'])")
            .unwrap();
        assert!(
            eval_root_no_builtins("module(name = 'a', bazel_compatibility = ['6.0.0'])").is_err()
        );
        assert!(
            eval_root_no_builtins("module(name = 'a', bazel_compatibility = ['>=6.0'])").is_err()
        );
    }

    #[test]
    fn use_repo_kwargs_template_the_importing_module() {
        // Bazel substitutes the importing module's own identity, so a
        // module can import a repo an extension named after it.
        let file = eval_root_no_builtins(
            r#"
module(name = "mine", version = "1.2")
ext = use_extension("//:ext.bzl", "ext")
use_repo(ext, deps = "{name}_{version}_deps")
"#,
        )
        .unwrap();
        assert_eq!(
            file.module.extension_usages[0].imports,
            [("deps".to_owned(), "mine_1.2_deps".to_owned())]
        );
    }

    #[test]
    fn use_repo_rule_records_an_innate_extension() {
        let file = eval_root_no_builtins(
            r#"
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name = "zlib", urls = ["https://e/z.tar.gz"], integrity = "sha256-x")
"#,
        )
        .unwrap();
        let usage = &file.module.extension_usages[0];
        assert_eq!(usage.bzl_file, "//:MODULE.bazel");
        assert_eq!(
            usage.extension_name,
            "@bazel_tools//tools/build_defs/repo:http.bzl http_archive"
        );
        assert_eq!(usage.tags[0].tag_class, "repo");
        // `name` becomes an attribute of the repo rule call *and* an
        // import, since the repo is visible to the calling module.
        assert!(
            usage.tags[0]
                .attrs
                .contains(&("name".to_owned(), AttrValue::String("zlib".to_owned())))
        );
        assert_eq!(usage.imports, [("zlib".to_owned(), "zlib".to_owned())]);
    }

    #[test]
    fn override_and_inject_repo_are_recorded_with_their_strictness() {
        let file = eval_root_no_builtins(
            r#"
ext = use_extension("//:ext.bzl", "ext")
override_repo(ext, "shared")
inject_repo(ext, extra = "my_repo")
"#,
        )
        .unwrap();
        let overrides = &file.module.extension_usages[0].repo_overrides;
        assert_eq!(overrides.len(), 2);
        assert_eq!(overrides[0].overridden_repo_name, "shared");
        assert_eq!(overrides[0].overriding_repo_name, "shared");
        assert!(overrides[0].must_exist);
        assert_eq!(overrides[1].overridden_repo_name, "extra");
        assert_eq!(overrides[1].overriding_repo_name, "my_repo");
        assert!(!overrides[1].must_exist);
    }

    #[test]
    fn repo_overrides_of_a_dependency_are_ignored() {
        // They name repos of the calling module, which a dependency has no
        // business redirecting.
        let source = r#"
module(name = "dep", version = "1")
ext = use_extension("//:ext.bzl", "ext")
override_repo(ext, "shared")
"#;
        let mut options =
            EvalOptions::dependency(ModuleKey::new("dep", Version::parse("1").unwrap()));
        options.builtin_modules = Vec::new();
        let file = eval_module_file("MODULE.bazel", source, &options).unwrap();
        assert!(file.module.extension_usages[0].repo_overrides.is_empty());
    }

    #[test]
    fn flag_aliases_are_recorded() {
        let file =
            eval_root_no_builtins("flag_alias(name = 'fast', starlark_flag = '//settings:fast')")
                .unwrap();
        assert_eq!(
            file.module.flag_aliases,
            [("fast".to_owned(), "//settings:fast".to_owned())]
        );
    }

    #[test]
    fn print_is_available_but_silent_for_dependencies() {
        // bazel_gazelle's own MODULE.bazel calls print(); Bazel makes it a
        // no-op for a dependency so a registry module cannot spam the
        // console during resolution.
        eval_root_no_builtins("print('hello')").unwrap();
        let mut options =
            EvalOptions::dependency(ModuleKey::new("dep", Version::parse("1").unwrap()));
        options.builtin_modules = Vec::new();
        eval_module_file(
            "MODULE.bazel",
            "module(name = 'dep', version = '1')\nprint('quiet')",
            &options,
        )
        .unwrap();
    }

    /// An in-memory [`IncludeSource`] for tests: a fixed table of label ->
    /// source text.
    #[derive(Debug)]
    struct FakeIncludeSource(std::collections::BTreeMap<&'static str, &'static str>);

    impl IncludeSource for FakeIncludeSource {
        fn read(&self, label: &str) -> Result<String> {
            self.0
                .get(label)
                .map(|s| s.to_string())
                .ok_or_else(|| BzlmodError::BadModule {
                    key: "<test>".to_owned(),
                    message: format!("no such fake include: {label}"),
                })
        }
    }

    fn eval_root_with_includes(
        source: &str,
        includes: &[(&'static str, &'static str)],
    ) -> Result<ModuleFile> {
        let mut options = EvalOptions::root();
        options.builtin_modules = Vec::new();
        options = options.with_include_source(Rc::new(FakeIncludeSource(
            includes.iter().copied().collect(),
        )));
        eval_module_file("MODULE.bazel", source, &options)
    }

    #[test]
    fn include_label_must_be_repo_relative_and_name_a_dot_module_bazel_file() {
        for bad in [
            "extra.MODULE.bazel",      // not repo-relative
            "//:.hidden.MODULE.bazel", // starts with a dot
            "//:extra.bzl",            // wrong suffix
            "//:extra",                // wrong suffix
        ] {
            let err = eval_root_with_includes(&format!("include('{bad}')"), &[]).unwrap_err();
            assert!(
                matches!(err, BzlmodError::BadModule { .. }),
                "expected {bad:?} to be rejected, got {err}"
            );
        }
        // A well-formed label, even with no configured source, fails for a
        // different reason (unresolved), so validation alone must have let
        // it through above.
    }

    #[test]
    fn include_without_a_configured_source_is_an_error() {
        let err = eval_root_no_builtins("include('//:extra.MODULE.bazel')").unwrap_err();
        assert!(err.to_string().contains("no include source"), "{err}");
    }

    #[test]
    fn include_runs_inline_in_the_same_state() {
        // A bazel_dep before the include, one inside it, and one after: all
        // three end up as deps of the including module, in that order —
        // exactly as if the included text had been pasted in place.
        let file = eval_root_with_includes(
            r#"
bazel_dep(name = "before", version = "1")
include("//:extra.MODULE.bazel")
bazel_dep(name = "after", version = "1")
"#,
            &[(
                "//:extra.MODULE.bazel",
                "bazel_dep(name = 'middle', version = '1')",
            )],
        )
        .unwrap();
        assert_eq!(
            file.module
                .deps
                .iter()
                .map(|d| d.spec.name.as_str())
                .collect::<Vec<_>>(),
            ["before", "middle", "after"]
        );
        assert_eq!(file.includes, ["//:extra.MODULE.bazel"]);
    }

    #[test]
    fn include_can_itself_include() {
        let file = eval_root_with_includes(
            "include('//:a.MODULE.bazel')",
            &[
                ("//:a.MODULE.bazel", "include('//:b.MODULE.bazel')"),
                (
                    "//:b.MODULE.bazel",
                    "bazel_dep(name = 'leaf', version = '1')",
                ),
            ],
        )
        .unwrap();
        assert_eq!(file.module.deps[0].spec.name, "leaf");
        assert_eq!(file.includes, ["//:a.MODULE.bazel", "//:b.MODULE.bazel"]);
    }

    #[test]
    fn include_cycle_is_rejected() {
        let err = eval_root_with_includes(
            "include('//:a.MODULE.bazel')",
            &[("//:a.MODULE.bazel", "include('//:a.MODULE.bazel')")],
        )
        .unwrap_err();
        assert!(err.to_string().contains("cycle"), "{err}");
    }

    #[test]
    fn include_is_refused_outside_root_and_non_registry_overrides() {
        // EvalOptions::dependency() defaults to allow_include: false — a
        // registry module using include() is an error, not a silent
        // no-op.
        let mut options =
            EvalOptions::dependency(ModuleKey::new("dep", Version::parse("1").unwrap()));
        options.builtin_modules = Vec::new();
        let err = eval_module_file(
            "MODULE.bazel",
            "module(name = 'dep', version = '1')\ninclude('//:extra.MODULE.bazel')",
            &options,
        )
        .unwrap_err();
        assert!(err.to_string().contains("only allowed"), "{err}");

        // A non-registry override lifts that, same as the root.
        let mut options =
            EvalOptions::dependency(ModuleKey::new("dep", Version::parse("1").unwrap()))
                .with_include_source(Rc::new(FakeIncludeSource(
                    [(
                        "//:extra.MODULE.bazel",
                        "bazel_dep(name = 'x', version = '1')",
                    )]
                    .into_iter()
                    .collect(),
                )));
        options.builtin_modules = Vec::new();
        let file = eval_module_file(
            "MODULE.bazel",
            "module(name = 'dep', version = '1')\ninclude('//:extra.MODULE.bazel')",
            &options,
        )
        .unwrap();
        assert_eq!(file.module.deps[0].spec.name, "x");
    }
}
