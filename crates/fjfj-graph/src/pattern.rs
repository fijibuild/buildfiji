//! Target patterns as `bazel build` takes them (buildfiji-gwl.3).
//!
//! Every spelling here, and the wording of each error, was checked against
//! Bazel 9.2.0 with `bazel build --nobuild -- <pattern>` and `bazel query`.
//! Parsing is pure: a pattern that names no `//`, no `:` and no repo may be a
//! package directory or a file, which only the filesystem can tell, so it
//! parses to [`Pattern::Path`] and the loading phase decides.

use crate::Label;
use crate::label;
use crate::parse::{LabelContext, LabelParseError, split_repo};

/// Where a pattern is written: the repo it defaults to and the directory,
/// relative to that repo's root, that a relative pattern is relative to (empty
/// at the root).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatternContext<'a> {
    pub repo: &'a str,
    pub offset: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PatternError {
    #[error(transparent)]
    Label(#[from] LabelParseError),
    #[error("Invalid target pattern {0}: '...' can only be used with wildcard targets")]
    WildcardTarget(String),
}

/// What a pattern selects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pattern {
    /// One target: `//p:t`, `//p` (which is `//p:p`), `p:t`, `@r`.
    Target(Label),
    /// A package's targets: `//p:all` is its rules, `//p:*` and
    /// `//p:all-targets` its files too.
    InPackage {
        repo: String,
        package: String,
        rules_only: bool,
    },
    /// Every package at or below a directory: `//d/...`, which is rules only
    /// like `:all`, or `//d/...:*` for files too.
    Below {
        repo: String,
        directory: String,
        rules_only: bool,
    },
    /// A relative pattern with no `:`: the package directory it names, or the
    /// file in the package that encloses it. `path` is from the repo root.
    Path { repo: String, path: String },
}

/// A pattern as given on the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetPattern {
    /// The argument as given, dash included, for messages.
    pub text: String,
    /// Written with a leading `-`: remove what it selects.
    pub negative: bool,
    /// The repo as written, with its `@` or `@@`, if the pattern names one.
    pub repo_written: Option<String>,
    pub pattern: Pattern,
}

const WILDCARD_TARGETS: [&str; 4] = ["", "all", "*", "all-targets"];

fn join(offset: &str, rest: &str) -> String {
    match (offset.is_empty(), rest.is_empty()) {
        (true, _) => rest.to_owned(),
        (_, true) => offset.to_owned(),
        _ => format!("{offset}/{rest}"),
    }
}

fn package_error(package: &str, source: label::LabelError) -> LabelParseError {
    LabelParseError::Package {
        package: package.to_owned(),
        source,
        suggestion: None,
    }
}

impl Pattern {
    pub fn repo(&self) -> &str {
        match self {
            Pattern::Target(l) => &l.repo,
            Pattern::InPackage { repo, .. }
            | Pattern::Below { repo, .. }
            | Pattern::Path { repo, .. } => repo,
        }
    }

    /// The package, or for the forms that name a directory, the directory.
    pub fn package(&self) -> &str {
        match self {
            Pattern::Target(l) => &l.package,
            Pattern::InPackage { package, .. } => package,
            Pattern::Below { directory, .. } => directory,
            Pattern::Path { path, .. } => path,
        }
    }
}

