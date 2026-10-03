//! `fjfj cquery` and `fjfj aquery` (buildfiji-9s8.3): the expression is
//! evaluated over the loaded packages to a set of labels, those targets are
//! analysed (nothing is built), and the configured targets, or the actions
//! they registered, are printed.

use crate::build_command;
use crate::configured_graph::ConfiguredGraph;
use crate::fetch_command;
use crate::query_graph::QueryGraph;
use crate::{CliError, bzlmod_flags, locate_workspace_root};
use fjfj_analysis::ConfiguredTarget;
use fjfj_bazel_compat::QueryArgs;
use fjfj_graph::config::Configuration;
use fjfj_graph::{Action, ActionKind, Label};
use fjfj_query::{Evaluator, Graph, Options};
use std::collections::BTreeSet;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Cquery,
    Aquery,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Cquery => "cquery",
            Kind::Aquery => "aquery",
        }
    }

    /// The `--output` values, the first being the default.
    fn formats(self) -> &'static [&'static str] {
        match self {
            Kind::Cquery => &["label", "label_kind"],
            Kind::Aquery => &["text"],
        }
    }
}

fn bad(message: impl Into<String>) -> CliError {
    CliError::CommandLine(anyhow::anyhow!(message.into()))
}

/// `--output` and the query flags `query` shares, with the rest returned.
fn extract(kind: Kind, args: &[String]) -> Result<(String, Options, Vec<String>), CliError> {
    let mut format = kind.formats()[0].to_owned();
    let mut options = Options::default();
    let mut rest = Vec::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let Some(body) = arg.strip_prefix("--") else {
            rest.push(arg.clone());
            continue;
        };
        let (name, value) = match body.split_once('=') {
            Some((n, v)) => (n, Some(v.to_owned())),
            None => (body, None),
        };
        match name {
            "output" => {
                let value = value
                    .or_else(|| iter.next().cloned())
                    .ok_or_else(|| bad("--output needs a value"))?;
                if !kind.formats().contains(&value.as_str()) {
                    return Err(bad(format!(
                        "Invalid output format '{value}'. Valid values are: {}",
                        kind.formats().join(", ")
                    )));
                }
                format = value;
            }
            "implicit_deps" => options.implicit_deps = true,
            "noimplicit_deps" => options.implicit_deps = false,
            "tool_deps" => options.tool_deps = true,
            "notool_deps" => options.tool_deps = false,
            "keep_going" | "nokeep_going" => {}
            _ => rest.push(arg.clone()),
        }
    }
    Ok((format, options, rest))
}

pub(crate) async fn run(args: QueryArgs, kind: Kind) -> Result<(), CliError> {
    let command = kind.name();
    let (format, query_options, rest) = extract(kind, &args.expr)?;
    let (build_flags, rest) = fjfj_bazel_compat::build_flags::extract(&rest, command);
    let (bzlmod, rest) = bzlmod_flags::extract(&rest, command);
    let (fetch, rest) = fetch_command::extract(&rest)?;
    let implemented: Vec<&'static str> = [
        bzlmod_flags::IMPLEMENTED,
        fetch_command::IMPLEMENTED,
        fjfj_bazel_compat::build_flags::IMPLEMENTED,
        &["output", "implicit_deps", "tool_deps", "keep_going"],
    ]
    .iter()
    .flat_map(|s| s.iter().copied())
    .collect();
    fjfj_bazel_compat::clap_flags::validate(&rest, command, &implemented)
        .map_err(|e| CliError::CommandLine(anyhow::Error::from(e)))?;
    let text = rest.join(" ");
    let expr =
        fjfj_query::parse(&text).map_err(|e| bad(format!("Error while parsing '{text}': {e}")))?;
    let query = Query {
        kind,
        format,
        options: query_options,
        expr,
    };
    let configuration = build_command::configuration_from(&build_flags).map_err(bad)?;
    let workspace_root = locate_workspace_root(command)?;
    let module_bazel_text =
        std::fs::read_to_string(workspace_root.join("MODULE.bazel")).map_err(|e| {
            bad(format!(
                "no MODULE.bazel found in {}: {e}",
                workspace_root.display()
            ))
        })?;
    let output = tokio::task::spawn_blocking(move || -> Result<String, CliError> {
        let (resolved, repos) =
            fetch_command::begin(&fetch, &bzlmod, &workspace_root, &module_bazel_text)?;
        let repos = Arc::new(repos);
        let mut options = build_options(configuration, &build_flags);
        if let Some(platforms) = repos.module_repo("platforms") {
            for (setting, value) in fjfj_graph::config::host_constraints() {
                options.configuration.constraints.insert(Label {
                    repo: platforms.clone(),
                    package: setting.to_owned(),
                    name: value.to_owned(),
                });
            }
        }
        let layout = fjfj_exec::execroot::Layout {
            workspace: workspace_root.clone(),
            output_base: fetch
                .output_base
                .clone()
                .unwrap_or_else(|| fetch_command::default_output_base(&workspace_root)),
        };
        let text = evaluate(&query, &repos, options, layout)?;
        fetch_command::finish(resolved, &repos)?;
        Ok(text)
    })
    .await
    .map_err(|e| CliError::Internal(anyhow::anyhow!("{command} task panicked: {e}")))??;
    print!("{output}");
    Ok(())
}

