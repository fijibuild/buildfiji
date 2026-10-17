# Flags

fjfj accepts the flags of **Bazel 9.2.0**. The complete flag table, with types
and defaults, is checked in as
`crates/fjfj-bazel-compat/src/bazel_flags/generated.rs`.

A flag given to `build`, `test` or `run` (and to a `.bazelrc` line that applies
to them) is in one of three states:

| State | What happens |
|---|---|
| **Implemented** | Accepted and read. Listed below. |
| **Recognized, not implemented** | A real Bazel flag for the command that nothing reads. Fails with `flag '--x' is recognized by Bazel for 'build' but not implemented by fjfj yet` (exit `2`). |
| **Unknown** | Not a Bazel flag for the command. Fails with a parser error (exit `2`). |

Anything not listed below is in the second or third state. Notable examples of
the second: `--remote_cache`, `--remote_executor`, `--disk_cache`,
`--build_event_json_file`, `--profile`.

Flags are accepted as `--flag=value` or `--flag value`; booleans also as
`--noflag`. Abbreviations and old names Bazel accepts resolve to the same flag.
Starlark build settings (`--//pkg:setting=value`) are accepted and
`--flag_alias` rewrites to them.

This page lists *which* flags are read. Their meaning is Bazel's; for what a
flag does, see Bazel's command-line reference. The authoritative list in code
is the `IMPLEMENTED` constant of each `*_flags` module in
`crates/fjfj-bazel-compat/src/`, unioned in `BUILD_IMPLEMENTED` in
`crates/fjfj-cli/src/lib.rs`.

## Build configuration

`build_flags`: read by `build`, `test`, `run`.

| Group | Flags |
|---|---|
| Mode and platform | `--compilation_mode` (`-c`), `--cpu`, `--platforms`, `--host_platform`, `--extra_toolchains`, `--extra_execution_platforms`, `--toolchain_resolution_debug`, `--experimental_platform_in_output_dir`, `--experimental_override_platform_cpu_name` |
| Environment and definitions | `--action_env`, `--define`, `--incompatible_strict_action_env` |
| Execution | `--jobs` (`-j`), `--spawn_strategy` |
| Output | `--symlink_prefix`, `--experimental_convenience_symlinks`, `--show_result`, `--stamp`, `--strip` |
| Target selection | `--build`, `--aspects`, `--output_groups`, `--skip_incompatible_explicit_targets`, `--expand_test_suites` |
| Runfiles | `--enable_runfiles`, `--build_runfile_links`, `--legacy_external_runfiles` |
| Coverage and compilation | `--collect_code_coverage`, `--force_pic`, `--save_temps`, `--fission` |
| C++ and tool options | `--copt`, `--cxxopt`, `--conlyopt`, `--linkopt`, `--host_copt`, `--custom_malloc`, `--grte_top`, `--fdo_optimize`, `--fdo_profile`, `--cs_fdo_profile`, `--xbinary_fdo`, `--memprof_profile`, `--propeller_optimize`, `--fdo_prefetch_hints` |
| Java and proto | `--javacopt`, `--host_javacopt`, `--java_runtime_version`, `--tool_java_runtime_version`, `--java_language_version`, `--tool_java_language_version`, `--java_launcher`, `--proto_compiler`, `--proto_toolchain_for_cc`, `--proto_toolchain_for_java` |

`--spawn_strategy` takes a comma-separated list and uses the first name fjfj
can run: `local` (or `standalone`); `sandboxed` or `linux-sandbox`;
`processwrapper-sandbox`. The default is `linux-sandbox`, which falls back to
`processwrapper-sandbox` where namespaces are unavailable (always, off Linux). An unknown list
exits `2` with `while parsing option --spawn_strategy=...: none of the
strategies is available`.

## `test` and `run`

| Command | Flags |
|---|---|
| `test` | `--test_output` (`summary`, `errors`, `all`, `streamed`; `streamed` is shown as `all`), `--test_env`, `--test_arg`, `--cache_test_results` |
| `run` | `--run_under` (a command line, or `//pkg:tool [options]`), `--script_path` |

`--test_env` and `--test_arg` shape the configuration of every command that
builds. `--run_under` and `--script_path` are refused by `build` and `test`.

## General

