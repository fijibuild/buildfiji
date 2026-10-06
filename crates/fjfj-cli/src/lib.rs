//! Command dispatch. Parses Bazel-compatible flags, sets up telemetry, and
//! runs the requested command.
//!
//! Exit codes and stderr line prefixes (`ERROR:`/`FATAL:`) follow Bazel's,
//! since CI scripts grep for them and check `$?` (see
//! `fjfj_bazel_compat::exit_code`); a bare `anyhow::Result` return from
//! `main` would always exit 1 with Rust's own `Error: {debug}` formatting,
//! which matches neither.

use std::io::IsTerminal;

use clap::Parser;
use fjfj_bazel_compat::bzlmod_flags::BzlmodFlags;
use fjfj_bazel_compat::exit_code::{ExitCode, messages};
use fjfj_bazel_compat::{
    Cli, Command, bes_flags, build_flags, bzlmod_flags, canonicalize_flags, clap_flags,
    console_flags, diagnostics_flags, execution_log_flags, flag_alias, misc_flags, output_filter,
    remote_flags, test_flags, workspace_status_flags,
};
use fjfj_bzlmod::attrs::AttrValue;
use fjfj_bzlmod::discovery::RegistrySource;
use fjfj_bzlmod::lockfile::{LockSession, LockfileMode};
use fjfj_bzlmod::overrides::{ModuleOverride, NonRegistryOverride, RepoRule, RepoSpec};
use fjfj_bzlmod::{
    BAZEL_CENTRAL_REGISTRY, Registry, Resolution, ResolveOptions, WorkspaceIncludeSource,
    YankedPolicy,
};
use fjfj_exec::console::ConsoleUi;
use fjfj_graph::pattern::{PatternContext, TargetPattern};
use fjfj_remote::execution_log::{CompactExecutionLogWriter, EntryType, ExecLogEntry, Invocation};

mod analysis_query;
mod analyze_profile;
mod aquery;
mod build_command;
mod configured_graph;
mod fetch_command;
mod info_command;
mod mod_command;
mod query_command;
mod query_graph;
mod query_io;
mod run_command;
mod test_command;
mod workspace;

/// `fjfj license`'s output. Bazel's own prints an equivalent short notice
/// (not the full license text — that's `LICENSE` in the repository root).
const LICENSE_NOTICE: &str = "\
Copyright 2026 The buildfiji Authors. All rights reserved.

Licensed under the Apache License, Version 2.0 (the \"License\");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an \"AS IS\" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
";

/// A command failure, tagged with the Bazel exit code it corresponds to.
/// `clap::Cli::parse()` handles its own flag-syntax errors (already exits
/// 2, matching [`ExitCode::CommandLineProblem`]); this covers everything
/// after that: pattern parsing, dispatch, and command execution.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// A target pattern (or other command-line value) that parsed as a
    /// flag but failed semantic validation, e.g. `//pkg:` with no target.
    #[error("{0}")]
    CommandLine(anyhow::Error),
    /// The requested command ran but didn't succeed.
    #[error("{0}")]
    Build(anyhow::Error),
    /// A query could not be evaluated: Bazel exits 7.
    #[error("{0}")]
    Query(anyhow::Error),
    /// A repository could not be made: Bazel's `fetch` exits 8 for it.
    #[error("{0}")]
    Fetch(anyhow::Error),
    /// A bug in fjfj itself, not something the user's command line or
    /// build can fix.
    #[error("{0}")]
    Internal(anyhow::Error),
    /// The build failed and has said why: only the exit code is left.
    #[error("the build failed")]
    Reported,
    /// A `--keep_going` query skipped some patterns and has said which:
    /// Bazel exits 3.
    #[error("query result incomplete")]
    QueryIncomplete,
    /// The build worked and some tests did not.
    #[error("tests failed")]
    TestsFailed,
    /// `test` was asked for and no target was a test.
    #[error("no tests")]
    NoTests,
    /// The program `run` ran exited with this code.
    #[error("the program exited with {0}")]
    Program(u8),
}

impl CliError {
    fn exit_code(&self) -> ExitCode {
        match self {
            CliError::CommandLine(_) => ExitCode::CommandLineProblem,
            CliError::Build(_) | CliError::Reported | CliError::Program(_) => ExitCode::BuildFailed,
            // Bazel gives a partial query result the code it gives failed tests.
            CliError::TestsFailed | CliError::QueryIncomplete => ExitCode::TestsFailed,
            CliError::NoTests => ExitCode::NoTestsFound,
            CliError::Fetch(_) => ExitCode::Interrupted,
            CliError::Query(_) => ExitCode::PartialAnalysisFailure,
            CliError::Internal(_) => ExitCode::InternalError,
        }
    }

    /// The line to write to stderr: `ERROR: ...` for anything the user
    /// can act on, `FATAL: ...` for [`CliError::Internal`].
    fn stderr_line(&self) -> String {
        match self {
            CliError::CommandLine(e)
            | CliError::Build(e)
            | CliError::Query(e)
            | CliError::Fetch(e) => messages::error(e),
            CliError::Internal(e) => messages::fatal(e),
            CliError::Reported
            | CliError::Program(_)
            | CliError::TestsFailed
            | CliError::QueryIncomplete
            | CliError::NoTests => String::new(),
        }
    }
}

/// The registries `--registry` names, or Bazel's own default list (just
/// `https://bcr.bazel.build`) when it wasn't given at all — repeatable
/// `--registry` *replaces* the default rather than adding to it.
fn bzlmod_registries(
    flags: &BzlmodFlags,
    lock: Option<&std::sync::Arc<LockSession>>,
) -> Result<Vec<Registry>, CliError> {
    let urls: Vec<&str> = if flags.registry.is_empty() {
        vec![BAZEL_CENTRAL_REGISTRY]
    } else {
        flags.registry.iter().map(String::as_str).collect()
    };
    urls.into_iter()
        .map(|url| {
            Registry::remote(url)
                .map(|registry| match lock {
                    Some(session) => registry.locked(session),
                    None => registry,
                })
                .map_err(|e| CliError::Internal(anyhow::anyhow!("--registry {url}: {e}")))
        })
        .collect()
}

/// [`ResolveOptions`] for `--allow_yanked_versions`, `--ignore_dev_dependency`
/// and `--override_module`, plus `include()` support rooted at
/// `workspace_root` (buildfiji-mum.22).
fn bzlmod_resolve_options(
    flags: &BzlmodFlags,
    workspace_root: &std::path::Path,
) -> Result<ResolveOptions, CliError> {
    let yanked = match &flags.allow_yanked_versions {
        Some(value) => YankedPolicy::parse(value)
            .map_err(|e| CliError::CommandLine(anyhow::anyhow!("--allow_yanked_versions: {e}")))?,
        None => YankedPolicy::default(),
    };
    let command_overrides = flags
        .override_module
        .iter()
        .map(|(name, path)| {
            (
                name.clone(),
                ModuleOverride::NonRegistry(NonRegistryOverride {
                    repo_spec: RepoSpec {
                        rule: RepoRule::LocalRepository,
                        attrs: vec![("path".to_owned(), AttrValue::String(path.clone()))],
                    },
                }),
            )
        })
        .collect();
    Ok(ResolveOptions {
        yanked,
        ignore_dev_dependency: flags.ignore_dev_dependency,
        command_overrides,
        include_source: Some(std::rc::Rc::new(WorkspaceIncludeSource::new(
            workspace_root,
        ))),
        for_lockfile: false,
        experimental_isolated_extension_usages: false,
    })
}

