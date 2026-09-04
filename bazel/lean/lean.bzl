"""Interim `lean_library` macro (buildfiji-4b0.4).

This stands in ahead of two beads that don't exist yet: buildfiji-4b0.1 (a
hermetic toolchain repository rule for the pinned Lean release in
`versions.bzl`) and buildfiji-4b0.2 (the real `lean_library` rule that
compiles `.lean` to `.olean`/`.ilean` with `LeanInfo` providers and a
depset-based `LEAN_PATH`). Neither exists yet, so this rule does not invoke
`lean` at all — it only tracks a target's own sources and its deps'
transitive sources. `bazel build //spec/...` therefore always succeeds
trivially; `lake build` (spec/README.md) remains the only thing that
actually type-checks spec/, per buildfiji-cmd.19.

What this rule buys now: a real Bazel target graph shaped like spec/'s
module graph, so spec/BUILD.bazel can state the same dependency edges
`Fjfj.lean`'s `import` statements do — checked for drift by
spec/check_modules.sh — instead of a `filegroup` blob with no structure.
buildfiji-4b0.2 replaces `_lean_library_impl` below with one that adds real
actions; callers of this macro do not change.
"""

LeanInfo = provider(
    doc = "Transitive Lean sources for a lean_library. Placeholder until buildfiji-4b0.2 adds compiled .olean/.ilean outputs.",
    fields = {
        "srcs": "depset of File: this target's own .lean sources plus its deps', transitively",
    },
)

def _lean_library_impl(ctx):
    srcs = depset(
        direct = ctx.files.srcs,
        transitive = [dep[LeanInfo].srcs for dep in ctx.attr.deps],
    )
    return [
        LeanInfo(srcs = srcs),
        DefaultInfo(files = depset(ctx.files.srcs)),
    ]

lean_library = rule(
    implementation = _lean_library_impl,
    attrs = {
        "srcs": attr.label_list(
            allow_files = [".lean"],
            doc = "This target's own .lean source files.",
        ),
        "deps": attr.label_list(
            providers = [LeanInfo],
            doc = "Other lean_library targets this one imports.",
        ),
    },
    doc = "Placeholder lean_library: tracks sources and deps only, no compilation (buildfiji-4b0.2 adds that).",
)
