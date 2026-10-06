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
use std::collections::{BTreeSet, HashMap};
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

/// `--transitions`: how much of the transitions on each edge `cquery` shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Transitions {
    #[default]
    None,
    Lite,
    Full,
}

/// `--show_config_fragments`: whether `cquery` follows a label with the
/// configuration fragments its target reads, and whose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum ShowFragments {
    #[default]
    Off,
    Direct,
    Transitive,
}

/// What `--output` and the flags around it asked for.
#[derive(Debug)]
struct Flags {
    fragments: ShowFragments,
    format: String,
    options: Options,
    expr: Option<String>,
    file: Option<String>,
    aquery: aquery::Settings,
    proto: fjfj_query::target_proto::ProtoOptions,
    graph: fjfj_query::output::GraphOptions,
    transitions: Transitions,
    terminator: char,
    relative_locations: bool,
    consistent_labels: bool,
    double_slash: bool,
    /// `--universe_scope`, each pattern of each flag.
    universe: Option<Vec<String>>,
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
            // No action of fjfj has a parameter file or discovers inputs.
            "include_param_files" | "include_pruned_inputs" => {}
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
        graph: Default::default(),
        transitions: Transitions::None,
        fragments: ShowFragments::Off,
        terminator: '\n',
        relative_locations: false,
        consistent_labels: false,
        double_slash: true,
        universe: None,
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
            "show_config_fragments" if kind == Kind::Cquery => {
                let value = value
                    .or_else(|| iter.next().cloned())
                    .ok_or_else(|| bad("--show_config_fragments needs a value"))?;
                flags.fragments = match value.as_str() {
                    "off" => ShowFragments::Off,
                    "direct" => ShowFragments::Direct,
                    "transitive" => ShowFragments::Transitive,
                    other => {
                        return Err(bad(format!(
                            "While parsing option --show_config_fragments={other}: Not a valid include config fragments provider option: '{other}' (should be off, direct or transitive)"
                        )));
                    }
                };
            }
            "transitions" if kind == Kind::Cquery => {
                let value = value
                    .or_else(|| iter.next().cloned())
                    .ok_or_else(|| bad("--transitions needs a value"))?;
                flags.transitions = match value.as_str() {
                    "none" => Transitions::None,
                    "lite" => Transitions::Lite,
                    "full" => Transitions::Full,
                    other => {
                        return Err(bad(format!(
                            "While parsing option --transitions={other}: Not a valid transition verbosity: '{other}' (should be full, lite or none)"
                        )));
                    }
                };
            }
            "universe_scope" => {
                let value = value
                    .or_else(|| iter.next().cloned())
                    .ok_or_else(|| bad("--universe_scope needs a value"))?;
                flags.universe.get_or_insert_with(Vec::new).extend(
                    value
                        .split(',')
                        .filter(|p| !p.is_empty())
                        .map(str::to_owned),
                );
            }
            "infer_universe_scope" | "noinfer_universe_scope" => {}
            "incompatible_package_group_includes_double_slash" => flags.double_slash = true,
            "noincompatible_package_group_includes_double_slash" => flags.double_slash = false,
            "consistent_labels" => flags.consistent_labels = true,
            "noconsistent_labels" => flags.consistent_labels = false,
            // The graphs have no edge that comes from an aspect.
            "include_aspects"
            | "noinclude_aspects"
            | "experimental_explicit_aspects"
            | "noexperimental_explicit_aspects"
                if kind == Kind::Cquery => {}
            "aspect_deps" => {
                let value = value
                    .or_else(|| iter.next().cloned())
                    .ok_or_else(|| bad("--aspect_deps needs a value"))?;
                if !["off", "conservative", "precise"].contains(&value.as_str()) {
                    return Err(bad(format!(
                        "While parsing option --aspect_deps={value}: Invalid value '{value}'; must be one of off, conservative, precise"
                    )));
                }
            }
            "relative_locations" => flags.relative_locations = true,
            "norelative_locations" => flags.relative_locations = false,
            "line_terminator_null" => flags.terminator = '\0',
            "noline_terminator_null" => flags.terminator = '\n',
            "implicit_deps" => flags.options.implicit_deps = true,
            "noimplicit_deps" => flags.options.implicit_deps = false,
            "tool_deps" => flags.options.tool_deps = true,
            "notool_deps" => flags.options.tool_deps = false,
            "nodep_deps" => flags.options.nodep_deps = true,
            "nonodep_deps" => flags.options.nodep_deps = false,
            "keep_going" | "nokeep_going" => {}
            n if n.starts_with("graph:") || n.starts_with("nograph:") => {
                let value = if n == "graph:node_limit" {
                    value.or_else(|| iter.next().cloned())
                } else {
                    value
                };
                if !flags.graph.flag(n, value.as_deref()).map_err(bad)? {
                    rest.push(arg.clone());
                }
            }
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
    // What the rc files give the command comes first, as for `build`, so the
    // command line overrides it.
    let with_rc: Vec<String> = crate::rc_flags(command)?
        .into_iter()
        .chain(args.expr.iter().cloned())
        .collect();
    let (flags, rest) = extract(kind, &with_rc)?;
    let (build_flags, rest) = fjfj_bazel_compat::build_flags::extract(&rest, command);
    let rest = crate::drop_build_family_flags(rest, command)?;
    let (bzlmod, rest) = bzlmod_flags::extract(&rest, command);
    let (fetch, rest) = fetch_command::extract(&rest)?;
    let (io, rest) = crate::query_io::extract(&rest)?;
    let mut implemented = crate::build_family_implemented();
    implemented.extend([
        "output",
        "implicit_deps",
        "tool_deps",
        "nodep_deps",
        "keep_going",
    ]);
    fjfj_bazel_compat::clap_flags::validate(
        &crate::query_io::flags_first(&rest),
        command,
        &implemented,
    )
    .map_err(|e| CliError::CommandLine(anyhow::Error::from(e)))?;
    let text = crate::query_io::expression(&io, &rest)?;
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
        graph: flags.graph,
        transitions: flags.transitions,
        fragments: flags.fragments,
        terminator: flags.terminator,
        relative_locations: flags.relative_locations,
        consistent_labels: flags.consistent_labels,
        double_slash: flags.double_slash,
        // `--infer_universe_scope` only fills in an unset scope, which is what
        // the targets of the expression are anyway.
        universe: flags.universe,
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
    crate::query_io::write(&io, &output)?;
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
        workspace_status: None,
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
    graph: fjfj_query::output::GraphOptions,
    transitions: Transitions,
    fragments: ShowFragments,
    /// `--line_terminator_null`: what ends a line of `cquery` output.
    terminator: char,
    relative_locations: bool,
    consistent_labels: bool,
    double_slash: bool,
    /// The patterns whose closure the expression is evaluated in, if the
    /// command line gave them.
    universe: Option<Vec<String>>,
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
    let graph = QueryGraph::new(repos.clone())
        .with_relative_locations(query.relative_locations)
        .with_consistent_labels(query.consistent_labels)
        .with_double_slash(query.double_slash);
    let mut named: BTreeSet<Label> = BTreeSet::new();
    // What is analysed is the closure of the universe, which is the targets
    // the expression names unless `--universe_scope` says otherwise.
    let patterns: Vec<&str> = match &query.universe {
        Some(scope) => scope.iter().map(String::as_str).collect(),
        None => query.expr.patterns(),
    };
    for pattern in patterns {
        named.extend(
            graph
                .pattern(pattern)
                .map_err(|e| CliError::Query(anyhow::anyhow!(e)))?,
        );
    }
    let top_level = options.configuration.clone();
    let platform_text = options.platform.clone();
    let host_platform_text = options.host_platform.clone();
    let request = build_command::Request {
        layout: layout.clone(),
        options,
    };
    // The platforms are dependencies of what resolves toolchains: analysed
    // too, so the graph can hold them.
    let mut platforms: Vec<Label> = Vec::new();
    for text in [platform_text.as_deref(), host_platform_text.as_deref()] {
        let text = text.unwrap_or("@bazel_tools//tools:host_platform");
        if let Ok(found) = graph.pattern(text) {
            for label in found {
                if !platforms.contains(&label) {
                    platforms.push(label);
                }
            }
        }
    }
    let mut targets: Vec<Label> = named.into_iter().collect();
    for platform in &platforms {
        if !targets.contains(platform) {
            targets.push(platform.clone());
        }
    }
    let report = build_command::run(repos, &targets, &request);
    if let Some((label, message)) = report.analysis_errors.first() {
        return Err(CliError::Query(anyhow::anyhow!(
            "{}: {message}",
            build_command::label_name(label)
        )));
    }
    let configured = ConfiguredGraph::new(&graph, &report.analysed, &top_level, platforms);
    let listing = Evaluator::new(&configured, query.options)
        .eval_ordered(&query.expr)
        .map_err(|e| CliError::Query(anyhow::anyhow!("Error doing post analysis query: {e}")))?;
    let evaluator = Evaluator::new(&configured, query.options).with_listing(listing);
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
            .flat_map(|target| {
                aquery::rows_of(
                    target,
                    configured.aspects_of(target),
                    query.aquery,
                    &configured,
                )
            })
            .filter(|row| filters.iter().all(|f| f.keeps(&row.action)))
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

/// The lines of `label` or `label_kind` output, each followed by the
/// configuration fragments its target reads.
fn with_fragments(
    query: &Query,
    labels: &BTreeSet<Label>,
    graph: &ConfiguredGraph<'_>,
    text: &str,
) -> String {
    let transitive = query.fragments == ShowFragments::Transitive;
    let by_name: HashMap<String, Vec<String>> = labels
        .iter()
        .map(|label| {
            (
                graph.output_name(label),
                graph.config_fragments(label, transitive),
            )
        })
        .collect();
    let terminator = query.terminator;
    text.split_terminator(terminator)
        .map(|line| {
            // `kind rule //a:b (hash)` or `//a:b (hash)`.
            let mut words = line.rsplitn(3, ' ');
            let (hash, label) = (words.next().unwrap_or(""), words.next().unwrap_or(""));
            let fragments = by_name
                .get(&format!("{label} {hash}"))
                .map(|f| f.join(", "))
                .unwrap_or_default();
            format!("{line} [{fragments}]{terminator}")
        })
        .collect()
}

/// `--transitions=lite|full`: each target with the transition that led to it,
/// then each of its dependencies by attribute, the transition on the edge and
/// the configuration it reached.
fn transitions_text(
    labels: &BTreeSet<Label>,
    graph: &ConfiguredGraph<'_>,
    mode: Transitions,
) -> String {
    let mut out = String::new();
    for label in labels {
        let Some(target) = graph.target(label) else {
            continue;
        };
        if !target.has_configuration() {
            out.push_str(&format!("{}\n", graph.output_name(label)));
            continue;
        }
        out.push_str(&format!("NoTransition -> {}\n", graph.output_name(label)));
        for edge in graph.transition_edges(label) {
            let reached = edge
                .configuration
                .as_ref()
                .map(|c| c.checksum()[..7].to_owned())
                .unwrap_or_default();
            out.push_str(&format!(
                "  {}#{}#{} -> {reached}\n",
                edge.attr,
                graph.loading_display(&edge.label),
                edge.description
            ));
            if mode == Transitions::Full {
                for (name, old, new) in &edge.changes {
                    out.push_str(&format!("    {name}:{old} -> [{new}]\n"));
                }
            }
        }
    }
    out
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
    let mut classes = BTreeSet::new();
    for label in labels {
        let Some(target) = graph.target(label) else {
            continue;
        };
        let message =
            fjfj_query::target_proto::target(evaluator, label, &query.proto, &mut classes)
                .map_err(failed)?;
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
    // Without the configurations the output is `query`'s.
    if !query.proto.include_configurations {
        let targets: Vec<Msg> = results
            .iter()
            .filter_map(|r| r.first_message(1))
            .cloned()
            .collect();
        return Ok(match query.format.as_str() {
            "proto" => Msg::new().many(1, "target", targets).binary(),
            "textproto" => Msg::new().many(1, "target", targets).text().into_bytes(),
            "jsonproto" => Msg::new().many(1, "target", targets).json().into_bytes(),
            _ => targets.iter().flat_map(|t| t.delimited()).collect(),
        });
    }
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
            "label" if query.transitions != Transitions::None => {
                Ok(transitions_text(labels, graph, query.transitions)
                    .replace('\n', &query.terminator.to_string()))
            }
            "label" | "label_kind" | "graph" | "build" => {
                let format = fjfj_query::output::Format::parse(&query.format)
                    .expect("a format the query command has");
                let text = fjfj_query::output::render_with(
                    evaluator,
                    labels,
                    format,
                    fjfj_query::output::Order::Auto,
                    query.terminator,
                    &query.graph,
                )
                .map_err(failed)?;
                if query.fragments == ShowFragments::Off
                    || !matches!(query.format.as_str(), "label" | "label_kind")
                {
                    return Ok(text);
                }
                Ok(with_fragments(query, labels, graph, &text))
            }
            "files" => {
                let mut out = String::new();
                for target in chosen(labels, graph) {
                    for file in target.files.to_vec() {
                        out.push_str(&file.exec_path());
                        out.push(query.terminator);
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
                Ok(lines
                    .into_iter()
                    .map(|l| l + &query.terminator.to_string())
                    .collect())
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
                .flat_map(|target| {
                    aquery::rows_of(target, graph.aspects_of(target), query.aquery, graph)
                })
                .filter(|row| filters.iter().all(|f| f.keeps(&row.action)))
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
        // `label` with `--transitions` is `transitions=lite` or `transitions=full`.
        let (format, transitions) = match format {
            "transitions=lite" => ("label", Transitions::Lite),
            "transitions=full" => ("label", Transitions::Full),
            other => (other, Transitions::None),
        };
        let (format, fragments) = match format {
            "fragments=direct" => ("label", ShowFragments::Direct),
            "fragments=transitive" => ("label", ShowFragments::Transitive),
            other => (other, ShowFragments::Off),
        };
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
            proto: PROTO.with(|p| p.borrow().clone()),
            graph: Default::default(),
            transitions,
            fragments,
            terminator: '\n',
            relative_locations: false,
            consistent_labels: false,
            double_slash: true,
            universe: UNIVERSE.with(|u| u.borrow().clone()),
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

    /// `cquery --show_config_fragments` on rules that read a fragment, a build
    /// setting and a `select()` on one, as bazel printed it (the digests
    /// here are not bazel's).
    #[tokio::test(flavor = "multi_thread")]
    async fn show_config_fragments_lists_what_each_target_reads() {
        const FILES: &[(&str, &str)] = &[
            ("MODULE.bazel", ""),
            (
                "r.bzl",
                "def _i(ctx): return []\nflag = rule(implementation=_i, build_setting=config.string(flag=True))\nusr = rule(implementation=_i, attrs={'f': attr.label(), 'deps': attr.label_list()})\nrcpp = rule(implementation=_i, fragments=['cpp'])\nasp = aspect(implementation=lambda target, ctx: [], fragments=['cpp'])\nwasp = rule(implementation=_i, attrs={'deps': attr.label_list(aspects=[asp])})\n",
            ),
            (
                "BUILD",
                "load(':r.bzl', 'flag', 'usr', 'rcpp', 'wasp')\nflag(name='myflag', build_setting_default='x')\nconfig_setting(name='on', flag_values={':myflag': 'y'})\nusr(name='reads', f=':myflag')\nusr(name='sels', deps=select({':on': [], '//conditions:default': []}))\nusr(name='outer', deps=[':reads', ':sels'])\nrcpp(name='cpp')\nconstraint_setting(name='s')\nconfig_setting(name='copt', values={'copt': '-O1', 'define': 'a=b'})\nwasp(name='w', deps=[':fgs'])\nfilegroup(name='fgs', srcs=['x.txt'])\n",
            ),
            ("x.txt", ""),
        ];
        let run = |format: &'static str| async move {
            let text = tokio::task::spawn_blocking(move || {
                query_on(
                    FILES,
                    Kind::Cquery,
                    format,
                    "//... + //:x.txt",
                    None,
                    &[],
                    aquery::Settings::default(),
                )
            })
            .await
            .unwrap()
            .unwrap();
            undigested_text(&String::from_utf8(text).unwrap())
        };
        const BASE: &str = "BazelRuleClassProvider$StrictActionEnvConfiguration, CoreOptions, PlatformConfiguration, PlatformOptions, ShellConfiguration";
        let flag = format!("//:myflag, {BASE}");
        let direct = run("fragments=direct").await;
        let expected = [
            (
                "//:cpp",
                "BazelRuleClassProvider$StrictActionEnvConfiguration, CoreOptions, CppConfiguration, PlatformConfiguration, PlatformOptions, ShellConfiguration",
            ),
            ("//:fgs", BASE),
            ("//:w", BASE),
            ("//:myflag", flag.as_str()),
            ("//:on", flag.as_str()),
            ("//:outer", BASE),
            ("//:reads", flag.as_str()),
            ("//:s", ""),
            ("//:sels", flag.as_str()),
            ("//:x.txt", ""),
        ];
        for (label, fragments) in expected {
            let line = direct
                .lines()
                .find(|l| l.starts_with(&format!("{label} ")))
                .unwrap_or_else(|| panic!("{label} in {direct}"));
            assert!(line.ends_with(&format!("[{fragments}]")), "{line}");
        }
        // Through its dependencies //:outer reads the flag too.
        let transitive = run("fragments=transitive").await;
        let outer = transitive
            .lines()
            .find(|l| l.starts_with("//:outer "))
            .unwrap();
        assert!(outer.ends_with(&format!("[{flag}]")), "{outer}");
        // The options of a config_setting, and what an aspect reads.
        let line = |text: &str, label: &str| {
            text.lines()
                .find(|l| l.starts_with(&format!("{label} ")))
                .unwrap()
                .to_owned()
        };
        assert!(
            line(&direct, "//:copt")
                .ends_with("[--define:a, BazelRuleClassProvider$StrictActionEnvConfiguration, CoreOptions, CppOptions, PlatformConfiguration, PlatformOptions, ShellConfiguration]")
        );
        assert!(line(&transitive, "//:w").contains("CppConfiguration"));
        assert!(run("label").await.lines().all(|l| !l.ends_with(']')));
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
    async fn a_union_is_listed_in_the_order_of_its_operands() {
        let union = cquery("//:s.txt + //:b + //:a").await;
        assert_eq!(
            undigested(&union),
            ["//:s.txt (null)", "//:b (7)", "//:b (7)", "//:a (7)"]
        );
        let sorted = cquery("(//:s.txt + //:b + //:a) - //:b").await;
        assert_eq!(undigested(&sorted), ["//:s.txt (null)", "//:a (7)"]);
        let functions = cquery("deps(//:a) - //:a").await;
        assert_eq!(undigested(&functions), ["//:b (7)", "//:s.txt (null)"]);
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
        // The summary counts the aspects too.
        let summary = tokio::task::spawn_blocking(|| {
            query_on(
                TRANSITION,
                Kind::Aquery,
                "summary",
                "deps(//:a) - //:s.txt",
                None,
                &["//:defs.bzl%asp"],
                aquery::Settings::default(),
            )
            .map(|bytes| String::from_utf8(bytes).unwrap())
            .unwrap()
        })
        .await
        .unwrap();
        assert!(
            summary.ends_with("\nAspects:\n  //:defs.bzl%asp: 2\n"),
            "{summary}"
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

    /// What `bazel cquery //:a --transitions=lite` printed for the same files.
    #[tokio::test(flavor = "multi_thread")]
    async fn transitions_show_each_edge_and_the_configuration_it_reaches() {
        let text = run_query(Kind::Cquery, "transitions=lite", "//:a + //:s.txt", None)
            .await
            .unwrap();
        let trimming = "(TestTrimmingTransition + ConfigFeatureFlagTaggedTrimmingTransition)";
        let lines = undigested(&text);
        assert_eq!(lines[0], "NoTransition -> //:a (7)");
        assert_eq!(lines[1], "  srcs#//:s.txt#(null transition) -> ");
        // The edge into //:b says where the transition was written and
        // which configuration it reached.
        let edge = &lines[2];
        assert!(
            edge.starts_with("  deps#//:b#(Starlark transition:")
                && edge.contains("/ws/defs.bzl:4:15 + ")
                && edge.contains(trimming),
            "{edge}"
        );
        assert_eq!(edge.rsplit_once(" -> ").unwrap().1.len(), 7);
        assert_eq!(
            text.lines()
                .nth(2)
                .unwrap()
                .rsplit_once(" -> ")
                .unwrap()
                .1
                .len(),
            7
        );
        assert_eq!(
            lines[3],
            "  $allowlist_function_transition#@bazel_tools//tools/allowlists/function_transition_allowlist:function_transition_allowlist#(null transition) -> "
        );
        // A source file has no configuration to come from.
        assert_eq!(lines[4], "//:s.txt (null)");
        // `full` adds the native options a transition changed: this one
        // changed a build setting only, which Bazel does not list.
        let full = run_query(Kind::Cquery, "transitions=full", "//:a", None)
            .await
            .unwrap();
        assert_eq!(full.lines().count(), 4);
    }

    /// What `bazel aquery //:k2` listed for an executable rule: the script
    /// it writes and the four actions of its runfiles.
    #[tokio::test(flavor = "multi_thread")]
    async fn aquery_lists_an_executables_runfiles_as_bazel_does() {
        let files: &[(&str, &str)] = &[
            ("MODULE.bazel", ""),
            (
                "defs.bzl",
                "def _impl(ctx):\n    exe = ctx.actions.declare_file(ctx.label.name + \".sh\")\n    ctx.actions.write(exe, \"#!/bin/sh\", is_executable = True)\n    return [DefaultInfo(files = depset([exe]), executable = exe, runfiles = ctx.runfiles(files = [exe]))]\nk2 = rule(implementation = _impl, executable = True)\n",
            ),
            ("BUILD", "load(\":defs.bzl\", \"k2\")\nk2(name = \"k2\")\n"),
        ];
        let text = tokio::task::spawn_blocking(|| {
            query_on(
                files,
                Kind::Aquery,
                "text",
                "//:k2",
                None,
                &[],
                aquery::Settings::default(),
            )
            .map(|bytes| String::from_utf8(bytes).unwrap())
            .unwrap()
        })
        .await
        .unwrap();
        let described: Vec<&str> = text
            .lines()
            .filter(|l| l.starts_with("action '") || l.starts_with("runfiles for"))
            .collect();
        let bin = format!("bazel-out/{}-fastbuild/bin", fjfj_graph::config::host_cpu());
        assert_eq!(
            described,
            [
                "action 'Writing script k2.sh'".to_owned(),
                "action 'Writing repo mapping manifest for //:k2'".to_owned(),
                "action 'Creating source manifest for //:k2'".to_owned(),
                format!("action 'Creating runfiles tree {bin}/k2.sh.runfiles'"),
                "runfiles for //:k2".to_owned(),
            ]
        );
        let mnemonics: Vec<&str> = text
            .lines()
            .filter_map(|l| l.strip_prefix("  Mnemonic: "))
            .collect();
        assert_eq!(
            mnemonics,
            [
                "FileWrite",
                "RepoMappingManifest",
                "SourceSymlinkManifest",
                "SymlinkTree",
                "RunfilesTree"
            ]
        );
        // The tree is made from the manifest, and stands for what it links.
        assert!(text.contains(&format!("  Inputs: [{bin}/k2.sh.runfiles_manifest]\n")));
        assert!(text.contains(&format!(
            "  Inputs: [{bin}/k2.sh, {bin}/k2.sh.repo_mapping, {bin}/k2.sh.runfiles/MANIFEST]\n  Outputs: [{bin}/k2.sh.runfiles]\n"
        )));
    }

    #[test]
    fn cquery_takes_nul_as_the_line_terminator() {
        let (flags, _) = extract(Kind::Cquery, &args(&["--line_terminator_null", "//a"])).unwrap();
        assert_eq!(flags.terminator, '\0');
        let (flags, _) = extract(
            Kind::Cquery,
            &args(&["--line_terminator_null", "--noline_terminator_null", "//a"]),
        )
        .unwrap();
        assert_eq!(flags.terminator, '\n');
    }

    #[test]
    fn universe_scope_collects_its_patterns_from_every_flag() {
        let (flags, rest) = extract(
            Kind::Cquery,
            &args(&[
                "--universe_scope=//a,//b",
                "--universe_scope",
                "//c",
                "--infer_universe_scope",
                "//d",
            ]),
        )
        .unwrap();
        assert_eq!(flags.universe.unwrap(), ["//a", "//b", "//c"]);
        assert_eq!(rest, ["//d"]);
        assert_eq!(
            extract(Kind::Cquery, &args(&["//d"])).unwrap().0.universe,
            None
        );
    }

    thread_local! {
        /// The `--universe_scope` of the next `query_on` on this thread.
        static UNIVERSE: std::cell::RefCell<Option<Vec<String>>> = const { std::cell::RefCell::new(None) };
    }

    /// What `bazel cquery --universe_scope=//:a //:b` printed: the //:b below
    /// //:a, and not the one built by itself.
    #[tokio::test(flavor = "multi_thread")]
    async fn the_universe_is_the_closure_of_its_scope() {
        let query = |scope: &'static str, expr: &'static str| async move {
            tokio::task::spawn_blocking(move || {
                UNIVERSE.with(|u| {
                    *u.borrow_mut() = Some(scope.split(',').map(str::to_owned).collect())
                });
                query_on(
                    TRANSITION,
                    Kind::Cquery,
                    "label",
                    expr,
                    None,
                    &[],
                    aquery::Settings::default(),
                )
                .map(|bytes| String::from_utf8(bytes).unwrap())
                .unwrap()
            })
            .await
            .unwrap()
        };
        let alone = cquery("//:b").await;
        let below = query("//:a", "//:b").await;
        assert_eq!(undigested(&below), ["//:b (7)"]);
        assert_ne!(
            below, alone,
            "the //:b below //:a is not the one built alone"
        );
        // Outside the universe there is nothing to find, and no error.
        assert_eq!(query("//:a", "//:flag").await, "");
        // Scopes add up, and the targets in both configurations are there.
        let both = query("//:a,//:b", "//:b").await;
        assert_eq!(both.lines().count(), 2);
    }

    thread_local! {
        /// The `--proto:` flags of the next `query_on` on this thread.
        static PROTO: std::cell::RefCell<fjfj_query::target_proto::ProtoOptions> =
            std::cell::RefCell::new(Default::default());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn cquery_proto_without_configurations_is_querys() {
        let json = |include: bool| async move {
            let text = tokio::task::spawn_blocking(move || {
                PROTO.with(|p| p.borrow_mut().include_configurations = include);
                let text = query_on(
                    TRANSITION,
                    Kind::Cquery,
                    "jsonproto",
                    "//:a + //:s.txt",
                    None,
                    &[],
                    aquery::Settings::default(),
                )
                .map(|bytes| String::from_utf8(bytes).unwrap())
                .unwrap();
                PROTO.with(|p| *p.borrow_mut() = Default::default());
                text
            })
            .await
            .unwrap();
            serde_json::from_str::<serde_json::Value>(&text).unwrap()
        };
        let with = json(true).await;
        assert!(with.get("results").is_some() && with.get("target").is_none());
        // `query`'s `QueryResult`: just the targets.
        let without = json(false).await;
        assert!(without.get("results").is_none() && without.get("configurations").is_none());
        let targets = without["target"].as_array().unwrap();
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0]["rule"]["name"], "//:a");
        // A Starlark rule carries the digest of its class, last.
        let attrs = targets[0]["rule"]["attribute"].as_array().unwrap();
        assert_eq!(attrs.last().unwrap()["name"], "$rule_implementation_hash");
    }

    /// What `bazel aquery` printed for a rule whose arguments go to a
    /// parameter file: the arguments inline, an `=` quoted, no file among the
    /// inputs and no action that writes it.
    #[tokio::test(flavor = "multi_thread")]
    async fn aquery_shows_the_arguments_of_a_parameter_file_inline() {
        let files: &[(&str, &str)] = &[
            ("MODULE.bazel", ""),
            (
                "defs.bzl",
                "def _impl(ctx):\n    out = ctx.actions.declare_file(ctx.label.name + \".out\")\n    args = ctx.actions.args()\n    args.add(\"--x=y\")\n    args.add(\"z\")\n    args.use_param_file(\"@%s\", use_always = True)\n    ctx.actions.run(outputs = [out], executable = \"/bin/echo\", arguments = [args], mnemonic = \"Echo\")\n    return [DefaultInfo(files = depset([out]))]\np = rule(implementation = _impl)\n",
            ),
            ("BUILD", "load(\":defs.bzl\", \"p\")\np(name = \"p\")\n"),
        ];
        let text = tokio::task::spawn_blocking(|| {
            query_on(
                files,
                Kind::Aquery,
                "text",
                "//:p",
                None,
                &[],
                aquery::Settings::default(),
            )
            .map(|bytes| String::from_utf8(bytes).unwrap())
            .unwrap()
        })
        .await
        .unwrap();
        assert_eq!(actions(&text), ["action 'Echo p.out'"], "{text}");
        assert!(text.contains("  Inputs: []\n"), "{text}");
        assert!(
            text.contains("  Command Line: (exec /bin/echo \\\n    '--x=y' \\\n    z)\n"),
            "{text}"
        );
    }

    /// What `bazel aquery --output=jsonproto` printed for an action whose
    /// `inputs` is a depset with a depset below it: the dep sets nest as the
    /// depsets do, the outer set has the lower id, and the artifacts of the
    /// inner set are numbered first. A flat list of the same files is a set of
    /// its own.
    #[tokio::test(flavor = "multi_thread")]
    async fn aquery_nests_the_dep_sets_of_a_depset_input() {
        let files = [
            ("MODULE.bazel", ""),
            ("a.txt", ""),
            ("b.txt", ""),
            (
                "defs.bzl",
                "def _i(ctx):\n    o = ctx.actions.declare_file(ctx.label.name + \".out\")\n    d = depset(ctx.files.srcs, transitive = [depset(ctx.files.deps)])\n    ctx.actions.run_shell(inputs = d, outputs = [o], command = \"true\")\n    p = ctx.actions.declare_file(ctx.label.name + \".l\")\n    ctx.actions.run_shell(inputs = ctx.files.srcs + ctx.files.deps, outputs = [p], command = \"true\")\n    return [DefaultInfo(files = depset([o, p]))]\nr = rule(_i, attrs = {\"srcs\": attr.label_list(allow_files = True), \"deps\": attr.label_list(allow_files = True)})\n",
            ),
            (
                "BUILD",
                "load(\":defs.bzl\", \"r\")\nfilegroup(name = \"fg\", srcs = [\"a.txt\", \"b.txt\"])\nr(name = \"s\", srcs = [\"a.txt\"], deps = [\":fg\"])\n",
            ),
        ];
        let json = tokio::task::spawn_blocking(move || {
            query_on(
                &files,
                Kind::Aquery,
                "jsonproto",
                "//:s",
                None,
                &[],
                aquery::Settings::default(),
            )
            .map(|bytes| String::from_utf8(bytes).unwrap())
            .unwrap()
        })
        .await
        .unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            value["depSetOfFiles"],
            serde_json::json!([
                {"id": 2, "directArtifactIds": [1, 2]},
                {"id": 1, "transitiveDepSetIds": [2], "directArtifactIds": [1]},
                // The same files in another set are another set.
                {"id": 3, "directArtifactIds": [1, 2]},
            ]),
            "{json}"
        );
        assert_eq!(
            value["actions"][0]["inputDepSetIds"],
            serde_json::json!([1])
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
