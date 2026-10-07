//! `ctx.actions` (buildfiji-136.3): what a rule's code registers.
//!
//! The mnemonics, progress messages and command lines are the ones
//! `bazel aquery` showed for each call (Bazel 9.2.0):
//!
//! - `write` is a `FileWrite` ("Writing file x", "Writing script x" when
//!   executable), `symlink` a `Symlink` ("Creating symlink x");
//! - `run_shell` is `/bin/bash -c <command>`, followed by an empty `$0` and
//!   the `arguments` if there are any; `run` is the executable and its
//!   arguments;
//! - the mnemonic defaults to `Action` and the progress message to the
//!   mnemonic and the first output's short path;
//! - with `use_default_shell_env` the environment is `PATH`, then `env`.

use super::ctx::CtxState;
use super::file::{FileValue, alloc_file, artifact_of};
use crate::args::{Wording, bind, describe, fatal, param, sequence};
use crate::depset::{depset_to_list, is_depset, nested_of};
use allocative::Allocative;
use fjfj_graph::{Action, ActionKind, Artifact, NestedSet};
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_module;
use starlark::starlark_simple_value;
use starlark::values::dict::{AllocDict, DictRef};
use starlark::values::none::NoneType;
use starlark::values::{NoSerialize, ProvidesStaticType, StarlarkValue, Value, ValueLike};
use starlark_derive::starlark_value;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

const SHELL: &str = "/bin/bash";

#[derive(ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct ActionsValue {
    #[allocative(skip)]
    pub(crate) state: Arc<CtxState>,
}

starlark_simple_value!(ActionsValue);

impl fmt::Debug for ActionsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("actions")
    }
}

impl fmt::Display for ActionsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ctx.actions")
    }
}

#[starlark_value(type = "actions")]
impl<'v> StarlarkValue<'v> for ActionsValue {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("actions", actions_members);
        Some(RES.methods())
    }
}

/// The length of a command line above which an `Args` that allows it goes to a
/// param file, as Bazel 9.2.0 does.
const PARAM_FILE_LIMIT: usize = 31744;

fn state<'v>(this: Value<'v>) -> &'v Arc<CtxState> {
    &this.downcast_ref::<ActionsValue>().expect("actions").state
}

/// The files of a list, tuple or depset argument.
fn files_of<'v>(
    eval: &Evaluator<'v, '_, '_>,
    function: &str,
    name: &str,
    value: Value<'v>,
) -> starlark::Result<Vec<Artifact>> {
    let items: Vec<Value<'v>> = if is_depset(value) {
        depset_to_list(value).expect("a depset")?
    } else if let Some(items) = sequence(value) {
        items
    } else if value.is_none() {
        Vec::new()
    } else {
        return Err(fatal(format!(
            "in call to {function}(), parameter '{name}' got value of type '{}', want 'sequence or depset'",
            value.get_type()
        )));
    };
    let _ = eval;
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        if let Some(artifact) = artifact_of(item) {
            out.push(artifact);
        } else if let Some(files) = files_to_run_inputs(item) {
            out.extend(files);
        } else if is_depset(item) {
            // `tools = [files_to_run, a_depset]`.
            for element in depset_to_list(item).expect("a depset")? {
                match artifact_of(element) {
                    Some(artifact) => out.push(artifact),
                    None => {
                        return Err(fatal(format!(
                            "expected value of type 'File' for element of {name}, but got {}",
                            describe(element)
                        )));
                    }
                }
            }
        } else {
            return Err(fatal(format!(
                "expected value of type 'File' for element of {name}, but got {}",
                describe(item)
            )));
        }
    }
    Ok(out)
}

/// `files` without repeats, in order.
fn unique(files: &[Artifact]) -> Vec<Artifact> {
    let mut out: Vec<Artifact> = Vec::with_capacity(files.len());
    for file in files {
        if !out.contains(file) {
            out.push(file.clone());
        }
    }
    out
}

