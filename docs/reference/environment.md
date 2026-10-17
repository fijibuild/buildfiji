# Environment variables

| Variable | Read by | Effect |
|---|---|---|
| `RUST_LOG` | `fjfj-telemetry` | A [`tracing` env filter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html). Default `info`. Logs go to stderr without module targets. An unparseable value falls back to `info`. |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | `fjfj-telemetry` | When set (to any value), enables OTLP export of traces and metrics over gRPC. When unset, only stderr logs are produced and metrics go to a no-op meter. |
| `HOSTNAME` | workspace status | Source of `BUILD_HOST`. If unset, fjfj runs `hostname(1)`. |
| `USER`, then `USERNAME` | workspace status | Source of `BUILD_USER`; `unknown` if neither is set. |

Other `OTEL_*` variables (service name, headers, protocol timeouts) are read
by the OpenTelemetry SDK, which fjfj does not configure beyond the endpoint.
fjfj speaks gRPC only, so an `http/protobuf` protocol setting does not
apply.

## Logs

At the default level, `fjfj build` logs one `build requested` line with every
parsed flag and the target patterns, plus lines for workspace status and
module resolution. These go to stderr. Use `RUST_LOG=warn` to hide them.

Every command runs inside a `run` span. Spans and metrics reach the
exporter when the process exits.
See [Export traces and metrics](../how-to/export-traces-and-metrics.md).
