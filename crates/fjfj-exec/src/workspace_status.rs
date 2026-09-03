//! Runs `--workspace_status_command` and writes `stable-status.txt` /
//! `volatile-status.txt`. The parsing, partitioning and invalidation rules
//! live in `fjfj_bazel_compat::workspace_status` (pure, no I/O); this
//! module is the I/O half: spawn the program, fail the build the way
//! Bazel does on a non-zero exit, fill in the built-in keys from the
//! environment and clock, and write the two files.

use std::path::Path;

use fjfj_bazel_compat::workspace_status::{WorkspaceStatus, WorkspaceStatusError};
use fjfj_bazel_compat::workspace_status_flags::WorkspaceStatusFlags;
use time::OffsetDateTime;

#[derive(Debug, thiserror::Error)]
pub enum ComputeError {
    #[error("workspace status command {0:?} could not be run: {1}")]
    Spawn(std::path::PathBuf, std::io::Error),
    #[error("workspace status command {0:?} exited with {1}")]
    NonZeroExit(std::path::PathBuf, std::process::ExitStatus),
    #[error("workspace status command {0:?} printed invalid output: {1}")]
    Invalid(std::path::PathBuf, WorkspaceStatusError),
}

/// Run `flags.workspace_status_command` (a no-op, matching Bazel's
/// documented `--workspace_status_command=/bin/true`, when unset) and
/// build the resulting [`WorkspaceStatus`].
pub async fn compute(flags: &WorkspaceStatusFlags) -> Result<WorkspaceStatus, ComputeError> {
    let raw = match &flags.workspace_status_command {
        Some(program) => run(program).await?,
        None => String::new(),
    };
    let host = std::env::var("HOSTNAME").unwrap_or_else(|_| hostname_fallback());
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    let now = OffsetDateTime::now_utc();

    WorkspaceStatus::parse(
        &raw,
        &flags.embed_label,
        &host,
        &user,
        now.unix_timestamp().max(0) as u64,
        &format_date(now),
    )
    .map_err(|e| {
        ComputeError::Invalid(
            flags.workspace_status_command.clone().unwrap_or_default(),
            e,
        )
    })
}

/// Write `stable-status.txt` and `volatile-status.txt` under `out_dir`
/// (Bazel's `bazel-out/`), overwriting any existing files.
pub fn write(status: &WorkspaceStatus, out_dir: &Path) -> std::io::Result<()> {
    std::fs::write(out_dir.join("stable-status.txt"), status.render_stable())?;
    std::fs::write(
        out_dir.join("volatile-status.txt"),
        status.render_volatile(),
    )?;
    Ok(())
}

async fn run(program: &Path) -> Result<String, ComputeError> {
    let output = tokio::process::Command::new(program)
        .output()
        .await
        .map_err(|e| ComputeError::Spawn(program.to_path_buf(), e))?;
    if !output.status.success() {
        return Err(ComputeError::NonZeroExit(
            program.to_path_buf(),
            output.status,
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// `hostname(1)`'s output, for platforms where `$HOSTNAME` isn't exported
/// by default (most shells only set it interactively, not for spawned
/// processes). Falls back to `"unknown"` rather than adding a dependency
/// just to call `gethostname(2)`.
fn hostname_fallback() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

/// `FORMATTED_DATE`'s documented format: `yyyy MMM d HH:mm:ss EEE`, UTC.
/// `time::Month`/`time::Weekday`'s `Display` spell the name out in full
/// (`"June"`, `"Friday"`); Bazel's format wants the three-letter form, so
/// take a prefix rather than hand-rolling the calendar math ourselves.
fn format_date(dt: OffsetDateTime) -> String {
    let month = dt.month().to_string();
    let weekday = dt.weekday().to_string();
    format!(
        "{} {} {} {:02}:{:02}:{:02} {}",
        dt.year(),
        &month[..3],
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
        &weekday[..3]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_epoch_dates() {
        // 1970-01-01 00:00:00 UTC was a Thursday.
        assert_eq!(
            format_date(OffsetDateTime::from_unix_timestamp(0).unwrap()),
            "1970 Jan 1 00:00:00 Thu"
        );
        // 2023-06-02 01:44:29 UTC, the doc's own example, was a Friday.
        assert_eq!(
            format_date(OffsetDateTime::from_unix_timestamp(1_685_670_269).unwrap()),
            "2023 Jun 2 01:44:29 Fri"
        );
    }

    #[tokio::test]
    async fn no_command_still_produces_builtins() {
        let status = compute(&WorkspaceStatusFlags::default()).await.unwrap();
        assert!(status.stable.contains_key("BUILD_HOST"));
        assert!(status.stable.contains_key("BUILD_USER"));
        assert!(status.volatile.contains_key("BUILD_TIMESTAMP"));
    }

    #[tokio::test]
    async fn nonzero_exit_fails_the_build() {
        let flags = WorkspaceStatusFlags {
            workspace_status_command: Some(std::path::PathBuf::from("/usr/bin/false")),
            ..Default::default()
        };
        let err = compute(&flags).await.unwrap_err();
        assert!(matches!(err, ComputeError::NonZeroExit(_, _)));
    }

    #[tokio::test]
    async fn command_output_is_parsed() {
        let script = std::env::temp_dir().join(format!(
            "fjfj-workspace-status-test-{}.sh",
            std::process::id()
        ));
        std::fs::write(&script, "#!/bin/sh\necho STABLE_GIT_COMMIT abc123\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let flags = WorkspaceStatusFlags {
            workspace_status_command: Some(script.clone()),
            ..Default::default()
        };
        let status = compute(&flags).await.unwrap();
        std::fs::remove_file(&script).ok();
        assert_eq!(status.stable["STABLE_GIT_COMMIT"], "abc123");
    }

    #[test]
    fn write_creates_both_files() {
        let dir = std::env::temp_dir().join(format!(
            "fjfj-workspace-status-write-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let status =
            WorkspaceStatus::parse("STABLE_X 1\n", "", "h", "u", 0, "1970 Jan 1 00:00:00 Thu")
                .unwrap();
        write(&status, &dir).unwrap();
        let stable = std::fs::read_to_string(dir.join("stable-status.txt")).unwrap();
        let volatile = std::fs::read_to_string(dir.join("volatile-status.txt")).unwrap();
        std::fs::remove_dir_all(&dir).ok();
        assert!(stable.contains("STABLE_X 1"));
        assert!(volatile.contains("BUILD_TIMESTAMP 0"));
    }
}
