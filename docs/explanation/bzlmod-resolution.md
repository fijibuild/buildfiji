# How bzlmod resolution fits in

`fjfj mod` and the first stages of `fjfj build` both start by working out
which modules your project depends on. This page explains what that process
is and why fjfj ports Bazel's algorithm rather than re-deriving it. The
detailed reasoning is in
[`design/starlark-and-loading.md`](../design/starlark-and-loading.md).

## What resolution answers

A `MODULE.bazel` lists direct dependencies, each with a version. Those
modules have their own `MODULE.bazel` files that list more. Resolution
answers: out of all the versions requested anywhere in that graph, which one
does each module end up at?

Bazel's answer is Minimal Version Selection. For each module, take the
highest version *requested by anyone*. Never invent a newer one, and never
pick lower than something asked for. Two modules asking for `platforms@1.0`
and `platforms@1.1` get `1.1`; a requested `1.2` that nobody asked for is
never chosen.

That is why `fjfj mod explain platforms` lists every requester: the selected
version is a function of the whole graph, and the question "why is this
`1.1`?" is answered by finding who asked for it.

## Three steps

1. **Discovery.** Starting at the root, fetch each requested module's
   `MODULE.bazel` from a registry and evaluate it. Modules at the same depth
   are fetched concurrently. This produces the *unpruned* graph, which
   contains every version anyone requested.
2. **Selection.** Apply minimal version selection to produce one version per
   module name, then prune everything not reachable from the root.
3. **Presentation.** `fjfj mod` renders the result four ways, and `build`
   uses it as the first input to the rest of the build. Rendering is separate
   from resolution and does no I/O of its own.

`mod explain` is the one command that needs step 1's unpruned graph
instead of step 2's pruned one. After selection, `build`, `test`, `run` and
`fetch` go on to make the repositories the build needs; see
[How a build runs](how-a-build-runs.md).

## Why it is a port, not a reimplementation

Resolution has to agree with Bazel exactly. One version off is a different
build, with different source on disk and different behaviour. So
`fjfj-bzlmod` is a port of the corresponding Bazel 9.2.0 classes
(`ModuleFileGlobals`, `Discovery`, `Selection`, `Version`, `IndexRegistry`),
checked against golden output from real Bazel runs rather than against the
documentation. The `MODULE.bazel` dialect itself is evaluated with the
`starlark` crate, with only the module-file builtins added.

The port is also where simplifications are *proved* rather than assumed.
Bazel 9.2.0 accepts compatibility levels but forces every module to level 0,
so a module has exactly one candidate per edge and the search over
combinations collapses. fjfj evaluates the single strategy directly, and the
Lean spec (`spec/Fjfj/Bzlmod.lean`) states the consequence as a theorem: two
modules in the resolved graph with the same name are the same module.

## What is a stand-in

Every module implicitly depends on `bazel_tools`, which Bazel ships inside
its own binary. fjfj has no such repository yet, so it supplies a
`bazel_tools` module with no dependencies. `mod graph` hides it, as Bazel
does, but it is not invisible to resolution: Bazel's real `bazel_tools`
depends on other modules, and those requests raise versions elsewhere. A
graph that includes them can therefore differ from Bazel's until the real
file is available. The design doc tracks this.

## What happens after selection

Resolution answers *which* modules. Making their contents is a separate stage
in `fjfj-repo`: a module's repository is made by running the rule its
`source.json` (or a non-registry override) names, and extensions generate
further repositories on demand. The outcome of resolution and of extension
runs is recorded in `MODULE.bazel.lock`, which the resolving commands read and
write according to `--lockfile_mode` (`off`, `update`, `refresh`, `error`).
The design record has the details:
[`design/starlark-and-loading.md`](../design/starlark-and-loading.md).

## Related

- Practical use: [Inspect a module graph](../how-to/inspect-a-module-graph.md)
  and [Resolve modules against a local registry or
  override](../how-to/use-a-local-registry-or-override.md).
- Facts about the flags: [Flags](../reference/flags.md#bzlmod).
