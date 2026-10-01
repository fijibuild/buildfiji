//! The directory actions run in (buildfiji-fyz.1, first strategy).
//!
//! Bazel runs an action in `<output base>/execroot/_main`, where the main
//! repository's top-level entries are symlinks to the workspace, `external/`
//! holds a link to each repository, and `bazel-out/` is where outputs are
//! made. A command names its inputs and outputs by their paths there.

use std::io;
use std::path::{Path, PathBuf};

/// Where things are on disk.
#[derive(Debug, Clone)]
pub struct Layout {
    /// The main repository's root.
    pub workspace: PathBuf,
    pub output_base: PathBuf,
}

/// What Bazel calls the main repository's directory in the execroot.
pub const MAIN_REPO_DIR: &str = "_main";

impl Layout {
    pub fn execroot(&self) -> PathBuf {
        self.output_base.join("execroot").join(MAIN_REPO_DIR)
    }

    pub fn bazel_out(&self) -> PathBuf {
        self.execroot().join("bazel-out")
    }

    /// Where the repositories that are not the main one are made.
    pub fn external(&self) -> PathBuf {
        self.output_base.join("external")
    }

    /// Make the execroot, or bring it up to date: a link for each top-level
    /// entry of the workspace (but the convenience links `bazel-*`) and for
    /// each repository made so far.
    pub fn prepare(&self) -> io::Result<()> {
        let execroot = self.execroot();
        std::fs::create_dir_all(self.bazel_out())?;
        for entry in std::fs::read_dir(&self.workspace)? {
            let entry = entry?;
            let name = entry.file_name();
            let Some(text) = name.to_str() else { continue };
            if text.starts_with("bazel-") || text == "bazel-out" {
                continue;
            }
            link(&entry.path(), &execroot.join(&name))?;
        }
        let external = execroot.join("external");
        std::fs::create_dir_all(&external)?;
        // Links to repositories that were removed would dangle, and a stale
        // one would hide a repository made again.
        for entry in std::fs::read_dir(&external)? {
            let entry = entry?;
            if !entry.path().exists() {
                std::fs::remove_file(entry.path())?;
            }
        }
        if let Ok(repos) = std::fs::read_dir(self.external()) {
            for entry in repos {
                let entry = entry?;
                let name = entry.file_name();
                if name.to_str().is_some_and(|n| n.starts_with('.')) {
                    continue;
                }
                link(&entry.path(), &external.join(&name))?;
            }
        }
        Ok(())
    }
}

impl Layout {
    /// The links `bazel build` leaves in the workspace: `bazel-bin`,
    /// `bazel-out`, `bazel-testlogs` and `bazel-<workspace name>`, each
    /// `<prefix><name>`. `mnemonic` is the configuration, `k8-fastbuild`.
    pub fn convenience_links(&self, prefix: &str, mnemonic: &str) -> io::Result<()> {
        let workspace_name = self.workspace.file_name().map_or_else(
            || "workspace".to_owned(),
            |n| n.to_string_lossy().into_owned(),
        );
        let out = self.bazel_out();
        for (name, target) in [
            ("bin", out.join(mnemonic).join("bin")),
            ("out", out.clone()),
            ("testlogs", out.join(mnemonic).join("testlogs")),
            (workspace_name.as_str(), self.execroot()),
        ] {
            link(&target, &self.workspace.join(format!("{prefix}{name}")))?;
        }
        Ok(())
    }
}

/// `to` as a symlink to `from`, replacing a symlink that points elsewhere.
fn link(from: &Path, to: &Path) -> io::Result<()> {
    match std::fs::read_link(to) {
        Ok(current) if current == from => return Ok(()),
        Ok(_) => std::fs::remove_file(to)?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        // Something that is not a link is in the way.
        Err(_) => return Ok(()),
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(from, to)
    }
    #[cfg(not(unix))]
    {
        let _ = (from, to);
        Err(io::Error::other("symlinks need a Unix host"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_execroot_links_the_workspace_and_the_repositories() {
        let dir = tempfile::tempdir().unwrap();
        let layout = Layout {
            workspace: dir.path().join("ws"),
            output_base: dir.path().join("ob"),
        };
        std::fs::create_dir_all(layout.workspace.join("a")).unwrap();
        std::fs::write(layout.workspace.join("in.txt"), "x").unwrap();
        std::os::unix::fs::symlink("/nowhere", layout.workspace.join("bazel-bin")).unwrap();
        std::fs::create_dir_all(layout.external().join("dep+")).unwrap();
        layout.prepare().unwrap();
        let root = layout.execroot();
        assert_eq!(std::fs::read_to_string(root.join("in.txt")).unwrap(), "x");
        assert!(root.join("a").is_dir());
        assert!(root.join("external/dep+").is_dir());
        assert!(root.join("bazel-out").is_dir());
        assert!(!root.join("bazel-bin").exists());
        layout.convenience_links("bazel-", "k8-fastbuild").unwrap();
        let bin = std::fs::read_link(layout.workspace.join("bazel-bin")).unwrap();
        assert_eq!(bin, root.join("bazel-out/k8-fastbuild/bin"));
        assert_eq!(
            std::fs::read_link(layout.workspace.join("bazel-ws")).unwrap(),
            root
        );
        // Again, after a repository went away.
        std::fs::remove_dir(layout.external().join("dep+")).unwrap();
        layout.prepare().unwrap();
        assert!(!root.join("external/dep+").exists());
        assert!(std::fs::symlink_metadata(root.join("external/dep+")).is_err());
    }
}
