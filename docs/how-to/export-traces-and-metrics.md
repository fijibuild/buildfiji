# Export traces and metrics to a local stack

fjfj emits `tracing` spans and OpenTelemetry metrics. Export is off unless
`OTEL_EXPORTER_OTLP_ENDPOINT` is set. This guide starts a throwaway local
backend and points fjfj at it.

You need Docker (or Apple's `container` with `container-compose`; see
[`tools/observability/README.md`](../../tools/observability/README.md) for
the one-time DNS step that needs).

## 1. Start the stack

```sh
docker compose -f tools/observability/docker-compose.yaml up
```

This runs an OpenTelemetry Collector (receiving OTLP on `:4317` gRPC and
`:4318` HTTP), VictoriaTraces for traces and VictoriaMetrics for metrics.
Nothing is persisted across restarts.

## 2. Point fjfj at it

In another shell:

```sh
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
bazel run //:fjfj -- build //...
```

Any fjfj command works. Today `build` is the most useful one: it gets as
far as resolving `MODULE.bazel` and then reports it is not implemented, but
the span for that run is still exported on exit.

fjfj speaks OTLP over gRPC only, which is why the collector sits in front of
the backends: VictoriaMetrics accepts OTLP over HTTP only.

## 3. Look at the data

- Traces: <http://localhost:10428/select/vmui>. Jaeger's query API is also
  served at `/select/jaeger` if you prefer a Jaeger UI or Grafana.
- Metrics: <http://localhost:8428/vmui> (PromQL/MetricsQL).

The exporter turns dots into underscores, so `fjfj.action.count` is
`fjfj_action_count_total` and `fjfj.build.critical_path` is
`fjfj_build_critical_path_bucket`/`_sum`/`_count`. The full list is in
`crates/fjfj-telemetry/src/metrics.rs`. Nothing records those metrics yet;
they will appear as the execution engine is built.

## Control local log output

Independently of export, fjfj writes logs to **stderr**. Control them with
the standard `RUST_LOG` filter:

```sh
RUST_LOG=warn fjfj build //...      # quiet
RUST_LOG=fjfj_bzlmod=debug fjfj mod graph
```

With no `RUST_LOG`, the level is `info`.

## Stop the stack

```sh
docker compose -f tools/observability/docker-compose.yaml down
```

## If nothing shows up

- Check the collector is listening: `curl -s localhost:4318` should connect.
- An exporter panic reading "there is no reactor running" means telemetry was
  initialised outside the Tokio runtime. That was fixed in
  buildfiji-k62.14; if you see it again it is a regression, so file a bead.
