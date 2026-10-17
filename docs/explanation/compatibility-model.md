# The compatibility model

fjfj's central promise is that a repository which builds with
`bazel build //...` builds with `fjfj build //...`, using the same
`MODULE.bazel`, `BUILD` files, `.bzl` rules, `.bazelrc` files and flags, and
can share a remote cache and remote executors with Bazel. This page explains
what that promise means, what it deliberately leaves out, and why fjfj
refuses to run rather than approximate.

## Bazel's behaviour is the specification

There is no separate fjfj language or flag set to design. The target is the
observable behaviour of **Bazel 9.2.0** (chosen 2026-09-03): which flags
exist and what they default to, which `--incompatible_*` migrations are on,
which Starlark builtins are available, what the output tree looks like, and
what exit code a failure produces. When the documentation is unclear, the
answer is whatever Bazel does when you run it.

The compatibility is *observable*, not structural. fjfj does not copy
Skyframe or any other Java internals. It copies what scripts, CI systems and
rule authors can see. That distinction decides most design questions: an
internal that nobody outside can observe is free to differ, while an exit
code, a flag spelling or a resolved module version is not.

## One table, generated from Bazel

Rather than maintaining a hand-written list of flags, fjfj generates its flag
table from Bazel itself. `bazel help flags-as-proto` lists every flag Bazel
accepts for every command, with types, defaults and metadata. That output is
decoded and checked in as `generated.rs`, and regenerated when the pinned
Bazel version changes. Two consequences follow:

- The question "is this a real Bazel flag?" never depends on someone
  remembering to add it.
- The question "does fjfj *do* anything with it?" is separate and explicit.

The second question is where the design is opinionated.

## Refusing is better than ignoring

Consider a CI job that runs `bazel build --remote_executor=grpc://host:8980
//...`. The easy way for fjfj to be "compatible" is to accept the flag, print
a warning, and build locally. fjfj used to work that way and stopped.

A warning scrolls past. The build then proceeds looking as though it
honoured the flag, and it did not. That is a worse failure than an error:
the output is wrong in a way that is hard to notice and harder to trace back
to a flag that was silently dropped. A `.bazelrc` shared with Bazel is
especially exposed, because the person who wrote the flag is not the person
running fjfj.

So fjfj sorts every flag into one of three states and treats the last two as
errors:

1. **Implemented**: some module reads it and acts on it.
2. **Recognized but not implemented**: a real Bazel flag. fjfj fails with a
   distinct message saying so, naming the command, and pointing at `bd
   ready`.
3. **Unknown**: not a flag for this command. fjfj fails with the argument
   parser's own message.

The cost is that fjfj cannot run a given repository until every flag it uses
is implemented. The benefit is that whenever fjfj does run, the flags mean
what they mean in Bazel. This is the trade the project chose, recorded in
[`design/cli-compat.md`](../design/cli-compat.md) under "Flag surface".

The gate is a validator, not a decoder. It checks tokens against the table
and the list of implemented names; each `*_flags` module separately decodes
the values it owns. A flag becomes implemented when a module claims it and
the command reads the result, which is why [wiring one
up](../how-to/wire-up-a-bazel-flag.md) touches both places.

## Exit codes and prefixes are part of the contract

Scripts test `$?` and grep stderr for `ERROR:`. fjfj therefore uses Bazel's
numbering (`1` build failed, `2` command-line problem, `3` tests failed,
`37` internal error, and so on) instead of a convention of its own, and maps
every failure to a code at one choke point. See
[Exit codes](../reference/exit-codes.md).

Internally, each failure is a variant of one error type that knows its exit
code: a command-line problem, a build failure, a failed test run, a query or
fetch failure, and so on. An internal error prints `FATAL:` rather than
`ERROR:` because it means a bug in fjfj, not something the user's input can
fix.

## What compatibility does not promise

- **Not every command or flag.** The command names are all accepted so the
  surface matches `bazel help`, but `shutdown`, `print_action` and `dump` are
  stubs, and remote caching and execution, build event output and `--profile`
  are among the flags that are refused. [Commands](../reference/commands.md)
  and [Flags](../reference/flags.md) are the current truth.
- **Not byte-identical everywhere.** The build and test summaries, query and
  aquery formats and error texts are compared with Bazel 9.2.0 and follow its
  wording. The known differences (hash-ordered sets, configuration checksum
  digits, and a few others) are listed with their beads in
  [`design/query-family.md`](../design/query-family.md). `mod`'s text output
  is fjfj's own; only `mod graph --output=json` is matched.
- **Not a fallback to Bazel.** An idea recorded in the design is a mode in
  which unsupported commands shell out to a real Bazel. It is not built, and
  fjfj today fails instead.

## Where the decisions live

The reasoning behind specific choices is in the design records, not in this
page. The ones that bear on compatibility:

- [`design/cli-compat.md`](../design/cli-compat.md): `.bazelrc` parsing, the
  flag table and registry, the flag surface, and the flags implemented so
  far.
- [`design/starlark-and-loading.md`](../design/starlark-and-loading.md): the
  Starlark dialect, label rules and bzlmod.
- [`ARCHITECTURE.md`](../ARCHITECTURE.md): the principles these follow from.
