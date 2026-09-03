/-!
# Kill-safe output publishing (crates `fjfj-exec`, `fjfj-remote`;
beads buildfiji-23d.11, buildfiji-2h9.2)

Invariant the Stateright model in `crates/fjfj-models/src/publish.rs`
checks exhaustively: an action-cache entry may exist only when the
published output is complete and durable, and a published output is never
partial. This module fixes the vocabulary and states the invariant.
-/
namespace Fjfj.Publish

inductive File
  | absent
  | truncated
  | complete (durable : Bool)
  deriving DecidableEq, Repr

structure State where
  scratch    : File
  published  : File
  cacheEntry : Bool
  deriving Repr

/-- Safety invariant, as a predicate over states. -/
def Safe (s : State) : Prop :=
  s.published ≠ File.truncated ∧
  (s.cacheEntry → s.published = File.complete true)

/-- The initial state is safe. -/
theorem init_safe : Safe { scratch := .absent, published := .absent, cacheEntry := false } := by
  refine ⟨by decide, ?_⟩
  intro h
  cases h

/-- Rename preserves safety exactly when the scratch file is durable and
complete: this is why fsync must precede rename. -/
theorem rename_safe (s : State) (hs : Safe s) (hc : ¬ s.cacheEntry)
    (hd : s.scratch = File.complete true) :
    Safe { s with published := s.scratch, scratch := .absent } := by
  refine ⟨?_, ?_⟩
  · simp [hd]
  · intro h; exact absurd h hc

end Fjfj.Publish
