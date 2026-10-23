//! `--[no]check_visibility` and `--memory_profile=<path>`: two flags with
//! no bigger home yet — visibility enforcement and memory profiling
//! aren't implemented, just the flags that will configure them once they
//! are — grouped in one module rather than two single-field ones.
//! Extraction follows [`crate::diagnostics_flags::extract`]'s shape.

use std::path::PathBuf;

use crate::flag_registry::FlagRegistry;

/// Flag names this module reads, for `clap_flags::validate`'s
/// unimplemented-flag gate.
pub const IMPLEMENTED: &[&str] = &[
    "check_visibility",
    "memory_profile",
    "enable_bzlmod",
    "incompatible_strict_action_env",
    "slim_profile",
    "experimental_profile_include_target_label",
    "experimental_profile_include_primary_output",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiscFlags {
    /// `--[no]check_visibility`: Bazel defaults to `true` (a visibility
    /// error fails the build); `--nocheck_visibility` demotes it to a
    /// warning.
    pub check_visibility: bool,
    /// `--memory_profile=<path>`: write memory usage data here at phase
    /// ends.
    pub memory_profile: Option<PathBuf>,
    /// `--[no]enable_bzlmod`: on, and bzlmod is the only way to name
    /// dependencies here; accepted so rc files that say so are read.
    pub enable_bzlmod: bool,
    /// `--[no]incompatible_strict_action_env`: actions here always run
    /// with the strict environment (`PATH` fixed), so off is not honoured yet.
    pub strict_action_env: bool,
    /// `--[no]slim_profile` (default on): merge small events of the JSON trace
    /// profile. Read once `--profile` is written, which it is not yet, so a
    /// `--profile` still stops the run (buildfiji-5dwx).
    pub slim_profile: bool,
    /// `--[no]experimental_profile_include_target_label`: the trace profile
    /// names the target in each action event; read with `--profile`
    /// (buildfiji-avk2).
    pub profile_include_target_label: bool,
    /// `--[no]experimental_profile_include_primary_output`: the trace profile
    /// names the action's primary output; read with `--profile`
    /// (buildfiji-96fe).
    pub profile_include_primary_output: bool,
}

impl Default for MiscFlags {
    fn default() -> Self {
        MiscFlags {
            check_visibility: true,
            memory_profile: None,
            enable_bzlmod: true,
            strict_action_env: false,
            slim_profile: true,
            profile_include_target_label: false,
            profile_include_primary_output: false,
        }
    }
}

/// Pull [`MiscFlags`] for `command` out of `args`, returning the flags
/// found and every argument *not* consumed, in original relative order.
pub fn extract(args: &[String], command: &str) -> (MiscFlags, Vec<String>) {
    let registry = FlagRegistry::global();
    let mut flags = MiscFlags::default();
    let mut rest = Vec::with_capacity(args.len());
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        if !arg.starts_with('-') {
            rest.push(arg.clone());
            continue;
        }
        let Ok(m) = registry.resolve(arg, command) else {
            rest.push(arg.clone());
            continue;
        };
        match m.flag.name {
            "check_visibility" => flags.check_visibility = !m.negated,
            "enable_bzlmod" => flags.enable_bzlmod = !m.negated,
            "incompatible_strict_action_env" => flags.strict_action_env = !m.negated,
            "slim_profile" => flags.slim_profile = !m.negated,
            "experimental_profile_include_target_label" => {
                flags.profile_include_target_label = !m.negated
            }
            "experimental_profile_include_primary_output" => {
                flags.profile_include_primary_output = !m.negated
            }
            "memory_profile" => {
                match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                    Some(value) => flags.memory_profile = Some(PathBuf::from(value)),
                    None => rest.push(arg.clone()),
                }
            }
            _ => rest.push(arg.clone()),
        }
    }

    (flags, rest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn defaults_are_bazels_defaults() {
        let (flags, rest) = extract(&args(&[]), "build");
        assert_eq!(flags, MiscFlags::default());
        assert!(flags.check_visibility);
        assert!(rest.is_empty());
    }

    #[test]
    fn noslim_profile_turns_it_off() {
        let (flags, rest) = extract(&args(&["--noslim_profile"]), "build");
        assert!(!flags.slim_profile);
        assert!(rest.is_empty());
        let (flags, _) = extract(&args(&["--noslim_profile", "--slim_profile"]), "build");
        assert!(flags.slim_profile);
    }

    #[test]
    fn the_profile_target_label_flag_is_taken() {
        let (flags, rest) = extract(
            &args(&["--experimental_profile_include_target_label"]),
            "build",
        );
        assert!(flags.profile_include_target_label);
        assert!(rest.is_empty());
    }

    #[test]
    fn the_profile_primary_output_flag_is_taken() {
        let (flags, rest) = extract(
            &args(&["--experimental_profile_include_primary_output"]),
            "build",
        );
        assert!(flags.profile_include_primary_output);
        assert!(rest.is_empty());
    }

    #[test]
    fn nocheck_visibility_turns_it_off() {
        let (flags, _) = extract(&args(&["--nocheck_visibility"]), "build");
        assert!(!flags.check_visibility);
    }

    #[test]
    fn memory_profile_attached_value() {
        let (flags, rest) = extract(&args(&["--memory_profile=/tmp/mem.json"]), "build");
        assert_eq!(flags.memory_profile, Some(PathBuf::from("/tmp/mem.json")));
        assert!(rest.is_empty());
    }

    #[test]
    fn memory_profile_space_separated_value() {
        let (flags, rest) = extract(&args(&["--memory_profile", "/tmp/mem.json"]), "build");
        assert_eq!(flags.memory_profile, Some(PathBuf::from("/tmp/mem.json")));
        assert!(rest.is_empty());
    }

    #[test]
    fn unrelated_tokens_pass_through() {
        let (flags, rest) = extract(&args(&["//foo:bar", "--keep_going"]), "build");
        assert_eq!(flags, MiscFlags::default());
        assert_eq!(rest, args(&["//foo:bar", "--keep_going"]));
    }
}
