# fjfj documentation

`fjfj` is a Bazel-compatible build tool written in Rust. These pages follow
the [Diátaxis](https://diataxis.fr) framework: each one answers exactly one
kind of question, so pick the section by what you are trying to do.

|  | **Learning** (study) | **Working** (do) |
|---|---|---|
| **Practical** | [Tutorials](tutorials/) — guided lessons that end in something working | [How-to guides](how-to/) — recipes for a specific job |
| **Theoretical** | [Explanation](explanation/) — why it is built this way | [Reference](reference/) — precise, complete facts to look up |

## Where fjfj is today

fjfj is early. Today it can parse Bazel's command line, resolve a bzlmod
module graph, and report on that graph. It cannot build anything yet:
`fjfj build //...` validates its flags and resolves `MODULE.bazel`, then exits
with `fjfj build is not implemented yet`. [Commands](reference/commands.md)
lists exactly what works. Planned work is tracked with `bd ready`.

## Tutorials

- [Getting started](tutorials/getting-started.md) — build fjfj, run it, and
  explore a module graph.

## How-to guides

- [Inspect a module graph](how-to/inspect-a-module-graph.md)
- [Resolve modules against a local registry or override](how-to/use-a-local-registry-or-override.md)
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
