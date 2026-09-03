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
