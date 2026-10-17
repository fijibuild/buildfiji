# Resolve modules against a local registry or override

Two flags change where fjfj finds modules without editing `MODULE.bazel`.
Both work on `fjfj build` and `fjfj mod`.

## Use a registry other than the Bazel Central Registry

```sh
fjfj mod graph --registry=file:///path/to/registry
fjfj mod graph --registry=https://registry.example.com
```

A registry is a directory (or URL) with a `bazel_registry.json` and a
`modules/<name>/<version>/MODULE.bazel` tree. The repository's own test
fixture, `crates/fjfj-bzlmod/tests/fixtures/registry`, is a small working
example.

`--registry` is repeatable, and **any use of it replaces the default**
`https://bcr.bazel.build` rather than adding to it, exactly as in Bazel. To
search your registry first and fall back to the central one, name both:

```sh
fjfj mod graph \
  --registry=file:///path/to/registry \
  --registry=https://bcr.bazel.build
```

Registries are consulted in the order given.

## Replace a module with a local directory

```sh
fjfj mod graph --override_module=foo=../foo
```

The value is `name=path`. fjfj treats `foo` as a `local_repository` at that
path for this invocation only, as if `MODULE.bazel` had a
`local_path_override`. Repeat the flag for several modules. A value with no
`=` is rejected rather than ignored.

## Allow yanked versions

A registry can mark a version as yanked, and resolution fails if it selects
one. To proceed anyway:

```sh
fjfj mod graph --allow_yanked_versions=all
fjfj mod graph --allow_yanked_versions=foo@1.2.3,bar@2.0
```

`all` allows every yanked version; otherwise give a comma-separated list of
`name@version`.

## Make it permanent

These are ordinary Bazel flags, so the same lines work in a `.bazelrc`
shared with Bazel:

```
common --registry=https://bcr.bazel.build
common --registry=file:///path/to/registry
```

Note that fjfj's `--bazelrc` startup option and `.bazelrc` discovery are
parsed by `fjfj-bazel-compat`, but are not yet applied to commands. Until
they are, pass flags on the command line.
