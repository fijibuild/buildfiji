# Observability: traces first

Bazel has several overlapping ways to learn what a build did: the Build
Event Protocol, the JSON trace profile, the execution log, and a pile of
console output. Each has its own format, producer and consumers. fjfj takes
a different starting point and this page explains it. The decisions are
recorded in [`design/telemetry.md`](../design/telemetry.md).

## One source of truth

Every phase, package load, analysis step, action and RPC is a `tracing`
span, carrying OpenTelemetry attributes. That span stream is the primary
data model. Setting the standard `OTEL_EXPORTER_OTLP_ENDPOINT` variable
exports it, with no fjfj-specific configuration, to any OTLP backend.

Bazel's formats are *derived* from that stream rather than being parallel
systems:

- The Build Event Protocol (`--build_event_json_file`, `--bes_backend`) is
  generated from spans and results.
- The JSON trace profile (`--profile`) is Chrome trace JSON built from the
  same spans.
- The execution log is a record of spawns.

The reason is consistency. If three outputs each had their own bookkeeping,
they would eventually disagree about what happened. If they are all views of
one trace, they cannot.

This is partly a design direction. Today fjfj emits a `run` span per command
and, for builds, an `action` span per action with `step` spans beneath it
(waiting for dependencies, waiting for a job slot, hashing, preparing the
sandbox, spawning, the command itself, collecting outputs). The spans can be
exported over OTLP or written to a Chrome trace file with `FJFJ_TRACE_FILE`
and summarised with `fjfj analyze-profile`. The compact execution log is
written. The Build Event Protocol and `--profile` exporters do not exist yet,
and their flags are refused.

## Metrics are recorded as metrics

Action counts by strategy and cache status, critical path, bytes moved
through the CAS, and worker utilisation are OpenTelemetry instruments, not
fields on a fjfj stats struct. Attributes such as `strategy` and
`cache_status` are attached where the event happens, rather than creating an
instrument per combination, so a backend can slice by any of them without
fjfj enumerating the combinations. Strategy names match Bazel's (`local`,
`worker`, `remote`) so dashboards built for Bazel still make sense.

Worker utilisation is recorded as a fraction between 0 and 1 because that is
what a dashboard wants, and computing it once is better than every consumer
doing it differently.

The instruments are tested against a real OpenTelemetry pipeline, but nothing
in the build calls them yet, so the metrics backend stays empty. The action
spans are what carry timing today.

## Why a collector sits in front of the backends

fjfj's exporter speaks OTLP over gRPC to a single endpoint. Not every
backend does: VictoriaMetrics ingests OTLP over HTTP only. Rather than
teach fjfj two endpoints and two protocols, the local stack in
`tools/observability/` puts an OpenTelemetry Collector between them. The
collector takes the one gRPC stream and splits traces and metrics to the
right places. fjfj stays configured by one variable.

## Console output is separate

The terminal UI (`--color`, `--curses`, `--show_progress`,
`--ui_event_filters`) and Bazel's `INFO:` build summary are not derived from
traces; fjfj's own `tracing` logs default to `warn` so they do not mix with
them. It is a deliberately small
rendering layer: a pure half that turns settings and progress updates into
text, and an I/O half that owns the stream. Keeping the pure half free of
terminals and clocks makes it testable byte for byte, which matters because
a curses-style progress line works by never ending in a newline. The next
write overwrites it.

## Related

- Using it: [Export traces and metrics to a local
  stack](../how-to/export-traces-and-metrics.md).
- Variables: [Environment variables](../reference/environment.md).
