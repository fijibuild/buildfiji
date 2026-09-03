//! `--build_event_publish_all_actions`, `--bes_results_url`, and
//! `--bes_timeout`: the flags an IDE or CI dashboard driving fjfj through
//! the Build Event Protocol cares about — whether every action (not just
//! failures) shows up as a BEP event, the URL to print pointing at wherever
//! those events end up, and how long to keep the connection open waiting for
//! the upload to finish after the build itself is done. Pulled out of a
//! command's raw argv slice the same way as [`crate::diagnostics_flags::extract`].
//!
//! There is no BEP writer yet to hand these to (`docs/design/telemetry.md`
//! notes BEP as a compatibility export still to build off the `tracing` span
//! stream) — this only makes the flags parse the way Bazel's do, including
//! `--bes_timeout`'s duration syntax, so nothing else has to.

use std::time::Duration;

use crate::flag_registry::FlagRegistry;

/// Flag values for this module, defaulting to Bazel's own: don't publish
/// every action, no results URL to print, and no upload timeout (Bazel
/// documents `0` as "wait indefinitely").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BesFlags {
    pub publish_all_actions: bool,
    /// `--bes_results_url=<url>`. Bazel prints this with the invocation id
    /// appended; empty means there is nothing to print.
    pub results_url: String,
    pub timeout: Duration,
}

impl Default for BesFlags {
    fn default() -> Self {
        BesFlags {
            publish_all_actions: false,
            results_url: String::new(),
            timeout: Duration::ZERO,
        }
    }
}

/// Pull [`BesFlags`] for `command` out of `args`, returning the flags found
/// and every argument *not* consumed, in their original relative order.
/// A `--bes_timeout` value that doesn't parse is left in the returned
/// tokens rather than rejected here, the same as an unrecognised
/// `--auto_output_filter` value in [`crate::output_filter::extract`] — this
/// function only extracts flags it can make sense of.
pub fn extract(args: &[String], command: &str) -> (BesFlags, Vec<String>) {
    let registry = FlagRegistry::global();
    let mut flags = BesFlags::default();
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
            "build_event_publish_all_actions" => flags.publish_all_actions = !m.negated,
            "bes_results_url" => {
                match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                    Some(value) => flags.results_url = value,
                    None => rest.push(arg.clone()),
                }
            }
            "bes_timeout" => match m.value.map(str::to_string).or_else(|| iter.next().cloned()) {
                Some(value) => match parse_duration(&value) {
                    Ok(timeout) => flags.timeout = timeout,
                    Err(_) => rest.push(arg.clone()),
                },
                None => rest.push(arg.clone()),
            },
            _ => rest.push(arg.clone()),
        }
    }

    (flags, rest)
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error(
    "invalid duration {0:?}: expected a number followed by one of d, h, m, s, ms, ns (or exactly \"0\")"
)]
pub struct DurationError(String);

/// Bazel's `Converters.DurationConverter` syntax
/// (`^([0-9]+)(d|h|m|s|ms|ns)$`, plus the special case that bare `0` needs
/// no unit): a single number immediately followed by one unit, never a
/// combination like `1h30m`.
pub fn parse_duration(s: &str) -> Result<Duration, DurationError> {
    if s == "0" {
        return Ok(Duration::ZERO);
    }
    let split_at = s
        .find(|c: char| !c.is_ascii_digit())
        .filter(|&i| i > 0)
        .ok_or_else(|| DurationError(s.to_string()))?;
    let (digits, unit) = s.split_at(split_at);
    let n: u64 = digits.parse().map_err(|_| DurationError(s.to_string()))?;
    match unit {
        "ns" => Ok(Duration::from_nanos(n)),
        "ms" => Ok(Duration::from_millis(n)),
        "s" => Ok(Duration::from_secs(n)),
        "m" => Ok(Duration::from_secs(n.saturating_mul(60))),
        "h" => Ok(Duration::from_secs(n.saturating_mul(3600))),
        "d" => Ok(Duration::from_secs(n.saturating_mul(86400))),
        _ => Err(DurationError(s.to_string())),
    }
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
        assert_eq!(flags, BesFlags::default());
        assert!(rest.is_empty());
    }

    #[test]
    fn publish_all_actions_and_negation() {
        let (flags, _) = extract(&args(&["--build_event_publish_all_actions"]), "build");
        assert!(flags.publish_all_actions);
        let (flags, _) = extract(
            &args(&[
                "--build_event_publish_all_actions",
                "--nobuild_event_publish_all_actions",
            ]),
            "build",
        );
        assert!(!flags.publish_all_actions);
    }

    #[test]
    fn results_url_attached_value() {
        let (flags, rest) = extract(
            &args(&["--bes_results_url=https://example.com/invocations/"]),
            "build",
        );
        assert_eq!(flags.results_url, "https://example.com/invocations/");
        assert!(rest.is_empty());
    }

    #[test]
    fn bes_timeout_space_separated_value() {
        let (flags, rest) = extract(&args(&["--bes_timeout", "10s"]), "build");
        assert_eq!(flags.timeout, Duration::from_secs(10));
        assert!(rest.is_empty());
    }

    #[test]
    fn bes_timeout_bare_zero_needs_no_unit() {
        let (flags, rest) = extract(&args(&["--bes_timeout=0"]), "build");
        assert_eq!(flags.timeout, Duration::ZERO);
        assert!(rest.is_empty());
    }

    #[test]
    fn malformed_bes_timeout_is_left_for_downstream_validation() {
        let (flags, rest) = extract(&args(&["--bes_timeout=1h30m"]), "build");
        assert_eq!(flags.timeout, Duration::ZERO);
        assert_eq!(rest, args(&["--bes_timeout=1h30m"]));
    }

    #[test]
    fn unrelated_tokens_pass_through() {
        let (flags, rest) = extract(&args(&["//foo:bar", "--keep_going"]), "build");
        assert_eq!(flags, BesFlags::default());
        assert_eq!(rest, args(&["//foo:bar", "--keep_going"]));
    }

    #[test]
    fn parse_duration_accepts_every_unit() {
        assert_eq!(parse_duration("5ns").unwrap(), Duration::from_nanos(5));
        assert_eq!(parse_duration("5ms").unwrap(), Duration::from_millis(5));
        assert_eq!(parse_duration("5s").unwrap(), Duration::from_secs(5));
        assert_eq!(parse_duration("5m").unwrap(), Duration::from_secs(300));
        assert_eq!(parse_duration("5h").unwrap(), Duration::from_secs(18_000));
        assert_eq!(parse_duration("5d").unwrap(), Duration::from_secs(432_000));
    }

    #[test]
    fn parse_duration_rejects_combined_units() {
        assert!(parse_duration("1h30m").is_err());
    }

    #[test]
    fn parse_duration_rejects_no_digits_or_no_unit() {
        assert!(parse_duration("s").is_err());
        assert!(parse_duration("10").is_err());
        assert!(parse_duration("").is_err());
    }
}
