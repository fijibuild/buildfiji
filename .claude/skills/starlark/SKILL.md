---
name: starlark
description: How fjfj implements Starlark and the Bazel dialect — the starlark crate is the front end (no custom lexer or parser), what Rust may and may not implement natively, BUILD versus .bzl dialect settings, and the AST memory rule. Use when working on fjfj-starlark, the loading phase, Bazel builtins, or anything that parses or evaluates Starlark.
---

# Starlark in fjfj

Design doc: `docs/design/starlark-and-loading.md`. Epic: `buildfiji-mum`.

## Front end: the `starlark` crate, settled

fjfj uses the `starlark` crate (Buck2's implementation) for lexing, parsing
and evaluation. A custom lexer (regal) and hand-written parser were measured
against envoy, tensorflow, bazel and a 136 MB synthetic tree and **rejected**:
parsing runs at 154-166 MB/s on one core and over 1 GB/s on ten, lexing is
only 51% of that, and reading the tree costs more than parsing it on
small-file repos. Do not re-propose a custom front end unless a profile of a
real fjfj load puts parsing above 20% of loading wall time.

## No native Starlark modules

`cc_common`, `java_common`, `proto_common`, `apple_common`, `platform_common`
and `coverage_common` are written **in Starlark**, either as an fjfj builtins
overlay (the equivalent of Bazel's `@_builtins`) or by the rules themselves.

Rust implements only the core language plus the `rule`, `aspect`, `provider`,
`ctx`, `Args`, depset, transition, toolchain and exec-group primitives — and
those must be complete enough to express the native modules in Starlark.

## Dialect

`fjfj_starlark::build_dialect()` and `bzl_dialect()` are the source of truth;
`Dialect::Standard` from the crate is **not** Bazel's dialect. Known deltas:

- BUILD files: no `def`, no `lambda`.
- `.bzl`: `enable_keyword_only_arguments` must be on — Bazel's own
  `@_builtins` and rule sets use `def f(*, x)`.
- `.bzl` accepts `lambda`, keyword-only parameters and type annotations
  (parsed, ignored at runtime); BUILD files reject `def`, `lambda`,
  annotations and top-level `if`/`for`. Neither accepts positional-only `/`
  or f-strings. `load()` bindings are private (`enable_load_reexport` off),
  a `_`-prefixed symbol cannot be loaded, and in a `.bzl` loads must come
  before other statements and no top-level name is declared twice. Use
  `fjfj_starlark::parse(path, src, FileKind)`, not `AstModule::parse`, so
  those file-level checks run. Verified against Bazel 9.2.0 (`buildfiji-mum.2`).

When a corpus file fails to parse, check the dialect before the parser.

## BUILD evaluation and `native.*`

`fjfj_starlark::evaluate_build_file` turns a BUILD file into a
`fjfj_graph::package::Package`. `build_globals()` puts `glob`, `package`,
`filegroup` and the rest directly in scope; a loader that evaluates `.bzl`
files must use `bzl_globals()`, which has them only under `native`. Fatal
errors stop the file; *events* (attribute errors) are recorded and fail the
package at the end. Every message is copied from Bazel 9.2.0 — probe before
changing one. Design: `docs/design/starlark-and-loading.md`.

## Memory: never retain an AST

A retained `AstModule` costs 6-7x its source text — 870 MB for a
chromium-scale tree. `Evaluator::eval_module` consumes the AST, so keep it
that way: caches hold the source text and the frozen module, never the syntax
tree (`buildfiji-mum.20`).

## Compatibility bar

Bazel 9.2.0 observable behaviour is the spec: dialect quirks, builtins,
`load()` semantics, `native.*`, bzlmod (`MODULE.bazel`, module extensions,
repository rules). Legacy `WORKSPACE` is out of scope — Bazel 9 removes it.
