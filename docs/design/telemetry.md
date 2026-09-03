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

## Build metrics (decision 2026-09-03)

`fjfj_telemetry::metrics::BuildMetrics` wraps the four instruments named
above as real `opentelemetry::metrics` handles (`Counter`, `Histogram`,
`Gauge`), created from `fjfj_telemetry::meter()` — the global OTel meter,
mirroring how the tracer is reached, and safe to call whether or not
`init` set up an OTLP meter provider (OTel's own no-op meter otherwise).
`init` now builds a `SdkMeterProvider` alongside the existing
`SdkTracerProvider` when `OTEL_EXPORTER_OTLP_ENDPOINT` is set, exported the
same way (periodic OTLP push), and shuts both down on drop.

Attributes are attached at the call site (`strategy`/`cache_status` on the
action counter, `direction` on CAS bytes) rather than one instrument per
label value, so a backend can slice by any combination without fjfj
enumerating them. Worker utilisation is recorded as a `0.0..=1.0` fraction
(`busy / total`) rather than two raw counts, since the fraction is what a
dashboard actually wants and computing it consistently belongs here rather
than in every caller.

Tested against a real in-process OTel pipeline
(`opentelemetry_sdk::metrics::ManualReader`, not a bespoke assertion helper)
so the test suite confirms actual OTel aggregation behaviour — sums,
histogram buckets, gauge values — not just that fjfj's own code ran.

## BES-facing flags (decision 2026-09-03)

`--build_event_publish_all_actions`, `--bes_results_url`, and
`--bes_timeout` are what an IDE or CI dashboard driving fjfj through BEP
actually reads, ahead of the BEP writer itself existing. `bes_flags`
extracts them the same way every other `*_flags` module does; the one
Bazel-specific behaviour worth a note is `--bes_timeout`'s duration syntax
(`Converters.DurationConverter`, `^([0-9]+)(d|h|m|s|ms|ns)$`, with bare `0`
special-cased to need no unit) — a single number plus one unit, never a
combination like `1h30m`, so `bes_flags::parse_duration` is a small
standalone parser rather than reaching for a duration-parsing crate.

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
