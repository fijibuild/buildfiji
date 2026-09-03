---
name: bazel
description: Building this repo with Bazel — the build and test gate, what the clippy/rustfmt aspects do, how per-crate BUILD.bazel files are written and kept in step with Cargo.toml, MODULE.bazel and repinning, and dogfooding fjfj on itself. Use when adding a crate or target, editing BUILD.bazel or MODULE.bazel, or diagnosing a build failure.
---

# Building buildfiji with Bazel

```bash
bazel build //... && bazel test //...   # the gate
bazel build -c opt //...                # for anything you intend to measure
bazel run //:fjfj -- build //...        # dogfood fjfj on this repo
```

Bazel 9.2.0 is the base compatibility target for what fjfj *implements*; it
is also the version this repo builds under.

## Aspects

Clippy and rustfmt run as `rules_rust` aspects on every `bazel build`, wired
once in `.bazelrc` and applied to all targets. Never add per-target clippy or
rustfmt rules. A build that fails with a rustfmt diff is fixed by
`cargo fmt --all`, not by editing the aspect.

## Per-crate BUILD.bazel

One `rust_library` (or `rust_binary`) plus a `rust_test` on the same crate:

```python
load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")

package(default_visibility = ["//visibility:public"])

rust_library(
    name = "fjfj-thing",
    srcs = glob(["src/**/*.rs"]),
    crate_name = "fjfj_thing",
    edition = "2024",
    version = "0.0.1",
    deps = ["@crates//:anyhow", "//crates/fjfj-graph"],
)

rust_test(
    name = "fjfj-thing_test",
    crate = ":fjfj-thing",
)
```

`srcs` is a glob, so new source files need no BUILD edit. **`deps` is not
generated**: external crates are `@crates//:<name>`, internal ones
`//crates/<crate>`, and both are maintained by hand to match `Cargo.toml`.

## MODULE.bazel and repinning

Dependencies come from `crate_universe` over the Cargo workspace. After
changing any `Cargo.toml`:

```bash
CARGO_BAZEL_REPIN=1 bazel build //...
```

which refreshes `Cargo.lock` and `MODULE.bazel.lock`; commit both. Then fix
the crate's `BUILD.bazel` `deps` by hand. A new crate under `crates/` joins
the Cargo workspace automatically (`members = ["crates/*"]`) but still needs
its own `BUILD.bazel`.

Toolchains — Rust and the Lean release artifacts — are pinned per platform in
`MODULE.bazel`; keep the Rust pin in step with `rust-toolchain.toml`.

## Spike crates

A spike lives in `crates/fjfj-spike-<topic>` as a `rust_binary`, with a
comment naming its bead, and is removed once its results are recorded in
`docs/design/`. The design doc cites the commit that still contains it.
