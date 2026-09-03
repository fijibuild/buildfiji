# fjfj architecture

`fjfj` is a Bazel-compatible build tool written in Rust. The bar for
"compatible" is: a repository that builds with `bazel build //...` builds
with `fjfj build //...` using the same `MODULE.bazel`, `BUILD` files, `.bzl`
rules, `.bazelrc` files and command-line flags, and can share a remote cache
and remote executors with Bazel.

## Principles

1. **Command-line and Starlark compatibility over internal fidelity.** We copy
   Bazel's observable behaviour (flags, target patterns, Starlark builtins,
   providers, output layout), not its implementation (Skyframe, Java).
2. **Reuse over rebuild.** Prefer existing crates and protocols:
   - Starlark: the `starlark` crate (Buck2's implementation). If its parser
     proves too slow for very large BUILD trees, the fallback is a custom
     lexer built on [regal](https://github.com/NathanHowell/regal) (build-time
     minimal-DFA, incremental relexing) feeding a hand-written parser that
     produces the same `starlark_syntax` AST.
   - Remote execution / caching: REAPI v2 via the `bazel-remote-apis` crate.
   - Telemetry: `tracing` + OpenTelemetry (OTLP). Bazel's Build Event Protocol
     and `--profile` JSON are *exports derived from the trace*, not separate
     systems.
   - CLI: `clap`, with a Bazel-flag compatibility layer on top.
   - Sandboxing: OS primitives (Linux namespaces, macOS seatbelt) behind one
     trait, with OCI as an optional strategy.
3. **Three phases, one graph.** Loading (Starlark evaluation), analysis
   (rules + aspects produce providers and actions), execution (actions run
   locally or remotely). All three are nodes in one memoised, incremental
   key/value graph so that `fjfj` can be a persistent server like Bazel.

## Crate map

| Crate | Responsibility |
|---|---|
| `fjfj` | Binary entry point only. |
| `fjfj-cli` | Command dispatch, server mode, output formatting. |
| `fjfj-bazel-compat` | Bazel flag parsing, `.bazelrc`, target patterns, unknown-flag policy. |
| `fjfj-starlark` | Starlark evaluation, Bazel builtins (`rule`, `aspect`, `provider`, `native`, `select`, `ctx`). |
| `fjfj-graph` | Labels, packages, targets, configured targets, aspects, actions, digests. Pure data. |
| `fjfj-exec` | Action scheduler: strategy selection, action cache lookup, local vs remote. |
| `fjfj-sandbox` | Local execution strategies (`local`, linux namespaces, darwin seatbelt, OCI). |
| `fjfj-remote` | REAPI client: CAS, action cache, execution, capabilities. Disk cache. |
| `fjfj-telemetry` | `tracing` setup, OTLP export, BEP/profile exporters. |

Planned crates: `fjfj-engine` (incremental memoising graph), `fjfj-bzlmod`
(module resolution and repository rules), `fjfj-query` (query/cquery/aquery).

## Where the hard problems live

See `docs/design/` for one document per hard problem. Each corresponds to an
epic in beads (`bd list --type epic`).
