---
name: rust
description: Rust conventions for this repo — toolchain and crate policy, how Cargo and Bazel divide responsibilities (including the repin step after any Cargo.toml change), formatting and lint gates, immutable data and optics, tracing spans, and which model checker to reach for. Use whenever writing, reviewing or building Rust in crates/.
---

# Rust in buildfiji

## Toolchain and crates

Edition 2024, latest stable Rust, pinned in `rust-toolchain.toml` and
`MODULE.bazel` (keep the two in step). Latest crate versions only — no
pinning to an older major to avoid a migration.

Reuse before rebuilding: `starlark`, `bazel-remote-apis`, `tonic`, `prost`,
`tracing`/OpenTelemetry, `clap`, `chumsky`. A hand-rolled equivalent of a
maintained crate needs a reason in the bead.

## Cargo versus Bazel

Bazel and `fjfj` are the build runners:

```bash
bazel build //... && bazel test //...   # the gate
bazel run //:fjfj -- build //...        # dogfood
```

Cargo exists only to maintain `Cargo.toml`/`Cargo.lock` (`cargo update`,
`cargo fmt --all`) and to run Kani. `cargo test` is not a gate; a green
`cargo test` proves nothing about the build.

After changing **any** `Cargo.toml`:

```bash
CARGO_BAZEL_REPIN=1 bazel build //...
```

then update that crate's `BUILD.bazel` `deps` by hand — the dep lists are not
generated, and a missing entry only shows up as a Bazel compile error.
External crates are `@crates//:<name>`, internal ones `//crates/<crate>`.

## Formatting and lints

Clippy and rustfmt run as `rules_rust` aspects on every `bazel build` (wired
in `.bazelrc`), never per-target. Warnings are errors, unused imports
included. Run `cargo fmt --all` before building or the rustfmt aspect fails
the build with a diff.

## Data and mutation

Immutable data and optics by default: values are immutable, updates go
through lenses/prisms that produce new values with structural sharing
(persistent collections, `Arc`). Interior mutability or in-place mutation is
allowed only where a profile shows it matters, and the bead must cite that
profile.

## Telemetry

Everything is a `tracing` span. BEP and profile output are exports of the
trace, not separate logging paths — so add spans rather than log lines, and
name them so the export reads well.

## Model checking

- **Stateright** for protocol interleavings, with crash and kill as actions
  (`crates/fjfj-models`).
- **Loom** for engine internals — concurrency inside one process.
- **Kani** for pure codecs: `cargo kani -p <crate>` (cargo is acceptable here
  until Bazel wiring lands). Keep harnesses on pure, allocation-free
  functions with `#[kani::unwind(N)]` matching the input bound; `BTreeMap`
  and `String` code does not terminate under CBMC.
- TLA+ only by explicit decision.

Invariants that the checkers enforce are recorded in Lean under `spec/` (see
the `lean` skill).
