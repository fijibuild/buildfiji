# Module extension probes

How `crates/fjfj-repo/src/replay_matrix.rs` was made (buildfiji-mum.8.4).

- `cases*.json`: the workspaces probed (`xcases.py` builds them: a
  `MODULE.bazel`, an extension `.bzl`, the repositories to fetch).
- `xrepro.py CASES.json OUT.json [N]` runs each in real Bazel 9.2.0 with
  `bazel fetch --repo=@name` (from a clean output base), in `N` parallel
  workspaces under `$PROBE_SCRATCH` (default `/tmp/fjfj-repo-probes`), and
  records the `print`s, the log, the exit code, the root module's repo mapping
  (`bazel mod dump_repo_mapping ""`) and the tree of every repository made.
- `xshow.py OUT.json [from to]` prints the results one case at a time.
- `mkext2.py OUT1.json OUT2.json ...` writes the table the test replays. It
  leaves out rows whose answer depends on the machine (the OS, parallel
  fetch order), experimental flags, Bazel's Java object names, or wording the
  Starlark runtime owns (buildfiji-v32).