| Flag | Notes |
|---|---|
| `--flag_alias=NAME=LABEL` | Rewrites later `--NAME[=value]` to `--LABEL[=value]`. Accepted by every command. A value without `=` fails. |
| `--check_visibility` / `--nocheck_visibility` | Default on. |
| `--memory_profile=PATH` | Parsed. |
| `--enable_bzlmod` / `--noenable_bzlmod` | Accepted so rc files that set it are read. Bzlmod is the only way dependencies are named. |
| `--incompatible_strict_action_env` | Actions always run with the strict environment; turning it off is not honoured yet. |

## Diagnostics

| Flag | Notes |
|---|---|
| `--keep_going` (`-k`), `--verbose_failures`, `--subcommands` (`-s`), `--explain=PATH` | Read. |
| `--verbose_explanations` | Accepted and ignored, as in Bazel, where it is a no-op. |

## Workspace status

| Flag | Notes |
|---|---|
| `--workspace_status_command=PROGRAM` | Run at the start of a build; its stdout is parsed as status keys. A failing command fails the build. |
| `--stamp` / `--nostamp` | Default `--nostamp`. |
| `--embed_label=VALUE` | Becomes `BUILD_EMBED_LABEL`. |

Built-in stable keys: `BUILD_EMBED_LABEL`, `BUILD_HOST`, `BUILD_USER`. A status
command cannot override them. The volatile and stable status files are the
ones `ctx.info_file` and `ctx.version_file` name.

## Output and console

| Flag | Notes |
|---|---|
| `--output_filter=REGEX` | Compiled at startup; an invalid regex exits `2`. |
| `--auto_output_filter=none\|all\|packages\|subpackages` | Default `none`. Used only when `--output_filter` is absent. |
| `--color=yes\|no\|auto`, `--curses=yes\|no\|auto` | Default `auto`, resolved against whether the stream is a terminal. Case-insensitive; other values are rejected. |
| `--show_progress` / `--noshow_progress` | Default on. |
| `--ui_event_filters=FILTERS` | Repeatable. A leading `+` or `-` adjusts the default set (every event kind except `DEBUG`); a bare entry replaces it from that point on. An invalid entry exits `2`. |

## Execution log, remote and build events

| Flag | Notes |
|---|---|
| `--execution_log_compact_file=PATH` | Writes the compact execution log. An empty value means unset. |
| `--remote_grpc_log=PATH` | Creates the file; fjfj makes no remote calls yet. An empty value means unset. |
| `--remote_cache_compression`, `--experimental_remote_cache_compression_threshold=BYTES` | Parsed. Threshold default `100`. |
| `--build_event_publish_all_actions`, `--bes_results_url=URL`, `--bes_timeout=DURATION` | Parsed. No build events are published yet. |

## bzlmod and repositories

Accepted by `build`, `test`, `run`, `fetch` and `mod` unless noted.

| Flag | Default | Notes |
|---|---|---|
| `--registry=URL` | `https://bcr.bazel.build` | Repeatable. Any use replaces the default list. Tried in order. `file://` and `https://`. |
| `--allow_yanked_versions=all\|NAME@VERSION,...` | none | An invalid value exits `2`. |
| `--ignore_dev_dependency` | off | Ignore `dev_dependency = True` `bazel_dep`s and extension usages in the root module. |
| `--override_module=NAME=PATH` | none | Repeatable. Treat the module as a `local_repository` at `PATH`. A value without `=` is rejected. |
| `--lockfile_mode=off\|update\|refresh\|error` | `update` | Whether and how `MODULE.bazel.lock` is read and written. An invalid value exits `2`. |
| `--repository_cache=PATH` | under the output user root | Empty means none. |
| `--distdir=PATH` | none | Local directories searched before downloading. |
| `--override_repository=NAME=PATH` | none | Repeatable. |
| `--credential_helper=[SCOPE=]PROGRAM`, `--credential_helper_timeout=DURATION` | none | A bad value is reported as `While parsing option --flag=value: why`. |
| `--experimental_isolated_extension_usages` | off | |

`fetch` also reads `--repo` and `--all`; `mod` also reads `--output` for
`graph`.

## Other commands

`query`, `cquery` and `aquery` additionally read their own output and graph
flags (`--output`, `--proto:*`, `--graph:*`, `--include_*`, `--query_file`,
`--universe_scope`, `--transitions`, and others); the lists are in
[`design/query-family.md`](../design/query-family.md). `clean` reads
`--expunge`, `--async` and `--expunge_async`; `info` reads `--show_make_env`.
`canonicalize-flags` checks flag names against Bazel's table for the command
named by `--for-command`.
