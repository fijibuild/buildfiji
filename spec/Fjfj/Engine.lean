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

end Fjfj.Engine