impl TargetPattern {
    /// Parse one command-line pattern. `map_repo` turns an apparent repo
    /// written `@r` into a canonical name; `@@r` and a pattern with no repo
    /// are not mapped.
    pub fn parse(
        input: &str,
        ctx: PatternContext<'_>,
        map_repo: &mut dyn FnMut(&str) -> String,
    ) -> Result<TargetPattern, PatternError> {
        let (negative, text) = match input.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, input),
        };
        let lctx = LabelContext {
            repo: ctx.repo,
            package: ctx.offset,
        };
        let (written, apparent, rest, repo_only) = split_repo(text);
        let repo_written = written.map(|_| text[..text.len() - rest.len()].to_owned());
        let wrap = |pattern| TargetPattern {
            text: input.to_owned(),
            negative,
            repo_written: repo_written.clone(),
            pattern,
        };
        if repo_only {
            let label = crate::Label::parse_mapped(text, lctx, map_repo)?;
            return Ok(wrap(Pattern::Target(label)));
        }
        let repo = match written {
            Some(r) => {
                label::validate_repo_name(r).map_err(|source| LabelParseError::Repo {
                    repo: r.to_owned(),
                    source,
                })?;
                if apparent { map_repo(r) } else { r.to_owned() }
            }
            None => ctx.repo.to_owned(),
        };
        let (absolute, body) = match rest.strip_prefix("//") {
            Some(body) => (true, body),
            None => (false, rest),
        };
        let offset = if absolute { "" } else { ctx.offset };
        let (package_text, target) = match body.split_once(':') {
            Some((p, t)) => (p, Some(t)),
            None => (body, None),
        };

        if package_text == "..." || package_text.ends_with("/...") {
            let dir = package_text
                .strip_suffix("...")
                .map(|d| d.strip_suffix('/').unwrap_or(d))
                .unwrap_or(package_text);
            let wildcard = target.unwrap_or("");
            if !WILDCARD_TARGETS.contains(&wildcard) {
                return Err(PatternError::WildcardTarget(text.to_owned()));
            }
            label::validate_package_name(dir).map_err(|e| package_error(dir, e))?;
            return Ok(wrap(Pattern::Below {
                repo,
                directory: join(offset, dir),
                rules_only: matches!(wildcard, "" | "all"),
            }));
        }

        match target {
            Some(t @ ("all" | "*" | "all-targets")) => {
                label::validate_package_name(package_text)
                    .map_err(|e| package_error(package_text, e))?;
                Ok(wrap(Pattern::InPackage {
                    repo,
                    package: join(offset, package_text),
                    rules_only: t == "all",
                }))
            }
            Some(name) if !absolute => {
                label::validate_package_name(package_text)
                    .map_err(|e| package_error(package_text, e))?;
                label::validate_target_name(name).map_err(|source| LabelParseError::Target {
                    name: name.to_owned(),
                    source,
                })?;
                let name = name.strip_suffix("/.").unwrap_or(name);
                Ok(wrap(Pattern::Target(Label {
                    repo,
                    package: join(offset, package_text),
                    name: name.to_owned(),
                })))
            }
            None if !absolute => {
                label::validate_target_name(text).map_err(|source| LabelParseError::Target {
                    name: text.to_owned(),
                    source,
                })?;
                let path = if text == "." {
                    ""
                } else {
                    text.strip_suffix("/.").unwrap_or(text)
                };
                Ok(wrap(Pattern::Path {
                    repo,
                    path: join(offset, path),
                }))
            }
            _ => {
                let label = crate::Label::parse_mapped(text, lctx, map_repo)?;
                Ok(wrap(Pattern::Target(label)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_in(offset: &str, input: &str) -> Result<TargetPattern, PatternError> {
        let ctx = PatternContext { repo: "", offset };
        // `@` alone is the main repo; any other `@r` is a module's `r+`.
        TargetPattern::parse(input, ctx, &mut |apparent| match apparent {
            "" => String::new(),
            r => format!("{r}+"),
        })
    }

    fn target(repo: &str, package: &str, name: &str) -> Pattern {
        Pattern::Target(Label {
            repo: repo.into(),
            package: package.into(),
            name: name.into(),
        })
    }

    fn in_package(package: &str, rules_only: bool) -> Pattern {
        Pattern::InPackage {
            repo: String::new(),
            package: package.into(),
            rules_only,
        }
    }

    fn below(directory: &str, rules_only: bool) -> Pattern {
        Pattern::Below {
            repo: String::new(),
            directory: directory.into(),
            rules_only,
        }
    }

    fn path(path: &str) -> Pattern {
        Pattern::Path {
            repo: String::new(),
            path: path.into(),
        }
    }

    /// Each row is a spelling, the directory it is written in, and what
    /// Bazel 9.2.0 resolved it to in a workspace with packages `//`, `//a`,
    /// `//a/b` and `//c`.
    #[test]
    fn spellings_bazel_accepts() {
        for (offset, input, want) in [
            ("", "//a", target("", "a", "a")),
            ("", "//a:a", target("", "a", "a")),
            ("", "//a:x.txt", target("", "a", "x.txt")),
            ("", "//:f.txt", target("", "", "f.txt")),
            ("", "//a/b:y.txt", target("", "a/b", "y.txt")),
            ("", "//a:all", in_package("a", true)),
            ("", "//a:*", in_package("a", false)),
            ("", "//a:all-targets", in_package("a", false)),
            ("", "//:all", in_package("", true)),
            ("", "//a/...", below("a", true)),
            ("", "//a/...:all", below("a", true)),
            ("", "//a/...:", below("a", true)),
            ("", "//a/...:*", below("a", false)),
            ("", "//a/...:all-targets", below("a", false)),
            ("", "//...", below("", true)),
            ("", "//...:*", below("", false)),
            ("", "...", below("", true)),
            ("", ":all", in_package("", true)),
            ("", "a:all", in_package("a", true)),
            ("", "a/...", below("a", true)),
            ("", "a", path("a")),
            ("", "a/b", path("a/b")),
            ("", "x.txt", path("x.txt")),
            ("", "a/b:y.txt", target("", "a/b", "y.txt")),
            ("", "@m", target("m+", "", "m")),
            ("", "@m//a", target("m+", "a", "a")),
            ("", "@@m+//a", target("m+", "a", "a")),
            ("", "@@//a:all", in_package("a", true)),
            ("", "@//a", target("", "a", "a")),
            ("a", "//a", target("", "a", "a")),
            ("a", ":a", target("", "a", "a")),
            ("a", ":x.txt", target("", "a", "x.txt")),
            ("a", ":all", in_package("a", true)),
            ("a", ":*", in_package("a", false)),
            ("a", "b", path("a/b")),
            ("a", "a", path("a/a")),
            ("a", "x.txt", path("a/x.txt")),
            ("a", "...", below("a", true)),
            ("a", "b/...", below("a/b", true)),
            ("a", "b:b", target("", "a/b", "b")),
            ("a", "b:y.txt", target("", "a/b", "y.txt")),
            ("a", ".", path("a")),
            ("a", "//...", below("", true)),
        ] {
            let got = parse_in(offset, input).unwrap_or_else(|e| panic!("{offset}: {input}: {e}"));
            assert_eq!(got.pattern, want, "in {offset:?}: {input}");
            assert!(!got.negative, "{input}");
        }
    }

    #[test]
    fn a_leading_dash_makes_it_negative_and_the_repo_is_kept_as_written() {
        let got = parse_in("", "-@m//a:b").unwrap();
        assert!(got.negative);
        assert_eq!(got.repo_written.as_deref(), Some("@m"));
        assert_eq!(got.pattern, target("m+", "a", "b"));
        assert_eq!(
            parse_in("", "@@x+//a").unwrap().repo_written.as_deref(),
            Some("@@x+")
        );
        assert_eq!(
            parse_in("", "@//a").unwrap().repo_written.as_deref(),
            Some("@")
        );
        assert_eq!(parse_in("", "//a").unwrap().repo_written, None);
    }

    /// Each row is a spelling, where it is written, and what Bazel 9.2.0
    /// printed after `ERROR: Skipping '<pattern>': `.
    #[test]
    fn spellings_bazel_rejects_and_how_it_says_so() {
        for (offset, input, want) in [
            ("", "", "invalid target name '': empty target name"),
            ("", "-", "invalid target name '': empty target name"),
            ("", "//", "invalid target name '': empty target name"),
            ("", "@", "invalid target name '': empty target name"),
            ("", "@m//", "invalid target name '': empty target name"),
            ("", "//a:", "invalid target name '': empty target name"),
            (
                "",
                "/a",
                "invalid target name '/a': target names may not start with '/'",
            ),
            (
                "",
                "//a:b:c",
                "invalid target name 'b:c': target names may not contain ':'",
            ),
            (
                "",
                "//a:a:",
                "invalid target name 'a:': target names may not contain ':'",
            ),
            (
                "",
                "//a:*:*",
                "invalid target name '*:*': target names may not contain ':'",
            ),
            (
                "",
                "//a:a/",
                "invalid target name 'a/': target names may not end with '/'",
            ),
            (
                "",
                "//a:../x",
                "invalid target name '../x': target names may not contain up-level references '..'",
            ),
            (
                "a",
                "../a",
                "invalid target name '../a': target names may not contain up-level references '..'",
            ),
            (
                "a",
                "..",
                "invalid target name '..': target names may not contain up-level references '..'",
            ),
            (
                "a",
                "./b",
                "invalid target name './b': target names may not contain '.' as a path segment",
            ),
            (
                "",
                " //a",
                "invalid target name ' //a': target names may not contain '//' path separators",
            ),
            (
                "",
                "//a/",
                "invalid package name 'a/': package names may not end with '/' (perhaps you meant \":\"?)",
            ),
            (
                "",
                "//a/b/",
                "invalid package name 'a/b/': package names may not end with '/' (perhaps you meant \":\"?)",
            ),
            (
                "",
                "//a//b",
                "invalid package name 'a//b': package names may not contain '//' path separators (perhaps you meant \":b\"?)",
            ),
            (
                "",
                "//a/..",
                "invalid package name 'a/..': package name component contains only '.' characters (perhaps you meant \":..\"?)",
            ),
            (
                "",
                "//a/../b",
                "invalid package name 'a/../b': package name component contains only '.' characters (perhaps you meant \":b\"?)",
            ),
            (
                "",
                "//a/.../b",
                "invalid package name 'a/.../b': package name component contains only '.' characters (perhaps you meant \":b\"?)",
            ),
            (
                "",
                "//.../a",
                "invalid package name '.../a': package name component contains only '.' characters (perhaps you meant \":a\"?)",
            ),
            (
                "a",
                "../...",
                "invalid package name '..': package name component contains only '.' characters",
            ),
            (
                "a",
                "./...",
                "invalid package name '.': package name component contains only '.' characters",
            ),
            (
                "",
                "//a/...:b",
                "Invalid target pattern //a/...:b: '...' can only be used with wildcard targets",
            ),
        ] {
            let got = parse_in(offset, input).unwrap_err().to_string();
            assert_eq!(got, want, "in {offset:?}: {input}");
        }
    }

    #[test]
    fn a_dash_is_stripped_before_the_rest_is_checked() {
        // Bazel names the text after the dash, not the whole argument.
        assert_eq!(
            parse_in("", "- //a").unwrap_err().to_string(),
            "invalid target name ' //a': target names may not contain '//' path separators"
        );
    }

    #[test]
    fn whitespace_and_a_missing_all_are_part_of_the_target_name() {
        assert_eq!(
            parse_in("", "//a:all ").unwrap().pattern,
            target("", "a", "all ")
        );
        assert_eq!(
            parse_in("", "//a:...").unwrap().pattern,
            target("", "a", "...")
        );
        assert_eq!(parse_in("", "//a:A").unwrap().pattern, target("", "a", "A"));
        assert_eq!(
            parse_in("", "//a:all-targets-x").unwrap().pattern,
            target("", "a", "all-targets-x")
        );
    }
}
