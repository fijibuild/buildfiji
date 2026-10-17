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

This is a design direction, not a description of today. At present fjfj
emits a `run` span per command and writes the execution-log header; the BEP
and profile exporters do not exist yet.

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

The instruments exist and are tested against a real OpenTelemetry pipeline,
but nothing records to them yet: there is no execution engine producing
actions to count.

## Why a collector sits in front of the backends

fjfj's exporter speaks OTLP over gRPC to a single endpoint. Not every
backend does: VictoriaMetrics ingests OTLP over HTTP only. Rather than
teach fjfj two endpoints and two protocols, the local stack in
`tools/observability/` puts an OpenTelemetry Collector between them. The
collector takes the one gRPC stream and splits traces and metrics to the
right places. fjfj stays configured by one variable.

## Console output is separate

The terminal UI (`--color`, `--curses`, `--show_progress`,
`--ui_event_filters`) is not derived from traces. It is a deliberately small
rendering layer: a pure half that turns settings and progress updates into
text, and an I/O half that owns the stream. Keeping the pure half free of
terminals and clocks makes it testable byte for byte, which matters because
a curses-style progress line works by never ending in a newline. The next
write overwrites it.

## Related

- Using it: [Export traces and metrics to a local
  stack](../how-to/export-traces-and-metrics.md).
- Variables: [Environment variables](../reference/environment.md).
