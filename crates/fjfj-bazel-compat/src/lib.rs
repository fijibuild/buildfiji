//! Bazel command-line compatibility.
//!
//! Goal: `fjfj <cmd> <flags> <targets>` accepts the same command names,
//! startup/command options, `.bazelrc` files and target-pattern syntax as
//! Bazel, so that existing scripts and CI can swap the binary name.
//!
//! A command's flag surface is validated against the generated
//! `bazel_flags` table before anything runs: a flag that isn't real Bazel
//! usage for that command, or is but no typed extractor reads it yet,
//! fails the invocation loudly rather than being silently accepted with
//! no effect — see [`clap_flags::validate`].

pub mod bazel_flags;
pub mod bazelrc;
pub mod bes_flags;
pub mod build_flags;
pub mod bzlmod_flags;
pub mod canonicalize_flags;
pub mod clap_flags;
pub mod console;
pub mod console_flags;
pub mod diagnostics_flags;
pub mod execution_log_flags;
pub mod exit_code;
pub mod flag_alias;
pub mod flag_registry;
pub mod misc_flags;
pub mod option_syntax;
pub mod output_filter;
pub mod remote_flags;
pub mod run_flags;
pub mod test_flags;
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
    Clean(QueryArgs),
    /// Displays runtime info about the server.
    Info(QueryArgs),
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
    /// Summarises a trace file written with FJFJ_TRACE_FILE: where the build
    /// spent its time, its critical path, its slowest actions.
    AnalyzeProfile(AnalyzeProfileArgs),
}

#[derive(Parser, Debug, Clone, Default)]
pub struct AnalyzeProfileArgs {
    /// The Chrome trace file.
    pub path: std::path::PathBuf,
    /// How many of the slowest actions and chain links to list.
    #[arg(long, default_value_t = 15)]
    pub top: usize,
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
    #[arg(long = "output_base", global = true)]
    pub output_base: Option<std::path::PathBuf>,
    /// Bazel startup option: the directory that holds the output bases and
    /// the repository cache.
    #[arg(long = "output_user_root", global = true)]
    pub output_user_root: Option<std::path::PathBuf>,
    /// Bazel startup option: path to a bazelrc file. Repeatable.
    #[arg(long, global = true)]
    pub bazelrc: Vec<std::path::PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

#[cfg(test)]
mod startup_option_tests {
    use super::*;

    #[test]
    fn startup_options_take_bazels_underscore_spelling() {
        let cli = Cli::try_parse_from([
            "fjfj",
            "--output_base=/tmp/ob",
            "--output_user_root",
            "/tmp/our",
            "build",
            "//...",
        ])
        .unwrap();
        assert_eq!(cli.output_base, Some("/tmp/ob".into()));
        assert_eq!(cli.output_user_root, Some("/tmp/our".into()));
        assert!(Cli::try_parse_from(["fjfj", "--output-base=/tmp/ob", "build"]).is_err());
    }
}
