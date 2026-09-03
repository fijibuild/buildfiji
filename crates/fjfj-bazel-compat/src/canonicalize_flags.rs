//! `canonicalize-flags`: turn a list of raw flag tokens written for one
//! command into Bazel's canonical `--name`, `--noname`, or `--name=value`
//! form — the same normalisation Bazel's own `canonicalize-flags` command
//! performs, useful for diffing two differently-abbreviated invocations.
//! Built on the same [`FlagRegistry`] every other flag-consuming module in
//! this crate uses, rather than a bespoke parser.

use crate::flag_registry::{FlagRegistry, UnknownFlagError};

/// Canonicalize `args` for `command`. Every token must resolve to a flag
/// known for `command` — `canonicalize-flags` takes only flags, no target
/// patterns or other positional arguments, matching Bazel — so the first
/// token that doesn't resolve is the error.
pub fn canonicalize(args: &[String], command: &str) -> Result<Vec<String>, UnknownFlagError> {
    let registry = FlagRegistry::global();
    let mut out = Vec::with_capacity(args.len());
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        let m = registry.resolve(arg, command)?;
        let canonical = if m.flag.type_converter == Some("Boolean") {
            if m.negated {
                format!("--no{}", m.flag.name)
            } else {
                format!("--{}", m.flag.name)
            }
        } else {
            match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                Some(value) => format!("--{}={value}", m.flag.name),
                // No value at all (attached or following): nothing sane
                // to canonicalize to; leave the bare name for the caller
                // to notice as still-wrong rather than inventing a value.
                None => format!("--{}", m.flag.name),
            }
        };
        out.push(canonical);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn abbreviation_and_negation_expand_to_canonical_form() {
        let out = canonicalize(&args(&["-k", "--nokeep_going"]), "build").unwrap();
        assert_eq!(out, vec!["--keep_going", "--nokeep_going"]);
    }

    #[test]
    fn space_separated_value_becomes_attached() {
        let out = canonicalize(&args(&["--explain", "/tmp/why.log"]), "build").unwrap();
        assert_eq!(out, vec!["--explain=/tmp/why.log"]);
    }

    #[test]
    fn already_attached_value_is_kept() {
        let out = canonicalize(&args(&["--explain=/tmp/why.log"]), "build").unwrap();
        assert_eq!(out, vec!["--explain=/tmp/why.log"]);
    }

    #[test]
    fn unknown_flag_is_an_error() {
        assert!(canonicalize(&args(&["--not_a_real_flag"]), "build").is_err());
    }

    #[test]
    fn target_pattern_is_an_error_same_as_bazel() {
        assert!(canonicalize(&args(&["//foo:bar"]), "build").is_err());
    }
}
