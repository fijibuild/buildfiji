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

## Several modules (buildfiji-lfe)

How `src/multi_matrix.rs` was made: `multi_cases.py` writes `multi_cases.json`
(a registry of local-path modules, the root's files, the canonical repos to
build), `python3 multi_runcase.py multi_cases.json multi_out.json` runs each in
real Bazel 9.2.0 with `bazel build @@<repo>//:f` (the prints, the errors, the
`BUILD.bazel` of each repo made), and `multi_matrix.py` writes the table.

## Bazel's own repository rules (buildfiji-mum.12)

How `src/http_archive_matrix.rs` was made: `ha_cases*.json` are the workspaces
(`module`: a `MODULE.bazel` that calls `http_archive`, `http_file`,
`git_repository`, `local_repository` and so on; `files`: what sits beside it;
`serve`: what a local HTTP server has, archives built on the fly; `git`: local
git repositories with fixed authors and dates). `python3 harepro.py
ha_cases1.json OUT.json` runs each in real Bazel 9.2.0 with `bazel fetch
--repo=@x` and records the prints, the log, the tree of every repository made
and the requests the server got; `python3 mkha.py OUT1.json OUT2.json ...`
writes the table. Placeholders in a case: `@URL@`, `@SHA:name@`, `@INT:name@`,
`@GIT:repo@`, `@COMMIT:repo@` and `@WS@` (the workspace directory).
