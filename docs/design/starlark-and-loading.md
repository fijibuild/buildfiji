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
| `module_extension repository_rule tag_class` | Z only | buildfiji-mum.8 |
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
(`RegistrySource`'s placeholder module, discovery.rs).

`--output=text` is not byte-matched against Bazel's own box-drawing tree,
and `deps`/`show_repo`/`explain`'s text isn't matched against Bazel's at
all — only the JSON graph shape is a conformance point today.
