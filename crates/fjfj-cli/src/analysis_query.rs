//! `fjfj cquery` and `fjfj aquery` (buildfiji-9s8.3): the expression is
//! evaluated over the loaded packages to a set of labels, those targets are
//! analysed (nothing is built), and the configured targets, or the actions
//! they registered, are printed.

use crate::aquery;
use crate::build_command;
use crate::configured_graph::ConfiguredGraph;
use crate::fetch_command;
use crate::query_graph::QueryGraph;
use crate::{CliError, bzlmod_flags, locate_workspace_root};
use fjfj_analysis::ConfiguredTarget;
use fjfj_bazel_compat::QueryArgs;
use fjfj_graph::Label;
use fjfj_graph::config::Configuration;
use fjfj_query::{Evaluator, Graph, Options};
use fjfj_starlark::RuleSource;
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

    /// The `--output` values, in the order Bazel lists them.
    fn formats(self) -> &'static [&'static str] {
        match self {
            Kind::Cquery => &[
                "label_kind",
                "label",
                "transitions",
                "proto",
                "streamed_proto",
                "textproto",
                "jsonproto",
                "build",
                "graph",
                "starlark",
                "files",
            ],
            Kind::Aquery => &[
                "proto",
                "streamed_proto",
                "textproto",
                "jsonproto",
                "text",
                "commands",
                "summary",
            ],
        }
    }

    fn dialect(self) -> fjfj_query::Dialect {
        match self {
            Kind::Cquery => fjfj_query::Dialect::Cquery,
            Kind::Aquery => fjfj_query::Dialect::Aquery,
        }
    }

    fn default_format(self) -> &'static str {
        match self {
            Kind::Cquery => "label",
            Kind::Aquery => "text",
        }
    }
}

fn bad(message: impl Into<String>) -> CliError {
    CliError::CommandLine(anyhow::anyhow!(message.into()))
}

/// The code `--output=starlark` runs.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Formatter {
    /// `--starlark:expr`, an expression of `target`.
    Expr(String),
    /// `--starlark:file`: its name and its text, defining `format(target)`.
    File(String, String),
}

/// What `--output` and the flags around it asked for.
#[derive(Debug)]
struct Flags {
    format: String,
    options: Options,
    expr: Option<String>,
    file: Option<String>,
    aquery: aquery::Settings,
    proto: fjfj_query::target_proto::ProtoOptions,
}

impl Flags {
    /// An `aquery` flag that is `--[no]name`, `--name=true` or `--name=false`.
    fn set_aquery(&mut self, name: &str, value: Option<&str>) -> bool {
        let (base, negated) = match name.strip_prefix("no") {
            Some(base) if value.is_none() => (base, true),
            _ => (name, false),
        };
        let on = match value {
            Some("false" | "0" | "no") => false,
            Some(_) | None => !negated,
        };
        match base {
            "include_commandline" => self.aquery.commandline = on,
            "include_artifacts" => self.aquery.artifacts = on,
            "include_file_write_contents" => self.aquery.file_write_contents = on,
            "include_aspects" => self.aquery.aspects = on,
            // No action of fjfj has a parameter file.
            "include_param_files" => {}
            _ => return false,
        }
        true
    }
}

impl Flags {
    fn formatter(&self) -> Result<Formatter, CliError> {
        match (&self.expr, &self.file) {
            (Some(_), Some(_)) => Err(CliError::Query(anyhow::anyhow!(
                "You must not specify both --starlark:expr and --starlark:file"
            ))),
            (Some(expr), None) => Ok(Formatter::Expr(expr.clone())),
            (None, Some(file)) => std::fs::read_to_string(file)
                .map(|text| Formatter::File(file.clone(), text))
                .map_err(|_| {
                    CliError::Query(anyhow::anyhow!(
                        "invalid --starlark:file: failed to read {file}"
                    ))
                }),
            (None, None) => Ok(Formatter::Expr("str(target.label)".to_owned())),
        }
    }
}

/// `--output` and the query flags `query` shares, with the rest returned.
fn extract(kind: Kind, args: &[String]) -> Result<(Flags, Vec<String>), CliError> {
    let mut flags = Flags {
        format: kind.default_format().to_owned(),
        options: Options::default(),
        expr: None,
        file: None,
        aquery: aquery::Settings::default(),
        proto: Default::default(),
    };
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
                flags.format = value;
            }
            "starlark:expr" if kind == Kind::Cquery => {
                flags.expr = Some(value.ok_or_else(|| bad("--starlark:expr needs a value"))?);
            }
            "starlark:file" if kind == Kind::Cquery => {
                flags.file = Some(value.ok_or_else(|| bad("--starlark:file needs a value"))?);
            }
            "implicit_deps" => flags.options.implicit_deps = true,
            "noimplicit_deps" => flags.options.implicit_deps = false,
            "tool_deps" => flags.options.tool_deps = true,
            "notool_deps" => flags.options.tool_deps = false,
            "keep_going" | "nokeep_going" => {}
            n if kind == Kind::Cquery && (n.starts_with("proto:") || n.starts_with("noproto:")) => {
                let value = if n.ends_with("output_rule_attrs") {
                    value.or_else(|| iter.next().cloned())
                } else {
                    value
                };
                if !flags.proto.flag(n, value.as_deref()).map_err(bad)? {
                    rest.push(arg.clone());
                }
            }
            _ if kind == Kind::Aquery && flags.set_aquery(name, value.as_deref()) => {}
            _ => rest.push(arg.clone()),
        }
    }
    Ok((flags, rest))
}

