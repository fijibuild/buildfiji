# Environment variables

| Variable | Read by | Effect |
|---|---|---|
| `RUST_LOG` | `fjfj-telemetry` | A [`tracing` env filter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html) for fjfj's own logs. Default `warn`. Logs go to stderr without module targets. An unparseable value falls back to `warn`. Bazel's `INFO:` lines are the console's, not the log's, so `RUST_LOG` does not hide them. |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | `fjfj-telemetry` | When set (to any value), enables OTLP export of traces and metrics over gRPC. When unset, metrics go to a no-op meter. |
| `FJFJ_TRACE_FILE` | `fjfj-telemetry` | A path. Writes the process's spans as a Chrome trace (one complete event per line, one row per action). Read it with `fjfj analyze-profile` or Perfetto. |
| `FJFJ_SPAN_TIMES` | `fjfj-telemetry` | When set, prints how long each span took as it closes, to stderr. |
| `FJFJ_TOOLCHAIN_DEBUG` | `fjfj-analysis` | When set, prints each toolchain constraint setting match to stderr during analysis. |
| `XDG_CACHE_HOME`, then `HOME` | output tree | Where the default output user root and repository cache live (`$XDG_CACHE_HOME` or `~/.cache`, then `fjfj/_fjfj_<user>`), unless `--output_user_root` is given. `HOME` is also where the home `.bazelrc` is looked for. |
| `HOSTNAME` | workspace status | Source of `BUILD_HOST`. If unset, fjfj runs `hostname(1)`. |
| `USER`, then `USERNAME` | workspace status | Source of `BUILD_USER`; `unknown` if neither is set. |

Other `OTEL_*` variables (service name, headers, timeouts) are read by the
OpenTelemetry SDK, which fjfj does not configure beyond the endpoint. fjfj speaks
gRPC only, so an `http/protobuf` protocol setting does not apply.

## What the processes inherit

`run` starts the program with fjfj's environment plus
`BUILD_WORKSPACE_DIRECTORY`, `BUILD_WORKING_DIRECTORY`, `BUILD_EXECROOT` and
`BUILD_ID`. Build actions run with the strict action environment (a fixed
`PATH`) plus any `--action_env`.

## Logs

fjfj's `tracing` logs and Bazel-style `INFO:` output are separate. To see
fjfj's own debug logs, set `RUST_LOG`, for example
`RUST_LOG=fjfj_exec=debug fjfj build //...`.

Every command runs inside a `run` span; each action the scheduler runs is an
`action` span with `step` spans beneath it (see
[`design/telemetry.md`](../design/telemetry.md)). Spans and metrics reach the
exporter when the process exits.
