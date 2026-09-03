//! Bazel command-line compatibility.
//!
//! Goal: `fjfj <cmd> <flags> <targets>` accepts the same command names,
//! startup/command options, `.bazelrc` files and target-pattern syntax as
//! Bazel, so that existing scripts and CI can swap the binary name.
//!
//! Unknown Bazel flags are *accepted and recorded* (not rejected) so that a
//! `.bazelrc` shared with Bazel keeps working; see
//! [`flag_registry::UnknownFlagPolicy`].

pub mod bazel_flags;
pub mod bazelrc;
pub mod canonicalize_flags;
pub mod diagnostics_flags;
pub mod exit_code;
pub mod flag_alias;
pub mod flag_registry;
pub mod misc_flags;
pub mod output_filter;
pub mod workspace_status;
pub mod workspace_status_flags;

use clap::{Parser, Subcommand};

/// Bazel commands that fjfj intends to support. Order matches `bazel help`.
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Builds the specified targets.
    Build(TargetArgs),
    /// Builds and runs the specified tests.
    Test(TargetArgs),
    /// Runs a single target.
    Run(TargetArgs),
    /// Executes a dependency graph query.
    Query(QueryArgs),
    /// Executes a query on the post-analysis graph.
    Cquery(QueryArgs),
    /// Executes a query on the action graph.
    Aquery(QueryArgs),
    /// Fetches external repositories.
    Fetch(TargetArgs),
    /// Removes output tree.
    Clean,
    /// Displays runtime info about the server.
    Info,
    /// Prints version information.
    Version,
    /// Stops the persistent server.
    Shutdown,
    /// Bzlmod module management.
    Mod(QueryArgs),
    /// Prints the command line args for compiling a file.
    #[command(name = "print_action")]
    PrintAction(TargetArgs),
    /// Dumps the internal state of the fjfj server process.
    Dump,
    /// Canonicalizes a list of fjfj options.
    CanonicalizeFlags(CanonicalizeFlagsArgs),
    /// Prints the license of this software.
    License,
}

#[derive(Parser, Debug, Clone, Default)]
pub struct TargetArgs {
    /// Target patterns, e.g. `//...`, `//pkg:all`, `-//pkg:excluded`.
    #[arg(
        value_name = "TARGET",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub patterns: Vec<String>,
}

#[derive(Parser, Debug, Clone, Default)]
pub struct CanonicalizeFlagsArgs {
    /// The command for which the options should be canonicalized.
    #[arg(long, default_value = "build")]
    pub for_command: String,
    /// The flags to canonicalize, e.g. `-k --nostamp`.
    #[arg(
        value_name = "FLAG",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub flags: Vec<String>,
}

#[derive(Parser, Debug, Clone, Default)]
pub struct QueryArgs {
    #[arg(
        value_name = "EXPR",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub expr: Vec<String>,
}

/// Top-level CLI: `fjfj [startup opts] <command> [command opts] [targets]`.
#[derive(Parser, Debug)]
#[command(
    name = "fjfj",
    about = "A Bazel-compatible build tool",
    disable_version_flag = true
)]
pub struct Cli {
    /// Bazel startup option: root of the output base tree.
    #[arg(long, global = true)]
    pub output_base: Option<std::path::PathBuf>,
    /// Bazel startup option: path to a bazelrc file. Repeatable.
    #[arg(long, global = true)]
    pub bazelrc: Vec<std::path::PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

/// A parsed Bazel target pattern (`//foo/...`, `//foo:bar`, `@repo//x`, `-//y`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetPattern {
    pub negative: bool,
    pub repo: Option<String>,
    pub package: String,
    /// `None` => `:all` semantics; `Some("...")` => recursive.
    pub target: Option<String>,
}

impl std::str::FromStr for TargetPattern {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (negative, s) = match s.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, s),
        };
        let (repo, s) = match s.strip_prefix('@') {
            Some(rest) => {
                let (r, rest) = rest
                    .split_once("//")
                    .ok_or_else(|| anyhow::anyhow!("missing // in {s}"))?;
                (Some(r.to_string()), rest)
            }
            None => (
                None,
                s.strip_prefix("//")
                    .ok_or_else(|| anyhow::anyhow!("pattern must start with // or @: {s}"))?,
            ),
        };
        let (package, target) = match s.split_once(':') {
            Some((p, t)) => (p.to_string(), Some(t.to_string())),
            None if s.ends_with("...") => (
                s.trim_end_matches("...").trim_end_matches('/').to_string(),
                Some("...".into()),
            ),
            None => (s.to_string(), None),
        };
        fjfj_graph::label::validate_package_name(&package)
            .map_err(|e| anyhow::anyhow!("invalid package name {package:?}: {e}"))?;
        if let Some(target) = &target {
            fjfj_graph::label::validate_target_name(target)
                .map_err(|e| anyhow::anyhow!("invalid target name {target:?}: {e}"))?;
        }
        Ok(TargetPattern {
            negative,
            repo,
            package,
            target,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_recursive() {
        let p: TargetPattern = "//...".parse().unwrap();
        assert_eq!(
            p,
            TargetPattern {
                negative: false,
                repo: None,
                package: "".into(),
                target: Some("...".into())
            }
        );
    }
    #[test]
    fn parses_negative_repo() {
        let p: TargetPattern = "-@rules_rust//foo:bar".parse().unwrap();
        assert!(p.negative);
        assert_eq!(p.repo.as_deref(), Some("rules_rust"));
        assert_eq!(p.package, "foo");
        assert_eq!(p.target.as_deref(), Some("bar"));
    }
    #[test]
    fn accepts_a_unicode_target_name() {
        // buildfiji-mum.18: Bazel target names allow any non-ASCII
        // character (see fjfj_graph::label's doc comment); a source file
        // with a non-ASCII name is a legal target.
        let p: TargetPattern = "//testdata:café.txt".parse().unwrap();
        assert_eq!(p.target.as_deref(), Some("café.txt"));
    }
    #[test]
    fn rejects_a_non_ascii_package_name() {
        // Unlike target names, package names are ASCII-only in Bazel.
        assert!("//café:foo".parse::<TargetPattern>().is_err());
    }
    #[test]
    fn rejects_an_invalid_target_character() {
        assert!("//foo:bar:baz".parse::<TargetPattern>().is_err());
    }
}
