//! `fjfj run` (buildfiji-gwl.8): build one target and run it.
//!
//! As `bazel run` on Linux does it (Bazel 9.2.0): the program is the
//! executable of the target, run with the real path in `argv[0]`, in the
//! `_main` directory of its runfiles tree, with the environment fjfj was run
//! in and `BUILD_WORKSPACE_DIRECTORY`, `BUILD_WORKING_DIRECTORY`,
//! `BUILD_EXECROOT` and `BUILD_ID` added. Its exit code is `run`'s.

use crate::{CliError, build_main};
use fjfj_bazel_compat::TargetArgs;
use std::os::unix::process::ExitStatusExt;
use std::process::Command;

pub(crate) async fn run(args: TargetArgs) -> Result<(), CliError> {
    let built = build_main(args, "run", true).await?;
    let [result] = &built.report.results[..] else {
        return Err(CliError::Build(anyhow::anyhow!(
            "Only one target can be run, not {}",
            built.report.results.len()
        )));
    };
    let layout = &built.layout;
    let Some(executable) = &result.target.executable else {
        let first = result.target.files.to_vec().into_iter().next();
        let shown = first.map_or_else(
            || crate::build_command::label_name(&result.label),
            |f| layout.resolve(&f).display().to_string(),
        );
        return Err(CliError::Build(anyhow::anyhow!(
            "Non-existent or non-executable {shown}"
        )));
    };
    let program = layout.resolve(executable);
    // The runfiles tree is beside the executable.
    let runfiles = std::path::PathBuf::from(format!("{}.runfiles", program.display()));
    let cwd = if runfiles.is_dir() {
        runfiles.join(crate::build_command::MAIN_REPO_NAME)
    } else {
        layout.workspace.clone()
    };
    let shown = crate::build_command::shown_path(
        &built.options.symlink_prefix,
        &built.options.configuration,
        executable,
    );
    if built.program_args.is_empty() {
        eprintln!("INFO: Running command line: {shown}");
    } else {
        eprintln!("INFO: Running command line: {shown} <args omitted>");
    }
    let working_directory = std::env::current_dir()
        .map_err(|e| CliError::Internal(anyhow::anyhow!("couldn't get current directory: {e}")))?;
    let status = Command::new(&program)
        .args(&built.program_args)
        .current_dir(&cwd)
        .env("PWD", &cwd)
        .env("BUILD_WORKSPACE_DIRECTORY", &layout.workspace)
        .env("BUILD_WORKING_DIRECTORY", working_directory)
        .env("BUILD_EXECROOT", layout.execroot())
        .env("BUILD_ID", build_id())
        .status()
        .map_err(|e| {
            CliError::Build(anyhow::anyhow!("Cannot execute {}: {e}", program.display()))
        })?;
    match (status.code(), status.signal()) {
        (Some(0), _) => Ok(()),
        (Some(code), _) => Err(CliError::Program(u8::try_from(code).unwrap_or(1))),
        (None, Some(signal)) => Err(CliError::Program(u8::try_from(128 + signal).unwrap_or(255))),
        _ => Err(CliError::Program(1)),
    }
}

/// An id for this invocation: a random-looking UUID.
fn build_id() -> String {
    use sha2::Digest as _;
    let seed = format!(
        "{:?}{}{}",
        std::time::SystemTime::now(),
        std::process::id(),
        std::env::args().collect::<Vec<_>>().join(" ")
    );
    let hash = sha2::Sha256::digest(seed.as_bytes());
    let h = hex::encode(&hash[..16]);
    format!(
        "{}-{}-{}-{}-{}",
        &h[..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..32]
    )
}
