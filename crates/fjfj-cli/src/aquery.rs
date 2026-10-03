//! What `aquery` prints of actions (buildfiji-89l): the text, `commands` and
//! `summary` outputs. Each was compared with Bazel 9.2.0 on a workspace with
//! a rule of every action kind.

use crate::build_command;
use fjfj_analysis::ConfiguredTarget;
use fjfj_exec::execroot::Layout;
use fjfj_graph::{Action, ActionKind, Artifact, Label};
use fjfj_query::ast::{Arg, Expr, Function};
use regex::Regex;
use std::collections::BTreeMap;

/// The `--include_*` flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Settings {
    /// `--include_commandline`.
    pub commandline: bool,
    /// `--include_artifacts`: the inputs and outputs.
    pub artifacts: bool,
    /// `--include_file_write_contents`.
    pub file_write_contents: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            commandline: true,
            artifacts: true,
            file_write_contents: false,
        }
    }
}

/// An action and the configured target that registered it.
#[derive(Clone, Copy)]
pub(crate) struct Row<'a> {
    pub target: &'a ConfiguredTarget,
    pub action: &'a Action,
}

/// The platform Bazel names when none was chosen.
fn host_platform() -> Label {
    Label {
        repo: "platforms".to_owned(),
        package: "host".to_owned(),
        name: "host".to_owned(),
    }
}

impl Row<'_> {
    /// `@@platforms//host:host`: where the action runs.
    pub(crate) fn execution_platform(&self) -> String {
        let platform = self
            .target
            .execution_platform
            .clone()
            .unwrap_or_else(host_platform);
        format!(
            "@@{}//{}:{}",
            platform.repo, platform.package, platform.name
        )
    }

    /// What the action says it is doing.
    fn description(&self) -> String {
        self.action.progress_message.clone().unwrap_or_else(|| {
            format!(
                "{} {}",
                self.action.mnemonic,
                build_command::label_name(&self.action.owner)
            )
        })
    }
}

/// The exec paths of `artifacts`, sorted, once each.
pub(crate) fn exec_paths(artifacts: &[Artifact]) -> Vec<String> {
    let mut paths: Vec<String> = artifacts.iter().map(Artifact::exec_path).collect();
    paths.sort();
    paths.dedup();
    paths
}

/// An argument as the shell would need it written.
pub(crate) fn shell_quote(arg: &str) -> String {
    let plain = !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./=:,+@%".contains(c));
    if plain {
        arg.to_owned()
    } else {
        format!("'{}'", arg.replace('\'', "'\\''"))
    }
}

fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, b)| n | (u32::from(*b) << (16 - 8 * i)));
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// A filter function of an expression: `inputs`, `outputs` or `mnemonic`, with
/// its regex, which must match the whole of a path or mnemonic.
pub(crate) struct ActionFilter {
    function: Function,
    pattern: Regex,
}

impl ActionFilter {
    pub(crate) fn keeps(&self, action: &Action) -> bool {
        let any = |artifacts: &[Artifact]| {
            artifacts
                .iter()
                .any(|a| self.pattern.is_match(&a.exec_path()))
        };
        match self.function {
            Function::Inputs => any(&action.inputs),
            Function::Outputs => any(&action.outputs),
            _ => self.pattern.is_match(&action.mnemonic),
        }
    }
}

/// The filters an expression asks for, and Bazel's complaint if it uses a
/// filter where it cannot. Bazel reads the filters off the top of the
/// expression only: the outermost function and the filter functions nested
/// in its expression argument. A filter under `+`, `^`, `-` or `let` still
/// passes the targets through but filters nothing.
pub(crate) fn action_filters(expr: &Expr) -> Result<Vec<ActionFilter>, String> {
    check_filters_only_nest_in_filters(expr)?;
    let mut filters = Vec::new();
    let mut at = expr;
    while let Expr::Call(call) = at {
        if !call.function.is_action_filter() {
            break;
        }
        let Some(Arg::Word(word)) = call.args.first() else {
            break;
        };
        let pattern = Regex::new(&format!("^(?:{word})$")).map_err(|e| {
            format!(
                "Wrong query syntax: {}",
                e.to_string().lines().last().unwrap_or("").trim()
            )
        })?;
        filters.push(ActionFilter {
            function: call.function,
            pattern,
        });
        match call.args.get(1) {
            Some(Arg::Expr(inner)) => at = inner,
            _ => break,
        }
    }
    Ok(filters)
}

fn check_filters_only_nest_in_filters(expr: &Expr) -> Result<(), String> {
    match expr {
        Expr::Binary(_, l, r) => {
            check_filters_only_nest_in_filters(l)?;
            check_filters_only_nest_in_filters(r)
        }
        Expr::Let { value, body, .. } => {
            check_filters_only_nest_in_filters(value)?;
            check_filters_only_nest_in_filters(body)
        }
        Expr::Call(call) => {
            for arg in &call.args {
                let Arg::Expr(inner) = arg else { continue };
                if !call.function.is_action_filter()
                    && let Expr::Call(inner_call) = inner
                    && inner_call.function.is_action_filter()
                {
                    return Err(format!(
                        "aquery filter functions (inputs, outputs, mnemonic) produce actions, and therefore can't be the input of other function types: {}",
                        call.function.name()
                    ));
                }
                check_filters_only_nest_in_filters(inner)?;
            }
            Ok(())
        }
        Expr::Word(_) | Expr::Variable(_) | Expr::Set(_) => Ok(()),
    }
}