/// Resolves the bzlmod module graph for `module_bazel_text` (the root
/// `MODULE.bazel`, already read from `workspace_root`) against the flags a
/// command was given, leaving the lockfile to [`write_lockfile`].
fn resolve_bzlmod_session(
    module_bazel_text: &str,
    workspace_root: &std::path::Path,
    flags: &BzlmodFlags,
    isolated_extension_usages: bool,
    repository_cache: Option<std::path::PathBuf>,
) -> Result<fetch_command::Resolved, CliError> {
    let mode = match &flags.lockfile_mode {
        Some(value) => LockfileMode::parse(value)
            .map_err(|e| CliError::CommandLine(anyhow::anyhow!("--lockfile_mode: {e}")))?,
        None => LockfileMode::default(),
    };
    let lock_path = workspace_root.join("MODULE.bazel.lock");
    let existing = if mode == LockfileMode::Off {
        None
    } else {
        std::fs::read_to_string(&lock_path).ok()
    };
    let session = match mode {
        LockfileMode::Off => None,
        _ => Some(
            LockSession::new(mode, existing.as_deref())
                .map_err(|e| CliError::Build(anyhow::anyhow!(e)))?,
        ),
    };
    // Registry files whose hash the lockfile has come from here, not the network.
    if let (Some(session), Some(cache)) = (&session, repository_cache) {
        session.set_repository_cache(cache);
    }
    let registries = bzlmod_registries(flags, session.as_ref())?;
    let source =
        RegistrySource::new(registries).with_isolated_extension_usages(isolated_extension_usages);
    let mut options = bzlmod_resolve_options(flags, workspace_root)?;
    options.for_lockfile = session.is_some();
    options.experimental_isolated_extension_usages = isolated_extension_usages;
    let resolution = fjfj_bzlmod::resolve(module_bazel_text, &source, &options)
        .map_err(|e| CliError::Build(anyhow::anyhow!(e)))?;
    Ok(fetch_command::Resolved {
        resolution,
        session,
        existing,
        lock_path,
    })
}

/// Writes the lockfile a run ends with, only if it changed (Bazel leaves an
/// unchanged file's timestamp alone).
fn write_lockfile(resolved: &fetch_command::Resolved) -> Result<(), CliError> {
    if let Some(session) = &resolved.session
        && let Some(text) = session.to_write(
            &resolved.resolution.selected_yanked,
            resolved.existing.as_deref(),
        )
    {
        std::fs::write(&resolved.lock_path, text).map_err(|e| {
            CliError::Build(anyhow::anyhow!(
                "cannot write {}: {e}",
                resolved.lock_path.display()
            ))
        })?;
    }
    Ok(())
}

/// Resolves the bzlmod module graph and writes the lockfile.
fn resolve_bzlmod(
    module_bazel_text: &str,
    workspace_root: &std::path::Path,
    flags: &BzlmodFlags,
) -> Result<Resolution, CliError> {
    let resolved = resolve_bzlmod_session(
        module_bazel_text,
        workspace_root,
        flags,
        false,
        Some(fetch_command::default_repository_cache()),
    )?;
    write_lockfile(&resolved)?;
    Ok(resolved.resolution)
}

/// Reads `MODULE.bazel` from the current directory and resolves it —
/// shared by `Command::Build` and `Command::Mod`, both of which need the
/// module graph before doing anything else. No workspace-root search
/// exists yet, so this is the current directory's own `MODULE.bazel`,
/// matching how `bazel build`/`bazel mod` are invoked from the workspace
/// root in this repo today.
///
/// Runs `resolve_bzlmod` inside `spawn_blocking`: `Registry::remote`
/// builds a `reqwest::blocking::Client`, which owns and tears down its
/// own inner Tokio runtime, and dropping it from a thread already inside
/// `rt.block_on` — exactly where this would otherwise run — panics
/// (buildfiji-k62.16).
async fn resolve_workspace_bzlmod(flags: &BzlmodFlags) -> Result<Resolution, CliError> {
    let workspace_root = locate_workspace_root("mod")?;
    let module_bazel_text =
        std::fs::read_to_string(workspace_root.join("MODULE.bazel")).map_err(|e| {
            CliError::CommandLine(anyhow::anyhow!(
                "no MODULE.bazel found in {}: {e}",
                workspace_root.display()
            ))
        })?;
    let flags = flags.clone();
    tokio::task::spawn_blocking(move || resolve_bzlmod(&module_bazel_text, &workspace_root, &flags))
        .await
        .map_err(|e| CliError::Internal(anyhow::anyhow!("bzlmod resolution task panicked: {e}")))?
}

/// The workspace root above the current directory, or Bazel's refusal.
/// The flags the `.bazelrc` files give `command`: the system, workspace and
/// home ones, `import`s and `--config`s expanded, those of the commands it
/// inherits from included. Nothing if there is no workspace here.
pub(crate) fn rc_flags(command: &str) -> Result<Vec<String>, CliError> {
    let Ok(cwd) = std::env::current_dir() else {
        return Ok(Vec::new());
    };
    let Some((workspace_root, _)) = workspace::locate(&cwd) else {
        return Ok(Vec::new());
    };
    let options = fjfj_bazel_compat::bazelrc::DiscoveryOptions {
        workspace_root: Some(workspace_root),
        home: std::env::var_os("HOME").map(std::path::PathBuf::from),
        ..Default::default()
    };
    let lines = fjfj_bazel_compat::bazelrc::discover_and_parse(&options)
        .map_err(|e| CliError::CommandLine(anyhow::anyhow!("{e}")))?;
    fjfj_bazel_compat::bazelrc::resolve_command(&lines, command)
        .map_err(|e| CliError::CommandLine(anyhow::anyhow!("{e}")))
}

fn locate_workspace_root(command: &str) -> Result<std::path::PathBuf, CliError> {
    let cwd = std::env::current_dir()
        .map_err(|e| CliError::Internal(anyhow::anyhow!("couldn't get current directory: {e}")))?;
    workspace::locate(&cwd)
        .map(|(root, _)| root)
        .ok_or_else(|| {
            CliError::CommandLine(anyhow::anyhow!(workspace::not_in_a_workspace(command)))
        })
}

