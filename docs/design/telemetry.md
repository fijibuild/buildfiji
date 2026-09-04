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

## Console UI (decision 2026-09-04)

`--color`, `--curses`, `--show_progress` and `--ui_event_filters` split the
same way as `--workspace_status_command`: `fjfj-bazel-compat::console_flags`
extracts raw values (a `TriState` for the two `YES`/`NO`/AUTO flags), and
`fjfj-bazel-compat::console` (pure, no I/O — no terminal, no clock) turns
them into a `ConsoleConfig` and knows how to render one `ProgressUpdate` —
`[<done> / <total>] <message>`, Bazel's own shape, `total: 0` meaning
"unknown yet". `fjfj-exec::console::ConsoleUi` is the I/O half: it owns the
output stream and the one bit of state rendering needs (was the last write
a progress line), and writes what the pure half computes.

`--color=auto`/`--curses=auto` resolve against a tty-ness `bool` the
*caller* supplies (`std::io::IsTerminal` on the real stream), not something
`ConsoleUi` detects itself — `IsTerminal` is a sealed trait, so no test
double can implement it, and Bazel resolves `auto` per output stream
anyway (stdout might be a tty while stderr is redirected, or vice versa).

The curses invariant that makes overwriting work: a progress line **never**
gets a trailing `\n` — every update is `\r\x1b[K<line>` (return to the
line's start, clear it, write the new text), so the next one lands in the
same place. The line only becomes permanent, and gets its `\n`, when
`ConsoleUi::line` needs to write something else after it. Getting this
backwards (appending `\n` to the *first* progress write, which seemed
natural until a second update needed to land on the same line) was caught
by a test that asserts the exact bytes two successive updates produce, not
just that each one's text is right in isolation.

`--ui_event_filters`'s grammar (leading `+`/`-` adjust Bazel's default set;
a bare name overrides it completely, from that point forward) is
`UiEventFilters::parse`, fed one flag occurrence at a time in argument
order — mixing `+DEBUG` (add) with a later bare `INFO,ERROR` (override) is
legal and does what the flag's own docs say: everything before the bare
entry is discarded once it appears. The default set is every `EventKind`
but `DEBUG` — Bazel's own default is documented the same way, but the
exact enumeration wasn't independently verified against Bazel's source, so
treat it as reasonable rather than exact.

Real fallout discovered wiring this in (buildfiji-gwl.17's own bzlmod
resolution, exercised for the first time by an actual `fjfj build //...`
run rather than a unit test): `reqwest::blocking::Client` (behind
`Registry::remote`) owns and tears down its own inner Tokio runtime, and
dropping it from a thread that's already inside `rt.block_on` — exactly
where `fjfj-cli`'s dispatch runs — panics ("Cannot drop a runtime in a
context where blocking is not allowed"). `fjfj-cli` now runs the whole
`resolve_bzlmod` call, `HttpFetcher` construction through drop, inside
`tokio::task::spawn_blocking`. No unit test caught this — `#[test]` fns
aren't async, so `resolve_bzlmod`'s own tests never ran inside a runtime —
which is the case for dogfooding `fjfj build //...` against this
repository's own `MODULE.bazel` after wiring in real console output.
