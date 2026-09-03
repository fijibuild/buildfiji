//! `--remote_grpc_log=<path>`, `--remote_cache_compression`, and
//! `--experimental_remote_cache_compression_threshold=<bytes>`: the
//! flags that feed `fjfj_remote::grpc_log` and REAPI CAS blob compression.
//! Pulled out of a command's raw argv slice the same way as
//! [`crate::diagnostics_flags::extract`].

use std::path::PathBuf;

use crate::flag_registry::FlagRegistry;

/// Flag values for this module, defaulting to Bazel's own: no gRPC log, no
/// cache compression, and a 100-byte compression threshold (ineffectual
/// while compression itself is off).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFlags {
    /// `--remote_grpc_log=<path>`. Bazel's `EmptyToNullPathFragment` type
    /// converter treats an empty string the same as the flag being absent.
    pub remote_grpc_log: Option<PathBuf>,
    /// `--remote_cache_compression`/`--noremote_cache_compression`
    /// (old name `--experimental_remote_cache_compression`).
    pub remote_cache_compression: bool,
    /// `--experimental_remote_cache_compression_threshold=<bytes>`: blobs
    /// smaller than this are never compressed even when the flag above is set.
    pub remote_cache_compression_threshold: i64,
}

impl Default for RemoteFlags {
    fn default() -> Self {
        RemoteFlags {
            remote_grpc_log: None,
            remote_cache_compression: false,
            remote_cache_compression_threshold: 100,
        }
    }
}

/// Pull [`RemoteFlags`] for `command` out of `args`, returning the flags
/// found and every argument *not* consumed, in their original relative
/// order.
pub fn extract(args: &[String], command: &str) -> (RemoteFlags, Vec<String>) {
    let registry = FlagRegistry::global();
    let mut flags = RemoteFlags::default();
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
            "remote_grpc_log" => {
                match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                    Some(value) if value.is_empty() => flags.remote_grpc_log = None,
                    Some(value) => flags.remote_grpc_log = Some(PathBuf::from(value)),
                    None => rest.push(arg.clone()),
                }
            }
            "remote_cache_compression" => flags.remote_cache_compression = !m.negated,
            "experimental_remote_cache_compression_threshold" => {
                match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                    Some(value) => match value.parse::<i64>() {
                        Ok(threshold) => flags.remote_cache_compression_threshold = threshold,
                        Err(_) => rest.push(arg.clone()),
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
        assert_eq!(flags, RemoteFlags::default());
        assert!(rest.is_empty());
    }

    #[test]
    fn remote_grpc_log_attached_value() {
        let (flags, rest) = extract(&args(&["--remote_grpc_log=/tmp/grpc.log"]), "build");
        assert_eq!(flags.remote_grpc_log, Some(PathBuf::from("/tmp/grpc.log")));
        assert!(rest.is_empty());
    }

    #[test]
    fn remote_grpc_log_old_name_alias_resolves() {
        let (flags, rest) = extract(
            &args(&["--experimental_remote_grpc_log=/tmp/grpc.log"]),
            "build",
        );
        assert_eq!(flags.remote_grpc_log, Some(PathBuf::from("/tmp/grpc.log")));
        assert!(rest.is_empty());
    }

    #[test]
    fn remote_cache_compression_and_negation() {
        let (flags, _) = extract(&args(&["--remote_cache_compression"]), "build");
        assert!(flags.remote_cache_compression);
        let (flags, _) = extract(
            &args(&["--remote_cache_compression", "--noremote_cache_compression"]),
            "build",
        );
        assert!(!flags.remote_cache_compression);
    }

    #[test]
    fn remote_cache_compression_old_name_alias_resolves() {
        let (flags, rest) = extract(&args(&["--experimental_remote_cache_compression"]), "build");
        assert!(flags.remote_cache_compression);
        assert!(rest.is_empty());
    }

    #[test]
    fn compression_threshold_attached_value() {
        let (flags, rest) = extract(
            &args(&["--experimental_remote_cache_compression_threshold=1024"]),
            "build",
        );
        assert_eq!(flags.remote_cache_compression_threshold, 1024);
        assert!(rest.is_empty());
    }

    #[test]
    fn unrelated_tokens_pass_through() {
        let (flags, rest) = extract(&args(&["//foo:bar", "--keep_going"]), "build");
        assert_eq!(flags, RemoteFlags::default());
        assert_eq!(rest, args(&["//foo:bar", "--keep_going"]));
    }
}
