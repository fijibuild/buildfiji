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

    /// Where an artifact is on disk: a source file where its repository has
    /// it, an output in the execroot.
    pub fn resolve(&self, artifact: &fjfj_graph::Artifact) -> PathBuf {
        if !artifact.is_source() {
            return self.execroot().join(artifact.exec_path());
        }
        match artifact.root.prefix.strip_prefix("external/") {
            Some(repo) => self.external().join(repo).join(&artifact.path),
            None => self.workspace.join(&artifact.path),
        }
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
            let _ = link(&entry.path(), &execroot.join(&name))?;
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
                let _ = link(&entry.path(), &external.join(&name))?;
            }
        }
        Ok(())
    }
}

impl Layout {
    /// The links `bazel build` leaves in the workspace: `bazel-bin`,
    /// `bazel-out`, `bazel-testlogs` and `bazel-<workspace name>`, each
    /// `<prefix><name>`. `mnemonic` is the configuration, `k8-fastbuild`.
    ///
    /// A prefix of `/` asks for no links; a prefix with a directory in it
    /// (`out/x-`) makes the directory. A link that cannot be made because a
    /// file or directory is in the way is not an error: the returned lines say
    /// which, as Bazel's warning does.
    pub fn convenience_links(&self, prefix: &str, mnemonic: &str) -> io::Result<Vec<String>> {
        let mut failed = Vec::new();
        if prefix == "/" {
            return Ok(failed);
        }
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
            let at = self.workspace.join(format!("{prefix}{name}"));
            if let Some(dir) = at.parent() {
                std::fs::create_dir_all(dir)?;
            }
            if let Err(why) = link(&target, &at)? {
                failed.push(format!(
                    "  cannot create symbolic link {prefix}{name} -> {}:  {} {why}",
                    target.display(),
                    at.display()
                ));
            }
        }
        Ok(failed)
    }
}

/// What `clean` did that the command says.
#[derive(Debug, PartialEq, Eq)]
pub enum Cleaned {
    /// The tree is gone.
    Removed,
    /// The tree was renamed to this and is being deleted in the background.
    Moved(PathBuf),
}

impl Layout {
    /// `bazel clean`: the convenience links go, the action cache goes, and so
    /// does the execroot, or with `expunge` the whole output base. With
    /// `asynchronous` the tree is renamed (`<tree>_tmp_<unique>`) and deleted
    /// by a process that outlives this one.
    pub fn clean(
        &self,
        prefix: &str,
        expunge: bool,
        asynchronous: bool,
        unique: &str,
    ) -> io::Result<Cleaned> {
        // Links that lead into the output base, which a build made.
        if prefix != "/" {
            let workspace_name = self.workspace.file_name().map_or_else(
                || "workspace".to_owned(),
                |n| n.to_string_lossy().into_owned(),
            );
            for name in [
                "bin",
                "out",
                "testlogs",
                "genfiles",
                workspace_name.as_str(),
            ] {
                let at = self.workspace.join(format!("{prefix}{name}"));
                if std::fs::read_link(&at).is_ok_and(|to| to.starts_with(&self.output_base)) {
                    std::fs::remove_file(&at)?;
                }
            }
        }
        let cache = self.output_base.join("fjfj-action-cache.json");
        match std::fs::remove_file(&cache) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
        let tree = if expunge {
            self.output_base.clone()
        } else {
            self.output_base.join("execroot")
        };
        if !tree.exists() {
            return Ok(Cleaned::Removed);
        }
        if !asynchronous {
            if expunge {
                crate::run::remove(&tree)?;
            } else {
                crate::run::remove(&tree.join(MAIN_REPO_DIR))?;
            }
            return Ok(Cleaned::Removed);
        }
        let mut moved = tree.clone().into_os_string();
        moved.push(format!("_tmp_{unique}"));
        let moved = PathBuf::from(moved);
        std::fs::rename(&tree, &moved)?;
        // A process of its own, which carries on after this one exits.
        std::process::Command::new("sh")
            .arg("-c")
            .arg("chmod -R u+w \"$1\"; rm -rf \"$1\"")
            .arg("fjfj-clean")
            .arg(&moved)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
        Ok(Cleaned::Moved(moved))
    }
}

/// `to` as a symlink to `from`, replacing a symlink that points elsewhere.
/// `Ok(Err(why))` when something that is not a link is in the way.
fn link(from: &Path, to: &Path) -> io::Result<Result<(), &'static str>> {
    match std::fs::read_link(to) {
        Ok(current) if current == from => return Ok(Ok(())),
        Ok(_) => std::fs::remove_file(to)?,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(_) => return Ok(Err("is not a symlink")),
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(from, to).map(Ok)
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
        assert!(
            layout
                .convenience_links("bazel-", "k8-fastbuild")
                .unwrap()
                .is_empty()
        );
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

    #[test]
    fn a_slash_prefix_makes_no_links_a_directory_prefix_makes_the_directory_and_an_obstacle_is_reported()
     {
        let dir = tempfile::tempdir().unwrap();
        let layout = Layout {
            workspace: dir.path().join("ws"),
            output_base: dir.path().join("ob"),
        };
        std::fs::create_dir_all(&layout.workspace).unwrap();
        assert!(layout.convenience_links("/", "k8-opt").unwrap().is_empty());
        assert_eq!(std::fs::read_dir(&layout.workspace).unwrap().count(), 0);
        layout.convenience_links("o/x-", "k8-opt").unwrap();
        assert!(
            std::fs::read_link(layout.workspace.join("o/x-testlogs"))
                .unwrap()
                .ends_with("k8-opt/testlogs")
        );
        std::fs::create_dir(layout.workspace.join("bazel-bin")).unwrap();
        let failed = layout.convenience_links("bazel-", "k8-opt").unwrap();
        assert_eq!(failed.len(), 1);
        assert!(failed[0].starts_with("  cannot create symbolic link bazel-bin -> "));
        assert!(failed[0].ends_with("bazel-bin is not a symlink"));
    }
}
