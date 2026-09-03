//! `--flag_alias=<name>=<label>`: a shorthand name for a Starlark build
//! setting flag (`--//pkg:setting=value`). Bazel resolves these in two
//! passes: collect every `--flag_alias` first (it's accepted by every
//! command, so order relative to other flags doesn't matter), then
//! rewrite any later `--<alias>`/`--<alias>=value` token into the real
//! `--<label>`/`--<label>=value` form. [`extract`] is the first pass,
//! [`apply`] the second — split apart because the caller needs the alias
//! table intact (e.g. for logging) independent of rewriting.
//!
//! `apply`'s output is the label form fjfj-starlark's build-setting
//! resolution will understand once it exists; until then it's just a
//! different unrecognised flag name, same as today.

use std::collections::BTreeMap;

use crate::flag_registry::FlagRegistry;

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum FlagAliasError {
    #[error("--flag_alias expects NAME=LABEL, got {0:?}")]
    Malformed(String),
}

/// Alias name -> the build setting label it stands for.
pub type FlagAliasTable = BTreeMap<String, String>;

/// Pull every `--flag_alias=<name>=<label>` (attached or space-separated
/// form) out of `args`, returning the alias table and every other
/// argument untouched, in order. `flag_alias` itself is accepted by every
/// command, so this doesn't need a `command` parameter the way most
/// `extract` functions in this crate do.
pub fn extract(args: &[String]) -> Result<(FlagAliasTable, Vec<String>), FlagAliasError> {
    let registry = FlagRegistry::global();
    let mut aliases = FlagAliasTable::new();
    let mut rest = Vec::with_capacity(args.len());
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        if !arg.starts_with('-') {
            rest.push(arg.clone());
            continue;
        }
        // "build" is just one command from flag_alias's (identical for
        // every command) `commands` list — any would resolve it.
        let Ok(m) = registry.resolve(arg, "build") else {
            rest.push(arg.clone());
            continue;
        };
        if m.flag.name != "flag_alias" {
            rest.push(arg.clone());
            continue;
        }
        let value = match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
            Some(value) => value,
            None => return Err(FlagAliasError::Malformed(arg.clone())),
        };
        let (name, label) = value
            .split_once('=')
            .ok_or_else(|| FlagAliasError::Malformed(value.clone()))?;
        aliases.insert(name.to_string(), label.to_string());
    }

    Ok((aliases, rest))
}

/// Rewrite `--<alias>`/`--<alias>=value` tokens in `args` per `aliases`,
/// to `--<label>`/`--<label>=value`. A token naming no known alias is
/// left untouched — it's a real flag, a target pattern, or an alias
/// `--flag_alias` never defined, none of which this function judges.
pub fn apply(aliases: &FlagAliasTable, args: &[String]) -> Vec<String> {
    args.iter()
        .map(|arg| {
            let Some(rest) = arg.strip_prefix("--") else {
                return arg.clone();
            };
            let (name, value) = match rest.split_once('=') {
                Some((n, v)) => (n, Some(v)),
                None => (rest, None),
            };
            match aliases.get(name) {
                Some(label) => match value {
                    Some(v) => format!("--{label}={v}"),
                    None => format!("--{label}"),
                },
                None => arg.clone(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn extracts_attached_form() {
        let (aliases, rest) = extract(&args(&["--flag_alias=foo=//pkg:setting"])).unwrap();
        assert_eq!(
            aliases.get("foo").map(String::as_str),
            Some("//pkg:setting")
        );
        assert!(rest.is_empty());
    }

    #[test]
    fn extracts_space_separated_form() {
        let (aliases, rest) = extract(&args(&["--flag_alias", "foo=//pkg:setting"])).unwrap();
        assert_eq!(
            aliases.get("foo").map(String::as_str),
            Some("//pkg:setting")
        );
        assert!(rest.is_empty());
    }

    #[test]
    fn repeatable_for_multiple_aliases() {
        let (aliases, _) = extract(&args(&[
            "--flag_alias=foo=//pkg:a",
            "--flag_alias=bar=//pkg:b",
        ]))
        .unwrap();
        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases["foo"], "//pkg:a");
        assert_eq!(aliases["bar"], "//pkg:b");
    }

    #[test]
    fn missing_equals_is_malformed() {
        assert_eq!(
            extract(&args(&["--flag_alias=foo"])),
            Err(FlagAliasError::Malformed("foo".to_string()))
        );
    }

    #[test]
    fn unrelated_tokens_pass_through() {
        let (aliases, rest) = extract(&args(&["--keep_going", "//foo:bar"])).unwrap();
        assert!(aliases.is_empty());
        assert_eq!(rest, args(&["--keep_going", "//foo:bar"]));
    }

    #[test]
    fn apply_rewrites_attached_value() {
        let mut aliases = FlagAliasTable::new();
        aliases.insert("foo".to_string(), "//pkg:setting".to_string());
        let out = apply(&aliases, &args(&["--foo=on", "--keep_going"]));
        assert_eq!(out, args(&["--//pkg:setting=on", "--keep_going"]));
    }

    #[test]
    fn apply_rewrites_bare_flag() {
        let mut aliases = FlagAliasTable::new();
        aliases.insert("foo".to_string(), "//pkg:setting".to_string());
        let out = apply(&aliases, &args(&["--foo"]));
        assert_eq!(out, args(&["--//pkg:setting"]));
    }

    #[test]
    fn apply_leaves_unknown_alias_untouched() {
        let aliases = FlagAliasTable::new();
        let out = apply(&aliases, &args(&["--foo=on"]));
        assert_eq!(out, args(&["--foo=on"]));
    }
}
