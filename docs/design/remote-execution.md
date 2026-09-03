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
