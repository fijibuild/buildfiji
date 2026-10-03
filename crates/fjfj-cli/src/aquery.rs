//! What `aquery` prints of actions (buildfiji-89l): the text, `commands` and
//! `summary` outputs. Each was compared with Bazel 9.2.0 on a workspace with
//! a rule of every action kind.

use crate::build_command;
use fjfj_analysis::ConfiguredTarget;
use fjfj_exec::execroot::Layout;
use fjfj_graph::{Action, ActionKind, Artifact, Label};
use fjfj_query::ast::{Arg, Expr, Function};
use fjfj_query::proto::Msg;
use fjfj_starlark::AspectRef;
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
    /// `--include_aspects`: the actions of aspects applied to the targets.
    pub aspects: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            commandline: true,
            artifacts: true,
            file_write_contents: false,
            aspects: true,
        }
    }
}

/// An action and the configured target that registered it.
#[derive(Clone)]
pub(crate) struct Row<'a> {
    pub target: &'a ConfiguredTarget,
    pub action: std::borrow::Cow<'a, Action>,
    /// The aspect that registered it, if one did.
    pub aspect: Vec<AspectRef>,
    /// Where it runs, as the main repository names the platform.
    pub platform: String,
}

/// The actions of `target`, then those of the aspects applied to it.
pub(crate) fn rows_of<'a>(
    target: &'a ConfiguredTarget,
    aspects: &'a [std::sync::Arc<ConfiguredTarget>],
    settings: Settings,
    names: &dyn Names,
) -> Vec<Row<'a>> {
    let platform = names.label(
        &target
            .execution_platform
            .clone()
            .unwrap_or_else(host_platform),
    );
    // Bazel keeps the actions that write a parameter file out of the graph
    // it prints; they are part of the action that reads the file.
    let listed = |action: &&Action| action.mnemonic != "ParameterFileWrite";
    let own_platform = platform.clone();
    let own = target.actions.iter().filter(listed).map(move |action| Row {
        target,
        action: std::borrow::Cow::Borrowed(action),
        aspect: Vec::new(),
        platform: own_platform.clone(),
    });
    let made = aspects
        .iter()
        .filter(|_| settings.aspects)
        .flat_map(move |made| {
            let platform = platform.clone();
            let chain = made
                .aspect
                .as_ref()
                .map(|a| names.aspects(a))
                .unwrap_or_default();
            made.actions.iter().filter(listed).map(move |action| Row {
                target,
                action: std::borrow::Cow::Borrowed(action),
                aspect: chain.clone(),
                platform: platform.clone(),
            })
        });
    own.chain(made)
        .map(|row| expand_param_file(row, target, aspects))
        .flat_map(expand_runfiles)
        .collect()
}

/// A command line that reads its arguments from a file, as Bazel shows it: the
/// arguments of the file in its place, and the file not among the inputs.
/// The file is one that an action of the same target writes, a line to an
/// argument.
fn expand_param_file<'a>(
    mut row: Row<'a>,
    target: &ConfiguredTarget,
    aspects: &[std::sync::Arc<ConfiguredTarget>],
) -> Row<'a> {
    let ActionKind::Spawn { argv, .. } = &row.action.kind else {
        return row;
    };
    let written = |action: &Action| -> Option<(String, String)> {
        let ActionKind::WriteFile { contents, .. } = &action.kind else {
            return None;
        };
        (action.mnemonic == "ParameterFileWrite").then(|| {
            (
                action
                    .outputs
                    .first()
                    .map(Artifact::exec_path)
                    .unwrap_or_default(),
                String::from_utf8_lossy(contents).into_owned(),
            )
        })
    };
    let files: Vec<(String, String)> = target
        .actions
        .iter()
        .chain(aspects.iter().flat_map(|a| a.actions.iter()))
        .filter_map(written)
        .collect();
    let mut expanded = Vec::with_capacity(argv.len());
    let mut used: Vec<&str> = Vec::new();
    for arg in argv {
        match arg
            .strip_prefix('@')
            .and_then(|path| files.iter().find(|(p, _)| p == path))
        {
            Some((path, contents)) => {
                expanded.extend(contents.lines().map(str::to_owned));
                used.push(path);
            }
            None => expanded.push(arg.clone()),
        }
    }
    if used.is_empty() {
        return row;
    }
    let mut action = row.action.into_owned();
    if let ActionKind::Spawn { argv, .. } = &mut action.kind {
        *argv = expanded;
    }
    action
        .inputs
        .retain(|a| !used.contains(&a.exec_path().as_str()));
    row.action = std::borrow::Cow::Owned(action);
    row
}

