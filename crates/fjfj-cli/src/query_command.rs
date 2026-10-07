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
    "nodep_deps",
    "keep_going",
    "line_terminator_null",
    "host_deps",
    "universe_scope",
    "infer_universe_scope",
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
    pub double_slash: bool,
    /// `--universe_scope`, the patterns of the last flag.
    pub universe: Vec<String>,
    /// `--infer_universe_scope`: the universe is what the expression names.
    pub infer_universe: bool,
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
        double_slash: true,
        universe: Vec::new(),
        infer_universe: false,
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
            "universe_scope" => {
                let value = take(value).ok_or_else(|| bad("--universe_scope needs a value"))?;
                // The last flag wins.
                flags.universe = value
                    .split(',')
                    .map(str::trim)
                    .filter(|p| !p.is_empty())
                    .map(str::to_owned)
                    .collect();
            }
            "infer_universe_scope" => flags.infer_universe = true,
            "noinfer_universe_scope" => flags.infer_universe = false,
            "host_deps" | "nohost_deps" => {
                eprintln!("WARNING: Option 'host_deps' is deprecated: Use --tool_deps instead");
                flags.options.tool_deps = name == "host_deps";
            }
            "implicit_deps" => flags.options.implicit_deps = true,
            "noimplicit_deps" => flags.options.implicit_deps = false,
            "tool_deps" => flags.options.tool_deps = true,
            "notool_deps" => flags.options.tool_deps = false,
            "nodep_deps" => flags.options.nodep_deps = true,
            "nonodep_deps" => flags.options.nodep_deps = false,
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
            "incompatible_package_group_includes_double_slash" => flags.double_slash = true,
            "noincompatible_package_group_includes_double_slash" => flags.double_slash = false,
            "consistent_labels" => flags.consistent_labels = true,
            "noconsistent_labels" => flags.consistent_labels = false,
            "relative_locations" => flags.relative_locations = true,
            "norelative_locations" => flags.relative_locations = false,
            "line_terminator_null" => flags.terminator = '\0',
            "noline_terminator_null" => flags.terminator = '\n',
            "keep_going" => flags.options.keep_going = true,
            "nokeep_going" => flags.options.keep_going = false,
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

/// What Bazel says last of a query that stopped on a package with errors,
/// which it has reported already: that the query failed. A pattern that
/// selects whole trees says which one it was.
fn failed_query(query: &str, message: String, package_in: &dyn Fn(&str) -> String) -> String {
    let wildcard_of_package = message
        .strip_prefix("Error evaluating '")
        .and_then(|rest| rest.split_once("': "))
        .filter(|(pattern, rest)| !pattern.ends_with("...") && rest.ends_with("' contains errors"));
    // A dependency's REPO.bazel that fails: parsing a tree of it stops, and
    // any other pattern is a package that cannot load.
    if let Some(repo) = message.strip_prefix("error evaluating REPO.bazel file for @@") {
        return if query.contains("...") {
            format!("Target parsing failed due to unexpected exception: {message}")
        } else {
            format!(
                "error loading package '@@{repo}//{}': bad REPO.bazel file",
                package_in(repo)
            )
        };
    }
    match wildcard_of_package {
        Some(_) => format!("Evaluation of query \"{query}\" failed"),
        None => message,
    }
}

/// The package of the first pattern of `expr` that is in the repo called
/// `repo`, which is where Bazel says a REPO.bazel that fails was needed.
fn first_package_in(expr: &fjfj_query::Expr, repos: &fjfj_repo::Repos, repo: &str) -> String {
    use fjfj_graph::pattern::{Pattern, PatternContext, TargetPattern};
    let ctx = PatternContext {
        repo: "",
        offset: "",
    };
    expr.patterns()
        .into_iter()
        .filter_map(|text| {
            TargetPattern::parse(text, ctx, &mut |apparent| match apparent {
                "" => String::new(),
                _ => repos
                    .main_repo_canonical(apparent)
                    .unwrap_or_else(|| apparent.to_owned()),
            })
            .ok()
        })
        .find_map(|parsed| match parsed.pattern {
            Pattern::Target(label) if label.repo == repo => Some(label.package),
            Pattern::InPackage {
                repo: r, package, ..
            } if r == repo => Some(package),
            Pattern::Below {
                repo: r, directory, ..
            } if r == repo => Some(directory),
            Pattern::Path { repo: r, path } if r == repo => Some(path),
            _ => None,
        })
        .unwrap_or_default()
}

