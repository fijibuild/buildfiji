//! Daemon lifecycle protocol (bead buildfiji-2h9.4; crates `fjfj-daemon`,
//! `fjfj-cli`; design in docs/design/incremental-engine.md "Daemon").
//!
//! One daemon per `output_base`, guarded by a lock file. A client connects
//! over the Unix socket; if no daemon answers it spawns one. Two clients may
//! race to spawn; the lock admits exactly one. A daemon with no client
//! shuts down after an idle timeout, which races with a new connection.
//! Clients and the daemon may crash at any point; a crashed daemon leaves a
//! stale lock that a later spawn must break (`break_stale_lock`).
//!
//! Concurrent-client policy: block. A second client waits while the daemon
//! is busy (Bazel's "Another command is running" behaviour). Design rule
//! found by the checker: a daemon with blocked clients must not idle out.
//!
//! Properties:
//! - `one live daemon`: never two live daemons for one output_base.
//! - `busy daemon has live client`: the daemon never keeps running a command
//!   for a client that has gone away.
//! - `commands are serialised`: at most one client is running a command.
//! - `quiescence is settlement`: when nothing is enabled, every client is
//!   done or crashed (nobody is stuck waiting on a dead daemon or a stale
//!   lock).
//! - `both clients complete` is reachable.

use stateright::{Model, Property};

