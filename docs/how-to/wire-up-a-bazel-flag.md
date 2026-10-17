# Wire up a Bazel flag

This guide is for contributors. It takes a flag that fjfj currently rejects
with `recognized by Bazel ... but not implemented by fjfj yet` and makes it
real. [The compatibility model](../explanation/compatibility-model.md)
explains why the gate exists.

Claim a bead first (`bd ready`, `bd update <id> --claim`), and follow
`CLAUDE.md` for labels and the build gate.

## 1. Confirm the flag is in the table

Every flag Bazel 9.2.0 accepts is in
`crates/fjfj-bazel-compat/src/bazel_flags/generated.rs`:

```sh
grep -n '"jobs"' crates/fjfj-bazel-compat/src/bazel_flags/generated.rs
```

That file is generated; do not edit it by hand. After a Bazel version bump,
regenerate it with a `bazel` on `PATH` at the pinned version:

```sh
cargo run -p fjfj-bazel-compat --bin refresh_bazel_flags
```

## 2. Pick or create a `*_flags` module

Flags are grouped by concern, one module each under
`crates/fjfj-bazel-compat/src/` (`diagnostics_flags`, `remote_flags`,
`bzlmod_flags`, ...). Add to the module that fits, or create a new one and
declare it in `lib.rs`. [`misc_flags.rs`](../../crates/fjfj-bazel-compat/src/misc_flags.rs)
is the smallest complete example. A module has three parts:

1. `pub const IMPLEMENTED: &[&str]` — the flag names (without `--`) this
   module reads. Add yours.
2. A flags struct whose `Default` is **Bazel's own default**.
3. `pub fn extract(args: &[String], command: &str) -> (Flags, Vec<String>)`
   — resolve each token through `FlagRegistry::global()`, match on
   `m.flag.name`, and return every token you did not consume, in order.
   Handle both `--flag=value` and `--flag value`, and `m.negated` for
   `--noflag`.

## 3. Register it with the command

In `crates/fjfj-cli/src/lib.rs`:

1. Add your module's `IMPLEMENTED` to the `BUILD_IMPLEMENTED` array near the
   bottom of `run`'s file. Until you do, `clap_flags::validate` keeps
   rejecting the flag. (A flag only `run` reads is added to the list inside
   `build_main` instead, as `run_flags` is.)
2. In `build_main`, call `your_flags::extract(&rest, command)` alongside the
   others and keep the returned `rest`. `drop_build_family_flags` is the
   matching list for `cquery` and `aquery`, which accept but ignore the build
   flags; add yours there if they should too.
3. Use the value. A flag that is extracted and then ignored is the failure
   the gate exists to prevent; if it is not used yet, say so in the bead.

Do other commands need it? Bazel's table lists which commands accept each
flag. `test`, `run`, `cquery` and `aquery` share `build_main`'s checks;
`query`, `clean`, `info`, `fetch` and `mod` each parse their own.

## 4. Test it

Put unit tests in the module, as the existing ones do: Bazel's default, the
attached and space-separated spellings, the negated form if there is one,
and that unrelated tokens pass through in order. Use the generated table to
check spellings; Bazel's observable behaviour is the spec, so when in doubt
run `bazel` and match it.

## 5. Run the gate

```sh
cargo fmt --all
bazel build //... && bazel test //...
```

Clippy and rustfmt run as aspects, so a lint failure appears in the build.
If you added a dependency, follow the repin steps in the `bazel` skill.

## 6. Record the decision

If you chose behaviour that is not obvious from Bazel (a default, an
ordering, an edge case), add a short dated section to
`docs/design/cli-compat.md` and close the bead. Do not copy it into
`Reference`; add the flag to [Flags](../reference/flags.md) so the table
there stays complete.
