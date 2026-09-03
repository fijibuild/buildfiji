# Local observability stack

One command to get somewhere to point `fjfj`'s OTLP export
(`crates/fjfj-telemetry`) at, without needing a real backend
(buildfiji-k62.14). Brings up an OTel Collector, VictoriaTraces
(traces), and VictoriaMetrics (metrics).

## Run it

```bash
docker compose -f tools/observability/docker-compose.yaml up
```

Then, in another shell:

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
bazel run //:fjfj -- build //...
```

- Traces: <http://localhost:10428/select/vmui> (VictoriaTraces' own UI);
  Jaeger's query API is also served at `/select/jaeger` for pointing an
  external Jaeger UI or Grafana at.
- Metrics: <http://localhost:8428/vmui> (VictoriaMetrics' UI, PromQL/
  MetricsQL); the OTel exporter renames dots to underscores, so
  `fjfj.action.count` shows up as `fjfj_action_count_total`,
  `fjfj.build.critical_path` as
  `fjfj_build_critical_path_bucket`/`_sum`/`_count`,
  `fjfj.remote.cas.bytes` as `fjfj_remote_cas_bytes_total`, and
  `fjfj.worker.utilization` as `fjfj_worker_utilization` — see
  `crates/fjfj-telemetry/src/metrics.rs` for the full set and their
  attributes.

`docker compose -f tools/observability/docker-compose.yaml down` to stop
it; nothing here persists data across restarts (no named volumes — this
is a look-at-the-last-build tool, not a long-lived backend).

## Layout

- `docker-compose.yaml` — the three services.
- `otel-collector-config.yaml` — receives OTLP on `:4317` (gRPC,
  fjfj-telemetry's only protocol) / `:4318` (HTTP), pushes traces to
  VictoriaTraces over OTLP/gRPC and metrics to VictoriaMetrics over
  OTLP/HTTP.

## Why a collector in front, rather than pointing `fjfj` straight at them

`fjfj-telemetry` sends both signals to one gRPC `OTEL_EXPORTER_OTLP_ENDPOINT`.
VictoriaTraces accepts OTLP/gRPC directly, but VictoriaMetrics' native
OTLP ingestion is HTTP-only (`/opentelemetry/v1/metrics`, protobuf) — no
gRPC receiver — so metrics need a protocol hop somewhere. The collector
does that hop and the traces/metrics split in one place, so `fjfj` only
ever needs the one endpoint. Both backends take a direct push straight
off that hop; nothing here polls or scrapes.
