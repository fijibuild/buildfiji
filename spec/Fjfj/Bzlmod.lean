/-!
# Module resolution (crate `fjfj-bzlmod`; bead buildfiji-mum.6)

Bzlmod turns a `MODULE.bazel` into the set of external repositories a
build uses. This module records the invariants that make that result
well-defined, at the level of keys, versions and edges.

The Bazel 9.2.0 fact this module is built around: **`compatibility_level`
is a no-op**. `ModuleFileGlobals.module` parses the argument, warns, and
then calls `setCompatibilityLevel(0)` unconditionally, and `bazel_dep`'s
`max_compatibility_level` is likewise ignored. So there is exactly one
selection group per module name (absent a `multiple_version_override`),
which is what `one_version_per_name` below turns into a theorem: after
selection, no two modules in the graph share a name. Reintroducing
compatibility levels would break that theorem, and with it the assumption
the resolver's walk is built on.
-/
namespace Fjfj.Bzlmod

/-! ## Versions

Selection uses two properties of the version order and nothing else: it
is linear, and the *empty* version — the sentinel for a module with a
non-registry override — sits at the top. Bazel's real order (relaxed
SemVer, a prerelease below its release, digits below text) is a linear
order on the non-empty versions, so a `Nat` stands in for one here. -/

/-- A version: `some n` is an ordinary version, `none` the empty version. -/
abbrev Version := Option Nat

/-- Bazel's `Version.COMPARATOR`, reduced to its two load-bearing cases. -/
def Version.le : Version → Version → Bool
  | _,      none   => true
  | none,   some _ => false
  | some a, some b => a ≤ b

/-- The empty version compares greater than every version. This is not a
convenience: selection uses it as the "no constraint" sentinel for the
target allowed version of a module with no multiple-version override, and
`snapUp` below relies on it. -/
theorem le_empty (v : Version) : Version.le v none = true := by
  cases v <;> rfl

theorem le_refl (v : Version) : Version.le v v = true := by
  cases v <;> simp [Version.le]

theorem le_total (a b : Version) : Version.le a b = true ∨ Version.le b a = true := by
  cases a with
  | none => cases b <;> simp [Version.le]
  | some m =>
    cases b with
    | none => simp [Version.le]
    | some n =>
      rcases Nat.le_total m n with h | h
      · exact Or.inl (by simpa [Version.le] using h)
      · exact Or.inr (by simpa [Version.le] using h)

theorem le_trans {a b c : Version}
    (h₁ : Version.le a b = true) (h₂ : Version.le b c = true) :
    Version.le a c = true := by
  cases a <;> cases b <;> cases c <;> simp_all [Version.le] <;> omega

/-- The larger of two versions. -/
def Version.max (a b : Version) : Version := if Version.le a b then b else a

theorem max_eq (a b : Version) : Version.max a b = a ∨ Version.max a b = b := by
  unfold Version.max
  by_cases h : Version.le a b = true
  · exact Or.inr (by simp [h])
  · exact Or.inl (by simp [h])

theorem le_max_left (a b : Version) : Version.le a (Version.max a b) = true := by
  unfold Version.max
  by_cases h : Version.le a b = true
  · simp [h]
  · simp [h, le_refl]