/// What an action that uses a `FilesToRunProvider` needs, as Bazel lists it:
/// the executable and its runfiles tree, which stands for the manifest, the
/// repository mapping and the files they name.
fn files_to_run_inputs(value: Value<'_>) -> Option<Vec<Artifact>> {
    let fields = crate::structs::fields_of(value)?;
    let field = |name: &str| fields.iter().find(|(n, _)| *n == name).map(|(_, v)| *v);
    field("runfiles_manifest")?;
    field("repo_mapping_manifest")?;
    let mut out = Vec::new();
    let exe = field("executable").and_then(artifact_of);
    out.extend(exe.clone());
    // A plain file has no runfiles tree.
    if let Some(exe) = exe
        && field("runfiles_manifest").and_then(artifact_of).is_some()
    {
        out.push(Artifact {
            root: exe.root.clone(),
            path: format!("{}.runfiles", exe.path),
            tree: false,
            symlink: false,
        });
    }
    Some(out)
}

fn string_dict(
    function: &str,
    name: &str,
    value: Option<Value<'_>>,
) -> starlark::Result<BTreeMap<String, String>> {
    let Some(value) = value.filter(|v| !v.is_none()) else {
        return Ok(BTreeMap::new());
    };
    let dict = DictRef::from_value(value).ok_or_else(|| {
        fatal(format!(
            "in call to {function}(), parameter '{name}' got value of type '{}', want 'dict or NoneType'",
            value.get_type()
        ))
    })?;
    let mut out = BTreeMap::new();
    for (k, v) in dict.iter() {
        match (k.unpack_str(), v.unpack_str()) {
            (Some(k), Some(v)) => {
                out.insert(k.to_owned(), v.to_owned());
            }
            _ => {
                return Err(fatal(format!(
                    "in call to {function}(), parameter '{name}' got an entry that is not a string to string"
                )));
            }
        }
    }
    Ok(out)
}

fn optional_string(
    function: &str,
    name: &str,
    value: Option<Value<'_>>,
) -> starlark::Result<Option<String>> {
    match value.filter(|v| !v.is_none()) {
        None => Ok(None),
        Some(v) => v.unpack_str().map(|s| Some(s.to_owned())).ok_or_else(|| {
            fatal(format!(
                "in call to {function}(), parameter '{name}' got value of type '{}', want 'string or NoneType'",
                v.get_type()
            ))
        }),
    }
}

fn flag(function: &str, name: &str, value: Option<Value<'_>>) -> starlark::Result<bool> {
    match value {
        None => Ok(false),
        Some(v) => v.unpack_bool().ok_or_else(|| {
            fatal(format!(
                "in call to {function}(), parameter '{name}' got value of type '{}', want 'bool'",
                v.get_type()
            ))
        }),
    }
}

impl CtxState {
    /// A file next to `sibling`.
    fn derived_next_to(&self, sibling: &Artifact, name: &str) -> Artifact {
        let path = match sibling.path.rsplit_once('/') {
            Some((dir, _)) => format!("{dir}/{name}"),
            None => name.to_owned(),
        };
        Artifact {
            root: sibling.root.clone(),
            path,
            tree: false,
            symlink: false,
        }
    }

    /// Register an action of this target.
    fn register(
        &self,
        mnemonic: &str,
        progress_message: Option<String>,
        kind: ActionKind,
        inputs: Vec<Artifact>,
        outputs: Vec<Artifact>,
    ) {
        self.register_nested(mnemonic, progress_message, kind, inputs, None, outputs);
    }

    /// Register an action whose inputs nest as `input_set`.
    fn register_nested(
        &self,
        mnemonic: &str,
        progress_message: Option<String>,
        kind: ActionKind,
        inputs: Vec<Artifact>,
        input_set: Option<Arc<NestedSet<Artifact>>>,
        outputs: Vec<Artifact>,
    ) {
        let progress = match progress_message {
            // `%{label}`, `%{input}` and `%{output}` stand for the label of
            // the target, and the first input and output by their short paths.
            Some(message) => message
                .replace("%{label}", &fjfj_graph::expand::label_text(&self.label))
                .replace(
                    "%{input}",
                    &inputs
                        .first()
                        .map(|i| FileValueView(i).short_path())
                        .unwrap_or_default(),
                )
                .replace(
                    "%{output}",
                    &outputs
                        .first()
                        .map(|o| FileValueView(o).short_path())
                        .unwrap_or_default(),
                ),
            None => {
                let first = outputs
                    .first()
                    .map(|o| FileValueView(o).short_path())
                    .unwrap_or_default();
                format!("{mnemonic} {first}")
            }
        };
        self.actions.lock().unwrap().push(Action {
            owner: self.label.clone(),
            owner_kind: self.rule_kind.clone(),
            location: self.location.clone(),
            configuration: self.configuration.mnemonic(),
            mnemonic: mnemonic.to_owned(),
            progress_message: Some(progress),
            kind,
            inputs,
            input_set,
            outputs,
            exec_group: None,
        });
    }
}