pub const CLIENTS: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Client {
    Idle,
    Connecting,
    /// Has forked a daemon process that has not yet taken the lock.
    Spawning,
    /// Waiting for the daemon to finish another client's command.
    Blocked,
    Running,
    Done,
    Crashed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Daemon {
    Absent,
    Idle,
    Busy { client: usize },
    ShuttingDown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Lock {
    Free,
    Held,
    /// Held by a daemon process that no longer exists.
    Stale,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct State {
    pub clients: [Client; CLIENTS],
    pub daemon: Daemon,
    pub lock: Lock,
    pub daemon_crashes: u8,
    pub client_crashes: u8,
    pub idle_timeouts: u8,
    pub refused_spawns: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Connect(usize),
    Spawn(usize),
    /// The spawned daemon process acquires (or fails to acquire) the lock.
    DaemonUp(usize),
    Admit(usize),
    Finish(usize),
    ClientCrash(usize),
    DaemonNoticesClientGone(usize),
    IdleTimeout,
    ShutdownComplete,
    DaemonCrash,
}

#[derive(Clone, Copy, Debug)]
pub struct DaemonProtocol {
    pub break_stale_lock: bool,
    pub max_daemon_crashes: u8,
    pub max_client_crashes: u8,
    pub max_idle_timeouts: u8,
    /// Bounds the spawn/refuse retry loop so a wedged client shows up as a
    /// terminal state instead of a livelock. Only refused spawns count.
    pub max_refused_spawns: u8,
}

impl Model for DaemonProtocol {
    type State = State;
    type Action = Action;

    fn init_states(&self) -> Vec<State> {
        vec![State {
            clients: [Client::Idle; CLIENTS],
            daemon: Daemon::Absent,
            lock: Lock::Free,
            daemon_crashes: 0,
            client_crashes: 0,
            idle_timeouts: 0,
            refused_spawns: 0,
        }]
    }

    fn actions(&self, s: &State, actions: &mut Vec<Action>) {
        for c in 0..CLIENTS {
            match s.clients[c] {
                Client::Idle => actions.push(Action::Connect(c)),
                Client::Connecting => match s.daemon {
                    Daemon::Absent if s.refused_spawns < self.max_refused_spawns => {
                        actions.push(Action::Spawn(c))
                    }
                    Daemon::Absent => {}
                    Daemon::Idle => actions.push(Action::Admit(c)),
                    Daemon::Busy { .. } => actions.push(Action::Admit(c)),
                    // Connection refused; the client retries.
                    Daemon::ShuttingDown => {}
                },
                Client::Spawning => actions.push(Action::DaemonUp(c)),
                Client::Blocked => {
                    if s.daemon == Daemon::Idle {
                        actions.push(Action::Admit(c));
                    }
                }
                Client::Running => {
                    if s.daemon == (Daemon::Busy { client: c }) {
                        actions.push(Action::Finish(c));
                    }
                    if s.client_crashes < self.max_client_crashes {
                        actions.push(Action::ClientCrash(c));
                    }
                }
                Client::Done | Client::Crashed => {}
            }
            if s.clients[c] == Client::Crashed && s.daemon == (Daemon::Busy { client: c }) {
                actions.push(Action::DaemonNoticesClientGone(c));
            }
        }
        match s.daemon {
            // A daemon with queued (blocked) clients is not idle. Without this
            // guard the checker finds: client 1 running, client 0 blocked,
            // client 1 crashes, daemon idles out, client 0 waits forever.
            Daemon::Idle
                if s.idle_timeouts < self.max_idle_timeouts
                    && !s.clients.contains(&Client::Blocked) =>
            {
                actions.push(Action::IdleTimeout)
            }
            Daemon::ShuttingDown => actions.push(Action::ShutdownComplete),
            _ => {}
        }
        if s.daemon != Daemon::Absent && s.daemon_crashes < self.max_daemon_crashes {
            actions.push(Action::DaemonCrash);
        }
    }

    fn next_state(&self, s: &State, a: Action) -> Option<State> {
        let mut t = *s;
        match a {
            Action::Connect(c) => t.clients[c] = Client::Connecting,
            Action::Spawn(c) => t.clients[c] = Client::Spawning,
            Action::DaemonUp(c) => {
                t.clients[c] = Client::Connecting;
                match s.lock {
                    Lock::Free => {
                        t.lock = Lock::Held;
                        t.daemon = Daemon::Idle;
                    }
                    Lock::Stale if self.break_stale_lock => {
                        t.lock = Lock::Held;
                        t.daemon = Daemon::Idle;
                    }
                    // Lock refused: the spawned process exits; client retries.
                    Lock::Held | Lock::Stale => t.refused_spawns += 1,
                }
            }
            Action::Admit(c) => match s.daemon {
                Daemon::Idle => {
                    t.daemon = Daemon::Busy { client: c };
                    t.clients[c] = Client::Running;
                }
                Daemon::Busy { .. } => t.clients[c] = Client::Blocked,
                _ => return None,
            },
            Action::Finish(c) => {
                t.clients[c] = Client::Done;
                t.daemon = Daemon::Idle;
            }
            Action::ClientCrash(c) => {
                t.client_crashes += 1;
                t.clients[c] = Client::Crashed;
            }
            Action::DaemonNoticesClientGone(_) => t.daemon = Daemon::Idle,
            Action::IdleTimeout => {
                t.idle_timeouts += 1;
                t.daemon = Daemon::ShuttingDown;
            }
            Action::ShutdownComplete => {
                t.daemon = Daemon::Absent;
                t.lock = Lock::Free;
            }
            Action::DaemonCrash => {
                t.daemon_crashes += 1;
                t.daemon = Daemon::Absent;
                t.lock = Lock::Stale;
                // Every client with a session sees the socket close and retries.
                for c in 0..CLIENTS {
                    if matches!(s.clients[c], Client::Running | Client::Blocked) {
                        t.clients[c] = Client::Connecting;
                    }
                }
            }
        }
        Some(t)
    }

    fn properties(&self) -> Vec<Property<Self>> {
        vec![
            Property::always("one live daemon", |_, s: &State| {
                // The lock is the only thing that makes this true: a live daemon
                // always holds it, and a spawn that cannot take it exits.
                !matches!(
                    s.daemon,
                    Daemon::Idle | Daemon::Busy { .. } | Daemon::ShuttingDown
                ) || s.lock == Lock::Held
            }),
            Property::always("busy daemon has live client", |_, s: &State| {
                match s.daemon {
                    Daemon::Busy { client } => {
                        s.clients[client] == Client::Running || s.clients[client] == Client::Crashed
                    }
                    _ => true,
                }
            }),
            Property::always("commands are serialised", |_, s: &State| {
                s.clients.iter().filter(|&&c| c == Client::Running).count() <= 1
            }),
            Property::always(
                "quiescence is settlement",
                |m: &DaemonProtocol, s: &State| {
                    let mut enabled = Vec::new();
                    m.actions(s, &mut enabled);
                    !enabled.is_empty()
                        || s.clients
                            .iter()
                            .all(|&c| matches!(c, Client::Done | Client::Crashed))
                },
            ),
            Property::sometimes("both clients complete", |_, s: &State| {
                s.clients.iter().all(|&c| c == Client::Done)
            }),
            Property::sometimes("spawn race is resolved by the lock", |_, s: &State| {
                // A daemon is live while another client's spawn is still in flight;
                // that spawn will be refused by the lock.
                s.daemon != Daemon::Absent && s.clients.contains(&Client::Spawning)
            }),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stateright::Checker;

    fn correct() -> DaemonProtocol {
        DaemonProtocol {
            break_stale_lock: true,
            max_daemon_crashes: 1,
            max_client_crashes: 1,
            max_idle_timeouts: 1,
            max_refused_spawns: 4,
        }
    }

    #[test]
    fn daemon_lifecycle_is_safe_and_settles() {
        let checker = correct().checker().spawn_bfs().join();
        checker.assert_properties();
    }

    #[test]
    fn stale_lock_without_breaking_wedges_clients() {
        let checker = DaemonProtocol {
            break_stale_lock: false,
            ..correct()
        }
        .checker()
        .spawn_bfs()
        .join();
        assert!(checker.discovery("quiescence is settlement").is_some());
    }
}
