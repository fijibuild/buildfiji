# Profile a build

Find out where a build spent its time without running a collector. fjfj can
write its spans to a Chrome trace file and summarise it.

## 1. Write a trace file

Set `FJFJ_TRACE_FILE` to a path for the run you want to look at:

```sh
FJFJ_TRACE_FILE=/tmp/build.trace fjfj build //...
```

The file has one complete event per line and one row per action. Perfetto
(<https://ui.perfetto.dev>) and `chrome://tracing` open it directly.

fjfj's `--profile` flag does not write this file yet.

## 2. Summarise it

```sh
fjfj analyze-profile /tmp/build.trace
fjfj analyze-profile /tmp/build.trace --top 30
```

It prints:

- time in each step of an action (below);
- how many commands ran at once;
- time by mnemonic;
- the critical path, followed through each action's `blocker`, the dependency
  it waited for last;
- the slowest commands. `--top` sets how many it lists (default 15).

## Reading the steps

Each action is a root `action` span with a `step` span per part of its work:

| Step | What it covers |
|---|---|
| `deps` | waiting for the actions that make its inputs |
| `slot` | waiting for one of `--jobs` slots |
| `key` | hashing the action and checking the action cache |
| `prepare` | the sandbox, the command line, removing stale outputs |
| `spawn` | fork, namespace setup and exec |
| `command` | the program itself |
| `collect` | outputs out of the sandbox, a test's log, the failure text |
| `record` | writing the action cache entry |
| `make` | an action that makes its outputs itself (no program) |

`deps` and `slot` are waits; `command` is the program; everything else is
fjfj's own work. If `slot` dominates, raise `--jobs`; if `command` dominates,
the time is in your tools; if `prepare`, `spawn` or `collect` dominate, it is in
fjfj and worth a bead. A step that runs on the blocking pool includes the wait
for a pool thread.

## Print span times instead

For a quick look without a file, `FJFJ_SPAN_TIMES=1` prints how long each span
took, to stderr, as it closes.

## Background

The span names and the reasoning behind them are in
[`design/telemetry.md`](../design/telemetry.md).