pub(crate) async fn run(args: QueryArgs, kind: Kind) -> Result<(), CliError> {
    let command = kind.name();
    let (flags, rest) = extract(kind, &args.expr)?;
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
    let expr = fjfj_query::parse_in(&text, kind.dialect())
        .map_err(|e| bad(format!("Error while parsing '{text}': {e}")))?;
    let formatter = (flags.format == "starlark")
        .then(|| flags.formatter())
        .transpose()?;
    let query = Query {
        kind,
        format: flags.format,
        options: flags.options,
        formatter,
        aquery: flags.aquery,
        proto: flags.proto,
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
    let output = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, CliError> {
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
    use std::io::Write;
    std::io::stdout()
        .write_all(&output)
        .map_err(|e| CliError::Internal(anyhow::anyhow!("writing the result failed: {e}")))?;
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
        record_execution_platforms: true,
        test: None,
    }
}

/// What was asked: the expression and how to print what it names.
struct Query {
    kind: Kind,
    format: String,
    options: Options,
    /// For `--output=starlark`.
    formatter: Option<Formatter>,
    aquery: aquery::Settings,
    proto: fjfj_query::target_proto::ProtoOptions,
    expr: fjfj_query::Expr,
}

/// Analyse the targets the expression names, run it over what that made, and
/// print the result.
fn evaluate(
    query: &Query,
    repos: &Arc<fjfj_repo::Repos>,
    options: build_command::Options,
    layout: fjfj_exec::execroot::Layout,
) -> Result<Vec<u8>, CliError> {
    let graph = QueryGraph::new(repos.clone());
    let mut named: BTreeSet<Label> = BTreeSet::new();
    for pattern in query.expr.patterns() {
        named.extend(
            graph
                .pattern(pattern)
                .map_err(|e| CliError::Query(anyhow::anyhow!(e)))?,
        );
    }
    let top_level = options.configuration.clone();
    let request = build_command::Request {
        layout: layout.clone(),
        options,
    };
    let targets: Vec<Label> = named.into_iter().collect();
    let report = build_command::run(repos, &targets, &request);
    if let Some((label, message)) = report.analysis_errors.first() {
        return Err(CliError::Query(anyhow::anyhow!(
            "{}: {message}",
            build_command::label_name(label)
        )));
    }
    let configured = ConfiguredGraph::new(&graph, &report.analysed, &top_level);
    let evaluator = Evaluator::new(&configured, query.options);
    let labels = evaluator
        .eval(&query.expr)
        .map_err(|e| CliError::Query(anyhow::anyhow!("Error doing post analysis query: {e}")))?;
    if query.kind == Kind::Aquery
        && matches!(
            query.format.as_str(),
            "proto" | "streamed_proto" | "textproto" | "jsonproto"
        )
    {
        let filters =
            aquery::action_filters(&query.expr).map_err(|e| CliError::Query(anyhow::anyhow!(e)))?;
        let rows: Vec<aquery::Row<'_>> = chosen(&labels, &configured)
            .into_iter()
            .flat_map(|target| aquery::rows_of(target, configured.aspects_of(target), query.aquery))
            .filter(|row| filters.iter().all(|f| f.keeps(row.action)))
            .collect();
        let pieces = aquery::pieces(&rows, query.aquery, &layout);
        return Ok(aquery::proto(&query.format, &pieces));
    }
    if query.kind == Kind::Cquery
        && matches!(
            query.format.as_str(),
            "proto" | "streamed_proto" | "textproto" | "jsonproto"
        )
    {
        return cquery_proto(query, &labels, &evaluator, &configured);
    }
    render(query, &labels, &evaluator, &configured, repos, &layout).map(String::into_bytes)
}

/// `cquery --output=proto` and its textual forms: a `CqueryResult` of the
/// targets, each with the checksum of its configuration, and the
/// configurations they are in.
fn cquery_proto(
    query: &Query,
    labels: &BTreeSet<Label>,
    evaluator: &Evaluator<'_>,
    graph: &ConfiguredGraph<'_>,
) -> Result<Vec<u8>, CliError> {
    use fjfj_query::proto::Msg;
    let failed = |e: String| CliError::Query(anyhow::anyhow!(e));
    let mut results: Vec<Msg> = Vec::new();
    // A configuration is numbered where its first target is.
    let mut configurations: Vec<(String, Msg)> = Vec::new();
    for label in labels {
        let Some(target) = graph.target(label) else {
            continue;
        };
        let message =
            fjfj_query::target_proto::target(evaluator, label, &query.proto).map_err(failed)?;
        let mut result = Msg::new().one(1, "target", message);
        if target.has_configuration() {
            let checksum = target.configuration.checksum();
            let at = match configurations.iter().position(|(c, _)| *c == checksum) {
                Some(at) => at,
                None => {
                    // Proto3: `is_tool` is there only when it is true.
                    let mut configuration = Msg::new()
                        .one(1, "id", configurations.len() as i64 + 1)
                        .one(2, "mnemonic", target.configuration.mnemonic())
                        .one(3, "platform_name", target.configuration.cpu.clone())
                        .one(4, "checksum", checksum.clone());
                    if target.configuration.exec {
                        configuration = configuration.one(5, "is_tool", true);
                    }
                    configurations.push((checksum.clone(), configuration));
                    configurations.len() - 1
                }
            };
            result = result
                .one(2, "configuration", Msg::new().one(4, "checksum", checksum))
                .one(3, "configuration_id", at as i64 + 1);
        } else {
            result = result.one(2, "configuration", Msg::new().one(4, "checksum", "null"));
        }
        results.push(result);
    }
    let configurations: Vec<Msg> = configurations.into_iter().map(|(_, m)| m).collect();
    let whole = || {
        Msg::new().many(1, "results", results.clone()).many(
            2,
            "configurations",
            configurations.clone(),
        )
    };
    Ok(match query.format.as_str() {
        "proto" => whole().binary(),
        "textproto" => whole().text().into_bytes(),
        "jsonproto" => whole().json().into_bytes(),
        // Each result in a message of its own, and the configurations in one
        // after them.
        _ => {
            let mut out: Vec<u8> = results
                .iter()
                .flat_map(|r| Msg::new().many(1, "results", [r.clone()]).delimited())
                .collect();
            if !configurations.is_empty() {
                out.extend(
                    Msg::new()
                        .many(2, "configurations", configurations.clone())
                        .delimited(),
                );
            }
            out
        }
    })
}

/// The configured targets `labels` stand for, in label then configuration order.
fn chosen<'a>(
    labels: &BTreeSet<Label>,
    graph: &'a ConfiguredGraph<'_>,
) -> Vec<&'a Arc<ConfiguredTarget>> {
    labels.iter().filter_map(|l| graph.target(l)).collect()
}

