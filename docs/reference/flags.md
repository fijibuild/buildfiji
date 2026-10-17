# Flags

fjfj accepts the flags of **Bazel 9.2.0**. The complete flag table, with
types and defaults, is checked in as
`crates/fjfj-bazel-compat/src/bazel_flags/generated.rs`.

A `build` flag is in one of three states:

| State | What happens |
|---|---|
| **Implemented** | Accepted and read. Listed below. |
| **Recognized, not implemented** | A real Bazel flag for `build` that nothing reads. Fails with `flag '--x' is recognized by Bazel for 'build' but not implemented by fjfj yet` (exit `2`). |
| **Unknown** | Not a Bazel flag for `build`. Fails with a clap error (exit `2`). |

Everything not listed below is in the second or third state. Flags are
accepted as `--flag=value` or `--flag value`; booleans also as `--noflag`.
Abbreviations and old names that Bazel accepts resolve to the same flag.

"Effect today" says what fjfj does with the value now, which is less than
what it will do once builds run.

## General

| Flag | Default | Effect today |
|---|---|---|
| `--flag_alias=NAME=LABEL` | none | Rewrites later `--NAME[=value]` to `--LABEL[=value]`. Accepted by every command. A value without `=` fails. The resulting label-form flag is then rejected as unknown until Starlark build settings exist. |
| `--check_visibility` / `--nocheck_visibility` | on | Parsed. |
| `--memory_profile=PATH` | none | Parsed. |

## Diagnostics

| Flag | Default | Effect today |
|---|---|---|
| `--keep_going`, `-k` | off | Parsed. |
| `--verbose_failures` | off | Parsed. |
| `--subcommands`, `-s` | off | Parsed. |
| `--explain=PATH` | none | Parsed. |
| `--verbose_explanations` | off | Accepted and ignored, as in Bazel, where it is a no-op. |

## Workspace status

| Flag | Default | Effect today |
|---|---|---|
| `--workspace_status_command=PROGRAM` | none | Run at the start of `build`; its stdout is parsed as status keys. A failing command fails the build. |
| `--stamp` / `--nostamp` | `--nostamp` | Parsed. |
| `--embed_label=VALUE` | empty | Becomes `BUILD_EMBED_LABEL`. |

The built-in stable keys are `BUILD_EMBED_LABEL`, `BUILD_HOST` and
`BUILD_USER`; a command cannot override them. The snapshot is computed but
not written to `stable-status.txt`/`volatile-status.txt` yet.

## Output

| Flag | Default | Effect today |
|---|---|---|
| `--output_filter=REGEX` | none | Compiled at startup; an invalid regex exits `2`. |
| `--auto_output_filter=none\|all\|packages\|subpackages` | `none` | Used only when `--output_filter` is absent. `all` shows nothing; `packages` shows rules in the packages named on the command line; `subpackages` also their subpackages. |

## Console

| Flag | Default | Effect today |
|---|---|---|
| `--color=yes\|no\|auto` | `auto` | Resolved against whether stdout is a terminal. |
| `--curses=yes\|no\|auto` | `auto` | Same. With curses on, progress lines overwrite each other. |
| `--show_progress` / `--noshow_progress` | on | Controls the `[n] message` progress lines. |
| `--ui_event_filters=FILTERS` | all event kinds except `DEBUG` | Repeatable. A leading `+` or `-` on an entry adds or removes an event kind from the default set; a bare entry replaces the set from that point on. An invalid entry exits `2`. |

Values are case-insensitive for `--color` and `--curses`; an unrecognised
value is rejected.

## Execution log, remote and build events

| Flag | Default | Effect today |
|---|---|---|
| `--execution_log_compact_file=PATH` | none | Creates the file and writes the invocation header. An empty value means unset. |
| `--remote_grpc_log=PATH` | none | Creates the file only. An empty value means unset. |
| `--remote_cache_compression` / `--noremote_cache_compression` | off | Parsed. Old name `--experimental_remote_cache_compression` also accepted. |
| `--experimental_remote_cache_compression_threshold=BYTES` | `100` | Parsed. |
| `--build_event_publish_all_actions` | off | Parsed. |
| `--bes_results_url=URL` | empty | Parsed. |
| `--bes_timeout=DURATION` | `0` (wait indefinitely) | Parsed. |

## bzlmod

Accepted by `build` and `mod`.

| Flag | Default | Effect |
|---|---|---|
| `--registry=URL` | `https://bcr.bazel.build` | Repeatable. Any use replaces the default list. Registries are tried in order. `file://` and `https://` URLs. |
| `--allow_yanked_versions=all\|NAME@VERSION,...` | none | Permit resolving to yanked versions. An invalid value exits `2`. |
| `--ignore_dev_dependency` / `--noignore_dev_dependency` | off | Ignore `dev_dependency = True` `bazel_dep`s and extensions in the root module. |
| `--override_module=NAME=PATH` | none | Repeatable. Treat the module as a `local_repository` at `PATH`. A value without `=` is left unconsumed and then rejected. |

## Other commands

`mod` reads only the four bzlmod flags above, plus `--output` for `graph`.
`canonicalize-flags` checks flag names against Bazel's table for the command
named by `--for-command`.