/// An artifact's short path, which `FileValue` computes.
struct FileValueView<'a>(&'a Artifact);

impl FileValueView<'_> {
    fn short_path(&self) -> String {
        FileValue {
            artifact: self.0.clone(),
            owner: fjfj_graph::Label {
                repo: String::new(),
                package: String::new(),
                name: String::new(),
            },
        }
        .short_path()
    }
}

/// `declare_file` and `declare_symlink`: a new file of the rule's package.
fn declare<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
    function: &str,
) -> starlark::Result<Value<'v>> {
    let s = state(this);
    let bound = bind(
        function,
        Wording::Signature,
        &[
            param("filename", true, true),
            param("sibling", false, false),
        ],
        args,
        eval,
    )?;
    let filename = bound[0].and_then(|v| v.unpack_str()).ok_or_else(|| {
        fatal(format!(
            "in call to {function}(), parameter 'filename' got value of type that is not 'string'"
        ))
    })?;
    let mut artifact = match bound[1].filter(|v| !v.is_none()) {
        Some(sibling) => {
            let sibling = artifact_of(sibling).ok_or_else(|| {
                    fatal(format!("in call to {function}(), parameter 'sibling' got value of type that is not 'File'"))
                })?;
            let path = match sibling.path.rsplit_once('/') {
                Some((dir, _)) => format!("{dir}/{filename}"),
                None => filename.to_owned(),
            };
            Artifact {
                root: sibling.root,
                path,
                tree: false,
                symlink: false,
            }
        }
        None => s.derived(filename),
    };
    artifact.symlink = function == "declare_symlink";
    // Declaring a path again is allowed (Bazel gives the same file); two
    // different actions creating it are not (checked when the rule is done).
    s.declared.lock().unwrap().insert(artifact.exec_path());
    Ok(alloc_file(eval.heap(), artifact, s.label.clone()))
}

