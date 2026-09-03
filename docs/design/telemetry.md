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