/// Resolves the module graph and makes the repositories `build`'s patterns
/// name; blocking work, run where `reqwest::blocking` can start (see
/// [`resolve_workspace_bzlmod`]).
async fn fetch_repositories_for_build(
    repo_flags: fetch_command::FetchFlags,
    bzlmod: &BzlmodFlags,
    patterns: Vec<String>,
    offset: String,
    build: build_command::Options,
) -> Result<fetch_command::BuildLoad, CliError> {
    let workspace_root = locate_workspace_root("build")?;
    let module_bazel_text =
        std::fs::read_to_string(workspace_root.join("MODULE.bazel")).map_err(|e| {
            CliError::CommandLine(anyhow::anyhow!(
                "no MODULE.bazel found in {}: {e}",
                workspace_root.display()
            ))
        })?;
    let bzlmod = bzlmod.clone();
    tokio::task::spawn_blocking(move || {
        fetch_command::run_for_build(
            &repo_flags,
            &bzlmod,
            &workspace_root,
            &module_bazel_text,
            &patterns,
            &offset,
            Some(&build),
        )
    })
    .await
    .map_err(|e| CliError::Internal(anyhow::anyhow!("build's repository task panicked: {e}")))?
}

pub fn main() -> std::process::ExitCode {
    // A command in namespaces starts as a fresh process of this program.
    fjfj_sandbox::run_helper_if_asked();
    fjfj_sandbox::use_helper();
    // exits 2 itself on a flag-syntax error
    let cli = Cli::parse_from(workspace::keep_end_of_options(
        std::env::args_os().collect(),
    ));
    // Analysis recurses once per level of the dependency graph, on the thread
    // that polls it: the default 2 MiB is too little for a Rust ruleset's graph.
    // The stack is address space until a thread touches it.
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(256 << 20)
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!(
                "{}",
                messages::fatal(format!("failed to start runtime: {e}"))
            );
            return ExitCode::InternalError.into();
        }
    };
    // fjfj_telemetry::init() is synchronous, but building its OTLP
    // exporters (opentelemetry_otlp's tonic/hyper-util plumbing) still
    // needs an entered Tokio runtime — panics with "there is no reactor
    // running" otherwise, only visible once OTEL_EXPORTER_OTLP_ENDPOINT
    // is actually set (buildfiji-k62.14). So `rt` must exist first, and
    // the guard stays alive through `block_on` below rather than being
    // dropped right after `init` — the periodic metrics exporter it sets
    // up spawns a background task that outlives this call.
    let _guard = rt.enter();
    let Ok(_telemetry) = fjfj_telemetry::init() else {
        eprintln!("{}", messages::fatal("failed to initialize telemetry"));
        return ExitCode::InternalError.into();
    };
    match rt.block_on(run(cli)) {
        Ok(()) => ExitCode::Success.into(),
        Err(e) => {
            let line = e.stderr_line();
            if !line.is_empty() {
                eprintln!("{line}");
            }
            if let CliError::Program(code) = e {
                return std::process::ExitCode::from(code);
            }
            e.exit_code().into()
        }
    }
}

#[tracing::instrument(skip_all)]
async fn run(cli: Cli) -> Result<(), CliError> {
    fetch_command::set_startup(cli.output_base.clone(), cli.output_user_root.clone());
    match cli.command {
        Command::Version => {
            println!("fjfj {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Command::License => {
            print!("{LICENSE_NOTICE}");
            Ok(())
        }
        Command::CanonicalizeFlags(args) => {
            let canonical = canonicalize_flags::canonicalize(&args.flags, &args.for_command)
                .map_err(|e| CliError::CommandLine(anyhow::Error::from(e)))?;
            println!("{}", canonical.join(" "));
            Ok(())
        }
        Command::Build(args) => build_main(args, "build", false).await.map(|_| ()),
        Command::Query(args) => query_command::run(args).await,
        Command::Info(args) => info_command::run(args).await,
        Command::Cquery(args) => analysis_query::run(args, analysis_query::Kind::Cquery).await,
        Command::Aquery(args) => analysis_query::run(args, analysis_query::Kind::Aquery).await,
        Command::Run(args) => run_command::run(args).await,
        Command::Test(args) => test_command::run(args).await,
        Command::Mod(args) => {
            let (subcommand, rest) = args.expr.split_first().ok_or_else(|| {
                CliError::CommandLine(anyhow::anyhow!(
                    "usage: fjfj mod <graph|deps|show_repo|explain> [args...]"
                ))
            })?;
            let (bzlmod, rest) = bzlmod_flags::extract(rest, "mod");
            // `--output=text|json` only means anything to `graph`; kept as
            // a plain scan rather than its own `*_flags` module, since no
            // other subcommand reads it and Bazel doesn't offer it to them
            // either.
            let mut output_json = false;
            let mut names: Vec<String> = Vec::new();
            for arg in &rest {
                match arg.as_str() {
                    "--output=json" => output_json = true,
                    "--output=text" => output_json = false,
                    other => names.push(other.to_owned()),
                }
            }
            let resolution = resolve_workspace_bzlmod(&bzlmod).await?;
            let output = match subcommand.as_str() {
                "graph" if output_json => {
                    let mut json =
                        serde_json::to_string_pretty(&mod_command::render_graph_json(&resolution))
                            .map_err(|e| CliError::Internal(anyhow::anyhow!(e)))?;
                    json.push('\n');
                    json
                }
                "graph" => mod_command::render_graph_text(&resolution),
                "deps" => {
                    mod_command::render_deps(&resolution, &names).map_err(CliError::CommandLine)?
                }
                "show_repo" => mod_command::render_show_repo(&resolution, &names)
                    .map_err(CliError::CommandLine)?,
                "explain" => mod_command::render_explain(&resolution, &names)
                    .map_err(CliError::CommandLine)?,
                other => {
                    return Err(CliError::CommandLine(anyhow::anyhow!(
                        "unknown mod subcommand '{other}'; expected graph, deps, show_repo, or explain"
                    )));
                }
            };
            print!("{output}");
            Ok(())
        }
        Command::Fetch(args) => {
            let (fetch, rest) = fetch_command::extract(&args.patterns)?;
            let (bzlmod, rest) = bzlmod_flags::extract(&rest, "fetch");
            let implemented: Vec<&'static str> =
                [bzlmod_flags::IMPLEMENTED, fetch_command::IMPLEMENTED]
                    .iter()
                    .flat_map(|s| s.iter().copied())
                    .collect();
            clap_flags::validate(&rest, "fetch", &implemented)
                .map_err(|e| CliError::CommandLine(anyhow::Error::from(e)))?;
            let workspace_root = locate_workspace_root("fetch")?;
            let module_bazel_text = std::fs::read_to_string(workspace_root.join("MODULE.bazel"))
                .map_err(|e| {
                    CliError::CommandLine(anyhow::anyhow!(
                        "no MODULE.bazel found in {}: {e}",
                        workspace_root.display()
                    ))
                })?;
            // `reqwest::blocking` does not start inside the runtime (see
            // `resolve_workspace_bzlmod`).
            tokio::task::spawn_blocking(move || {
                fetch_command::run(&fetch, &bzlmod, &workspace_root, &module_bazel_text)
            })
            .await
            .map_err(|e| CliError::Internal(anyhow::anyhow!("fetch task panicked: {e}")))?
        }
        Command::AnalyzeProfile(args) => analyze_profile::run(&args)
            .map_err(|e| CliError::CommandLine(anyhow::anyhow!("{}: {e}", args.path.display()))),
        other => Err(CliError::Build(anyhow::anyhow!(
            "command not implemented yet: {other:?}"
        ))),
    }
}

/// The flags the modules `build` reads give: what an rc file may hand any
/// command that inherits from `build`.
const BUILD_IMPLEMENTED: &[&[&str]] = &[
    flag_alias::IMPLEMENTED,
    build_flags::IMPLEMENTED,
    test_flags::IMPLEMENTED,
    diagnostics_flags::IMPLEMENTED,
    workspace_status_flags::IMPLEMENTED,
    misc_flags::IMPLEMENTED,
    output_filter::IMPLEMENTED,
    execution_log_flags::IMPLEMENTED,
    remote_flags::IMPLEMENTED,
    bes_flags::IMPLEMENTED,
    bzlmod_flags::IMPLEMENTED,
    console_flags::IMPLEMENTED,
    fetch_command::BUILD_IMPLEMENTED,
];

/// The names of [`BUILD_IMPLEMENTED`].
pub(crate) fn build_family_implemented() -> Vec<&'static str> {
    BUILD_IMPLEMENTED
        .iter()
        .flat_map(|s| s.iter().copied())
        .collect()
}

