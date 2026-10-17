# Commands

```
fjfj [--output_base=DIR] [--output_user_root=DIR] [--bazelrc=FILE]... <COMMAND> [ARGS]...
```

Command names and order match `bazel help`. Run fjfj from anywhere inside a
workspace; it finds the root by searching upward for `MODULE.bazel`,
`REPO.bazel`, `WORKSPACE` or `WORKSPACE.bazel`. Outside a workspace, commands
that need one exit `2` with Bazel's message.

| Command | Status |
|---|---|
| [`build`](#build) | Works |
| [`test`](#test) | Works |
| [`run`](#run) | Works |
| [`query`](#query-cquery-aquery) | Works |
| [`cquery`](#query-cquery-aquery) | Works |
| [`aquery`](#query-cquery-aquery) | Works |
| [`fetch`](#fetch) | Works |
| [`mod`](#mod) | Works: `graph`, `deps`, `show_repo`, `explain` |
| [`clean`](#clean) | Works |
| [`info`](#info) | Works, without Bazel's server keys |
| [`version`](#version), [`license`](#license), [`canonicalize-flags`](#canonicalize-flags) | Work |
| [`analyze-profile`](#analyze-profile) | Works (fjfj only) |
| `shutdown`, `print_action`, `dump` | Parsed only. Exit `1` with `ERROR: command not implemented yet: ...` |

fjfj has no server process yet, so there is nothing for `shutdown` to stop and
every invocation starts cold. The daemon is tracked under the incremental
engine epic (`bd show buildfiji-23d`).

## Platform support

Development and CI have been on **Linux**; **macOS** (Apple silicon, macOS 27)
builds and runs `build`, `test`, `query` and `mod`, including `fjfj build //...`
and `fjfj test //...` on this repository.

The default execution strategy is `linux-sandbox`, which uses user, mount, pid
and ipc namespaces. Namespaces are Linux's, so on any other OS fjfj uses
`processwrapper-sandbox`: each action runs in a directory of links to its
declared inputs, with no isolation of the rest of the file system or the
network. macOS sandboxing (`darwin-sandbox`) is not implemented
(`buildfiji-fyz.4`).

### Building on macOS

- `.bazelrc` turns on `--enable_platform_specific_config`, and its
  `build:macos` line passes `-Cstrip=none` to Rust crates built for the exec
  configuration. Without it macOS 27's `strip` corrupts proc-macro dylibs and
  rustc reports `E0463: can't find crate for ...`; real Bazel fails the same
  way. fjfj reads `--enable_platform_specific_config` from rc files only; given
  on the command line it is still refused.
- The exec configuration keeps `--//pkg:flag=value` settings, as Bazel's does
  (unless `--incompatible_exclude_starlark_flags_from_exec_config`), which is
  how `--@rules_rust//rust/settings:extra_exec_rustc_flag` reaches tools.
- Real Bazel 9.2.0 cannot yet build this repository's C dependencies on macOS
  27 (`absolute path inclusion ... SDKSettings.json`); fjfj has no such check.
  Tracked in `buildfiji-rq41`.

## Startup options

| Option | Effect |
|---|---|
| `--output_base=DIR` | Root of the output tree for this workspace. |
| `--output_user_root=DIR` | Directory that holds output bases and the repository cache. Default: `$XDG_CACHE_HOME` or `~/.cache`, then `fjfj/_fjfj_<user>`. |
| `--bazelrc=FILE` | Parsed, repeatable. Startup options use Bazel's underscore spelling; `--output-base` is rejected. |

## Reading `.bazelrc`

`build`, `test`, `run`, `query`, `cquery`, `aquery`, `info` and `clean` apply
the flags that the system, workspace and home `.bazelrc` files give them:
`import`s and `--config`s expanded, and the lines of every command they
inherit from included. Flags from rc files come first, so the command line
overrides them. They are checked the same way as flags typed on the command
line, so a shared `.bazelrc` that uses a flag fjfj has not implemented stops
the command (see [Flags](flags.md)).

## `build`

```
fjfj build [FLAGS] [--] <TARGET>...
```

Loads packages, resolves `MODULE.bazel` and makes the repositories the
patterns need, analyses the targets, runs their actions and reports, in
Bazel's words. Progress and the summary go to stderr, for example
`INFO: Analyzed target //:g (1 packages loaded, 1 target configured).`,
`Target //:g up-to-date:` with the files, `INFO: Elapsed time: ...`, and
`INFO: Build completed successfully, N total actions`. A failed build ends
`ERROR: Build did NOT complete successfully`.

Outputs appear under the `bazel-bin`, `bazel-out` and `bazel-testlogs`
convenience links, whose prefix `--symlink_prefix` sets. The resolved module
graph is recorded in `MODULE.bazel.lock` according to `--lockfile_mode`.

A negative pattern (`-//pkg:skip`) is accepted only after `--`, as in Bazel.
Patterns: `//pkg:target`, `//pkg:all`, `//pkg/...`, `//...`, `@repo//pkg:target`,
and relative patterns from a subdirectory. Package names are ASCII only;
target names may contain any non-ASCII character.

Actions run with `--spawn_strategy`; see [Platform support](#platform-support).
There is no remote execution or remote cache yet; `--remote_cache`,
`--remote_executor` and `--disk_cache` are recognized but not implemented.

## `test`

```
fjfj test [FLAGS] <TARGET>...
```

Builds the targets and runs the tests among them, then prints Bazel's summary
(`PASSED`, `FAILED`, `NO STATUS`, and `Executed N out of M tests: ...`).
`--test_output`, `--test_env`, `--test_arg` and `--cache_test_results` are read.
Exit `3` if a test failed, `4` if no target was a test.

## `run`

```
fjfj run [FLAGS] <TARGET> [-- PROGRAM_ARGS...]
```

Builds one target and runs it as `bazel run` does on Linux: with the real path
in `argv[0]`, in the `_main` directory of its runfiles tree, with fjfj's
environment plus `BUILD_WORKSPACE_DIRECTORY`, `BUILD_WORKING_DIRECTORY`,
`BUILD_EXECROOT` and `BUILD_ID`. The program's exit code is `run`'s.
`--run_under` and `--script_path` are read.

## `query`, `cquery`, `aquery`

```
fjfj query  [FLAGS] <EXPRESSION>
fjfj cquery [FLAGS] <EXPRESSION>
fjfj aquery [FLAGS] <EXPRESSION>
```

`query` evaluates over the loaded packages. `cquery` and `aquery` analyse the
targets the expression names without building them, then evaluate over the
configured targets or the actions they registered. Output formats, per command:

| Command | `--output` |
|---|---|
| `query` | `label`, `label_kind`, `location`, `minrank`, `maxrank`, `graph`, `xml`, `build`, `package`, `proto`, `streamed_proto`, `streamed_jsonproto` |
| `cquery` | `label`, `label_kind`, `graph`, `build`, `files`, `starlark`, `proto`, `streamed_proto`, `textproto`, `jsonproto` |
| `aquery` | `text`, `commands`, `summary`, `proto`, `streamed_proto`, `textproto`, `jsonproto` |

Where fjfj's output differs from Bazel 9.2.0 (ordering within hash-ordered
sets, configuration checksum digits, and a few others), the differences are
listed with their beads in
[`design/query-family.md`](../design/query-family.md).

## `fetch`

```
fjfj fetch [--repo=@name]... [--all] [FLAGS]
```

Makes external repositories. `--repo` makes one repository and what it needs;
`--all` makes every repository in the module graph. A repository that cannot be
made exits `8`, as in Bazel. Reads `--repository_cache`, `--distdir`,
`--override_repository`, `--credential_helper` and
`--credential_helper_timeout`.

## `mod`

```
fjfj mod graph [--output=text|json] [FLAGS]
fjfj mod deps <module>... [FLAGS]
fjfj mod show_repo <repo>... [FLAGS]
fjfj mod explain <module>... [FLAGS]
```

Resolves the workspace's `MODULE.bazel` and reports on the result. `FLAGS` are
the [bzlmod flags](flags.md#bzlmod). Output goes to stdout. Resolution writes
`MODULE.bazel.lock` unless `--lockfile_mode=off`.

| Subcommand | Output |
|---|---|
| `graph` | The dependency tree, rooted at `<root>`. `--output=json` uses the shape of `bazel mod graph --output=json`. `bazel_tools` is hidden, as in Bazel. |
| `deps` | Each module's own `bazel_dep` edges and what each resolved to, with the originally requested version. |
| `show_repo` | Each repo's `key`, `name`, `version`, `repo_name`, `canonical_repo_name` and `registry`. Repository rule attributes are not shown. |
| `explain` | Each module's selected version, then every requester in the unpruned graph and the version it asked for. |

Only the JSON form of `graph` is checked against Bazel; the other output is
fjfj's own format. A missing subcommand, an unknown one, a module not in the
graph, or no workspace exits `2`; a resolution failure exits `1`.

## `clean`

Removes the output tree as Bazel's `clean` does: the execroot, the action cache
and the convenience links. `--expunge` removes the whole output base; `--async`
(or `--expunge_async`) renames the tree and deletes it in a separate process,
saying where it went.

## `info`

With no keys, prints every key as `key: value`, in byte order. One key prints
its value alone; several print `key: value` in the order asked. An unknown key is
reported after the rest and exits `2`. `--show_make_env` adds the configuration's
make variables. Bazel's server keys (`server_pid`, the JVM's) are not present,
because there is no server.

## `version`

Prints `fjfj <version>`.

## `license`

Prints a short Apache-2.0 notice. The full text is `LICENSE` in the repository
root.

## `canonicalize-flags`

```
fjfj canonicalize-flags [--for-command <COMMAND>] [--] [FLAG]...
```

Prints the flags in canonical form on one line: abbreviations expanded to long
names, order preserved. `--for-command` defaults to `build`. A flag that is not a
Bazel flag prints `ERROR: unrecognized option '<flag>'` and exits `2`. It
checks that a flag exists in Bazel, not that fjfj implements it.

## `analyze-profile`

```
fjfj analyze-profile <TRACE> [--top N]   # N defaults to 15
```

Summarises a trace file written with `FJFJ_TRACE_FILE`: time in each step,
how many commands ran at once, time by mnemonic, the critical path and the
slowest actions. Not a Bazel command. See
[Profile a build](../how-to/profile-a-build.md).