/// `build`'s options for an analysis alone: nothing is built.
fn build_options(
    configuration: Configuration,
    flags: &fjfj_bazel_compat::build_flags::BuildFlags,
) -> build_command::Options {
    build_command::Options {
        configuration,
        platform: flags.platforms.clone(),
        extra_toolchains: flags.extra_toolchains.clone(),
        extra_execution_platforms: flags.extra_execution_platforms.clone(),
        host_platform: flags.host_platform.clone(),
        aspects: flags.aspects.clone(),
        output_groups: Vec::new(),
        keep_going: true,
        build: false,
        symlink_prefix: "bazel-".to_owned(),
        jobs: None,
        strategy: fjfj_exec::run::Options::default().strategy,
        show_result: 0,
        test: None,
    }
}

/// What was asked: the expression and how to print what it names.
struct Query {
    kind: Kind,
    format: String,
    options: Options,
    expr: fjfj_query::Expr,
}

/// Analyse the targets the expression names, run it over what that made, and
/// print the result.
fn evaluate(
    query: &Query,
    repos: &Arc<fjfj_repo::Repos>,
    options: build_command::Options,
    layout: fjfj_exec::execroot::Layout,
) -> Result<String, CliError> {
    let graph = QueryGraph::new(repos.clone());
    let mut named: BTreeSet<Label> = BTreeSet::new();
    for pattern in query.expr.patterns() {
        named.extend(
            graph
                .pattern(pattern)
                .map_err(|e| CliError::Query(anyhow::anyhow!(e)))?,
        );
    }
    let request = build_command::Request { layout, options };
    let targets: Vec<Label> = named.into_iter().collect();
    let report = build_command::run(repos, &targets, &request);
    if let Some((label, message)) = report.analysis_errors.first() {
        return Err(CliError::Query(anyhow::anyhow!(
            "{}: {message}",
            build_command::label_name(label)
        )));
    }
    let configured = ConfiguredGraph::new(&graph, &report.analysed);
    let labels = Evaluator::new(&configured, query.options)
        .eval(&query.expr)
        .map_err(|e| CliError::Query(anyhow::anyhow!(e)))?;
    Ok(render(query.kind, &query.format, &labels, &configured))
}

/// The configured targets `labels` stand for, in label then configuration order.
fn chosen<'a>(
    labels: &BTreeSet<Label>,
    graph: &'a ConfiguredGraph<'_>,
) -> Vec<&'a Arc<ConfiguredTarget>> {
    let mut chosen: Vec<_> = labels.iter().filter_map(|l| graph.target(l)).collect();
    chosen.sort_by_key(|t| {
        (
            build_command::label_name(&t.label),
            t.configuration.checksum(),
        )
    });
    chosen
}

