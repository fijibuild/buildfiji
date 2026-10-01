//! The configuration a target is built in (buildfiji-136.21), as far as the
//! output tree sees it.
//!
//! Observable on Bazel 9.2.0 (`bazel info output_path`, `bazel aquery`): a
//! default build is configured `k8-fastbuild` on an x86-64 Linux host, and its
//! outputs are under `bazel-out/k8-fastbuild/bin`. `bazel-genfiles` is the
//! same directory as `bazel-bin`.

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

/// The value of a build setting or a command-line option a transition reads
/// or writes.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum SettingValue {
    Bool(bool),
    Int(i64),
    Str(String),
    List(Vec<String>),
}

impl SettingValue {
    /// How Java prints it, which is how Bazel spells it in the hash of an
    /// output directory: `[-O1, -O2]` for a list.
    fn java(&self) -> String {
        match self {
            SettingValue::Bool(b) => b.to_string(),
            SettingValue::Int(i) => i.to_string(),
            SettingValue::Str(s) => s.clone(),
            SettingValue::List(items) => format!("[{}]", items.join(", ")),
        }
    }
}

/// The prefix of a command-line option in a transition's settings.
pub const COMMAND_LINE_OPTION: &str = "//command_line_option:";

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
    /// Build settings and command-line options set away from their defaults,
    /// by `--//pkg:flag=value` or a transition: the label of the setting
    /// (`//pkg:flag`), or `//command_line_option:name`.
    pub settings: BTreeMap<String, SettingValue>,
    /// The settings a Starlark transition changed. They name the output
    /// directory (`k8-fastbuild-ST-<hash>`).
    pub affected: BTreeSet<String>,
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
            settings: BTreeMap::new(),
            affected: BTreeSet::new(),
        }
    }
}

impl Configuration {
    /// The configuration a tool is built in (`cfg = "exec"`): optimised, on the
    /// execution platform, with the target's flags dropped. Bazel names it
    /// `k8-opt-exec`.
    pub fn to_exec(&self) -> Configuration {
        Configuration {
            cpu: self.cpu.clone(),
            compilation_mode: CompilationMode::Opt,
            constraints: self.constraints.clone(),
            exec: true,
            ..Configuration::default()
        }
    }

    /// The directory of `bazel-out` this configuration's outputs are in:
    /// `k8-fastbuild`. A configuration that builds tools says so.
    pub fn mnemonic(&self) -> String {
        let mut name = format!("{}-{}", self.cpu, self.compilation_mode.name());
        if self.exec {
            name.push_str("-exec");
        }
        if let Some(hash) = self.transition_hash() {
            name.push_str("-ST-");
            name.push_str(&hash);
        }
        name
    }

    /// The twelve hex digits a Starlark transition adds to the name of the
    /// output directory: SHA-256 of `setting=value` for each setting it
    /// changed, in name order, each prefixed by its length. `--cpu` and
    /// `--compilation_mode` are in the name already and are left out.
    /// Probed on Bazel 9.2.0.
    fn transition_hash(&self) -> Option<String> {
        let mut digest = Sha256::new();
        let mut any = false;
        for name in &self.affected {
            let Some(value) = self.settings.get(name) else {
                continue;
            };
            let entry = format!("{name}={}", value.java());
            let mut len = entry.len();
            while len >= 0x80 {
                digest.update([(len & 0x7f) as u8 | 0x80]);
                len >>= 7;
            }
            digest.update([len as u8]);
            digest.update(entry.as_bytes());
            any = true;
        }
        any.then(|| hex::encode(digest.finalize())[..12].to_owned())
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
    fn a_tool_is_built_optimised_in_the_exec_configuration() {
        let target = Configuration {
            compilation_mode: CompilationMode::Dbg,
            ..Configuration::default()
        };
        assert_eq!(
            target.to_exec().mnemonic(),
            format!("{}-opt-exec", target.cpu)
        );
        assert_eq!(
            target.to_exec().bin_dir(),
            format!("bazel-out/{}-opt-exec/bin", target.cpu)
        );
    }

    /// Probed with `bazel build` and `bazel aquery` on transitions that set
    /// `//:flag`, `//:flag2` and `--copt`.
    #[test]
    fn a_transition_names_the_output_directory_as_bazel_does() {
        let mut config = Configuration::default();
        config
            .settings
            .insert("//:flag".into(), SettingValue::Str("on".into()));
        config.affected.insert("//:flag".into());
        assert_eq!(config.mnemonic(), "k8-fastbuild-ST-c59cc04586de");
        config
            .settings
            .insert("//:flag2".into(), SettingValue::Str("x".into()));
        config.affected.insert("//:flag2".into());
        assert_eq!(config.mnemonic(), "k8-fastbuild-ST-92296e9f19db");
        let mut copt = Configuration::default();
        copt.settings.insert(
            format!("{COMMAND_LINE_OPTION}copt"),
            SettingValue::List(vec!["-O2".into()]),
        );
        copt.affected.insert(format!("{COMMAND_LINE_OPTION}copt"));
        assert_eq!(copt.mnemonic(), "k8-fastbuild-ST-bf371aea6388");
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
            settings: BTreeMap::new(),
            affected: BTreeSet::new(),
        };
        assert_eq!(config.mnemonic(), "k8-opt-exec");
        assert_eq!(CompilationMode::parse("dbg"), Some(CompilationMode::Dbg));
        assert_eq!(CompilationMode::parse("fast"), None);
    }
}
