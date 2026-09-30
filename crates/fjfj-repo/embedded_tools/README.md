# `@bazel_tools`

`../embedded_tools.tar.zst` is `embedded_tools/` of the Bazel 9.2.0 install (`bazel
info install_base`) without `jdk/` (the JRE, 69 MB): the repository `@bazel_tools`, as
Bazel serves it (Apache-2.0, like Bazel). fjfj extracts it into
`<output base>/external/bazel_tools` (`src/tools.rs`); `MODULE.bazel` in it is the real
one. Refresh it with

```sh
IB=$(bazel info install_base)/embedded_tools
(cd "$IB" && tar --exclude=./jdk --sort=name --mtime=@0 --owner=0 --group=0 \
  --numeric-owner --mode=u+rw,go+r-w -cf - .) | zstd -19 -q -o crates/fjfj-repo/embedded_tools.tar.zst
```

and copy its `MODULE.bazel` to `crates/fjfj-bzlmod/src/bazel_tools.MODULE.bazel`.
