//! Kill-safe output publishing (bead buildfiji-2h9.2; crates `fjfj-exec`,
//! `fjfj-remote`).
//!
//! Protocol: an action writes its output into a per-action scratch file,
//! fsyncs it, renames it into `bazel-out`, and only then records the
//! action-cache entry. A SIGKILL (or power loss) may land between any two
//! steps. Non-durable data is lost on kill. Recovery deletes scratch and
//! restarts unless the output is already durably published.
//!
//! Properties:
//! - `never partial output`: a published output is always complete.
//! - `cache implies output`: an action-cache entry exists only when the
//!   published output is complete and durable.
//! - `publishes`: the protocol can complete (sanity).
//!
//! The model is parameterised by `fsync_before_rename`. With it `false`,
//! the checker finds the classic bug: rename, kill before the data hits
//! disk, and `bazel-out` contains a truncated file.

use stateright::{Model, Property};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum File {
    Absent,
    /// Bytes written but not all of them, or written but not durable and
    /// then lost. Reads may observe garbage.
    Partial,
    Complete {
        durable: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Step {
    Writing,
    Written,
    Synced,
    Renamed,
    Cached,
    /// Process was killed; the recovery path runs next.
    Recovering,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct State {
    pub step: Step,
    pub scratch: File,
    pub published: File,
    pub cache_entry: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    WriteScratch,
    Fsync,
    Rename,
    WriteCache,
    Kill,
    Recover,
}

#[derive(Clone, Copy, Debug)]
pub struct Publish {
    pub fsync_before_rename: bool,
}

impl Publish {
    fn lose_non_durable(f: File) -> File {
        match f {
            File::Complete { durable: false } => File::Partial,
            other => other,
        }
    }
}

impl Model for Publish {
    type State = State;
    type Action = Action;

    fn init_states(&self) -> Vec<State> {
        vec![State {
            step: Step::Writing,
            scratch: File::Absent,
            published: File::Absent,
            cache_entry: false,
        }]
    }

    fn actions(&self, s: &State, actions: &mut Vec<Action>) {
        match s.step {
            Step::Writing => actions.push(Action::WriteScratch),
            Step::Written => {
                if self.fsync_before_rename {
                    actions.push(Action::Fsync)
                } else {
                    actions.push(Action::Rename)
                }
            }
            Step::Synced => actions.push(Action::Rename),
            Step::Renamed => actions.push(Action::WriteCache),
            Step::Cached => {}
            Step::Recovering => actions.push(Action::Recover),
        }
        if s.step != Step::Recovering {
            actions.push(Action::Kill);
        }
    }

    fn next_state(&self, s: &State, a: Action) -> Option<State> {
        let mut n = *s;
        match a {
            Action::WriteScratch => {
                n.scratch = File::Complete { durable: false };
                n.step = Step::Written;
            }
            Action::Fsync => {
                n.scratch = File::Complete { durable: true };
                n.step = Step::Synced;
            }
            Action::Rename => {
                // rename(2) is atomic: the name flips to the scratch inode as-is.
                n.published = n.scratch;
                n.scratch = File::Absent;
                n.step = Step::Renamed;
            }
            Action::WriteCache => {
                n.cache_entry = true;
                n.step = Step::Cached;
            }
            Action::Kill => {
                n.scratch = Self::lose_non_durable(n.scratch);
                n.published = Self::lose_non_durable(n.published);
                n.step = Step::Recovering;
            }
            Action::Recover => {
                n.scratch = File::Absent;
                n.step = match n.published {
                    File::Complete { durable: true } if n.cache_entry => Step::Cached,
                    File::Complete { durable: true } => Step::Renamed,
                    _ => {
                        n.published = File::Absent;
                        Step::Writing
                    }
                };
            }
        }
        Some(n)
    }

    fn properties(&self) -> Vec<Property<Self>> {
        vec![
            Property::always("never partial output", |_, s: &State| {
                s.published != File::Partial
            }),
            Property::always("cache implies output", |_, s: &State| {
                !s.cache_entry || s.published == File::Complete { durable: true }
            }),
            Property::sometimes("publishes", |_, s: &State| s.step == Step::Cached),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stateright::Checker;

    #[test]
    fn fsync_before_rename_is_kill_safe() {
        let checker = Publish {
            fsync_before_rename: true,
        }
        .checker()
        .spawn_bfs()
        .join();
        checker.assert_properties();
    }

    #[test]
    fn rename_without_fsync_publishes_partial_output() {
        let checker = Publish {
            fsync_before_rename: false,
        }
        .checker()
        .spawn_bfs()
        .join();
        // The checker must find the truncated-output counterexample.
        checker.assert_discovery(
            "never partial output",
            vec![Action::WriteScratch, Action::Rename, Action::Kill],
        );
    }
}
