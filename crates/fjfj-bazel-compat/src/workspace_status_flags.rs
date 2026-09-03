//! `--workspace_status_command=<program>`, `--[no]stamp`, and
//! `--embed_label=<value>`: the flags that feed
//! [`crate::workspace_status::WorkspaceStatus`]. Pulled out of a command's
//! raw argv slice the same way as [`crate::diagnostics_flags::extract`]
//! rather than given individual clap fields — see that module's doc
//! comment for why.

use std::path::PathBuf;

use crate::flag_registry::FlagRegistry;

/// Flag names this module reads, for `clap_flags::validate`'s
/// unimplemented-flag gate.
pub const IMPLEMENTED: &[&str] = &["workspace_status_command", "stamp", "embed_label"];

/// Values for the flags in this module, defaulting to Bazel's own:
/// `--nostamp` (test rules always build unstamped regardless), no
/// `--workspace_status_command` (equivalent to Bazel's documented
/// `/bin/true` — no extra keys, just the built-ins), and an empty
/// `--embed_label`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkspaceStatusFlags {
    /// `--workspace_status_command=<program>`: a native executable run
    /// once at the start of the build; its stdout is parsed by
    /// [`crate::workspace_status::WorkspaceStatus::parse`].
    pub workspace_status_command: Option<PathBuf>,
    /// `--stamp`/`--nostamp`: whether `stamp = -1` rules (the default for
    /// `*_binary`) embed workspace status into their outputs.
    pub stamp: bool,
    /// `--embed_label=<value>`: becomes the `BUILD_EMBED_LABEL` status key.
    pub embed_label: String,
}

/// Pull [`WorkspaceStatusFlags`] for `command` out of `args`, returning the
/// flags found and every argument *not* consumed, in their original
/// relative order. A token this function doesn't recognise — including a
/// real Bazel flag fjfj hasn't wired up elsewhere — is left untouched.
pub fn extract(args: &[String], command: &str) -> (WorkspaceStatusFlags, Vec<String>) {
    let registry = FlagRegistry::global();
    let mut flags = WorkspaceStatusFlags::default();
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
            "stamp" => flags.stamp = !m.negated,
            "workspace_status_command" => {
                match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                    Some(value) => flags.workspace_status_command = Some(PathBuf::from(value)),
                    None => rest.push(arg.clone()),
                }
            }
            "embed_label" => match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                Some(value) => flags.embed_label = value,
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
    fn defaults_are_bazels_defaults() {
        let (flags, rest) = extract(&args(&[]), "build");
        assert_eq!(flags, WorkspaceStatusFlags::default());
        assert!(rest.is_empty());
    }

    #[test]
    fn stamp_and_nostamp() {
        let (flags, _) = extract(&args(&["--stamp"]), "build");
        assert!(flags.stamp);
        let (flags, _) = extract(&args(&["--stamp", "--nostamp"]), "build");
        assert!(!flags.stamp);
    }

    #[test]
    fn workspace_status_command_attached_value() {
        let (flags, rest) = extract(
            &args(&["--workspace_status_command=/usr/bin/true"]),
            "build",
        );
        assert_eq!(
            flags.workspace_status_command,
            Some(PathBuf::from("/usr/bin/true"))
        );
        assert!(rest.is_empty());
    }

    #[test]
    fn workspace_status_command_space_separated_value() {
        let (flags, rest) = extract(
            &args(&["--workspace_status_command", "/usr/bin/true"]),
            "build",
        );
        assert_eq!(
            flags.workspace_status_command,
            Some(PathBuf::from("/usr/bin/true"))
        );
        assert!(rest.is_empty());
    }

    #[test]
    fn embed_label() {
        let (flags, rest) = extract(&args(&["--embed_label=v1.2.3"]), "build");
        assert_eq!(flags.embed_label, "v1.2.3");
        assert!(rest.is_empty());
    }

    #[test]
    fn unrelated_tokens_pass_through() {
        let (flags, rest) = extract(&args(&["//foo:bar", "--keep_going"]), "build");
        assert_eq!(flags, WorkspaceStatusFlags::default());
        assert_eq!(rest, args(&["//foo:bar", "--keep_going"]));
    }
}
