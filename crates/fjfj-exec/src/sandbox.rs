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
            match std::os::unix::fs::symlink(layout.resolve(input), &at) {
                Err(e) if e.kind() != io::ErrorKind::AlreadyExists => return Err(e),
                _ => {}
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
