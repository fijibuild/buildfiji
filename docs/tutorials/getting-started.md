# Getting started

In this tutorial you will build `fjfj`, run its first commands, and use it to
explore the module graph of a small project. It takes about ten minutes and
needs no network access.

fjfj cannot build targets yet, so this tutorial is about the parts that do
work: the Bazel-compatible command line and bzlmod module resolution.

You need a Rust toolchain (see `rust-toolchain.toml`) and a checkout of this
repository.

## 1. Build fjfj

From the repository root:

```sh
cargo build -p fjfj
```

Cargo is enough for a quick local check. The project's real gate uses Bazel
(`bazel build //... && bazel test //...`), but you do not need it for this
tutorial.

Check that it runs:

```sh
./target/debug/fjfj version
```

You should see `fjfj 0.0.1` (or the current version). For the rest of this
tutorial, `fjfj` means `./target/debug/fjfj`; the commands below use an
absolute path through a shell variable so they work from other directories:

```sh
export FJFJ="$PWD/target/debug/fjfj"
```

## 2. See Bazel's command line at work

fjfj accepts the same commands and flags as Bazel. List them:

```sh
$FJFJ --help
```

The same command names as `bazel help` appear. Many are not implemented yet.
Try one:

```sh
$FJFJ test //...
echo "exit code: $?"
```

You will see `ERROR: command not implemented yet: ...` and an exit code of
`1`. Notice the `ERROR:` prefix: scripts that grep Bazel's output for it keep
working unchanged.

## 3. Watch fjfj refuse a flag it cannot honour

Now give `build` a flag that Bazel accepts but fjfj has not implemented:

```sh
$FJFJ build --jobs=4 //...
echo "exit code: $?"
```

The result:

```
ERROR: flag '--jobs' is recognized by Bazel for 'build' but not implemented by fjfj yet; see `bd ready`
```

The exit code is `2` (a command-line problem). fjfj stops rather than
building while quietly ignoring `--jobs`. This is a deliberate choice, and
[the compatibility model](../explanation/compatibility-model.md) explains
why.

A misspelled flag fails the same way, with a different message:

```sh
$FJFJ build --nosuchflag //...
```

## 4. Create a small project

fjfj reads `MODULE.bazel` from the current directory. Make a throwaway
project:

```sh
mkdir /tmp/fjfj-tutorial && cd /tmp/fjfj-tutorial
cat > MODULE.bazel <<'EOF'
module(name = "root", version = "0")
bazel_dep(name = "a", version = "1.0")
EOF
```

The module depends on `a@1.0`. That module is not on the network registry, so
point fjfj at the small registry fixture that ships with this repository.
Substitute the path to your checkout:

```sh
export REGISTRY="file://<path-to-your-checkout>/crates/fjfj-bzlmod/tests/fixtures/registry"
```

## 5. Print the module graph

```sh
RUST_LOG=warn $FJFJ mod graph --registry=$REGISTRY
```

You should see:

```
<root> (root)
  a@1.0 (a)
```

`RUST_LOG=warn` quiets fjfj's informational logging, which otherwise shares
your terminal. The output on stdout is the graph itself; `a@1.0` is the
module fjfj resolved your `bazel_dep` to, taken from the registry.

## 6. Ask about one module

```sh
RUST_LOG=warn $FJFJ mod deps a --registry=$REGISTRY
RUST_LOG=warn $FJFJ mod show_repo a --registry=$REGISTRY
```

`deps` shows the module's own dependency edges and what they resolved to.
`show_repo` shows what fjfj knows about the repository: its key, version,
canonical name (`a+`) and the registry it came from.

Ask about a module that is not in the graph:

```sh
RUST_LOG=warn $FJFJ mod explain b --registry=$REGISTRY
echo "exit code: $?"
```

You get `ERROR: module 'b' is not in the resolved graph`.

## 7. See `build` get as far as it can

```sh
$FJFJ build --registry=$REGISTRY //...
```

fjfj parses your target pattern, computes workspace status, resolves
`MODULE.bazel`, prints `Resolved 3 bzlmod module(s)` and then stops. The
count of three is your root module, `a@1.0`, and the `bazel_tools`
placeholder every module implicitly depends on (`mod graph` hides that one,
as Bazel does):

```
ERROR: fjfj build is not implemented yet; see `bd ready`
```

That final line is expected. Everything before it ran for real.

## Where next

- To explore a larger graph, see
  [Inspect a module graph](../how-to/inspect-a-module-graph.md).
- To learn what each command does today, see
  [Commands](../reference/commands.md).
- To find out why fjfj is built this way, start with
  [The compatibility model](../explanation/compatibility-model.md).
- To help build the missing parts, run `bd ready` and read
  [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

Clean up with `rm -rf /tmp/fjfj-tutorial`.
