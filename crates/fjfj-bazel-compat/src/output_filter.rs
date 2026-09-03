//! `--output_filter=<regex>` / `--auto_output_filter`: which rule's build
//! warnings and action output Bazel actually prints. A large build can
//! produce warnings from hundreds of targets; without this, output from
//! everything except what you actually asked for scrolls past unread.
//!
//! Two layers, matching this crate's usual extraction/decision split:
//! [`extract`] pulls the raw flag values out of argv; [`OutputFilter::compile`]
//! turns them into something that can actually decide, one rule at a
//! time — an explicit `--output_filter` regex takes precedence over
//! `--auto_output_filter`, which computes a filter from the packages
//! named on the command line instead.

use std::collections::BTreeSet;

use regex::Regex;

use crate::flag_registry::FlagRegistry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutoOutputFilter {
    /// Show everything. Bazel's own default.
    #[default]
    None,
    /// Show nothing.
    All,
    /// Show only rules in a package named on the command line.
    Packages,
    /// Like `Packages`, but subpackages too.
    Subpackages,
}

/// Flag names this module reads, for `clap_flags::validate`'s
/// unimplemented-flag gate.
pub const IMPLEMENTED: &[&str] = &["output_filter", "auto_output_filter"];

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OutputFilterFlags {
    /// `--output_filter=<regex>`, kept as source text: compiling it is
    /// [`OutputFilter::compile`]'s job, so a malformed regex is reported
    /// there rather than silently swallowed during extraction.
    pub output_filter: Option<String>,
    /// `--auto_output_filter`, consulted only when `output_filter` above
    /// is `None`, same as Bazel.
    pub auto_output_filter: AutoOutputFilter,
}

/// Pull [`OutputFilterFlags`] for `command` out of `args`, returning the
/// flags found and every argument *not* consumed, in original relative
/// order.
pub fn extract(args: &[String], command: &str) -> (OutputFilterFlags, Vec<String>) {
    let registry = FlagRegistry::global();
    let mut flags = OutputFilterFlags::default();
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
            "output_filter" => match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                Some(value) => flags.output_filter = Some(value),
                None => rest.push(arg.clone()),
            },
            "auto_output_filter" => {
                match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                    Some(value) => match value.to_ascii_uppercase().as_str() {
                        "NONE" => flags.auto_output_filter = AutoOutputFilter::None,
                        "ALL" => flags.auto_output_filter = AutoOutputFilter::All,
                        "PACKAGES" => flags.auto_output_filter = AutoOutputFilter::Packages,
                        "SUBPACKAGES" => flags.auto_output_filter = AutoOutputFilter::Subpackages,
                        // Not one of the four values: leave the flag for
                        // whatever validates enum flags next to reject.
                        _ => rest.push(arg.clone()),
                    },
                    None => rest.push(arg.clone()),
                }
            }
            _ => rest.push(arg.clone()),
        }
    }

    (flags, rest)
}

#[derive(Debug, thiserror::Error)]
#[error("invalid --output_filter regex: {0}")]
pub struct OutputFilterError(#[from] regex::Error);

/// A compiled, ready-to-use output filter.
#[derive(Debug)]
pub enum OutputFilter {
    Regex(Regex),
    Auto {
        packages: BTreeSet<String>,
        subpackages: bool,
    },
    ShowAll,
    ShowNone,
}

impl OutputFilter {
    /// Build the filter Bazel would use for `flags`, given the packages
    /// named by the command's target patterns (only consulted for
    /// `--auto_output_filter=packages`/`subpackages`).
    pub fn compile<I, S>(
        flags: &OutputFilterFlags,
        command_line_packages: I,
    ) -> Result<OutputFilter, OutputFilterError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        if let Some(pattern) = &flags.output_filter {
            return Ok(OutputFilter::Regex(Regex::new(pattern)?));
        }
        Ok(match flags.auto_output_filter {
            AutoOutputFilter::None => OutputFilter::ShowAll,
            AutoOutputFilter::All => OutputFilter::ShowNone,
            AutoOutputFilter::Packages => OutputFilter::Auto {
                packages: command_line_packages.into_iter().map(Into::into).collect(),
                subpackages: false,
            },
            AutoOutputFilter::Subpackages => OutputFilter::Auto {
                packages: command_line_packages.into_iter().map(Into::into).collect(),
                subpackages: true,
            },
        })
    }

    /// Whether output for a rule with label text `label` (what
    /// `--output_filter`'s regex matches against — Bazel matches the full
    /// label string) in `package` (what `--auto_output_filter`'s
    /// packages/subpackages mode compares) should be shown.
    pub fn allows(&self, label: &str, package: &str) -> bool {
        match self {
            OutputFilter::Regex(re) => re.is_match(label),
            OutputFilter::ShowAll => true,
            OutputFilter::ShowNone => false,
            OutputFilter::Auto {
                packages,
                subpackages,
            } => packages
                .iter()
                .any(|p| package == p || (*subpackages && package.starts_with(&format!("{p}/")))),
        }
    }
}

