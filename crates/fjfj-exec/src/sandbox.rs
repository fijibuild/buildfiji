//! The directory a command runs in when it is sandboxed (buildfiji-fyz.19),
//! as Bazel's `processwrapper-sandbox` makes it.
//!
//! `<output base>/sandbox/processwrapper-sandbox/<n>/execroot/_main` holds a
//! link to each declared input, where the command names it, and the
//! directories its declared outputs go in. The command sees nothing else of
//! the execroot. Once it has run, the declared outputs are moved to the real
//! execroot, and the rest of what it made goes with the directory.

use crate::execroot::Layout;
use crate::run::remove;
use fjfj_graph::Action;
use std::io;
use std::path::{Path, PathBuf};

pub(crate) struct Sandbox {
    pub root: PathBuf,
    /// The command's working directory.
    pub exec: PathBuf,
}

impl Sandbox {
    /// Make the directory for `action`.
    pub fn prepare(&self, layout: &Layout, action: &Action) -> io::Result<()> {
        std::fs::create_dir_all(&self.exec)?;
        // A directory's link before the links inside it.
        let mut inputs: Vec<_> = action
            .inputs
            .iter()
            .map(|input| (input.exec_path(), input))
            .collect();
        inputs.sort_by(|a, b| a.0.cmp(&b.0));
        inputs.dedup_by(|a, b| a.0 == b.0);
        for (path, input) in inputs {
            let at = self.exec.join(&path);
            if inside_a_link(&self.exec, &at) {
                continue;
            }
            if let Some(dir) = at.parent() {
                std::fs::create_dir_all(dir)?;
            }
            let real = layout.resolve(input);
            if real.is_dir() {
                // A directory input, a runfiles tree or a tree artifact, is
                // made of directories here, so a command can make files next
                // to what it holds, as under Bazel's sandbox.
                mirror(&real, &at)?;
            } else {
                match std::os::unix::fs::symlink(&real, &at) {
                    Err(e) if e.kind() != io::ErrorKind::AlreadyExists => return Err(e),
                    _ => {}
                }
            }
        }
        for out in &action.outputs {
            let at = self.exec.join(out.exec_path());
            if out.tree {
                std::fs::create_dir_all(&at)?;
            } else if let Some(dir) = at.parent() {
                std::fs::create_dir_all(dir)?;
            }
        }
        Ok(())
    }

    /// Move the outputs `action` made to where they belong.
    pub fn collect(&self, layout: &Layout, action: &Action) -> io::Result<()> {
        let execroot = layout.execroot();
        for out in &action.outputs {
            let path = out.exec_path();
            let from = self.exec.join(&path);
            if std::fs::symlink_metadata(&from).is_err() {
                continue;
            }
            let to = execroot.join(&path);
            remove(&to)?;
            std::fs::rename(&from, &to)?;
        }
        Ok(())
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = remove(&self.root);
    }
}

/// Whether a link lies between `root` and `at`: whatever is made at `at` would
/// be made in what the link names.
fn inside_a_link(root: &Path, at: &Path) -> bool {
    at.ancestors()
        .skip(1)
        .take_while(|dir| dir.starts_with(root) && *dir != root)
        .any(|dir| std::fs::symlink_metadata(dir).is_ok_and(|m| m.file_type().is_symlink()))
}

/// `to` as a directory holding a link to each file of the directory `from`,
/// and a directory of its own for each directory of it.
fn mirror(from: &Path, to: &Path) -> io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        if src.is_dir() {
            mirror(&src, &dst)?;
        } else {
            match std::os::unix::fs::symlink(&src, &dst) {
                Err(e) if e.kind() != io::ErrorKind::AlreadyExists => return Err(e),
                _ => {}
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_directory_input_is_made_of_directories_so_files_can_be_made_beside_its_own() {
        let dir = tempfile::tempdir().unwrap();
        let from = dir.path().join("from");
        std::fs::create_dir_all(from.join("sub")).unwrap();
        std::fs::write(from.join("sub/a"), "a").unwrap();
        let to = dir.path().join("to");
        mirror(&from, &to).unwrap();
        assert!(!std::fs::symlink_metadata(&to).unwrap().is_symlink());
        assert!(
            !std::fs::symlink_metadata(to.join("sub"))
                .unwrap()
                .is_symlink()
        );
        assert!(
            std::fs::symlink_metadata(to.join("sub/a"))
                .unwrap()
                .is_symlink()
        );
        assert_eq!(std::fs::read_to_string(to.join("sub/a")).unwrap(), "a");
        std::fs::write(to.join("sub/new"), "n").unwrap();
        assert!(!from.join("sub/new").exists());
    }
}
