//! Engine persistence: immutable snapshot + append-only delta log, compacted
//! on idle, crash anywhere (bead buildfiji-2h9.5; crate `fjfj-engine`;
//! docs/design/incremental-engine.md "Persistence"). Store-independent: the
//! model is about ordering and durability, not the file format.
//!
//! Versions are a counter. A write appends a log entry (not yet durable);
//! `FsyncLog` makes the log durable and acknowledges the writes. Compaction
//! writes a temp snapshot at the current version, fsyncs it, renames it over
//! the snapshot, then truncates the log up to the snapshot version.
//!
//! Recovery reads the durable snapshot and replays durable log entries that
//! follow it contiguously. The safety invariant is checked in *every* state,
//! not only after a crash: the recoverable version is never below the
//! acknowledged version.
//!
//! `truncate_after_rename = false` models truncating the log before the new
//! snapshot is durable; a crash in between loses acknowledged writes.

use stateright::{Model, Property};

type Version = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Compaction {
    Idle,
    TempWritten { version: Version, durable: bool },
    Renamed { version: Version },
    Truncated { version: Version },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct State {
    /// In-memory version (may run ahead of what is durable).
    pub mem: Version,
    /// Highest version acknowledged to callers (requires the log to be durable).
    pub acked: Version,
    /// Log entries exist for versions in (log_start, log_written].
    pub log_start: Version,
    pub log_written: Version,
    pub log_durable: Version,
    /// Durable snapshot version.
    pub snapshot: Version,
    pub compaction: Compaction,
    pub writes: u8,
    pub crashes: u8,
    pub compactions: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Write,
    FsyncLog,
    WriteTemp,
    FsyncTemp,
    Rename,
    Truncate,
    FinishCompaction,
    Crash,
}

#[derive(Clone, Copy, Debug)]
pub struct Persistence {
    pub truncate_after_rename: bool,
    pub max_writes: u8,
    pub max_crashes: u8,
    pub max_compactions: u8,
}

impl State {
    /// Version reachable from durable state alone.
    pub fn recoverable(&self) -> Version {
        let durable_log_end = self.log_durable;
        // Durable log entries cover (log_start, durable_log_end]. They are
        // usable only if they start at or before the snapshot version.
        if self.log_start <= self.snapshot && durable_log_end > self.snapshot {
            durable_log_end
        } else {
            self.snapshot
        }
    }
}

impl Model for Persistence {
    type State = State;
    type Action = Action;

    fn init_states(&self) -> Vec<State> {
        vec![State {
            mem: 0,
            acked: 0,
            log_start: 0,
            log_written: 0,
            log_durable: 0,
            snapshot: 0,
            compaction: Compaction::Idle,
            writes: 0,
            crashes: 0,
            compactions: 0,
        }]
    }

    fn actions(&self, s: &State, actions: &mut Vec<Action>) {
        if s.writes < self.max_writes {
            actions.push(Action::Write);
        }
        if s.log_written > s.log_durable {
            actions.push(Action::FsyncLog);
        }
        match s.compaction {
            Compaction::Idle => {
                if s.compactions < self.max_compactions && s.mem > s.snapshot {
                    actions.push(Action::WriteTemp);
                }
            }
            Compaction::TempWritten { durable: false, .. } => actions.push(Action::FsyncTemp),
            Compaction::TempWritten { durable: true, .. } => {
                actions.push(Action::Rename);
                if !self.truncate_after_rename {
                    actions.push(Action::Truncate);
                }
            }
            Compaction::Renamed { .. } => actions.push(Action::Truncate),
            Compaction::Truncated { .. } => actions.push(Action::FinishCompaction),
        }
        if s.crashes < self.max_crashes {
            actions.push(Action::Crash);
        }
    }

    fn next_state(&self, s: &State, a: Action) -> Option<State> {
        let mut t = *s;
        match a {
            Action::Write => {
                t.mem += 1;
                t.log_written = t.mem;
                t.writes += 1;
            }
            Action::FsyncLog => {
                t.log_durable = t.log_written;
                t.acked = t.log_durable;
            }
            Action::WriteTemp => {
                t.compaction = Compaction::TempWritten {
                    version: s.mem,
                    durable: false,
                };
                t.compactions += 1;
            }
            Action::FsyncTemp => {
                let Compaction::TempWritten { version, .. } = s.compaction else {
                    return None;
                };
                t.compaction = Compaction::TempWritten {
                    version,
                    durable: true,
                };
            }
            Action::Rename => {
                let Compaction::TempWritten {
                    version,
                    durable: true,
                } = s.compaction
                else {
                    return None;
                };
                t.snapshot = version;
                t.compaction = Compaction::Renamed { version };
            }
            Action::Truncate => {
                let version = match s.compaction {
                    Compaction::Renamed { version } | Compaction::TempWritten { version, .. } => {
                        version
                    }
                    _ => return None,
                };
                t.log_start = version;
                t.compaction = Compaction::Truncated { version };
            }
            Action::FinishCompaction => t.compaction = Compaction::Idle,
            Action::Crash => {
                t.crashes += 1;
                // Non-durable log tail and temp snapshot are lost.
                t.log_written = s.log_durable;
                t.compaction = Compaction::Idle;
                // Recovery rebuilds memory from durable state.
                t.mem = s.recoverable();
            }
        }
        Some(t)
    }

    fn properties(&self) -> Vec<Property<Self>> {
        vec![
            Property::always("acknowledged writes are recoverable", |_, s: &State| {
                s.recoverable() >= s.acked
            }),
            Property::always("log is contiguous with snapshot", |_, s: &State| {
                s.log_start <= s.snapshot
            }),
            Property::always("memory never behind acknowledged", |_, s: &State| {
                s.mem >= s.acked
            }),
            Property::sometimes(
                "compaction completes after acknowledged writes",
                |_, s: &State| {
                    s.compaction == Compaction::Idle
                        && s.snapshot > 0
                        && s.acked >= s.snapshot
                        && s.compactions > 0
                },
            ),
            Property::sometimes("recovery replays log past snapshot", |_, s: &State| {
                s.crashes > 0 && s.mem > s.snapshot && s.mem == s.acked
            }),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stateright::Checker;

    fn correct() -> Persistence {
        Persistence {
            truncate_after_rename: true,
            max_writes: 3,
            max_crashes: 1,
            max_compactions: 2,
        }
    }

    #[test]
    fn snapshot_then_truncate_is_crash_safe() {
        let checker = correct().checker().spawn_bfs().join();
        checker.assert_properties();
    }

    #[test]
    fn truncating_before_rename_loses_acknowledged_writes() {
        let checker = Persistence {
            truncate_after_rename: false,
            ..correct()
        }
        .checker()
        .spawn_bfs()
        .join();
        assert!(
            checker
                .discovery("acknowledged writes are recoverable")
                .is_some()
        );
    }
}
