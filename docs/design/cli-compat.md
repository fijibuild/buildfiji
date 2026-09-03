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
