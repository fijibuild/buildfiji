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
still differed from Bazel on `enable_lambda` and `enable_load_reexport`.

buildfiji-mum.2 settled both by running Bazel 9.2.0 rather than trusting the
spike: `load_reexport` is off in both dialects, but `lambda` is **on** in
`.bzl` (only BUILD files reject it, as a "function"). It also found that
`.bzl` parses type annotations, requires `load()` before every other
statement, and rejects a top-level name declared twice; BUILD files allow
all of those. `fjfj_starlark::parse` enforces the file-level rules the
starlark crate does not.

## Test strategy
Run Bazel's own Starlark test corpus and the `starlark-spec` test files;
run `rules_rust`, `rules_go`, `rules_python`, `aspect_bazel_lib` loading
phase as integration tests and diff `fjfj query` against `bazel query`.

**Done (buildfiji-mum.10):** Bazel's Starlark script tests (38 files,
`crates/fjfj-starlark/testdata/bazel-starlark`, copied from the 9.2.0 tag)
run in `conformance.rs` the way its `ScriptTest.java` runs them (chunks
separated by `---`, `### regexp` for an expected error, `assert_`,
`assert_eq`, `assert_fails`). 67 of about 600 chunks fail, all listed in
`testdata/conformance_known.txt` with the bead that owns the difference;
the test fails if a chunk not listed starts to fail and if a listed one
starts to pass, so the list only shrinks. Most are the crate's wording of an
error (buildfiji-v32); the real differences are recursion (buildfiji-2r5),
a huge repeat that panics (buildfiji-gpj, run by nothing: it takes minutes),
`split(sep=...)` (buildfiji-wtt), `elems()` (buildfiji-9zq) and cyclic reprs
(buildfiji-sib). The `starlark-spec` test suite is not run yet.

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

## Packages, boundaries and visibility (implemented 2026-09-30, buildfiji-mum.5)

Split by I/O. `fjfj-graph` holds the pure part (`Label::parse`,
`package::{Package, PackageBuilder}`, `visibility::*`); `fjfj-loading`
holds `PackageLookup`, the only code that asks the filesystem what a package
is. The builder takes "is this a package?" as a predicate, which is how the
two meet. Every rule below was read off Bazel 9.2.0, and each unit test
names the behaviour it copies.

- **What is a package.** A directory with a `BUILD.bazel` or `BUILD` *file*;
  `BUILD.bazel` wins. A directory of either name does not count and the
  search falls through; a symlink to a file counts, a dangling one does not.
- **What removes one.** `--deleted_packages` removes exactly the packages
  named. `.bazelignore` removes a directory's whole subtree: no wildcards, no
  trimming, `c/` and `./c` normalise, `..` entries match nothing, an absolute
  path is an error. Both report "Package is considered deleted due to
  --deleted_packages", which is what Bazel says for `.bazelignore` too.
  `REPO.bazel`'s `ignore_directories()` is a third source and is not done
  (buildfiji-e4r).
- **Subpackage crossing.** A target name inside package `a` may not walk into
  a subpackage: `//a:sub/f.txt` fails when `a/sub` is a package, naming the
  *deepest* such package and suggesting `//a/sub:f.txt`. Only names in the
  package being loaded are checked; `//a:sub/f.txt` written in package `b`
  loads and fails later as a missing target. The check applies to rule,
  `exports_files` and `package_group` names alike.
- **Declaring names.** Two targets of one name conflict, and the message
  names both kinds and the first's location. `package()` may be called once,
  anywhere in the file, and its `default_visibility` covers targets declared
  before it. `exports_files` of an already exported file is fine unless it
  gives `visibility`, which is "declared twice"; an exported file with no
  `visibility` is public. Source files that are not exported are not targets.
- **Label parsing.** `Label::parse` takes a string as a BUILD file writes it,
  in a package context: `:x`, `x`, `//p`, `//p:q`, `@r`, `@r//p:q`, `@@r//p`,
  `@//p`. `//p` is `//p:p`, `@r` is `@r//:r`, whitespace is kept, and a
  relative `p:q` is an error. Error text is Bazel's, minus the "in element 0
  of attribute" context the caller adds. `Label::parse` takes `@r` as a
  canonical name; `Label::parse_mapped` takes a repo mapping function and
  treats `@r` as apparent (`@@r` never is), which is what `Label()` uses
  (buildfiji-mum.3.2). Bazel finds what else is wrong with a label before it
  says the label is not absolute (`a:b:c` is a bad target name), drops a
  trailing `/.` from a target name, refuses a bare `...` or `a/...` as if it
  were a package, and quotes control characters in a name as `<?>` (a carriage
  return as `\r`).
- **Visibility.** A target is visible to its own package always, to others
  through its `visibility` else the package's `default_visibility`, else not
  at all. Entries are `//visibility:public`, `//visibility:private` (grants
  nothing, may sit beside others), `//p:__pkg__`, `//p:__subpackages__` and
  `package_group` labels. A group's specs come from its `packages` and,
  transitively, its `includes`; a `-` spec denies wherever it is written,
  including from an included group; `//p:__pkg__` and `//p` are the same;
  `public` and `//...` match everything (every repo, and one repo).
- **Not here.** Enforcing visibility on a dependency edge is an analysis
  step (buildfiji-8sq); load visibility reuses these types (buildfiji-ps4).

## `native.*` for BUILD files (implemented 2026-09-30, buildfiji-mum.4)

Three crates, split by I/O again. `fjfj-loading::glob` walks the tree;
`fjfj-graph::rule` holds the attribute schema of the two rules Bazel builds
in that need no Starlark (`filegroup`, `alias`) and `AttrValue`;
`fjfj-starlark::native` binds all of it to Starlark and drives
`PackageBuilder`. `evaluate_build_file` is the entry point: it returns the
`Package` and what `print()` wrote, or the events that failed the package.
Every rule below was read off Bazel 9.2.0, and each test names what it copies.

