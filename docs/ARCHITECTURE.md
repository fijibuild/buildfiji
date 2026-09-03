# fjfj architecture

`fjfj` is a Bazel-compatible build tool written in Rust. The bar for
"compatible" is: a repository that builds with `bazel build //...` builds
with `fjfj build //...` using the same `MODULE.bazel`, `BUILD` files, `.bzl`
rules, `.bazelrc` files and command-line flags, and can share a remote cache
and remote executors with Bazel.

## Principles

1. **Bazel 9.2.0 is the compatibility target** (decided 2026-09-03). Flags, `--incompatible_*` defaults and Starlark builtins follow that release.
2. **Command-line and Starlark compatibility over internal fidelity.** We copy
   Bazel's observable behaviour (flags, target patterns, Starlark builtins,
   providers, output layout), not its implementation (Skyframe, Java).
3. **Reuse over rebuild.** Prefer existing crates and protocols:
   - Starlark: the `starlark` crate (Buck2's implementation), front end
     included; the custom-lexer fallback was measured and rejected in
     docs/design/starlark-and-loading.md.
   - Remote execution / caching: REAPI v2 via the `bazel-remote-apis` crate.
   - Telemetry: `tracing` + OpenTelemetry (OTLP). Bazel's Build Event Protocol
     and `--profile` JSON are *exports derived from the trace*, not separate
     systems.
   - CLI: `clap`, with a Bazel-flag compatibility layer on top.
   - Sandboxing: OS primitives (Linux namespaces, macOS seatbelt) behind one
     trait, with OCI as an optional strategy.
4. **Three phases, one graph.** Loading (Starlark evaluation), analysis
   (rules + aspects produce providers and actions), execution (actions run
   locally or remotely). All three are nodes in one memoised, incremental
   key/value graph so that `fjfj` can be a persistent server like Bazel.

## Crate map

| Crate | Responsibility |
|---|---|
| `fjfj` | Binary entry point only. |
| `fjfj-cli` | Command dispatch, server mode, output formatting. |
| `fjfj-bazel-compat` | Bazel flag parsing, `.bazelrc`, target patterns, unknown-flag policy. |
| `fjfj-bzlmod` | `MODULE.bazel` evaluation, module discovery, Minimal Version Selection, registry (BCR) client. |
| `fjfj-starlark` | Starlark evaluation, Bazel builtins (`rule`, `aspect`, `provider`, `native`, `select`, `ctx`). |
| `fjfj-graph` | Labels, packages, targets, configured targets, aspects, actions, digests. Pure data. |
| `fjfj-exec` | Action scheduler: strategy selection, action cache lookup, local vs remote. |
| `fjfj-sandbox` | Local execution strategies (`local`, linux namespaces, darwin seatbelt, OCI). |
| `fjfj-remote` | REAPI client: CAS, action cache, execution, capabilities. Disk cache. |
| `fjfj-telemetry` | `tracing` setup, OTLP export, BEP/profile exporters. |
| `fjfj-models` | Stateright models of protocols (publish, scheduler, daemon, compaction); not shipped. |
| `fjfj-proto` | Command service proto (`RunCommand` streaming, `Cancel`, `Ping`, `Shutdown`, `Info`); prost/tonic codegen under both Cargo (`build.rs`) and Bazel (`rust_prost_library`) from one `.proto` source. |

Planned crates: `fjfj-engine` (incremental memoising graph), `fjfj-daemon`
(gRPC over UDS client/server, consumes `fjfj-proto`), `fjfj-query`
(query/cquery/aquery).

## Where the hard problems live

See `docs/design/` for one document per hard problem. Each corresponds to an
epic in beads (`bd list --type epic`).

## Data modelling: immutable optics by default

All engine and graph values are immutable. Updates are expressed with optics
(lenses, prisms, traversals) that return new values sharing structure with
the old ones, backed by persistent collections and `Arc`. This keeps
memoisation, cancellation and persistence simple: a node's value never
changes under a reader, snapshots are free, and equality for early cutoff is
structural.

In-place mutation, `RefCell`/`Mutex` interior mutability, or arena tricks are
allowed only where a profile shows the immutable version is a bottleneck.
The bead that introduces such code must link the profile.