/// `--output=text`.
pub(crate) fn text(rows: &[Row<'_>], settings: Settings, layout: &Layout) -> String {
    let mut out = String::new();
    for row in rows {
        text_one(&mut out, row, settings, layout);
    }
    out
}

fn text_one(out: &mut String, row: &Row<'_>, settings: Settings, layout: &Layout) {
    let action = row.action;
    let platform = row.execution_platform();
    let target = build_command::label_name(&row.target.label);
    if matches!(action.kind, ActionKind::RunfilesTree { .. }) {
        out.push_str(&format!("runfiles for {target}\n"));
    } else {
        out.push_str(&format!("action '{}'\n", row.description()));
    }
    out.push_str(&format!("  Mnemonic: {}\n", action.mnemonic));
    out.push_str(&format!("  Target: {target}\n"));
    out.push_str(&format!(
        "  Configuration: {}\n",
        row.target.configuration.mnemonic()
    ));
    out.push_str(&format!("  Execution platform: {platform}\n"));
    out.push_str(&format!("  ActionKey: {}\n", action.key()));
    if settings.artifacts {
        out.push_str(&format!(
            "  Inputs: [{}]\n",
            exec_paths(&action.inputs).join(", ")
        ));
        out.push_str(&format!(
            "  Outputs: [{}]\n",
            exec_paths(&action.outputs).join(", ")
        ));
    }
    match &action.kind {
        ActionKind::Spawn {
            argv,
            env,
            execution_requirements,
        } => {
            if !env.is_empty() {
                let list: Vec<String> = env.iter().map(|(k, v)| format!("{k}={v}")).collect();
                out.push_str(&format!("  Environment: [{}]\n", list.join(", ")));
            }
            if settings.commandline {
                let quoted: Vec<String> = argv.iter().map(|a| shell_quote(a)).collect();
                out.push_str(&format!(
                    "  Command Line: (exec {})\n",
                    quoted.join(" \\\n    ")
                ));
                out.push_str(&format!(
                    "# Configuration: {}\n",
                    row.target.configuration.checksum()
                ));
                out.push_str(&format!("# Execution platform: {platform}\n"));
            }
            if !execution_requirements.is_empty() {
                let list: Vec<String> = execution_requirements
                    .iter()
                    .map(|(k, v)| format!("{k}: {}", shell_quote(v)))
                    .collect();
                out.push_str(&format!("  ExecutionInfo: {{{}}}\n", list.join(", ")));
            }
        }
        ActionKind::WriteFile {
            contents,
            executable,
        } => {
            out.push_str(&format!("  IsExecutable: {executable}\n"));
            if settings.file_write_contents {
                out.push_str(&format!("  FileWriteContents: [{}]\n", base64(contents)));
            }
        }
        ActionKind::Template {
            template,
            substitutions,
            executable: _,
        } => {
            // The text of the template, which is a file the action reads.
            let content = action
                .inputs
                .iter()
                .find(|a| a.exec_path() == *template)
                .and_then(|a| std::fs::read_to_string(layout.resolve(a)).ok())
                .unwrap_or_default();
            out.push_str(&format!("  Template: {content}\n"));
            out.push_str("  Substitutions: [\n");
            for (key, value) in substitutions {
                out.push_str(&format!("    {{{key}: {value}}}\n"));
            }
            out.push_str("  ]\n");
        }
        ActionKind::UnresolvedSymlink { target } => {
            out.push_str(&format!("  UnresolvedSymlinkTarget: {target}\n"));
        }
        ActionKind::Symlink { .. } | ActionKind::RunfilesTree { .. } => {}
    }
    out.push('\n');
}

/// `--output=commands`: the command line of each action that has one.
pub(crate) fn commands(rows: &[Row<'_>]) -> String {
    let mut out = String::new();
    for row in rows {
        if let Some(argv) = row.action.argv() {
            let quoted: Vec<String> = argv.iter().map(|a| shell_quote(a)).collect();
            out.push_str(&quoted.join(" "));
            out.push('\n');
        }
    }
    out
}

/// `--output=summary`: how many actions there are, by mnemonic,
/// configuration and execution platform.
pub(crate) fn summary(rows: &[Row<'_>]) -> String {
    let mut mnemonics: BTreeMap<String, usize> = BTreeMap::new();
    let mut configurations: BTreeMap<String, usize> = BTreeMap::new();
    let mut platforms: BTreeMap<String, usize> = BTreeMap::new();
    for row in rows {
        *mnemonics.entry(row.action.mnemonic.clone()).or_default() += 1;
        *configurations
            .entry(row.target.configuration.mnemonic())
            .or_default() += 1;
        *platforms.entry(row.execution_platform()).or_default() += 1;
    }
    let mut out = format!(
        "{} total action{}.\n",
        rows.len(),
        if rows.len() == 1 { "" } else { "s" }
    );
    for (title, counts) in [
        ("Mnemonics", &mnemonics),
        ("Configurations", &configurations),
        ("Execution Platforms", &platforms),
    ] {
        out.push_str(&format!("\n{title}:\n"));
        for (name, count) in counts {
            out.push_str(&format!("  {name}: {count}\n"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_are_quoted_for_the_shell() {
        assert_eq!(shell_quote("-c"), "-c");
        assert_eq!(shell_quote("echo it's"), "'echo it'\\''s'");
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn contents_are_base64() {
        assert_eq!(base64(b"content"), "Y29udGVudA==");
        assert_eq!(base64(b"ab"), "YWI=");
        assert_eq!(base64(b""), "");
    }
}
