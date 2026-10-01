//! Where a command was run from and how its arguments divide (buildfiji-gwl.3).

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The files whose presence marks a workspace root, as Bazel 9.2.0 looks for
/// them.
const ROOT_MARKERS: [&str; 4] = ["MODULE.bazel", "REPO.bazel", "WORKSPACE", "WORKSPACE.bazel"];

/// What Bazel says when it finds no root; the command is the one run.
pub(crate) fn not_in_a_workspace(command: &str) -> String {
    format!(
        "The '{command}' command is only supported from within a workspace (below a directory \
         having a MODULE.bazel file).\nSee documentation at \
         https://bazel.build/concepts/build-ref#workspace"
    )
}

/// The workspace root at or above `cwd` and `cwd`'s path under it, `/`
/// separated and empty at the root.
pub(crate) fn locate(cwd: &Path) -> Option<(PathBuf, String)> {
    let root = cwd
        .ancestors()
        .find(|dir| ROOT_MARKERS.iter().any(|m| dir.join(m).is_file()))?;
    let offset = cwd
        .strip_prefix(root)
        .ok()?
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/");
    Some((root.to_path_buf(), offset))
}

/// The command-line parser drops the first `--`, which is what tells a
/// negative pattern from a flag. Writing it twice leaves one in the
/// positionals of `build`.
pub(crate) fn keep_end_of_options(args: Vec<OsString>) -> Vec<OsString> {
    let command = args
        .iter()
        .position(|a| a == "build" || a == "run" || a == "test");
    let marker = command.and_then(|c| args[c..].iter().position(|a| a == "--"));
    let mut args = args;
    if let (Some(c), Some(m)) = (command, marker) {
        args.insert(c + m, OsString::from("--"));
    }
    args
}

/// Arguments before the first `--` and the target patterns after it. The
/// parser keeps both of the markers [`keep_end_of_options`] wrote when an
/// argument came before them and one when none did, so a second marker right
/// after the first is that one.
pub(crate) fn split_end_of_options(args: &[String]) -> (&[String], &[String]) {
    match args.iter().position(|a| a == "--") {
        Some(i) => {
            let after = &args[i + 1..];
            let after = after.strip_prefix(&["--".to_owned()]).unwrap_or(after);
            (&args[..i], after)
        }
        None => (args, &[]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os(words: &[&str]) -> Vec<OsString> {
        words.iter().map(OsString::from).collect()
    }

    #[test]
    fn the_end_of_options_marker_survives_once_for_build() {
        assert_eq!(
            keep_end_of_options(os(&["fjfj", "build", "-k", "--", "//a", "-//a:b"])),
            os(&["fjfj", "build", "-k", "--", "--", "//a", "-//a:b"])
        );
        assert_eq!(
            keep_end_of_options(os(&["fjfj", "build", "//a"])),
            os(&["fjfj", "build", "//a"])
        );
        assert_eq!(
            keep_end_of_options(os(&["fjfj", "query", "--", "x"])),
            os(&["fjfj", "query", "--", "x"])
        );
    }

    #[test]
    fn arguments_split_at_the_first_marker() {
        let words = |w: &[&str]| w.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let args = words(&["-k", "//a", "--", "-//b", "--", "c"]);
        let (before, after) = split_end_of_options(&args);
        assert_eq!(before, ["-k", "//a"]);
        assert_eq!(after, ["-//b", "--", "c"]);
        // Both markers survived, or only the second.
        let args = words(&["-k", "--", "--", "-//b"]);
        assert_eq!(split_end_of_options(&args), (&args[..1], &args[3..]));
        let args = words(&["--", "-//b"]);
        assert_eq!(split_end_of_options(&args), (&args[..0], &args[1..]));
    }

    #[test]
    fn the_root_is_the_nearest_directory_with_a_marker() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("MODULE.bazel"), "").unwrap();
        std::fs::create_dir_all(dir.path().join("a/b")).unwrap();
        let (root, offset) = locate(&dir.path().join("a/b")).unwrap();
        assert_eq!((root.as_path(), offset.as_str()), (dir.path(), "a/b"));
        let (_, offset) = locate(dir.path()).unwrap();
        assert_eq!(offset, "");
    }
}
