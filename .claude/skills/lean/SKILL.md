---
name: lean
description: The Lean 4 architecture spec under spec/ — what belongs in it, when a bead needs a module before it closes, how it relates to docs/design markdown, and how to build it. Use when writing or changing anything in spec/, or when closing a fable-level decision bead.
---

# The Lean 4 spec

Architecture is recorded in Lean 4 under `spec/`, one module per crate or
protocol, checked with `lake build` in CI (Lean 4.33.1, pinned in
`spec/lean-toolchain`; toolchain artifacts are pinned per platform in
`MODULE.bazel`).

```bash
cd spec && lake build     # until Lean rules land in Bazel
```

Existing modules live in `spec/Fjfj/` (`Engine`, `Persistence`, `Daemon`,
`ActionKey`, `Configuration`, `Dynamic`, `Publish`) and are re-exported from
`spec/Fjfj.lean`.

## Rules

- **No Mathlib.** The spec is self-contained; a dependency on Mathlib is a
  decision, not a convenience.
- **Spec at the level of keys, values, edges and protocols** — the shapes and
  invariants of the architecture, not Rust implementation detail. If a change
  to a function body would force a spec edit, the spec is too concrete.
- **Every fable-level decision bead gets a module here before it closes.**
  That is the gate, not a nice-to-have.
- `docs/design/*.md` is the prose companion. **When Lean and markdown
  disagree, the Lean wins** — fix the markdown.

## Working on it

Add the module to `spec/Fjfj.lean`'s imports so `lake build` covers it. Prove
what the design actually claims (an invariant the engine or protocol must
hold); a module of definitions with no theorem states nothing.
