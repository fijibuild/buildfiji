/-!
# Engine graph (crate `fjfj-engine`, beads buildfiji-23d.3)

A demand-driven memoising key/value graph with versioned dependency edges
and early cutoff. Values are immutable (architecture principle: immutable
optics by default).
-/
namespace Fjfj.Engine

/-- Kinds of graph keys. Mirrors the enum in `fjfj-engine`. -/
inductive KeyKind
  | package
  | configuredTarget
  | aspect
  | action
  | file
  | repo
  deriving DecidableEq, Repr

/-- A version stamp. Monotonically increasing per graph. -/
abbrev Version := Nat

/-- A memoised node: the version at which its value was last *changed*
(`changedAt`) and the version at which it was last *verified* up to date
(`verifiedAt`). Early cutoff is the gap between the two. -/
structure Node where
  kind       : KeyKind
  changedAt  : Version
  verifiedAt : Version
  h          : changedAt ≤ verifiedAt
  deriving Repr

/-- A dependency edge records the dependency's `changedAt` as observed when
the dependent was last computed. -/
structure Edge where
  observedChangedAt : Version

/-- A dependent is up to date at version `v` if it was verified at `v` and
every dependency has not changed since it was observed. -/
def upToDate (n : Node) (deps : List (Node × Edge)) (v : Version) : Prop :=
  n.verifiedAt = v ∧ ∀ d ∈ deps, d.1.changedAt ≤ d.2.observedChangedAt

/-- Early cutoff: if every dependency's value is unchanged since it was
observed, the dependent need not be recomputed, only re-verified.
The theorem records the invariant the engine relies on: re-verification
does not move `changedAt`. -/
theorem reverify_preserves_changedAt (n : Node) (v : Version) (hv : n.verifiedAt ≤ v) :
    ({ n with verifiedAt := v, h := Nat.le_trans n.h hv } : Node).changedAt = n.changedAt := rfl

/-- Invariants checked exhaustively by the Stateright model in
`crates/fjfj-models/src/scheduler.rs`, stated over a completed node and the
versions it observed for its dependencies. -/
structure Observation where
  depChangedAt : Version
  observed     : Version

/-- No reads from the future: a node verified at `v` observed nothing newer
than `v`. Violated by the mixed-version bug (finishing an evaluation with
reads taken at different global versions). -/
def NoFutureReads (n : Node) (obs : List Observation) : Prop :=
  ∀ o ∈ obs, o.observed ≤ n.verifiedAt

/-- Verified implies current: a node verified at the current version `v`
observed each dependency's current `changedAt`. -/
def CurrentDeps (n : Node) (obs : List Observation) (v : Version) : Prop :=
  n.verifiedAt = v → ∀ o ∈ obs, o.observed = o.depChangedAt

/-- A node with no dependencies trivially satisfies both invariants. -/
theorem leaf_consistent (n : Node) (v : Version) :
    NoFutureReads n [] ∧ CurrentDeps n [] v := by
  refine ⟨?_, ?_⟩
  · intro o h; cases h
  · intro _ o h; cases h

end Fjfj.Engine
