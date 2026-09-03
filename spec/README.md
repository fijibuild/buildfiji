# fjfj architecture spec (Lean 4)

Decision 2026-09-03: architecture is recorded in Lean 4, one module per
crate or protocol, checked with `lake build` in CI. Markdown in
`docs/design/` is the prose companion; when they disagree, the Lean wins.

```sh
cd spec && lake build
```

Rules: no Mathlib; specs at the level of keys, values, edges and
protocols; every fable-level decision bead gets a module here before it
closes.
