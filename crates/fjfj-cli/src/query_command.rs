//! `fjfj query` (buildfiji-9s8.1): the expression parsed, evaluated over the
//! loaded packages, and printed in the format asked for.

use crate::fetch_command;
use crate::query_graph::QueryGraph;
use crate::{CliError, bzlmod_flags, locate_workspace_root};
use fjfj_bazel_compat::QueryArgs;
use fjfj_query::output::{Format, Order};
use fjfj_query::{Evaluator, Options};
use std::sync::Arc;

/// The flags `query` reads itself.
pub(crate) const IMPLEMENTED: &[&str] = &[
    "output",
    "order_output",
    "implicit_deps",
    "tool_deps",
    "keep_going",
    "line_terminator_null",
];

#[derive(Debug, Clone)]
pub(crate) struct Flags {
    pub format: Format,
    pub order: Order,
    pub options: Options,
    pub terminator: char,
    pub proto: fjfj_query::target_proto::ProtoOptions,
    pub graph: fjfj_query::output::GraphOptions,
    pub relative_locations: bool,
    pub consistent_labels: bool,
}

/// `--output=bogus` is refused with Bazel's list of the valid ones.
const FORMATS: &str = "label, label_kind, build, minrank, maxrank, package, location, graph, xml, proto, streamed_jsonproto, streamed_proto";

fn bad(message: impl Into<String>) -> CliError {
    CliError::CommandLine(anyhow::anyhow!(message.into()))
}

/// Pull the query flags out of `args`, returning the rest.
pub(crate) fn extract(args: &[String]) -> Result<(Flags, Vec<String>), CliError> {
    let mut flags = Flags {
        format: Format::Label,
        order: Order::Auto,
        options: Options::default(),
        terminator: '\n',
        proto: Default::default(),
        graph: Default::default(),
        relative_locations: false,
        consistent_labels: false,
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
        let mut take = |value: Option<String>| value.or_else(|| iter.next().cloned());
        match name {
            "output" => {
                let value = take(value).ok_or_else(|| bad("--output needs a value"))?;
                flags.format = Format::parse(&value).ok_or_else(|| {
                    bad(format!(
                        "Invalid output format '{value}'. Valid values are: {FORMATS}"
                    ))
                })?;
            }
            "order_output" => {
                let value = take(value).ok_or_else(|| bad("--order_output needs a value"))?;
                flags.order = Order::parse(&value).ok_or_else(|| {
                    bad(format!(
                        "While parsing option --order_output={value}: Invalid value '{value}'; must be one of no, deps, auto, full"
                    ))
                })?;
            }
            "implicit_deps" => flags.options.implicit_deps = true,
            "noimplicit_deps" => flags.options.implicit_deps = false,
            "tool_deps" => flags.options.tool_deps = true,
            "notool_deps" => flags.options.tool_deps = false,
            // The graph has no edge that comes from an aspect.
            "aspect_deps" => {
                let value = take(value).ok_or_else(|| bad("--aspect_deps needs a value"))?;
                if !["off", "conservative", "precise"].contains(&value.as_str()) {
                    return Err(bad(format!(
                        "While parsing option --aspect_deps={value}: Invalid value '{value}'; must be one of off, conservative, precise"
                    )));
                }
            }
            "include_aspects"
            | "noinclude_aspects"
            | "experimental_explicit_aspects"
            | "noexperimental_explicit_aspects" => {}
            "consistent_labels" => flags.consistent_labels = true,
            "noconsistent_labels" => flags.consistent_labels = false,
            "relative_locations" => flags.relative_locations = true,
            "norelative_locations" => flags.relative_locations = false,
            "line_terminator_null" => flags.terminator = '\0',
            "noline_terminator_null" => flags.terminator = '\n',
            "keep_going" | "nokeep_going" => {}
            n if n.starts_with("graph:") || n.starts_with("nograph:") => {
                let value = if n == "graph:node_limit" {
                    take(value)
                } else {
                    value
                };
                if !flags.graph.flag(n, value.as_deref()).map_err(bad)? {
                    rest.push(arg.clone());
                }
            }
            n if n.starts_with("proto:") || n.starts_with("noproto:") => {
                let value = if n.ends_with("output_rule_attrs") {
                    take(value)
                } else {
                    value
                };
                if !flags.proto.flag(n, value.as_deref()).map_err(bad)? {
                    rest.push(arg.clone());
                }
            }
            _ => rest.push(arg.clone()),
        }
    }
    Ok((flags, rest))
}

pub(crate) async fn run(args: QueryArgs) -> Result<(), CliError> {
    let (flags, rest) = extract(&args.expr)?;
    let (bzlmod, rest) = bzlmod_flags::extract(&rest, "query");
    let (fetch, rest) = fetch_command::extract(&rest)?;
    let (io, rest) = crate::query_io::extract(&rest)?;
    let implemented: Vec<&'static str> = [
        bzlmod_flags::IMPLEMENTED,
        fetch_command::IMPLEMENTED,
        IMPLEMENTED,
    ]
    .iter()
    .flat_map(|s| s.iter().copied())
    .collect();
    fjfj_bazel_compat::clap_flags::validate(&rest, "query", &implemented)
        .map_err(|e| CliError::CommandLine(anyhow::Error::from(e)))?;
    let query = crate::query_io::expression(&io, &rest)?;
    let expr = fjfj_query::parse(&query)
        .map_err(|e| bad(format!("Error while parsing '{query}': {e}")))?;
    let workspace_root = locate_workspace_root("query")?;
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
        let graph = QueryGraph::new(repos.clone())
            .with_relative_locations(flags.relative_locations)
            .with_consistent_labels(flags.consistent_labels);
        let evaluator = Evaluator::new(&graph, flags.options);
        let text = evaluator
            .eval(&expr)
            .and_then(|set| {
                fjfj_query::output::render_bytes(
                    &evaluator,
                    &set,
                    flags.format,
                    flags.order,
                    flags.terminator,
                    &flags.proto,
                    &flags.graph,
                )
            })
            .map_err(|e| CliError::Query(anyhow::anyhow!(e)))?;
        drop(evaluator);
        fetch_command::finish(resolved, &repos)?;
        Ok(text)
    })
    .await
    .map_err(|e| CliError::Internal(anyhow::anyhow!("query task panicked: {e}")))??;
    crate::query_io::write(&io, &output)?;
    Ok(())
}