- **Where the functions live.** A BUILD file has `glob`, `package`,
  `package_group`, `exports_files`, `existing_rule(s)`, `filegroup`, `alias`,
  `package_name`, `repository_name` and `repo_name` as globals and has no
  `native`. A `.bzl` has them only as `native.x` (`bzl_globals()`), and
  calling one while a `.bzl` *loads* is an error ("can only be used while
  evaluating a BUILD file or a legacy macro"), while calling one from a macro
  a BUILD file invokes works. The mechanism is `Evaluator::extra`: the BUILD
  file's evaluator carries the context and a loading `.bzl`'s does not.
- **Two kinds of error.** A *fatal* error stops the file with a traceback:
  bad arguments to `glob`, `package`, `package_group`, `exports_files`, a
  name conflict, an illegal rule name. An *event* is recorded and the file
  goes on, so one run reports several: unknown or mistyped rule attributes, a
  missing mandatory one, a duplicate label, a bad `package_group` spec. Events
  fail the package at the end. A rule with attribute errors is still declared
  (a later rule of the same name conflicts with it), with the bad attributes
  taken as unset.
- **glob.** Sorted by bytes, no duplicates. `*` matches within a segment,
  `**` is a whole segment of zero or more directories, `?` is an error, and
  `[a]` and `{a,b}` are literal. A dot-file is matched by a segment that is
  exactly `*` or starts with a literal `.`, and by no other that starts with
  a wildcard, so `*` finds `.hidden.txt` and `*.txt` does not. Directories
  are dropped by default (`exclude_directories = 1`; any int counts, a bool is
  a type error), and `sub/**` lists `sub` itself only when they are kept. A
  directory that is a package is invisible: not listed, not entered. A
  `.bazelignore`d one is listed but not entered, and a `--deleted_packages`
  one is ordinary. Symlinks are followed; a dangling one is absent; one that
  leads back to a directory being walked fails the package, and `ELOOP` from
  a two-link cycle is an I/O error. With `allow_empty = False` (the default in
  Bazel 9) each `include` pattern must match something, naming the first that
  does not, and something must survive `exclude`. `exclude` is looser: a
  pattern without `*` or `?` is compared to whole paths as a string (so `sub`
  does not exclude `sub/x`), one with a wildcard is matched segment-wise with
  `?` as any character and is checked for empty segments and misplaced `**`.
- **Arguments.** Bazel's natives do not word argument errors alike, so
  `bind` takes the wording per function: `glob`, `exports_files` and
  `existing_rule` use a declared signature ("accepts no more than 4 positional
  arguments", "got unexpected keyword argument"), `package` its own
  ("unexpected keyword argument: x"), `package_group` a third. Sequence
  parameters take a list or tuple and reject a string. `package()` accepts
  `default_visibility`, `default_testonly`, `default_deprecation`,
  `default_compatible_with`, `default_restricted_to`, `default_hdrs_check`,
  `licenses`, `default_applicable_licenses`, `default_package_metadata` and
  `features`, and setting both metadata spellings is an error.
- **Rule attributes.** `filegroup` takes `srcs`, `data`, `output_group`,
  `output_licenses`, `licenses`, `distribs` and the common ones; `alias`
  takes `actual` (mandatory, one label) and the common ones but not `srcs`,
  `data`, `licenses` or `distribs`. `None` means "as if unset" for every
  attribute, even `actual`. Every label list except `visibility` rejects a
  duplicate, however it was spelled. An unknown attribute gets Bazel's "did
  you mean" when an attribute is within a third of its length in edit distance
  (`rule::suggest`, fitted to two dozen probed spellings).
- **existing_rule(s).** A dict of what the rule set plus its class's
  defaults, in Bazel's attribute order: lists as tuples, labels relative to
  the package (`:x`, `//p:x`, `@@r//p:x`). Attributes whose default is unset
  (`testonly`, `deprecation`, ...) are absent until set, and `package()`'s
  defaults never appear. Only rules are listed: not exported files, not
  package groups. **Deviation:** Bazel returns a live read-only `Map` view;
  fjfj returns a snapshot dict (buildfiji-ie2).
- **Locations.** `file:line:col` where the column is the call's `(`, which is
  what Bazel prints, and conflict messages quote it.
- **Not here.** The native rules other
  than `filegroup` and `alias` (buildfiji-136.10), and `subpackages`,
  `package_default_visibility`, `module_name` and `module_version`
  (buildfiji-hrx). `package_relative_label` is in `label.rs` (below).

## `depset` (implemented 2026-09-30, buildfiji-mum.14)

`fjfj-starlark::depset`: a `StarlarkValue` in both heaps (`Depset` and
`FrozenDepset`, one generic `DepsetGen<V>`), and the `depset()` global in BUILD
and `.bzl` files. The flattening rules are Bazel's `NestedSet`, read off Bazel
9.2.0 by fitting a model to about 150 probes (`depset_tests.rs` replays them
all, generated from the probe runs). The docs are loose about them, and the
model is not obvious:

- **Layout.** Building a set lays out its children once, by *its own* order:
  `default` and `postorder` put transitive sets before direct items,
  `preorder` puts direct items first, `topological` is the reverse of the
  `preorder` layout (transitive sets last-to-first, then items last-to-first).
  Direct items are de-duplicated (first wins) at that point.
- **Flattening.** `to_list` is one depth-first walk over those layouts,
  whatever the orders of the sets it passes through: first occurrence of each
  element wins, a set already entered is never entered again, and the result
  is reversed if the *root* is topological. That single rule reproduces all
  four orders, including the odd mixed cases: a `default` set holding a
  `topological` one shows that one's own layout (its items reversed), and a
  `topological` root holding `default` sets reverses them.
- **Orders.** `default` is compatible with anything and with itself; two
  other orders only with themselves ("Order 'postorder' is incompatible with
  order 'preorder'"), checked between a set and each transitive set, not
  between siblings. The result keeps the set's own order, so a `default` set
  over `postorder` ones is `default`. `stable`, `compile`, `link` and
  `naive_link` are gone in Bazel 9.
- **Identity.** A set with no direct items and exactly one non-empty
  transitive set of the same order *is* that set (`depset(transitive = [a])
  == a`); empty transitive sets are ignored. Otherwise `==` is identity,
  except that all empty depsets of one order are equal. Depsets hash (so they
  key dicts) but do not order, add, iterate, index, or have a length.
- **Elements.** Hashable, all of one type (`bool` is not `int`), `==` decides
  duplicates. Hashable stands in for Bazel's "not mutable"; builtin functions
  pass here and not there (buildfiji-ahp). A depset may hold depsets.
- **Errors.** Bazel's wording for the constructor (`depset elements must not
  be mutable values`, `cannot add an item of type 'string' to a depset of
  'int'`, `at index 0 of transitive, got element of type int, want depset`,
  parameter type errors, `Invalid order: x`), the operator errors
  (`unsupported binary operation: depset + depset`, `unsupported comparison`),
  and `len()`. How iterating, indexing, attributes and method arity fail is
  the Starlark runtime's own wording, listed in buildfiji-v32.
- **Cost.** A node is two vectors and three flags; building is
  O(direct + number of transitive sets), never the size below. Flattening and
  `repr` walk with an explicit stack and a visited set, so depth costs no
  stack and a DAG with 2^60 paths is walked once (both tested, at 100,000
  levels and 60). Freezing recurses through the `starlark` crate at about 3 KiB
  a level, so `Freeze` grows the stack (`stacker`) instead of capping a chain:
  a 50,000-level chain freezes.
- **Not here.** `ctx.actions` and providers that carry depsets (buildfiji-136),
  `Args` and `depset.to_list` on values from `.bzl` builtins overlays
  (buildfiji-136.15), and a memory benchmark against a real tree
  (buildfiji-mum.19).

## `struct`, `json`, `proto` and `set` (implemented 2026-09-30, buildfiji-mum.3.1)

Each was replayed against Bazel 9.2.0 (~570 probes, tables in
`builtins_tests.rs`), and the `starlark` crate's own `set` and `struct` were
not close enough to keep:

- **struct**: fields are sorted, repr is `struct(a = 1)`, `+` joins two and
  refuses a shared field, and it is hashable only if every field is.
- **json**: the messages carry a path (`in struct field .a: at list index 0:
  ...`); `decode` reports byte offsets and takes any value as a candidate
  object key. `indent` is a lenient reformatter, not a validator (it stops
  after the first complete value and copies `true`/`false`/`null` by length);
  where Bazel crashes on it (`json.indent("t")`) fjfj reports an error.
- **proto.encode_text** takes only a struct; dicts are repeated `key`/`value`
  messages; only `"`, `\` and newline are escaped.
- **set**: `pop()` takes the first element, the variadic methods
  (`update union intersection difference` and the `_update` forms) take any
  number of collections, and a frozen or iterated set refuses every mutator,
  even a no-op one.

Known differences, each with its bead: floats and non-ASCII strings print
as the `starlark` crate does in `repr` (buildfiji-s9u; `json` and `proto`
format floats the Bazel way themselves); `s |= t` and its siblings rebind
rather than mutate (buildfiji-tg2); the generic runtime wording for
operators, iteration, attribute lookup and `hash()` on these types
(buildfiji-v32); `\u`, `\U` and `\x` string escapes are accepted where
Bazel rejects them (buildfiji-8q5).

## `Label` and repository mapping (implemented 2026-09-30, buildfiji-mum.3.2)

`Label` is a value type in `label.rs`, replayed against Bazel 9.2.0 (about
140 probes in `label_tests.rs`, and the dependency-module cases in the
tests beside them and in `native.rs`).

- **What a label is.** `str` is `@@repo//pkg:name` (`@@//pkg:name` in the main
  repo), `repr` is `Label("//pkg:name")` in the main repo and
  `Label("@@repo//pkg:name")` elsewhere, and labels are equal, hashable
  and ordered by (repo, package, name). `Label(label)` is the label. The
  members are `name package repo_name workspace_name workspace_root` and the
  methods `relative(s)` and `same_package_label(name)`. All three parameters
  are positional-only.
- **A name the mapping lacks is not an error.** `Label("@r//a:b")` is a label
  in the repo `[unknown repo 'r' requested from @@]`, with a `(did you mean
  'x'?)` when a mapped name is close. It fails only when `repo_name`,
  `workspace_name` or `workspace_root` is read (`'repo_name' is not allowed on
  invalid Label ...`), and `hasattr` still says it has them.
- **The caller's file decides.** `Label(s)` and `label.relative(s)` read `s`
  in the `.bzl` whose code makes the call, not the BUILD file being loaded:
  `:x` is in that file's package, and `@r` goes through that file's repo's
  mapping. A native function cannot ask which module defined its caller, so a
  `.bzl` is *evaluated under its canonical label as its file name*
  (`@@dep+//sub:m.bzl`) and the call's frame says where it is. Use
  `evaluate_bzl` to evaluate one; it also puts the mappings where `Label` can
  find them. `native.package_relative_label` reads in the BUILD package being
  loaded, through its repo's mapping.
- **Repository mapping is a table.** `RepoMappings` is `repo -> (apparent ->
  canonical)`. buildfiji-mum.15 fills it from the module graph, and nothing
  in `Label` changes when it does: the main repo's table has `""` and its own
  module name mapping to itself (`@//a:b`, `@probe//a:b`), a dependency's has
  its own name and its `bazel_dep` names, and `@` alone is unknown in a
  dependency. The BUILD-side entry point, `BuildFile`, takes the same table.
- **Known differences.** `print(label)` writes `str`, where Bazel writes the
  main repo's display form (`//a:b`, `@dep//a:b`; buildfiji-xq5). The crate's
  `%s` and unnumbered `{}` use `repr` for non-strings, so `"%s" % label` is
  `Label("//a:b")` where Bazel gives `@@//a:b` (buildfiji-b9c). A label passed
  around and called as `L("...")` reads in the file that *calls* it, as
  Bazel does.

## `attr.*` (implemented 2026-09-30, buildfiji-mum.3.3)

`attr` is a value of type `attr` in `attr.rs` (`.bzl` only), with the fourteen
builders as methods. Each returns an `Attribute` descriptor, which prints as
`<attr.string>`, has no members and is not hashable. About 3,100 probes of
Bazel 9.2.0 are replayed: every keyword each builder takes against sixteen
values of every type (`attr_matrix.rs`), and the conversions, orderings and
equalities (`attr_tests.rs`).

- **Which keywords.** A table per builder (`BOOL`, `LABEL`, ...) says which
  keywords it takes and which are positional (`int_list` and `string_list`
  take `mandatory, allow_empty`; the other list and dict builders and
  `output_list` take `allow_empty`; the rest take none). `materializer` is
  refused whenever it is given (it is behind `--experimental_dormant_deps`),
  and `for_dependency_resolution` takes anything and is kept.
- **Order of errors.** Arguments are checked as they are bound, in the order
  the call writes them (positional first), and a keyword the builder does
  not take is an error at its place in that order; a surplus positional
  argument is reported after the keywords. Only when all are bound is
  anything converted, in this order: `default`, `flags`, `executable` without
  `cfg`, `allow_files`/`allow_single_file` (both given, then the elements),
  `allow_rules`, `providers`, `cfg`, `aspects`. The attribute name in a
  default's message is the builder's own for a label builder and blank for
  the others.
- **What is stored.** The pure data is a `fjfj_graph::rule::AttrDef`, which a
  native rule's `AttrSpec` also converts to: type, the default converted to an
  `AttrValue` (a label is read in the `.bzl` that made the call, `:x` in its
  package and `@r` through its repo's mapping; dicts keep their order), a
  set of `AttrFlag`s, the file types, allowed rule kinds, `cfg`, `configurable`
  and `skip_validations`. `mandatory`, `allow_empty = False`, `executable` and
  `allow_single_file` set the flag of the same meaning, so `flags =
  ["MANDATORY"]` is `mandatory = True`. What needs a Starlark value stays on
  the descriptor: `values` as given, `providers` as alternatives (a flat list
  of providers is one), `aspects`, a transition, and a `default` that is a
  function, which is kept and not called. `attr::view` reads it back for
  `rule()` (buildfiji-mum.3.5).
- **Equality** is Bazel's `Attribute.equals`: by content, except that a value
  list, a suffix list, a configuration other than the target's and a computed
  default are never equal to another descriptor's (only to itself). A default
  left out is the type's zero, dicts compare in any order and lists do not,
  and the alternatives of `providers` compare as sets of sets in order.
- **Known differences.** `attr.label(default=print)` is accepted, as the crate
  calls a builtin a `function` (buildfiji-v32); `1<<31` wraps in the crate
  (buildfiji-sbj). The did-you-mean for a misspelt keyword is a fit, not
  a known rule (`suggest_keyword`, buildfiji-2cb).

## `provider()` (implemented 2026-09-30, buildfiji-mum.3.4)

`provider(doc, *, fields, init)` (`provider.rs`, `.bzl` only) returns a
`Provider`, which prints as `<provider>`, has no members, hashes by identity
and equals only itself. About 360 probes of Bazel 9.2.0 are replayed in
`provider_tests.rs`, and the hand tests there cover identity across freezing
and `load()`.

- **An instance is a struct.** Calling a provider makes a value of type
  `struct` that prints as `struct(a = 1)`, whichever provider made it:
  Bazel's repr does not show the provider's name, and `dir`, `json.encode`
  and `proto.encode_text` treat it as a struct. It is the `structs.rs` value
  with a reference to its provider, so two instances are equal only if the
  providers are the same and the fields are, `+` joins instances of one
  provider and says `Cannot use '+' operator on instances of different
  providers (P and Q)` otherwise (`struct` is a provider too, for that
  message), and a field cannot be assigned. Arguments are keywords only, and
  a field left out is absent, not `None`. `fields`, as a list, tuple, range
  or dict of names, restricts them.
- **`init`.** With `init` the call is `(provider, raw_constructor)`. Calling
  the provider passes the arguments to `init`, whose own argument errors come
  first, and its result must be a dict with string keys (`got dict<int, int>
  for 'return value of provider init()', want dict<string, unknown>`); those
  are the fields. The raw constructor (type `RawConstructor`, printed the way
  Bazel prints a Java class it has no name for) makes an instance directly.
  Both check `fields`.
- **Argument checks** follow `provider()`'s own order: the first positional
  (`doc`) and each keyword are type-checked as bound, an unknown keyword is
  an error at its place, and a surplus positional comes after the keywords.
  Then `fields` is read. The wording is the signature's (`in call to
  provider(), parameter 'doc' got value of type ...`), and the dict-shaped
  errors name the types of the first entry that is wrong.
- **Names.** Bazel names a provider at the assignment that binds it to a
  top-level name, the first such name winning (`Q = provider()` then `P = Q`
  is `Q`), and by value: `P = make()` names what `make()` returned, `P, R =
  provider(init = ...)` names `P`, and one that is only in a list, a dict or
  a struct is never named (`<no name>`). The name is in `got unexpected field
  'b' in call to instantiate provider P`, in `P: unexpected positional
  arguments` (`<raw constructor for P>` for the raw constructor), and in what
  `providers=` and `provides=` accept: an unnamed provider gives `Providers
  should be top-level values in extension files that define them.` The
  `starlark` crate has no hook on an assignment and will not show a native
  function a private name, so `exports::name_at_assignment` reads the name
  off the line being run: a call made by the module itself (not from a
  function) as the whole right side of `NAME = provider(...)` is named `NAME`
  at once, a `_P` included. Anything else (`P, R = provider(init = ...)`, a
  value a function returned) looks for its name among the *public* top-level
  names of the module being evaluated when it needs one, and `evaluate_bzl`
  names the rest once the module is frozen (the names come from the parsed
  file, values from the frozen module).
  A provider is identified by a counter that survives freezing, not by
  address, and its name is a `OnceLock` that freezing carries across.
- **Known differences.** A provider a function made and a `_private` name was
  bound to is anonymous until its module ends, so an error about it *during*
  that evaluation says `<no name>` where Bazel says `_Q`, and
  `providers=[_Q]` is accepted in any module that binds a private name (it
  cannot tell `_Q` from a provider bound to nothing). `provider(init = struct)` is accepted
  (Bazel's `struct` is a `Provider` and not callable; the crate's is a
  function); `provider(fields = ["a", "a"])` is an error here and a crash in
  Bazel, which prints no message. The crate words a bad call to `init`'s
  own function, `x.a = 1`'s field-assignment error on other types, and the
  generic operator errors its own way (buildfiji-v32). `DefaultInfo` and
  the other predeclared providers are buildfiji-136.4's.

## `rule()` and instantiating a rule (implemented 2026-09-30, buildfiji-mum.3.5)

`rule(implementation, *, test, attrs, outputs, executable, ...)` (`rule.rs`,
`.bzl` only, and only while a `.bzl` initialises) returns a `rule`, which
prints as `<rule NAME>` once named and `<rule>` before, hashes by identity
and equals only itself. About 450 probes of `rule()` and 800 of BUILD files
that call rules (and `filegroup` and `alias`, which now take the same path)
are replayed in `rule_tests.rs`.

- **One schema.** A rule class is a `fjfj_graph::schema::RuleSchema`: the
  attributes a call may set, in the order `native.existing_rule` lists them,
  each an `AttrDef` (type, default, flags) with the `values` it may take. A
  native rule's static `RuleClass` and a `rule()`'s `attrs` both become one
  (`RuleSchema::native`, `RuleSchema::starlark`), and `instantiate.rs`, which
  is all that calls a rule, reads nothing else. A Starlark rule's schema is
  the universal attributes (`name`, `visibility`, `tags`, ... with their
  types and defaults, read off Bazel), an executable's `args` and
  `output_licenses`, a test's `size`, `timeout`, `flaky`, `shard_count`,
  `local` and `args` (and a `testonly` that defaults to true), and then the
  rule's own in declaration order. None of those names can be redeclared
  (`attribute `x`: built-in attributes cannot be overridden.`), nor an
  invalid identifier; a `_private` attribute cannot be set by a call or seen
  in `existing_rule`; `applicable_licenses` is an alias of `package_metadata`
  unless the rule declares its own.
- **Naming.** A rule is named like a provider (`exports.rs`: the first
  top-level name it is bound to, by value, found in the running module or at
  its end), a rule that is never named fails with `Invalid rule class hasn't
  been exported by a bzl file`, a test rule must be named `*_test` and no
  other may be, and `rule()` called from a function that runs while a BUILD
  file is loaded is `rule() can only be used during .bzl initialization
  (top-level evaluation)`. Calling a rule while a `.bzl` loads is `a rule can
  only be instantiated while evaluating a BUILD file or a legacy or symbolic
  macro`.
- **Argument checks** of `rule()` are in `provider()`'s order (positional
  `implementation`, each keyword as bound, a surplus positional last) with
  the signature's wording; what is checked afterwards is Bazel's own: the
  names of `attrs`, `provides` (an element must be an exported provider),
  `toolchains`, `exec_compatible_with` and `fragments`. `cfg` takes a
  transition (not a string, not `config.exec()`), `build_setting` a
  `config.*` setting (which adds the mandatory `build_setting_default` and
  `help` after the rule's own attributes, and refuses `cfg`),
  `exec_groups` a dict of `exec_group()`s with identifier names, and
  `subrules` exported subrules (buildfiji-mum.3.7); `analysis_test = True`
  makes a test (its class name must end in `_test`); `parent` only takes its
  empty value.
- **A call** takes keywords only (`Unexpected positional arguments` for a
  Starlark rule, `unexpected positional arguments` for a native one). An
  unknown or private attribute, a value of the wrong type, a label that does
  not parse, a mandatory attribute left out (`None` counts as left out, for a
  native rule too: `alias(actual = None)` is missing its `actual`), and a
  value outside `values` are events
  and the file goes on. A list-typed attribute takes a list, tuple, range,
  dict (its keys), set or depset; a label is a string read in the BUILD
  file's package and through its repo's mapping, or a `Label`; a dict-typed
  attribute's errors do not name the attribute. Then come the events for a
  label given twice (not in `visibility`, `transitive_configs` (a sorted
  set), outputs or dict keys), a test's size and timeout, and a rule name that
  enters a subpackage (an event, the rule is still added); and a label
  attribute that enters a subpackage is reported when the BUILD file is done,
  after every other event. An event is at the `(` of the call *in the BUILD
  file*, even when a macro makes the call.
- **Outputs.** `outputs = {"o": "%{name}.txt"}` (and `outputs = f`, called
  with `name` and the attributes its parameters name) and each `attr.output`
  or `attr.output_list` value make `GeneratedFile` targets of the package.
  `%{attr}` is a string attribute's value, a label's name without its
  extension, or one output per element of a list. A file that conflicts with
  another target, or that the rule makes twice, is fatal; one that is not a
  legal name is an event.
- **`existing_rule`** shows the attributes the call set and the schema's
  defaults, skipping `_private` ones and what has no default; the schema a
  call used is kept by target name in the `BuildContext`.
- **Known differences.** A rule a function made, bound to a name, is anonymous
  until its module ends (buildfiji-10f). Native rules' attribute labels are now read
  through the repo mapping like a `rule()`'s, so an unknown repo is
  `@@[unknown repo 'r' requested from @@dep+]//s:t` for them too.
  `provider(init = struct)`-style crate differences and the crate's generic
  wording are as for `attr.*` (buildfiji-v32).

## `select()` (implemented 2026-09-30, buildfiji-mum.3.6)

`select.rs`. Read off about 1,800 probes of Bazel 9.2.0 and replayed in
`select_matrix.rs`.

- **The value.** `select(x, no_match_error = "")` takes a non-empty dict (`x`
  is positional-only) whose keys are label strings or `Label`s, and copies
  it; keys are not read as labels until an attribute takes the value. It has
  type `select`, prints `select({"a": 1})`, is always true, and is neither
  hashable nor iterable. It is a global of BUILD files and `.bzl` files and
  not of `native`.
- **`+` and `|`** make a list of elements: a `select()` and a plain value are
  one each, and `[1] + select(..) + [2]` prints as written (two plain values
  in a row are added first). Each element has a type (a plain value's, or the
  type of a `select`'s first value, which is what "select of T" in a message
  means), and `combine` refuses in Bazel's order. `+`: a plain dict is
  unsupported; two types that differ are incompatible (list, tuple and range
  are one type); a dict type is unsupported again. `|`: a plain value that is
  not a dict is unsupported; neither type a dict is unsupported; exactly one
  is incompatible. Equality is Java's: dicts ignore order, `1` is not `1.0`,
  and `no_match_error` counts.
- **Attributes.** A select is converted branch by branch as the attribute's
  type, with the error wording `... for each branch in select expression of
  attribute 'a' of 'r' (including '//:k')`. A key is read like a label
  attribute's string (`//conditions:default`, written or resolved, is the
  default condition), two spellings of one label are one branch with the later
  value, and a `None` branch is the type's zero (a label's stays `None`).
  Only the attributes Bazel fixes (`visibility`, `tags`, `testonly`,
  `deprecation`, `compatible_with`, `restricted_to`, `exec_compatible_with`,
  `package_metadata` (also as `applicable_licenses`), `transitive_configs`,
  `generator_name`, `exec_group_compatible_with`, a test's `size`, `timeout`,
  `flaky` and `local`, and every output) refuse one:
  `attribute "x" is not configurable`. A bool or label attribute cannot be
  concatenated (`type 'boolean' doesn't support select concatenation`); ints
  add. `configurable =` of `attr.*` is refused in `rule()`.
- **What is kept.** `AttrValue::Select(SelectorList)` in `fjfj-graph`: each
  selector has its branches (labels, in the order written), its
  `no_match_error` and whether it was *unconditional*, which a plain value and
  a `select()` written with only the default are. A list that is all
  unconditional is joined and stored as the plain value, and that is what
  `existing_rule` shows; any other shows `select({Label("//:a"): ("x",)})`,
  with a plain value in it as a `select()` of only the default.
  Duplicate labels are checked in each branch, and a label or a condition that
  enters a subpackage is an event. Resolving a select against a configuration
  is buildfiji-136.5's.
- **Known differences.** A plain dict on the left of `|` (`{} | select(..)`)
  fails in the crate's words: the dict's `|` has no hook for the right operand.
  Bazel's `print(select)` and `type(select)` are `<built-in function select>`
  and `builtin_function_or_method` (buildfiji-v32).

## Declaration values (implemented 2026-09-30, buildfiji-mum.3.7)

`decl.rs`. `aspect()`, `transition()`, `analysis_test_transition()`,
`exec_group()`, `configuration_field()`, `subrule()` and the `config`
namespace check their arguments with Bazel 9.2.0's words and return inert
values that freeze and keep what a later bead reads: an aspect or subrule
keeps the validated arguments by parameter name (`aspect_arg`,
`subrule_arg`), a transition its implementation and settings, an
`exec_group` its constraint and toolchain labels. About 1,100 probe rows
replay in `decl_matrix.rs`.

- **Binding and order.** Each argument is converted to its parameter's type as
  it is read (positional ones first, then in the order written), before a
  duplicate or surplus argument is found; then `aspect()` checks the shape of
  `attrs` and its names, `subrules` (and that each is exported), what `attrs`
  says of each attribute (a private one needs a default, a public one must be
  a bool, int or string), `exec_compatible_with`, `exec_groups`,
  `toolchains`, `attr_aspects`, `toolchains_aspects`, `required_providers`,
  `required_aspect_providers`, `provides` (exported providers), `requires`
  and `fragments`. A missing required argument is reported with all the names
  (`missing 3 required named arguments: a, b, c`).
- **Names.** An aspect and a subrule are named by the top-level name they are
  bound to, as rules are (`exports.rs`), and an aspect used in
  `attr.*(aspects = ...)` must have one. An aspect always prints `<aspect>`; a
  subrule prints `<subrule NAME>`. Only `aspect()` and `configuration_field()` refuse to run
  outside `.bzl` initialization.
- **Transitions.** A setting is `//command_line_option:x` or an absolute
  label, with Bazel's words for each way it can be wrong; none may repeat
  (as written, or as two spellings of one label). `and_then` joins two. All
  have type `transition`, except `config.exec()`, whose type is
  `ExecTransitionFactory`. Equality: two `transition()`s of one implementation
  and one set of settings are equal, `config.target()` equals any other one,
  and so does `config.none()`; nothing else is. An attribute whose `cfg` is one
  of the `.bzl`'s own is never equal to another, `config.target()` is
  `cfg = "target"`, and `config.exec()` is equal to itself only.
- **`configuration_field`** accepts the twelve fragments Bazel 9.2.0 knows and
  the late-bound fields found on them (`cpp.zipper`, `proto.proto_compiler`,
  ...); the list was found by probing candidate names, and may miss a rare
  one.
- **Build settings.** `config.bool(flag)`, `int(flag)`, `string(flag,
  allow_multiple)`, `string_list(flag, repeatable)` and `string_set(flag,
  repeatable)` print `<build_setting.boolean>` and so on; a `string_set`'s
  default must be a `set` and `existing_rule` does not show it.
- **Known differences.** The crate prints a builtin or a bound method by its
  name and gives it type `function` (buildfiji-v32), and `print(config)` is a
  namespace's.

## `macro()` (implemented 2026-09-30, buildfiji-mum.3.8)

`macros.rs`, replayed from about 440 probe rows in `macros_matrix.rs`.

- **Declaring.** `macro(*, implementation, attrs, inherit_attrs, finalizer,
  doc)`. `attrs` may not declare `name` or `visibility` (the macro's own),
  nor a computed or late-bound default; a `None` value removes an inherited
  attribute. `inherit_attrs` is a rule, a macro or `"common"`: the public
  attributes of the rule, less `name` and `generator_*`; those of the macro;
  or the ones every rule has. Natives are not rule values here, so
  `inherit_attrs = native.filegroup` waits for buildfiji-136.10. A macro is
  named like a rule (`NAME = macro(...)` at once, `Kind::Macro`); one with no
  name cannot be instantiated.
- **Instantiating** (keywords only, `name` a string, from a BUILD file, a
  legacy macro, or another macro): an unknown attribute, a value of the wrong
  type and a `select()` for an attribute that is not configurable are fatal
  (a rule's are events), with the conversion's words (`convert`); a missing
  mandatory attribute and a value outside `values` are events *without a
  location*. The implementation is called with `name` and every attribute
  (`decl` and `rule()`'s `bind` do not apply: it is an ordinary call, so one
  that does not take `visibility` fails with the crate's words): a
  configurable attribute is a `select`, so a plain `v` is `select({"//conditions:default":
  v})`; an inherited attribute not given is `None`; `visibility` is the
  labels given plus `//pkg:__pkg__` of the call, sorted, or
  `//visibility:public` alone; a label is a `Label`. It returns `None`.
- **Names.** A macro's name is a target name: it may not be another macro's or
  a target's (`macro 'a' conflicts with an existing target.`), and a target may
  not take a macro's unless that macro made it. A macro that calls itself,
  directly or not, is an event and is not run. A label of a rule a macro
  declares is not checked for entering a subpackage.
- **Inside a macro**, `glob()` and `package()` and, unless it is a finalizer,
  `existing_rule()` and `existing_rules()` are errors.
- **Finalizers** are queued and run after the BUILD file, in order, and then
  `existing_rules()` shows the rules there were before the first (not what
  finalizers made); a finalizer may not be instantiated by a macro that is
  not one, and may instantiate another.
- **Known differences.** `print(label)` writes `//a:b` in Bazel and the crate
  `@@//a:b` (buildfiji-v32); an event inside a macro is at the call in the
  BUILD file (`native::location` reads the whole call stack).

## Load visibility (implemented 2026-09-30, buildfiji-ps4)

`load_visibility.rs`. A `.bzl` may call `visibility(value)` once at its top
level (`value` positional-only): `"public"`, `"private"`, or a list of
`public`, `private` and package specs (`//pkg`, `//pkg/...`, `//...`, `//`,
`@repo//pkg`, `@@repo//pkg`; parsed by `PackageSpec::parse`, the same as a
`package_group`'s, with `@repo` read through the file's repo mapping and an
unknown repo reaching nobody; a `-` spec is refused). No call means public;
`private` and `[]` let no other package load it, `public` in a list wins, a
package always loads its own files, and `//a` does not reach `//a/sub`.
`load visibility may not be set more than once`, `... may only be set at the
top level, not inside a function`, and outside `.bzl` initialization
`visibility() can only be used during .bzl initialization (top-level
evaluation)`. The declaration is the module's extra value, so a loader reads
it from the frozen module with `load_visibility(&module)` and asks
`check_load_visibility(importer, file, &visibility, check)` (`check` is
`--check_bzl_visibility`) for Bazel's error: `Starlark file //a:lib.bzl is
not visible for loading from package //b. Check the file's `visibility()`
declaration.`, in which the importer is the file doing the load (a `.bzl`
too, not only a BUILD file) and the main repo's root package is `//`.
Nothing in this crate loads files, so the loader of buildfiji-mum.19 makes
the call; `load_visibility_tests.rs` has a loader that does, and replays the
probes. The error is reported at the load statement's string, and every
refused load of a file is reported; here a loader fails at the first.

## The loader (implemented 2026-09-30, buildfiji-mum.19, mum.20, mum.21)

`loader.rs`. One `BzlLoader` serves a build and is `Sync`: any number of
threads call `load_package(repo, package)` (which reads the BUILD file and
evaluates it) and each `load()` is served by the loader for the file that
makes it (`importing(label)`), so a relative label, a repo name and load
visibility (buildfiji-ps4) are the importer's.

- **Once, shared.** Each `.bzl` has a slot (`Empty`, `Running(thread)`,
  `Done(frozen module or error)`); the first to ask evaluates it and the
  others wait on the slot's condvar and take the same `FrozenModule`. Errors
  are shared too. A file asked for by the thread already evaluating it is a
  cycle; a file whose evaluating thread is, through others, waiting for one
  this thread is evaluating is a cycle as well (checked and registered in one
  step under a lock, so two threads cannot both wait), so a cycle is an
  error naming its files (`cycle detected in extension files: \n.-> //a:x.bzl
  ...`) and never a hang. `evaluations()` counts files evaluated.
- **What stays.** Only the `FrozenModule`s: `evaluate_bzl` parses the source,
  `eval_module` consumes the syntax tree, and the source is dropped when
  the file is evaluated, so no `AstModule` and no source text is held.
  Spans `read`, `parse` and `evaluate_bzl` break a load down.
- **Messages** as Bazel words them (probed): `in load statement: ...` for a
  label that is wrong or not a `.bzl`/`.scl`, `Every .bzl file must have a
  corresponding package, but '//a:x.bzl' does not have one. ...`, `cannot
  load '//a:x.bzl': no such file` or `is a directory`, the subpackage
  message, `Unable to find package for @@[unknown repo 'nope' requested from
  @@]//a:x.bzl: The repository ... No repository visible as '@nope' from main
  repository.`, and the refused load of ps4. Bazel reports each load that is
  refused and a file that does not evaluate as `initialization of module
  ... failed`; here a load stops at the first error, with the evaluation
  error's own text.
- **Numbers** (`loader_scale`, `--release`, one `.bzl` and 4,000 packages of
  eight targets on this machine): 340 ms on one thread, 182 ms on two, 96 ms on
  four and 72 ms on eight, peak RSS 16 MB with the packages dropped as they
  finish; what a build *keeps* of 100,000 packages is the engine's to budget
  (buildfiji-23d.3), the loader keeps one frozen module per `.bzl`.

## `repository_rule()`, `module_extension()` and `tag_class()` (implemented 2026-09-30, buildfiji-mum.8.1)

`crates/fjfj-starlark/src/ext.rs` declares the three values a `.bzl` uses to
define a repository rule and a module extension. They check their arguments
with Bazel 9.2.0's words (about 200 probe rows replayed in `ext_matrix.rs`,
plus hand tests) and keep the validated arguments by parameter name for the
beads that run them (`repository_rule_arg`, `module_extension_arg`,
`tag_class_arg`). Running is `repository_ctx` (buildfiji-mum.8.2) and
`module_ctx` (buildfiji-mum.8.4).

- **Where.** `.bzl` files only; BUILD files and `native` do not have them.
- **Checks.** Parameter types first, positional then named in the order
  written (only `implementation`, and `tag_class()`'s `attrs`, may be
  positional); then contents: `repository_rule()` checks the elements of
  `environ` (`at index 0 of repository_rule, got element of type int, want
  string`), then the shape of `attrs`, then that `attrs` does not redeclare
  `name`; `module_extension()` checks the shape of `tag_classes`, then the
  elements of `environ` (worded `of environ`). Attribute names are not
  validated (`"a b"` and `"1a"` are fine). `remotable=` is refused whatever
  it is given, as experimental behind `--experimental_repo_remote_exec`.
  `bind_checked`'s `P` gained `experimental` for that.
- **Printing.** A repository rule prints `<starlark repository rule
  @@//pkg:file.bzl%NAME>` (named, as a rule is, by the top-level name it is
  assigned to, a private one too) or `<anonymous starlark repository rule>`;
  a module extension and a tag class print as the unknown Java objects they
  are in Bazel, `<unknown object
  com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>` and `...TagClass>`.
  Their types are `repository_rule`, `ModuleExtension` and `tag_class`.
- **Equality and hashing.** A repository rule and a module extension are
  equal to themselves only, and a tag class to any tag class with equal
  `attrs` and `doc`. A repository rule hashes; a module extension and a tag
  class do not (`unhashable type`).
- **Calling.** A repository rule may only be called from a module
  extension's implementation: elsewhere it says `repo rules can only be
  called from within module extension impl functions` (`unexpected
  positional arguments` if given any). A module extension and a tag class
  are not callable.

## `repository_ctx`, local operations (implemented 2026-09-30, buildfiji-mum.8.2)

`crates/fjfj-starlark/src/repo_ctx.rs` is what a repository rule's
implementation is given, and `run_repository_rule(module, rule_name, env,
mappings, print)` runs one: it finds the `implementation` the
`repository_rule()` (buildfiji-mum.8.1) was made with, calls it with a
`repository_ctx`, removes the repository directory if it failed, and fails a
rule that finished without the directory existing. Everything is replayed
from `bazel fetch` runs of the same rules (`repo_ctx_matrix.rs`: about 215
rows of what each printed, the error it stopped with and the tree it left),
captured by a harness that writes a workspace with a `use_repo_rule` call and
reads the repository out of the output base.

The caller says everything about the world through `RepoEnv`: the canonical
name (`+r+x` for `use_repo_rule` from the root module: `<module repo>+<rule
name>+<repo name>`), the directory, the workspace root, the client
environment, a function from a `Label` to the file it names, and the
attributes the rule was called with (`repository_rule_defaults` supplies the
ones the call left out). The filesystem and process work is in the same file.

What Bazel does, read off those probes:

- **The directory** does not exist when the implementation starts
  (`ctx.path(".").exists` is false). Writing a file, making a symlink, renaming
  into it or running `execute` creates it; a rule that ends without it has
  failed (`+r+x must create a directory`), and so has a rule that deleted it.
  A failed rule leaves no directory behind. (`REPO.bazel` is written by the
  caller's fetch, not here.)
- **Paths.** A string is relative to the directory unless absolute; `..` is
  resolved by text. A `Label` is its file (`ctx.path(Label(...))`, `read`,
  `template`, `symlink`, `execute` take one; `delete` does not). Writes
  (`file`, `template`, `symlink`'s link, `rename`'s destination) must be under
  the directory (`Cannot write outside of the repository directory for path
  <path>`, naming a label's file by its package-relative path). Files are
  executable unless `executable = False`; the directories above a file are
  made; writing over a file replaces it; a file in the way is `<path> (File
  exists)`.
- **Parameters.** The first parameter of each method is positional-only
  (`file(path=...)` is `got named argument for positional-only parameter
  'path'`), the rest of `file`, `template` and `execute` take either form,
  `read`'s `watch`, `readdir`'s `watch` and all of `repo_metadata` are
  keyword-only. The wordings are Bazel's (`in call to file(), parameter
  'content' got value of type 'int', want 'string'`).
- **System errors** are the Java ones: `java.io.FileNotFoundException: <path>
  (No such file or directory)`, `attempting to read() a directory: <path>`,
  `Could not rename <from> to <to>: already exists`, `Could not create symlink
  from <target> to <link>: [unix_jni.cc:297] <link> (File exists)`,
  `[unix_jni.cc:382] <first missing path> (No such file or directory)` for
  `realpath`.
- **`execute`** runs with the client environment plus `environment` (`None`
  removes), in the directory or `working_directory` (made if need be), for 600
  seconds unless `timeout` says. Its result has `return_code`, `stdout` and
  `stderr`: 256 and `Timed out` (and no output) after the timeout, 128 plus the
  signal for a killed process, and 1 with `src/main/tools/process-wrapper-legacy.cc:80:
  "execvp(<program>, ...)": No such file or directory` for a program that cannot
  be started. `which` looks in the environment's `PATH` and refuses a name with
  a slash; `getenv` and `os.environ` read the same environment.
- **Values.** `ctx` and `ctx.attr` print as unknown Java objects; `dir(ctx.attr)`
  is `["name"]` though every attribute reads (an unknown one is an error) and
  `ctx.attr.name` is the canonical name; `path` prints as its string, is
  `"<string>"` inside a container, compares by path, and has `basename`,
  `dirname`, `exists`, `is_dir`, `realpath`, `readdir()` and `get_child()`;
  `os` has `name` (`linux`), `arch` (`amd64`) and `environ`.
- `watch`, `watch_tree` and `read(watch = "yes")` of a path under the directory
  are an error (`attempted to watch path under working directory`); `repo_metadata`
  returns an inert object.

Not here: `download`, `download_and_extract`, `extract` and `patch`
(buildfiji-mum.8.3), the canonical-name and `use_repo` mapping work
(buildfiji-mum.8.5), and what calls the rule from a module extension
(buildfiji-mum.8.4). Known differences: a string is bytes in Bazel and
Unicode here (`ctx.read` of a file with non-ASCII content; that is the bug
filed with this bead), a process is killed, not its group, on timeout,
`readdir` returns the directory's own order (as Bazel does, so a test
cannot compare it), and a repository rule that deletes a file of the
workspace crashes Bazel 9.2.0 (exit 37) where this deletes it.

## `module_ctx` and module extensions (implemented 2026-09-30, buildfiji-mum.8.4)

Two places. `fjfj-starlark` has `run_module_extension` (`module_ctx.rs`): it runs
an extension's implementation over what the modules wrote for it and returns the
repositories the implementation made by calling repository rules. `fjfj-repo`
(new) is the driver that joins that to the module graph: it evaluates the
extensions the root module uses, names and records what they generate, and makes
a repository on demand by running its rule (`Repos::run_extensions`,
`Repos::fetch`). About 85 workspaces from real `bazel fetch` runs replay against
it (`crates/fjfj-repo/src/replay_matrix.rs`; the harness that took them is under
`crates/fjfj-repo/probes`).

What it covers: extensions defined in the main repository and used by the root
module. Extensions of other modules need those modules' repositories fetched
(buildfiji-mum.12), and what several modules' tags do to one extension goes
with that.

What Bazel does, read off those probes:

- **Order.** `use_repo`'s names are checked after the implementation has run;
  tags are checked against their tag classes before it runs, and an error
  says where the tag was written.
- **Tag errors** (`MODULE.bazel:L:C` is the call's opening parenthesis, and
  the parser records it: `ExtensionUsage::location`, `Tag::location`):
  `in 'tag' tag, unknown attribute 'x' provided`, `... mandatory attribute 'x'
  isn't being specified`, `... expected value of type 'string' for attribute 'x',
  but got 1 (int)`; `The module extension defined at ext.bzl:7:23 does not have
  a tag class named x, but its use is attempted at MODULE.bazel:4:12`. A tag
  gets the default of each attribute it left out; an unset label is `None`.
- **`mctx`**: `modules` (each a `bazel_module`: `name`, `version`, `is_root`,
  `tags` with one list per tag class, used or not), `is_dev_dependency(tag)`,
  `root_module_has_non_dev_dependency`, `os`, `facts` (`Facts({})`),
  `extension_metadata(...)` (inert), and `path`, `file`, `read`, `execute`,
  `which`, `getenv`, `report_progress` and `watch` as a repository rule has them,
  in the extension's own working directory (`modextwd/<prefix>` in the output
  base; a path under it may not be watched).
- **Repositories.** A repository rule called from the implementation makes a
  repository named `<repo of the .bzl>+<extension>+<name>` (`+ext+a`;
  `rules_java++toolchains+local_jdk`), the extension being named as
  `use_extension` names it. The name must be a string of letters, digits and
  `-_.`, starting with one of the first two, and not used already (`A repo named a
  is already generated by this module extension at ext.bzl:6:9`); a rule that is
  not exported cannot be called (`attempting to instantiate a non-exported
  repository rule`), and neither can one outside an extension. The call
  returns `None`. The attributes are checked against the rule's (`in call to
  'repo' repo rule with name 'a', unknown attribute 'x' provided`).
- **`use_repo_rule`** is an extension of its own, recorded under the file
  `//:MODULE.bazel` with the name `<bzl> <rule>`: its repositories are named
  `+<rule>+<name>`, and its attributes are read as the rule's types only when
  it is fetched.
- **Errors loading**: `Error loading '//:ext.bzl' for module extensions,
  requested by MODULE.bazel:2:20: ...` (the inner message twice, as Bazel's
  exception does); `//:ext.bzl does not export a module extension called x, yet
  its use is requested at MODULE.bazel:2:20`; the same with `repository_rule`
  for `use_repo_rule`; `expected module extension @@//:ext.bzl%ext to return
  None or extension_metadata, got int`.
- The repo name conflict errors of `MODULE.bazel` itself now carry both locations
  (`cannot be defined by a use_repo() call at MODULE.bazel:4:9 as it is already
  defined by a use_repo() call at MODULE.bazel:3:9`).

Not here, each filed: `mctx.download`, `download_and_extract` and `extract`
(buildfiji-mum.8.3), what `extension_metadata` asks for and `facts`, `isolate`
(an experimental flag), the lockfile's extension results (buildfiji-mum.8.6),
`override_repo` and `inject_repo` and the naming of other modules' repositories
(buildfiji-mum.8.5), and `mctx.path` of a label in the main repository, which
Bazel prints relative.

## `download`, `extract` and `patch` (implemented 2026-09-30, buildfiji-mum.8.3)

`repository_ctx` (and `module_ctx`, for `download`, `download_and_extract` and
`extract`) can fetch, unpack and patch. The work is in three places:
`fjfj-archive` (pure: `extract`, `apply_patch`), `repo_download.rs` in
`fjfj-starlark` (the methods, the checksums and the repository cache), and
`fjfj-repo::HttpDownloader` (a `reqwest` implementation of the
`Downloader` trait `RepoEnv` is given; none is given in tests, which serve from
a table). About 150 rows of real `bazel fetch` runs against a local HTTP server
replay against it (`repo_download_matrix.rs`; the harness is under
`crates/fjfj-starlark/probes/download`).

What Bazel does, read off those probes (the doc comments of `repo_download.rs`
and `fjfj-archive` list it in full):

- **URLs.** Tried in order; `http`, `https` and `file` only; a plain `http` URL
  with no checksum is refused (so every probe names one). A `5xx` is tried
  again (eight attempts in all; the downloader does that), a `4xx` is final.
- **Checksums.** `sha256` or `integrity`, not both, checked after the download,
  with the checksum in the form it was given in the error; one that is not a
  checksum is only found out after.
- **The repository cache** keeps a download at
  `content_addressable/sha256/<hex>/file`, with an empty `id-<SHA-256 of the
  canonical id>` beside it for each canonical id it was put under: the
  `canonical_id` argument, or the URL it came from when there is none. A download that
  names its SHA-256 (as `sha256` or a `sha256-` SRI) is served from it without asking
  the network only if the file has the id of the call (another `canonical_id`, or
  none after one, is a miss: it is downloaded again and the id added). Everything
  downloaded is put there, `file://` included; `--repository_cache=` (empty) turns the
  cache off. **`--distdir`** directories are looked in after the cache for a file
  named like the URL's last segment that the checksum accepts (one that does not is
  passed over, the network is asked); a file from there is not put in the cache.
  Probed in two steps, a second repository fetched in the same output base after the
  first (`then` in `probes/ha_cases7.json`).
- **Results.** `struct(integrity, sha256, success)`; `allow_fail` turns a failed
  download into `struct(success = False)`; `block = False` returns a
  `PendingDownload` whose `wait()` gives the result or the error.
- **download_and_extract** downloads into `<output>/temp<digits>/<name>`, where
  `<name>` is the URL's last segment plus `.<type>` if `type` was given, extracts,
  and removes the directory (which stays behind after an `allow_fail` failure).
- **Archives.** `.zip .jar .war .aar .nupkg .whl .tar .tar.gz .tgz .tar.xz .txz
  .tar.zst .tzst .tar.bz2 .tbz`, the single-file `.gz .xz .zst .bz2`, and `.ar`
  and `.deb` (extracted as the `ar` archive they are). `rename_files` is applied
  to a member's path in the archive, then `strip_prefix` keeps what starts with it.
  Nothing stops a member from leaving the output (`../x` lands beside it, `/x` is
  `x`); a hard link needs its target extracted first.
- **Patches** are unified diffs, plain or from `git diff`, applied where the
  hunk's lines are found, with `strip` leading components removed from the names;
  files are made, removed and renamed. **A plain patch of several files applies
  only the last**: Bazel 9.2.0 drops the others without a word, and this does too;
  with `diff --git` lines every file applies.

Differences and gaps, filed: `.7z` is refused as Bazel does (with `null`); an
`integrity` of `sha1-` is not computed; a timed-out process is killed but not its
process group; downloads run one at a time where Bazel's `block = False` ones
overlap; two fjfj processes sharing a cache do not lock or write atomically; the flags
themselves wait for the CLI wiring (buildfiji-mum.12.2); `--credential_helper_timeout` and the helper's
result cache (`--credential_helper_cache_duration`) are not probed.

## Extensions of every module, and repositories made on demand (implemented 2026-09-30, buildfiji-lfe)

`fjfj-repo`'s `Repos::from_resolution(options, resolution)` serves every
repository of a workspace: the main one, `@bazel_tools`, each selected module
and what extensions generate. The `BzlLoader` asks a `RepoProvider` (in
`fjfj-starlark`) for a repo's files when a load first needs one, and the
provider makes it then: a module's repo by running the rule its `source.json`
(or non-registry override) names (`http_archive`, `git_repository`,
`local_repository`, Bazel's own Starlark), a generated repo by running its
extension first, and then its rule. So an extension's `.bzl` may load from
`@helper` (another module) or from `@gen` (made by a different extension), as
Bazel's does; both are replay rows.

Read off probes of `bazel build` against a local registry of local-path modules
(`src/multi_matrix.rs`, 8 rows):

- An extension runs once over every module that uses it: `mctx.modules` is the
  root first, then the others in breadth-first order (the order of the root's
  `bazel_dep`s), with each module's tags in the order written. A dependency's
  `dev_dependency = True` usage is dropped whole (tags and `use_repo`); the
  root's dev tags are in its list. `root_module_has_non_dev_dependency` is
  false if the root has no usage or only dev ones.
- Every module's `use_repo` is checked against what the extension generated
  (unless the root overrides the name), and the message names the usage by the
  file it is in: a dependency's is `<registry url>/modules/<name>/<version>/MODULE.bazel:3:18`.
- A `use_repo_rule` of a dependency makes `<module repo>+<rule>+<name>`
  (`a++made+mine`) with `ctx.original_name` the name given.
- What a lazily run extension prints is kept and handed to whoever asked for
  the repo that needed it, in order (`Repos::fetch`, `run_extensions`).

`--override_repository=<name>=<path>` (`Options::repo_overrides`, 9 more rows):
the name is one the main repository sees (a `bazel_dep`'s `repo_name`, a
`use_repo` import, a `use_repo_rule` call), else `no repository visible as
'@lib' from the main repository, but overridden with --override_repository.
Use --inject_repository to add new repositories.` (exit 48). The repo is a link
to the path (relative to the workspace), whatever it was: an extension still
runs, and the repo it generated is replaced. The errors: `The repository's path
is "lib+" (absolute: "<dir>") but it does not exist or is not a directory.` (the
canonical name stands where local.bzl prints `path`) and `No MODULE.bazel,
REPO.bazel, or WORKSPACE file found in <dir>`. An empty name is ignored.
`--override_module` is resolution's (`ResolveOptions::command_overrides`); it
loses to `--override_repository` for the same repo.

Not covered: a module whose source is an archive or git repository
(`Resolution::module_repo_spec` gives the attributes; the rules are the ones
`http_archive_matrix` replays), `archive_override` and friends, and several
threads asking for repos at once (a repo may then be made twice).

## Carrying changes to the `starlark` crate (decided 2026-09-30, buildfiji-6z4)

**Decision: a patch queue, applied by Bazel to the crate it downloads.** `MODULE.bazel`
has `crate.annotation(crate = "starlark", patches = ["//third_party/starlark/NNNN-*.patch"],
patch_args = ["-p1"])`; `third_party/starlark/README.md` says how to make one. It was tried
on buildfiji-b9c (`"%s" % label`) and works: the patched crate is what `bazel build` links,
the probe row that was skipped now passes, and cargo (which builds nothing here) is unaffected.

Options considered:

1. **A fork under `[patch.crates-io]`.** The same patches, plus a copy of four crates
   (`starlark`, `starlark_syntax`, `starlark_derive`, `starlark_map`) to keep in step with
   every release, and a splice of path crates into crate_universe. Rejected: the patch queue
   gives the same diff without the copy.
2. **Upstream PRs only.** Right for changes Buck2 wants, but it leaves fjfj with Bazel
   differences for as long as a review takes. Kept as the second step: each patch says
   whether it was sent.
3. **Workarounds in fjfj** (AST rewrites, wrappers, hooks), as mum.3.x did. Kept where it
   reaches the gap cheaply and cleanly: buildfiji-8q5 (`\u`, `\U`, `\x` escapes) is a
   check over the string literals after parsing, in `dialect.rs`, and stays there.
4. **Replacing the evaluator.** Not reconsidered: the mum.1 spike rejected a custom parser on
   speed, and nothing here is about evaluation speed.

What goes where: patch: buildfiji-b9c (done, `0001`), buildfiji-tg2 (in-place set
operators: `stmt.rs`), buildfiji-2r5 (recursion: the call path of `def`), buildfiji-gpj (a
huge repeat panics), buildfiji-sib (cyclic `repr`, raw strings) and buildfiji-wtt, buildfiji-9zq
(split keywords, `elems()` iterators) when each is taken; fjfj side: buildfiji-8q5. Error
wording (buildfiji-v32, print of a label) is decided row by row: a patch when a whole family of
messages is the crate's, a wrapper when it is one call.

## Bazel's own `http_archive` and `git_repository` (implemented 2026-09-30, buildfiji-mum.12)

`@bazel_tools`' repository rules are Starlark, and they run as Bazel ships
them: `crates/fjfj-repo/embedded_tools/` holds `utils.bzl`, `cache.bzl`,
`http.bzl`, `git.bzl` and `git_worker.bzl` from 9.2.0, and `tools.rs`
(`BAZEL_TOOLS_FILES`, `materialize_bazel_tools`) writes them into the
`bazel_tools` repository. Nothing about these rules is reimplemented in Rust;
what they need from the runtime is `repository_ctx` (see above).

Conformance is 37 rows in `http_archive_matrix.rs`, read off Bazel 9.2.0 against a
local HTTP server and local git repositories, and replayed by
`http_archive_tests.rs`: the resulting tree, the prints, or the error. The
harness makes each row's git repositories with a fixed author, date and
`GIT_CONFIG_NOSYSTEM`, so commit hashes are stable, and the rows compare them as
`<commit>`. `local_repository` and `new_local_repository` are in the table too (the table has 92 rows): `local_repository` makes the repository directory a link to the user's
directory (`rctx.symlink(path, ".")`), so a directory with no `MODULE.bazel`,
`REPO.bazel` or `WORKSPACE` in it is refused (`No MODULE.bazel, REPO.bazel, or
WORKSPACE file found in <output base>/external/+local_repository+x`) where a
generated directory gets an empty `REPO.bazel`. `netrc`, `auth_patterns` and
`--credential_helper` are in the table too (92 rows in all; the test double records
the `Authorization` header it would have sent and what each helper script
logged). Bazel's Starlark (`utils.bzl`) reads `netrc`, and `crates/fjfj-repo/src/credentials.rs`
is the helpers: `<program> get` with `{"uri":...}` on stdin and
`{"headers":{...}}` back, a host scope or `*.<domain>` (which includes the domain itself),
the host's own scope before a wildcard (the longest) before the default, the last flag
of a scope winning, and a helper's `Authorization` beating `netrc`. A helper that
fails is a warning and the download goes on without. `verbose = True` output is
not replayed: it depends on the machine's files and environment.

Bazel quirks these rows pin: a plain multi-file patch applies only its last file
(`fjfj-archive`), and `strip_components` is applied after `rename_files`.

Still open: the flags `--override_repository`, `--credential_helper` and the rest
in `fjfj build`, with `--registry` and `--lockfile_mode` (buildfiji-mum.12.2).

## Bazel 9.2.0's builtin namespaces, and who owns each name (buildfiji-mum.3)

Read off Bazel 9.2.0 by asking `type(name)` in a BUILD file and in a `.bzl`
loaded by one, and `dir()` of each namespace. `B` is visible in BUILD files,
`Z` in `.bzl` files. fjfj has, so far, `depset` (B, Z), `set`, `json`, `proto` (B, Z),
`struct`, `Label`, `attr`, `provider`, `rule` and `macro` (Z), `select` (B, Z), `print` and the standard Starlark library, and the natives of
buildfiji-mum.4.

| Names | Where | Owner |
|---|---|---|
| `abs all any bool dict dir enumerate fail float getattr hasattr hash int len list max min print range repr reversed sorted str tuple type zip` | B Z | the `starlark` crate; wording gaps in buildfiji-v32 |
| `set` (with `add clear difference ... update`) | B Z | buildfiji-mum.3.1, done |
| `struct` | Z only | buildfiji-mum.3.1, done |
| `json` (`encode decode encode_indent indent`), `proto` (`encode_text`) | B Z | buildfiji-mum.3.1, done |
| `Label` (`name package relative repo_name same_package_label workspace_name workspace_root`) | Z only | buildfiji-mum.3.2, done |
| `attr` (`bool int int_list label label_keyed_string_dict label_list label_list_dict output output_list string string_dict string_keyed_label_dict string_list string_list_dict`) | Z only | buildfiji-mum.3.3, done |
| `provider` | Z only | buildfiji-mum.3.4, done |
| `rule` | Z only | buildfiji-mum.3.5, done |
| `select` | B Z | buildfiji-mum.3.6, done |
| `aspect transition exec_group configuration_field subrule analysis_test_transition` | Z only | buildfiji-mum.3.7, done |
| `config` (`bool exec int none string string_list string_set target`) | Z only | buildfiji-mum.3.7, done |
| `macro` | Z only | buildfiji-mum.3.8, done |
| `visibility` | Z only | buildfiji-ps4, done |
| `module_extension repository_rule tag_class` | Z only | buildfiji-mum.8.1, done (declarations); what runs them is buildfiji-mum.8.2 and .8.4 |
| `DefaultInfo OutputGroupInfo RunEnvironmentInfo InstrumentedFilesInfo PackageSpecificationInfo` | Z only | buildfiji-136.4 |
| `platform_common` (`ConstraintSettingInfo ConstraintValueInfo PlatformInfo TemplateVariableInfo ToolchainInfo`), `config_common` (`FeatureFlagInfo config_feature_flag_transition toolchain_type`), `coverage_common` (`instrumented_files_info`), `testing` (`ExecutionInfo TestEnvironment analysis_test`), `cc_common java_common apple_common android_common` | Z only | Starlark or absent by decision (buildfiji-136.14): buildfiji-136.16, buildfiji-136.17, buildfiji-136.15 |
| `native` | Z only | buildfiji-mum.4 (done), buildfiji-hrx |
| `glob package package_group exports_files existing_rule existing_rules package_name repository_name subpackages package_relative_label licenses filegroup alias` | B only | buildfiji-mum.4 and buildfiji-mum.3.2 (done), buildfiji-hrx |
| `genrule config_setting test_suite toolchain_type` and the language rules (`cc_library`, `java_library`, ...), still native in 9.2.0 | B only | buildfiji-136.10, buildfiji-136.11 |

### Repo mapping from the graph (implemented 2026-09-30, buildfiji-mum.15)

`Resolution::repo_mappings()` gives, for each selected module (breadth first),
its canonical repo name and what each apparent name means there: the main
repo sees `""` and its `repo_name` as itself, every module sees itself under
its `repo_name`, then its `bazel_dep`s under the names they were given, and
the built-in `bazel_tools` last; with a `multiple_version_override` the
canonical name carries the version (`a+1.0`). Each workspace fixture has an
`expected_repo_mapping.txt` that `bazel run
//crates/fjfj-bzlmod/tests/fixtures:refresh_golden` records from `bazel mod
dump_repo_mapping`, and `repo_mappings_match_bazel` compares line for line.
`fjfj_starlark::RepoMappings::from_repos` takes those rows. What `use_repo`
brings in (`module+ext+repo` names) waits for module extensions
(buildfiji-mum.8), and the `_repo_mapping` runfiles manifest is
buildfiji-136.9's.

## bzlmod: module resolution (implemented 2026-09-03, buildfiji-mum.6)

`crates/fjfj-bzlmod` evaluates `MODULE.bazel`, walks out to the whole
dependency graph, and runs Minimal Version Selection over it. The
algorithms are ports of Bazel 9.2.0's `ModuleFileGlobals`, `Discovery`,
`Selection`, `Version` and `IndexRegistry`, not reconstructions from the
documentation: a resolution that differs from Bazel's by one version is a
different build.

Spec: `spec/Fjfj/Bzlmod.lean`. Out of scope here and tracked separately:
running module extensions and repository rules (buildfiji-mum.8). The
lockfile (buildfiji-mum.7, the last subsection here) and the
apparent-name half of repo mapping (buildfiji-mum.15, in "`Label` and
repository mapping") have landed.

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

### `bazel_tools` is Bazel's own module file (buildfiji-mum.23)

Every module implicitly depends on `bazel_tools`, which Bazel ships inside
its own binary rather than serving from a registry
(`NonRegistryOverride.BAZEL_TOOLS_OVERRIDE`). fjfj embeds the
`MODULE.bazel` from that binary's `embedded_tools`
(`crates/fjfj-bzlmod/src/bazel_tools.MODULE.bazel`, copied from
`$(bazel info install_base)/embedded_tools/MODULE.bazel`; refresh it when
the Bazel version moves) and serves it as `discovery::BAZEL_TOOLS_MODULE`.

It matters because `bazel_tools` has `bazel_dep`s of its own (`rules_cc`,
`rules_java`, `protobuf`, `rules_python`, ...), which are part of every
module graph and raise the selected version of anything that shares them:
with a dependency-free placeholder protobuf resolved to 29.1 where Bazel
picks 33.4. `fjfj mod graph` hides the `bazel_tools` subtree as Bazel does;
Bazel's `--include_builtin` shows it, and so does the conformance test.
Its `use_extension`/`use_repo`/`use_repo_rule` calls are recorded but not
run (buildfiji-mum.8), and the tools repository those labels point into is
not shipped yet (buildfiji-mum.12).

### Conformance method

The fixtures under `crates/fjfj-bzlmod/tests/fixtures` are a local module
registry and one workspace per resolution scenario — MVS, pruning of a
module that lost its only dependent, both override kinds, fulfilled and
unfulfilled nodep edges, a yanked version. The expected result of each is
**Bazel's own output**, captured by
`bazel run //crates/fjfj-bzlmod/tests/fixtures:refresh_golden` and
committed, so the test compares against Bazel rather than against a
restatement of the implementation.

Every module graph includes `bazel_tools`' own, so the fixtures resolve
against two registries, in the order the goldens were captured with: the
fixture registry, then `fixtures/bcr`, the 184 files of the Bazel Central
Registry that graph reads (`vendor_bcr.py` copies them, each checked
against the hash in `lockfiles/9.2.0.lock`, a lockfile Bazel 9.2.0 wrote).
Each workspace has two graph goldens: `expected_graph.txt` (`bazel mod
graph`) and `expected_graph_builtin.txt` (`--include_builtin`, which adds
`bazel_tools`' subtree and so the versions of all 29 BCR modules it pulls
in). `graph_to_golden.py` also records the edges Bazel lists under
`cycles`, where `bazel_tools` and the modules it depends on depend on each
other.

Two ignored tests reach the network, run by hand: one reads real modules,
`source.json` and `metadata.json` from `bcr.bazel.build`, and one resolves
a copy of this repository's own `MODULE.bazel` (`fixtures/repo`) against it
and compares every edge of the graph (131 of them) with `bazel mod graph`'s.
With `BCR_DIR` set to a directory laid out like the registry, the second
runs offline: built from the files in a Bazel repository cache by the URLs
in this repository's own `MODULE.bazel.lock`, it agrees with Bazel on all of
them. Before `bazel_tools`' real module file was embedded it did not.

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

### `fjfj mod` (buildfiji-9s8.4)

Rendering, not resolution: `fjfj-cli::mod_command` takes a `&Resolution`
`fjfj-bzlmod` already produced and presents it four ways —
`graph` (a tree, indented text by default or Bazel's own `--output=json`
shape), `deps <module>...` (a module's own edges and what they resolved
to), `show_repo <repo>...` (what fjfj knows about a repo — name, version,
canonical/apparent names, registry; not the repo rule's own attributes,
which need it fetched first, buildfiji-mum.8), and `explain <repo>...`
(every requester in the *unpruned* graph, which is exactly what
`Selection.unpruned` exists for). None of the four talk to a registry or
touch a file — `Command::Build` and `Command::Mod` share one
`resolve_workspace_bzlmod` for that.

`graph --output=json`'s shape (`key`/`name`/`version`/`apparentName`/
`dependencies`/`indirectDependencies`/`cycles`, `root: true` only on the
root) is transcribed from a real `bazel mod graph --output=json` run
against `fjfj-bzlmod`'s own conformance fixtures, not reconstructed from
docs — and `mod_command`'s own test flattens its JSON tree back into the
`<parent> <apparent> <child>` edge format those fixtures' golden files
already use, so the same golden data conformance-tests two things: bzlmod
resolution (buildfiji-mum.6) and its `mod graph` rendering, without a
second run of real Bazel. `bazel_tools`'s subtree is hidden the same way
`bazel mod graph` hides it — it isn't something the user wrote
(its module file is `discovery::BAZEL_TOOLS_MODULE`).

`--output=text` is not byte-matched against Bazel's own box-drawing tree,
and `deps`/`show_repo`/`explain`'s text isn't matched against Bazel's at
all — only the JSON graph shape is a conformance point today.

### Extension repo names and `use_repo` in the repo mapping (implemented 2026-09-30, buildfiji-mum.8.5)

`crates/fjfj-bzlmod/src/extension_repos.rs`, read off `bazel mod dump_repo_mapping`
for the `extensions` and `extensions_versions` fixtures (the goldens dump every
repo, extension repos too):

- An extension's repo is `<canonical repo of its .bzl>+<unique extension
  name>+<repo>`: `ext++gen+r1`, `+gen+a` (the `.bzl` is in the main repo),
  `ext+2.0+gen+r1` (two versions of `ext` are selected).
- The unique name is the extension's name, with `2`, `3` on later ones of the
  same name in the same repo (two `.bzl` files both define `gen`). Later means
  by first use: modules in breadth-first order (so the root's `ext.bzl` is
  `gen` and `q`'s `a.bzl` is `gen2`), each module's usages in file order.
- A `use_repo_rule` is an extension per rule: the `.bzl` repo is the calling
  module (`p++rr+made`, `+rr+x`), named for the rule and numbered the same way
  (`+rr2+y`), and every call is an import.
- A module's mapping lists its `use_repo` imports first, in file order, then
  its own name, its deps, and `bazel_tools`.
- Only the root module's `override_repo` and `inject_repo` count (a
  dependency's are silently dropped), and they apply to every module: `r1` is
  `+gen+a` for `mid` too. `override_repo` targets are read in the root's mapping.
- A generated repo's mapping is the mapping of the module that holds its
  `.bzl`, then the extension's other repos, then the root's injected repos.
  `Resolution::extension_repo_mapping` takes the generated names as input,
  because running an extension is `fjfj-repo`'s job.
- `isolate = True` names repos after the variable the usage is assigned to;
  left out for now (buildfiji-mum.8.7).

### Module extensions in `MODULE.bazel.lock` (implemented 2026-09-30, buildfiji-mum.8.6)

What `moduleExtensions` holds, read off Bazel 9.2.0's lockfiles **and its bytecode**
(`A-server.jar`'s classes: `SingleExtensionUsagesValue`, `GsonTypeAdapterUtil`,
`Fingerprint`, `RepoRecordedInput`; a hand-assembled `Probe.class` run on the
embedded JRE calls them, see the notes in `probes/README.md`). 36 probed rows replay
(`lock_matrix.rs`).

- The key is the extension as a label then `%` and its name: `//:ext.bzl%collect`
  for the main repository, `@@lib+//:ext.bzl%collect` for another one. Under it
  the factors the result depends on: `general`, or `os:linux`, `arch:amd64`,
  `os:linux,arch:amd64` for `os_dependent` and `arch_dependent`. An extension that
  returns `extension_metadata(reproducible = True)` is not in the file.
- `bzlTransitiveDigest` is base64 of `sha256(sha256(text of the .bzl) ++ the
  transitive digest of each file it loads)`, in the order of its load statements
  (nothing about Bazel's builtins or flags goes in). `BzlLoader::transitive_digest`.
- `usagesDigest` is base64 of the SHA-256, of the UTF-16LE bytes, of the compact JSON
  Gson writes for a trimmed `SingleExtensionUsagesValue`: `extensionUsages` (a
  module key, `<root>` or `name@version`, to `{extensionBzlFile, extensionName,
  proxies: [], tags, repoOverrides: {}}`), `extensionUniqueName`
  (`<repo of the .bzl>+<unique name>`: `+collect`, `lib++collect`),
  `abridgedModules` (`{name, version, key}` of the modules that use it),
  `repoMappings: {}` and the root's `repoOverrides`. A tag is `{tagName,
  attributeValues, devDependency, location}` with the location always
  `{file: "<builtin>", line: 0, column: 0}`; tags of one module's dev and non-dev
  usages are together, in the order the calls were made (`Tag::seq`). The `.bzl`
  written `//pkg:x.bzl` or `:x.bzl` is `@<module's repo name>//pkg:x.bzl`, one
  written `@dep//...` stays. `isolationKey` (none) and a `None` attribute are left
  out. `Resolution::extension_usages_digest`.
- `recordedInputs` are what the implementation read, once each, in the order it
  first did: `ENV:<name> <value>` (`\0` for not set, `\s` for a space, `\n`, `\\`),
  `FILE:@@<repo>//<path> <sha256 of the content>` (`read`, `watch`) and
  `DIRENTS:@@<repo>//<path> <digest>` (`readdir`; the digest is `Fingerprint.addStrings`
  of the sorted names: the count, then each name's length and bytes, as protobuf
  varints, then SHA-256). A path outside every repository is not recorded; a file
  that only `exists` is not.
- `generatedRepoSpecs` is each repository as `{repoRuleId, attributes}` in
  the order the extension made them, the attributes as given (a `None` left out, a
  label as `@@repo//pkg:name`, a label-keyed dict's keys too).
- A `Label` passed to a repository rule inside an extension is accepted, and a
  `None` attribute means not given.

### `extension_metadata`, `facts` and `isolate` (implemented 2026-09-30, buildfiji-6go)

Read off Bazel 9.2.0 (62 rows of `lock_matrix.rs`, and the strings in
`LockfileModuleExtensionMetadata`):

- `extension_metadata(root_module_direct_deps, root_module_direct_dev_deps,
  reproducible, facts)` checks, in this order: each is a list, `"all"` or `None`
  (`want 'sequence, string, or NoneType'`, `at index 1 of root_module_direct_deps,
  got element of type int, want string`); `"all"` needs the other to be an empty list;
  the two are both given or both not; no duplicate (`in root_module_direct_deps:
  duplicate entry 'x'`) and nothing in both lists (`in root_module_direct_dev_deps:
  entry 'x' is also in root_module_direct_deps`); `facts` is a dict.
- After the run: a list that names a repository the extension did not generate is an
  error (`root_module_direct_deps contained the following repositories not generated
  by the extension: b, a`, in the order given), and so is a non-empty list, or `"all"`,
  for regular or dev deps when the root module has no usage of that kind (`must be empty
  if the root module contains no usages with dev_dependency = False`).
- Otherwise the root's `use_repo` imports are compared with the lists and a warning is
  made (`Repos::warnings`), at the first root usage's `MODULE.bazel:2:18`: `The module
  extension collect defined in @root//:ext.bzl reported incorrect imports of
  repositories via use_repo():` and, each if there is one, `Not imported, but reported as
  direct dependencies ...`, `Imported as a regular dependency, but reported as a dev
  dependency ...`, `Imported as a dev dependency, but reported as a regular
  dependency ...`, `Imported, but reported as indirect dependencies by the extension`,
  then `Fix the use_repo calls by running 'bazel mod tidy'.` An extension that says
  nothing about deps (both `None`) is never warned about.
- `moduleExtensionMetadata` of the lockfile entry is there when deps or facts were
  given: `explicitRootModuleDirectDeps` and `...DirectDevDeps` (only when neither is
  `"all"`), `useAllRepos` (`NO`, `REGULAR`, `DEV`) and `reproducible`. The `facts`
  section holds each extension's facts by id with keys sorted and `None` left out
  (`LockSession::set_facts`).
- `mctx.facts` is what a previous run kept (`Options::facts`): `facts.get(key,
  default)`, `facts[key]` (`None` when missing), `key in facts`; `dir()` is
  `["get"]`, there is no `len`, and `str()` is `Facts(<opaque, inspect with
  print()>)`.
- `use_extension(isolate = True)` is refused without
  `--experimental_isolated_extension_usages` (`in call to use_extension(), parameter
  'isolate' is experimental and thus unavailable with the current flags. It may be
  enabled by setting --experimental_isolated_extension_usages`).
  `EvalOptions::isolated_extension_usages`; the flag itself is for the CLI wiring.

Not done (buildfiji-mum.8.8): reading `facts` from the lockfile in the CLI, reusing a locked result instead of running the
extension (and the `error` mode's refusal when the digests differ), the
`REPO_MAPPING:` recorded input a `Label("@dep//...")` read adds, what `use_repo_rule`
and `isolate = True` extensions write, and the file's whole pretty-printed text
(the section is carried as ordered JSON, which the tests of the whole file already
round-trip).

### `MODULE.bazel.lock` (implemented 2026-09-30, buildfiji-mum.7)

`fjfj_bzlmod::lockfile` reads and writes Bazel 9.2.0's lockfile, version
28. Everything below was read off probes of real Bazel (a local HTTP
server standing in for a registry, a fresh server per run so Skyframe's
memory does not pose as the lockfile), not its documentation.

- **Shape.** `lockFileVersion`, `registryFileHashes`,
  `selectedYankedVersions`, `moduleExtensions`, `facts`, `factsVersions`,
  two-space JSON, `{}` and `[]` inline, a trailing newline, no HTML
  escaping. There is no module graph in it: version 28 re-resolves on
  every run. A real lockfile (the 184 hashes and four extension results
  of `bazel_tools`' own graph) round-trips byte for byte, which is a
  conformance test. `moduleExtensions`, `facts` and `factsVersions` are
  kept as an order-preserving `Json` (their keys are neither sorted nor a
  map's own order) until buildfiji-mum.8 owns them.
- **What is recorded.** The SHA-256 of every `bazel_registry.json`,
  `MODULE.bazel` and (for selected modules) `source.json` read from a
  registry that is not `file://`, `"not found"` for one the registry did
  not have, sorted by URL. `metadata.json` is not recorded. Only what the
  run read is written back; the rest is dropped. `Registry::locked(&session)`
  wraps a registry's fetcher in a `LockedFetcher` that does this, and reads
  `bazel_registry.json` before a registry's first module file, as Bazel does.
- **What is checked.** A recorded hash is verified after fetching:
  `Error accessing registry R: Failed to fetch registry file U: Checksum
  was X but wanted Y`. A file recorded `not found` is not asked for again,
  and "module not found in registries" says `previously not found (as
  recorded in MODULE.bazel.lock, refresh with --lockfile_mode=refresh)`.
- **Yanked versions.** Each selected yanked version, allowed or not, is
  recorded as `name@version: reason`; a module with an entry there does not
  have its `metadata.json` read. `ResolveOptions::track_yanked` makes
  `--allow_yanked_versions=all` still look them up.
- **Modes.** `off` neither reads nor writes; `update` and `refresh` write
  (only when the text changed); `refresh` also asks again for `not found`
  files and `metadata.json`; `error` never writes and refuses a file with
  no recorded hash (`Missing checksum for registry file U not permitted with
  --lockfile_mode=error. Please run `bazel mod deps --lockfile_mode=update`
  to update your lockfile.`). A lockfile that is not JSON or not version 28
  is unusable: `error` refuses it, the others replace it.
- **The CLI** (`fjfj-cli`) reads `MODULE.bazel.lock` from the workspace
  root, resolves through a `LockSession`, and writes the file when
  resolution succeeded.

Bazel also reads the `source.json` of every selected registry module (to
build its repo specs), which resolution itself does not need; with
`ResolveOptions::for_lockfile` fjfj does too, so the hashes are recorded.

Conformance (`tests/conformance.rs`): `expected_lock_hashes.txt` per
workspace is what Bazel recorded for the fixture registry, captured by
`lock_hashes.py` (which serves the registry with every `source.json`
rewritten to an archive source, because Bazel refuses `local_path` from a
remote registry, and includes the 156 `not found` probes of `bazel_tools`'
graph). What Bazel recorded for the Bazel Central Registry is the same for
every fixture and is `lockfiles/9.2.0.lock`'s. The comparison is exact in
both directions: same files, same hashes.

`selectedYankedVersions` is in the order of a Java `HashMap<ModuleKey, _>` of
every selected module (`java_map.rs`, buildfiji-avh): `ModuleKey` hashes as
`31 * name.hashCode() + version.hashCode()` (and `Version` as `Arrays.hashCode` of
`"version"` and its normalized text's hash), the map doubles from 16 buckets when
three quarters full, iteration is by `(h ^ h>>>16) & (capacity - 1)`, and keys
in one bucket go by unsigned hash. The same order decides which yanked version
Bazel names first when it refuses one. Checked by `yanked_many` (12 yanked
modules) and by unit tests of the collisions found by probes.

A registry file whose hash the lockfile records is read from the repository cache
(`content_addressable/sha256/<hash>/file`, with no id file: `LockSession::set_repository_cache`)
before the network is asked, and refused if what is there hashes otherwise (`Checksum was X
but wanted Y`); what is fetched is put there (buildfiji-g1z). Known differences, each with a
bead: what `--lockfile_mode=error` does about stale extension or yanked entries is not
probed yet (buildfiji-cob), and the locked extension results are not yet reused
(buildfiji-mum.8.8).
