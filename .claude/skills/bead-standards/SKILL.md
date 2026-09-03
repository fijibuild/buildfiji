---
name: bead-standards
description: Standards for creating and labelling beads in this repo. Use whenever creating, triaging, or claiming a bead. Every bead carries exactly one model/* label and one effort/* label so an orchestrator can route it to the right agent at the right reasoning budget.
---

# Bead standards

Every bead (epic, task, spike, decision, milestone) MUST carry:

- exactly one `model/{haiku,sonnet,opus,fable}` label
- exactly one `effort/{low,medium,high,xhigh,max}` label

Apply them at creation time:

```bash
bd create "Title" -t task -p 1 -l model/sonnet,effort/medium -d "..."
bd label add <id> model/opus && bd label add <id> effort/high   # one label per call
bd list -l model/haiku                        # find cheap work
bd lint                                        # run before session close
```

Labels describe the work needed to *complete* the bead, not the work of writing it.
For an epic, label with the model/effort of the hardest child; children carry their own labels.
When a bead is re-scoped, re-label it. When in doubt, go one step *up* on model and one step *down* on effort: a stronger model with a tight budget beats a weak model spinning.

## model/* — which agent should do the work

| Label | Select when | Do NOT select when |
|---|---|---|
| `model/haiku` | Mechanical, fully specified, single-file or scripted work: renames, adding a flag to a table, updating BUILD deps from Cargo.toml, running a fixture and pasting results, formatting, dependency bumps. The bead description is a complete recipe. | Any judgement call, any Bazel-semantics question, anything touching more than ~3 files, anything without an obvious test. |
| `model/sonnet` | Well-scoped implementation with a clear spec and existing patterns to copy: implementing a documented Bazel flag, a new `ctx.actions.*` method mirroring an existing one, a query output format, unit tests for existing behaviour, CI wiring, docs from code. | Design is undecided, semantics must be reverse-engineered from Bazel source, cross-crate refactors, concurrency or unsafe code. |
| `model/opus` | Design-heavy or semantics-heavy work: reverse-engineering Bazel behaviour, cross-crate architecture, the incremental engine, sandboxing and namespaces, REAPI wire compatibility, transitions/toolchain resolution, anything with `unsafe`, performance work, spikes that must produce a recommendation. | The task is a recipe (waste) or needs the very highest reasoning (see fable). |
| `model/fable` | Load-bearing decisions and the hardest problems: `decision` beads that lock in architecture (engine choice, output-tree virtualisation, WORKSPACE scope), correctness-critical parity work (action-digest parity with Bazel), novel algorithms (aspect-on-aspect propagation, cycle detection with error semantics), security-sensitive sandbox design, and reviews of opus output on those topics. | Routine implementation; anything a smaller model can do with a spec. Reserve for beads where a wrong answer is expensive to unwind. |

## effort/* — how much reasoning budget to allow

Effort is about the *thinking* required, not the amount of typing. A 2,000-line generated flag table is `effort/low`; a 40-line namespace setup can be `effort/xhigh`.

| Label | Select when | Typical shape |
|---|---|---|
| `effort/low` | The answer is known; execution is the whole task. Verification is a single command. | Rename, bump, add entry, run script, fix typo, apply reviewer nit. |
| `effort/medium` | One clear approach; some local reasoning about edge cases; tests exist or are obvious. | Implement one flag/builtin/output format; write tests for existing code; small bugfix with repro. |
| `effort/high` | Multiple viable approaches or non-obvious semantics; must read Bazel docs/source or other crates; touches several modules; needs new tests designed. | New subsystem piece (disk cache, worker protocol), bzlmod resolver step, aspect propagation, cquery. |
| `effort/xhigh` | Open design space; must build a prototype or benchmark to decide; correctness depends on subtle invariants; failure is costly. | Spikes with a recommendation, incremental engine keys and invalidation, sandbox isolation, build-without-the-bytes, action-key parity investigation. |
| `effort/max` | Foundational and hard to reverse; expect iteration, adversarial self-review, and written rationale. Budget is unbounded because getting it wrong costs weeks. | Engine library decision, output-tree virtualisation design, aspect-on-aspect semantics, cross-tool cache-key compatibility contract. |

## Pairing guidance

Common valid pairs: `haiku+low`, `sonnet+low/medium`, `opus+medium/high/xhigh`, `fable+high/xhigh/max`.
Suspicious pairs worth a second look: `haiku+high` or above (under-modelled), `fable+low` (over-modelled; prefer opus unless the task is a review of a fable-level decision).

## Description quality

A bead is ready to route only if its description states: the Bazel behaviour being matched (flag, builtin, doc link), the crate(s) it lands in, how it will be verified (test, conformance fixture, benchmark number), and what is explicitly out of scope. Spikes must name the question and the form of the answer. Decisions must list options considered.
