# Incremental engine

Bazel's Skyframe and Buck2's DICE are memoising, demand-driven key/value
graphs with dependency tracking and invalidation. fjfj needs the same to
support a persistent server, `--watchfs`, and fast no-op builds.

Options to evaluate (spike):
1. Buck2's `dice` crate (not on crates.io as of this writing; vendor or git dep).
2. `salsa` (used by rust-analyzer): mature, Rust-native, but single-threaded
   revision model may not suit a highly parallel build.
3. Custom: keys are enums (`PackageKey`, `ConfiguredTargetKey`, `AspectKey`,
   `ActionKey`, `FileKey`), values are `Arc<dyn Any>`, with a versioned
   dependency graph. Highest control, highest cost.

Decision criteria: parallelism, cancellation, memory footprint on 100k+
targets, and ease of expressing Bazel's cycle detection and error
propagation semantics.