#[starlark_module]
fn actions_members(builder: &mut MethodsBuilder) {
    /// `ctx.actions.declare_file(filename, *, sibling = None)`.
    fn declare_file<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        declare(this, args, eval, "declare_file")
    }

    /// `ctx.actions.declare_symlink(filename, *, sibling = None)`: a file that
    /// `symlink(target_path = ...)` makes a link of.
    fn declare_symlink<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        declare(this, args, eval, "declare_symlink")
    }

    /// `ctx.actions.transform_version_file(*, transform_func, template, output_file_name)`
    /// and `transform_info_file`: the template with the substitutions
    /// `transform_func` makes of the workspace status. Stamping is not
    /// implemented (buildfiji-wiq), so the status is Bazel's unstamped one.
    fn transform_version_file<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        transform_status_file(
            this,
            args,
            eval,
            "transform_version_file",
            &[("BUILD_TIMESTAMP", "0"), ("BUILD_SCM_REVISION", "0")],
        )
    }

    fn transform_info_file<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        transform_status_file(
            this,
            args,
            eval,
            "transform_info_file",
            &[
                ("BUILD_EMBED_LABEL", ""),
                ("BUILD_HOST", "hostname"),
                ("BUILD_USER", "username"),
            ],
        )
    }

    /// `ctx.actions.declare_directory(filename, *, sibling = None)`: a tree
    /// artifact, a directory an action fills.
    fn declare_directory<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let s = state(this);
        let bound = bind(
            "declare_directory",
            Wording::Signature,
            &[
                param("filename", true, true),
                param("sibling", false, false),
            ],
            args,
            eval,
        )?;
        let filename = bound[0]
            .and_then(|v| v.unpack_str())
            .ok_or_else(|| fatal("in call to declare_directory(), parameter 'filename' got value of type that is not 'string'"))?;
        let mut artifact = match bound[1].filter(|v| !v.is_none()) {
            Some(sibling) => {
                let sibling = artifact_of(sibling).ok_or_else(|| {
                    fatal("in call to declare_directory(), parameter 'sibling' got value of type that is not 'File'")
                })?;
                s.derived_next_to(&sibling, filename)
            }
            None => s.derived(filename),
        };
        artifact.tree = true;
        s.declared.lock().unwrap().insert(artifact.exec_path());
        Ok(alloc_file(eval.heap(), artifact, s.label.clone()))
    }

    /// `ctx.actions.declare_shareable_artifact(path)`: a file in the output
    /// directory at a path from its root, not from the rule's package, which
    /// rules_cc uses for the files that are shared between targets.
    fn declare_shareable_artifact<'v>(
        this: Value<'v>,
        path: &str,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let s = state(this);
        let artifact = Artifact {
            root: fjfj_graph::artifact::Root::derived(s.bin_dir()),
            path: path.to_owned(),
            tree: false,
            symlink: false,
        };
        s.declared.lock().unwrap().insert(artifact.exec_path());
        Ok(alloc_file(eval.heap(), artifact, s.label.clone()))
    }

    /// `ctx.actions.template_dict()`.
    fn template_dict<'v>(
        this: Value<'v>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let _ = this;
        let make = super::target::builtin("_template_dict")
            .ok_or_else(|| fatal("the builtins have no _template_dict"))?;
        eval.eval_function(make, &[], &[])
    }

    /// `ctx.actions.args()`.
    fn args<'v>(this: Value<'v>, heap: starlark::values::Heap<'v>) -> starlark::Result<Value<'v>> {
        let _ = this;
        Ok(heap.alloc(super::args_object::ArgsValue::new()))
    }

    /// `ctx.actions.write(output, content, is_executable = False)`.
    fn write<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let s = state(this);
        let bound = bind(
            "write",
            Wording::Signature,
            &[
                param("output", true, true),
                param("content", true, true),
                param("is_executable", true, false),
            ],
            args,
            eval,
        )?;
        let output = bound[0].and_then(artifact_of).ok_or_else(|| {
            fatal("in call to write(), parameter 'output' got value of type that is not 'File'")
        })?;
        let content = bound[1].and_then(|v| v.unpack_str()).ok_or_else(|| {
            fatal("ctx.actions.write: content must be a string (Args is not supported yet)")
        })?;
        let executable = flag("write", "is_executable", bound[2])?;
        let message = if executable {
            format!("Writing script {}", basename(&output))
        } else {
            format!("Writing file {}", basename(&output))
        };
        s.register(
            "FileWrite",
            Some(message),
            ActionKind::WriteFile {
                contents: content.as_bytes().to_vec(),
                executable,
            },
            Vec::new(),
            vec![output],
        );
        Ok(NoneType)
    }

    /// `ctx.actions.run_shell(...)`.
    fn run_shell<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        spawn(this, args, eval, true)
    }

    /// `ctx.actions.run(...)`.
    fn run<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        spawn(this, args, eval, false)
    }

    /// `ctx.actions.symlink(*, output, target_file, target_path, is_executable, progress_message, use_exec_root_for_source)`.
    fn symlink<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let s = state(this);
        let bound = bind(
            "symlink",
            Wording::Signature,
            &[
                param("output", false, true),
                param("target_file", false, false),
                param("target_path", false, false),
                param("is_executable", false, false),
                param("progress_message", false, false),
                // Where the link points is always the exec root here.
                param("use_exec_root_for_source", false, false),
            ],
            args,
            eval,
        )?;
        let output = bound[0].and_then(artifact_of).ok_or_else(|| {
            fatal("in call to symlink(), parameter 'output' got value of type that is not 'File'")
        })?;
        let Some(target) = bound[1].filter(|v| !v.is_none()).and_then(artifact_of) else {
            // `target_path`: the link says exactly that, with no input.
            let Some(path) = bound[2].and_then(|v| v.unpack_str()) else {
                return Err(fatal(
                    "ctx.actions.symlink: needs target_file or target_path",
                ));
            };
            s.register(
                "UnresolvedSymlink",
                Some(format!("Creating symlink {}", basename(&output))),
                ActionKind::UnresolvedSymlink {
                    target: path.to_owned(),
                },
                Vec::new(),
                vec![output],
            );
            return Ok(NoneType);
        };
        let message = optional_string("symlink", "progress_message", bound[4])?
            .or_else(|| Some(format!("Creating symlink {}", basename(&output))));
        let executable = flag("symlink", "is_executable", bound[3])?;
        s.register(
            if executable {
                "ExecutableSymlink"
            } else {
                "Symlink"
            },
            message,
            ActionKind::Symlink {
                target: target.exec_path(),
            },
            vec![target],
            vec![output],
        );
        Ok(NoneType)
    }

    /// `ctx.actions.expand_template(*, template, output, substitutions, is_executable, computed_substitutions)`.
    fn expand_template<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let s = state(this);
        let bound = bind(
            "expand_template",
            Wording::Signature,
            &[
                param("template", false, true),
                param("output", false, true),
                param("substitutions", false, true),
                param("is_executable", false, false),
                param("computed_substitutions", false, false),
            ],
            args,
            eval,
        )?;
        let template = bound[0]
            .and_then(artifact_of)
            .ok_or_else(|| fatal("in call to expand_template(), parameter 'template' got value of type that is not 'File'"))?;
        let output = bound[1]
            .and_then(artifact_of)
            .ok_or_else(|| fatal("in call to expand_template(), parameter 'output' got value of type that is not 'File'"))?;
        let mut substitutions: BTreeMap<String, String> =
            string_dict("expand_template", "substitutions", bound[2])?;
        // Substitutions computed from lists (`template_dict()`).
        if let Some(computed) = bound[4].filter(|v| !v.is_none()) {
            let compute = super::target::builtin("_computed_substitutions")
                .ok_or_else(|| fatal("the builtins have no _computed_substitutions"))?;
            let made = eval.eval_function(compute, &[computed], &[])?;
            let dict = DictRef::from_value(made)
                .ok_or_else(|| fatal("computed_substitutions is not a template_dict()"))?;
            for (k, v) in dict.iter() {
                if let (Some(k), Some(v)) = (k.unpack_str(), v.unpack_str()) {
                    substitutions.insert(k.to_owned(), v.to_owned());
                }
            }
        }
        let substitutions: Vec<(String, String)> = substitutions.into_iter().collect();
        let executable = flag("expand_template", "is_executable", bound[3])?;
        s.register(
            "TemplateExpand",
            Some(format!("Expanding template {}", basename(&output))),
            ActionKind::Template {
                template: template.exec_path(),
                substitutions,
                executable,
            },
            vec![template],
            vec![output],
        );
        Ok(NoneType)
    }
}

