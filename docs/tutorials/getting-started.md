# Getting started

In this tutorial you will build `fjfj`, use it to build a target in a small
workspace, ask it questions about that workspace, and see how it treats a flag
it does not implement. It takes about fifteen minutes.

You need a Linux machine with a Rust toolchain (see `rust-toolchain.toml`) and
Bazel, and a checkout of this repository. fjfj is developed on Linux; see
[Platform support](../reference/commands.md#platform-support) before trying
another OS.

## 1. Build fjfj

From the repository root, use the project's own gate:

```sh
bazel build //:fjfj
```

The binary is at `bazel-bin/fjfj`. Check it runs:

```sh
bazel-bin/fjfj version
```

You should see `fjfj` followed by a version. For the rest of this tutorial,
`fjfj` means that binary; set a variable so the commands work from any
directory:

```sh
export FJFJ="$PWD/bazel-bin/fjfj"
```

(`cargo build -p fjfj` also produces `target/debug/fjfj` on Linux, but Bazel is
the project's gate, and `cargo test` is not.)

## 2. Create a workspace

fjfj finds your workspace by looking for `MODULE.bazel`. Make one with a single
target that copies a file:

```sh
mkdir /tmp/fjfj-tutorial && cd /tmp/fjfj-tutorial
cat > MODULE.bazel <<'EOF'
module(name = "tutorial", version = "0")
EOF
cat > BUILD.bazel <<'EOF'
genrule(
    name = "greeting",
    srcs = ["in.txt"],
    outs = ["out.txt"],
    cmd = "cat $(SRCS) > $@",
)
EOF
echo hello > in.txt
```

## 3. Build a target

```sh
$FJFJ build //:greeting
```

fjfj loads the package, analyses the target, runs the genrule and reports in
Bazel's words, on stderr. Expect lines of this shape:

```
INFO: Analyzed target //:greeting (1 packages loaded, 1 target configured).
INFO: Found 1 target...
Target //:greeting up-to-date:
  bazel-bin/out.txt
INFO: Elapsed time: ..., Critical Path: ...
INFO: ... process...
INFO: Build completed successfully, ... total action...
```

The result is where Bazel would put it:

```sh
cat bazel-bin/out.txt
```

prints `hello`. The `bazel-bin`, `bazel-out` and `bazel-testlogs` links in your
workspace are the same convenience links Bazel makes.

Run the build again. Nothing changed, so the genrule's output is still valid
and the summary should count it as an action cache hit instead of running it
again.

Make it fail on purpose:

```sh
cat >> BUILD.bazel <<'EOF'
genrule(name = "broken", outs = ["broken.txt"], cmd = "exit 3")
EOF
$FJFJ build //:broken
echo "exit code: $?"
```

fjfj prints the failing command in Bazel's wording
(`(Exit 3): bash failed: error executing Genrule command ...`), then
`ERROR: Build did NOT complete successfully`, and exits `1`.

## 4. Ask questions about the workspace

```sh
$FJFJ query '//...'
$FJFJ query --output=graph 'deps(//:greeting)'
$FJFJ cquery '//:greeting'
$FJFJ aquery '//:greeting'
```

`query` answers from the loaded packages. `cquery` and `aquery` analyse the
target first (without building it) and show the configured target and the
actions it registered, including the genrule's command line. Their output
formats follow Bazel 9.2.0; see [Commands](../reference/commands.md#query-cquery-aquery).

## 5. Watch fjfj refuse a flag it cannot honour

fjfj accepts every flag Bazel does, but implements only some of them. Give
`build` one it does not:

```sh
$FJFJ build --remote_executor=grpc://localhost:8980 //:greeting
echo "exit code: $?"
```

```
ERROR: flag '--remote_executor' is recognized by Bazel for 'build' but not implemented by fjfj yet; see `bd ready`
```

The exit code is `2`. fjfj stops rather than build locally while silently
ignoring a flag that asked for remote execution. [The compatibility
model](../explanation/compatibility-model.md) explains why. A misspelled flag
fails too, with a different message:

```sh
$FJFJ build --nosuchflag //:greeting
```

The same check applies to flags that come from a `.bazelrc`: fjfj reads the
workspace and home rc files for the command, so a shared `.bazelrc` that uses an
unimplemented flag stops fjfj in the same way.

## 6. Look at the module graph

Add a dependency to `MODULE.bazel` and print the graph. This step needs network
access to the Bazel Central Registry (or a registry of your own; see [Resolve
modules against a local registry or
override](../how-to/use-a-local-registry-or-override.md)):

```sh
cat >> MODULE.bazel <<'EOF'
bazel_dep(name = "platforms", version = "1.0.0")
EOF
$FJFJ mod graph
$FJFJ mod explain platforms
```

`graph` prints the resolved tree; `explain` shows which version was selected
and who asked for it. Resolution also writes a `MODULE.bazel.lock` beside
`MODULE.bazel`, as Bazel does.

## Where next

- To do more with the module graph, see
  [Inspect a module graph](../how-to/inspect-a-module-graph.md).
- To find out where a build spent its time, see
  [Profile a build](../how-to/profile-a-build.md).
- To see what each command supports today, see
  [Commands](../reference/commands.md).
- To understand the build pipeline you just used, see
  [How a build runs](../explanation/how-a-build-runs.md).
- To help build the missing parts, run `bd ready` and read
  [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

Clean up with `rm -rf /tmp/fjfj-tutorial`.