fn render(
    kind: Kind,
    format: &str,
    labels: &BTreeSet<Label>,
    graph: &ConfiguredGraph<'_>,
) -> String {
    let mut out = String::new();
    match kind {
        Kind::Cquery => {
            for t in chosen(labels, graph) {
                // A source file has no configuration.
                let config = if t.rule_class.is_some() {
                    t.configuration.checksum()[..7].to_owned()
                } else {
                    "null".to_owned()
                };
                let name = build_command::label_name(&t.label);
                match (format, &t.rule_class) {
                    ("label_kind", Some(class)) => {
                        out.push_str(&format!("{class} rule {name} ({config})\n"))
                    }
                    ("label_kind", None) => {
                        out.push_str(&format!("source file {name} ({config})\n"))
                    }
                    _ => out.push_str(&format!("{name} ({config})\n")),
                }
            }
        }
        Kind::Aquery => {
            let mut actions: Vec<(&Action, &Configuration)> = chosen(labels, graph)
                .into_iter()
                .flat_map(|t| t.actions.iter().map(|a| (a, &t.configuration)))
                .collect();
            actions.sort_by_key(|(a, _)| {
                (
                    build_command::label_name(&a.owner),
                    a.mnemonic.clone(),
                    a.outputs.first().map(|o| o.exec_path()),
                )
            });
            for (action, configuration) in actions {
                aquery_text(&mut out, action, configuration);
            }
        }
    }
    out
}

