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

Spike (`crates/fjfj-spike-starlark-parse` at commit fd5f11f, removed
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

## bzlmod: module resolution (implemented 2026-09-03, buildfiji-mum.6)

`crates/fjfj-bzlmod` evaluates `MODULE.bazel`, walks out to the whole
dependency graph, and runs Minimal Version Selection over it. The
algorithms are ports of Bazel 9.2.0's `ModuleFileGlobals`, `Discovery`,
`Selection`, `Version` and `IndexRegistry`, not reconstructions from the
documentation: a resolution that differs from Bazel's by one version is a
different build.

Spec: `spec/Fjfj/Bzlmod.lean`. Out of scope here and tracked separately:
running module extensions and repository rules (buildfiji-mum.8),
`MODULE.bazel.lock` (buildfiji-mum.7), and the apparent-name half of repo
mapping (buildfiji-mum.15).

### Compatibility levels are gone, and selection is simpler for it

Bazel 9.2.0 accepts `compatibility_level` on `module()` and
`max_compatibility_level` on `bazel_dep()`, warns that they are no-ops,
and then hard-codes every module's level to 0 —
`ModuleFileGlobals.module` calls `setCompatibilityLevel(0)`
unconditionally, whatever the argument said.

That collapses a large part of `Selection.java`. With one level, a
`DepSpec` has exactly one candidate version, so Bazel's search over
combinations of candidates (`enumerateStrategies`, a cartesian product of
per-edge choices, retried until a walk succeeds) always has exactly one
element. fjfj evaluates that single strategy directly. Selection groups,
the "snap up to the nearest allowed version" rule for
`multiple_version_override`, and the walk's error checks are all ported
as they stand; only the search around them is absent.

The observable consequence is a theorem rather than a comment:
`Fjfj.Bzlmod.one_version_per_name` says two modules in the resolved graph
with the same name are the same module. That is what makes an apparent
repo name unambiguous, and it is precisely what a
`multiple_version_override` opts out of. If compatibility levels ever come
back, the choice would key on `(name, level)`, the theorem would fail, and
the search would have to come back with it.

### The `MODULE.bazel` dialect

A module file is a declaration, not a program. Bazel enforces that with
`DotBazelFileSyntaxChecker`; fjfj gets the same result from the parser by
turning the features off in the dialect, so a rejected file is rejected at
parse time with a location:

| Setting | Why |
|---|---|
| `enable_def: false` | no functions in a module file |
| `enable_lambda: false` | Bazel has no `lambda` anywhere |
| `enable_load: false` | `include()` is the only way to pull in another file, and it is checked syntactically |
| `enable_top_level_stmt: false` | no top-level `if`/`for`, as in every `.bazel` file |

`print` is a Bazel builtin but a starlark-crate *extension*, so it has to
be added explicitly — real module files use it (bazel_gazelle's prints a
warning). For a dependency it is wired to a discarding handler, as Bazel
does with `printIsNoop`: a module from a registry must not be able to spam
the console during resolution.

### One flag behind two documented rules

"A dependency's dev dependencies don't affect your build" and "a
dependency's overrides are ignored" read as two features. In Bazel they
are one flag: `ignoreDevDeps`, set for every non-root module, which
`ModuleThreadContext.addOverride` checks before recording anything. fjfj
keeps them as one flag (`EvalOptions::ignore_dev_deps`) for the same
reason — splitting them would be an invitation for the two to drift.

### Registry client

A Bazel registry is an index of files under a base URL, so the client is
URL construction, JSON parsing and integrity checking. Transport is behind
a `Fetcher` trait because a `file://` registry is a first-class case: it
is how the BCR's own tests run and how the fixtures below work.

HTTP is `reqwest` with rustls. One HTTP stack for the whole tool, since
repository rules need the same downloader (`repository_ctx.download`,
`http_archive`, buildfiji-mum.8) — a second client for that would be two
sets of proxy, redirect and TLS behaviour to keep in step. It builds under
Bazel unmodified; `aws-lc-sys` compiles through `crate_universe` with no
annotation, taking about 80 seconds once.

### `bazel_tools` is a placeholder

Every module implicitly depends on `bazel_tools`, which Bazel ships inside
its own binary rather than serving from a registry. fjfj has no embedded
tools repository yet, so `RegistrySource` supplies a `bazel_tools` module
file with no dependencies (buildfiji-mum.23). This is invisible to
`fjfj mod graph`, which hides the `bazel_tools` subtree as Bazel does, but
it is not invisible to resolution: Bazel's real `bazel_tools` has
`bazel_dep`s of its own, and they raise selected versions elsewhere in the
graph. Feeding fjfj the real file makes the difference disappear (below).

### Conformance method

The fixtures under `crates/fjfj-bzlmod/tests/fixtures` are a local module
registry and one workspace per resolution scenario — MVS, pruning of a
module that lost its only dependent, both override kinds, fulfilled and
unfulfilled nodep edges, a yanked version. The expected result of each is
**Bazel's own output**, captured by
`bazel run //crates/fjfj-bzlmod/tests/fixtures:refresh_golden` and
committed, so the test compares against Bazel rather than against a
restatement of the implementation.

Two ignored tests reach the network, run by hand: one reads real modules,
`source.json` and `metadata.json` from `bcr.bazel.build`, and one resolves
this repository's own `MODULE.bazel` against it. On the run that closed
buildfiji-mum.6, the second produced the same selected version as
`bazel mod graph` for all 29 modules Bazel reports for this repository —
including the ones where the answer is not the obvious one, such as
protobuf 33.4 winning over the 29.1 that `rules_proto` and `rules_python`
ask for. That match requires the real `bazel_tools` module file; with the
placeholder, protobuf resolves to 29.1 instead, which is the clearest
statement of why buildfiji-mum.23 matters.

### `include()` runs inline, in the same evaluation

`include()` pulls more directives in from another file, and Bazel's own
`ModuleFileFunction.execModuleFile` runs them in the *same*
`ModuleThreadContext` as the including file, at the call site — not merged
in before or after. fjfj gets that literally: the `include()` builtin
(eval.rs) makes a second, nested `Evaluator::eval_module` call against the
same `Evaluator` and the same `ModuleContext`, so a `bazel_dep` inside an
include lands exactly where it would if the included text had been pasted
in place. This works because `Evaluator::eval_module` is designed to be
re-entrant (it saves and restores `module_def_info` around the call); nesting
it from inside one of its own builtins is unusual but not unsupported.

A label is validated before it is resolved: it must be repo-relative
(start with `//`), and its basename must be a real `*.MODULE.bazel`
file that doesn't start with a dot. `include()` is refused outright in a
registry module — only the root module and a module with a non-registry
override may call it — and a self- or mutually-including cycle is caught
by an explicit stack (`ModuleContext::include_stack`), since Bazel relies
on Skyframe's cycle detector for that and fjfj has no equivalent yet.

Resolving a label to text is the caller's job (`eval::IncludeSource`),
since eval.rs has no filesystem access. `resolve()` wires this up for the
root module only, via `WorkspaceIncludeSource`, which reads the label
relative to the workspace directory the root `MODULE.bazel` came from.
`include()` inside a non-registry-overridden dependency validates and
permits, but has no source configured yet — resolving one needs the
override's contents fetched first, which is buildfiji-mum.8 territory.

Conformance: `tests/fixtures/workspaces/include` — real Bazel needs a
`BUILD.bazel` in a workspace before `//:foo.MODULE.bazel` resolves at all,
even for the *root* package, so the fixture carries an empty one.

### Discovery fetches one horizon concurrently (buildfiji-mum.24)

`discover_round` (discovery.rs) already computes a horizon — every module
key newly reachable this round — as a batch before fetching any of them;
the only change here is fetching that batch with one OS thread per key
(`std::thread::scope`) instead of a loop. `ModuleFileSource` gained a
`Sync` supertrait bound for it, which cost nothing to satisfy:
`RegistrySource`'s own `Fetcher` trait was already `Send + Sync`
(`reqwest::blocking::Client` is both).

No async runtime involved — this crate doesn't depend on tokio, and
plain OS threads are the right tool for a handful of blocking HTTP calls
per round, not a scheduler. `apply` (the override-rewriting closure
`read_module` takes) only borrows `root.module.name` and `overrides`, so
it's `Copy`, and each spawned thread gets its own.

Measured on this repository's own `MODULE.bazel` against the real BCR
(`resolves_this_repository_against_the_real_registry`, ignored by default
— reaches the network): 10.05s sequential → 2.74s concurrent, a real
resolution with several rounds and multiple modules per round, not a
synthetic worst case. `discovery::tests::
one_horizons_module_files_are_fetched_concurrently` pins the mechanism
itself down with a fake source that records the highest number of
`module_file` calls it ever saw in flight — a sleep-and-count probe, not
a timing assertion, so it can't flake on a loaded CI box.
