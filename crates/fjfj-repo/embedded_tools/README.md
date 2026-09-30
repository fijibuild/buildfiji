# The files of `@bazel_tools` that fjfj runs

Copied from `embedded_tools/` of the Bazel 9.2.0 install (`bazel info
install_base`), Apache-2.0 like Bazel: the repository rules a module file
names by `use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl",
"http_archive")`. Bazel serves the whole directory as the repository
`@bazel_tools`; fjfj serves these files until it ships the rest
(buildfiji-mum.23, buildfiji-mum.12). Refresh by copying again; the packages'
`BUILD.bazel` files are the ones Bazel has, kept here as `BUILD.bazel.in` so that
this directory is not a package of this repository.
