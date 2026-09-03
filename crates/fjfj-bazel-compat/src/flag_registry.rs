//! A typed, indexed registry over the generated Bazel flag table
//! (`bazel_flags`), and the unknown-flag policy that lets fjfj accept a
//! `.bazelrc` written for Bazel flags it doesn't implement yet (see
//! `docs/design/cli-compat.md`'s "Flag policy").

use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;

use crate::bazel_flags::{FLAGS, FlagInfo};

/// What to do with a flag token that names a flag [`FlagRegistry`] doesn't
/// recognise for the command it's used with — either genuinely unknown, or
/// a real Bazel flag fjfj hasn't wired to a command's behavior yet.
/// Default is `Warn`, so a `.bazelrc` shared with Bazel keeps working
/// during the migration; `--fjfj_strict_flags` selects `Strict`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnknownFlagPolicy {
    #[default]
    Warn,
    Strict,
}

/// One resolved flag token: which [`FlagInfo`] it names, whether it was
/// written in `--no<name>` negated form, and any attached value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlagMatch<'a> {
    pub flag: &'static FlagInfo,
    pub negated: bool,
    pub value: Option<&'a str>,
}

/// A flag token [`FlagRegistry::resolve`] couldn't match to `command`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownFlagError {
    pub token: String,
    pub command: String,
    /// Set when the name matches a real Bazel flag, just not for `command`.
    pub known_for: Option<&'static [&'static str]>,
}

impl fmt::Display for UnknownFlagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.known_for {
            Some(commands) => write!(
                f,
                "unrecognized option '{}' for command '{}' (known for: {})",
                self.token,
                self.command,
                commands.join(", ")
            ),
            None => write!(f, "unrecognized option '{}'", self.token),
        }
    }
}

impl std::error::Error for UnknownFlagError {}

/// Apply `policy` to a flag [`FlagRegistry::resolve`] couldn't match: `Warn`
/// turns it into a one-line warning string to print and keep going; `Strict`
/// turns it into the error to propagate. Mirrors Bazel's own two-tier
/// behavior around flags it doesn't recognise.
pub fn apply_policy(
    err: UnknownFlagError,
    policy: UnknownFlagPolicy,
) -> Result<String, UnknownFlagError> {
    match policy {
        UnknownFlagPolicy::Warn => Ok(format!("WARNING: {err}")),
        UnknownFlagPolicy::Strict => Err(err),
    }
}

/// Fast lookup over [`FLAGS`] by every name a flag can be written with: its
/// own name, a deprecated `old_name`, `--no<name>` (if `has_negative_flag`),
/// and its single-character abbreviation.
pub struct FlagRegistry {
    by_name: HashMap<&'static str, &'static FlagInfo>,
    negatable: HashMap<&'static str, &'static FlagInfo>,
    by_abbreviation: HashMap<&'static str, &'static FlagInfo>,
}

impl FlagRegistry {
    /// The registry over the checked-in [`FLAGS`] table, built once.
    pub fn global() -> &'static FlagRegistry {
        static REGISTRY: OnceLock<FlagRegistry> = OnceLock::new();
        REGISTRY.get_or_init(FlagRegistry::build)
    }

    fn build() -> Self {
        let mut by_name = HashMap::with_capacity(FLAGS.len() * 2);
        let mut negatable = HashMap::new();
        let mut by_abbreviation = HashMap::new();
        for flag in FLAGS {
            by_name.insert(flag.name, flag);
            // A flag's own name always wins over an old alias pointing at
            // a different flag; FLAGS has no duplicate `name`s (tested in
            // `bazel_flags::tests`), so this only matters for old_name
            // collisions, which we resolve first-registered-wins.
            if let Some(old) = flag.old_name {
                by_name.entry(old).or_insert(flag);
            }
            if flag.has_negative_flag {
                negatable.insert(flag.name, flag);
            }
            if let Some(abbr) = flag.abbreviation {
                by_abbreviation.insert(abbr, flag);
            }
        }
        FlagRegistry {
            by_name,
            negatable,
            by_abbreviation,
        }
    }

    /// Resolve one raw flag token — `--name`, `--name=value`, `--noname`,
    /// `-x` or `-xvalue` (Bazel's single-dash abbreviation form) — against
    /// `command` (use `"startup"` for a startup option). `token` must
    /// start with `-`; a bare positional argument is never a flag and
    /// isn't this function's concern.
    pub fn resolve<'a>(
        &self,
        token: &'a str,
        command: &str,
    ) -> Result<FlagMatch<'a>, UnknownFlagError> {
        let unknown = || UnknownFlagError {
            token: token.to_string(),
            command: command.to_string(),
            known_for: None,
        };

        let (flag, negated, value) = if let Some(rest) = token.strip_prefix("--") {
            let (name, value) = split_value(rest);
            match self.by_name.get(name) {
                Some(flag) => (*flag, false, value),
                None => match name.strip_prefix("no").and_then(|n| self.negatable.get(n)) {
                    Some(flag) if value.is_none() => (*flag, true, None),
                    _ => return Err(unknown()),
                },
            }
        } else if let Some(rest) = token.strip_prefix('-') {
            if rest.is_empty() {
                return Err(unknown());
            }
            let abbr = &rest[..1];
            let value = if rest.len() > 1 {
                Some(&rest[1..])
            } else {
                None
            };
            match self.by_abbreviation.get(abbr) {
                Some(flag) => (*flag, false, value),
                None => return Err(unknown()),
            }
        } else {
            return Err(unknown());
        };

        if flag.commands.contains(&command) {
            Ok(FlagMatch {
                flag,
                negated,
                value,
            })
        } else {
            Err(UnknownFlagError {
                token: token.to_string(),
                command: command.to_string(),
                known_for: Some(flag.commands),
            })
        }
    }
}

