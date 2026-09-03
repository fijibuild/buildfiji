//! Sandboxing strategies for local execution.
//!
//! Bazel offers `local`, `sandboxed` (linux-sandbox / darwin-sandbox /
//! processwrapper-sandbox), `worker`, and `docker`. fjfj mirrors that with a
//! `Sandbox` trait so strategies are pluggable and selectable via the Bazel
//! `--spawn_strategy` / `--strategy=Mnemonic=...` flags.

use std::path::Path;

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
