//! Bazel module versions, ported from Bazel 9.2.0's
//! `bazel/bzlmod/Version.java`.
//!
//! The format is `RELEASE[-PRERELEASE][+BUILD]`, a deliberate loosening of
//! SemVer: the release part may have any number of dot-separated segments
//! (not just three) and a segment may contain letters as well as digits.
//! Every valid SemVer version is a valid module version and compares the
//! same way, which is the property the registry ecosystem relies on.
//!
//! Two details drive most of the code below and both come straight from
//! Bazel:
//!
//! - The **build metadata is dropped on parse**. Bazel never stores it,
//!   never prints it and never sends it to a registry, which is what makes
//!   ordering "consistent with equals": `a == b` exactly when
//!   `a.cmp(b) == Equal`.
//! - The **empty version compares greater than everything else**. It is
//!   not "version zero"; it is the sentinel for a module with a
//!   non-registry override, and selection relies on it sorting above every
//!   real version (see [`crate::selection`], where it doubles as the "no
//!   allowed version" sentinel).

use std::cmp::Ordering;
use std::fmt;
use std::sync::LazyLock;

use regex::Regex;

/// A version in the Bazel module system.
///
/// Ordering is Bazel's: empty last, then release identifiers
/// lexicographically, then a prerelease before its own release, then
/// prerelease identifiers lexicographically.
#[derive(Debug, Clone)]
pub struct Version {
    release: Vec<Identifier>,
    prerelease: Vec<Identifier>,
    /// The version string with any build metadata stripped. This is the
    /// identity of the version: equality and hashing use it alone, and it
    /// is what gets written into URLs, lockfiles and repo names.
    normalized: String,
}

/// A dot-separated segment of a version.
///
/// Bazel compares digits-only segments numerically and everything else by
/// ASCII, with digits-only sorting below the rest — so `1.9 < 1.10` but
/// `1.9 < 1.alpha`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Identifier {
    /// A digits-only segment. The text is kept because `01` and `1` are
    /// numerically equal but not the same version, and Bazel breaks that
    /// tie on the text.
    Numeric {
        value: u64,
        text: String,
    },
    Text(String),
}

impl Ord for Identifier {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (
                Identifier::Numeric { value: a, text: at },
                Identifier::Numeric { value: b, text: bt },
            ) => a.cmp(b).then_with(|| at.cmp(bt)),
            // Bazel's comparator sorts digits-only first, then falls
            // through to `asString` — but `asNumber` is 0 for text
            // identifiers, so the numeric step never separates them.
            (Identifier::Numeric { .. }, Identifier::Text(_)) => Ordering::Less,
            (Identifier::Text(_), Identifier::Numeric { .. }) => Ordering::Greater,
            (Identifier::Text(a), Identifier::Text(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Identifier {
    fn parse(s: &str) -> Result<Identifier, VersionParseError> {
        if s.is_empty() {
            return Err(VersionParseError::EmptyIdentifier);
        }
        if s.bytes().all(|b| b.is_ascii_digit()) {
            let value = s
                .parse::<u64>()
                .map_err(|_| VersionParseError::SegmentTooLarge(s.to_owned()))?;
            Ok(Identifier::Numeric {
                value,
                text: s.to_owned(),
            })
        } else {
            Ok(Identifier::Text(s.to_owned()))
        }
    }
}

/// Bazel's version regex. The build part is matched so that it is accepted
/// but is deliberately not captured: it is discarded, not stored.
static VERSION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?<release>[a-zA-Z0-9.]+)(?:-(?<prerelease>[a-zA-Z0-9.\-]+))?(?:\+[a-zA-Z0-9.\-]+)?$",
    )
    .expect("version regex")
});

/// Why a version string could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VersionParseError {
    #[error("bad version (does not match regex): {0}")]
    Malformed(String),
    #[error("identifier is empty")]
    EmptyIdentifier,
    #[error("numeric version segment is too large: {0}")]
    SegmentTooLarge(String),
}

