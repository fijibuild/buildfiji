//! `--registry`, `--allow_yanked_versions`, `--ignore_dev_dependency`, and
//! `--override_module`: the bzlmod resolution flags. Pulled out of a
//! command's raw argv slice the same way as
//! [`crate::diagnostics_flags::extract`].
//!
//! This module only produces raw values — turning `allow_yanked_versions`
//! into a `fjfj_bzlmod::YankedPolicy`, or `override_module` into a
//! `ModuleOverride`, needs types from `fjfj-bzlmod`, which this crate does
//! not (and should not) depend on. That's `fjfj-cli`'s job
//! (buildfiji-gwl.17).

use crate::flag_registry::FlagRegistry;

/// Flag names this module reads, for `clap_flags::validate`'s
/// unimplemented-flag gate.
pub const IMPLEMENTED: &[&str] = &[
    "registry",
    "allow_yanked_versions",
    "ignore_dev_dependency",
    "override_module",
];

/// Raw bzlmod flag values, defaulting to Bazel's own.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BzlmodFlags {
    /// `--registry`, repeatable. Bazel: "any occurrence replaces the
    /// default `https://bcr.bazel.build` list" — empty here means exactly
    /// that default, not "no registries".
    pub registry: Vec<String>,
    /// `--allow_yanked_versions`'s raw value: the literal `all`, or a
    /// `name1@version1,name2@version2` list. Left unparsed — the
    /// module/version split needs `fjfj_bzlmod::Version`.
    pub allow_yanked_versions: Option<String>,
    /// `--ignore_dev_dependency`/`--noignore_dev_dependency`.
    pub ignore_dev_dependency: bool,
    /// `--override_module=name=path`, repeatable, in `(name, path)` form.
    /// A malformed value (no `=`) is left in the returned `rest` rather
    /// than silently dropped.
    pub override_module: Vec<(String, String)>,
}

/// Pull [`BzlmodFlags`] for `command` out of `args`, returning the flags
/// found and every argument *not* consumed, in their original relative
/// order.
pub fn extract(args: &[String], command: &str) -> (BzlmodFlags, Vec<String>) {
    let registry = FlagRegistry::global();
    let mut flags = BzlmodFlags::default();
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
            "registry" => match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                Some(value) => flags.registry.push(value),
                None => rest.push(arg.clone()),
            },
            "allow_yanked_versions" => {
                match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                    Some(value) => flags.allow_yanked_versions = Some(value),
                    None => rest.push(arg.clone()),
                }
            }
            "ignore_dev_dependency" => flags.ignore_dev_dependency = !m.negated,
            "override_module" => {
                match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                    Some(value) => match value.split_once('=') {
                        Some((name, path)) => flags
                            .override_module
                            .push((name.to_owned(), path.to_owned())),
                        None => rest.push(arg.clone()),
                    },
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
        assert_eq!(flags, BzlmodFlags::default());
        assert!(rest.is_empty());
    }

    #[test]
    fn registry_is_repeatable_and_ordered() {
        let (flags, rest) = extract(
            &args(&["--registry=file:///a", "--registry=file:///b"]),
            "build",
        );
        assert_eq!(flags.registry, ["file:///a", "file:///b"]);
        assert!(rest.is_empty());
    }

    #[test]
    fn allow_yanked_versions_attached_value() {
        let (flags, rest) = extract(&args(&["--allow_yanked_versions=all"]), "build");
        assert_eq!(flags.allow_yanked_versions.as_deref(), Some("all"));
        assert!(rest.is_empty());
    }

    #[test]
    fn ignore_dev_dependency_and_negation() {
        let (flags, _) = extract(&args(&["--ignore_dev_dependency"]), "build");
        assert!(flags.ignore_dev_dependency);
        let (flags, _) = extract(
            &args(&["--ignore_dev_dependency", "--noignore_dev_dependency"]),
            "build",
        );
        assert!(!flags.ignore_dev_dependency);
    }

    #[test]
    fn override_module_splits_name_and_path() {
        let (flags, rest) = extract(&args(&["--override_module=foo=../foo"]), "build");
        assert_eq!(
            flags.override_module,
            [("foo".to_owned(), "../foo".to_owned())]
        );
        assert!(rest.is_empty());
    }

    #[test]
    fn override_module_repeatable() {
        let (flags, _) = extract(
            &args(&[
                "--override_module=foo=../foo",
                "--override_module=bar=../bar",
            ]),
            "build",
        );
        assert_eq!(
            flags.override_module,
            [
                ("foo".to_owned(), "../foo".to_owned()),
                ("bar".to_owned(), "../bar".to_owned()),
            ]
        );
    }

    #[test]
    fn override_module_without_equals_is_left_unconsumed() {
        let (flags, rest) = extract(&args(&["--override_module=nope"]), "build");
        assert!(flags.override_module.is_empty());
        assert_eq!(rest, args(&["--override_module=nope"]));
    }

    #[test]
    fn unrelated_tokens_pass_through() {
        let (flags, rest) = extract(&args(&["//foo:bar", "--keep_going"]), "build");
        assert_eq!(flags, BzlmodFlags::default());
        assert_eq!(rest, args(&["//foo:bar", "--keep_going"]));
    }
}
