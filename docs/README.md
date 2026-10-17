# fjfj documentation

`fjfj` is a Bazel-compatible build tool written in Rust. These pages follow
the [Diátaxis](https://diataxis.fr) framework: each one answers exactly one
kind of question, so pick the section by what you are trying to do.

|  | **Learning** (study) | **Working** (do) |
|---|---|---|
| **Practical** | [Tutorials](tutorials/) — guided lessons that end in something working | [How-to guides](how-to/) — recipes for a specific job |
| **Theoretical** | [Explanation](explanation/) — why it is built this way | [Reference](reference/) — precise, complete facts to look up |

## Where fjfj is today

fjfj builds. `build`, `test`, `run`, `query`, `cquery`, `aquery`, `fetch`, `mod`,
`clean` and `info` work against Bazel 9.2.0's behaviour, on Linux. What is still
missing is substantial: no server process (every invocation starts cold),
no remote cache or execution, no macOS sandbox, no build event or `--profile`
output, and `shutdown`, `print_action` and `dump` are stubs. Any Bazel flag
fjfj has not implemented is refused rather than ignored.
[Commands](reference/commands.md) and [Flags](reference/flags.md) list exactly
what works. Planned work is tracked with `bd ready`.

These pages were written against the source and tests of `main`; the
tutorial's output has not been re-run on a Linux build of every revision, so
treat the shape of the sample output, not the exact numbers, as the contract.

## Tutorials

- [Getting started](tutorials/getting-started.md) — build fjfj, run it, and
  explore a module graph.

## How-to guides

- [Inspect a module graph](how-to/inspect-a-module-graph.md)
- [Resolve modules against a local registry or override](how-to/use-a-local-registry-or-override.md)
- [Profile a build](how-to/profile-a-build.md)
- [Export traces and metrics to a local stack](how-to/export-traces-and-metrics.md)
- [Check how fjfj will read a set of Bazel flags](how-to/check-bazel-flags.md)
- [Wire up a Bazel flag](how-to/wire-up-a-bazel-flag.md) (contributors)

## Reference

- [Commands](reference/commands.md)
- [Flags](reference/flags.md)
- [Exit codes and message prefixes](reference/exit-codes.md)
- [Environment variables](reference/environment.md)

## Explanation

- [The compatibility model](explanation/compatibility-model.md)
- [How a build runs](explanation/how-a-build-runs.md)
- [How bzlmod resolution fits in](explanation/bzlmod-resolution.md)
- [Observability: traces first](explanation/observability.md)

## Related material

These are not part of the Diátaxis set; they are the project's working records.

- [`ARCHITECTURE.md`](ARCHITECTURE.md) — crate map and principles.
- [`design/`](design/) — one decision record per hard problem, each backed by
  a beads epic. The design docs are the source of truth for *why*; the
  explanation pages here summarise and link to them rather than copy them.
- [`../CONTRIBUTING.md`](../CONTRIBUTING.md) — build gate and PR process.
- [`../spec/`](../spec/) — the Lean 4 architecture spec. Where it disagrees
  with prose, the Lean wins.
