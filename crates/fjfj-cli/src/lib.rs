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
use fjfj_bazel_compat::{Cli, Command, TargetPattern, diagnostics_flags};

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
        Command::Build(args) => {
            let (diagnostics, rest) = diagnostics_flags::extract(&args.patterns, "build");
            let patterns = rest
                .iter()
                .map(|p| p.parse::<TargetPattern>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(CliError::CommandLine)?;
            tracing::info!(?patterns, ?diagnostics, "build requested");
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