/// Split `--name=value`'s tail into `("name", Some("value"))`, or
/// `--name` into `("name", None)`.
fn split_value(rest: &str) -> (&str, Option<&str>) {
    match rest.split_once('=') {
        Some((name, value)) => (name, Some(value)),
        None => (rest, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> &'static FlagRegistry {
        FlagRegistry::global()
    }

    #[test]
    fn resolves_long_flag_with_value() {
        let m = registry().resolve("--copt=-O2", "build").unwrap();
        assert_eq!(m.flag.name, "copt");
        assert!(!m.negated);
        assert_eq!(m.value, Some("-O2"));
    }

    #[test]
    fn resolves_boolean_flag_without_value() {
        let m = registry().resolve("--keep_going", "build").unwrap();
        assert_eq!(m.flag.name, "keep_going");
        assert!(!m.negated);
        assert_eq!(m.value, None);
    }

    #[test]
    fn resolves_negated_flag() {
        let m = registry().resolve("--nokeep_going", "build").unwrap();
        assert_eq!(m.flag.name, "keep_going");
        assert!(m.negated);
    }

    #[test]
    fn resolves_abbreviation_with_attached_value() {
        let m = registry().resolve("-j4", "build").unwrap();
        assert_eq!(m.flag.name, "jobs");
        assert_eq!(m.value, Some("4"));
    }

    #[test]
    fn resolves_abbreviation_alone() {
        let m = registry().resolve("-j", "build").unwrap();
        assert_eq!(m.flag.name, "jobs");
        assert_eq!(m.value, None);
    }

    #[test]
    fn unknown_name_is_an_error() {
        let err = registry()
            .resolve("--not-a-real-flag", "build")
            .unwrap_err();
        assert_eq!(err.known_for, None);
    }

    #[test]
    fn known_flag_for_wrong_command_reports_where_it_is_known() {
        // `jobs` isn't a startup option.
        let err = registry().resolve("--jobs=4", "startup").unwrap_err();
        assert!(err.known_for.unwrap().contains(&"build"));
    }

    #[test]
    fn negating_a_non_negatable_flag_is_unknown() {
        // `copt` has no `--nocopt` form.
        assert!(registry().resolve("--nocopt", "build").is_err());
    }

    #[test]
    fn warn_policy_produces_a_message_and_keeps_going() {
        let err = registry()
            .resolve("--not-a-real-flag", "build")
            .unwrap_err();
        let warned = apply_policy(err, UnknownFlagPolicy::Warn).unwrap();
        assert!(warned.contains("not-a-real-flag"));
    }

    #[test]
    fn strict_policy_propagates_the_error() {
        let err = registry()
            .resolve("--not-a-real-flag", "build")
            .unwrap_err();
        assert!(apply_policy(err, UnknownFlagPolicy::Strict).is_err());
    }

    #[test]
    fn startup_flag_resolves_under_the_startup_pseudo_command() {
        let m = registry()
            .resolve("--output_base=/tmp/ob", "startup")
            .unwrap();
        assert_eq!(m.flag.name, "output_base");
    }
}
