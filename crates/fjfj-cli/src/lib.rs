//! Command dispatch. Parses Bazel-compatible flags, sets up telemetry, and
//! runs the requested command.
//!
//! Exit codes and stderr line prefixes (`ERROR:`/`FATAL:`) follow Bazel's,
//! since CI scripts grep for them and check `$?` (see
//! `fjfj_bazel_compat::exit_code`); a bare `anyhow::Result` return from
//! `main` would always exit 1 with Rust's own `Error: {debug}` formatting,
//! which matches neither.

use clap::Parser;
use fjfj_bazel_compat::bzlmod_flags::BzlmodFlags;
use fjfj_bazel_compat::exit_code::{ExitCode, messages};
use fjfj_bazel_compat::{
    Cli, Command, TargetPattern, bes_flags, bzlmod_flags, canonicalize_flags, clap_flags,
    diagnostics_flags, execution_log_flags, flag_alias, misc_flags, output_filter, remote_flags,
    workspace_status_flags,
};
use fjfj_bzlmod::attrs::AttrValue;
use fjfj_bzlmod::discovery::RegistrySource;
use fjfj_bzlmod::overrides::{ModuleOverride, NonRegistryOverride, RepoRule, RepoSpec};
use fjfj_bzlmod::{
    BAZEL_CENTRAL_REGISTRY, Registry, Resolution, ResolveOptions, WorkspaceIncludeSource,
    YankedPolicy,
};
use fjfj_remote::execution_log::{CompactExecutionLogWriter, EntryType, ExecLogEntry, Invocation};

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
    /// A bug in fjfj itself, not something the user's command line or
    /// build can fix.
    #[error("{0}")]
    Internal(anyhow::Error),
}

impl CliError {
    fn exit_code(&self) -> ExitCode {
        match self {
            CliError::CommandLine(_) => ExitCode::CommandLineProblem,
            CliError::Build(_) => ExitCode::BuildFailed,
            CliError::Internal(_) => ExitCode::InternalError,
        }
    }

    /// The line to write to stderr: `ERROR: ...` for anything the user
    /// can act on, `FATAL: ...` for [`CliError::Internal`].
    fn stderr_line(&self) -> String {
        match self {
            CliError::CommandLine(e) | CliError::Build(e) => messages::error(e),
            CliError::Internal(e) => messages::fatal(e),
        }
    }
}

/// The registries `--registry` names, or Bazel's own default list (just
/// `https://bcr.bazel.build`) when it wasn't given at all — repeatable
/// `--registry` *replaces* the default rather than adding to it.
fn bzlmod_registries(flags: &BzlmodFlags) -> Result<Vec<Registry>, CliError> {
    let urls: Vec<&str> = if flags.registry.is_empty() {
        vec![BAZEL_CENTRAL_REGISTRY]
    } else {
        flags.registry.iter().map(String::as_str).collect()
    };
    urls.into_iter()
        .map(|url| {
            Registry::remote(url)
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
    })
}

/// Resolves the bzlmod module graph for `module_bazel_text` (the root
/// `MODULE.bazel`, already read from `workspace_root`) against the flags a
/// command was given.
fn resolve_bzlmod(
    module_bazel_text: &str,
    workspace_root: &std::path::Path,
    flags: &BzlmodFlags,
) -> Result<Resolution, CliError> {
    let registries = bzlmod_registries(flags)?;
    let source = RegistrySource::new(registries);
    let options = bzlmod_resolve_options(flags, workspace_root)?;
    fjfj_bzlmod::resolve(module_bazel_text, &source, &options)
        .map_err(|e| CliError::Build(anyhow::anyhow!(e)))
}

pub fn main() -> std::process::ExitCode {
    let cli = Cli::parse(); // exits 2 itself on a flag-syntax error
    let rt = match tokio::runtime::Runtime::new() {
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
            eprintln!("{}", e.stderr_line());
            e.exit_code().into()
        }
    }
}

#[tracing::instrument(skip_all)]
async fn run(cli: Cli) -> Result<(), CliError> {
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
        Command::Build(args) => {
            let (aliases, rest) = flag_alias::extract(&args.patterns)
                .map_err(|e| CliError::CommandLine(anyhow::Error::from(e)))?;
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
            const BUILD_IMPLEMENTED: &[&[&str]] = &[
                flag_alias::IMPLEMENTED,
                diagnostics_flags::IMPLEMENTED,
                workspace_status_flags::IMPLEMENTED,
                misc_flags::IMPLEMENTED,
                output_filter::IMPLEMENTED,
                execution_log_flags::IMPLEMENTED,
                remote_flags::IMPLEMENTED,
                bes_flags::IMPLEMENTED,
                bzlmod_flags::IMPLEMENTED,
            ];
            let implemented: Vec<&'static str> = BUILD_IMPLEMENTED
                .iter()
                .flat_map(|s| s.iter().copied())
                .collect();
            clap_flags::validate(&rest, "build", &implemented)
                .map_err(|e| CliError::CommandLine(anyhow::Error::from(e)))?;
            let (diagnostics, rest) = diagnostics_flags::extract(&rest, "build");
            let (workspace_status, rest) = workspace_status_flags::extract(&rest, "build");
            let (misc, rest) = misc_flags::extract(&rest, "build");
            let (output_filter_flags, rest) = output_filter::extract(&rest, "build");
            let (execution_log, rest) = execution_log_flags::extract(&rest, "build");
            let (remote, rest) = remote_flags::extract(&rest, "build");
            let (bes, rest) = bes_flags::extract(&rest, "build");
            let (bzlmod, rest) = bzlmod_flags::extract(&rest, "build");
            // Everything left is a bare positional now that `validate`
            // above has ruled out any unimplemented or unrecognized flag.
            let patterns = rest
                .iter()
                .map(|p| p.parse::<TargetPattern>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(CliError::CommandLine)?;
            let command_line_packages = patterns.iter().map(|p| p.package.clone());
            let _output_filter =
                output_filter::OutputFilter::compile(&output_filter_flags, command_line_packages)
                    .map_err(|e| CliError::CommandLine(anyhow::Error::from(e)))?;
            tracing::info!(
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
                "build requested"
            );
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
            // above. No workspace-root search exists yet, so this is the
            // current directory's own MODULE.bazel — matching how `bazel
            // build` is invoked from the workspace root in this repo today.
            let workspace_root = std::env::current_dir().map_err(|e| {
                CliError::Internal(anyhow::anyhow!("couldn't get current directory: {e}"))
            })?;
            let module_bazel_text = std::fs::read_to_string(workspace_root.join("MODULE.bazel"))
                .map_err(|e| {
                    CliError::CommandLine(anyhow::anyhow!(
                        "no MODULE.bazel found in {}: {e}",
                        workspace_root.display()
                    ))
                })?;
            let resolution = resolve_bzlmod(&module_bazel_text, &workspace_root, &bzlmod)?;
            tracing::info!(
                selected_modules = resolution.selection.keys().count(),
                "bzlmod module graph resolved"
            );
            Err(CliError::Build(anyhow::anyhow!(
                "fjfj build is not implemented yet; see `bd ready`"
            )))
        }
        other => Err(CliError::Build(anyhow::anyhow!(
            "command not implemented yet: {other:?}"
        ))),
    }
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

    #[test]
    fn registry_flag_resolves_a_fixture_workspace() {
        let args = vec![format!(
            "--registry=file://{}",
            fixture_registry_dir().display()
        )];
        let (bzlmod, rest) = bzlmod_flags::extract(&args, "build");
        assert!(rest.is_empty());
        assert_eq!(bzlmod.registry.len(), 1);

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
}
