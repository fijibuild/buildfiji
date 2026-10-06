//! `fjfj test` (buildfiji-fyz.9, fyz.12): build the targets, run the tests
//! among them, and say how it went.

use crate::build_command::TestStatus;
use crate::{CliError, build_main};
use fjfj_bazel_compat::TargetArgs;

pub(crate) async fn run(args: TargetArgs) -> Result<(), CliError> {
    let built = build_main(args, "test", false).await?;
    let report = &built.report;
    if report.tests.is_empty() {
        eprintln!("ERROR: No test targets were found, yet testing was requested");
        return Err(CliError::NoTests);
    }
    if report
        .tests
        .iter()
        .any(|t| !matches!(t.status, TestStatus::Passed | TestStatus::Skipped))
    {
        return Err(CliError::TestsFailed);
    }
    Ok(())
}
