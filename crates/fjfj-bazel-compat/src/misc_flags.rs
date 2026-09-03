//! `--[no]check_visibility` and `--memory_profile=<path>`: two flags with
//! no bigger home yet — visibility enforcement and memory profiling
//! aren't implemented, just the flags that will configure them once they
//! are — grouped in one module rather than two single-field ones.
//! Extraction follows [`crate::diagnostics_flags::extract`]'s shape.

use std::path::PathBuf;

use crate::flag_registry::FlagRegistry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiscFlags {
    /// `--[no]check_visibility`: Bazel defaults to `true` (a visibility
    /// error fails the build); `--nocheck_visibility` demotes it to a
    /// warning.
    pub check_visibility: bool,
    /// `--memory_profile=<path>`: write memory usage data here at phase
    /// ends.
    pub memory_profile: Option<PathBuf>,
}

impl Default for MiscFlags {
    fn default() -> Self {
        MiscFlags {
            check_visibility: true,
            memory_profile: None,
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
