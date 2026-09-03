//! Dynamic execution: race a local and a remote branch per action, publish
//! the winner exactly once, cancel the loser (bead buildfiji-2h9.3; crate
//! `fjfj-exec`; docs/design/sandboxing.md "Cancellation").
//!
//! Design rules encoded here:
//! - A branch that fails does not win; the other branch continues. Only if
//!   both fail does the action fail.
//! - Winning is an atomic *claim* (compare-and-swap on a winner slot), taken
//!   before publishing. Publishing is the kill-safe sequence from `publish`.
//! - Cancellation is asynchronous: a cancelled loser may still finish before
//!   the cancel lands. Its outputs are discarded, never published.
//! - Loser scratch directories are cleaned up (no leaked sandboxes).
//!
//! `atomic_claim = false` models the check-then-publish bug (each branch
//! checks "not yet published" and then publishes as two separate steps).
//! The checker finds the double publish.

use stateright::{Model, Property};

pub const LOCAL: usize = 0;
pub const REMOTE: usize = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Branch {
    Idle,
    Running,
    Done { ok: bool },
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct State {
    pub branches: [Branch; 2],
    /// Scratch directory holds outputs for this branch.
    pub scratch: [bool; 2],
    pub cancel_pending: [bool; 2],
    /// Atomic winner slot.
    pub claim: Option<usize>,
    /// Non-atomic variant: "I checked and nothing was published yet".
    pub checked: [bool; 2],
    pub publishes: u8,
    pub failed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Start(usize),
    Finish(usize, bool),
    Claim(usize),
    Check(usize),
    Publish(usize),
    IssueCancel(usize),
    CancelLands(usize),
    Cleanup(usize),
    FailAction,
}

#[derive(Clone, Copy, Debug)]
pub struct Dynamic {
    pub atomic_claim: bool,
}

impl State {
    fn published(&self) -> bool {
        self.publishes > 0
    }
    fn settled(&self) -> bool {
        self.published() || self.failed
    }
}

impl Model for Dynamic {
    type State = State;
    type Action = Action;

    fn init_states(&self) -> Vec<State> {
        vec![State {
            branches: [Branch::Idle; 2],
            scratch: [false; 2],
            cancel_pending: [false; 2],
            claim: None,
            checked: [false; 2],
            publishes: 0,
            failed: false,
        }]
    }

    fn actions(&self, s: &State, actions: &mut Vec<Action>) {
        for b in [LOCAL, REMOTE] {
            let other = 1 - b;
            match s.branches[b] {
                Branch::Idle if !s.settled() => actions.push(Action::Start(b)),
                Branch::Idle => {}
                Branch::Running => {
                    actions.push(Action::Finish(b, true));
                    actions.push(Action::Finish(b, false));
                    if s.cancel_pending[b] {
                        actions.push(Action::CancelLands(b));
                    }
                }
                Branch::Done { ok: true } => {
                    if self.atomic_claim {
                        if s.claim.is_none() {
                            actions.push(Action::Claim(b));
                        } else if s.claim == Some(b) && !s.published() {
                            actions.push(Action::Publish(b));
                        }
                    } else if !s.checked[b] && !s.published() {
                        actions.push(Action::Check(b));
                    } else if s.checked[b] && s.scratch[b] {
                        actions.push(Action::Publish(b));
                    }
                }
                Branch::Done { ok: false } => {}
                Branch::Cancelled => {}
            }
            // The winner (or a successful finisher, in the flawed variant)
            // asks for the other branch to be cancelled.
            let i_won = if self.atomic_claim {
                s.claim == Some(b)
            } else {
                s.checked[b]
            };
            if i_won && s.branches[other] == Branch::Running && !s.cancel_pending[other] {
                actions.push(Action::IssueCancel(other));
            }
            // Loser / failed / cancelled scratch is removed once it can no longer publish.
            let can_publish = match self.atomic_claim {
                true => s.claim == Some(b) && !s.published(),
                false => s.checked[b],
            };
            if s.scratch[b]
                && matches!(s.branches[b], Branch::Done { .. } | Branch::Cancelled)
                && !can_publish
                && s.settled()
            {
                actions.push(Action::Cleanup(b));
            }
        }
        if !s.settled()
            && s.branches
                .iter()
                .all(|&b| matches!(b, Branch::Done { ok: false } | Branch::Cancelled))
            && s.branches.contains(&Branch::Done { ok: false })
        {
            actions.push(Action::FailAction);
        }
    }

    fn next_state(&self, s: &State, a: Action) -> Option<State> {
        let mut t = *s;
        match a {
            Action::Start(b) => t.branches[b] = Branch::Running,
            Action::Finish(b, ok) => {
                t.branches[b] = Branch::Done { ok };
                t.scratch[b] = ok;
                t.cancel_pending[b] = false;
            }
            Action::Claim(b) => t.claim = Some(b),
            Action::Check(b) => t.checked[b] = true,
            Action::Publish(b) => {
                t.publishes += 1;
                t.scratch[b] = false;
            }
            Action::IssueCancel(b) => t.cancel_pending[b] = true,
            Action::CancelLands(b) => {
                t.branches[b] = Branch::Cancelled;
                t.scratch[b] = false;
                t.cancel_pending[b] = false;
            }
            Action::Cleanup(b) => t.scratch[b] = false,
            Action::FailAction => t.failed = true,
        }
        Some(t)
    }

    fn properties(&self) -> Vec<Property<Self>> {
        vec![
            Property::always("publish exactly once", |_, s: &State| s.publishes <= 1),
            Property::always("cancelled branch never publishes", |_, s: &State| {
                // A cancelled branch has no scratch, so it cannot have been the publisher.
                (0..2).all(|b| s.branches[b] != Branch::Cancelled || !s.scratch[b])
            }),
            Property::always("failure does not win", |_, s: &State| {
                !s.failed || s.branches.iter().all(|&b| b != Branch::Done { ok: true })
            }),
            Property::always("quiescence is settlement", |m: &Dynamic, s: &State| {
                let mut enabled = Vec::new();
                m.actions(s, &mut enabled);
                !enabled.is_empty()
                    || (s.settled()
                        && !s.branches.contains(&Branch::Running)
                        && (0..2).all(|b| !s.scratch[b]))
            }),
            Property::sometimes("local wins", |_, s: &State| {
                s.published() && s.claim == Some(LOCAL)
            }),
            Property::sometimes("remote wins", |_, s: &State| {
                s.published() && s.claim == Some(REMOTE)
            }),
            Property::sometimes("loser finishes before cancel lands", |_, s: &State| {
                s.claim.is_some() && s.branches.iter().all(|&b| b == Branch::Done { ok: true })
            }),
            Property::sometimes("both fail", |_, s: &State| s.failed),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stateright::Checker;

    #[test]
    fn atomic_claim_publishes_exactly_once() {
        let checker = Dynamic { atomic_claim: true }.checker().spawn_bfs().join();
        checker.assert_properties();
    }

    #[test]
    fn check_then_publish_double_publishes() {
        let checker = Dynamic {
            atomic_claim: false,
        }
        .checker()
        .spawn_bfs()
        .join();
        // Both branches finish, both check "nothing published yet", both publish.
        assert!(checker.discovery("publish exactly once").is_some());
    }
}
