# Aspects

## What Bazel provides
An aspect is a function applied along the dependency edges of a configured
target graph. `aspect(implementation, attr_aspects=[...], attrs=..., provides=...,
required_providers=..., toolchains=...)` defines one; it propagates along the
named attributes, sees the target's providers and rule attributes, and returns
its own providers plus optional `OutputGroupInfo`. They are invoked from the
command line (`--aspects=//a.bzl%my_aspect --output_groups=...`) or required
by rules (`attr.label_list(aspects=[...])`). IDE integration (IntelliJ,
rules_ide), linting (`rules_lint`), coverage, and `bazel-compile-commands`
all depend on aspects.

## What compatibility means
- Aspect keys: `(configured target, aspect, aspect parameters)`; aspects
  on aspects (`required_aspect_providers`) must be supported.
- Aspects run in analysis, so they are nodes in the incremental engine and
  are memoised the same way configured targets are.
- Output groups from aspects must be buildable and appear in BEP/query.
- `cquery --aspects` and `aquery` need to see aspect-generated actions.

## Design sketch
`fjfj-graph::AspectKey { target: ConfiguredTargetKey, aspect: AspectId, params }`.
The analysis function for an aspect is the same as for a rule, with a
`ctx.rule` view instead of raw attributes. Propagation is computed once per
`(aspect, rule kind)` from `attr_aspects`.

## Open questions
- Aspect parameter typing (Bazel only allows string/int/bool with restricted values).
- Cycle detection with aspect-on-aspect.
- Toolchain resolution inside aspects (`toolchains=` on aspects).
