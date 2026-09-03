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
- Every work bead gets one model/* and one effort/* label; epics and milestones get neither; see .claude/skills/bead-standards/SKILL.md
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
bazel build //... && bazel test //...   # the gate; clippy + rustfmt aspects run on every build
bazel run //:fjfj -- build //...        # dogfood
cd spec && lake build                    # Lean spec, until Lean rules land
```

Bazel and fjfj are the build runners; cargo only maintains Cargo.toml/Cargo.lock and runs rustfmt. Details are in the `bazel`, `rust` and `lean` skills.


## Architecture Overview

See `docs/ARCHITECTURE.md` (crate map, principles) and `docs/design/*.md` (one doc per hard problem, each backed by a beads epic).

## Conventions & Patterns

Project-wide:

- Bazel 9.2.0 observable behaviour is the spec: flags, --incompatible_* defaults, Starlark builtins, output layout, exit codes. When in doubt, run Bazel and match what it does.
- Reuse existing crates and protocols before writing custom code; a hand-rolled replacement needs a reason in the bead.
- Architecture is recorded in Lean 4 under spec/; docs/design markdown is the prose companion, and the Lean wins when they disagree.
- Keep `fjfj-graph` free of I/O — it is pure data.
- A decision belongs in its docs/design doc and its bead, not in a second copy elsewhere.

Language and toolchain guidance lives in skills, not here:

| Skill | Covers |
|---|---|
| `rust` | Toolchain and crate policy, cargo vs Bazel, fmt/clippy gates, immutable data and optics, tracing spans, Stateright/Loom/Kani |
| `bazel` | Build and test gate, BUILD.bazel and MODULE.bazel maintenance, repinning, spike crates |
| `starlark` | The starlark crate front end, no native modules, BUILD vs .bzl dialect, AST memory rule |
| `lean` | What the spec/ modules must state and when a bead needs one |
| `bead-standards` | model/* and effort/* labels on every work bead |
