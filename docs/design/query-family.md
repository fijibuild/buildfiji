# query, cquery and aquery

Epic: `buildfiji-9s8`. Code: `crates/fjfj-query` (the language, the output
formats, the protocol buffer writer), `crates/fjfj-cli` (`query_graph.rs`,
`configured_graph.rs`, `analysis_query.rs`, `aquery.rs`, `query_command.rs`).

Bazel 9.2.0 is the spec. Every output below was compared with it on scratch
workspaces; the tests carry what Bazel printed.

## Shape

- `query` evaluates over the loading-phase graph (`QueryGraph`): packages
  loaded on demand, attributes read with their defaults.
- `cquery` and `aquery` analyse the targets their expression names (or the
  closure of `--universe_scope`) without building, then evaluate over the
  configured targets that made (`ConfiguredGraph`). A configured target is
  one node for each configuration it was analysed in; its edges are the
  dependencies analysis followed, so `deps()` crosses a transition into the
  configuration it chose and a `select()` shows the branch taken.
- The evaluator keys its sets by `Label`. A configured target gets a label of
  its own, the label with its number after a `\u{1}` in the repository name
  (`configured_graph.rs`); nothing outside that module sees the number.
  Numbers follow label then checksum order, so the evaluator's label order is
  the output order.
- The aquery filters `inputs()`, `outputs()` and `mnemonic()` are read off the
  top of the expression, as Bazel does: they pass their targets through, and
  only a filter at the top, or nested in the expression of another, filters
  the actions that are printed. A filter under `+`, `^` or `-` filters
  nothing, and as the argument of another function it is an error.

## Output formats

Everything is written through two layers. `fjfj-query::output` renders the
text formats from a `Graph`; `fjfj-query::proto` is a message tree
(`Msg`) with a binary, a text and a JSON writer, so no `.proto` is compiled.
The schemas (`build.proto`, `analysis_v2.proto`) are in the code that builds
the messages (`target_proto.rs`, `aquery.rs`); `protoc --decode` against the
protos in `@bazel_tools//src/main/protobuf` reads what fjfj writes.

| command | formats |
|---|---|
| query | label, label_kind, location, minrank, maxrank, graph, xml, build, package, proto, streamed_proto, streamed_jsonproto |
| cquery | label, label_kind, graph, build, files, starlark (`--starlark:expr`, `--starlark:file`, `providers()`, `build_options()`), proto, streamed_proto, textproto, jsonproto, `--transitions=lite|full` |
| aquery | text, commands, summary, proto, streamed_proto, textproto, jsonproto |

Flags that shape them: `--proto:` (`flatten_selects`, `default_values`,
`rule_inputs_and_outputs`, `locations`, `output_rule_attrs`,
`instantiation_stack`, `definition_stack`, `include_configurations`,
`include_attribute_source_aspects`), `--graph:factored`, `--graph:node_limit`,
`--include_commandline`, `--include_artifacts`, `--include_file_write_contents`,
`--include_aspects`, `--consistent_labels`, `--relative_locations`,
`--query_file`, `--output_file`, `--line_terminator_null`,
`--universe_scope`.

## What is not Bazel's, and why

These are known; each has a bead.

- **Configuration checksum digits** (`buildfiji-5at`). Bazel hashes the
  serialised options of every fragment. fjfj hashes its own `Configuration`
  (`Configuration::checksum`, SHA-256), shows seven digits in cquery and all
  of them in aquery, and they tell configurations apart as Bazel's do. A
  conformance diff masks them.
- **Hash order.** Bazel's order of the configured targets of a cquery or
  aquery, of the entries of an aquery summary, of the messages of aquery
  textproto and of the labels merged into a factored graph node depends on
  hashes (`buildfiji-ddw`, `buildfiji-c19`). fjfj prints label order. Bazel's
  own order changes from one run to the next for the graph.
- **Dependency sets** (`buildfiji-gon`). An aquery proto lists the inputs of
  an action as one set; Bazel nests them (a genrule's setup script, its
  sources and its tools are three).
- **`$rule_implementation_hash`** is a digest of the rule's `.bzl` and name,
  not Bazel's.
- **`build_options()`** has what a `Configuration` holds, not Bazel's ~330
  options (`buildfiji-lcc`). `--transitions=full` lists the native options of
  that table that changed.
- **Targets Bazel adds as dependencies**: the target platform and the targets
  it reaches (`buildfiji-ihl`).
- **Runfiles.** fjfj makes one action; aquery lists the four Bazel does
  (`RepoMappingManifest`, `SourceSymlinkManifest`, `SymlinkTree`,
  `RunfilesTree`) from it.

## Probing Bazel

Run `bazel shutdown` between probes. The server keeps what earlier commands
analysed: after an `--aspects` run, `bazel aquery //:a` lists the aspect's
actions; after analysing a genrule that uses a target as a tool, `bazel cquery
//:tool` lists the tool in its exec configuration too. A probe that disagrees
with fjfj is a stale server until a fresh one says otherwise.
