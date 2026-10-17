# Inspect a module graph

Use `fjfj mod` to see what your `MODULE.bazel` resolves to. Run it from the
directory containing `MODULE.bazel`; fjfj does not search upward for a
workspace root yet.

fjfj's logging goes to stderr and shares your terminal. Set `RUST_LOG=warn`
to quiet it, or redirect stderr (`2>/dev/null`) when piping stdout.

## See the whole graph

```sh
fjfj mod graph
```

Prints an indented tree. A module that several others depend on appears
under each of them.

For machine-readable output, in Bazel's own JSON shape:

```sh
fjfj mod graph --output=json
```

Each node has `key`, `name`, `version`, `apparentName`, `dependencies`,
`indirectDependencies` and `cycles`; only the root carries `root: true`.
For example, to list every selected module with its version:

```sh
fjfj mod graph --output=json 2>/dev/null \
  | jq -r '.. | objects | select(.key?) | "\(.name) \(.version)"' | sort -u
```

## See one module's dependencies

```sh
fjfj mod deps rules_rust platforms
```

Takes one or more module names. For each it prints the module's own
`bazel_dep` edges and what each resolved to, including the version that was
originally *requested* when selection picked a different one.

## Find out why a version was selected

```sh
fjfj mod explain platforms
```

Prints the selected version, then every module in the *unpruned* graph that
asked for `platforms` and which version it asked for. Use it when a version
is higher than you wrote in your own `bazel_dep`: some other module
requested it.

## See where a repository came from

```sh
fjfj mod show_repo platforms
```

Prints the module key, name, version, apparent and canonical repository
names, and the registry it was fetched from. It does not print the
repository rule's attributes; those need the repository to be fetched, which
fjfj cannot do yet.

## Change how the graph resolves

`mod` accepts the bzlmod flags, so you can ask "what if":

```sh
fjfj mod graph --ignore_dev_dependency
fjfj mod graph --allow_yanked_versions=all
```

See [Resolve modules against a local registry or
override](use-a-local-registry-or-override.md) for `--registry` and
`--override_module`, and [Flags](../reference/flags.md#bzlmod) for all four.

## Limits

- Only the JSON form of `graph` is checked for conformance against
  `bazel mod graph`. The text form of `graph`, and the text of `deps`,
  `show_repo` and `explain`, are fjfj's own format. Do not parse them.
- Resolution contacts the registry (`https://bcr.bazel.build` by default),
  so `mod` needs network access unless you supply a local `--registry`.
