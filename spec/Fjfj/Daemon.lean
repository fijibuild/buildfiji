/-!
# Daemon lifecycle (crates `fjfj-daemon`, `fjfj-cli`; beads buildfiji-23d.4,
buildfiji-2h9.4)

One daemon per `output_base`, guarded by a lock. The Stateright model in
`crates/fjfj-models/src/daemon.rs` checks the interleavings; this module
fixes the vocabulary and the two invariants the lock is for.
-/
namespace Fjfj.Daemon

inductive Client
  | idle | connecting | spawning | blocked | running | done | crashed
  deriving DecidableEq, Repr

inductive Daemon
  | absent | idle | busy (client : Nat) | shuttingDown
  deriving DecidableEq, Repr

inductive Lock
  | free | held | stale
  deriving DecidableEq, Repr

structure State where
  clients : List Client
  daemon  : Daemon
  lock    : Lock
  deriving Repr

def Daemon.live : Daemon → Bool
  | .absent => false
  | _ => true

/-- A live daemon always holds the lock; this is what makes "one daemon per
output_base" true, because a spawn that cannot take the lock exits. -/
def LiveHoldsLock (s : State) : Prop :=
  s.daemon.live = true → s.lock = Lock.held

/-- Commands are serialised: at most one client is running. -/
def Serialised (s : State) : Prop :=
  (s.clients.filter (· == Client.running)).length ≤ 1

/-- Design rule found by the model checker: a daemon with blocked clients
is not idle and must not time out. -/
def MayIdleOut (s : State) : Prop :=
  s.daemon = Daemon.idle ∧ Client.blocked ∉ s.clients

theorem absent_trivially_holds (cs : List Client) (l : Lock) :
    LiveHoldsLock { clients := cs, daemon := .absent, lock := l } := by
  intro h; cases h

end Fjfj.Daemon
