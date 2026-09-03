//! Engine scheduler with versioned edges, early cutoff, cancellation and
//! cycle detection (bead buildfiji-2h9.1; crate `fjfj-engine`; Lean
//! `Fjfj.Engine`).
//!
//! Graph under test: inputs `X`, `Y`; derived `B(X)` (constant: its value
//! never changes, so dependents get early cutoff), `C(Y)`, `A(B, C)`. With
//! `cyclic` set, `C` also depends on `A`.
//!
//! Every node carries `changed_at` and `verified_at` (the Lean `Node`).
//! Evaluation reads each dependency after that dependency is verified at
//! the current global version, records the dependency's `changed_at`, and
//! on finish either early-cuts (all observations equal the previous ones)
//! or recomputes.
//!
//! `restart_on_version_change` switches the protocol between the correct
//! behaviour (if the global version moved during evaluation, re-verify the
//! stale reads before finishing) and the classic mixed-version bug (finish
//! with reads taken at different versions). The checker finds the bug.
//!
//! Properties:
//! - `no reads from the future`: `observed[d] <= verified_at`.
//! - `verified implies current deps`: a node verified at the current
//!   version observed every dependency's current `changed_at`.
//! - `quiescence is settlement`: when no action is enabled, every demanded
//!   node is verified at the current version or in error (liveness stated
//!   as a safety property over terminal states).
//! - `early cutoff happens` and, when cyclic, `cycle is reported`.

use stateright::{Model, Property};

pub const X: usize = 0;
pub const Y: usize = 1;
pub const B: usize = 2;
pub const C: usize = 3;
pub const A: usize = 4;
pub const N: usize = 5;

