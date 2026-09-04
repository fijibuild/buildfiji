# Command line compatibility

Target: `sed -i s/bazel/fjfj/ .github/workflows/*.yml` is enough to switch.

- Same commands: build, test, run, query, cquery, aquery, fetch, clean,
  info, version, shutdown, mod, coverage, print_action, dump, help.
- Same `.bazelrc` discovery order (system, workspace, home, `--bazelrc`,
  `try-import`, `import`, `common`/`build:config` sections, `--config`).
- Same target pattern syntax and `--` handling; same exit codes (0, 1
  build failed, 2 command line, 3 tests failed, 4 no tests found, 8
  interrupted, 36/37 infra).
- Flag policy (superseded 2026-09-03, see "Flag surface" below): a flag
  Bazel accepts but fjfj doesn't implement now fails the invocation
  rather than being accepted with a warning — a `.bazelrc` shared with
  Bazel that uses a flag fjfj hasn't wired up yet stops fjfj outright
  instead of quietly building with that flag doing nothing.
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

`UnknownFlagPolicy` (`Warn`/`Strict`) and `apply_policy` still back
`canonicalize_flags`, which reports Bazel's own `unrecognized option`
message for a single flag in isolation; `build`'s own leftover-token
handling moved to `clap_flags::validate`, below.

## Flag surface (decision 2026-09-03, supersedes "Flag policy" above)

`fjfj-bazel-compat::clap_flags::command_for(command)` builds a
`clap::Command` with one `Arg` per `bazel_flags::FLAGS` entry
`command` accepts — negatable flags get a second hidden `--no<name>`
switch, `old_name`/abbreviation become a clap alias/short, value-taking
flags accept one attached-or-space-separated string each (`Append` when
`allows_multiple`). It doesn't attempt real per-flag typed parsing (no
`type_converter` -> `clap::builder::ValueParser` mapping) — that's still
each `*_flags` module's job over the same raw tokens, run separately.

`clap_flags::validate(args, command, implemented)` runs `command`'s raw
argv slice (after `flag_alias::apply`) through that `Command` and fails
loudly rather than warning: a token that isn't a real Bazel flag for
`command` fails via clap's own strict argument matching (its usual
unknown-flag behavior, which this module doesn't relax); a token that
*is* a real flag but isn't in the caller's `implemented` list — the union
of every `*_flags` module's own `IMPLEMENTED` constant for that command —
also fails, with a distinct message ("recognized... but not implemented
by fjfj yet"). Both cases used to be `UnknownFlagPolicy::Warn`'s job
(print a warning, keep going): a flag Bazel accepts but fjfj silently
does nothing with is a *worse* failure mode than refusing to run, since
the build proceeds looking like it honored a flag it didn't — see
`buildfiji-gwl.15`/`buildfiji-gwl.16`.

One wrinkle clap's own hyphen-token handling can't resolve on its own: a
negative target pattern (`-//pkg:excluded`, `-@repo//pkg:excluded`) looks
exactly like an unmatched flag syntactically. `command_for`'s trailing
positional deliberately has no `allow_hyphen_values` (that would swallow
*any* unmatched hyphen token, typos included, defeating the fail-loud
goal), so `validate` pulls out anything shaped like a negative target
pattern before handing the rest to clap; `TargetPattern::from_str` still
does the real parsing (and can still reject a malformed one) on the
original, unfiltered tokens downstream.

`fjfj build`'s dispatch runs `validate` once, right after
`flag_alias::apply`, before any `*_flags::extract` call — once it passes,
every remaining `*_flags::extract` call and the final
`TargetPattern::from_str` pass only ever see tokens `validate` already
vouched for.

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

## `--flag_alias`, `--check_visibility`, `--memory_profile` (decision 2026-09-03)

`--flag_alias=<name>=<label>` splits into two functions rather than one,
matching Bazel's own two-pass handling: `flag_alias::extract` collects
every occurrence into a name→label table (it's accepted by every command,
so unlike this crate's other `extract` functions it takes no `command`
parameter), then `flag_alias::apply` rewrites later `--<alias>` tokens in
the rest of argv to `--<label>`. Splitting them lets a caller log or
inspect the alias table independent of rewriting. `apply`'s output is the
label form fjfj-starlark's build-setting resolution will understand once
it exists — until then it's just a different, still-unrecognised flag
name, the same as any other Starlark flag today.

`--[no]check_visibility` and `--memory_profile=<path>` are grouped in one
`misc_flags` module rather than given one each: both are single-field
concerns with no implementation to attach to yet (visibility enforcement,
memory profiling), so a struct-per-flag module would be pure ceremony.

## `--registry`, `--allow_yanked_versions`, `--ignore_dev_dependency`, `--override_module` (decision 2026-09-04)

Split across two crates on purpose. `fjfj-bazel-compat::bzlmod_flags`
extracts these into a `BzlmodFlags` of raw strings — same shape as every
other `*_flags` module — and stops there: turning
`--allow_yanked_versions`'s value into a `fjfj_bzlmod::YankedPolicy`, or
`--override_module=name=path` into a `ModuleOverride`, needs types from
`fjfj-bzlmod`, and this crate doesn't depend on it (bazel-compat is a
flag/CLI-syntax layer, bzlmod resolution is a separate concern one layer
up). `fjfj-cli` is where both meet: `bzlmod_registries` turns `--registry`
into `fjfj_bzlmod::Registry`s (repeated occurrences *replace* Bazel's
`https://bcr.bazel.build` default, per the flag's own docs, rather than
adding to it), and `bzlmod_resolve_options` turns the rest into a
`ResolveOptions` — including wiring `include()` support
(buildfiji-mum.22) to a `WorkspaceIncludeSource` rooted at the workspace
directory.

`fjfj build`'s dispatch resolves the module graph right after computing
the workspace status snapshot, same fail-fast reasoning: reads
`MODULE.bazel` from the current directory (no workspace-root search
exists yet — matching how `bazel build` is invoked from the repository
root in this repo today) and logs the selected module count. There is no
`fjfj build` yet for it to feed into; this is forward progress on a real
prerequisite, not a finished pipeline.

`--check_direct_dependencies` and the lockfile modes are out of scope
here — the former needs the direct-dependency comparison Bazel's `mod`
output would show (buildfiji-9s8.4), the latter needs
`MODULE.bazel.lock` (buildfiji-mum.7).
