//! Command dispatch. Parses Bazel-compatible flags, sets up telemetry, and
//! runs the requested command.
//!
//! Exit codes and stderr line prefixes (`ERROR:`/`FATAL:`) follow Bazel's,
//! since CI scripts grep for them and check `$?` (see
//! `fjfj_bazel_compat::exit_code`); a bare `anyhow::Result` return from
//! `main` would always exit 1 with Rust's own `Error: {debug}` formatting,
//! which matches neither.

use clap::Parser;
use fjfj_bazel_compat::exit_code::{ExitCode, messages};
use fjfj_bazel_compat::{
    Cli, Command, TargetPattern, canonicalize_flags, diagnostics_flags, workspace_status_flags,
};

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

pub fn main() -> std::process::ExitCode {
    let cli = Cli::parse(); // exits 2 itself on a flag-syntax error
    let Ok(_telemetry) = fjfj_telemetry::init() else {
        eprintln!("{}", messages::fatal("failed to initialize telemetry"));
        return ExitCode::InternalError.into();
    };
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
            let (diagnostics, rest) = diagnostics_flags::extract(&args.patterns, "build");
            let (workspace_status, rest) = workspace_status_flags::extract(&rest, "build");
            let patterns = rest
                .iter()
                .map(|p| p.parse::<TargetPattern>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(CliError::CommandLine)?;
            tracing::info!(
                ?patterns,
                ?diagnostics,
                ?workspace_status,
                "build requested"
            );
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
