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

Rules from the model checker (`crates/fjfj-models/src/daemon.rs`): a live
daemon always holds the output_base lock and a spawn that cannot take it
exits; a crashed daemon's stale lock is broken by the next spawn; a daemon
with blocked (queued) clients is not idle and must not time out.

Telemetry: the client opens the root span and propagates W3C `traceparent`
in gRPC metadata so client and daemon spans form one trace; the daemon
exports OTLP and also emits daemon-lifecycle metrics.

The spike (buildfiji-23d.1) studies dice and salsa to decide what to borrow.

## Persistence (decided 2026-09-03)

Decision: a single format, snapshot + append-only delta log, and no separate
KV store. Because every configuration stays resident in memory, disk is a
serialization of the in-memory graph, not a store that is queried during a
build. The action cache and file digests are engine nodes in the same
snapshot and log. The CAS keeps Bazel's `--disk_cache` layout so it can be
shared with Bazel.

Encoding: hand-rolled columnar sections, each one zstd frame, decoded eagerly
on daemon start:
- one string table (labels, paths, mnemonics, argv words), referenced by
  `u32` id everywhere else;
- dependency edge lists as sorted delta+varint byte strings, deduplicated by
  content and referenced by index;
- per-node columns (kind, key ids, value ids, digest, edge-list ref).

Rejected: rkyv zero-copy (raw is 6x larger; with zstd it loses zero-copy and
is still 33% larger than columnar for a 2x faster scan-only load, which
eager materialisation erases); redb and fjall (7 to 8x larger, 3x and 18x
slower to load; per-row storage defeats cross-node compression); RocksDB
(not built: the KV layout loses regardless of store, so its C++ build under
Bazel was never worth measuring).

Spike (`crates/fjfj-spike-persist` at commit 8a6ed31, removed afterwards; bead buildfiji-23d.9) on this repo's own
graph from `bazel aquery`/`cquery` jsonproto dumps, Bazel 9.2.0, arm64,
`-c opt`, best of 5. 37,685 nodes (15,034 configured targets, 2,003
actions, 4,501 depsets, 16,147 files), 98,239 edges, 40,225 strings:

| candidate | bytes | bytes/node | write | cold load |
|---|---:|---:|---:|---:|
| columnar raw (uncompressed) | 4,420,718 | 117.3 | 9.6ms | 7.9ms |
| columnar + zstd, eager decode | 1,095,514 | 29.1 | 9.6ms | 7.9ms |
| rkyv raw, validated access + scan | 6,925,120 | 183.8 | 4.0ms | 1.4ms |
| rkyv raw, materialised | 6,925,120 | 183.8 | 4.0ms | 7.8ms |
| rkyv + zstd, validated access + scan | 1,459,999 | 38.7 | 15.0ms | 4.2ms |
| redb (B-tree, no compression) | 8,425,472 | 223.6 | 102.9ms | 25.4ms |
| fjall (LSM, lz4) | 8,345,648 | 221.5 | 253.3ms | 143.6ms |

Section sizes (raw/zstd): strings 3.2MB/317KB, edges 126KB/58KB (6,948
unique lists), nodes 1.1MB/720KB. Budget derived from this: 30 bytes/node
and 250ns/node cold load, so a 10M-node graph is about 300MB and 2.5s
single-threaded (sections decode in parallel). The nodes section is the next
target: action argv lists dominate it and should be deduplicated by content
like edge lists.

Budget confirmed (buildfiji-23d.19) on two other graph shapes and a
synthetic 1M-node graph, same machine/method as above, `crates/fjfj-spike-persist`
restored from commit 8a6ed31 (`fixtures/generate.sh` reproduces the first
two dumps; `--replicate` produces the third):

| graph | nodes | edges | columnar+zstd bytes/node | cold load |
|---|---:|---:|---:|---:|
| this repo (37,685 nodes, above) | 37,685 | 98,239 | 29.1 | 7.9ms |
| rules_go `examples/hello` (flat depsets, few actions) | 6,862 | 10,251 | 27.4 | 8.9ms |
| rules_python `examples/pip_parse` (runfiles-heavy) | 24,885 | 44,834 | 25.5 | 42.6ms |
| synthetic, 41x replication of pip_parse | 1,020,285 | 1,838,194 | 2.0 | 80.2ms |

Both new shapes land in the same 25-30 bytes/node band as the original
measurement — flat depsets and runfiles trees don't break the format.
Cold load holds well inside the 250ns/node budget throughout (80.2ms /
1,020,285 nodes ≈ 79ns/node); redb and fjall were skipped on the 1M-node run
(`--skip-kv`) since they were already 7-8x larger and much slower to write
and are not going to close that gap by writing more of the same shape.

Caveat on the 1M-node number: it's built by replicating pip_parse's graph
verbatim (`synth::replicate`), sharing one string table across copies, so
zstd compresses the repeated node/edge bytes far better than a real 1M-node
graph with a million unique labels and argv lists would. Treat 2.0
bytes/node as an artifact of the replication method, not a projection for a
real graph at that size — the load-time-per-node figure is the number this
run is actually good evidence for, since decode work scales with distinct
sections, not with how compressible their content happens to be.

Lean: `spec/Fjfj/Persistence.lean` (crash contract and delta-coding
roundtrip). Later, profile-driven: lazy per-section loading, zstd dictionary
for small delta-log entries, LRU eviction.

Crash-consistency rules (model-checked, `crates/fjfj-models/src/compaction.rs`):
fsync the temp snapshot, rename it over the old one, and only then truncate
the log up to the snapshot version. Truncating first loses acknowledged
writes on a crash in between. Acknowledge writes only after the log fsync.
The invariant "recoverable version >= acknowledged version" must hold in
every state, and recovery replays durable log entries past the snapshot.

## Phases and configurations (decided 2026-09-03)

- Analysis and execution interleave from day one (Skymeld). There is no
  phase barrier: an action is demanded as soon as its configured target is
  analysed. Conformance diffs against Bazel compare outputs, not phase
  timing.
- Configured target keys carry the configuration hash, and nodes from every
  configuration stay resident in memory and on disk. Changing build options
  never discards the analysis cache; switching `-c opt` and back is a no-op
  build. Memory grows with the number of configurations used; an LRU
  eviction policy is a later optimisation, not part of the key design.
  Lean: `spec/Fjfj/Configuration.lean`.
