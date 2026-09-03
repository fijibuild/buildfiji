# Starlark, loading and bzlmod

## Compatibility bar
- Parse and evaluate all Starlark that Bazel accepts, including Bazel
  dialect quirks (no `def` in BUILD, `load()` semantics, `native`).
- Bazel builtins: `rule`, `aspect`, `provider`, `attr.*`, `select`,
  `configuration_field`, `exec_group`, `toolchain_type`, `ctx.actions.*`,
  `ctx.actions.args()`, `depset`, `struct`, `json`, `proto`, `visibility`,
  `package_group`, `glob`, `exports_files`, `licenses`, `Label`.
- bzlmod: `MODULE.bazel`, `bazel_dep`, `use_extension`, `use_repo`,
  `single_version_override`, registries (BCR), `MODULE.bazel.lock`,
  module extensions, repository rules (`repository_ctx`).
- Legacy `WORKSPACE` is out of scope (Bazel 9 removes it).

## Parser performance (decided 2026-09-03)

Decision: keep the `starlark` crate's front end. No regal-based lexer, no
hand-written parser. Parsing is not, and on this evidence cannot become, the
bottleneck of the loading phase; the regal option stays on the shelf and is
reopened only if a profile of a real fjfj load shows parsing above ~20% of
loading wall time.

Spike (`crates/fjfj-spike-starlark-parse` at commit SPIKE_COMMIT, removed
afterwards; bead buildfiji-mum.1). Corpora are blobless sparse clones of
whole repos (`fixtures/fetch.sh`), every `BUILD`, `BUILD.bazel`, `*.bzl`,
`*.star` and `MODULE.bazel` in the tree. Bazel 9.2.0-era sources, starlark
crate 0.14.2, `-c opt`, Apple M1 Max (8 performance + 2 efficiency cores),
best of 5:

| corpus | files | MB | read | lex | parse | parse, 10 threads | AST/source | parse share of load |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| envoy | 1,982 | 3.9 | 33 MB/s | 299 MB/s | 154 MB/s | 619 MB/s | 7.0x | 39% |
| tensorflow | 1,730 | 13.6 | 168 MB/s | 323 MB/s | 166 MB/s | 779 MB/s | 6.2x | 42% |
| bazel | 723 | 3.3 | 74 MB/s | 314 MB/s | 162 MB/s | 670 MB/s | 6.5x | 37% |
| tensorflow x10 (chromium scale) | 17,300 | 136.5 | 225 MB/s | 324 MB/s | 166 MB/s | 1,095 MB/s | 6.4x | 42% |

Reading it as wall time: the largest Starlark tree in open source, all of
TensorFlow, parses in 82 ms on one core and 18 ms on ten. A synthetic
chromium-scale tree of 17,300 files and 136 MB — no open source Bazel repo is
close — is 0.82 s on one core and 0.12 s on ten. Per file, parsing is 15 us at
the median and 1.5 ms for the largest file in any corpus (a 232 KB
`BUILD`), so a `--watchfs` edit reparses its file in microseconds and
incremental relexing has nothing to save.

Three measurements say the parser is the wrong thing to optimise:

- **Lexing is half of parse** (51% in every corpus). An infinitely fast lexer
  — the entire regal proposition — at best doubles parse throughput, which is
  25 ms on the largest real repo and 60 ms at chromium scale.
- **Reading the tree costs more than parsing it.** envoy's 1,982 mostly small
  files take 120 ms to read from a warm page cache against 25 ms to parse;
  bazel's take 44 ms against 20 ms. I/O and the package machinery around it
  are where loading time goes, and both parallelise.
- **Evaluation already outweighs parsing**, at 58% to 63% of load, and that is
  a floor: the spike binds every Bazel builtin to a no-op stub, so it charges
  evaluation for pure Starlark work only and nothing for globbing, rule
  instantiation, providers or depsets. Real builtins only shrink the parse
  share further.

Parsing scales to 6.6x on 10 cores at chromium scale (4x on the smaller
corpora, where the whole job is 25 ms and thread startup dominates), which is
what matters for Skymeld's parallel loading.

The number that does deserve attention is memory, not speed: a retained AST
costs 6.2x to 7.0x its source, 870 MB of RSS for the chromium-scale tree.
`Evaluator::eval_module` consumes the `AstModule`, so nothing forces fjfj to
keep one; incremental reload should cache the source text and the evaluated
module, never the syntax tree (buildfiji-mum.20).

Dialect findings from the same run, for buildfiji-mum.2: with
`enable_keyword_only_arguments` on (Bazel's own `@_builtins` and TensorFlow
use `def f(*, x)`; `Dialect::Standard` rejects it, so `bzl_dialect()` now
enables it), 100% of the Starlark in all three repos parses — 4,428 of 4,435
files, the 7 exceptions being Bazel's Java-interpreter test data, which are
`---`-separated chunk files rather than Starlark modules. `Dialect::Standard`
still differs from Bazel on `enable_lambda` and `enable_load_reexport`, both
of which Bazel forbids.

## Test strategy
Run Bazel's own Starlark test corpus and the `starlark-spec` test files;
run `rules_rust`, `rules_go`, `rules_python`, `aspect_bazel_lib` loading
phase as integration tests and diff `fjfj query` against `bazel query`.

## No native modules (decision 2026-09-03)

fjfj implements no native Starlark modules. `cc_common`, `java_common`,
`proto_common`, `apple_common`, `platform_common` and `coverage_common` are
written in Starlark, either shipped by fjfj as a builtins overlay (the
equivalent of Bazel's `@_builtins`) or provided by the rules themselves.
Rust implements only the core language plus the rule, aspect, provider,
`ctx`, `Args`, depset, transition, toolchain and exec-group primitives, and
those must be complete enough to express the native modules in Starlark.

## Label character rules and path representation (decision 2026-09-03)

Package names and target names follow Bazel's own `LabelValidator`
exactly (`fjfj_graph::label`), not a plausible-looking approximation, and
the two are asymmetric on purpose: a package name is ASCII-only, but a
target name additionally allows any non-ASCII character — Bazel treats
every code point above U+007F as automatically legal in a target name, so
a source file with a non-ASCII name (a common case for localized test
fixtures) is not an edge case to reject (buildfiji-mum.18).

Once real filesystem code exists (globbing, package loading, the
execroot), paths are `PathBuf`/`OsString`, never `String`: a Unix path is
an arbitrary byte sequence with no UTF-8 guarantee, and Windows paths
carrying more than `MAX_PATH` (260 UTF-16 units) need the `\\?\`
extended-length prefix, which only applies to well-formed `OsString`
paths, not to a `String` that has already lost the platform's native
encoding. Label validation (above) stays on `&str`, since labels are
Bazel-language identifiers with a defined character set, not filesystem
paths.
