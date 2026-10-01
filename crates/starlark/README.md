# starlark (vendored)

starlark-rust 0.14.2 (Buck2's Starlark evaluator), copied here and edited so it behaves as Bazel 9.2.0 does.
`crates/starlark_syntax` is its parser crate, vendored alongside. Do not restyle these files: upstream's
formatting keeps a rebase onto a new release readable. Every difference from upstream is listed in
`docs/design/starlark-and-loading.md` ("Carrying changes to the `starlark` crate").
