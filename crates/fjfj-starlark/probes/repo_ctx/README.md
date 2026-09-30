# repository_ctx probes

How `crates/fjfj-starlark/src/repo_ctx_matrix.rs` was made (buildfiji-mum.8.2).

- `cases*.json`: the repository rules probed (`rcases.py` builds them: an
  implementation body, the attributes of the rule, the call's keywords and the
  files of the main repository).
- `repro.py CASES.json OUT.json [N]` runs each in real Bazel (9.2.0, through
  `USE_BAZEL_VERSION`) with `bazel fetch --repo=@x`, in `N` parallel workspaces
  and output bases under `$PROBE_SCRATCH` (default `/tmp/fjfj-repo-probes`),
  and records what it printed, the `ERROR:` lines, the exit code, and the
  repository directory's tree (files as their content, `x ` first if
  executable, `-> target` for links).
- `rshow.py OUT.json [from to]` prints the results one case at a time.
- `mkrepo.py OUT1.json OUT2.json ...` writes the table the test replays. It
  leaves out rows that depend on the machine (the environment, `PATH`, the OS
  name), on Bazel's byte strings (buildfiji-b67), or that crash Bazel.

The cases need network only if Bazel has to fetch its own dependencies.
