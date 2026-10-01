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
use crate::depset::{depset_to_list, is_depset};
use allocative::Allocative;
use fjfj_graph::{Action, ActionKind, Artifact};
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_module;
use starlark::starlark_simple_value;
use starlark::values::dict::DictRef;
use starlark::values::none::NoneType;
use starlark::values::{NoSerialize, ProvidesStaticType, StarlarkValue, Value, ValueLike};
use starlark_derive::starlark_value;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

const SHELL: &str = "/bin/bash";
const SHELL_PATH: &str = "/bin:/usr/bin:/usr/local/bin";

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
    items
        .into_iter()
        .map(|item| {
            artifact_of(item).ok_or_else(|| {
                fatal(format!(
                    "expected value of type 'File' for element of {name}, but got {}",
                    describe(item)
                ))
            })
        })
        .collect()
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
    /// Register an action of this target.
    fn register(
        &self,
        mnemonic: &str,
        progress_message: Option<String>,
        kind: ActionKind,
        inputs: Vec<Artifact>,
        outputs: Vec<Artifact>,
    ) {
        let progress = progress_message.unwrap_or_else(|| {
            let first = outputs
                .first()
                .map(|o| FileValueView(o).short_path())
                .unwrap_or_default();
            format!("{mnemonic} {first}")
        });
        self.actions.lock().unwrap().push(Action {
            owner: self.label.clone(),
            owner_kind: self.rule_kind.clone(),
            location: self.location.clone(),
            configuration: self.configuration.mnemonic(),
            mnemonic: mnemonic.to_owned(),
            progress_message: Some(progress),
            kind,
            inputs,
            outputs,
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

#[starlark_module]
fn actions_members(builder: &mut MethodsBuilder) {
    /// `ctx.actions.declare_file(filename, *, sibling = None)`.
    fn declare_file<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let s = state(this);
        let bound = bind(
            "declare_file",
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
            .ok_or_else(|| fatal("in call to declare_file(), parameter 'filename' got value of type that is not 'string'"))?;
        let artifact = match bound[1].filter(|v| !v.is_none()) {
            Some(sibling) => {
                let sibling = artifact_of(sibling).ok_or_else(|| {
                    fatal("in call to declare_file(), parameter 'sibling' got value of type that is not 'File'")
                })?;
                let path = match sibling.path.rsplit_once('/') {
                    Some((dir, _)) => format!("{dir}/{filename}"),
                    None => filename.to_owned(),
                };
                Artifact {
                    root: sibling.root,
                    path,
                }
            }
            None => s.derived(filename),
        };
        if !s.declared.lock().unwrap().insert(artifact.exec_path()) {
            return Err(fatal(format!(
                "ctx.actions.declare_file: '{}' was already declared",
                artifact.exec_path()
            )));
        }
        Ok(alloc_file(eval.heap(), artifact, s.label.clone()))
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

    /// `ctx.actions.symlink(*, output, target_file, target_path, is_executable, progress_message)`.
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
            ],
            args,
            eval,
        )?;
        let output = bound[0].and_then(artifact_of).ok_or_else(|| {
            fatal("in call to symlink(), parameter 'output' got value of type that is not 'File'")
        })?;
        let Some(target) = bound[1].filter(|v| !v.is_none()).and_then(artifact_of) else {
            return Err(fatal(
                "ctx.actions.symlink: only target_file is supported yet (buildfiji-136.13)",
            ));
        };
        let message = optional_string("symlink", "progress_message", bound[4])?
            .or_else(|| Some(format!("Creating symlink {}", basename(&output))));
        s.register(
            "Symlink",
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
        let substitutions: Vec<(String, String)> =
            string_dict("expand_template", "substitutions", bound[2])?
                .into_iter()
                .collect();
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

fn basename(artifact: &Artifact) -> String {
    artifact
        .path
        .rsplit('/')
        .next()
        .unwrap_or(&artifact.path)
        .to_owned()
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
    let outputs = files_of(eval, function, "outputs", bound[0].expect("required"))?;
    if outputs.is_empty() {
        return Err(fatal(format!("{function}() requires at least one output")));
    }
    let mut inputs = match bound[1] {
        Some(v) => files_of(eval, function, "inputs", v)?,
        None => Vec::new(),
    };
    if let Some(v) = bound[2] {
        inputs.extend(files_of(eval, function, "tools", v)?);
    }
    let mut arguments: Vec<String> = Vec::new();
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
        env.insert("PATH".to_owned(), SHELL_PATH.to_owned());
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
            inputs.push(file.clone());
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
    s.register(
        &mnemonic,
        progress,
        ActionKind::Spawn {
            argv,
            env,
            execution_requirements,
        },
        inputs,
        outputs,
    );
    Ok(NoneType)
}