fn render(
    query: &Query,
    labels: &BTreeSet<Label>,
    evaluator: &Evaluator<'_>,
    graph: &ConfiguredGraph<'_>,
    repos: &fjfj_repo::Repos,
    layout: &fjfj_exec::execroot::Layout,
) -> Result<String, CliError> {
    let failed = |e: String| CliError::Query(anyhow::anyhow!(e));
    match query.kind {
        Kind::Cquery => match query.format.as_str() {
            "label" | "label_kind" | "graph" | "build" => {
                let format = fjfj_query::output::Format::parse(&query.format)
                    .expect("a format the query command has");
                fjfj_query::output::render(
                    evaluator,
                    labels,
                    format,
                    fjfj_query::output::Order::Auto,
                    '\n',
                )
                .map_err(failed)
            }
            "files" => {
                let mut out = String::new();
                for target in chosen(labels, graph) {
                    for file in target.files.to_vec() {
                        out.push_str(&file.exec_path());
                        out.push('\n');
                    }
                }
                Ok(out)
            }
            "starlark" => {
                let default = Formatter::Expr("str(target.label)".to_owned());
                let formatter = query.formatter.as_ref().unwrap_or(&default);
                let (name, source, expr) = match formatter {
                    Formatter::Expr(expr) => ("--starlark:expr", expr.as_str(), true),
                    Formatter::File(name, text) => (name.as_str(), text.as_str(), false),
                };
                let targets: Vec<fjfj_starlark::FormatTarget> = chosen(labels, graph)
                    .into_iter()
                    .map(|t| fjfj_starlark::FormatTarget {
                        info: Arc::new(fjfj_analysis::dep_info(
                            t,
                            t.rule_class.is_none()
                                && t.files.to_vec().first().is_some_and(|f| !f.is_source()),
                        )),
                        build_options: t.configuration.build_options().into_iter().collect(),
                    })
                    .collect();
                let lines =
                    fjfj_starlark::format_targets(name, source, expr, &repos.mappings(), &targets)
                        .map_err(failed)?;
                Ok(lines.into_iter().map(|l| l + "\n").collect())
            }
            "transitions" => Err(CliError::Query(anyhow::anyhow!(
                "Instead of using --output=transitions, set the --transitions flag explicitly to 'lite' or 'full'"
            ))),
            other => Err(CliError::Query(anyhow::anyhow!(
                "--output={other} is not implemented yet"
            ))),
        },
        Kind::Aquery => {
            let filters = aquery::action_filters(&query.expr).map_err(failed)?;
            let rows: Vec<aquery::Row<'_>> = chosen(labels, graph)
                .into_iter()
                .flat_map(|target| aquery::rows_of(target, graph.aspects_of(target), query.aquery))
                .filter(|row| filters.iter().all(|f| f.keeps(row.action)))
                .collect();
            match query.format.as_str() {
                "text" => Ok(aquery::text(&rows, query.aquery, layout)),
                "commands" => Ok(aquery::commands(&rows)),
                "summary" => Ok(aquery::summary(&rows)),
                other => Err(CliError::Query(anyhow::anyhow!(
                    "--output={other} is not implemented yet"
                ))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fjfj_bzlmod::eval::{EvalOptions, eval_module_file};
    use std::collections::BTreeMap;

    /// `fjfj <kind> <expr>` on a workspace of `files`.
    fn query_on(
        files: &[(&str, &str)],
        kind: Kind,
        format: &str,
        expr: &str,
        formatter: Option<Formatter>,
        aspects: &[&str],
        settings: aquery::Settings,
    ) -> Result<Vec<u8>, CliError> {
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
            formatter,
            aquery: settings,
            proto: Default::default(),
            expr: fjfj_query::parse_in(expr, kind.dialect()).unwrap(),
        };
        let flags = fjfj_bazel_compat::build_flags::BuildFlags {
            aspects: aspects.iter().map(|a| (*a).to_owned()).collect(),
            ..Default::default()
        };
        let options = build_options(Configuration::default(), &flags);
        let layout = fjfj_exec::execroot::Layout {
            workspace: ws,
            output_base: dir.path().join("exec"),
        };
        evaluate(&query, &repos, options, layout)
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
    ctx.actions.run_shell(outputs = [out], inputs = ctx.files.srcs, command = "cat $@ > " + out.path, mnemonic = "Cat", progress_message = "Cat %{label}", env = {"A": "b"})
    return [DefaultInfo(files = depset([out]))]
r = rule(implementation = _r, attrs = {"srcs": attr.label_list(allow_files = True), "deps": attr.label_list(cfg = t), "ss": attr.string_list()})
def _asp(target, ctx):
    out = ctx.actions.declare_file(target.label.name + ".asp")
    ctx.actions.run_shell(outputs = [out], command = "echo hi > " + out.path, mnemonic = "AspAct", progress_message = "Asp %{label}")
    return [OutputGroupInfo(asp = depset([out]))]
asp = aspect(implementation = _asp, attr_aspects = ["deps"])
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
config_setting(name = "fast", values = {"compilation_mode": "fastbuild"})
r(name = "sel", ss = select({":fast": ["f"], "//conditions:default": ["d"]}))
"#,
        ),
        ("s.txt", ""),
    ];

    /// The text with each configuration's seven digits replaced by `7`.
    fn undigested_text(text: &str) -> String {
        let mut out = String::new();
        let mut rest = text;
        while let Some(at) = rest.find('(') {
            out.push_str(&rest[..=at]);
            rest = &rest[at + 1..];
            let hex = rest.chars().take_while(char::is_ascii_hexdigit).count();
            if hex == 7 && rest[7..].starts_with(')') {
                out.push('7');
                rest = &rest[7..];
            }
        }
        out.push_str(rest);
        out
    }

    fn undigested(text: &str) -> Vec<String> {
        undigested_text(text).lines().map(str::to_owned).collect()
    }

    async fn run_query(
        kind: Kind,
        format: &'static str,
        expr: &'static str,
        formatter: Option<Formatter>,
    ) -> Result<String, CliError> {
        run_bytes(kind, format, expr, formatter)
            .await
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }

    async fn run_bytes(
        kind: Kind,
        format: &'static str,
        expr: &'static str,
        formatter: Option<Formatter>,
    ) -> Result<Vec<u8>, CliError> {
        tokio::task::spawn_blocking(move || {
            query_on(
                TRANSITION,
                kind,
                format,
                expr,
                formatter,
                &[],
                aquery::Settings::default(),
            )
        })
        .await
        .unwrap()
    }

    async fn cquery(expr: &'static str) -> String {
        run_query(Kind::Cquery, "label", expr, None).await.unwrap()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cquery_lists_a_target_once_for_each_configuration() {
        assert_eq!(undigested(&cquery("//:b").await), ["//:b (7)"]);
        let text = cquery("//...").await;
        // //:b by itself and //:b below //:a, which transitions it.
        assert_eq!(
            undigested(&text),
            [
                "//:a (7)",
                "//:b (7)",
                "//:b (7)",
                "//:fast (7)",
                "//:flag (7)",
                "//:sel (7)"
            ]
        );
        let digits: Vec<&str> = text.lines().filter(|l| l.starts_with("//:b")).collect();
        assert_ne!(digits[0], digits[1]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn deps_follow_the_edge_into_the_configuration_the_transition_chose() {
        let all = cquery("//:b").await;
        let below = cquery("deps(//:a) - //:a").await;
        assert_eq!(undigested(&below), ["//:b (7)", "//:s.txt (null)"]);
        assert_ne!(
            all.lines().next().unwrap(),
            below.lines().next().unwrap(),
            "//:b below //:a is not the //:b that was built on its own"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cquery_prints_kinds_graphs_and_files() {
        let kinds = run_query(Kind::Cquery, "label_kind", "//:a + //:s.txt", None)
            .await
            .unwrap();
        assert_eq!(
            undigested(&kinds),
            ["r rule //:a (7)", "source file //:s.txt (null)"]
        );
        let graph = run_query(Kind::Cquery, "graph", "deps(//:a)", None)
            .await
            .unwrap();
        let lines = undigested(&graph);
        assert_eq!(lines[..2], ["digraph mygraph {", "  node [shape=box];"]);
        // The edges of a node by the name of what they reach.
        assert_eq!(
            lines[3..5],
            [
                "  \"//:a (7)\" -> \"//:b (7)\"",
                "  \"//:a (7)\" -> \"//:s.txt (null)\""
            ]
        );
        let files = run_query(Kind::Cquery, "files", "//:a + //:s.txt", None)
            .await
            .unwrap();
        assert_eq!(files, "bazel-out/k8-fastbuild/bin/a.txt\ns.txt\n");
        let error = run_query(Kind::Cquery, "transitions", "//:a", None)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("set the --transitions flag"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn starlark_output_formats_each_target_with_the_users_code() {
        let expr = |code: &str| Some(Formatter::Expr(code.to_owned()));
        let by_default = run_query(Kind::Cquery, "starlark", "//:a", None)
            .await
            .unwrap();
        assert_eq!(by_default, "@@//:a\n");
        let names = run_query(
            Kind::Cquery,
            "starlark",
            "//:a + //:b",
            expr("target.label.name + ':' + str(len(providers(target)))"),
        )
        .await
        .unwrap();
        // //:b is also a dependency of //:a, in a configuration of its own.
        assert_eq!(names, "a:1\nb:1\nb:1\n");
        // Not a string: shown as str() shows it.
        let number = run_query(Kind::Cquery, "starlark", "//:a", expr("1 + 1"))
            .await
            .unwrap();
        assert_eq!(number, "2\n");
        let cpu = run_query(
            Kind::Cquery,
            "starlark",
            "//:a",
            expr("build_options(target)['//command_line_option:cpu']"),
        )
        .await
        .unwrap();
        assert_eq!(cpu, format!("{}\n", fjfj_graph::config::host_cpu()));
        let file = Some(Formatter::File(
            "fmt.bzl".to_owned(),
            "def format(target):\n  return 'F:' + target.label.name\n".to_owned(),
        ));
        let from_file = run_query(Kind::Cquery, "starlark", "//:a", file)
            .await
            .unwrap();
        assert_eq!(from_file, "F:a\n");
        let broken = run_query(Kind::Cquery, "starlark", "//:a", expr("target.nope"))
            .await
            .unwrap_err();
        assert!(
            broken
                .to_string()
                .contains("Starlark evaluation error for //:a"),
            "{broken}"
        );
    }

    /// The text with each 64-digit hash (an `ActionKey`, a configuration's
    /// checksum) replaced by `HASH`.
    fn unhashed(text: &str) -> String {
        let mut out = String::new();
        let mut run = String::new();
        for c in text.chars().chain(std::iter::once('\u{0}')) {
            if c.is_ascii_hexdigit() {
                run.push(c);
                continue;
            }
            out.push_str(&if run.len() == 64 {
                "HASH".to_owned()
            } else {
                run.clone()
            });
            run.clear();
            if c != '\u{0}' {
                out.push(c);
            }
        }
        out
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn aquery_prints_actions_as_bazel_does() {
        let text = run_query(Kind::Aquery, "text", "//:a", None).await.unwrap();
        // What `bazel aquery //:a` printed for the same files.
        assert_eq!(
            unhashed(&text),
            "action 'Cat //:a'
  Mnemonic: Cat
  Target: //:a
  Configuration: k8-fastbuild
  Execution platform: @@platforms//host:host
  ActionKey: HASH
  Inputs: [s.txt]
  Outputs: [bazel-out/k8-fastbuild/bin/a.txt]
  Environment: [A=b]
  Command Line: (exec /bin/bash \\
    -c \\
    'cat $@ > bazel-out/k8-fastbuild/bin/a.txt')
# Configuration: HASH
# Execution platform: @@platforms//host:host

"
            .replace(
                "k8-fastbuild",
                &format!("{}-fastbuild", fjfj_graph::config::host_cpu())
            )
        );
        let commands = run_query(Kind::Aquery, "commands", "//:a", None)
            .await
            .unwrap();
        assert_eq!(
            commands,
            format!(
                "/bin/bash -c 'cat $@ > bazel-out/{}-fastbuild/bin/a.txt'\n",
                fjfj_graph::config::host_cpu()
            )
        );
        let summary = run_query(Kind::Aquery, "summary", "deps(//:a)", None)
            .await
            .unwrap();
        assert!(
            summary.starts_with("2 total actions.\n\nMnemonics:\n  Cat: 2\n"),
            "{summary}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn config_selects_a_configuration_as_bazel_does() {
        // The top-level configuration has //:b, the other is below //:a.
        let all = cquery("//:b + //:a").await;
        assert_eq!(all.lines().count(), 3);
        assert_eq!(
            undigested(&cquery("config(//:b, target)").await),
            ["//:b (7)"]
        );
        let below = cquery("config(deps(//:a), null)").await;
        assert_eq!(undigested(&below), ["//:s.txt (null)"]);
        // A short checksum names the configuration of the dependency.
        let transitioned = all
            .lines()
            .filter(|l| l.starts_with("//:b"))
            .find(|l| !cquery_top_level_line(l, &all))
            .unwrap()
            .to_owned();
        let digits = transitioned
            .rsplit_once('(')
            .unwrap()
            .1
            .trim_end_matches(')')
            .to_owned();
        let query: &'static str =
            Box::leak(format!("config(//:b + //:a, {digits})").into_boxed_str());
        assert_eq!(cquery(query).await, format!("{transitioned}\n"));
        let error = |expr: &'static str| async move {
            run_query(Kind::Cquery, "label", expr, None)
                .await
                .unwrap_err()
                .to_string()
        };
        assert_eq!(
            error("config(//:b, host)").await,
            "Error doing post analysis query: Evaluation failed: 'host' configuration no longer exists. Use a specific configuration hash instead"
        );
        assert_eq!(
            error("config(//:b + //:a, null)").await,
            "Error doing post analysis query: Evaluation failed: No target (in) (//:b + //:a) could be found in the 'null' configuration"
        );
        assert!(
            error("config(//:b, nonsense)")
                .await
                .contains("Unknown configuration ID 'nonsense'.\nconfig()'s second argument must identify a unique configuration.")
        );
    }

    /// Whether `line` of `all` is the one `config(.., target)` gives.
    fn cquery_top_level_line(line: &str, all: &str) -> bool {
        // //:a is analysed in the top-level configuration, and so is the
        // //:b that was asked for by itself: they share their digits.
        let digits = |l: &str| l.rsplit_once('(').map(|(_, d)| d.to_owned());
        let a = all.lines().find(|l| l.starts_with("//:a")).unwrap();
        digits(line) == digits(a)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn aquery_lists_the_actions_aspects_registered_after_the_targets() {
        let with = |settings: aquery::Settings| {
            tokio::task::spawn_blocking(move || {
                query_on(
                    TRANSITION,
                    Kind::Aquery,
                    "text",
                    "deps(//:a) - //:s.txt",
                    None,
                    &["//:defs.bzl%asp"],
                    settings,
                )
                .map(|bytes| String::from_utf8(bytes).unwrap())
                .unwrap()
            })
        };
        let text = with(aquery::Settings::default()).await.unwrap();
        // //:a, then what the aspect made of it, then //:b the same way: the
        // aspect follows `deps`.
        assert_eq!(
            actions(&text),
            [
                "action 'Cat //:a'",
                "action 'Asp //:a'",
                "action 'Cat //:b'",
                "action 'Asp //:b'"
            ]
        );
        assert_eq!(
            text.matches("  AspectDescriptors: [//:defs.bzl%asp()]\n")
                .count(),
            2
        );
        let without = with(aquery::Settings {
            aspects: false,
            ..Default::default()
        })
        .await
        .unwrap();
        assert_eq!(
            actions(&without),
            ["action 'Cat //:a'", "action 'Cat //:b'"]
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn attr_sees_the_branch_a_select_took_in_the_configuration() {
        // The default configuration is fastbuild: `:fast` matches.
        assert_eq!(
            undigested(&cquery("attr(ss, f, //:sel)").await),
            ["//:sel (7)"]
        );
        assert!(cquery("attr(ss, d, //:sel)").await.is_empty());
        // The condition is a dependency, the branch's labels are not.
        let deps = cquery("deps(//:sel)").await;
        assert!(deps.contains("//:fast"), "{deps}");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cquery_proto_is_a_cquery_result_with_each_targets_configuration() {
        let json = run_query(Kind::Cquery, "jsonproto", "//:a + //:s.txt", None)
            .await
            .unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let results = value["results"].as_array().unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["target"]["rule"]["name"], "//:a");
        assert_eq!(results[0]["configurationId"], 1);
        let checksum = results[0]["configuration"]["checksum"].as_str().unwrap();
        assert_eq!(checksum.len(), 64);
        // A source file has no configuration and no id.
        assert_eq!(results[1]["target"]["sourceFile"]["name"], "//:s.txt");
        assert_eq!(results[1]["configuration"]["checksum"], "null");
        assert!(results[1].get("configurationId").is_none());
        let configurations = value["configurations"].as_array().unwrap();
        assert_eq!(configurations.len(), 1);
        assert_eq!(configurations[0]["id"], 1);
        assert_eq!(configurations[0]["checksum"], checksum);
        assert_eq!(
            configurations[0]["mnemonic"],
            format!("{}-fastbuild", fjfj_graph::config::host_cpu())
        );
        // proto3: a tool that is not one has no `isTool`.
        assert!(configurations[0].get("isTool").is_none());
        // The same message as text.
        let text = run_query(Kind::Cquery, "textproto", "//:s.txt", None)
            .await
            .unwrap();
        assert!(
            text.starts_with(
                "results {\n  target {\n    type: SOURCE_FILE\n    source_file {\n      name: \"//:s.txt\"\n"
            ),
            "{text}"
        );
        assert!(
            text.ends_with("  configuration {\n    checksum: \"null\"\n  }\n}\n"),
            "{text}"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cquery_streamed_proto_is_a_message_for_each_result_then_the_configurations() {
        let bytes = run_bytes(Kind::Cquery, "streamed_proto", "//:a + //:s.txt", None)
            .await
            .unwrap();
        // Each message is a length, then a `CqueryResult` with one field: a
        // result (tag 0x0a), and for the last the configurations (tag 0x12).
        let mut tags = Vec::new();
        let mut rest = &bytes[..];
        while !rest.is_empty() {
            let (mut len, mut shift, mut at) = (0usize, 0, 0);
            loop {
                let byte = rest[at];
                len |= usize::from(byte & 0x7f) << shift;
                shift += 7;
                at += 1;
                if byte < 0x80 {
                    break;
                }
            }
            tags.push(rest[at]);
            rest = &rest[at + len..];
        }
        assert_eq!(tags, [0x0a, 0x0a, 0x12]);
        // `proto` is those results and configurations in one message.
        let whole = run_bytes(Kind::Cquery, "proto", "//:a + //:s.txt", None)
            .await
            .unwrap();
        assert_eq!(whole[0], 0x0a);
    }

    /// What `bazel aquery //:a --output=jsonproto` printed for the same files,
    /// its two hashes replaced by `HASH`.
    const AQUERY_A_JSONPROTO: &str = r#"{
  "artifacts": [{
    "id": 1,
    "pathFragmentId": 1
  }, {
    "id": 2,
    "pathFragmentId": 2
  }],
  "actions": [{
    "targetId": 1,
    "actionKey": "HASH",
    "mnemonic": "Cat",
    "configurationId": 1,
    "arguments": ["/bin/bash", "-c", "cat $@ \u003e bazel-out/k8-fastbuild/bin/a.txt"],
    "environmentVariables": [{
      "key": "A",
      "value": "b"
    }],
    "inputDepSetIds": [1],
    "outputIds": [2],
    "primaryOutputId": 2,
    "executionPlatform": "@@platforms//host:host"
  }],
  "targets": [{
    "id": 1,
    "label": "//:a",
    "ruleClassId": 1
  }],
  "depSetOfFiles": [{
    "id": 1,
    "directArtifactIds": [1]
  }],
  "configuration": [{
    "id": 1,
    "mnemonic": "k8-fastbuild",
    "platformName": "k8",
    "checksum": "HASH"
  }],
  "ruleClasses": [{
    "id": 1,
    "name": "r"
  }],
  "pathFragments": [{
    "id": 1,
    "label": "s.txt"
  }, {
    "id": 5,
    "label": "bazel-out"
  }, {
    "id": 4,
    "label": "k8-fastbuild",
    "parentId": 5
  }, {
    "id": 3,
    "label": "bin",
    "parentId": 4
  }, {
    "id": 2,
    "label": "a.txt",
    "parentId": 3
  }]
}"#;

    /// The same with `--output=textproto`.
    const AQUERY_A_TEXTPROTO: &str = r#"rule_classes {
id: 1
name: "r"
}
targets {
id: 1
label: "//:a"
rule_class_id: 1
}
configuration {
id: 1
mnemonic: "k8-fastbuild"
platform_name: "k8"
checksum: "HASH"
}
path_fragments {
id: 1
label: "s.txt"
}
artifacts {
id: 1
path_fragment_id: 1
}
dep_set_of_files {
id: 1
direct_artifact_ids: 1
}
path_fragments {
id: 5
label: "bazel-out"
}
path_fragments {
id: 4
label: "k8-fastbuild"
parent_id: 5
}
path_fragments {
id: 3
label: "bin"
parent_id: 4
}
path_fragments {
id: 2
label: "a.txt"
parent_id: 3
}
artifacts {
id: 2
path_fragment_id: 2
}
actions {
target_id: 1
action_key: "HASH"
mnemonic: "Cat"
configuration_id: 1
arguments: "/bin/bash"
arguments: "-c"
arguments: "cat $@ > bazel-out/k8-fastbuild/bin/a.txt"
environment_variables {
  key: "A"
  value: "b"
}
input_dep_set_ids: 1
output_ids: 2
primary_output_id: 2
execution_platform: "@@platforms//host:host"
}
"#;

    #[tokio::test(flavor = "multi_thread")]
    async fn aquery_protos_are_what_bazel_prints() {
        let host = fjfj_graph::config::host_cpu();
        let like_ours = |expected: &str| {
            expected
                .replace("k8-fastbuild", &format!("{host}-fastbuild"))
                .replace("\"k8\"", &format!("\"{host}\""))
        };
        let json = run_query(Kind::Aquery, "jsonproto", "//:a", None)
            .await
            .unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&unhashed(&json)).unwrap(),
            serde_json::from_str::<serde_json::Value>(&like_ours(AQUERY_A_JSONPROTO)).unwrap()
        );
        // The text is Bazel's, down to its pieces sitting at the left edge.
        let text = run_query(Kind::Aquery, "textproto", "//:a", None)
            .await
            .unwrap();
        assert_eq!(unhashed(&text), like_ours(AQUERY_A_TEXTPROTO));
        // The wire format is those pieces one after the other, and the
        // streamed form puts a length before each.
        let whole = run_bytes(Kind::Aquery, "proto", "//:a", None)
            .await
            .unwrap();
        let streamed = run_bytes(Kind::Aquery, "streamed_proto", "//:a", None)
            .await
            .unwrap();
        assert_eq!(whole[0], 0x3a, "rule_classes (field 7) comes first");
        assert!(streamed.len() > whole.len());
        // `--noinclude_artifacts` leaves out what names files.
        let bare = tokio::task::spawn_blocking(|| {
            query_on(
                TRANSITION,
                Kind::Aquery,
                "jsonproto",
                "//:a",
                None,
                &[],
                aquery::Settings {
                    artifacts: false,
                    ..Default::default()
                },
            )
            .map(|bytes| String::from_utf8(bytes).unwrap())
            .unwrap()
        })
        .await
        .unwrap();
        let value: serde_json::Value = serde_json::from_str(&bare).unwrap();
        assert!(value.get("artifacts").is_none() && value.get("pathFragments").is_none());
        assert!(value["actions"][0].get("outputIds").is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cquery_build_shows_a_rule_once_with_its_stack() {
        let text = run_query(Kind::Cquery, "build", "//:b + //:a + //:s.txt", None)
            .await
            .unwrap();
        // //:b is in two configurations and shown once; a source file not at all.
        assert_eq!(text.matches("# Rule b instantiated").count(), 1);
        assert_eq!(text.matches("# Rule a instantiated").count(), 1);
        assert!(text.contains("  deps = [\"//:b\"],\n"), "{text}");
        assert!(
            text.contains("# Rule a instantiated at (most recent call last):\n#   "),
            "{text}"
        );
        assert!(
            text.contains("# Rule r defined at (most recent call last):\n#   "),
            "{text}"
        );
    }

    /// The descriptions of the actions an `aquery` printed.
    fn actions(text: &str) -> Vec<&str> {
        text.lines().filter(|l| l.starts_with("action '")).collect()
    }

    async fn aquery(expr: &'static str) -> String {
        run_query(Kind::Aquery, "text", expr, None).await.unwrap()
    }

    /// Each expectation is what `bazel aquery` printed for the same files.
    #[tokio::test(flavor = "multi_thread")]
    async fn aquery_filters_read_off_the_top_of_the_expression() {
        assert_eq!(actions(&aquery("mnemonic(Cat, //:a)").await).len(), 1);
        // The regex must match the whole mnemonic or path.
        assert!(actions(&aquery("mnemonic(Ca, //:a)").await).is_empty());
        assert_eq!(actions(&aquery("inputs(s.txt, //:a)").await).len(), 1);
        assert!(actions(&aquery("inputs(s, //:a)").await).is_empty());
        // The filter applies to every target of the expression it is given.
        let below = aquery("outputs(.*b.txt, deps(//:a))").await;
        assert_eq!(actions(&below), ["action 'Cat //:b'"]);
        // Filters nest, and each must be met.
        assert!(actions(&aquery("mnemonic(Nope, mnemonic(Cat, //:a))").await).is_empty());
        assert_eq!(
            actions(&aquery("mnemonic(Cat, mnemonic(Cat, //:a))").await).len(),
            1
        );
        // Under an operator a filter filters nothing: it passes its targets.
        let kept = aquery("mnemonic(Cat, //:a) ^ outputs(.*b.txt, deps(//:a))").await;
        assert_eq!(actions(&kept), ["action 'Cat //:a'"]);
        assert!(actions(&aquery("mnemonic(Cat, //:a) ^ //:b").await).is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn aquery_filters_are_checked_where_bazel_checks_them() {
        let error = |expr: &'static str| async move {
            run_query(Kind::Aquery, "text", expr, None)
                .await
                .unwrap_err()
                .to_string()
        };
        assert!(
            error("deps(mnemonic(Cat, //:a))")
                .await
                .contains("can't be the input of other function types: deps")
        );
        assert!(
            error("mnemonic(Cat)")
                .await
                .contains("must have exactly 2 arguments")
        );
    }

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn output_is_checked_against_the_commands_formats() {
        let (flags, rest) =
            extract(Kind::Cquery, &args(&["--output=label_kind", "//a:b"])).unwrap();
        assert_eq!(flags.format, "label_kind");
        assert_eq!(rest, ["//a:b"]);
        assert!(extract(Kind::Aquery, &args(&["--output=label"])).is_err());
        assert_eq!(
            extract(Kind::Aquery, &args(&["//a"])).unwrap().0.format,
            "text"
        );
        assert_eq!(
            extract(Kind::Cquery, &args(&["//a"])).unwrap().0.format,
            "label"
        );
    }

    #[test]
    fn the_starlark_flags_must_not_both_be_given() {
        let (flags, _) = extract(
            Kind::Cquery,
            &args(&["--starlark:expr=1", "--starlark:file=f.bzl", "//a"]),
        )
        .unwrap();
        assert!(
            flags
                .formatter()
                .unwrap_err()
                .to_string()
                .contains("must not specify both")
        );
    }
}
