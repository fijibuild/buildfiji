//! Bazel's process exit codes and the `ERROR:`/`FATAL:`/`INFO:` stderr line
//! prefixes scripts grep for (`bazel ... ; echo $?`, `grep -q '^ERROR:'`).
//! Both are part of Bazel's observable behaviour (see `docs/ARCHITECTURE.md`
//! principle 1) and are exercised by CI scripts that shell out to `bazel`,
//! so fjfj must reproduce them exactly rather than pick its own numbering.
//!
//! Values and names come from Bazel's own documentation
//! (<https://bazel.build/run/scripts#exit-codes>). Only the codes fjfj can
//! currently produce are wired up by callers; the rest are recorded here so
//! the numbering stays centralised as more of them apply.

use std::fmt;

/// A Bazel-compatible process exit code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ExitCode {
    /// Command completed successfully.
    Success = 0,
    /// Build (or the requested command) failed.
    BuildFailed = 1,
    /// Command line problem: bad or illegal flags, or an unparseable
    /// target pattern.
    CommandLineProblem = 2,
    /// Build succeeded, but some tests failed or timed out.
    TestsFailed = 3,
    /// Build succeeded, but `test` found no tests to run.
    NoTestsFound = 4,
    /// `query` succeeded only partially.
    PartialAnalysisFailure = 7,
    /// Build interrupted (e.g. client Ctrl-C) but shut down in an orderly
    /// way; no partial output, no unverified cache entry.
    Interrupted = 8,
    /// Server lock already held and `--noblock_for_lock` was set.
    LockHeldNoblock = 9,
    /// External environment failure not local to this machine (e.g. a
    /// remote service is down).
    ExternalEnvironmentalIssue = 32,
    /// fjfj ran out of memory and crashed.
    OutOfMemory = 33,
    /// Remote execution, cache, or Build Event Service error.
    RemoteError = 34,
    /// Local environmental issue, suspected permanent (vs. 32's transient
    /// external one) — e.g. a required local tool is missing.
    LocalEnvironmentalIssue = 36,
    /// Unhandled exception or internal fjfj error: a bug, not a build
    /// failure. Corresponds to Bazel's "crash" exit code.
    InternalError = 37,
    /// External dependency error (e.g. a repository rule failed to fetch).
    ExternalDependencyError = 48,
}

impl ExitCode {
    /// The numeric value passed to `std::process::exit`.
    pub const fn code(self) -> i32 {
        self as i32
    }
}

impl fmt::Display for ExitCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl From<ExitCode> for std::process::ExitCode {
    fn from(value: ExitCode) -> Self {
        // Every variant above fits in a u8; Bazel itself never exceeds 63.
        std::process::ExitCode::from(value.code() as u8)
    }
}

/// Bazel's stderr line prefixes. A line starting with one of these is part
/// of the stable, grep-able output surface (unlike free-form progress
/// text), so these are plain `format!`-style helpers rather than a
/// `tracing` layer: callers write the returned line straight to stderr.
pub mod messages {
    use std::fmt::Display;

    /// `ERROR: <message>` — a command failed for a reason the user can
    /// act on (bad input, build failure, missing file).
    pub fn error(message: impl Display) -> String {
        format!("ERROR: {message}")
    }

    /// `FATAL: <message>` — an internal error; the user can't fix this by
    /// changing their command or their build.
    pub fn fatal(message: impl Display) -> String {
        format!("FATAL: {message}")
    }

    /// `INFO: <message>` — informational, not an error.
    pub fn info(message: impl Display) -> String {
        format!("INFO: {message}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_match_bazels_documented_values() {
        assert_eq!(ExitCode::Success.code(), 0);
        assert_eq!(ExitCode::BuildFailed.code(), 1);
        assert_eq!(ExitCode::CommandLineProblem.code(), 2);
        assert_eq!(ExitCode::TestsFailed.code(), 3);
        assert_eq!(ExitCode::NoTestsFound.code(), 4);
        assert_eq!(ExitCode::Interrupted.code(), 8);
        assert_eq!(ExitCode::LocalEnvironmentalIssue.code(), 36);
        assert_eq!(ExitCode::InternalError.code(), 37);
    }

    #[test]
    fn converts_to_process_exit_code() {
        let code: std::process::ExitCode = ExitCode::BuildFailed.into();
        // std::process::ExitCode doesn't expose its value for comparison;
        // exercising the conversion (no panic, right type) is the point.
        let _ = code;
    }

    #[test]
    fn message_prefixes_match_bazel() {
        assert_eq!(messages::error("bad target"), "ERROR: bad target");
        assert_eq!(messages::fatal("panic in worker"), "FATAL: panic in worker");
        assert_eq!(messages::info("Build completed"), "INFO: Build completed");
    }
}
