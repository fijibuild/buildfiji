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
        "invalid package name '{}': {source}{}",
        shown(package),
        package_hint(suggestion)
    )]
    Package {
        package: String,
        source: LabelError,
        /// For a label written with no `:`, the last `/`-segment of its
        /// package, which is most likely what was meant as the target.
        suggestion: Option<String>,
    },
    #[error("invalid target name '{}': {source}", shown(name))]
    Target { name: String, source: LabelError },
    #[error("invalid repository name '{repo}': {source}")]
    Repo { repo: String, source: LabelError },
}

/// A name as Bazel quotes it in an error: a carriage return as `\r`, any
/// other control character as `<?>`.
fn shown(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '\r' => "\\r".to_owned(),
            c if (c as u32) < 0x20 || c == '\u{7f}' => "<?>".to_owned(),
            c => c.to_string(),
        })
        .collect()
}

/// A package name that is wrong in a label with no `:` is probably a target
/// name that lost its colon, so Bazel suggests where the colon might go.
fn package_hint(suggestion: &Option<String>) -> String {
    match suggestion {
        Some(target) => format!(" (perhaps you meant \":{target}\"?)"),
        None => String::new(),
    }
}

/// Splits `@r//p:q` into the repo as written (`None` when there is none),
/// whether it is apparent (`@r`, not `@@r`), the rest from the `//`, and
/// whether the input was only a repo (`@r`).
pub(crate) fn split_repo(input: &str) -> (Option<&str>, bool, &str, bool) {
    match input.strip_prefix('@') {
        Some(after) => {
            let (after, apparent) = match after.strip_prefix('@') {
                Some(canonical) => (canonical, false),
                None => (after, true),
            };
            match after.find("//") {
                Some(i) => (Some(&after[..i]), apparent, &after[i..], false),
                None => (Some(after), apparent, "", true),
            }
        }
        None => (None, false, input, false),
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
    /// [`Label::parse_mapped`] is the form that maps an apparent one.
    pub fn parse(input: &str, ctx: LabelContext<'_>) -> Result<Label, LabelParseError> {
        Label::parse_mapped(input, ctx, &mut |apparent| apparent.to_owned())
    }

    /// [`Label::parse`], with a repo written `@r` taken as an *apparent*
    /// name and turned into a canonical one by `map_repo` (repository
    /// mapping, buildfiji-mum.15). `@@r` is always canonical, and a label
    /// that names no repo stays in `ctx.repo`. `map_repo` sees the name as
    /// written, after it has been checked, and what it returns is not
    /// checked again: it may say a repo is unknown by returning text no
    /// repo could be called.
    pub fn parse_mapped(
        input: &str,
        ctx: LabelContext<'_>,
        map_repo: &mut dyn FnMut(&str) -> String,
    ) -> Result<Label, LabelParseError> {
        let (repo, apparent, rest, repo_only) = split_repo(input);
        if let Some(repo) = repo {
            label::validate_repo_name(repo).map_err(|source| LabelParseError::Repo {
                repo: repo.to_owned(),
                source,
            })?;
        }
        let written = repo.unwrap_or(ctx.repo);
        let repo = if apparent {
            map_repo(written)
        } else {
            written.to_owned()
        };

        let mut colonless_package = None;
        // A label with a `:` that is not `//p:x`, `@r//p:x` or `:x` is not
        // absolute, but Bazel finds what else is wrong with it first.
        let mut not_absolute = false;
        // What a `...` would make a package wildcard.
        let mut wildcard = None;
        let (package, name) = if repo_only {
            // `@r` is shorthand for `@r//:r`, with the name as written.
            ("", written)
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
        } else if let Some((package, name)) = rest.split_once(':') {
            not_absolute = true;
            (package, name)
        } else {
            // A bare `...` or `a/...` is refused as a package, though it
            // would be a target here.
            wildcard = Some(rest);
            (ctx.package, rest)
        };

        let wildcard = wildcard.unwrap_or(package);
        if wildcard == "..." || wildcard.ends_with("/...") {
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
        if not_absolute {
            return Err(LabelParseError::NotAbsolute(input.to_owned()));
        }
        // `b/.` is `b`: Bazel accepts the trailing dot segment and drops it.
        let name = name.strip_suffix("/.").unwrap_or(name);
        Ok(Label {
            repo,
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
            // A trailing dot segment is dropped, a lone `.` is a name, and
            // `...` is a package wildcard only as a package.
            ("//a:b/.", label("", "a", "b")),
            ("//a:b/c/.", label("", "a", "b/c")),
            ("n/.", label("", "a/b", "n")),
            (".", label("", "a/b", ".")),
            (":...", label("", "a/b", "...")),
            ("x...", label("", "a/b", "x...")),
            ("//a:b/...", label("", "a", "b/...")),
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

    /// `@r` is apparent and goes through the mapping; `@@r` and a label
    /// with no repo do not.
    #[test]
    fn an_apparent_repo_is_mapped_and_a_canonical_one_is_not() {
        let ctx = LabelContext {
            repo: "here+",
            package: "x",
        };
        let mut asked = Vec::new();
        let mut parse = |s: &str| {
            Label::parse_mapped(s, ctx, &mut |apparent| {
                asked.push(apparent.to_owned());
                format!("{apparent}+")
            })
            .unwrap()
        };
        assert_eq!(parse("@r//p:q"), label("r+", "p", "q"));
        assert_eq!(parse("@@r//p:q"), label("r", "p", "q"));
        assert_eq!(parse("//p:q"), label("here+", "p", "q"));
        assert_eq!(parse(":q"), label("here+", "x", "q"));
        assert_eq!(parse("@//p:q"), label("+", "p", "q"));
        assert_eq!(parse("@@//p:q"), label("", "p", "q"));
        // `@r` alone keeps the name as written.
        assert_eq!(parse("@r"), label("r+", "", "r"));
        assert_eq!(parse("@@r"), label("r", "", "r"));
        assert_eq!(asked, ["r", "", "r"]);
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
            // What else is wrong is found before the label is called
            // relative.
            (
                "a:b:c",
                "invalid target name 'b:c': target names may not contain ':'",
            ),
            (
                " //a:b",
                "invalid package name ' //a': package names may not contain '//' path separators",
            ),
            (
                "a/../b:c",
                "invalid package name 'a/../b': package name component contains only '.' \
                 characters",
            ),
            (
                "...",
                "invalid label '...': package name cannot contain '...'",
            ),
            (
                "a/...",
                "invalid label 'a/...': package name cannot contain '...'",
            ),
            // Control characters are quoted `<?>`, except a carriage return.
            (
                "c\td",
                "invalid target name 'c<?>d': target names may not contain non-printable \
                 characters: '\\x09'",
            ),
            (
                "c\nd",
                "invalid target name 'c<?>d': target names may not contain non-printable \
                 characters: '\\x0A'",
            ),
            (
                "c\rd",
                "invalid target name 'c\\rd': target names may not contain non-printable \
                 characters: '\\x0D'",
            ),
            (
                "//p:c\r",
                "invalid target name 'c\\r': target names may not end with carriage returns \
                 (perhaps the input source is CRLF-terminated)",
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
