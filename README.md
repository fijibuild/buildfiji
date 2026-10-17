# fjfj

A Bazel-compatible build tool written in Rust. Same `MODULE.bazel`, same
`BUILD` files, same flags, same remote cache; `bazel build //...` and
`fjfj build //...` both work on this repository.

```sh
cargo build && ./target/debug/fjfj version
bazel build //... && bazel test //...
bd ready   # planned work
```

Documentation is in `docs/` (see `docs/README.md`): tutorials, how-to guides,
reference and explanation. For the shape of the project, see
`docs/ARCHITECTURE.md` and `docs/design/`.
