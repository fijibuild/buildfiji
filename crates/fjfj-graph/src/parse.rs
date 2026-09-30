//! Parsing a label string as a BUILD file writes it, relative to the package
//! being loaded (buildfiji-mum.5).
//!
//! Every accepted and rejected spelling here was checked against Bazel 9.2.0,
//! including the wording of the error, since a BUILD file's label errors are
//! shown to the user verbatim.

use crate::Label;
use crate::label::{self, LabelError};

/// The package a label is written in: what `:x`, `x` and `//p` resolve
/// against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabelContext<'a> {
    /// Canonical repo name; empty for the main repository.
    pub repo: &'a str,
    pub package: &'a str,
}

/// Why a label string did not parse. `Display` is Bazel's own text, without
/// the "in element 0 of attribute 'srcs'" context the caller adds.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LabelParseError {
    #[error("invalid label '{0}': absolute label must begin with '@' or '//'")]
    NotAbsolute(String),
    #[error("invalid label '{0}': package name cannot contain '...'")]
    PackageWildcard(String),
    #[error(
        "invalid package name '{package}': {source}{}",
        package_hint(suggestion)
    )]
    Package {
        package: String,
        source: LabelError,
        /// For a label written with no `:`, the last `/`-segment of its
        /// package, which is most likely what was meant as the target.
        suggestion: Option<String>,
    },
    #[error("invalid target name '{name}': {source}")]
    Target { name: String, source: LabelError },
    #[error("invalid repository name '{repo}': {source}")]
    Repo { repo: String, source: LabelError },
}

/// A package name that is wrong in a label with no `:` is probably a target
/// name that lost its colon, so Bazel suggests where the colon might go.
fn package_hint(suggestion: &Option<String>) -> String {
    match suggestion {
        Some(target) => format!(" (perhaps you meant \":{target}\"?)"),
        None => String::new(),
    }
}

