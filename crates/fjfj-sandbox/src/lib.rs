//! Sandboxing strategies for local execution.
//!
//! Bazel offers `local`, `sandboxed` (linux-sandbox / darwin-sandbox /
//! processwrapper-sandbox), `worker`, and `docker`. fjfj mirrors that with a
//! `Sandbox` trait so strategies are pluggable and selectable via the Bazel
//! `--spawn_strategy` / `--strategy=Mnemonic=...` flags.

use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::{isolate, isolated, namespaces_available, run_helper_if_asked, use_helper};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// No isolation; run in a scratch execroot.
    Local,
    /// Linux user namespaces + mount namespaces (like `linux-sandbox`).
    LinuxNamespaces,
    /// macOS `sandbox-exec` profile (like `darwin-sandbox`).
    DarwinSeatbelt,
    /// Overlay/hermetic container via OCI runtime.
    Oci,
}

pub trait Sandbox {
    fn strategy(&self) -> Strategy;
    /// Materialise an execroot containing exactly `inputs` at `root`.
    fn prepare(&self, root: &Path) -> anyhow::Result<()>;
}

/// Pick the best available strategy for the host OS.
pub fn default_strategy() -> Strategy {
    if cfg!(target_os = "linux") {
        Strategy::LinuxNamespaces
    } else if cfg!(target_os = "macos") {
        Strategy::DarwinSeatbelt
    } else {
        Strategy::Local
    }
}

/// What a namespace-isolated command may touch (Bazel's `linux-sandbox`).
#[derive(Debug, Clone, Default)]
pub struct Isolation {
    /// Directories that stay writable; the rest of the file system is
    /// read-only.
    pub writable: Vec<PathBuf>,
    /// Give the command a network namespace with nothing in it
    /// (`block-network`).
    pub block_network: bool,
}

/// Namespaces are Linux's; elsewhere a command runs as it is.
#[cfg(not(target_os = "linux"))]
mod other {
    use super::*;
    use std::io;

    pub fn use_helper() {}

    pub fn run_helper_if_asked() {}

    pub fn isolated(
        program: &str,
        args: &[String],
        cwd: &Path,
        iso: &Isolation,
    ) -> io::Result<std::process::Command> {
        let mut command = std::process::Command::new(program);
        command.args(args).current_dir(cwd);
        isolate(&mut command, iso)?;
        Ok(command)
    }

    pub fn isolate(_command: &mut std::process::Command, _iso: &Isolation) -> io::Result<()> {
        Ok(())
    }

    pub fn namespaces_available() -> bool {
        false
    }
}
#[cfg(not(target_os = "linux"))]
pub use other::{isolate, isolated, namespaces_available, run_helper_if_asked, use_helper};
