//! Drives a validating `clap::Command` straight off the generated
//! `bazel_flags::FLAGS` table, instead of hand-maintaining a parallel list
//! of "known" flag names. This replaces the earlier
//! `flag_registry::UnknownFlagPolicy` pre-partition (`Warn`-by-default,
//! `Strict` opt-in): a flag Bazel accepts but fjfj hasn't wired to actual
//! behavior is easy to silently mis-build with (its value just does
//! nothing), which is worse than refusing to run — see
//! `docs/design/cli-compat.md`'s "Flag surface (decision 2026-09-03)".
//! So [`validate`] now fails loudly on *any* flag a command's typed
//! extractors don't claim, whether or not Bazel itself recognises it,
//! rather than warning and guessing.
//!
//! [`command_for`] doesn't attempt real per-flag typed parsing (no
//! `type_converter` -> `ValueParser` mapping) — that's still each `*_flags`
//! module's job, run separately over the same raw tokens once `validate`
//! has passed. This module is a gate, not a decoder: value-taking flags
//! all use a permissive string parser here, and negatable ones register
//! both spellings, purely so `clap` can tell "real Bazel flag, presented
//! correctly" apart from "not a flag at all" or "malformed value" and let
//! [`crate::TargetPattern`] parsing see only genuine positionals.

use std::collections::{HashMap, HashSet};

use clap::{Arg, ArgAction, Command, parser::ValueSource};

use crate::bazel_flags::{FLAGS, FlagInfo};

/// One flag a command's `*_flags::extract` (or equivalent) actually reads,
/// so [`validate`] knows not to reject it. Each module that reads flags
/// out of a raw argv slice declares its own `IMPLEMENTED` constant; the
/// command wiring in `fjfj-cli` unions them for the flags surface it
/// passes to [`validate`].
pub type ImplementedFlags = &'static [&'static str];

/// A flag `validate` rejected: either not a real Bazel flag/not accepted
/// by `command`, or a real one no typed extractor for `command` claims
/// yet.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum FlagSurfaceError {
    /// `clap` itself couldn't parse the token against the flags
    /// `command_for` registered — not a real Bazel flag for this
    /// command, a malformed value, or similar.
    #[error("{0}")]
    Rejected(String),
    /// A real Bazel flag for `command`, but fjfj has no behavior wired
    /// for it yet.
    #[error(
        "flag '--{flag}' is recognized by Bazel for '{command}' but not implemented by fjfj yet; see `bd ready`"
    )]
    NotImplemented { flag: String, command: String },
}

/// Build a `clap::Command` carrying one `Arg` per [`FLAGS`] entry accepted
/// by `bazel_command`, plus a trailing catch-all positional for target
/// patterns (including a negative one like `-//pkg:excluded`, which looks
/// like a flag but isn't). Boolean flags (`requires_value == false`)
/// become plain `SetTrue` switches, with a second `--no<name>` `Arg` when
/// `has_negative_flag` is set; value-taking flags accept one attached or
/// space-separated string each (`Append` when `allows_multiple`) — no
/// attempt at real per-`type_converter` parsing, see the module doc.
pub fn command_for(bazel_command: &'static str) -> Command {
    build(bazel_command).0
}

/// [`command_for`]'s actual builder, plus the `--no<name>` id (when one
/// was registered) each flag's canonical `name` maps to — [`validate`]
/// needs that mapping to check a negated flag's presence without
/// querying an id `clap` might not have registered (which panics, unlike
/// `FlagRegistry`'s plain `HashMap` lookups).
fn build(bazel_command: &'static str) -> (Command, HashMap<&'static str, &'static str>) {
    let mut cmd = Command::new(bazel_command)
        .no_binary_name(true)
        .disable_help_flag(true)
        .disable_version_flag(true);
    let mut long_names: HashSet<String> = HashSet::new();
    let mut shorts: HashSet<char> = HashSet::new();
    let mut negated_id_of: HashMap<&'static str, &'static str> = HashMap::new();

    for flag in FLAGS.iter().filter(|f| f.commands.contains(&bazel_command)) {
        if !long_names.insert(flag.name.to_string()) {
            continue; // FLAGS has no duplicate names; guards old_name collisions below.
        }
        cmd = cmd.arg(base_arg(flag));
        if let Some(old) = flag.old_name
            && long_names.insert(old.to_string())
        {
            cmd = cmd.mut_arg(flag.name, |a| a.alias(old));
        }
        if let Some(abbr) = flag.abbreviation
            && abbr.len() == 1
            && shorts.insert(abbr.chars().next().unwrap())
        {
            cmd = cmd.mut_arg(flag.name, |a| a.short(abbr.chars().next().unwrap()));
        }
        if flag.has_negative_flag {
            let negated = format!("no{}", flag.name);
            // Bazel's own name/old_name/abbreviation collisions are rare
            // enough (see `FlagRegistry::build`) that a `--no<name>`
            // collision with another flag's own long name has never
            // shown up in the generated table; skip it rather than panic
            // if one ever does — `negated_id_of` simply has no entry for
            // `flag.name` then, so `validate` won't look for it.
            if long_names.insert(negated.clone()) {
                // clap's builder `Str`/`Id` types are `&'static str`-only
                // in this version (no owned variant), unlike
                // `FlagRegistry`'s plain `HashMap<&str, _>`; leaking is
                // cheap here since there's at most one per negatable flag,
                // built at most once per `bazel_command` per process.
                let negated: &'static str = Box::leak(negated.into_boxed_str());
                cmd = cmd.arg(
                    Arg::new(negated)
                        .long(negated)
                        .action(ArgAction::SetTrue)
                        .hide(true),
                );
                negated_id_of.insert(flag.name, negated);
            }
        }
    }

    // No `allow_hyphen_values` here, deliberately: a hyphen-prefixed
    // token that matches no registered flag above must still fail
    // (that's the whole point of this gate), not fall back to a silently
    // accepted positional. `validate` pulls the one legitimate
    // hyphen-prefixed positional shape — a negative target pattern like
    // `-//pkg:excluded` or `-@repo//pkg:excluded` — out before parsing.
    let cmd = cmd.arg(Arg::new("patterns").num_args(0..).trailing_var_arg(true));
    (cmd, negated_id_of)
}