/// `rest` without the flags the build modules read, for a command that
/// analyses and builds nothing and has no use for them: an rc file gives
/// them to `cquery` and `aquery` as it does to `build`.
pub(crate) fn drop_build_family_flags(
    rest: Vec<String>,
    command: &'static str,
) -> Result<Vec<String>, CliError> {
    let (aliases, rest) =
        flag_alias::extract(&rest).map_err(|e| CliError::CommandLine(anyhow::Error::from(e)))?;
    let rest = flag_alias::apply(&aliases, &rest);
    let (_, rest) = diagnostics_flags::extract(&rest, command);
    let (_, rest) = workspace_status_flags::extract(&rest, command);
    let (_, rest) = misc_flags::extract(&rest, command);
    let (_, rest) = output_filter::extract(&rest, command);
    let (_, rest) = execution_log_flags::extract(&rest, command);
    let (_, rest) = remote_flags::extract(&rest, command);
    let (_, rest) = bes_flags::extract(&rest, command);
    let (_, rest) = console_flags::extract(&rest, command);
    Ok(rest)
}

/// What a `build`, `run` or `test` has built.
pub(crate) struct Built {
    pub(crate) report: build_command::Report,
    pub(crate) layout: fjfj_exec::execroot::Layout,
    pub(crate) options: build_command::Options,
    /// `run`: what followed the target, for the program.
    pub(crate) program_args: Vec<String>,
}

