# Command line compatibility

Target: `sed -i s/bazel/fjfj/ .github/workflows/*.yml` is enough to switch.

- Same commands: build, test, run, query, cquery, aquery, fetch, clean,
  info, version, shutdown, mod, coverage, print_action, dump, help.
- Same `.bazelrc` discovery order (system, workspace, home, `--bazelrc`,
  `try-import`, `import`, `common`/`build:config` sections, `--config`).
- Same target pattern syntax and `--` handling; same exit codes (0, 1
  build failed, 2 command line, 3 tests failed, 4 no tests found, 8
  interrupted, 36/37 infra).
- Flag policy: known flags are typed; flags Bazel accepts but fjfj does not
  implement are accepted with a one-line warning (opt-in
  `--fjfj_strict_flags` turns them into errors). This keeps shared
  `.bazelrc` files working during the transition.
- Output layout: `bazel-bin`, `bazel-out`, `bazel-testlogs` convenience
  symlinks are created with the same names (configurable via
  `--symlink_prefix`).
- `bazelisk` support: honour `.bazelversion` for a `fjfj` version file and
  consider a `USE_BAZEL_FALLBACK` mode where unsupported commands shell out
  to real Bazel.

## `.bazelrc` lexing/parsing (decision 2026-09-03)

`.bazelrc` is not a standard config format: `import`/`try-import`
directives, `[command[:config]] flag flag...` lines with POSIX-shell-style
word splitting (quotes, backslash escapes, no variable expansion or
globbing), line continuation, and a separate config-expansion pass
(`--config=foo` pulls in every `command:foo` line for the active command,
recursively, with cycle detection) — no crate models this grammar
directly.

