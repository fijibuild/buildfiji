# How a build runs

`fjfj build` is the command the rest of the tool exists to serve. This page
follows one invocation from the command line to the summary, to show where each
crate fits and why the stages are ordered as they are. It describes the shape
of the code on `main`; the design records in [`design/`](../design/) give the
reasoning behind individual stages.

## The stages

1. **Read the command line.** Flags from the workspace, system and home
   `.bazelrc` files are placed in front of the ones you typed, so the command
   line wins. `--flag_alias` aliases are rewritten. Negative patterns are
   accepted only after `--`, as in Bazel.
2. **Check every flag.** Each token is checked against Bazel's generated flag
   table and against the list of flags fjfj implements. This happens *before*
   anything runs, so an unimplemented flag stops the invocation at once; see
   [The compatibility model](compatibility-model.md).
3. **Find the workspace and parse the patterns.** The root is the nearest
   ancestor holding `MODULE.bazel`, `REPO.bazel`, `WORKSPACE` or
   `WORKSPACE.bazel`. Patterns are parsed relative to where you are in it.
4. **Resolve modules and make repositories.** `MODULE.bazel` is resolved (see
   [How bzlmod resolution fits in](bzlmod-resolution.md)) and the repositories
   the patterns need are made. This is blocking work, so it runs off the async
   runtime.
5. **Load and analyse.** BUILD files are evaluated to packages (`fjfj-starlark`,
   with `fjfj-loading` deciding which directories are packages), and rules are
   analysed into configured targets, providers and actions (`fjfj-analysis`).
   The `INFO: Analyzed ...` line is printed here.
6. **Execute.** The scheduler (`fjfj-exec`) runs the actions, up to `--jobs`
   at a time, each in the sandbox the spawn strategy names, checking the action
   cache first. By design, outputs are written into a scratch directory and moved
   into `bazel-out`, and the cache entry is recorded last, so a kill at any
   point costs at most a rerun.
7. **Report.** The `Target ... up-to-date`, `INFO: Elapsed time`, process and
   `Build completed` lines are printed, and the convenience links are updated.
   For `test`, the tests that were built are run and summarised; for `run`, the
   program is started.

`cquery` and `aquery` stop after stage 5, and `query` after loading: they
analyse (or only load) what the expression names and print, building nothing.

## Why three phases and one graph

Loading, analysis and execution are separate in Bazel because each has a
different cost and a different reason to be redone. fjfj keeps the separation
but plans for all three to be nodes of one memoising graph (`fjfj-engine`), so
that a persistent server can invalidate and recompute only what changed. That
server does not exist yet: today each invocation builds the graph from
scratch, so every build starts cold. The
reasoning, and the options that were weighed, are in
[`design/incremental-engine.md`](../design/incremental-engine.md).

## Why the sandbox is part of execution

Each action runs in a directory that holds links to its declared inputs and
nothing else. Anything it writes that is not a declared output is gone
afterwards. This is what catches a rule that reads an undeclared file: the
build fails here instead of working by accident and breaking on another
machine or under remote execution. On Linux the default strategy adds user,
mount, pid and ipc namespaces, with the file system read-only apart from the
sandbox and `/tmp`. See [`design/sandboxing.md`](../design/sandboxing.md).

## Where to look

| To understand | Read |
|---|---|
| The crates and their roles | [`ARCHITECTURE.md`](../ARCHITECTURE.md) |
| Flag handling | [`design/cli-compat.md`](../design/cli-compat.md) |
| Starlark, loading, repositories | [`design/starlark-and-loading.md`](../design/starlark-and-loading.md) |
| The query family | [`design/query-family.md`](../design/query-family.md) |
| Spans and the trace file | [`design/telemetry.md`](../design/telemetry.md) |