fn aquery_text(out: &mut String, action: &Action, configuration: &Configuration) {
    let described = action.progress_message.clone().unwrap_or_else(|| {
        format!(
            "{} {}",
            action.mnemonic,
            build_command::label_name(&action.owner)
        )
    });
    out.push_str(&format!("action '{described}'\n"));
    out.push_str(&format!("  Mnemonic: {}\n", action.mnemonic));
    out.push_str(&format!(
        "  Owner: {}\n",
        build_command::label_name(&action.owner)
    ));
    out.push_str(&format!("  Configuration: {}\n", action.configuration));
    let list = |artifacts: &[fjfj_graph::Artifact]| {
        artifacts
            .iter()
            .map(|a| a.exec_path())
            .collect::<Vec<_>>()
            .join(", ")
    };
    out.push_str(&format!("  Inputs: [{}]\n", list(&action.inputs)));
    out.push_str(&format!("  Outputs: [{}]\n", list(&action.outputs)));
    if let ActionKind::Spawn { argv, env, .. } = &action.kind {
        if !env.is_empty() {
            out.push_str("  Environment Variables: [");
            out.push_str(
                &env.iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            out.push_str("]\n");
        }
        let quoted: Vec<String> = argv.iter().map(|a| shell_quote(a)).collect();
        out.push_str(&format!(
            "  Command Line: (exec {})\n",
            quoted.join(" \\\n    ")
        ));
    }
    out.push_str(&format!("# Configuration: {}\n", configuration.checksum()));
    out.push('\n');
}

/// An argument as the shell would need it written.
fn shell_quote(arg: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use fjfj_bzlmod::eval::{EvalOptions, eval_module_file};
    use std::collections::BTreeMap;

    /// `fjfj <kind> <expr>` on a workspace of `files`.
    fn query_on(files: &[(&str, &str)], kind: Kind, format: &str, expr: &str) -> String {
        let dir = tempfile::tempdir().unwrap();
        let ws = dir.path().join("ws");
        for (file, text) in files {
            let at = ws.join(file);
            std::fs::create_dir_all(at.parent().unwrap()).unwrap();
            std::fs::write(at, text).unwrap();
        }
        let module = eval_module_file("MODULE.bazel", "module(name = 'm')\n", &EvalOptions::root())
            .unwrap()
            .module;
        let repos = Arc::new(
            fjfj_repo::Repos::new(
                fjfj_repo::Options {
                    workspace_root: ws.clone(),
                    output_base: dir.path().join("ob"),
                    environ: BTreeMap::new(),
                    downloader: None,
                    repository_cache: None,
                    distdirs: Vec::new(),
                    registries: Vec::new(),
                    facts: Vec::new(),
                    repo_overrides: Vec::new(),
                },
                module,
            )
            .unwrap(),
        );
        let query = Query {
            kind,
            format: format.to_owned(),
            options: Options::default(),
            expr: fjfj_query::parse(expr).unwrap(),
        };
        let options = build_options(Configuration::default(), &Default::default());
        let layout = fjfj_exec::execroot::Layout {
            workspace: ws,
            output_base: dir.path().join("exec"),
        };
        evaluate(&query, &repos, options, layout).unwrap()
    }

    /// Two targets of one rule, the second reached through a transition that
    /// sets a build setting, which `//:b` is also built without.
    const TRANSITION: &[(&str, &str)] = &[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _t(settings, attr):
    return {"//:flag": "x"}
t = transition(implementation = _t, inputs = [], outputs = ["//:flag"])
def _r(ctx):
    out = ctx.actions.declare_file(ctx.label.name + ".txt")
    ctx.actions.run_shell(outputs = [out], inputs = ctx.files.srcs, command = "cat $@ > " + out.path, mnemonic = "Cat", env = {"A": "b"})
    return [DefaultInfo(files = depset([out]))]
r = rule(implementation = _r, attrs = {"srcs": attr.label_list(allow_files = True), "deps": attr.label_list(cfg = t)})
def _f(ctx):
    return []
flag_rule = rule(implementation = _f, build_setting = config.string(flag = True))
"#,
        ),
        (
            "BUILD",
            r#"
load(":defs.bzl", "r", "flag_rule")
flag_rule(name = "flag", build_setting_default = "d")
r(name = "a", srcs = ["s.txt"], deps = [":b"])
r(name = "b", srcs = ["s.txt"])
"#,
        ),
        ("s.txt", ""),
    ];

    /// Each line without its configuration digits.
    fn undigested(text: &str) -> Vec<String> {
        text.lines()
            .map(|l| match l.rsplit_once(" (") {
                Some((head, tail)) if tail.ends_with(')') && tail != "null)" => {
                    format!("{head} ({})", tail.len() - 1)
                }
                _ => l.to_owned(),
            })
            .collect()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cquery_lists_a_target_once_for_each_configuration() {
        let text =
            tokio::task::spawn_blocking(|| query_on(TRANSITION, Kind::Cquery, "label", "//:b"))
                .await
                .unwrap();
        assert_eq!(undigested(&text), ["//:b (7)"]);
        let text =
            tokio::task::spawn_blocking(|| query_on(TRANSITION, Kind::Cquery, "label", "//..."))
                .await
                .unwrap();
        // //:b by itself and //:b below //:a, which transitions it.
        assert_eq!(
            undigested(&text),
            ["//:a (7)", "//:b (7)", "//:b (7)", "//:flag (7)"]
        );
        let digits: Vec<&str> = text.lines().filter(|l| l.starts_with("//:b")).collect();
        assert_ne!(digits[0], digits[1]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn deps_follow_the_edge_into_the_configuration_the_transition_chose() {
        let all =
            tokio::task::spawn_blocking(|| query_on(TRANSITION, Kind::Cquery, "label", "//:b"))
                .await
                .unwrap();
        let below = tokio::task::spawn_blocking(|| {
            query_on(TRANSITION, Kind::Cquery, "label", "deps(//:a) - //:a")
        })
        .await
        .unwrap();
        assert_eq!(undigested(&below), ["//:b (7)", "//:s.txt (null)"]);
        assert_ne!(
            all.lines().next().unwrap(),
            below.lines().next().unwrap(),
            "//:b below //:a is not the //:b that was built on its own"
        );
    }

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn output_is_checked_against_the_commands_formats() {
        let (format, _, rest) =
            extract(Kind::Cquery, &args(&["--output=label_kind", "//a:b"])).unwrap();
        assert_eq!(format, "label_kind");
        assert_eq!(rest, ["//a:b"]);
        assert!(extract(Kind::Aquery, &args(&["--output=label"])).is_err());
        assert_eq!(extract(Kind::Aquery, &args(&["//a"])).unwrap().0, "text");
    }

    #[test]
    fn arguments_are_quoted_for_the_shell() {
        assert_eq!(shell_quote("-c"), "-c");
        assert_eq!(shell_quote("echo it's"), "'echo it'\\''s'");
        assert_eq!(shell_quote(""), "''");
    }
}
