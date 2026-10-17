# Commands

```
fjfj [--output-base <DIR>] [--bazelrc <FILE>]... <COMMAND> [ARGS]...
```

The command names and order match `bazel help`. Status is as of the
current `main`; `bd ready` lists the work that changes it.

| Command | Status |
|---|---|
| [`version`](#version) | Works |
| [`license`](#license) | Works |
| [`canonicalize-flags`](#canonicalize-flags) | Works |
| [`mod`](#mod) | Works: `graph`, `deps`, `show_repo`, `explain` |
| [`build`](#build) | Partial: validates, then stops (see below) |
| `test`, `run`, `query`, `cquery`, `aquery`, `fetch`, `clean`, `info`, `shutdown`, `print_action`, `dump` | Parsed only. Exit `1` with `ERROR: command not implemented yet: ...` |

## Startup options

| Option | Status |
|---|---|
| `--output-base <DIR>` | Parsed; not used yet |
| `--bazelrc <FILE>` | Parsed, repeatable; not used yet. `.bazelrc` files are not applied to any command |

fjfj reads `MODULE.bazel` from the current directory. It does not search
parent directories for a workspace root.

## `version`

Prints `fjfj <version>`.

## `license`

Prints a short Apache-2.0 notice. The full text is `LICENSE` in the
repository root.

## `canonicalize-flags`

```
fjfj canonicalize-flags [--for-command <COMMAND>] [--] [FLAG]...
```

Prints the flags in canonical form on one line: abbreviations expanded to
long names, order preserved. `--for-command` defaults to `build`. A flag
that is not a Bazel flag prints `ERROR: unrecognized option '<flag>'` and
exits `2`. It checks that a flag exists in Bazel; it does not check that
fjfj implements it.

## `mod`

```
fjfj mod graph [--output=text|json] [FLAGS]
fjfj mod deps <module>... [FLAGS]
fjfj mod show_repo <repo>... [FLAGS]
fjfj mod explain <module>... [FLAGS]
```

Resolves the current directory's `MODULE.bazel` and reports on the result.
`FLAGS` are the [bzlmod flags](flags.md#bzlmod). Output goes to stdout.

| Subcommand | Output |
|---|---|
| `graph` | The dependency tree, rooted at `<root>`. Text by default. `--output=json` uses Bazel's `bazel mod graph --output=json` shape. `bazel_tools` is hidden, as in Bazel. |
| `deps` | For each module: its own `bazel_dep` edges and what each resolved to, including the originally requested version. |
| `show_repo` | For each repo: `key`, `name`, `version`, `repo_name`, `canonical_repo_name`, `registry`. Repository rule attributes are not shown. |
| `explain` | For each module: the selected version, then every requester in the unpruned graph and the version it asked for. |

`--output` is read by `graph` only. Only the JSON shape of `graph` is
checked for conformance against Bazel; the other output is fjfj's own
format and may change.

Errors: no subcommand, or an unknown one, exits `2` with a usage message. A
module not in the graph exits `2` (`module '<name>' is not in the resolved
graph`). A missing `MODULE.bazel` exits `2`. A resolution failure (for
example a version that cannot be found) exits `1`.

## `build`

```
fjfj build [FLAGS] <TARGET>...
```

Today `build` runs the setup half of a build and stops before doing any
work:

1. Rewrites `--flag_alias` aliases.
2. Rejects any flag that is not a real Bazel flag for `build`, or is one
   fjfj does not implement (exit `2`; see [Flags](flags.md)).
3. Parses target patterns (exit `2` on a malformed one).
4. Opens `--execution_log_compact_file` and writes its header, and creates
   `--remote_grpc_log`, so an unwritable path fails immediately.
5. Computes workspace status (`[1] Computing workspace status`).
6. Resolves `MODULE.bazel` (`[2] Resolving MODULE.bazel`) and prints
   `Resolved <n> bzlmod module(s)`.
7. Exits `1` with `ERROR: fjfj build is not implemented yet; see `bd ready``.

Progress lines and the final count go to stdout; logs go to stderr.

### Target patterns

Accepted syntax: `//pkg:target`, `//pkg:all`, `//pkg/...`, `//...`,
`@repo//pkg:target`, and a leading `-` for an exclusion (`-//pkg:skip`).
Patterns must start with `//` or `@`. Package names are ASCII only; target
names may contain any non-ASCII character. Patterns are validated, not yet
expanded.
