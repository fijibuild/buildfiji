//! `--execution_log_compact_file=<path>`: where to write the compact spawn
//! log (see `fjfj_remote::execution_log`). Pulled out of a command's raw
//! argv slice the same way as [`crate::diagnostics_flags::extract`].
//!
//! Bazel's `--execution_log_binary_file` and `--execution_log_json_file`
//! (the two older, larger formats) aren't extracted here: the compact
//! format is what Bazel itself now recommends, and there is no spawn
//! execution yet to log in any format, so there is nothing gained by
//! carrying three flags' worth of plumbing instead of one until a second
//! format is actually requested.

use std::path::PathBuf;

use crate::flag_registry::FlagRegistry;

/// Flag values for this module, defaulting to Bazel's own: no log written.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExecutionLogFlags {
    /// `--execution_log_compact_file=<path>`. Bazel's `EmptyToNullPathFragment`
    /// type converter treats an empty string the same as the flag being
    /// absent, so `--execution_log_compact_file=` clears rather than sets it.
    pub execution_log_compact_file: Option<PathBuf>,
}

/// Pull [`ExecutionLogFlags`] for `command` out of `args`, returning the
/// flags found and every argument *not* consumed, in their original
/// relative order.
pub fn extract(args: &[String], command: &str) -> (ExecutionLogFlags, Vec<String>) {
    let registry = FlagRegistry::global();
    let mut flags = ExecutionLogFlags::default();
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
            "execution_log_compact_file" => {
                match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                    Some(value) if value.is_empty() => flags.execution_log_compact_file = None,
                    Some(value) => flags.execution_log_compact_file = Some(PathBuf::from(value)),
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
    fn defaults_to_no_log() {
        let (flags, rest) = extract(&args(&[]), "build");
        assert_eq!(flags, ExecutionLogFlags::default());
        assert!(rest.is_empty());
    }

    #[test]
    fn attached_value() {
        let (flags, rest) = extract(
            &args(&["--execution_log_compact_file=/tmp/exec.log"]),
            "build",
        );
        assert_eq!(
            flags.execution_log_compact_file,
            Some(PathBuf::from("/tmp/exec.log"))
        );
        assert!(rest.is_empty());
    }

    #[test]
    fn space_separated_value() {
        let (flags, rest) = extract(
            &args(&["--execution_log_compact_file", "/tmp/exec.log"]),
            "build",
        );
        assert_eq!(
            flags.execution_log_compact_file,
            Some(PathBuf::from("/tmp/exec.log"))
        );
        assert!(rest.is_empty());
    }

    #[test]
    fn old_name_alias_resolves() {
        // buildfiji-k62.3: --execution_log_compact_file's old_name in the
        // flag registry, from when it was --experimental_execution_log_compact_file.
        let (flags, rest) = extract(
            &args(&["--experimental_execution_log_compact_file=/tmp/exec.log"]),
            "build",
        );
        assert_eq!(
            flags.execution_log_compact_file,
            Some(PathBuf::from("/tmp/exec.log"))
        );
        assert!(rest.is_empty());
    }

    #[test]
    fn empty_value_clears_it() {
        let (flags, rest) = extract(&args(&["--execution_log_compact_file="]), "build");
        assert_eq!(flags.execution_log_compact_file, None);
        assert!(rest.is_empty());
    }

    #[test]
    fn unrelated_tokens_pass_through() {
        let (flags, rest) = extract(&args(&["//foo:bar", "--keep_going"]), "build");
        assert_eq!(flags, ExecutionLogFlags::default());
        assert_eq!(rest, args(&["//foo:bar", "--keep_going"]));
    }
}