type Version = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Status {
    Idle,
    Reading { start: Version },
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Node {
    pub changed_at: Version,
    pub verified_at: Option<Version>,
    pub status: Status,
    /// Observations taken during the current evaluation.
    pub observed: [Option<Version>; N],
    /// Observations recorded by the last completed evaluation.
    pub committed: [Option<Version>; N],
    pub demanded: bool,
    /// Dependency this node is currently waiting for (for cycle detection).
    pub waiting_on: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct State {
    pub version: Version,
    pub nodes: [Node; N],
    pub edits: u8,
    pub cancels: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Edit(usize),
    Start(usize),
    Read(usize, usize),
    Finish(usize),
    Cancel(usize),
}

#[derive(Clone, Copy, Debug)]
pub struct Scheduler {
    pub restart_on_version_change: bool,
    pub cyclic: bool,
    pub max_edits: u8,
    pub max_cancels: u8,
}

impl Scheduler {
    pub fn deps(&self, n: usize) -> &'static [usize] {
        match (n, self.cyclic) {
            (B, _) => &[X],
            (C, false) => &[Y],
            (C, true) => &[Y, A],
            (A, _) => &[B, C],
            _ => &[],
        }
    }

    fn is_input(n: usize) -> bool {
        n == X || n == Y
    }

    /// Constant nodes recompute to the same value; dependents early-cut.
    fn constant(n: usize) -> bool {
        n == B
    }

    /// Does following `waiting_on` from `from` reach `target`?
    fn reaches(s: &State, mut from: usize, target: usize) -> bool {
        for _ in 0..N {
            if from == target {
                return true;
            }
            match s.nodes[from].waiting_on {
                Some(next) => from = next,
                None => return false,
            }
        }
        false
    }
}

impl Model for Scheduler {
    type State = State;
    type Action = Action;

    fn init_states(&self) -> Vec<State> {
        let blank = Node {
            changed_at: 0,
            verified_at: None,
            status: Status::Idle,
            observed: [None; N],
            committed: [None; N],
            demanded: false,
            waiting_on: None,
        };
        let mut nodes = [blank; N];
        for n in [X, Y] {
            nodes[n].verified_at = Some(0);
        }
        nodes[A].demanded = true;
        vec![State {
            version: 0,
            nodes,
            edits: 0,
            cancels: 0,
        }]
    }

    fn actions(&self, s: &State, actions: &mut Vec<Action>) {
        if s.edits < self.max_edits {
            actions.push(Action::Edit(X));
            actions.push(Action::Edit(Y));
        }
        for n in 0..N {
            let node = &s.nodes[n];
            if Self::is_input(n) || !node.demanded {
                continue;
            }
            match node.status {
                Status::Idle if node.verified_at != Some(s.version) => {
                    actions.push(Action::Start(n))
                }
                Status::Reading { .. } => {
                    let pending: Vec<usize> = self
                        .deps(n)
                        .iter()
                        .copied()
                        .filter(|&d| node.observed[d].is_none())
                        .collect();
                    if pending.is_empty() {
                        actions.push(Action::Finish(n));
                    } else {
                        for d in pending {
                            actions.push(Action::Read(n, d));
                        }
                    }
                    if s.cancels < self.max_cancels {
                        actions.push(Action::Cancel(n));
                    }
                }
                _ => {}
            }
        }
    }

    fn next_state(&self, s: &State, a: Action) -> Option<State> {
        let mut t = *s;
        match a {
            Action::Edit(i) => {
                t.version += 1;
                t.edits += 1;
                t.nodes[i].changed_at = t.version;
                // Inputs are always current: an edit re-verifies every input.
                for input in [X, Y] {
                    t.nodes[input].verified_at = Some(t.version);
                }
            }
            Action::Start(n) => {
                t.nodes[n].status = Status::Reading { start: s.version };
                t.nodes[n].observed = [None; N];
            }
            Action::Read(n, d) => {
                let dep = s.nodes[d];
                if dep.status == Status::Error {
                    t.nodes[n].status = Status::Error;
                    t.nodes[n].waiting_on = None;
                } else if dep.verified_at == Some(s.version) {
                    t.nodes[n].observed[d] = Some(dep.changed_at);
                    t.nodes[n].waiting_on = None;
                } else if Self::reaches(s, d, n) {
                    // d is (transitively) waiting on n: dependency cycle.
                    t.nodes[n].status = Status::Error;
                    t.nodes[n].waiting_on = None;
                } else {
                    t.nodes[d].demanded = true;
                    t.nodes[n].waiting_on = Some(d);
                    if t.nodes[d].status == Status::Idle {
                        // Demand is answered by a later Start(d); nothing else.
                    }
                }
            }
            Action::Finish(n) => {
                let Status::Reading { start } = s.nodes[n].status else {
                    return None;
                };
                let is_stale = |d: usize| {
                    s.nodes[d].verified_at != Some(s.version)
                        || s.nodes[n].observed[d] != Some(s.nodes[d].changed_at)
                };
                let stale = self.deps(n).iter().any(|&d| is_stale(d));
                if self.restart_on_version_change && stale {
                    // Global version moved under us: drop stale reads and re-read.
                    for &d in self.deps(n) {
                        if is_stale(d) {
                            t.nodes[n].observed[d] = None;
                        }
                    }
                    t.nodes[n].status = Status::Reading { start: s.version };
                    return Some(t);
                }
                let verified = if self.restart_on_version_change {
                    s.version
                } else {
                    start
                };
                let node = &mut t.nodes[n];
                let unchanged = node.verified_at.is_some() && node.observed == node.committed;
                if !(unchanged || Self::constant(n) && node.verified_at.is_some()) {
                    node.changed_at = verified;
                }
                node.verified_at = Some(verified);
                node.committed = node.observed;
                node.status = Status::Idle;
                node.waiting_on = None;
            }
            Action::Cancel(n) => {
                t.cancels += 1;
                t.nodes[n].status = Status::Idle;
                t.nodes[n].observed = [None; N];
                t.nodes[n].waiting_on = None;
            }
        }
        Some(t)
    }

    fn properties(&self) -> Vec<Property<Self>> {
        let mut props = vec![
            Property::always("no reads from the future", |_, s: &State| {
                s.nodes.iter().all(|n| {
                    n.verified_at
                        .is_none_or(|v| n.committed.iter().flatten().all(|&o| o <= v))
                })
            }),
            Property::always(
                "verified implies current deps",
                |m: &Scheduler, s: &State| {
                    (0..N).all(|n| {
                        let node = &s.nodes[n];
                        node.status == Status::Error
                            || node.verified_at != Some(s.version)
                            || m.deps(n)
                                .iter()
                                .all(|&d| node.committed[d] == Some(s.nodes[d].changed_at))
                    })
                },
            ),
            Property::always("quiescence is settlement", |m: &Scheduler, s: &State| {
                let mut enabled = Vec::new();
                m.actions(s, &mut enabled);
                !enabled.is_empty()
                    || s.nodes.iter().all(|n| {
                        !n.demanded || n.status == Status::Error || n.verified_at == Some(s.version)
                    })
            }),
        ];
        if self.cyclic {
            props.push(Property::sometimes("cycle is reported", |_, s: &State| {
                s.nodes[A].status == Status::Error || s.nodes[C].status == Status::Error
            }));
        } else {
            props.push(Property::sometimes(
                "early cutoff happens",
                |_, s: &State| {
                    let a = &s.nodes[A];
                    s.version > 0 && a.verified_at == Some(s.version) && a.changed_at < s.version
                },
            ));
        }
        props
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stateright::Checker;

    fn correct() -> Scheduler {
        Scheduler {
            restart_on_version_change: true,
            cyclic: false,
            max_edits: 2,
            max_cancels: 1,
        }
    }

    #[test]
    fn diamond_with_edits_and_cancel_is_consistent() {
        let checker = correct().checker().spawn_bfs().join();
        checker.assert_properties();
    }

    #[test]
    fn cycle_is_detected_and_still_settles() {
        let checker = Scheduler {
            cyclic: true,
            ..correct()
        }
        .checker()
        .spawn_bfs()
        .join();
        checker.assert_properties();
    }

    #[test]
    fn finishing_without_reverify_mixes_versions() {
        let checker = Scheduler {
            restart_on_version_change: false,
            ..correct()
        }
        .checker()
        .spawn_bfs()
        .join();
        // A starts at version 0, reads B, then Y is edited, C is recomputed at
        // version 1, A reads the new C and finishes labelled as version 0.
        assert!(checker.discovery("no reads from the future").is_some());
    }
}
