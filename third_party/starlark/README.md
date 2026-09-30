# Patches to the `starlark` crate

fjfj runs `starlark` 0.14.2 (Buck2's evaluator) and Bazel's behaviour is the spec.
Where the two differ in a way only a change inside the crate can fix, the change is a
patch here, applied by Bazel to the crate it downloads: `crate.annotation(crate =
"starlark", patches = [...], patch_args = ["-p1"])` in `MODULE.bazel`. Nothing is
forked or vendored. Cargo does not build fjfj (it only keeps `Cargo.toml` and
`Cargo.lock`), so it never sees them.

Make a patch against the crate's source in the cargo registry:

```sh
R=$(ls -d ~/.cargo/registry/src/*/starlark-0.14.2)
cp -r "$R" /tmp/a && cp -r "$R" /tmp/b
# edit /tmp/b
(cd /tmp && diff -u a/src/... b/src/... | sed 's|^--- a/|--- a/|') > third_party/starlark/NNNN-what.patch
```

A patch that changes more than one file needs a `diff --git a/x b/x` line before
each file's `---`/`+++` lines: Bazel's patch applies only the last file of a plain
multi-file diff. Number them in the order they apply, say in the patch's first lines which bead needs
it and whether it went upstream, add it to the annotation in `MODULE.bazel`, and
replay the probe row that was skipped for it. When the crate is upgraded, a patch
that no longer applies fails the build, which is the reminder.

| patch | bead | what |
|---|---|---|
| 0001-percent-s-is-str | buildfiji-b9c | `"%s" % x` formats a non-string with `str()`, not `repr()` |
| 0002-no-recursion | buildfiji-2r5 | a `def` called while it is already being called is `function 'f' called recursively` |
