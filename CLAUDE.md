# Project Instructions for AI Agents

This file provides instructions and context for AI coding agents working on this project.

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:6cd5cc61 -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Every bead gets one model/* and one effort/* label; see .claude/skills/bead-standards/SKILL.md
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

**Architecture in one line:** issues live in a local Dolt DB; sync uses `refs/dolt/data` on your git remote; `.beads/issues.jsonl` is a passive export. See https://github.com/gastownhall/beads/blob/main/docs/SYNC_CONCEPTS.md for details and anti-patterns.

## Agent Context Profiles

The managed Beads block is task-tracking guidance, not permission to override repository, user, or orchestrator instructions.

- **Conservative (default)**: Use `bd` for task tracking. Do not run git commits, git pushes, or Dolt remote sync unless explicitly asked. At handoff, report changed files, validation, and suggested next commands.
- **Minimal**: Keep tool instruction files as pointers to `bd prime`; use the same conservative git policy unless active instructions say otherwise.
- **Team-maintainer**: Only when the repository explicitly opts in, agents may close beads, run quality gates, commit, and push as part of session close. A current "do not commit" or "do not push" instruction still wins.

## Session Completion

This protocol applies when ending a Beads implementation workflow. It is subordinate to explicit user, repository, and orchestrator instructions.

1. **File issues for remaining work** - Create beads for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **Handle git/sync by active profile**:
   ```bash
   # Conservative/minimal/default: report status and proposed commands; wait for approval.
   git status

   # Team-maintainer opt-in only, unless current instructions forbid it:
   git pull --rebase
   git push
   git status
   ```
5. **Hand off** - Summarize changes, validation, issue status, and any blocked sync/commit/push step

**Critical rules:**
- Explicit user or orchestrator instructions override this Beads block.
- Do not commit or push without clear authority from the active profile or the current user request.
- If a required sync or push is blocked, stop and report the exact command and error.
<!-- END BEADS INTEGRATION -->


## Build & Test

```bash
cargo build && cargo test          # Rust workspace (authoritative)
bazel build //... && bazel test //...  # via rules_rust + crate_universe from Cargo.lock
./target/debug/fjfj build //...       # dogfood
```

After changing any Cargo.toml, run `CARGO_BAZEL_REPIN=1 bazel build //...` and update the crate BUILD.bazel deps by hand.


## Architecture Overview

See `docs/ARCHITECTURE.md` (crate map, principles) and `docs/design/*.md` (one doc per hard problem, each backed by a beads epic).

## Conventions & Patterns

- Bazel 9.2.0 observable behaviour is the spec: flags, --incompatible_* defaults, Starlark builtins, output layout, exit codes.
- Reuse crates and protocols (starlark, bazel-remote-apis, tonic, tracing/OpenTelemetry, clap) before writing custom code.
- Telemetry: everything is a `tracing` span; BEP and profiles are exports of the trace.
- Parser fallback if starlark crate is slow: lexer on https://github.com/NathanHowell/regal, target the `starlark_syntax` AST.
- Architecture is recorded in Lean 4 under spec/ (checked by `lake build`); docs/design markdown is the prose companion. Fable-level decisions get a Lean module before closing.
- Model checking in Rust: Stateright for protocol interleavings (crash and kill as actions), Loom for engine internals, Kani for pure codecs. TLA+ only by decision.
- Keep `fjfj-graph` free of I/O.
- Immutable data and optics by default: values are immutable, updates go through lenses/prisms producing new values with structural sharing (persistent collections, Arc). Interior mutability or in-place mutation is allowed only where a profile shows it matters, and the bead must cite the profile.
- No native Starlark modules: cc_common, java_common and friends are written in Starlark (builtins overlay or the rules themselves). Rust only implements the core language and rule/aspect/provider/ctx primitives.
- Edition 2024, latest stable Rust (pinned in rust-toolchain.toml and MODULE.bazel), latest crate versions only.
- Clippy and rustfmt run as aspects on every `bazel build` via .bazelrc; run `cargo fmt --all` before building.