/// Whether `token` has the one shape [`TargetPattern::from_str`] accepts
/// starting with `-` that isn't a flag: a negative target pattern
/// (`-@repo//pkg:target` or `-//pkg:target`).
fn looks_like_negative_pattern(token: &str) -> bool {
    token.starts_with("-@") || token.starts_with("-//")
}

fn base_arg(flag: &'static FlagInfo) -> Arg {
    let arg = Arg::new(flag.name).long(flag.name).help(flag.documentation);
    if flag.requires_value {
        let arg = arg.allow_hyphen_values(true).num_args(1);
        if flag.allows_multiple {
            arg.action(ArgAction::Append)
        } else {
            arg.action(ArgAction::Set)
        }
    } else {
        arg.action(ArgAction::SetTrue)
    }
}

/// Run `args` (a command's raw argv slice, after `flag_alias::apply`)
/// through [`command_for`]`(bazel_command)`, then reject any flag that
/// parsed but isn't in `implemented`. Returns `Ok(())` when every flag
/// token is a real, implemented flag and only bare positionals remain for
/// `TargetPattern` parsing.
pub fn validate(
    args: &[String],
    bazel_command: &'static str,
    implemented: &[&'static str],
) -> Result<(), FlagSurfaceError> {
    let (cmd, negated_id_of) = build(bazel_command);
    // Negative target patterns are the one legitimate hyphen-prefixed
    // positional; pull them out before `clap` sees them (order doesn't
    // matter — this is a pass/fail gate, not the positional extraction
    // itself, which `TargetPattern::from_str` still does on the original
    // `args` afterward) so a genuinely unrecognized `--flag` still hits
    // clap's own strict, no-`allow_hyphen_values` rejection.
    let flag_tokens: Vec<&String> = args
        .iter()
        .filter(|t| !looks_like_negative_pattern(t))
        .collect();
    let matches = cmd
        .try_get_matches_from(flag_tokens)
        .map_err(|e| FlagSurfaceError::Rejected(e.render().to_string().trim_end().to_string()))?;

    for flag in FLAGS.iter().filter(|f| f.commands.contains(&bazel_command)) {
        let explicit = matches.value_source(flag.name) == Some(ValueSource::CommandLine)
            || negated_id_of.get(flag.name).is_some_and(|negated| {
                matches.value_source(negated) == Some(ValueSource::CommandLine)
            });
        if explicit && !implemented.contains(&flag.name) {
            return Err(FlagSurfaceError::NotImplemented {
                flag: flag.name.to_string(),
                command: bazel_command.to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    const BUILD_IMPLEMENTED: &[&str] = &["keep_going", "jobs"];

    #[test]
    fn implemented_flag_and_pattern_both_pass() {
        validate(
            &args(&["--keep_going", "//foo:bar"]),
            "build",
            BUILD_IMPLEMENTED,
        )
        .unwrap();
    }

    #[test]
    fn negative_target_pattern_is_not_mistaken_for_a_flag() {
        validate(&args(&["-//pkg:excluded"]), "build", BUILD_IMPLEMENTED).unwrap();
    }

    #[test]
    fn genuinely_unknown_flag_is_rejected() {
        let err = validate(&args(&["--not-a-real-flag"]), "build", BUILD_IMPLEMENTED).unwrap_err();
        assert!(matches!(err, FlagSurfaceError::Rejected(_)));
    }

    #[test]
    fn known_but_unimplemented_flag_is_rejected() {
        // `copt` is a real Bazel build flag but isn't in BUILD_IMPLEMENTED.
        let err = validate(&args(&["--copt=-O2"]), "build", BUILD_IMPLEMENTED).unwrap_err();
        assert_eq!(
            err,
            FlagSurfaceError::NotImplemented {
                flag: "copt".to_string(),
                command: "build".to_string(),
            }
        );
    }

    #[test]
    fn negation_of_a_known_but_unimplemented_flag_is_also_rejected() {
        let err = validate(&args(&["--nokeep_going"]), "build", &[]).unwrap_err();
        assert_eq!(
            err,
            FlagSurfaceError::NotImplemented {
                flag: "keep_going".to_string(),
                command: "build".to_string(),
            }
        );
    }

    #[test]
    fn abbreviation_resolves_to_the_same_flag_name() {
        // `-k` is `keep_going`'s abbreviation; validate under its long
        // name only, so this exercises that clap's `.short()` wiring
        // actually reaches the same Arg id `command_for` registered.
        validate(&args(&["-k"]), "build", BUILD_IMPLEMENTED).unwrap();
    }
}
