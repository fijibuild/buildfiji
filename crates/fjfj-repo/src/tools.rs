//! The files of `@bazel_tools` that fjfj serves (see `embedded_tools/README.md`).

use std::path::Path;

/// `@bazel_tools` as (path, contents), copied from Bazel 9.2.0's install.
pub const BAZEL_TOOLS_FILES: &[(&str, &str)] = &[
    (
        "tools/build_defs/repo/BUILD.bazel",
        include_str!("../embedded_tools/tools/build_defs/repo/BUILD.bazel.in"),
    ),
    (
        "tools/build_defs/repo/cache.bzl",
        include_str!("../embedded_tools/tools/build_defs/repo/cache.bzl"),
    ),
    (
        "tools/build_defs/repo/git.bzl",
        include_str!("../embedded_tools/tools/build_defs/repo/git.bzl"),
    ),
    (
        "tools/build_defs/repo/git_worker.bzl",
        include_str!("../embedded_tools/tools/build_defs/repo/git_worker.bzl"),
    ),
    (
        "tools/build_defs/repo/http.bzl",
        include_str!("../embedded_tools/tools/build_defs/repo/http.bzl"),
    ),
    (
        "tools/build_defs/repo/local.bzl",
        include_str!("../embedded_tools/tools/build_defs/repo/local.bzl"),
    ),
    (
        "tools/build_defs/repo/utils.bzl",
        include_str!("../embedded_tools/tools/build_defs/repo/utils.bzl"),
    ),
];

/// Write `@bazel_tools` into `dir`, leaving files that are already the right
/// ones alone.
pub fn materialize_bazel_tools(dir: &Path) -> std::io::Result<()> {
    for (path, contents) in BAZEL_TOOLS_FILES {
        let file = dir.join(path);
        if std::fs::read_to_string(&file).is_ok_and(|existing| existing == *contents) {
            continue;
        }
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&file, contents)?;
    }
    // A repository's root package exists.
    let root = dir.join("BUILD.bazel");
    if !root.exists() {
        std::fs::write(root, "")?;
    }
    Ok(())
}
