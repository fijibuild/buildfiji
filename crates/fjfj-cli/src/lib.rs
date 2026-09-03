//! Command dispatch. Parses Bazel-compatible flags, sets up telemetry, and
//! runs the requested command.

use clap::Parser;
use fjfj_bazel_compat::{Cli, Command, TargetPattern, diagnostics_flags};

pub fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let _telemetry = fjfj_telemetry::init()?;
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run(cli))
}

#[tracing::instrument(skip_all)]
async fn run(cli: Cli) -> anyhow::Result<()> {
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
                .collect::<Result<Vec<_>, _>>()?;
            tracing::info!(?patterns, ?diagnostics, "build requested");
            anyhow::bail!("fjfj build is not implemented yet; see `bd ready`")
        }
        other => anyhow::bail!("command not implemented yet: {other:?}"),
    }
}