impl Version {
    /// The empty version: the sentinel for a module under a non-registry
    /// override, and the top of the ordering.
    pub const EMPTY: Version = Version {
        release: Vec::new(),
        prerelease: Vec::new(),
        normalized: String::new(),
    };

    /// Parses a version string, dropping any build metadata.
    pub fn parse(version: &str) -> Result<Version, VersionParseError> {
        if version.is_empty() {
            return Ok(Version::EMPTY);
        }
        let caps = VERSION_RE
            .captures(version)
            .ok_or_else(|| VersionParseError::Malformed(version.to_owned()))?;
        let release_str = caps.name("release").expect("release group").as_str();
        let prerelease_str = caps.name("prerelease").map(|m| m.as_str()).unwrap_or("");

        let release = release_str
            .split('.')
            .map(Identifier::parse)
            .collect::<Result<Vec<_>, _>>()?;
        let prerelease = if prerelease_str.is_empty() {
            Vec::new()
        } else {
            prerelease_str
                .split('.')
                .map(Identifier::parse)
                .collect::<Result<Vec<_>, _>>()?
        };

        let normalized = if prerelease_str.is_empty() {
            release_str.to_owned()
        } else {
            format!("{release_str}-{prerelease_str}")
        };
        Ok(Version {
            release,
            prerelease,
            normalized,
        })
    }

    /// Whether this is the empty version, i.e. the module has a
    /// non-registry override.
    pub fn is_empty(&self) -> bool {
        self.normalized.is_empty()
    }

    /// Whether the version carries a prerelease part, which sorts it below
    /// the same release without one.
    pub fn is_prerelease(&self) -> bool {
        !self.prerelease.is_empty()
    }

    /// The version string with build metadata stripped — the form Bazel
    /// puts in URLs, repo names and lockfiles.
    pub fn as_str(&self) -> &str {
        &self.normalized
    }
}

/// The empty version — the same default Bazel gives a module file with no
/// `module()` call, and the sentinel for a non-registry override.
impl Default for Version {
    fn default() -> Version {
        Version::EMPTY
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.normalized == other.normalized
    }
}

impl Eq for Version {}

impl std::hash::Hash for Version {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.normalized.hash(state);
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        // Empty last; then release; then a prerelease below its release;
        // then the prerelease identifiers. `Vec::cmp` is already the
        // lexicographical order Bazel uses (a prefix sorts below the
        // longer list).
        self.is_empty()
            .cmp(&other.is_empty())
            .then_with(|| self.release.cmp(&other.release))
            .then_with(|| other.is_prerelease().cmp(&self.is_prerelease()))
            .then_with(|| self.prerelease.cmp(&other.prerelease))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.normalized)
    }
}

impl serde::Serialize for Version {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.normalized)
    }
}

impl<'de> serde::Deserialize<'de> for Version {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Version, D::Error> {
        let s = String::deserialize(deserializer)?;
        Version::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    #[test]
    fn parses_relaxed_semver() {
        assert_eq!(v("1.2.3").as_str(), "1.2.3");
        // Any number of release segments, not just three.
        assert_eq!(v("1").as_str(), "1");
        assert_eq!(v("1.2.3.4.5").as_str(), "1.2.3.4.5");
        // Letters are allowed in the release part.
        assert_eq!(v("1.2.3b").as_str(), "1.2.3b");
        assert_eq!(v("hello").as_str(), "hello");
        assert_eq!(v("1.0-pre.1").as_str(), "1.0-pre.1");
        // Prerelease identifiers may contain hyphens; the release may not,
        // so the first hyphen always starts the prerelease.
        assert_eq!(v("1.0-alpha-1").as_str(), "1.0-alpha-1");
    }