/// What Bazel lists for the runfiles of an executable: an action that writes
/// the repository mapping, one that writes the manifest of the sources, one
/// that makes the tree of links and the `RunfilesTree` that stands for it.
/// fjfj does all that in one action, which it lists as the four.
fn expand_runfiles(row: Row<'_>) -> Vec<Row<'_>> {
    let ActionKind::RunfilesTree {
        dir,
        manifest,
        repo_mapping,
        repo_mapping_contents,
        entries,
        ..
    } = &row.action.kind
    else {
        return vec![row];
    };
    let find = |path: &str| row.action.outputs.iter().find(|a| a.exec_path() == path);
    let (Some(dir_file), Some(manifest_file), Some(mapping_file)) =
        (find(dir), find(manifest), find(repo_mapping))
    else {
        return vec![row];
    };
    let label = build_command::label_name(&row.target.label);
    let made = |mnemonic: &str,
                message: String,
                kind: ActionKind,
                inputs: Vec<Artifact>,
                outputs: Vec<Artifact>| Row {
        target: row.target,
        action: std::borrow::Cow::Owned(Action {
            owner: row.action.owner.clone(),
            owner_kind: row.action.owner_kind.clone(),
            location: row.action.location.clone(),
            configuration: row.action.configuration.clone(),
            mnemonic: mnemonic.to_owned(),
            progress_message: Some(message),
            kind,
            inputs,
            outputs,
        }),
        aspect: row.aspect.clone(),
        platform: row.platform.clone(),
    };
    let tree_manifest = Artifact {
        root: dir_file.root.clone(),
        path: format!("{}/MANIFEST", dir_file.path),
        tree: false,
    };
    let sources: String = entries
        .iter()
        .map(|(path, file)| format!("{path} {}\n", file.exec_path()))
        .collect();
    let mut runfiles_inputs = row.action.inputs.clone();
    runfiles_inputs.push(tree_manifest.clone());
    runfiles_inputs.push(mapping_file.clone());
    vec![
        made(
            "RepoMappingManifest",
            format!("Writing repo mapping manifest for {label}"),
            ActionKind::WriteFile {
                contents: repo_mapping_contents.clone().into_bytes(),
                executable: false,
            },
            Vec::new(),
            vec![mapping_file.clone()],
        ),
        made(
            "SourceSymlinkManifest",
            format!("Creating source manifest for {label}"),
            ActionKind::WriteFile {
                contents: sources.into_bytes(),
                executable: false,
            },
            Vec::new(),
            vec![manifest_file.clone()],
        ),
        made(
            "SymlinkTree",
            format!("Creating runfiles tree {}", dir_file.exec_path()),
            ActionKind::Symlink {
                target: String::new(),
            },
            vec![manifest_file.clone()],
            vec![tree_manifest],
        ),
        made(
            "RunfilesTree",
            format!("runfiles for {label}"),
            row.action.kind.clone(),
            runfiles_inputs,
            vec![dir_file.clone()],
        ),
    ]
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
        self.platform.clone()
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

/// How the names in the output are written, which only the graph knows.
pub(crate) trait Names {
    /// A label as the main repository names it.
    fn label(&self, label: &Label) -> String;
    /// The aspects an action's aspect stands for: the file that defines each,
    /// the aspect itself first and then those it requires.
    fn aspects(&self, aspect: &AspectRef) -> Vec<AspectRef>;
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
            .all(|c| c.is_ascii_alphanumeric() || "-_./:,+@%".contains(c));
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

/// The text of the template of a `Template` action, which is a file the action
/// reads: empty if it cannot be read (it is made by the build).
fn template_content(action: &Action, template: &str, layout: &Layout) -> String {
    action
        .inputs
        .iter()
        .find(|a| a.exec_path() == template)
        .and_then(|a| std::fs::read_to_string(layout.resolve(a)).ok())
        .unwrap_or_default()
}

/// An aspect as a descriptor names it: `//pkg:file.bzl%name` in the main
/// repository, `@@repo//pkg:file.bzl%name` in another.
fn aspect_name(aspect: &AspectRef) -> String {
    let bzl = &aspect.bzl;
    let repo = if bzl.repo.is_empty() {
        String::new()
    } else {
        format!("@@{}", bzl.repo)
    };
    format!("{repo}//{}:{}%{}", bzl.package, bzl.name, aspect.name)
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
    let action: &Action = &row.action;
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
    if !row.aspect.is_empty() {
        let chain: Vec<String> = row
            .aspect
            .iter()
            .map(|a| format!("{}()", aspect_name(a)))
            .collect();
        out.push_str(&format!(
            "  AspectDescriptors: [{}]\n",
            chain.join("\n    -> ")
        ));
    }
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
                // The comment gives the platform's canonical label.
                let canonical = row
                    .target
                    .execution_platform
                    .clone()
                    .unwrap_or_else(host_platform);
                out.push_str(&format!(
                    "# Execution platform: @@{}//{}:{}\n",
                    canonical.repo, canonical.package, canonical.name
                ));
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
            let content = template_content(action, template, layout);
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
        ActionKind::Symlink { .. } if action.mnemonic == "SymlinkTree" => {
            // The tree is made with the environment of a shell.
            let env: Vec<String> = row
                .target
                .configuration
                .default_shell_env()
                .iter()
                .map(|(k, v)| format!("{k}={v}"))
                .collect();
            out.push_str(&format!("  Environment: [{}]\n", env.join(", ")));
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
    let mut aspects: BTreeMap<String, usize> = BTreeMap::new();
    for row in rows {
        for aspect in &row.aspect {
            *aspects.entry(aspect_name(aspect)).or_default() += 1;
        }
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
        ("Aspects", &aspects),
    ] {
        if counts.is_empty() && title == "Aspects" {
            continue;
        }
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

/// One message of `analysis.ActionGraphContainer`'s repeated fields, in the
/// order the dump created it.
pub(crate) struct Piece {
    pub number: u32,
    pub name: &'static str,
    pub message: Msg,
}

/// The ids of what an action graph refers to, and the messages made so far.
#[derive(Default)]
struct Dump {
    pieces: Vec<Piece>,
    rule_classes: BTreeMap<String, i64>,
    targets: BTreeMap<(String, String), i64>,
    configurations: BTreeMap<String, i64>,
    aspects: BTreeMap<String, i64>,
    artifacts: BTreeMap<String, i64>,
    fragments: BTreeMap<String, i64>,
    dep_sets: BTreeMap<Vec<String>, i64>,
}

impl Dump {
    fn add(&mut self, number: u32, name: &'static str, message: Msg) {
        self.pieces.push(Piece {
            number,
            name,
            message,
        });
    }

    /// The id of `key` in `ids`, and whether it is new.
    fn id<K: Ord + Clone>(ids: &mut BTreeMap<K, i64>, key: &K) -> (i64, bool) {
        if let Some(id) = ids.get(key) {
            return (*id, false);
        }
        let id = ids.len() as i64 + 1;
        ids.insert(key.clone(), id);
        (id, true)
    }

    /// The id of the fragment of the path `path` (up to and including its
    /// last name). The leaf is numbered before its parents and written after
    /// them.
    fn fragment(&mut self, path: &str) -> i64 {
        if let Some(id) = self.fragments.get(path) {
            return *id;
        }
        let id = self.fragments.len() as i64 + 1;
        self.fragments.insert(path.to_owned(), id);
        let (parent, label) = match path.rsplit_once('/') {
            Some((parent, label)) => (Some(self.fragment(parent)), label),
            None => (None, path),
        };
        let mut message = Msg::new().one(1, "id", id).one(2, "label", label);
        if let Some(parent) = parent {
            message = message.one(3, "parent_id", parent);
        }
        self.add(8, "path_fragments", message);
        id
    }

    fn artifact(&mut self, artifact: &Artifact) -> i64 {
        let path = artifact.exec_path();
        if let Some(id) = self.artifacts.get(&path) {
            return *id;
        }
        let fragment = self.fragment(&path);
        let id = self.artifacts.len() as i64 + 1;
        self.artifacts.insert(path, id);
        let mut message = Msg::new()
            .one(1, "id", id)
            .one(2, "path_fragment_id", fragment);
        if artifact.tree {
            message = message.one(3, "is_tree_artifact", true);
        }
        self.add(1, "artifacts", message);
        id
    }

    fn action(&mut self, row: &Row<'_>, settings: Settings, layout: &Layout) {
        let action: &Action = &row.action;
        let target = row.target;
        let rule_class = target.rule_class.clone().unwrap_or_default();
        let label = build_command::label_name(&target.label);
        let (class_id, new) = Self::id(&mut self.rule_classes, &rule_class);
        if new {
            self.add(
                7,
                "rule_classes",
                Msg::new().one(1, "id", class_id).one(2, "name", rule_class),
            );
        }
        let (target_id, new) = Self::id(
            &mut self.targets,
            &(label.clone(), target.configuration.checksum()),
        );
        if new {
            self.add(
                3,
                "targets",
                Msg::new()
                    .one(1, "id", target_id)
                    .one(2, "label", label)
                    .one(3, "rule_class_id", class_id),
            );
        }
        let checksum = target.configuration.checksum();
        let (configuration_id, new) = Self::id(&mut self.configurations, &checksum);
        if new {
            let mut message = Msg::new()
                .one(1, "id", configuration_id)
                .one(2, "mnemonic", target.configuration.mnemonic())
                .one(3, "platform_name", target.configuration.cpu.clone())
                .one(4, "checksum", checksum);
            if target.configuration.exec {
                message = message.one(5, "is_tool", true);
            }
            self.add(5, "configuration", message);
        }
        let mut aspect_ids = Vec::new();
        for aspect in &row.aspect {
            let name = aspect_name(aspect);
            let (id, new) = Self::id(&mut self.aspects, &name);
            if new {
                self.add(
                    6,
                    "aspect_descriptors",
                    Msg::new().one(1, "id", id).one(2, "name", name),
                );
            }
            aspect_ids.push(id);
        }
        let mut input_sets = Vec::new();
        let mut outputs = Vec::new();
        if settings.artifacts {
            // In the order the action lists them, each once.
            let mut seen = std::collections::BTreeSet::new();
            let inputs: Vec<&Artifact> = action
                .inputs
                .iter()
                .filter(|a| seen.insert(a.exec_path()))
                .collect();
            let ids: Vec<i64> = inputs.iter().map(|a| self.artifact(a)).collect();
            if !ids.is_empty() {
                let paths: Vec<String> = inputs.iter().map(|a| a.exec_path()).collect();
                let (id, new) = Self::id(&mut self.dep_sets, &paths);
                if new {
                    self.add(
                        4,
                        "dep_set_of_files",
                        Msg::new()
                            .one(1, "id", id)
                            .packed(3, "direct_artifact_ids", ids),
                    );
                }
                input_sets.push(id);
            }
            outputs = action.outputs.iter().map(|a| self.artifact(a)).collect();
        }
        let mut message = Msg::new()
            .one(1, "target_id", target_id)
            .packed(2, "aspect_descriptor_ids", aspect_ids)
            .one(3, "action_key", action.key())
            .one(4, "mnemonic", action.mnemonic.as_str())
            .one(5, "configuration_id", configuration_id);
        let pair = |k: &str, v: &str| Msg::new().text_field(1, "key", k).text_field(2, "value", v);
        match &action.kind {
            ActionKind::Spawn {
                argv,
                env,
                execution_requirements,
            } => {
                if settings.commandline {
                    message = message.many(6, "arguments", argv.iter().cloned());
                }
                message = message
                    .many(
                        7,
                        "environment_variables",
                        env.iter().map(|(k, v)| pair(k, v)),
                    )
                    .many(
                        11,
                        "execution_info",
                        execution_requirements.iter().map(|(k, v)| pair(k, v)),
                    );
            }
            ActionKind::WriteFile {
                contents,
                executable,
            } => {
                if settings.file_write_contents {
                    message = message.text_field(
                        17,
                        "file_contents",
                        String::from_utf8_lossy(contents).into_owned(),
                    );
                }
                if *executable {
                    message = message.one(19, "is_executable", true);
                }
            }
            ActionKind::Template {
                template,
                substitutions,
                executable,
            } => {
                message = message
                    .one(
                        15,
                        "template_content",
                        template_content(action, template, layout),
                    )
                    .many(
                        16,
                        "substitutions",
                        substitutions.iter().map(|(k, v)| pair(k, v)),
                    );
                if *executable {
                    message = message.one(19, "is_executable", true);
                }
            }
            ActionKind::UnresolvedSymlink { target } => {
                message = message.text_field(18, "unresolved_symlink_target", target.as_str());
            }
            ActionKind::Symlink { .. } | ActionKind::RunfilesTree { .. } => {}
        }
        message = message.packed(8, "input_dep_set_ids", input_sets).packed(
            9,
            "output_ids",
            outputs.clone(),
        );
        if let Some(primary) = outputs.first() {
            message = message.one(13, "primary_output_id", *primary);
        }
        message = message.one(14, "execution_platform", row.execution_platform());
        self.add(2, "actions", message);
    }
}

/// The action graph of `rows` as the pieces `analysis.ActionGraphContainer`
/// is made of, in the order Bazel's dump makes them.
pub(crate) fn pieces(rows: &[Row<'_>], settings: Settings, layout: &Layout) -> Vec<Piece> {
    let mut dump = Dump::default();
    for row in rows {
        dump.action(row, settings, layout);
    }
    dump.pieces
}

/// `--output=proto`, `textproto`, `jsonproto` and `streamed_proto`.
pub(crate) fn proto(format: &str, pieces: &[Piece]) -> Vec<u8> {
    let whole = || {
        let mut container = Msg::new();
        for piece in pieces {
            container = container.many(piece.number, piece.name, [piece.message.clone()]);
        }
        container
    };
    match format {
        // Bazel writes the pieces one after another, as it makes them; the
        // result reads as one message.
        "proto" => pieces
            .iter()
            .flat_map(|p| {
                Msg::new()
                    .many(p.number, p.name, [p.message.clone()])
                    .binary()
            })
            .collect(),
        "jsonproto" => whole().json().into_bytes(),
        // Each message on its own, its fields at the left edge.
        "textproto" => pieces
            .iter()
            .map(|p| format!("{} {{\n{}}}\n", p.name, p.message.text()))
            .collect::<String>()
            .into_bytes(),
        _ => pieces
            .iter()
            .flat_map(|p| {
                Msg::new()
                    .many(p.number, p.name, [p.message.clone()])
                    .delimited()
            })
            .collect(),
    }
}