/// Bazel does not print the exact same warning text twice within one
/// invocation. `should_print` tracks what's already been shown, letting a
/// caller skip a repeat before ever formatting it for a terminal.
#[derive(Debug, Default)]
pub struct WarningDeduplicator {
    seen: std::collections::HashSet<String>,
}

impl WarningDeduplicator {
    /// Returns `true` the first time `message` is seen, `false` on every
    /// repeat within this deduplicator's lifetime (one build invocation).
    pub fn should_print(&mut self, message: impl Into<String>) -> bool {
        self.seen.insert(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn defaults_show_everything() {
        let (flags, rest) = extract(&args(&[]), "build");
        assert_eq!(flags, OutputFilterFlags::default());
        assert!(rest.is_empty());
        let filter = OutputFilter::compile(&flags, Vec::<String>::new()).unwrap();
        assert!(filter.allows("//foo:bar", "foo"));
    }

    #[test]
    fn output_filter_regex_takes_precedence_over_auto() {
        let (flags, rest) = extract(
            &args(&["--output_filter=^//foo", "--auto_output_filter=none"]),
            "build",
        );
        assert_eq!(flags.output_filter.as_deref(), Some("^//foo"));
        assert!(rest.is_empty());
        let filter = OutputFilter::compile(&flags, Vec::<String>::new()).unwrap();
        assert!(filter.allows("//foo:bar", "foo"));
        assert!(!filter.allows("//baz:qux", "baz"));
    }

    #[test]
    fn invalid_regex_is_an_error() {
        let flags = OutputFilterFlags {
            output_filter: Some("(unclosed".to_string()),
            ..Default::default()
        };
        assert!(OutputFilter::compile(&flags, Vec::<String>::new()).is_err());
    }

    #[test]
    fn auto_all_shows_nothing() {
        let (flags, _) = extract(&args(&["--auto_output_filter=all"]), "build");
        let filter = OutputFilter::compile(&flags, Vec::<String>::new()).unwrap();
        assert!(!filter.allows("//foo:bar", "foo"));
    }

    #[test]
    fn auto_packages_matches_only_named_packages() {
        let (flags, _) = extract(&args(&["--auto_output_filter=packages"]), "build");
        let filter = OutputFilter::compile(&flags, vec!["foo"]).unwrap();
        assert!(filter.allows("//foo:bar", "foo"));
        assert!(!filter.allows("//foo/sub:bar", "foo/sub"));
        assert!(!filter.allows("//baz:qux", "baz"));
    }

    #[test]
    fn auto_subpackages_matches_named_packages_and_below() {
        let (flags, _) = extract(&args(&["--auto_output_filter=subpackages"]), "build");
        let filter = OutputFilter::compile(&flags, vec!["foo"]).unwrap();
        assert!(filter.allows("//foo:bar", "foo"));
        assert!(filter.allows("//foo/sub:bar", "foo/sub"));
        assert!(!filter.allows("//baz:qux", "baz"));
    }

    #[test]
    fn unrelated_tokens_pass_through() {
        let (flags, rest) = extract(&args(&["//foo:bar", "--keep_going"]), "build");
        assert_eq!(flags, OutputFilterFlags::default());
        assert_eq!(rest, args(&["//foo:bar", "--keep_going"]));
    }

    #[test]
    fn deduplicator_suppresses_repeats() {
        let mut dedup = WarningDeduplicator::default();
        assert!(dedup.should_print("warning: foo"));
        assert!(!dedup.should_print("warning: foo"));
        assert!(dedup.should_print("warning: bar"));
    }
}