pub(crate) async fn run(args: QueryArgs) -> Result<(), CliError> {
    // What the rc files give `query`, which the command line overrides.
    let with_rc: Vec<String> = crate::rc_flags("query")?
        .into_iter()
        .chain(args.expr.iter().cloned())
        .collect();
    let (flags, rest) = extract(&with_rc)?;
    let rest = crate::drop_build_family_flags(rest, "query")?;
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
    fjfj_bazel_compat::clap_flags::validate(
        &crate::query_io::flags_first(&rest),
        "query",
        &implemented,
    )
    .map_err(|e| CliError::CommandLine(anyhow::Error::from(e)))?;
    let query = crate::query_io::expression(&io, &rest)?;
    // A universe is what turns `query` into a query over a graph, which has
    // `allrdeps` and `rbuildfiles` and limits what it sees.
    let universe = !flags.universe.is_empty() || flags.infer_universe;
    let dialect = match universe {
        true => fjfj_query::Dialect::Sky,
        false => fjfj_query::Dialect::Query,
    };
    let expr = fjfj_query::parse_in(&query, dialect)
        .map_err(|e| bad(format!("Error while parsing '{query}': {e}")))?;
    let scope = match flags.universe.is_empty() {
        true => {
            let mut seen = Vec::new();
            for p in expr.patterns() {
                if !seen.iter().any(|s: &String| s == p) {
                    seen.push(p.to_owned());
                }
            }
            seen
        }
        false => flags.universe.clone(),
    };
    let workspace_root = locate_workspace_root("query")?;
    let module_bazel_text =
        std::fs::read_to_string(workspace_root.join("MODULE.bazel")).map_err(|e| {
            bad(format!(
                "no MODULE.bazel found in {}: {e}",
                workspace_root.display()
            ))
        })?;
    let output =
        tokio::task::spawn_blocking(move || -> Result<(Vec<u8>, Vec<String>, bool), CliError> {
            let (resolved, repos) =
                fetch_command::begin(&fetch, &bzlmod, &workspace_root, &module_bazel_text)
                    .map_err(|e| fetch_command::asked(e, fetch_command::Asked::Query(&query)))?;
            let repos = Arc::new(repos);
            let graph = QueryGraph::new(repos.clone())
                .with_relative_locations(flags.relative_locations)
                .with_consistent_labels(flags.consistent_labels)
                .with_double_slash(flags.double_slash);
            let evaluator = Evaluator::new(&graph, flags.options);
            let evaluator = match universe {
                false => evaluator,
                true => match evaluator.with_universe(&scope) {
                    Ok(evaluator) => evaluator,
                    Err(message) => {
                        fetch_command::print_warnings(&repos);
                        return Err(CliError::Query(anyhow::anyhow!(message)));
                    }
                },
            };
            // The scope patterns that did not load are said before anything
            // else, and fail the query when it has otherwise run.
            for line in evaluator.scope_errors() {
                eprintln!("{line}");
            }
            let scope_failed = !evaluator.scope_errors().is_empty();
            let evaluated = evaluator.eval(&expr).and_then(|set| {
                if scope_failed {
                    return Err(format!(
                        "Evaluation of query \"{query}\" failed due to BUILD file errors"
                    ));
                }
                fjfj_query::output::render_bytes(
                    &evaluator,
                    &set,
                    flags.format,
                    flags.order,
                    flags.terminator,
                    &flags.proto,
                    &flags.graph,
                )
            });
            let text = match evaluated {
                Ok(text) => text,
                Err(message) => {
                    // The errors of the packages come first, as Bazel says
                    // them, and a package that has them is a failed query.
                    fetch_command::print_warnings(&repos);
                    let package_in = |repo: &str| first_package_in(&expr, &repos, repo);
                    return Err(CliError::Query(anyhow::anyhow!(failed_query(
                        &query,
                        message,
                        &package_in
                    ))));
                }
            };
            for line in evaluator.warnings() {
                eprintln!("{line}");
            }
            let skipped = evaluator.skipped();
            let incomplete = evaluator.incomplete();
            drop(evaluator);
            fetch_command::finish(resolved, &repos)?;
            Ok((text, skipped, incomplete))
        })
        .await
        .map_err(|e| CliError::Internal(anyhow::anyhow!("query task panicked: {e}")))??;
    let (output, skipped, incomplete) = output;
    if !incomplete {
        crate::query_io::write(&io, &output)?;
        return Ok(());
    }
    // Bazel says what it skipped, then gives the result it has and exits 3.
    for line in &skipped {
        eprintln!("{line}");
    }
    eprintln!("WARNING: --keep_going specified, ignoring errors. Results may be inaccurate");
    crate::query_io::write(&io, &output)?;
    Err(CliError::QueryIncomplete)
}

#[cfg(test)]
mod universe_flag_tests {
    use super::extract;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn the_last_universe_scope_wins_and_host_deps_is_tool_deps() {
        let (flags, rest) = extract(&args(&[
            "--universe_scope=//a,//b",
            "--universe_scope",
            "//c",
            "--nohost_deps",
            "--infer_universe_scope",
            "deps(//c)",
        ]))
        .unwrap();
        assert_eq!(flags.universe, ["//c"]);
        assert!(flags.infer_universe);
        assert!(!flags.options.tool_deps);
        assert_eq!(rest, ["deps(//c)"]);
        let (flags, _) = extract(&args(&["--host_deps"])).unwrap();
        assert!(flags.options.tool_deps);
    }
}

#[cfg(test)]
mod failed_query_tests {
    use super::failed_query;

    #[test]
    fn a_wildcard_of_a_package_with_errors_is_a_failed_query() {
        let message =
            "Error evaluating '//:all': error loading package '': Package '' contains errors";
        assert_eq!(
            failed_query("deps(//:all)", message.to_owned(), &|_| String::new()),
            "Evaluation of query \"deps(//:all)\" failed"
        );
        let tree = "Error evaluating '//...': error loading package '': Package '' contains errors";
        assert_eq!(
            failed_query("//...", tree.to_owned(), &|_| String::new()),
            tree
        );
        let repo_file = "error evaluating REPO.bazel file for @@dep+";
        assert_eq!(
            failed_query("@dep//pkg:t", repo_file.to_owned(), &|_| "pkg".to_owned()),
            "error loading package '@@dep+//pkg': bad REPO.bazel file"
        );
        assert_eq!(
            failed_query("@dep//...", repo_file.to_owned(), &|_| String::new()),
            format!("Target parsing failed due to unexpected exception: {repo_file}")
        );
        let other = "no such target '//:g'";
        assert_eq!(
            failed_query("//:g", other.to_owned(), &|_| String::new()),
            other
        );
    }
}