/// `fjfj build`, and the building half of `run` and `test`. `args` are flags
/// and target patterns; `Ok(None)` if the targets did not build.
async fn build_main(
    args: fjfj_bazel_compat::TargetArgs,
    command: &'static str,
    run_mode: bool,
) -> Result<Built, CliError> {
    // The flags the rc files give the command come first, so the command
    // line overrides them.
    let rc = rc_flags(command)?;
    let with_rc: Vec<String> = rc
        .into_iter()
        .chain(args.patterns.iter().cloned())
        .collect();
    // Only what follows `--` is a pattern that may start with `-`.
    let (before, after_marker) = workspace::split_end_of_options(&with_rc);
    if let Some(negative) = before
        .iter()
        .find(|a| a.starts_with("-@") || a.starts_with("-//"))
    {
        return Err(CliError::CommandLine(anyhow::anyhow!(
            "Invalid options syntax: {negative}\nNote: Negative target patterns can only \
             appear after the end of options marker ('--'). Flags corresponding to \
             Starlark-defined build settings always start with '--', not '-'."
        )));
    }
    let (aliases, rest) =
        flag_alias::extract(before).map_err(|e| CliError::CommandLine(anyhow::Error::from(e)))?;
    let rest = flag_alias::apply(&aliases, &rest);
    // buildfiji-gwl.15/gwl.16: validate every flag token against
    // the full generated `bazel_flags` table *before* any typed
    // extraction runs, and fail loudly — rather than warning and
    // continuing — on a flag that isn't a real Bazel flag for
    // `build`, or is one but no module below actually reads it.
    // Silently accepting the latter would let a build proceed
    // with the flag's value doing nothing, which is worse than
    // refusing to run: see `docs/design/cli-compat.md`'s "Flag
    // surface" decision. This also keeps a leftover token from
    // ever reaching `TargetPattern::from_str`, whose "pattern
    // must start with // or @" error is misleading for a flag
    // typo.
    let implemented = build_family_implemented();
    clap_flags::validate(&rest, command, &implemented)
        .map_err(|e| CliError::CommandLine(anyhow::Error::from(e)))?;
    let (build_flags, rest) = build_flags::extract(&rest, command);
    // Only `test` has test flags; for it, `--test_env` and `--test_arg` shape
    // the configuration.
    let (test_flags, rest) = if command == "test" {
        let (flags, rest) = test_flags::extract(&rest, command)
            .map_err(|e| CliError::CommandLine(anyhow::anyhow!(e)))?;
        (Some(flags), rest)
    } else {
        (None, rest)
    };
    let (diagnostics, rest) = diagnostics_flags::extract(&rest, command);
    let (workspace_status, rest) = workspace_status_flags::extract(&rest, command);
    let (misc, rest) = misc_flags::extract(&rest, command);
    let (output_filter_flags, rest) = output_filter::extract(&rest, command);
    let (execution_log, rest) = execution_log_flags::extract(&rest, command);
    let (remote, rest) = remote_flags::extract(&rest, command);
    let (bes, rest) = bes_flags::extract(&rest, command);
    let (bzlmod, rest) = bzlmod_flags::extract(&rest, command);
    let (repo_flags, rest) = fetch_command::extract(&rest)?;
    let (console_flags, rest) = console_flags::extract(&rest, command);
    // Everything left is a bare positional now that `validate`
    // above has ruled out any unimplemented or unrecognized flag.
    let cwd = std::env::current_dir()
        .map_err(|e| CliError::Internal(anyhow::anyhow!("couldn't get current directory: {e}")))?;
    let (_, offset) = workspace::locate(&cwd).ok_or_else(|| {
        CliError::CommandLine(anyhow::anyhow!(workspace::not_in_a_workspace(command)))
    })?;
    let context = PatternContext {
        repo: "",
        offset: &offset,
    };
    // The main module's repo mapping is not known until the module
    // graph is resolved, so an apparent name stays as written; what
    // the pattern wrote is what `fetch --repo` takes.
    let patterns = rest
        .iter()
        .chain(after_marker)
        .map(|p| TargetPattern::parse(p, context, &mut |apparent| apparent.to_owned()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::CommandLine(anyhow::Error::from(e)))?;
    let command_line_packages = patterns.iter().map(|p| p.pattern.package().to_owned());
    let _output_filter =
        output_filter::OutputFilter::compile(&output_filter_flags, command_line_packages)
            .map_err(|e| CliError::CommandLine(anyhow::Error::from(e)))?;
    tracing::debug!(
        ?patterns,
        ?diagnostics,
        ?workspace_status,
        ?misc,
        ?aliases,
        ?output_filter_flags,
        ?execution_log,
        ?remote,
        ?bes,
        ?bzlmod,
        ?console_flags,
        "build requested"
    );
    // Bazel prints its progress on standard error and fjfj has none of its own
    // yet, so the console is made only to check `--ui_event_filters`.
    ConsoleUi::new(
        std::io::stdout(),
        &console_flags,
        std::io::stdout().is_terminal(),
    )
    .map_err(|e| CliError::CommandLine(anyhow::anyhow!("--ui_event_filters: {e}")))?;
    // Just a writability check: there is no REAPI client yet to
    // make a gRPC call worth logging, so unlike the execution log
    // above there is no header entry to write. Still fails fast on
    // a bad path rather than waiting for remote execution to exist.
    if let Some(path) = &remote.remote_grpc_log {
        std::fs::File::create(path).map_err(|e| {
            CliError::CommandLine(anyhow::anyhow!(
                "couldn't open --remote_grpc_log {}: {e}",
                path.display()
            ))
        })?;
    }
    // Opened and given its Invocation header now, for the same
    // fail-fast reason as --workspace_status_command above: an
    // unwritable --execution_log_compact_file path should reject
    // the build immediately, not silently produce nothing once
    // there are real spawns to log. The invocation id is left
    // empty until there is a daemon-assigned one to put here (see
    // `invocation_id` in fjfj-proto's command.proto).
    if let Some(path) = &execution_log.execution_log_compact_file {
        let file = std::fs::File::create(path).map_err(|e| {
            CliError::CommandLine(anyhow::anyhow!(
                "couldn't open --execution_log_compact_file {}: {e}",
                path.display()
            ))
        })?;
        let mut writer = CompactExecutionLogWriter::new(file)
            .map_err(|e| CliError::Internal(anyhow::Error::from(e)))?;
        writer
            .write_entry(&ExecLogEntry {
                id: 0,
                r#type: Some(EntryType::Invocation(Invocation {
                    hash_function_name: "SHA-256".into(),
                    workspace_runfiles_directory: "_main".into(),
                    sibling_repository_layout: true,
                    id: String::new(),
                })),
            })
            .map_err(|e| CliError::Internal(anyhow::Error::from(e)))?;
        writer
            .finish()
            .map_err(|e| CliError::Internal(anyhow::Error::from(e)))?;
    }
    // Computed and logged now so `--workspace_status_command` and
    // `--stamp` fail fast the way Bazel does, even before there's
    // a real build to stamp; the snapshot isn't written to disk
    // yet since there's no execroot/bazel-out layout for
    // stable-status.txt/volatile-status.txt to land in (see
    // fjfj_exec::workspace_status).
    let status = fjfj_exec::workspace_status::compute(&workspace_status)
        .await
        .map_err(|e| CliError::Build(anyhow::anyhow!(e)))?;
    tracing::info!(stable = ?status.stable, "workspace status computed");
    // buildfiji-gwl.17: resolve the bzlmod module graph now, same
    // fail-fast reasoning as the workspace status and execution log
    // above.
    // `run` takes the first target and gives the rest to the program.
    let (texts, program_args): (Vec<String>, Vec<String>) = if run_mode {
        let mut all = rest.iter().chain(after_marker).cloned();
        (all.next().into_iter().collect(), all.collect())
    } else {
        (
            rest.iter().chain(after_marker).cloned().collect(),
            Vec::new(),
        )
    };
    let mut configuration = build_command::configuration_from(&build_flags)
        .map_err(|e| CliError::CommandLine(anyhow::anyhow!(e)))?;
    if let Some(flags) = &test_flags {
        build_command::apply_test_flags(&mut configuration, flags);
    }
    let build = build_command::Options {
        configuration,
        platform: build_flags.platforms.clone(),
        extra_toolchains: build_flags.extra_toolchains.clone(),
        extra_execution_platforms: build_flags.extra_execution_platforms.clone(),
        host_platform: build_flags.host_platform.clone(),
        aspects: build_flags.aspects.clone(),
        output_groups: build_flags.output_groups.clone(),
        keep_going: diagnostics.keep_going,
        build: build_flags.build.unwrap_or(true),
        symlink_prefix: build_flags
            .symlink_prefix
            .clone()
            .unwrap_or_else(|| "bazel-".to_owned()),
        jobs: build_command::jobs_from(build_flags.jobs.as_deref())
            .map_err(|e| CliError::CommandLine(anyhow::anyhow!(e)))?,
        strategy: match build_flags.spawn_strategy.as_deref() {
            None => fjfj_exec::run::Options::default().strategy,
            Some(list) => fjfj_exec::run::Strategy::parse(list)
                .map_err(|e| CliError::CommandLine(anyhow::anyhow!(e)))?,
        },
        show_result: match build_flags.show_result.as_deref() {
            None => 1,
            Some(n) => n.parse().map_err(|_| {
                CliError::CommandLine(anyhow::anyhow!(
                    "--show_result: '{n}' is not a non-negative integer"
                ))
            })?,
        },
        record_execution_platforms: false,
        test: test_flags.as_ref().map(|t| t.output),
        workspace_status: Some((status.render_stable(), status.render_volatile())),
        incompatible: Some(build_command::IncompatibleRoots {
            explicit: Default::default(),
            skip_explicit: build_flags.skip_incompatible_explicit_targets,
        }),
    };
    let build_show_result = build.show_result;
    let build_options = build.clone();
    let build_options_test_output = build.test;
    let loaded = fetch_repositories_for_build(repo_flags, &bzlmod, texts, offset, build).await?;
    let resolution = loaded.resolution;
    let targets = loaded.targets;
    tracing::info!(
        selected_modules = resolution.selection.keys().count(),
        "bzlmod module graph resolved"
    );
    // A repo whose REPO.bazel fails stops the parsing of every pattern.
    if let Some(failure) = targets.failures.iter().find(|f| {
        f.message
            .starts_with("error evaluating REPO.bazel file for ")
    }) {
        return Err(CliError::Build(anyhow::anyhow!(
            "{}\nERROR: Build did NOT complete successfully",
            failure.message
        )));
    }
    if !targets.failures.is_empty() {
        eprintln!("WARNING: Target pattern parsing failed.");
        for failure in &targets.failures {
            eprintln!("ERROR: Skipping '{}': {}", failure.pattern, failure.message);
        }
        if !diagnostics.keep_going {
            return Err(CliError::Build(anyhow::anyhow!(
                "{}",
                targets.failures[0].message
            )));
        }
    }
    let Some(report) = loaded.report else {
        return Err(CliError::Build(anyhow::anyhow!(
            "command succeeded, but there were errors parsing the target pattern"
        )));
    };
    let layout = report.layout.clone();
    let succeeded = build_command::print(
        &report,
        targets.targets.len(),
        build_show_result,
        diagnostics.keep_going,
        &layout,
        diagnostics.verbose_failures,
        build_options_test_output,
    );
    if !succeeded {
        return Err(CliError::Reported);
    }
    if !targets.failures.is_empty() {
        return Err(CliError::Build(anyhow::anyhow!(
            "command succeeded, but there were errors parsing the target pattern"
        )));
    }
    Ok(Built {
        report,
        layout,
        options: build_options,
        program_args,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bzlmod fixture registry `fjfj-bzlmod` already maintains
    /// (`tests/fixtures/registry`, containing modules `a`-`f` and `y`) —
    /// reused here rather than duplicated, since this test exercises the
    /// CLI's flag-to-`ResolveOptions` plumbing, not resolution itself
    /// (that's `fjfj-bzlmod`'s own conformance suite).
    fn fixture_registry_dir() -> std::path::PathBuf {
        let bazel_path = std::path::Path::new("crates/fjfj-bzlmod/tests/fixtures/registry");
        if bazel_path.is_dir() {
            return bazel_path.canonicalize().unwrap();
        }
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../fjfj-bzlmod/tests/fixtures/registry")
            .canonicalize()
            .expect("fjfj-bzlmod fixture registry")
    }

    /// `--registry` for the fixture registry and for the part of the Bazel
    /// Central Registry that `bazel_tools`' own dependencies need, which
    /// `fjfj-bzlmod` vendors next to it.
    fn fixture_registry_flags() -> Vec<String> {
        let registry = fixture_registry_dir();
        let bcr = registry.parent().unwrap().join("bcr");
        vec![
            format!("--registry=file://{}", registry.display()),
            format!("--registry=file://{}", bcr.display()),
        ]
    }

    #[test]
    fn registry_flag_resolves_a_fixture_workspace() {
        let mut args = fixture_registry_flags();
        // The workspace root here is the current directory, which is not
        // the place for a MODULE.bazel.lock.
        args.push("--lockfile_mode=off".to_owned());
        let (bzlmod, rest) = bzlmod_flags::extract(&args, "build");
        assert!(rest.is_empty());
        assert_eq!(bzlmod.registry.len(), 2);

        let module_bazel =
            "module(name = 'root', version = '0')\nbazel_dep(name = 'a', version = '1.0')\n";
        let resolution = resolve_bzlmod(module_bazel, std::path::Path::new("."), &bzlmod).unwrap();
        assert!(
            resolution
                .selection
                .keys()
                .any(|k| k.to_string() == "a@1.0")
        );
    }

    #[test]
    fn allow_yanked_versions_and_override_module_flow_into_resolve_options() {
        let args = vec![
            "--allow_yanked_versions=all".to_owned(),
            "--override_module=foo=../foo".to_owned(),
        ];
        let (bzlmod, rest) = bzlmod_flags::extract(&args, "build");
        assert!(rest.is_empty());
        let options = bzlmod_resolve_options(&bzlmod, std::path::Path::new(".")).unwrap();
        assert_eq!(options.yanked, YankedPolicy::AllowAll);
        assert_eq!(options.command_overrides.len(), 1);
        assert_eq!(options.command_overrides[0].0, "foo");
        assert!(options.command_overrides[0].1.is_non_registry());
    }

    #[test]
    fn command_line_error_maps_to_exit_code_2() {
        let err = CliError::CommandLine(anyhow::anyhow!("bad pattern"));
        assert_eq!(err.exit_code(), ExitCode::CommandLineProblem);
        assert_eq!(err.stderr_line(), "ERROR: bad pattern");
    }

    #[test]
    fn build_error_maps_to_exit_code_1() {
        let err = CliError::Build(anyhow::anyhow!("build failed"));
        assert_eq!(err.exit_code(), ExitCode::BuildFailed);
        assert_eq!(err.stderr_line(), "ERROR: build failed");
    }

    #[test]
    fn internal_error_maps_to_exit_code_37_and_is_fatal() {
        let err = CliError::Internal(anyhow::anyhow!("unreachable state"));
        assert_eq!(err.exit_code(), ExitCode::InternalError);
        assert_eq!(err.stderr_line(), "FATAL: unreachable state");
    }

    /// A scratch workspace directory, removed when dropped.
    struct Scratch(std::path::PathBuf);

    impl Scratch {
        fn new(name: &str) -> Scratch {
            let dir = std::env::temp_dir().join(format!("fjfj-cli-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            Scratch(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn resolve_in(workspace: &std::path::Path, extra: &[&str]) -> Result<Resolution, CliError> {
        let mut args = fixture_registry_flags();
        args.extend(extra.iter().map(|a| (*a).to_owned()));
        let (bzlmod, rest) = bzlmod_flags::extract(&args, "build");
        assert!(rest.is_empty(), "{rest:?}");
        let module_bazel =
            "module(name = 'root', version = '0')\nbazel_dep(name = 'a', version = '1.0')\n";
        resolve_bzlmod(module_bazel, workspace, &bzlmod)
    }

    #[test]
    fn resolving_writes_module_bazel_lock_unless_the_mode_is_off() {
        let dir = Scratch::new("writes");
        let lock = dir.0.join("MODULE.bazel.lock");
        resolve_in(&dir.0, &["--lockfile_mode=off"]).unwrap();
        assert!(!lock.exists());
        // `file://` registries are not locked, so what is written is an
        // empty lockfile, in Bazel's format.
        resolve_in(&dir.0, &[]).unwrap();
        assert_eq!(
            std::fs::read_to_string(&lock).unwrap(),
            fjfj_bzlmod::lockfile::Lockfile::default().to_text()
        );
    }

    #[test]
    fn error_mode_does_not_write_and_refuses_a_lockfile_it_cannot_read() {
        let dir = Scratch::new("error");
        let lock = dir.0.join("MODULE.bazel.lock");
        resolve_in(&dir.0, &["--lockfile_mode=error"]).unwrap();
        assert!(!lock.exists());
        std::fs::write(&lock, "{\"lockFileVersion\": 27}").unwrap();
        let Err(e) = resolve_in(&dir.0, &["--lockfile_mode=error"]) else {
            panic!("accepted");
        };
        assert!(
            e.to_string()
                .contains("is not supported by this version of Bazel"),
            "{e}"
        );
        // `update` replaces it.
        resolve_in(&dir.0, &["--lockfile_mode=update"]).unwrap();
        assert!(
            std::fs::read_to_string(&lock)
                .unwrap()
                .contains("\"lockFileVersion\": 28")
        );
    }

    #[test]
    fn a_bad_lockfile_mode_is_a_command_line_error() {
        let dir = Scratch::new("bad-mode");
        let Err(CliError::CommandLine(e)) = resolve_in(&dir.0, &["--lockfile_mode=sometimes"])
        else {
            panic!("accepted");
        };
        assert!(e.to_string().contains("lockfile_mode"), "{e}");
    }

    #[test]
    fn fetch_flags_are_read_as_bazel_reads_them() {
        let args: Vec<String> = [
            "--repo=@a",
            "--repo",
            "@@b+",
            "--repository_cache=",
            "--distdir=/d",
            "--override_repository=lib=/p",
            "--credential_helper=example.com=/bin/h",
            "--credential_helper_timeout=2m",
            "--registry=x",
        ]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
        let (flags, rest) = fetch_command::extract(&args).unwrap();
        assert_eq!(flags.repos, ["@a", "@@b+"]);
        assert_eq!(flags.repository_cache, Some(None));
        assert_eq!(flags.distdirs, [std::path::PathBuf::from("/d")]);
        assert_eq!(
            flags.repo_overrides,
            [("lib".to_owned(), std::path::PathBuf::from("/p"))]
        );
        assert_eq!(
            flags.credential_helpers[0].scope.as_deref(),
            Some("example.com")
        );
        assert_eq!(
            flags.credential_helper_timeout,
            Some(std::time::Duration::from_secs(120))
        );
        assert_eq!(rest, ["--registry=x"]);
        for (flag, message) in [
            (
                "--override_repository=lib",
                "While parsing option --override_repository=lib: Repository overrides must be of the form 'repository-name=path'",
            ),
            (
                "--credential_helper==/h",
                "While parsing option --credential_helper==/h: Credential helper scope must not be empty",
            ),
            (
                "--credential_helper=",
                "While parsing option --credential_helper=: Credential helper path must not be empty",
            ),
        ] {
            let Err(CliError::CommandLine(e)) = fetch_command::extract(&[flag.to_owned()]) else {
                panic!("{flag} accepted");
            };
            assert_eq!(e.to_string(), message);
        }
    }

    #[test]
    fn fetch_makes_a_repository_an_extension_generates_and_locks_the_extension() {
        let dir = Scratch::new("fetch");
        let module = "module(name = 'root', version = '0')\n\
            bazel_dep(name = 'ext', version = '2.0')\n\
            e = use_extension('@ext//:ext.bzl', 'gen')\n\
            use_repo(e, 'r1')\n";
        std::fs::write(dir.0.join("BUILD.bazel"), "").unwrap();
        let mut args = fixture_registry_flags();
        args.push("--lockfile_mode=update".to_owned());
        let (bzlmod, rest) = bzlmod_flags::extract(&args, "fetch");
        assert!(rest.is_empty(), "{rest:?}");
        let flags = fetch_command::FetchFlags {
            repos: vec!["@r1".to_owned()],
            output_base: Some(dir.0.join("ob")),
            repository_cache: Some(None),
            ..fetch_command::FetchFlags::default()
        };
        fetch_command::run(&flags, &bzlmod, &dir.0, module).unwrap();
        assert!(dir.0.join("ob/external/ext++gen+r1/BUILD.bazel").is_file());
        let lock = std::fs::read_to_string(dir.0.join("MODULE.bazel.lock")).unwrap();
        assert!(lock.contains("\"@@ext+//:ext.bzl%gen\""), "{lock}");
        assert!(lock.contains("\"usagesDigest\""), "{lock}");
        // A repository that nothing names is the command line's problem, and
        // one that cannot be made is exit code 8.
        let asked = |repo: &str| fetch_command::FetchFlags {
            repos: vec![repo.to_owned()],
            output_base: Some(dir.0.join("ob")),
            repository_cache: Some(None),
            ..fetch_command::FetchFlags::default()
        };
        let Err(CliError::CommandLine(e)) =
            fetch_command::run(&asked("@nope"), &bzlmod, &dir.0, module)
        else {
            panic!("accepted");
        };
        assert!(
            e.to_string().contains("no repository visible as '@nope'"),
            "{e}"
        );
        let Err(e) = fetch_command::run(&asked("@@nope+"), &bzlmod, &dir.0, module) else {
            panic!("accepted");
        };
        assert_eq!(e.exit_code(), ExitCode::Interrupted);
    }

    #[test]
    fn build_makes_the_repositories_its_patterns_reach_and_writes_the_lock_as_the_mode_says() {
        let dir = Scratch::new("build-repos");
        let module = "module(name = 'root', version = '0')\n\
            bazel_dep(name = 'ext', version = '2.0')\n\
            e = use_extension('@ext//:ext.bzl', 'gen')\n\
            use_repo(e, 'r1')\n";
        std::fs::write(dir.0.join("BUILD.bazel"), "").unwrap();
        let mut args = fixture_registry_flags();
        args.push("--lockfile_mode=update".to_owned());
        let (bzlmod, _) = bzlmod_flags::extract(&args, "build");
        let flags = fetch_command::FetchFlags {
            output_base: Some(dir.0.join("ob")),
            repository_cache: Some(None),
            ..fetch_command::FetchFlags::default()
        };
        let patterns = ["@r1//:all".to_owned()];
        let loaded =
            fetch_command::run_for_build(&flags, &bzlmod, &dir.0, module, &patterns, "", None)
                .unwrap();
        assert_eq!(loaded.targets.failures, []);
        assert!(loaded.resolution.selection.keys().count() >= 2);
        assert!(dir.0.join("ob/external/ext++gen+r1/BUILD.bazel").is_file());
        let lock = std::fs::read_to_string(dir.0.join("MODULE.bazel.lock")).unwrap();
        assert!(lock.contains("\"@@ext+//:ext.bzl%gen\""), "{lock}");
        // No pattern names a repository: nothing is made, and the lock is still written.
        let other = Scratch::new("build-none");
        std::fs::write(other.0.join("BUILD.bazel"), "").unwrap();
        let none = fetch_command::FetchFlags {
            output_base: Some(other.0.join("ob")),
            repository_cache: Some(None),
            ..fetch_command::FetchFlags::default()
        };
        let patterns = ["//:all".to_owned()];
        fetch_command::run_for_build(&none, &bzlmod, &other.0, module, &patterns, "", None)
            .unwrap();
        assert!(!other.0.join("ob/external/ext++gen+r1").exists());
        assert!(other.0.join("MODULE.bazel.lock").is_file());
    }

    #[test]
    fn build_runs_a_genrule_and_leaves_its_output_where_bazel_does() {
        let dir = Scratch::new("build-genrule");
        let module = "module(name = 'root', version = '0')\n";
        std::fs::write(
            dir.0.join("BUILD.bazel"),
            "genrule(name = 'g', srcs = ['in.txt'], outs = ['out.txt'], cmd = 'cat $(SRCS) > $@ && echo $(location in.txt) >> $@')\n\
             genrule(name = 'bad', outs = ['bad.txt'], cmd = 'exit 3')\n",
        )
        .unwrap();
        std::fs::write(dir.0.join("in.txt"), "hello\n").unwrap();
        let mut args = fixture_registry_flags();
        args.push("--lockfile_mode=update".to_owned());
        let (bzlmod, _) = bzlmod_flags::extract(&args, "build");
        let flags = fetch_command::FetchFlags {
            output_base: Some(dir.0.join("ob")),
            repository_cache: Some(None),
            ..fetch_command::FetchFlags::default()
        };
        let options = build_command::Options {
            configuration: fjfj_graph::Configuration {
                cpu: "k8".into(),
                ..fjfj_graph::Configuration::default()
            },
            platform: None,
            extra_toolchains: Vec::new(),
            extra_execution_platforms: Vec::new(),
            host_platform: None,
            aspects: Vec::new(),
            output_groups: Vec::new(),
            keep_going: true,
            build: true,
            symlink_prefix: "bazel-".into(),
            jobs: None,
            strategy: fjfj_exec::run::Options::default().strategy,
            show_result: 1,
            record_execution_platforms: false,
            test: None,
            workspace_status: None,
            incompatible: None,
        };
        let patterns = ["//:g".to_owned(), "//:bad".to_owned()];
        // The build is blocking work that needs a runtime to be current.
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let root = dir.0.clone();
        let loaded = runtime
            .block_on(async move {
                tokio::task::spawn_blocking(move || {
                    fetch_command::run_for_build(
                        &flags,
                        &bzlmod,
                        &root,
                        module,
                        &patterns,
                        "",
                        Some(&options),
                    )
                })
                .await
            })
            .unwrap()
            .unwrap();
        let report = loaded.report.expect("the targets were built");
        let made = dir.0.join("bazel-bin/out.txt");
        assert_eq!(std::fs::read_to_string(made).unwrap(), "hello\n./in.txt\n");
        assert_eq!(report.failures.len(), 1);
        assert_eq!(report.failures[0].owner, "//:bad");
        assert_eq!(
            report.failures[0].message,
            "(Exit 3): bash failed: error executing Genrule command (from genrule rule target //:bad) /bin/bash -c 'source external/bazel_tools/tools/genrule/genrule-setup.sh; exit 3'"
        );
        let good = report.results.iter().find(|r| r.label.name == "g").unwrap();
        assert!(good.built);
        assert_eq!(good.files, ["bazel-bin/out.txt"]);
        assert!(
            !report
                .results
                .iter()
                .find(|r| r.label.name == "bad")
                .unwrap()
                .built
        );
    }

    #[test]
    fn test_runs_the_tests_among_its_targets_and_remembers_the_ones_that_passed() {
        use build_command::TestStatus;
        let dir = Scratch::new("test-run");
        // @bazel_tools' tools/test/BUILD loads rules_shell, which the offline
        // fixture registry cannot serve: a stand-in for its `sh_binary`.
        let shell = dir.0.join("rules_shell");
        std::fs::create_dir_all(shell.join("shell")).unwrap();
        std::fs::write(shell.join("MODULE.bazel"), "module(name = 'rules_shell')\n").unwrap();
        std::fs::write(shell.join("BUILD.bazel"), "").unwrap();
        std::fs::write(shell.join("shell/BUILD.bazel"), "").unwrap();
        std::fs::write(
            shell.join("shell/sh_binary.bzl"),
            "def sh_binary(**kwargs):\n    pass\n",
        )
        .unwrap();
        let module = "module(name = 'root', version = '0')\nbazel_dep(name = 'rules_shell', version = '0.6.1')\n";
        std::fs::write(
            dir.0.join("defs.bzl"),
            r#"
def _impl(ctx):
    exe = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.write(exe, "echo testing $TEST_TARGET args=$@\nexit " + str(ctx.attr.exit) + "\n", is_executable = True)
    return [DefaultInfo(executable = exe)]

my_test = rule(implementation = _impl, test = True, attrs = {"exit": attr.int()})
"#,
        )
        .unwrap();
        std::fs::write(
            dir.0.join("BUILD.bazel"),
            "load(':defs.bzl', 'my_test')\nmy_test(name = 'good', size = 'small', args = ['a', 'b c'])\nmy_test(name = 'bad', exit = 3)\n",
        )
        .unwrap();
        let mut args = fixture_registry_flags();
        args.push("--lockfile_mode=update".to_owned());
        let (bzlmod, _) = bzlmod_flags::extract(&args, "test");
        let flags = fetch_command::FetchFlags {
            output_base: Some(dir.0.join("ob")),
            repository_cache: Some(None),
            repo_overrides: vec![("rules_shell".to_owned(), shell)],
            ..fetch_command::FetchFlags::default()
        };
        let options = build_command::Options {
            configuration: fjfj_graph::Configuration {
                cpu: "k8".into(),
                ..fjfj_graph::Configuration::default()
            },
            platform: None,
            extra_toolchains: Vec::new(),
            extra_execution_platforms: Vec::new(),
            host_platform: None,
            aspects: Vec::new(),
            output_groups: Vec::new(),
            keep_going: true,
            build: true,
            symlink_prefix: "bazel-".into(),
            jobs: None,
            strategy: fjfj_exec::run::Options::default().strategy,
            show_result: 1,
            record_execution_platforms: false,
            test: Some(fjfj_bazel_compat::test_flags::TestOutput::Summary),
            workspace_status: None,
            incompatible: None,
        };
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let run_once = || {
            let (flags, bzlmod, root, options) = (
                flags.clone(),
                bzlmod.clone(),
                dir.0.clone(),
                options.clone(),
            );
            runtime
                .block_on(async move {
                    tokio::task::spawn_blocking(move || {
                        fetch_command::run_for_build(
                            &flags,
                            &bzlmod,
                            &root,
                            module,
                            &["//:good".to_owned(), "//:bad".to_owned()],
                            "",
                            Some(&options),
                        )
                    })
                    .await
                })
                .unwrap()
                .unwrap()
                .report
                .expect("built")
        };
        let first = run_once();
        assert!(
            first.analysis_errors.is_empty(),
            "{:?}",
            first.analysis_errors
        );
        let statuses: Vec<(String, TestStatus, bool)> = first
            .tests
            .iter()
            .map(|t| (t.label.name.clone(), t.status.clone(), t.cached))
            .collect();
        assert_eq!(
            statuses,
            [
                ("bad".to_owned(), TestStatus::Failed(3), false),
                ("good".to_owned(), TestStatus::Passed, false)
            ]
        );
        let log = std::fs::read_to_string(dir.0.join("bazel-testlogs/good/test.log")).unwrap();
        assert!(log.contains("testing //:good args=a b c"), "{log}");
        assert!(dir.0.join("bazel-testlogs/good/test.xml").is_file());
        // The test that passed is not run again; the one that failed is.
        let second = run_once();
        let again: Vec<(String, bool)> = second
            .tests
            .iter()
            .map(|t| (t.label.name.clone(), t.cached))
            .collect();
        assert_eq!(
            again,
            [("bad".to_owned(), false), ("good".to_owned(), true)]
        );
    }
}