- Lexing: [Chumsky](https://github.com/zesterer/chumsky) by default; the
  in-house [Regal](https://github.com/NathanHowell/regal) lexer is reserved
  for crates where a generated-DFA, incremental-relex front end is worth
  the extra machinery (e.g. Starlark loading at scale, see
  `starlark-and-loading.md`). `.bazelrc` files are small, so Chumsky alone
  is the practical default here.
- Parsing: Chumsky combinators throughout, producing a typed directive AST
  (`Import`, `TryImport`, `CommandFlags { command, config, flags }`)
  independent of the discovery-order and config-expansion logic, which
  stay hand-written (they're pure Bazel semantics, not parsing).
- Diagnostics: [Ariadne](https://github.com/zesterer/ariadne) for rendering
  parse errors with source spans, matching Chumsky's error/span model.

## Bazel flag table (decision 2026-09-03)

`bazel help flags-as-proto` prints a base64-encoded, serialized
`bazel_flags.FlagCollection` message (proto2; vendored at
`fjfj-bazel-compat/proto/bazel_flags.proto` for reference) — every flag
Bazel accepts, per command, with defaults, types, and metadata tags
(`INCOMPATIBLE_CHANGE` marks the `--incompatible_*` migration flags; there's
no separate table for those).

`fjfj-bazel-compat::bazel_flags::FLAGS` is this table, generated once
against the pinned Bazel version and checked in as plain Rust data
(`src/bazel_flags/generated.rs`), refreshed by
`cargo run -p fjfj-bazel-compat --bin refresh_bazel_flags` when the pinned
version bumps. That refresh binary decodes the proto with a ~150-line
hand-rolled wire-format reader rather than `protoc`/`prost-build`: pulling
in a protoc dependency (system binary or `prost-build`'s vendored one) is
disproportionate machinery for a script that runs rarely against one small,
stable, all-scalar/repeated-string proto2 message. The reader's
field-number mapping must be kept in sync with the vendored `.proto` by
hand if Bazel ever changes it. It is a `cargo run`-only dev tool (writes
into the source tree, shells out to `bazel`), not built by
`bazel build //...` — see `BUILD.bazel`'s glob exclude.

## Flag registry (decision 2026-09-03)

`fjfj-bazel-compat::flag_registry::FlagRegistry` indexes the generated
`bazel_flags::FLAGS` table by every name a flag can be written with — its
own name, a deprecated `old_name`, `--no<name>` (if negatable), and its
single-character abbreviation (Bazel's `-j`/`-c`/etc.) — and resolves one
raw token (`--name`, `--name=value`, `--noname`, `-x`, `-xvalue`) against a
command, distinguishing "not a flag at all" from "a real Bazel flag, just
not for this command" (the latter still reports which commands it *is*
known for). `"startup"` is the pseudo-command for startup options.

`UnknownFlagPolicy` (`Warn` default, `Strict` behind `--fjfj_strict_flags`)
turns an unresolved token into either a one-line warning to print and
continue, or the error to propagate — Bazel-flag-compatibility-during-
migration is the point of `Warn`, per this doc's "Flag policy" section
above.

## Diagnostics flags (decision 2026-09-03)

`--keep_going`/`-k`, `--verbose_failures`, `--subcommands`/`-s`,
`--explain=<path>`, and the now-no-op `--verbose_explanations` are pulled
out of a command's raw argv slice by
`fjfj-bazel-compat::diagnostics_flags::extract`, using `FlagRegistry`
rather than a clap field per flag — consistent with target patterns
already being captured as raw strings (`TargetArgs::patterns`) rather than
individually typed. A token `extract` doesn't recognise is left untouched
for the caller (a real flag or a target pattern); it only peels off these
five. This is the shape future flag groups (test flags, action-env, …)
should follow rather than growing clap's own flag surface.

## Workspace status and stamping (decision 2026-09-03)

`--workspace_status_command=<program>`, `--[no]stamp`, and
`--embed_label=<value>` are pulled out the same way, by
`fjfj-bazel-compat::workspace_status_flags::extract`. The parsing and
partitioning rules for the command's output live separately, in
`fjfj-bazel-compat::workspace_status` (pure, no I/O, matching Bazel's own
[documented contract](https://bazel.build/docs/user-manual#workspace-status)):
a `KEY VALUE` line per key, `[A-Z_]+` only, no duplicates; keys prefixed
`STABLE_` are "stable", everything else "volatile"; `BUILD_EMBED_LABEL`,
`BUILD_HOST`, `BUILD_USER` are always stable and `BUILD_TIMESTAMP`,
`FORMATTED_DATE` always volatile, regardless of what the command printed.
`fjfj-exec::workspace_status` is the I/O half — runs the program (failing
the build on a non-zero exit, per spec), reads `$USER`/hostname and the
clock for the built-ins, and writes `stable-status.txt`/
`volatile-status.txt`.

The contract that makes `--stamp` useful without forcing a rebuild every
build: a *stable* status change invalidates stamped actions; a *volatile*
change alone (the common case — `BUILD_TIMESTAMP` differs every time)
never does. `WorkspaceStatus::invalidates` encodes this by comparing only
the stable map. `FORMATTED_DATE`'s calendar math uses the `time` crate
(`OffsetDateTime::now_utc`); `time::Month`/`time::Weekday` spell names out
in full, so `fjfj-exec::workspace_status::format_date` takes a 3-letter
prefix to match Bazel's `EEE`/`MMM`.

## Remaining commands (decision 2026-09-03)

`help` needed no new `Command` variant: clap's `Subcommand` derive already
generates a `help` subcommand (`fjfj help`, `fjfj help build`) unless
disabled, so adding one of our own would only collide with it.

`canonicalize-flags` is built on the same `FlagRegistry` every other
flag-consuming module in this crate uses
(`fjfj-bazel-compat::canonicalize_flags::canonicalize`), rewriting each
token to `--name`, `--noname`, or `--name=value`. Unlike
`diagnostics_flags`/`workspace_status_flags`, an unresolved token is the
command's *whole* job failing, not something to leave for a caller
downstream — `canonicalize-flags` takes only flags, no target patterns, so
`FlagRegistry`'s existing `UnknownFlagError` is the right and only error.

`license` prints a short notice, matching Bazel's own `license` command —
the full text lives in the repository's `LICENSE` file, same as Bazel's.
