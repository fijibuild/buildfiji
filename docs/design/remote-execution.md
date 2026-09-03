# Remote execution and caching

## What Bazel provides
`--remote_cache`, `--remote_executor`, `--remote_instance_name`,
`--remote_header`, `--remote_download_minimal|toplevel|all`,
`--remote_default_exec_properties`, platform `exec_properties`,
`--disk_cache`, `--remote_upload_local_results`, `--remote_local_fallback`,
`--experimental_remote_cache_compression`, `--remote_grpc_log`.

## Compatibility bar
1. **Wire**: REAPI v2 (`build.bazel.remote.execution.v2`) via gRPC, with the
   `bazel-remote-apis` crate. Talk to bazel-remote, Buildbarn, BuildBuddy,
   EngFlow, NativeLink, RBE.
2. **Cache-key compatibility with Bazel** (stretch goal): identical action
   digests for the same action means a Bazel CI build warms the cache for
   fjfj users and vice versa. This requires bit-for-bit identical `Command`,
   `Platform`, input `Directory` trees, environment, and output path lists.
   Start with a conformance test that runs the same target under both tools
   against one bazel-remote and compares action keys.
3. **Build-without-the-bytes**: `--remote_download_minimal` needs a
   lazy output tree (a virtual filesystem or on-demand materialisation);
   design this into `fjfj-exec` from day one rather than retrofitting.

## Parity findings (verified 2026-09-03 against Bazel 9.2.0 CAS blobs)

`fjfj-remote::action_key` reproduces Bazel's `Command` and `Action` bytes
and the logged action digest for a genrule fixture
(`crates/fjfj-remote/testdata/genrule`). Facts that were not obvious from
the REAPI spec:

- Bazel sets `is_executable = true` on every input `FileNode`, regardless
  of filesystem mode.
- `Action.salt` is a serialised `CacheSalt { may_be_executed_remotely }`
  message, never empty.
- The deprecated `Command.platform` is still populated and repeated in
  `Action.platform`; `output_paths` is used, not `output_files`.
- Environment variables, platform properties and output paths are sorted;
  directory entries are sorted by name.

Still to verify with more fixtures: tree artifacts, symlink inputs,
runfiles trees, `exec_properties` from platforms, tool inputs, timeouts,
and `--experimental_remote_cache_key_workspace`-style salt fields.

## Design sketch
- `fjfj-graph::Action` is shaped like REAPI `Command` + input Merkle tree.
- `fjfj-remote` exposes `ContentAddressableStore`, `ActionCache`, `Executor`
  traits; `DiskCache` and `GrpcRemote` implement them; a `Tiered` wrapper
  composes disk + remote.
- Retry/backoff, `--remote_timeout`, and `--remote_max_connections` come
  from `tonic` + `tower` middleware, not custom code.
- Every REAPI call is a `tracing` span with `rpc.*` OTel semantic attributes.

## Open questions
- Output tree virtualisation strategy (FUSE? NFS? lazy copy on access?).
- Tree artifacts and symlink handling parity with Bazel.
- Remote persistent workers (Bazel has none; do we want them?).

## Shared disk cache rules (model-checked, `crates/fjfj-models/src/disk_cache.rs`)

Bazel and fjfj may share one `--disk_cache`. Blobs and action-cache
entries are written temp + fsync + rename. An action-cache hit counts only
if every referenced CAS blob is present at use time; a missing blob is a
miss and re-executes. Garbage collection removes action-cache entries
before the blobs they reference. Trusting an action-cache entry without
checking the CAS serves a missing blob under concurrent GC; the checker
finds it.