fn transform_status_file<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
    function: &str,
    status: &[(&str, &str)],
) -> starlark::Result<Value<'v>> {
    let s = state(this);
    let bound = bind(
        function,
        Wording::Signature,
        &[
            param("transform_func", false, true),
            param("template", false, true),
            param("output_file_name", false, true),
        ],
        args,
        eval,
    )?;
    let template = bound[1].and_then(artifact_of).ok_or_else(|| {
        fatal(format!(
            "in call to {function}(), parameter 'template' got value of type that is not 'File'"
        ))
    })?;
    let name = bound[2]
        .and_then(|v| v.unpack_str())
        .ok_or_else(|| fatal(format!("in call to {function}(), parameter 'output_file_name' got value of type that is not 'string'")))?;
    let heap = eval.heap();
    let status = heap.alloc(AllocDict(
        status.iter().map(|(k, v)| (heap.alloc(*k), heap.alloc(*v))),
    ));
    let transform = bound[0].expect("required");
    let made = eval.eval_function(transform, &[status], &[])?;
    let substitutions: Vec<(String, String)> = string_dict(function, "transform_func", Some(made))?
        .into_iter()
        .collect();
    let output = s.derived(name);
    if !s.declared.lock().unwrap().insert(output.exec_path()) {
        return Err(fatal(format!(
            "'{}' was already declared",
            output.exec_path()
        )));
    }
    s.register(
        "TemplateExpand",
        Some(format!("Expanding template {}", basename(&template))),
        ActionKind::Template {
            template: template.exec_path(),
            substitutions,
            executable: false,
        },
        vec![template],
        vec![output.clone()],
    );
    Ok(alloc_file(eval.heap(), output, s.label.clone()))
}

