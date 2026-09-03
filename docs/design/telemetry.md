# Telemetry, BEP and profiling

Principle: OpenTelemetry is the primary observability model. Every phase,
package load, configured target analysis, action, and RPC is a `tracing`
span with OTel semantic attributes. `OTEL_EXPORTER_OTLP_ENDPOINT` turns on
export with no fjfj-specific config.

Bazel compatibility exports derived from the same span stream:
- `--build_event_json_file` / `--build_event_binary_file` /
  `--bes_backend`: Build Event Protocol, generated from spans and results.
- `--profile` / `--generate_json_trace_profile`: Chrome trace JSON.
- `--execution_log_binary_file` / `--execution_log_compact_file`: spawn log.

Metrics (`opentelemetry` meters): action counts by strategy and cache
status, critical path, CAS bytes up/down, worker pool utilisation.

## Output filtering and warning deduplication (decision 2026-09-03)

`--output_filter`/`--auto_output_filter` decide which rule's warnings and
action output actually reach the terminal (`fjfj_bazel_compat::output_filter`):
an explicit `--output_filter` regex is matched against the full label
text; `--auto_output_filter=packages`/`subpackages` instead compares the
rule's package against the packages named on the command line. Bazel's
own default (`none`) shows everything, so this only prunes output on
request. Separately, `WarningDeduplicator` tracks exact warning text
already shown in this invocation so a message repeated by many actions is
only printed once — a plain `HashSet<String>`, since Bazel's own
deduplication is by exact text, not by warning "kind".

## Compact execution log (decision 2026-09-03)

`--execution_log_compact_file`'s wire format (`ExecLogEntry` from Bazel's
`src/main/protobuf/spawn.proto`, length-delimited and zstd-compressed as one
continuous stream) is hand-transcribed as `prost::Message` structs in
`fjfj_remote::execution_log`, the same way `action_key::CacheSalt` transcribes
`remote_execution_log.proto` — a vendored `.proto` file plus a protoc/prost-build
step isn't worth it for a message set this small and stable. It lives in
`fjfj-remote`, not `fjfj-bazel-compat`, because its payload is spawn/action
data (`Spawn`, `InputSet`, `File`) rather than a flag value, and this crate
already owns the other action-cache-key wire types it shares digest and
platform message shapes with.

Entries reference each other by a caller-assigned id (Bazel requires that an
entry be written only after everything it references, without requiring
increasing id order), so the writer only encodes what it's given — assigning
ids by walking the action graph belongs to whatever produces the entries.
Only the `Invocation`, `File`, `InputSet` and `Spawn` variants of
`ExecLogEntry`'s oneof are transcribed so far; `Directory`,
`UnresolvedSymlink`, `SymlinkAction`, `SymlinkEntrySet` and `RunfilesTree`
exist in Bazel's proto for runfiles-tree reconstruction, which fjfj doesn't
build yet.
