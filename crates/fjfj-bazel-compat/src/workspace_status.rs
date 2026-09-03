//! `--workspace_status_command`'s output format, and the stable/volatile
//! partition and invalidation contract that make `--stamp` behave the way
//! Bazel documents it (see the "Workspace status" section of
//! <https://bazel.build/docs/user-manual>, and `docs/design/cli-compat.md`):
//!
//! - The command prints zero or more `KEY VALUE` lines to stdout, one per
//!   line; keys are `[A-Z_]+` and must not repeat.
//! - Every key whose name starts with `STABLE_` is a "stable" key; every
//!   other key is "volatile". Bazel always adds its own built-in keys on
//!   top of whatever the command printed: `BUILD_EMBED_LABEL`,
//!   `BUILD_HOST`, `BUILD_USER` (stable, despite not starting with
//!   `STABLE_` — they're exceptions baked into Bazel, not into the
//!   prefix rule) and `BUILD_TIMESTAMP`, `FORMATTED_DATE` (volatile).
//! - `bazel-out/stable-status.txt` holds the stable keys,
//!   `bazel-out/volatile-status.txt` the volatile ones.
//! - The contract: a change to the *stable* file invalidates stamped
//!   actions that depend on it (rerun them). A change to the *volatile*
//!   file alone never does — Bazel "pretends" it never changes, precisely
//!   so a timestamp changing every build doesn't force a rebuild every
//!   build. [`WorkspaceStatus::invalidates`] encodes exactly that: it
//!   compares stable maps only.
//!
//! This module is the pure parsing/model half; actually running the
//! command, reading the environment for `BUILD_HOST`/`BUILD_USER`, and
//! writing the two files is `fjfj-exec`'s job (it already owns process
//! execution), which calls into [`WorkspaceStatus::parse`].

use std::collections::BTreeMap;

/// A workspace status key Bazel computes itself; a user's
/// `--workspace_status_command` output for one of these is not an error,
/// it's just superseded — matching the doc's framing that Bazel "always
/// outputs" these regardless of the command.
const BUILTIN_STABLE: &[&str] = &["BUILD_EMBED_LABEL", "BUILD_HOST", "BUILD_USER"];
const BUILTIN_VOLATILE: &[&str] = &["BUILD_TIMESTAMP", "FORMATTED_DATE"];

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum WorkspaceStatusError {
    #[error("workspace status line has no KEY VALUE separator: {0:?}")]
    MissingSeparator(String),
    #[error("workspace status key {0:?} is not all upper-case letters and underscores")]
    InvalidKey(String),
    #[error("workspace status key {0:?} is set more than once")]
    DuplicateKey(String),
}

/// The parsed, partitioned output of `--workspace_status_command`, plus
/// Bazel's own built-in keys.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceStatus {
    pub stable: BTreeMap<String, String>,
    pub volatile: BTreeMap<String, String>,
}

fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.bytes().all(|b| b == b'_' || b.is_ascii_uppercase())
}

impl WorkspaceStatus {
    /// Parse `raw` (the command's stdout; empty if there was no command,
    /// per `--workspace_status_command=/bin/true`'s documented use to
    /// disable it), validate it, partition it by the `STABLE_` prefix,
    /// then add Bazel's built-ins.
    ///
    /// `timestamp_unix_seconds`/`formatted_date` are passed in rather than
    /// read from the clock here so parsing stays pure and testable; see
    /// `fjfj-exec` for where they come from.
    pub fn parse(
        raw: &str,
        embed_label: &str,
        host: &str,
        user: &str,
        timestamp_unix_seconds: u64,
        formatted_date: &str,
    ) -> Result<WorkspaceStatus, WorkspaceStatusError> {
        let mut stable = BTreeMap::new();
        let mut volatile = BTreeMap::new();
        let mut seen = BTreeMap::new();

        for line in raw.lines() {
            if line.is_empty() {
                continue;
            }
            let (key, value) = line
                .split_once(' ')
                .ok_or_else(|| WorkspaceStatusError::MissingSeparator(line.to_string()))?;
            if !valid_key(key) {
                return Err(WorkspaceStatusError::InvalidKey(key.to_string()));
            }
            if seen.insert(key.to_string(), ()).is_some() {
                return Err(WorkspaceStatusError::DuplicateKey(key.to_string()));
            }
            if BUILTIN_STABLE.contains(&key) || BUILTIN_VOLATILE.contains(&key) {
                continue; // superseded by Bazel's own value below
            }
            if key.starts_with("STABLE_") {
                stable.insert(key.to_string(), value.to_string());
            } else {
                volatile.insert(key.to_string(), value.to_string());
            }
        }

        stable.insert("BUILD_EMBED_LABEL".to_string(), embed_label.to_string());
        stable.insert("BUILD_HOST".to_string(), host.to_string());
        stable.insert("BUILD_USER".to_string(), user.to_string());
        volatile.insert(
            "BUILD_TIMESTAMP".to_string(),
            timestamp_unix_seconds.to_string(),
        );
        volatile.insert("FORMATTED_DATE".to_string(), formatted_date.to_string());

        Ok(WorkspaceStatus { stable, volatile })
    }