impl Label {
    /// Parse `input` the way a BUILD file spells a label, resolving it
    /// against `ctx`:
    ///
    /// - `:x` and `x` are in the current package;
    /// - `//p/q:r` is in this repo's package `p/q`, and `//p/q` is `//p/q:q`;
    /// - `@r//p:q` and `@@r//p:q` name a repo, `@//p:q` the main one, and
    ///   `@r` alone is `@r//:r`.
    ///
    /// Whitespace is not trimmed. `@r` is taken as a canonical repo name;
    /// mapping an apparent name is the repository-mapping bead's job
    /// (buildfiji-mum.15).
    pub fn parse(input: &str, ctx: LabelContext<'_>) -> Result<Label, LabelParseError> {
        let (repo, rest, repo_only) = match input.strip_prefix('@') {
            Some(after) => {
                let after = after.strip_prefix('@').unwrap_or(after);
                match after.find("//") {
                    Some(i) => (Some(&after[..i]), &after[i..], false),
                    None => (Some(after), "", true),
                }
            }
            None => (None, input, false),
        };
        if let Some(repo) = repo {
            label::validate_repo_name(repo).map_err(|source| LabelParseError::Repo {
                repo: repo.to_owned(),
                source,
            })?;
        }
        let repo = repo.unwrap_or(ctx.repo);

        let mut colonless_package = None;
        let (package, name) = if repo_only {
            // `@r` is shorthand for `@r//:r`.
            ("", repo)
        } else if let Some(absolute) = rest.strip_prefix("//") {
            match absolute.split_once(':') {
                Some((package, name)) => (package, name),
                // `//p/q` is `//p/q:q`.
                None => {
                    let last = absolute.rsplit('/').next().unwrap_or("");
                    colonless_package = Some(last);
                    (absolute, last)
                }
            }
        } else if let Some(name) = rest.strip_prefix(':') {
            (ctx.package, name)
        } else if rest.contains(':') {
            return Err(LabelParseError::NotAbsolute(input.to_owned()));
        } else {
            (ctx.package, rest)
        };

        if package == "..." || package.ends_with("/...") {
            return Err(LabelParseError::PackageWildcard(input.to_owned()));
        }
        label::validate_package_name(package).map_err(|source| LabelParseError::Package {
            package: package.to_owned(),
            source,
            suggestion: colonless_package.map(str::to_owned),
        })?;
        label::validate_target_name(name).map_err(|source| LabelParseError::Target {
            name: name.to_owned(),
            source,
        })?;
        Ok(Label {
            repo: repo.to_owned(),
            package: package.to_owned(),
            name: name.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CTX: LabelContext<'static> = LabelContext {
        repo: "",
        package: "a/b",
    };

    fn parse(s: &str) -> Label {
        Label::parse(s, CTX).unwrap_or_else(|e| panic!("{s}: {e}"))
    }

    fn label(repo: &str, package: &str, name: &str) -> Label {
        Label {
            repo: repo.into(),
            package: package.into(),
            name: name.into(),
        }
    }

    fn rejection(s: &str) -> String {
        Label::parse(s, CTX).unwrap_err().to_string()
    }

    /// Each row is a spelling and the label `bazel query 'labels(srcs, ..)'`
    /// reported for it on 9.2.0.
    #[test]
    fn spellings_bazel_accepts() {
        for (input, want) in [
            (":n", label("", "a/b", "n")),
            ("n", label("", "a/b", "n")),
            ("n/m", label("", "a/b", "n/m")),
            ("//p/q", label("", "p/q", "q")),
            ("//p/q:r", label("", "p/q", "r")),
            ("//:t", label("", "", "t")),
            ("//p:all", label("", "p", "all")),
            ("//p:*", label("", "p", "*")),
            ("//p:a b", label("", "p", "a b")),
            ("@r", label("r", "", "r")),
            ("@r//p", label("r", "p", "p")),
            ("@r//p:q", label("r", "p", "q")),
            ("@@r//p:q", label("r", "p", "q")),
            ("@r//:t", label("r", "", "t")),
            ("@//p:q", label("", "p", "q")),
            ("@@//p:q", label("", "p", "q")),
            // Whitespace is part of the name.
            (" n", label("", "a/b", " n")),
            ("n ", label("", "a/b", "n ")),
        ] {
            assert_eq!(parse(input), want, "{input}");
        }
    }

    #[test]
    fn a_label_in_another_repo_context_resolves_there() {
        let ctx = LabelContext {
            repo: "dep+",
            package: "x",
        };
        assert_eq!(Label::parse("//p:q", ctx).unwrap(), label("dep+", "p", "q"));
        assert_eq!(Label::parse(":q", ctx).unwrap(), label("dep+", "x", "q"));
        assert_eq!(Label::parse("@//p:q", ctx).unwrap(), label("", "p", "q"));
    }

    /// Each row is a spelling and the message Bazel 9.2.0 gave for it.
    #[test]
    fn spellings_bazel_rejects_with_its_wording() {
        for (input, want) in [
            (
                "p:q",
                "invalid label 'p:q': absolute label must begin with '@' or '//'",
            ),
            (
                "a:b/c",
                "invalid label 'a:b/c': absolute label must begin with '@' or '//'",
            ),
            (
                "//p/q/",
                "invalid package name 'p/q/': package names may not end with '/' \
                 (perhaps you meant \":\"?)",
            ),
            (
                "//p/q/:x",
                "invalid package name 'p/q/': package names may not end with '/'",
            ),
            (
                "//a//b",
                "invalid package name 'a//b': package names may not contain '//' path separators \
                 (perhaps you meant \":b\"?)",
            ),
            (
                "//a///b",
                "invalid package name 'a///b': package names may not contain '//' path separators \
                 (perhaps you meant \":b\"?)",
            ),
            (
                "//a//b//c",
                "invalid package name 'a//b//c': package names may not contain '//' path \
                 separators (perhaps you meant \":c\"?)",
            ),
            (
                "//a/b//",
                "invalid package name 'a/b//': package names may not end with '/' \
                 (perhaps you meant \":\"?)",
            ),
            (
                "//a/./b",
                "invalid package name 'a/./b': package name component contains only '.' \
                 characters (perhaps you meant \":b\"?)",
            ),
            (
                "//a/./b:x",
                "invalid package name 'a/./b': package name component contains only '.' characters",
            ),
            (
                "//a/b/.",
                "invalid package name 'a/b/.': package name component contains only '.' \
                 characters (perhaps you meant \":.\"?)",
            ),
            ("//p:", "invalid target name '': empty target name"),
            ("//", "invalid target name '': empty target name"),
            ("@r//", "invalid target name '': empty target name"),
            (":", "invalid target name '': empty target name"),
            ("", "invalid target name '': empty target name"),
            (
                "//p//q:r",
                "invalid package name 'p//q': package names may not contain '//' path separators",
            ),
            (
                "//p:q/",
                "invalid target name 'q/': target names may not end with '/'",
            ),
            (
                "//p:q//r",
                "invalid target name 'q//r': target names may not contain '//' path separators",
            ),
            (
                "//p/./q:r",
                "invalid package name 'p/./q': package name component contains only '.' characters",
            ),
            (
                "//p/../q:r",
                "invalid package name 'p/../q': package name component contains only '.' characters",
            ),
            (
                "//p:./q",
                "invalid target name './q': target names may not contain '.' as a path segment",
            ),
            (
                "//p:q/./r",
                "invalid target name 'q/./r': target names may not contain '.' as a path segment",
            ),
            (
                "/x",
                "invalid target name '/x': target names may not start with '/'",
            ),
            (
                "//p/q:r:s",
                "invalid target name 'r:s': target names may not contain ':'",
            ),
            (
                "@r//p/...",
                "invalid label '@r//p/...': package name cannot contain '...'",
            ),
            (
                "@r:n",
                "invalid repository name 'r:n': repo names may contain only A-Z, a-z, 0-9, \
                 '-', '_', '.' and '+'",
            ),
        ] {
            assert_eq!(rejection(input), want, "{input}");
        }
    }
}
