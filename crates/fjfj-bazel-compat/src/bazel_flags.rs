//! The full Bazel flag table, as reported by `bazel help flags-as-proto`
//! for the pinned Bazel release, checked in as generated Rust data.
//!
//! Mirrors `bazel_flags.FlagInfo` from `proto/bazel_flags.proto` (vendored
//! from the Bazel release this table was generated against — that `.proto`
//! is documentation for readers of `src/bin/refresh_bazel_flags.rs`, not
//! compiled; see that file for why). A flag is one of Bazel's
//! `--incompatible_*` migration flags iff its `metadata_tags` contains
//! `"INCOMPATIBLE_CHANGE"` — there's no separate table for those.
//!
//! To refresh after bumping the pinned Bazel version:
//! `cargo run -p fjfj-bazel-compat --bin refresh_bazel_flags`.

mod generated;

pub use generated::FLAGS;

/// The Bazel release [`FLAGS`] was generated from.
pub const BAZEL_VERSION: &str = generated::BAZEL_VERSION;

/// One flag, e.g. `--copt` or `--incompatible_no_implicit_watch_label`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlagInfo {
    /// Flag name, without leading dashes.
    pub name: &'static str,
    /// True if `--no<name>` also exists.
    pub has_negative_flag: bool,
    /// Help text.
    pub documentation: &'static str,
    /// Commands this flag is accepted by, e.g. `["build", "test"]`.
    /// `"common"` does not appear here; startup flags carry `["startup"]`.
    pub commands: &'static [&'static str],
    /// Single-character abbreviation, without leading dash, if any.
    pub abbreviation: Option<&'static str>,
    /// True if the flag may be repeated in one invocation.
    pub allows_multiple: bool,
    /// Bazel's effect tags, e.g. `["AFFECTS_OUTPUTS"]`.
    pub effect_tags: &'static [&'static str],
    /// Bazel's metadata tags, e.g. `["INCOMPATIBLE_CHANGE"]` — see the
    /// module docs for how this marks `--incompatible_*` flags.
    pub metadata_tags: &'static [&'static str],
    /// Documentation category Bazel groups this flag under.
    pub documentation_category: &'static str,
    /// False for a value-less flag, e.g. `--subcommands`.
    pub requires_value: bool,
    /// The default value, if any, as Bazel's help formats it.
    pub default_value: Option<&'static str>,
    /// A deprecated former name for this flag, without leading dashes.
    pub old_name: Option<&'static str>,
    /// The deprecation warning shown when this flag is used, if any.
    pub deprecation_warning: Option<&'static str>,
    /// Other flags implicitly added when this one is set.
    pub option_expansions: &'static [&'static str],
    /// The expected value type/"converter", e.g. `"Boolean"`, `"Integer"`.
    pub type_converter: Option<&'static str>,
    /// Valid values, for an enum-typed flag.
    pub enum_values: &'static [&'static str],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_non_empty_and_has_no_duplicate_names() {
        assert!(
            !FLAGS.is_empty(),
            "flag table is empty — did the refresh script run?"
        );
        let mut names: Vec<&str> = FLAGS.iter().map(|f| f.name).collect();
        names.sort_unstable();
        let mut deduped = names.clone();
        deduped.dedup();
        assert_eq!(
            names, deduped,
            "duplicate flag names in the generated table"
        );
    }

    #[test]
    fn every_flag_has_a_name_and_at_least_one_command() {
        for flag in FLAGS {
            assert!(!flag.name.is_empty());
            assert!(
                !flag.commands.is_empty(),
                "flag {:?} has no commands",
                flag.name
            );
        }
    }

    #[test]
    fn build_command_is_well_represented() {
        assert!(
            FLAGS
                .iter()
                .any(|f| f.name == "jobs" && f.commands.contains(&"build")),
            "expected a `build`-applicable `jobs` flag in the table"
        );
    }
}
