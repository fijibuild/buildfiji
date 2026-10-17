# Exit codes and message prefixes

fjfj reproduces Bazel's process exit codes and stderr prefixes, because CI
scripts check `$?` and grep for `ERROR:`. The numbering is Bazel's, from
<https://bazel.build/run/scripts#exit-codes>, and is defined in
`crates/fjfj-bazel-compat/src/exit_code.rs`.

## Exit codes

| Code | Name | Meaning | Produced today |
|---|---|---|---|
| 0 | Success | Command completed. | Yes |
| 1 | Build failed | The command ran but did not succeed. Also used for commands that are not implemented yet. | Yes |
| 2 | Command line problem | Bad or unknown flag, unparseable target pattern, bad flag value, missing `MODULE.bazel`. | Yes |
| 3 | Tests failed | Build succeeded, some tests failed or timed out. | No |
| 4 | No tests found | `test` found no tests. | No |
| 7 | Partial analysis failure | `query` succeeded only partially. | No |
| 8 | Interrupted | Orderly shutdown after Ctrl-C. | No |
| 9 | Lock held, no block | Server lock held with `--noblock_for_lock`. | No |
| 32 | External environmental issue | An external service is unavailable. | No |
| 33 | Out of memory | fjfj ran out of memory. | No |
| 34 | Remote error | Remote execution, cache or build event service error. | No |
| 36 | Local environmental issue | Suspected permanent local problem, such as a missing tool. | No |
| 37 | Internal error | A bug in fjfj. | Yes |
| 48 | External dependency error | A repository failed to fetch. | No |

Flag-syntax errors detected by the argument parser itself exit `2` before
fjfj starts its runtime.

## Message prefixes

| Prefix | Used for |
|---|---|
| `ERROR: ` | A failure the user can act on. Written to stderr. |
| `FATAL: ` | An internal error (exit `37`). Written to stderr. |

Free-form progress text (`[1] Computing workspace status`) is not part of
the stable surface; do not parse it.

## Streams

| Stream | Content |
|---|---|
| stdout | Command output (`version`, `mod`, ...), progress lines, `Resolved n bzlmod module(s)`. |
| stderr | `ERROR:` and `FATAL:` lines, and `tracing` logs (see [Environment variables](environment.md)). |
