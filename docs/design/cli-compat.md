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
