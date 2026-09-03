/-!
# Engine keying and phase interleaving (crate `fjfj-engine`; beads
buildfiji-23d.12, buildfiji-23d.14)

Decisions 2026-09-03:
- Analysis and execution interleave from day one (Skymeld): there is no
  phase barrier in the engine; an action key may be demanded as soon as its
  configured target key is evaluated.
- Configured targets are keyed by label *and* configuration hash, and
  nodes from every configuration stay resident (in memory and on disk).
  Changing build options never discards the analysis cache.
-/
namespace Fjfj.Configuration

/-- A configuration is identified by the hash of its build options. -/
abbrev ConfigHash := Nat

structure Label where
  repo    : String
  package : String
  name    : String
  deriving DecidableEq, Repr

/-- The configured target key carries the configuration hash. Two builds
with different options never share a key, so nothing has to be discarded
when options change. -/
structure ConfiguredTargetKey where
  label  : Label
  config : ConfigHash
  deriving DecidableEq, Repr

/-- Keys for the same label under different configurations are distinct. -/
theorem distinct_configs_distinct_keys (l : Label) (a b : ConfigHash) (h : a ≠ b) :
    ({ label := l, config := a } : ConfiguredTargetKey) ≠ { label := l, config := b } := by
  intro heq
  exact h (congrArg ConfiguredTargetKey.config heq)

/-- Engine key kinds, with the ordering constraint for Skymeld: an action
key depends on its configured target key, never on "all analysis done". -/
inductive Key
  | package (pkg : String)
  | configuredTarget (k : ConfiguredTargetKey)
  | action (owner : ConfiguredTargetKey) (index : Nat)
  deriving DecidableEq, Repr

/-- Direct dependencies an action key may have: only its owner. There is
no phase-barrier key for it to wait on. -/
def actionDeps : Key → List Key
  | .action owner _ => [.configuredTarget owner]
  | _ => []

theorem action_depends_only_on_owner (o : ConfiguredTargetKey) (i : Nat) :
    actionDeps (.action o i) = [.configuredTarget o] := rfl

end Fjfj.Configuration