/// The path of an output from the root of its directory (`pkg/name`), which
/// is how Bazel's messages for the actions that make files name them.
fn basename(artifact: &Artifact) -> String {
    artifact.path.clone()
}

/// `run` and `run_shell`, which differ in what runs.
fn spawn<'v>(
    this: Value<'v>,
    args: &Arguments<'v, '_>,
    eval: &mut Evaluator<'v, '_, '_>,
    shell: bool,
) -> starlark::Result<NoneType> {
    let s = state(this);
    let function = if shell { "run_shell" } else { "run" };
    let params = [
        param("outputs", false, true),
        param("inputs", false, false),
        param("tools", false, false),
        param("arguments", false, false),
        param("mnemonic", false, false),
        param("progress_message", false, false),
        param("use_default_shell_env", false, false),
        param("env", false, false),
        param("execution_requirements", false, false),
        param("input_manifests", false, false),
        param("exec_group", false, false),
        param("shadowed_action", false, false),
        param("resource_set", false, false),
        param("toolchain", false, false),
        param(if shell { "command" } else { "executable" }, false, true),
        param("unused_inputs_list", false, false),
    ];
    let bound = bind(function, Wording::Signature, &params, args, eval)?;
    // An aspect's own groups are not read yet, so only a rule's are checked.
    if let Some(group) = bound[10].filter(|v| !v.is_none())
        && s.rule.is_none()
    {
        let name = group.unpack_str().unwrap_or_default();
        if !s.schema.exec_groups.iter().any(|g| g.name == name) {
            return Err(fatal(format!(
                "Action declared for non-existent exec group '{name}'."
            )));
        }
    }
    let outputs = files_of(eval, function, "outputs", bound[0].expect("required"))?;
    if outputs.is_empty() {
        return Err(fatal(format!("{function}() requires at least one output")));
    }
    let mut inputs = match bound[1] {
        Some(v) => files_of(eval, function, "inputs", v)?,
        None => Vec::new(),
    };
    // How the inputs nest, as aquery shows: the `inputs` as given, the
    // `tools` as a list, then the files the action adds.
    let mut nested = vec![match bound[1] {
        Some(v) if is_depset(v) => nested_of(v, &artifact_of, &mut s.nested.lock().unwrap())
            .unwrap_or_else(|| Arc::new(NestedSet::of(inputs.clone()))),
        _ => Arc::new(NestedSet::of(unique(&inputs))),
    }];
    if let Some(v) = bound[2] {
        let tools = files_of(eval, function, "tools", v)?;
        nested.push(Arc::new(NestedSet::of(unique(&tools))));
        inputs.extend(tools);
    }
    let mut arguments: Vec<String> = Vec::new();
    let mut param_files = 0;
    // An `Args` that asked for a param file gets one when the command line is
    // too long for the system: each word and its separator count, the
    // executable and, for a shell command, `-c` and the script among them.
    // Probed on Bazel 9.2.0, where the limit is 31744 and no flag moves it.
    let too_long = {
        let last = bound[14].expect("required");
        let mut total = if shell {
            SHELL.len() + 1 + "-c".len() + 1 + last.unpack_str().map_or(0, |c| c.len() + 1) + 1
        } else if let Some(file) = artifact_of(last) {
            file.exec_path().len() + 1
        } else if let Some(file) = files_to_run_inputs(last).and_then(|f| f.first().cloned()) {
            file.exec_path().len() + 1
        } else {
            last.unpack_str().map_or(0, |p| p.len() + 1)
        };
        for item in bound[3]
            .filter(|v| !v.is_none())
            .and_then(sequence)
            .unwrap_or_default()
        {
            total += if let Some(text) = item.unpack_str() {
                text.len() + 1
            } else if let Some(file) = artifact_of(item) {
                file.exec_path().len() + 1
            } else if let Some(built) = super::args_object::args_value(item) {
                built
                    .state
                    .lock()
                    .unwrap()
                    .items
                    .iter()
                    .map(|a| a.len() + 1)
                    .sum()
            } else {
                0
            };
        }
        total > PARAM_FILE_LIMIT
    };
    if let Some(v) = bound[3].filter(|v| !v.is_none()) {
        for item in sequence(v).ok_or_else(|| {
            fatal(format!(
                "in call to {function}(), parameter 'arguments' got value of type '{}', want 'sequence'",
                v.get_type()
            ))
        })? {
            if let Some(text) = item.unpack_str() {
                arguments.push(text.to_owned());
            } else if let Some(file) = artifact_of(item) {
                arguments.push(file.exec_path());
            } else if let Some(built) = super::args_object::args_value(item) {
                let state = built.state.lock().unwrap();
                match &state.param_file {
                    Some(param) if param.use_always || too_long => {
                        // The arguments go to a file the command is told of.
                        let first = outputs[0].path.clone();
                        let file = s.derived_next_to(&outputs[0], &format!(
                            "{}-{}.params",
                            first.rsplit('/').next().unwrap_or(&first),
                            param_files
                        ));
                        param_files += 1;
                        s.register(
                            "ParameterFileWrite",
                            Some(format!("Writing file {}", basename(&file))),
                            ActionKind::WriteFile {
                                contents: super::args_object::param_file_contents(
                                    &state.items,
                                    param.format,
                                )
                                .into_bytes(),
                                executable: false,
                            },
                            Vec::new(),
                            vec![file.clone()],
                        );
                        arguments.push(param.pattern.replacen("%s", &file.exec_path(), 1));
                        nested.push(Arc::new(NestedSet::of(vec![file.clone()])));
                        inputs.push(file);
                    }
                    _ => arguments.extend(state.items.iter().cloned()),
                }
            } else {
                return Err(fatal(format!(
                    "ctx.actions.{function}: arguments must be strings or Files (Args is not supported yet), got {}",
                    describe(item)
                )));
            }
        }
    }
    let mnemonic =
        optional_string(function, "mnemonic", bound[4])?.unwrap_or_else(|| "Action".to_owned());
    let progress = optional_string(function, "progress_message", bound[5])?;
    let default_env = flag(function, "use_default_shell_env", bound[6])?;
    let mut env = BTreeMap::new();
    if default_env {
        env.extend(s.configuration.default_shell_env());
    }
    env.extend(string_dict(function, "env", bound[7])?);
    let execution_requirements = string_dict(function, "execution_requirements", bound[8])?;

    let last = bound[14].expect("required");
    let argv = if shell {
        let command = last.unpack_str().ok_or_else(|| {
            fatal(if sequence(last).is_some() {
                "'command' must be of type string. passing a sequence of strings as 'command' is deprecated. To temporarily disable this check, set --incompatible_run_shell_command_string=false."
                    .to_owned()
            } else {
                format!(
                    "in call to run_shell(), parameter 'command' got value of type '{}', want 'string'",
                    last.get_type()
                )
            })
        })?;
        let mut argv = vec![SHELL.to_owned(), "-c".to_owned(), command.to_owned()];
        if !arguments.is_empty() {
            argv.push(String::new());
            argv.extend(arguments);
        }
        argv
    } else {
        let executable = if let Some(file) = artifact_of(last) {
            // A tool of the rule comes with its runfiles.
            let mut tool = vec![file.clone()];
            tool.extend(s.executable_runfiles(&file));
            nested.push(Arc::new(NestedSet::of(tool.clone())));
            inputs.extend(tool);
            file.exec_path()
        } else if let Some(files) = files_to_run_inputs(last)
            && let Some(file) = files.first().cloned()
        {
            nested.push(Arc::new(NestedSet::of(files.clone())));
            inputs.extend(files);
            file.exec_path()
        } else if let Some(path) = last.unpack_str() {
            path.to_owned()
        } else {
            return Err(fatal(format!(
                "in call to run(), parameter 'executable' got value of type '{}', want 'File or string or FilesToRunProvider'",
                last.get_type()
            )));
        };
        let mut argv = vec![executable];
        argv.extend(arguments);
        argv
    };
    let input_set = NestedSet::join(&nested);
    s.register_nested(
        &mnemonic,
        progress,
        ActionKind::Spawn {
            argv,
            env,
            execution_requirements,
        },
        inputs,
        Some(input_set),
        outputs,
    );
    if let Some(group) = bound[10]
        .filter(|v| !v.is_none())
        .and_then(|v| v.unpack_str())
        && let Some(action) = s.actions.lock().unwrap().last_mut()
    {
        action.exec_group = Some(group.to_owned());
    }
    Ok(NoneType)
}
