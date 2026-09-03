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

/-!
## Snapshot encoding (decided 2026-09-03, bead buildfiji-23d.8)

One format: columnar snapshot + delta log, no separate KV store. Strings are
interned to ids; dependency edge lists are sorted and delta-coded, then
deduplicated by content. The spike (`crates/fjfj-spike-persist`) measured
29 bytes/node for this encoding against 184 (rkyv) and 220+ (redb, fjall).
-/
namespace Fjfj.Persistence.Snapshot

/-- Interning invariant: every string reference is inside the table. -/
def StringRefsValid (tableLen : Nat) (refs : List Nat) : Prop :=
  ∀ r ∈ refs, r < tableLen

/-- `prev ≤ x₁ ≤ x₂ ≤ …`: the precondition for delta coding. Edge lists are
sorted and deduplicated before encoding, so this holds with `prev = 0`. -/
def Ascending : Nat → List Nat → Prop
  | _, [] => True
  | p, x :: xs => p ≤ x ∧ Ascending x xs

/-- Delta-code a sorted list relative to `prev`. -/
def deltas : List Nat → Nat → List Nat
  | [], _ => []
  | x :: xs, prev => (x - prev) :: deltas xs x

/-- Inverse of `deltas`. -/
def undeltas : List Nat → Nat → List Nat
  | [], _ => []
  | d :: ds, prev => (prev + d) :: undeltas ds (prev + d)

/-- Delta coding is lossless on ascending lists. Mirrors
`crates/fjfj-spike-persist/src/varint.rs` (`put_deltas`/`get_deltas`), which
is the Kani target once it moves into `fjfj-engine` (bead buildfiji-2h9.8). -/
theorem undeltas_deltas :
    ∀ (l : List Nat) (prev : Nat), Ascending prev l → undeltas (deltas l prev) prev = l
  | [], _, _ => rfl
  | x :: xs, prev, h => by
    simp only [deltas, undeltas]
    have hx : prev + (x - prev) = x := Nat.add_sub_cancel' h.1
    rw [hx]
    exact congrArg (x :: ·) (undeltas_deltas xs x h.2)

/-- Deltas of an ascending list never need more than the width of the largest
id: each delta is bounded by its element. -/
theorem deltas_le : ∀ (l : List Nat) (prev : Nat), ∀ d ∈ deltas l prev, ∃ x ∈ l, d ≤ x
  | [], _, d, h => nomatch h
  | x :: xs, prev, d, h => by
    simp only [deltas, List.mem_cons] at h
    rcases h with rfl | h
    · exact ⟨x, List.mem_cons_self .., Nat.sub_le x prev⟩
    · obtain ⟨y, hy, hd⟩ := deltas_le xs x d h
      exact ⟨y, List.mem_cons_of_mem x hy, hd⟩

end Fjfj.Persistence.Snapshot
