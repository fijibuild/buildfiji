/-!
# Dynamic execution (crate `fjfj-exec`; bead buildfiji-2h9.3)

Local and remote branches race per action. The Stateright model in
`crates/fjfj-models/src/dynamic.rs` checks the interleavings; this module
records the rules.
-/
namespace Fjfj.Dynamic

inductive Branch
  | idle | running | done (ok : Bool) | cancelled
  deriving DecidableEq, Repr

structure State where
  local_  : Branch
  remote  : Branch
  /-- Atomic winner slot: the claim is taken before publishing. -/
  claim   : Option Bool   -- some true = local, some false = remote
  publishes : Nat
  failed  : Bool
  deriving Repr

/-- Exactly-once: never more than one publish. -/
def PublishOnce (s : State) : Prop := s.publishes ≤ 1

/-- A failed branch does not win: the action fails only when no branch
succeeded. -/
def FailureDoesNotWin (s : State) : Prop :=
  s.failed = true → s.local_ ≠ Branch.done true ∧ s.remote ≠ Branch.done true

/-- Publishing requires the claim (this is what makes `PublishOnce` hold
under any interleaving; the check-then-publish variant violates it). -/
def PublishRequiresClaim (s : State) : Prop :=
  s.publishes > 0 → s.claim.isSome

theorem init_ok :
    PublishOnce { local_ := .idle, remote := .idle, claim := none, publishes := 0, failed := false } ∧
    PublishRequiresClaim { local_ := .idle, remote := .idle, claim := none, publishes := 0, failed := false } := by
  refine ⟨Nat.zero_le 1, ?_⟩
  intro h
  exact absurd h (Nat.lt_irrefl 0)

end Fjfj.Dynamic