theorem le_max_right (a b : Version) : Version.le b (Version.max a b) = true := by
  unfold Version.max
  by_cases h : Version.le a b = true
  · simp [h, le_refl]
  · rcases le_total a b with h' | h'
    · exact absurd h' h
    · simp [h, h']

/-! ## Minimal version selection

Each selection group keeps the highest version anyone asked for. Two
properties matter, and both are what "minimal" means in MVS: the result
is never below a request (no silent downgrade), and it is always one of
the requested versions (no invented version, so the registry is always
asked for something someone named). -/

/-- The version a selection group settles on, folding the requests in
declaration order. There is no starting value: an empty group does not
exist, since a group is created by a module being in the graph. -/
def selectedFrom (acc : Version) : List Version → Version
  | []      => acc
  | v :: vs => selectedFrom (Version.max acc v) vs

theorem le_selectedFrom (vs : List Version) (acc : Version) :
    Version.le acc (selectedFrom acc vs) = true := by
  induction vs generalizing acc with
  | nil => simpa [selectedFrom] using le_refl acc
  | cons w ws ih =>
    have := ih (Version.max acc w)
    simpa [selectedFrom] using le_trans (le_max_left acc w) this

/-- Selection never downgrades: every requested version is at or below
the selected one. -/
theorem selectedFrom_ge (vs : List Version) (acc : Version) :
    ∀ v ∈ vs, Version.le v (selectedFrom acc vs) = true := by
  induction vs generalizing acc with
  | nil => intro v hv; cases hv
  | cons w ws ih =>
    intro v hv
    rcases List.mem_cons.mp hv with h | h
    · subst h
      have := le_selectedFrom ws (Version.max acc v)
      simpa [selectedFrom] using le_trans (le_max_right acc v) this
    · simpa [selectedFrom] using ih (Version.max acc w) v h

/-- Selection never invents a version: the winner is either the version
the group started from or one of the later requests, so a registry is
only ever asked for a version some module file named. -/
theorem selectedFrom_mem (vs : List Version) (acc : Version) :
    selectedFrom acc vs = acc ∨ selectedFrom acc vs ∈ vs := by
  induction vs generalizing acc with
  | nil => exact Or.inl rfl
  | cons w ws ih =>
    show selectedFrom (Version.max acc w) ws = acc
      ∨ selectedFrom (Version.max acc w) ws ∈ w :: ws
    rcases ih (Version.max acc w) with h | h
    · rcases max_eq acc w with hm | hm
      · exact Or.inl (by rw [h, hm])
      · exact Or.inr (by rw [h, hm]; exact List.mem_cons_self ..)
    · exact Or.inr (List.mem_cons_of_mem w h)

/-! ## Multiple-version overrides

A `multiple_version_override` splits one module name into several
selection groups, one per allowed version, and every module "snaps up" to
the lowest allowed version that is not below it. The empty version closes
the set as a sentinel meaning "no allowed version is high enough", which
is the condition the resolver reports as an error — but only if the
module is still reachable. -/

/-- The lowest allowed version that is not below `v`, or `none` when
there is none. The allowed list is sorted ascending, as Bazel's
`ImmutableSortedSet.ceiling` requires. -/
def snapUp (v : Version) : List Version → Version
  | []      => none
  | a :: as => if Version.le v a then a else snapUp v as

/-- Snapping never moves a module down. -/
theorem snapUp_ge (v : Version) (allowed : List Version) :
    Version.le v (snapUp v allowed) = true := by
  induction allowed with
  | nil => simpa [snapUp] using le_empty v
  | cons a as ih =>
    unfold snapUp
    by_cases h : Version.le v a = true
    · simp [h]
    · simpa [h] using ih

/-! ## The resolved graph

Selection rewrites every dependency edge to the version its group chose,
then keeps only what is reachable from the root. Both invariants the rest
of the build depends on follow from that construction. -/

/-- A node in the module graph. -/
structure Key where
  name : String
  version : Version
deriving DecidableEq, Repr

/-- The dependency graph before selection: each key's requested deps. -/
structure Graph where
  root : Key
  deps : Key → List Key

/-- Rewriting one edge to the selected version. With no
multiple-version override, the choice depends only on the module's name —
this is the formal content of "one selection group per name". -/
def resolveKey (chosen : String → Version) (k : Key) : Key :=
  { name := k.name, version := chosen k.name }

/-- What survives selection: the root, plus whatever its rewritten edges
reach. -/
inductive Reachable (g : Graph) (chosen : String → Version) : Key → Prop where
  | root : Reachable g chosen g.root
  | dep {k d} : Reachable g chosen k → d ∈ g.deps k →
      Reachable g chosen (resolveKey chosen d)

/-- The resolved graph is closed: following an edge out of a module that
survived selection lands on a module that also survived. Nothing dangles,
so a repo name in the graph always resolves. -/
theorem reachable_closed (g : Graph) (chosen : String → Version) {k d : Key}
    (hk : Reachable g chosen k) (hd : d ∈ g.deps k) :
    Reachable g chosen (resolveKey chosen d) :=
  Reachable.dep hk hd

/-- Every surviving module carries its group's selected version. The root
is assumed to already be its own resolution, which holds because the root
is never rewritten — it has no version to select. -/
theorem reachable_is_resolved (g : Graph) (chosen : String → Version)
    (hroot : g.root = resolveKey chosen g.root) :
    ∀ k, Reachable g chosen k → k = resolveKey chosen k := by
  intro k hk
  induction hk with
  | root => exact hroot
  | dep _ _ _ => rfl

/-- **One version per module name.** Two modules in the resolved graph
with the same name are the same module.

This is what makes an apparent repo name unambiguous, and it is exactly
the invariant that a `multiple_version_override` opts out of — which is
why the resolver skips its duplicate-name check for overridden modules.
It holds because `resolveKey` reads the version from the name alone;
restoring compatibility levels, which would key the choice on
`(name, level)` instead, would break it. -/
theorem one_version_per_name (g : Graph) (chosen : String → Version)
    (hroot : g.root = resolveKey chosen g.root) {a b : Key}
    (ha : Reachable g chosen a) (hb : Reachable g chosen b)
    (hname : a.name = b.name) : a = b := by
  have ha' := reachable_is_resolved g chosen hroot a ha
  have hb' := reachable_is_resolved g chosen hroot b hb
  rw [ha', hb']
  simp [resolveKey, hname]

end Fjfj.Bzlmod