    /// Contents of `bazel-out/stable-status.txt`.
    pub fn render_stable(&self) -> String {
        render(&self.stable)
    }

    /// Contents of `bazel-out/volatile-status.txt`.
    pub fn render_volatile(&self) -> String {
        render(&self.volatile)
    }

    /// Whether a stamped action that depended on `previous`'s status
    /// should be invalidated now that the status is `self`: true iff the
    /// *stable* keys differ. A volatile-only change (the common case —
    /// `BUILD_TIMESTAMP` changes on every build) never invalidates on its
    /// own, matching Bazel's documented contract.
    pub fn invalidates(&self, previous: &WorkspaceStatus) -> bool {
        self.stable != previous.stable
    }
}

fn render(map: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    for (k, v) in map {
        out.push_str(k);
        out.push(' ');
        out.push_str(v);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(raw: &str) -> Result<WorkspaceStatus, WorkspaceStatusError> {
        WorkspaceStatus::parse(
            raw,
            "stable-label",
            "host1",
            "alice",
            1_700_000_000,
            "2023 Nov 14 22:13:20 Tue",
        )
    }

    #[test]
    fn empty_command_output_still_gets_builtins() {
        let s = parse("").unwrap();
        assert_eq!(s.stable["BUILD_EMBED_LABEL"], "stable-label");
        assert_eq!(s.stable["BUILD_HOST"], "host1");
        assert_eq!(s.stable["BUILD_USER"], "alice");
        assert_eq!(s.volatile["BUILD_TIMESTAMP"], "1700000000");
        assert_eq!(s.volatile["FORMATTED_DATE"], "2023 Nov 14 22:13:20 Tue");
    }

    #[test]
    fn stable_prefix_partitions_into_stable_file() {
        let s = parse("STABLE_GIT_COMMIT abc123\nRANDOM_SEED 42\n").unwrap();
        assert_eq!(s.stable["STABLE_GIT_COMMIT"], "abc123");
        assert!(!s.volatile.contains_key("STABLE_GIT_COMMIT"));
        assert_eq!(s.volatile["RANDOM_SEED"], "42");
    }

    #[test]
    fn value_may_contain_spaces() {
        let s = parse("STABLE_MESSAGE hello world\n").unwrap();
        assert_eq!(s.stable["STABLE_MESSAGE"], "hello world");
    }

    #[test]
    fn missing_separator_is_an_error() {
        assert_eq!(
            parse("NOVALUE"),
            Err(WorkspaceStatusError::MissingSeparator(
                "NOVALUE".to_string()
            ))
        );
    }

    #[test]
    fn lower_case_key_is_an_error() {
        assert_eq!(
            parse("stable_git_commit abc"),
            Err(WorkspaceStatusError::InvalidKey(
                "stable_git_commit".to_string()
            ))
        );
    }

    #[test]
    fn duplicate_key_is_an_error() {
        assert_eq!(
            parse("STABLE_X 1\nSTABLE_X 2\n"),
            Err(WorkspaceStatusError::DuplicateKey("STABLE_X".to_string()))
        );
    }

    #[test]
    fn user_output_for_a_builtin_key_is_superseded_not_an_error() {
        let s = parse("BUILD_HOST attacker-controlled\n").unwrap();
        assert_eq!(s.stable["BUILD_HOST"], "host1");
    }

    #[test]
    fn render_is_sorted_and_newline_terminated() {
        let s = parse("STABLE_B 2\nSTABLE_A 1\n").unwrap();
        let rendered = s.render_stable();
        let a = rendered.find("STABLE_A").unwrap();
        let b = rendered.find("STABLE_B").unwrap();
        assert!(a < b);
        assert!(rendered.ends_with('\n'));
    }

    #[test]
    fn stable_change_invalidates() {
        let a = parse("STABLE_GIT_COMMIT abc\n").unwrap();
        let b = WorkspaceStatus::parse(
            "STABLE_GIT_COMMIT def\n",
            "stable-label",
            "host1",
            "alice",
            1_700_000_100,
            "2023 Nov 14 22:15:00 Tue",
        )
        .unwrap();
        assert!(b.invalidates(&a));
    }

    #[test]
    fn volatile_only_change_does_not_invalidate() {
        let a = parse("STABLE_GIT_COMMIT abc\n").unwrap();
        // Only the timestamp/formatted date (volatile) changed.
        let b = WorkspaceStatus::parse(
            "STABLE_GIT_COMMIT abc\n",
            "stable-label",
            "host1",
            "alice",
            1_700_000_100,
            "2023 Nov 14 22:15:00 Tue",
        )
        .unwrap();
        assert!(!b.invalidates(&a));
    }
}
