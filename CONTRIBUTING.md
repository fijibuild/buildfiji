# Contributing to fjfj

fjfj is a Bazel-compatible build tool written in Rust — see
`docs/ARCHITECTURE.md` for the shape of the project and
`docs/design/*.md` for the reasoning behind specific decisions before
touching a hard problem; each design doc is backed by a beads epic.

## Getting set up

You need a recent Rust toolchain (edition 2024, latest stable — see
`rust-toolchain.toml`) and Bazel. Bazel is the source of truth:

```sh
bazel build //... && bazel test //...   # the real gate; clippy + rustfmt
                                          # run as aspects on every build
bazel run //:fjfj -- build //...         # dogfooding fjfj on itself
```

Cargo is used only to maintain `Cargo.toml`/`Cargo.lock` — `cargo build`
works for a quick local check, but `cargo test` is **not** a gate; Bazel
is. After changing any `Cargo.toml`, run `cargo fmt --all`, then
`CARGO_BAZEL_REPIN=1 bazel build //...` and update the crate
`BUILD.bazel` deps by hand to match.

## Before you open a PR

- `bazel build //... && bazel test //...` passes — this also runs
  clippy and rustfmt as aspects, so a lint or formatting problem shows
  up here, not in review.
- `cargo fmt --all` if you touched any Rust.
- If you changed `spec/` (the Lean 4 architecture spec), `cd spec &&
  lake build` until the Lean Bazel rules land (`buildfiji-4b0`).
- Follow the conventions in `CLAUDE.md` (immutable data and optics,
  no native Starlark modules, Bazel 9.2.0 as the compatibility target,
  etc.) — it's the same instructions AI agents working on this repo
  follow, and applies to human contributions too.

## Tracking work

This project uses [`bd` (beads)](https://github.com/gastownhall/beads)
for issue tracking rather than GitHub Issues for in-progress work — see
`bd prime` for the full workflow if you have `bd` installed. GitHub
Issues (via the templates in `.github/ISSUE_TEMPLATE/`) are still the
right place to report a bug or request a feature from outside the
project; they get triaged into beads.

```sh
bd ready           # what's available to work on
bd show <id>       # an issue's full context
bd update <id> --claim
```

## Pull requests

- Branch off `main`; keep commits focused (one logical change per
  commit is fine, doesn't need squashing).
- Reference the beads issue id in the PR description if one exists
  (`buildfiji-xyz.N`).
- A design decision worth remembering later belongs in `docs/design/`
  or a `bd remember`, not just the PR description — PRs merge and
  scroll out of view; those don't.
