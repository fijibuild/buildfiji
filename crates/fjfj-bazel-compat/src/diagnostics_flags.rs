//! Bazel's cross-command diagnostics flags: `--keep_going`/`-k`,
//! `--verbose_failures`, `--subcommands`/`-s`, `--explain=<path>`, and the
//! now-no-op `--verbose_explanations`. Pulled out of a raw argv slice with
//! [`FlagRegistry`] rather than given individual clap fields — target
//! patterns are captured as raw strings too (see `Cli`/`TargetArgs`);
//! fjfj doesn't attempt a typed clap field for each of Bazel's ~1000
//! flags, only for the handful whose value a command actually needs.

use std::path::PathBuf;

use crate::flag_registry::FlagRegistry;

/// Flag names this module reads, for `clap_flags::validate`'s
/// unimplemented-flag gate.
pub const IMPLEMENTED: &[&str] = &[
    "keep_going",
    "verbose_failures",
    "subcommands",
    "explain",
    "verbose_explanations",
];

/// Values for the flags in this module, defaulting to Bazel's own (all
/// off, no `--explain` file).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiagnosticsFlags {
    /// `--keep_going`/`-k`: continue building/testing unaffected targets
    /// after a failure, instead of stopping at the first one.
    pub keep_going: bool,
    /// `--verbose_failures`: print a failed action's full command line.
    pub verbose_failures: bool,
    /// `--subcommands`/`-s`: print each action's command line as it runs.
    pub subcommands: bool,
    /// `--explain=<path>` (or `--explain <path>`): write a step-by-step
    /// rebuild explanation here.
    pub explain: Option<PathBuf>,
}

/// Pull [`DiagnosticsFlags`] for `command` (e.g. `"build"`) out of `args`,
/// returning the flags found and every argument *not* consumed — other
/// flags and target patterns — in their original relative order.
///
/// A token this module doesn't recognise, including a real Bazel flag
/// fjfj hasn't wired up elsewhere, is left in the returned `Vec`
/// untouched: classifying every flag isn't this function's job, only
/// peeling off these five. `--verbose_explanations`/
/// `--noverbose_explanations` are recognised and dropped silently — a
/// documented no-op in Bazel itself.
pub fn extract(args: &[String], command: &str) -> (DiagnosticsFlags, Vec<String>) {
    let registry = FlagRegistry::global();
    let mut flags = DiagnosticsFlags::default();
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
            "keep_going" => flags.keep_going = !m.negated,
            "verbose_failures" => flags.verbose_failures = !m.negated,
            "subcommands" => flags.subcommands = !m.negated,
            "verbose_explanations" => {} // documented no-op in Bazel; drop
            "explain" => match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                Some(value) => flags.explain = Some(PathBuf::from(value)),
                // Missing required value: leave the bare flag for
                // whatever validates flags next to report the real error.
                None => rest.push(arg.clone()),
            },
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
    fn extracts_boolean_flags() {
        let (flags, rest) = extract(&args(&["--keep_going", "--verbose_failures"]), "build");
        assert!(flags.keep_going);
        assert!(flags.verbose_failures);
        assert!(rest.is_empty());
    }

    #[test]
    fn extracts_abbreviations() {
        let (flags, rest) = extract(&args(&["-k", "-s"]), "build");
        assert!(flags.keep_going);
        assert!(flags.subcommands);
        assert!(rest.is_empty());
    }

    #[test]
    fn negation_turns_a_flag_off() {
        let (flags, _) = extract(&args(&["--keep_going", "--nokeep_going"]), "build");
        assert!(!flags.keep_going);
    }

    #[test]
    fn explain_accepts_the_attached_value_form() {
        let (flags, rest) = extract(&args(&["--explain=/tmp/why.log"]), "build");
        assert_eq!(flags.explain, Some(PathBuf::from("/tmp/why.log")));
        assert!(rest.is_empty());
    }

    #[test]
    fn explain_accepts_the_space_separated_value_form() {
        let (flags, rest) = extract(&args(&["--explain", "/tmp/why.log"]), "build");
        assert_eq!(flags.explain, Some(PathBuf::from("/tmp/why.log")));
        assert!(rest.is_empty());
    }

    #[test]
    fn explain_missing_a_value_is_left_for_the_caller() {
        let (flags, rest) = extract(&args(&["--explain"]), "build");
        assert_eq!(flags.explain, None);
        assert_eq!(rest, vec!["--explain".to_string()]);
    }

    #[test]
    fn verbose_explanations_is_dropped_silently() {
        let (_, rest) = extract(&args(&["--verbose_explanations"]), "build");
        assert!(rest.is_empty());
    }

    #[test]
    fn unrelated_flags_and_target_patterns_pass_through_in_order() {
        let (flags, rest) = extract(
            &args(&["-k", "//foo:bar", "--jobs=4", "//baz/...", "--nostrip"]),
            "build",
        );
        assert!(flags.keep_going);
        assert_eq!(
            rest,
            vec!["//foo:bar", "--jobs=4", "//baz/...", "--nostrip"]
        );
    }

    #[test]
    fn unknown_token_passes_through() {
        let (_, rest) = extract(&args(&["--not-a-real-flag"]), "build");
        assert_eq!(rest, vec!["--not-a-real-flag".to_string()]);
    }
}