    #[test]
    fn drops_build_metadata() {
        assert_eq!(v("1.2.3+build4"), v("1.2.3"));
        assert_eq!(v("1.2.3+build4").as_str(), "1.2.3");
        assert_eq!(v("1.2.3-pre+build.4").as_str(), "1.2.3-pre");
    }

    #[test]
    fn rejects_malformed_versions() {
        for bad in [
            "",
            "-1.2.3",
            "1.2.3-",
            "1.2.3+",
            "1_2_3",
            "1.2.3-pre_1",
            "v1.2.3+",
        ] {
            if bad.is_empty() {
                continue;
            }
            assert!(
                Version::parse(bad).is_err(),
                "expected {bad:?} to be rejected"
            );
        }
        // A dot with nothing between: matches the regex, fails on the
        // empty identifier.
        assert_eq!(
            Version::parse("1..2"),
            Err(VersionParseError::EmptyIdentifier)
        );
        // u64 is the bound, as in Bazel's parseUnsignedLong.
        assert!(Version::parse("18446744073709551615").is_ok());
        assert!(matches!(
            Version::parse("18446744073709551616"),
            Err(VersionParseError::SegmentTooLarge(_))
        ));
    }

    #[test]
    fn numeric_segments_compare_numerically() {
        assert!(v("1.9") < v("1.10"));
        assert!(v("1.2") < v("1.10"));
        // Not string order: "10" would sort below "9" as text.
        assert!(v("9.0") < v("10.0"));
    }

    #[test]
    fn digits_sort_below_text() {
        assert!(v("1.9") < v("1.alpha"));
        assert!(v("1.99999") < v("1.a"));
    }

    #[test]
    fn shorter_release_sorts_first() {
        assert!(v("1.2") < v("1.2.0"));
        assert!(v("1") < v("1.0"));
    }

    #[test]
    fn prerelease_sorts_below_its_release() {
        assert!(v("1.0-pre") < v("1.0"));
        assert!(v("1.0-pre") < v("1.0-pre.1"));
        assert!(v("1.0-alpha") < v("1.0-beta"));
        assert!(v("1.0-1") < v("1.0-alpha"));
    }

    #[test]
    fn empty_version_compares_highest() {
        assert!(Version::EMPTY > v("999999.0.0"));
        assert!(Version::EMPTY > v("zzz"));
        assert!(Version::EMPTY.is_empty());
        assert_eq!(Version::EMPTY, Version::parse("").unwrap());
    }

    #[test]
    fn ordering_is_consistent_with_equality() {
        // Bazel guarantees a.compareTo(b) == 0 iff a.equals(b); that is
        // why build metadata is dropped rather than stored.
        let a = v("1.2.3+x");
        let b = v("1.2.3+y");
        assert_eq!(a, b);
        assert_eq!(a.cmp(&b), Ordering::Equal);
        // Numerically equal but textually distinct segments are neither.
        let c = v("1.01");
        let d = v("1.1");
        assert_ne!(c, d);
        assert_eq!(c.cmp(&d), Ordering::Less);
    }

    #[test]
    fn semver_examples_from_the_spec() {
        // https://semver.org/#spec-item-11, which Bazel's format is a
        // superset of.
        let ordered = [
            "1.0.0-alpha",
            "1.0.0-alpha.1",
            "1.0.0-alpha.beta",
            "1.0.0-beta",
            "1.0.0-beta.2",
            "1.0.0-beta.11",
            "1.0.0-rc.1",
            "1.0.0",
        ];
        for pair in ordered.windows(2) {
            assert!(v(pair[0]) < v(pair[1]), "{} < {}", pair[0], pair[1]);
        }
    }

    #[test]
    fn round_trips_through_serde() {
        let json = serde_json::to_string(&v("1.2.3-pre")).unwrap();
        assert_eq!(json, "\"1.2.3-pre\"");
        assert_eq!(
            serde_json::from_str::<Version>("\"1.2.3+meta\"").unwrap(),
            v("1.2.3")
        );
    }
}
