//! The configuration a target is built in (buildfiji-136.21), as far as the
//! output tree sees it.
//!
//! Observable on Bazel 9.2.0 (`bazel info output_path`, `bazel aquery`): a
//! default build is configured `k8-fastbuild` on an x86-64 Linux host, and its
//! outputs are under `bazel-out/k8-fastbuild/bin`. `bazel-genfiles` is the
//! same directory as `bazel-bin`.

use std::collections::BTreeMap;

/// `-c fastbuild|dbg|opt`.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum CompilationMode {
    #[default]
    Fastbuild,
    Dbg,
    Opt,
}

impl CompilationMode {
    pub fn name(self) -> &'static str {
        match self {
            CompilationMode::Fastbuild => "fastbuild",
            CompilationMode::Dbg => "dbg",
            CompilationMode::Opt => "opt",
        }
    }

    pub fn parse(name: &str) -> Option<CompilationMode> {
        Some(match name {
            "fastbuild" => CompilationMode::Fastbuild,
            "dbg" => CompilationMode::Dbg,
            "opt" => CompilationMode::Opt,
            _ => return None,
        })
    }
}

/// The options that decide where outputs go and what actions say.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct Configuration {
    /// `--cpu`, in the spelling the output directory uses: `k8`, `aarch64`,
    /// `darwin_arm64`, `darwin_x86_64`.
    pub cpu: String,
    pub compilation_mode: CompilationMode,
    /// `--define`s, by name.
    pub defines: BTreeMap<String, String>,
    /// The `constraint_value`s the target platform has, as canonical labels:
    /// what `ctx.target_platform_has_constraint` and `select()` ask.
    pub constraints: std::collections::BTreeSet<crate::Label>,
    /// Other flags, by name, for `config_setting(values = ...)`: `copt`,
    /// `linkopt`, ...
    pub options: BTreeMap<String, String>,
    /// `--test_env`: set for every test, a name inherited from the client
    /// already given its value.
    pub test_env: BTreeMap<String, String>,
    /// `--test_arg`: appended to the arguments of every test.
    pub test_args: Vec<String>,
    /// Built to run on the execution platform: a tool, not a target.
    pub exec: bool,
}

/// The `@platforms` constraint values of the machine fjfj runs on, as
/// `(constraint setting, value)` names: `("os", "linux")`, `("cpu", "x86_64")`.
pub fn host_constraints() -> [(&'static str, &'static str); 2] {
    let os = match std::env::consts::OS {
        "macos" => "osx",
        "windows" => "windows",
        _ => "linux",
    };
    let cpu = match std::env::consts::ARCH {
        "aarch64" => "aarch64",
        "x86" => "x86_32",
        _ => "x86_64",
    };
    [("os", os), ("cpu", cpu)]
}

/// The `--cpu` of the machine fjfj runs on, as Bazel names it.
pub fn host_cpu() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "k8",
        ("linux", "aarch64") => "aarch64",
        ("macos", "aarch64") => "darwin_arm64",
        ("macos", "x86_64") => "darwin_x86_64",
        ("windows", _) => "x64_windows",
        (_, "x86_64") => "k8",
        (_, arch) => arch,
    }
}

impl Default for Configuration {
    fn default() -> Configuration {
        Configuration {
            cpu: host_cpu().to_owned(),
            compilation_mode: CompilationMode::default(),
            defines: BTreeMap::new(),
            constraints: std::collections::BTreeSet::new(),
            options: BTreeMap::new(),
            test_env: BTreeMap::new(),
            test_args: Vec::new(),
            exec: false,
        }
    }
}

impl Configuration {
    /// The directory of `bazel-out` this configuration's outputs are in:
    /// `k8-fastbuild`. A configuration that builds tools says so.
    pub fn mnemonic(&self) -> String {
        let mut name = format!("{}-{}", self.cpu, self.compilation_mode.name());
        if self.exec {
            name.push_str("-exec");
        }
        name
    }

    /// `bazel-out/k8-fastbuild/bin`, where the rules' outputs go.
    pub fn bin_dir(&self) -> String {
        format!("bazel-out/{}/bin", self.mnemonic())
    }

    /// Where test logs go.
    pub fn testlogs_dir(&self) -> String {
        format!("bazel-out/{}/testlogs", self.mnemonic())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_is_fastbuild_for_the_host() {
        let config = Configuration::default();
        assert_eq!(config.mnemonic(), format!("{}-fastbuild", host_cpu()));
        assert_eq!(
            config.bin_dir(),
            format!("bazel-out/{}-fastbuild/bin", host_cpu())
        );
    }

    #[test]
    fn the_mode_and_exec_show_in_the_name() {
        let config = Configuration {
            cpu: "k8".into(),
            compilation_mode: CompilationMode::Opt,
            defines: BTreeMap::new(),
            constraints: std::collections::BTreeSet::new(),
            options: BTreeMap::new(),
            test_env: BTreeMap::new(),
            test_args: Vec::new(),
            exec: true,
        };
        assert_eq!(config.mnemonic(), "k8-opt-exec");
        assert_eq!(CompilationMode::parse("dbg"), Some(CompilationMode::Dbg));
        assert_eq!(CompilationMode::parse("fast"), None);
    }
}
