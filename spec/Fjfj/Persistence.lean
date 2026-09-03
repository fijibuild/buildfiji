/-!
# Engine persistence: snapshot + delta log (crate `fjfj-engine`; beads
buildfiji-23d.8, buildfiji-23d.10, buildfiji-2h9.5)

Store-independent crash-consistency contract, checked by the Stateright
model in `crates/fjfj-models/src/compaction.rs`.
-/
namespace Fjfj.Persistence

abbrev Version := Nat

/-- Durable on-disk state. Log entries exist for versions in
`(logStart, logDurable]`. -/
structure Durable where
  snapshot   : Version
  logStart   : Version
  logDurable : Version
  deriving Repr

/-- The version recovery can reconstruct from durable state alone. -/
def recoverable (d : Durable) : Version :=
  if d.logStart ≤ d.snapshot ∧ d.snapshot < d.logDurable then d.logDurable else d.snapshot

/-- Crash consistency: every acknowledged write is recoverable, in every
state, not only after a crash. -/
def CrashConsistent (d : Durable) (acked : Version) : Prop :=
  acked ≤ recoverable d

/-- The log must stay contiguous with the snapshot. Truncating the log
before the new snapshot is durable breaks this and loses writes. -/
def Contiguous (d : Durable) : Prop := d.logStart ≤ d.snapshot

/-- Recovery never goes below the snapshot. -/
theorem snapshot_le_recoverable (d : Durable) : d.snapshot ≤ recoverable d := by
  unfold recoverable
  split
  · next h => exact Nat.le_of_lt h.2
  · exact Nat.le_refl _

end Fjfj.Persistence
