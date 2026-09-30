//! `@bazel_tools`: the files Bazel 9.2.0 ships in its binary
//! (`embedded_tools.tar.zst`, see `embedded_tools/README.md`), put in a
//! directory of the output base.

use sha2::Digest as _;
use std::path::Path;

/// The archive of `@bazel_tools`.
pub const BAZEL_TOOLS_ARCHIVE: &[u8] = include_bytes!("../embedded_tools.tar.zst");

/// What the directory holds when it is the archive's: the archive's digest.
const MARKER: &str = ".fjfj-bazel-tools";

/// Put `@bazel_tools` in `dir`, unless it is there already.
pub fn materialize_bazel_tools(dir: &Path) -> std::io::Result<()> {
    let digest = hex::encode(sha2::Sha256::digest(BAZEL_TOOLS_ARCHIVE));
    let marker = dir.join(MARKER);
    if std::fs::read_to_string(&marker).is_ok_and(|text| text == digest) {
        return Ok(());
    }
    // Whatever was there is an older one, or another process is at it.
    if dir.exists() {
        std::fs::remove_dir_all(dir)?;
    }
    let parent = dir.parent().unwrap_or(dir);
    std::fs::create_dir_all(parent)?;
    let archive = parent.join(format!(".bazel_tools-{}.tar.zst", std::process::id()));
    std::fs::write(&archive, BAZEL_TOOLS_ARCHIVE)?;
    let made = fjfj_archive::extract(&fjfj_archive::ExtractRequest {
        archive: &archive,
        format: fjfj_archive::Format::TarZst,
        output: dir,
        strip_prefix: "",
        strip_components: 0,
        rename: &[],
    });
    let _ = std::fs::remove_file(&archive);
    made.map_err(std::io::Error::other)?;
    // A repository's root package exists.
    let root = dir.join("BUILD.bazel");
    if !root.exists() && !dir.join("BUILD").exists() {
        std::fs::write(root, "")?;
    }
    std::fs::write(marker, digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_archive_is_extracted_once_and_holds_the_repository_rules() {
        let dir = tempfile::tempdir().unwrap();
        let tools = dir.path().join("external/bazel_tools");
        materialize_bazel_tools(&tools).unwrap();
        for file in [
            "MODULE.bazel",
            "tools/build_defs/repo/http.bzl",
            "tools/build_defs/repo/local.bzl",
            "tools/cpp/cc_configure.bzl",
            "tools/osx/xcode_configure.bzl",
        ] {
            assert!(tools.join(file).is_file(), "{file}");
        }
        // Left alone when it is already there.
        std::fs::write(tools.join("tools/extra.txt"), "x").unwrap();
        materialize_bazel_tools(&tools).unwrap();
        assert!(tools.join("tools/extra.txt").exists());
    }
}
