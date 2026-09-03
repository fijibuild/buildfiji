# Incremental engine

Bazel's Skyframe and Buck2's DICE are memoising, demand-driven key/value
graphs with dependency tracking and invalidation. fjfj needs the same to
support a persistent server, `--watchfs`, and fast no-op builds.

Decision (2026-09-03): build a custom engine, crate `fjfj-engine`.

Rejected:
1. Buck2 `dice`: not published on crates.io; API tracks Buck2 needs.
2. `salsa`: single-revision IDE model; poor fit for thousands of concurrent
   actions with cancellation and streaming results.

Requirements for `fjfj-engine`:
- Demand-driven memoised key/value graph; keys are enums (`PackageKey`,
  `ConfiguredTargetKey`, `AspectKey`, `ActionKey`, `FileKey`, `RepoKey`).
- Versioned dependency edges with early cutoff (unchanged value does not
  invalidate dependents).
- Fully async and parallel (tokio); cancellation of in-flight computations.
- Bazel cycle detection and error propagation semantics.
- Designed for persistence across server restarts.

## Daemon (decision 2026-09-03)

fjfj keeps a persistent daemon per `output_base`, like Bazel. Transport is
gRPC over a Unix domain socket (tonic + tokio `UnixListener`); the proto
lives in `fjfj-proto`. A thin client parses argv and rc files, connects or
spawns the daemon, streams the command and relays output and exit code.
The daemon owns the in-memory engine graph, persistent worker pools, warm
remote connections and the file watcher.

Telemetry: the client opens the root span and propagates W3C `traceparent`
in gRPC metadata so client and daemon spans form one trace; the daemon
exports OTLP and also emits daemon-lifecycle metrics.

The spike (buildfiji-23d.1) studies dice and salsa to decide what to borrow.

## Persistence (open; see decision bead)

Compactness comes from the encoding, not the store: intern labels, paths and
mnemonics to u32 ids; store dependency edges as sorted delta+varint lists,
deduplicated by hash; content-address values by digest and deduplicate;
zstd block compression with a trained dictionary.

Leading option: an immutable zero-copy snapshot (rkyv or columnar) that the
daemon mmaps on start, plus an append-only delta log compacted on idle, with
small hot indexes (file digests, action cache) in a pure-Rust compressed KV
store (fjall). RocksDB is a fallback if a single store for everything is
worth its C++ build cost. The CAS keeps Bazel's `--disk_cache` layout so it
can be shared with Bazel.
