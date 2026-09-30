# Download, extract and patch probes

How `crates/fjfj-starlark/src/repo_download_matrix.rs` was made
(buildfiji-mum.8.3).

- `cases*.json`: the repository rules probed, and what a local HTTP server
  serves them (`dlcases.py`; a served file is text, hex, a status or an
  archive of members, in any format the `tar`, `gzip`, `bzip2`, `xz`, `zstd`
  and `ar` tools and `zipfile` can make). `@URL@`, `@SHA:name@` and
  `@INT:name@` in a rule stand for the server, and what a served file hashes
  to.
- `dlrepro.py CASES.json OUT.json [N]` runs each in real Bazel 9.2.0 with
  `bazel fetch --repo=@name --repository_cache=<fresh>`, and records what it
  printed, its log, the exit code, the repository's tree, and the paths the
  server was asked for.
- `dlshow.py OUT.json [from to]` prints the results one case at a time.
- `mkdl.py OUT1.json ...` writes the table the test replays. It leaves out
  rows that depend on the machine, on the exact bytes of an archive (which the
  test builds itself), or on a Bazel quirk the log cannot show.

Bazel refuses a plain `http` URL without a checksum, so every probe that
downloads names one.
