//! Package-name and target-name validation, matching Bazel's
//! `LabelValidator` exactly (`src/main/java/.../cmdline/LabelValidator.java`)
//! rather than a plausible-looking approximation — this is the one place
//! "is this a legal label" gets decided, and conformance diffs against
//! Bazel compare outputs, not our own idea of what should be legal.
//!
//! The two grammars are asymmetric on purpose, straight from Bazel's own
//! source: a **package** name is ASCII-only (the directory-name half of a
//! label has to round-trip through every filesystem and shell Bazel
//! supports); a **target** name additionally allows any non-ASCII
//! character at all — Bazel treats every code point above U+007F as
//! automatically valid, since it can't distinguish "meaningful Unicode
//! identifier character" from "arbitrary byte of an encoded code point"
//! without doing far more work than a label check warrants. That's what
//! "Unicode labels" (buildfiji-mum.18) is about: source *files* commonly
//! have non-ASCII names (e.g. localized test fixtures), and those are
//! legal Bazel target names today, not an edge case to reject.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LabelError {
    #[error("package names may not start with '/'")]
    PackageStartsWithSlash,
    #[error("package names may not end with '/'")]
    PackageEndsWithSlash,
    #[error("package names may not contain '//' path separators")]
    PackageDoubleSlash,
    #[error(
        "package names may contain A-Z, a-z, 0-9, or any of ' !\"#$%&'()*+,-./;<=>?[]^_`{{|}}~' \
         (any ASCII character except 0-31, 127, ':', or '\\')"
    )]
    PackageInvalidChar,
    #[error("package name component contains only '.' characters")]
    PackageDotOnlyComponent,
    #[error("empty target name")]
    TargetEmpty,
    #[error("target names may not start with '/'")]
    TargetStartsWithSlash,
    #[error("target names may not end with '/'")]
    TargetEndsWithSlash,
    #[error("target names may not contain up-level references '..'")]
    TargetUpLevelReference,
    #[error("target names may not contain '.' as a path segment")]
    TargetDotSegment,
    #[error("target names may not contain '//' path separators")]
    TargetDoubleSlash,
    #[error("target names may not end with carriage returns")]
    TargetTrailingCr,
    #[error("target names may not contain non-printable character {0:#04x}")]
    TargetNonPrintable(u8),
    #[error("target names may not contain {0:?}")]
    TargetInvalidChar(char),
    #[error(
        "invalid repository name '{0}': valid names may contain only A-Z, a-z, 0-9, '-', '_', \
         '.' and '+'"
    )]
    RepoInvalidChar(String),
    #[error(
        "invalid user-provided repo name '{0}': valid names may contain only A-Z, a-z, 0-9, '-', \
         '_', '.', and must start with a letter or a number"
    )]
    UserProvidedRepoName(String),
}

/// ASCII characters (besides letters and digits) a package name may
/// contain — Bazel's `ALLOWED_CHARACTERS_IN_PACKAGE_NAME`, punctuation
/// plus a literal space. `br##"..."##` (extra `#`) because the content
/// itself contains a `"#` sequence that would otherwise close the string.
const PACKAGE_PUNCTUATION: &[u8] = br##" !"#$%&'()*+,-./;<=>?@[]^_`{|}~"##;

/// ASCII punctuation always legal in a target name outside the `.`/`/`
/// special-casing below — Bazel's two `PUNCTUATION_*` matchers merged,
/// since fjfj doesn't need the blaze-query-quoting distinction between
/// them.
const TARGET_PUNCTUATION: &[u8] = br##" "#$&'()*+,;<=>?[]{|}~!%-@^_`"##;

/// Bazel's `validatePackageName`: ASCII-only, no leading/trailing `/`, no
/// `//`, and no path segment made up entirely of `.` characters.
pub fn validate_package_name(package: &str) -> Result<(), LabelError> {
    if package.is_empty() {
        return Ok(()); // the root package, `//:foo`
    }
    if package.starts_with('/') {
        return Err(LabelError::PackageStartsWithSlash);
    }
    if !package
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || PACKAGE_PUNCTUATION.contains(&b))
    {
        return Err(LabelError::PackageInvalidChar);
    }
    if package.ends_with('/') {
        return Err(LabelError::PackageEndsWithSlash);
    }
    for segment in package.split('/') {
        if segment.is_empty() {
            return Err(LabelError::PackageDoubleSlash);
        }
        if segment.bytes().all(|b| b == b'.') {
            return Err(LabelError::PackageDotOnlyComponent);
        }
    }
    Ok(())
}

/// Bazel's `validateTargetName`: any Unicode character is allowed except
/// ASCII control characters and a handful of punctuation marks reserved
/// for label syntax (`:`, `\`) or given special meaning (`.`, `/`).
pub fn validate_target_name(target: &str) -> Result<(), LabelError> {
    if target.is_empty() {
        return Err(LabelError::TargetEmpty);
    }
    if target == "." {
        return Ok(()); // Bazel special-cases this rather than rejecting it.
    }
    if target.starts_with('/') {
        return Err(LabelError::TargetStartsWithSlash);
    }
    if target == ".." || target.starts_with("../") {
        return Err(LabelError::TargetUpLevelReference);
    }
    if target.starts_with("./") {
        return Err(LabelError::TargetDotSegment);
    }
    if target.ends_with('\r') {
        return Err(LabelError::TargetTrailingCr);
    }

    for c in target.chars() {
        match c {
            '.' | '/' => continue, // segment structure checked separately below
            c if is_always_allowed_target_char(c) => continue,
            c if (c as u32) <= 0x1f || c == '\u{7f}' => {
                return Err(LabelError::TargetNonPrintable(c as u8));
            }
            c => return Err(LabelError::TargetInvalidChar(c)),
        }
    }
    for window in target.as_bytes().windows(2) {
        if window == b"//" {
            return Err(LabelError::TargetDoubleSlash);
        }
    }
    if target.contains("/../") || target.ends_with("/..") {
        return Err(LabelError::TargetUpLevelReference);
    }
    if target.contains("/./") {
        return Err(LabelError::TargetDotSegment);
    }
    if target.ends_with("/.") {
        return Ok(()); // Bazel special-cases this too.
    }
    if target.ends_with('/') {
        return Err(LabelError::TargetEndsWithSlash);
    }
    Ok(())
}

/// A letter, digit, target-name punctuation mark, or any non-ASCII
/// character — Bazel's `ALWAYS_ALLOWED_TARGET_CHARACTERS`.
fn is_always_allowed_target_char(c: char) -> bool {
    c.is_alphanumeric()
        || (c.is_ascii() && TARGET_PUNCTUATION.contains(&(c as u8)))
        || !c.is_ascii()
}

/// Bazel's `RepositoryName.VALID_REPO_NAME` (`[\w\-.+]*`): the character
/// set of a *canonical* repo name, which is wider than a user may write.
/// The `+` is there because bzlmod builds canonical names by joining a
/// module name and version with it (`rules_foo+`, `rules_foo+1.2.3`), and
/// the empty string is legal because that is the main repository.
pub fn validate_repo_name(repo: &str) -> Result<(), LabelError> {
    let ok = repo
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+'));
    if ok {
        Ok(())
    } else {
        Err(LabelError::RepoInvalidChar(repo.to_owned()))
    }
}

/// Bazel's `RepositoryName.validateUserProvidedRepoName`
/// (`[a-zA-Z0-9][-.\w]*`): the narrower set a user may write in
/// `repo_name = ...` on `module()`, `bazel_dep()` or `use_repo()`. Note
/// the two asymmetries with [`validate_repo_name`]: a user-provided name
/// may not be empty and may not contain `+`, which keeps user-written
/// names disjoint from generated canonical ones.
pub fn validate_user_provided_repo_name(repo: &str) -> Result<(), LabelError> {
    let mut chars = repo.chars();
    let first_ok = chars.next().is_some_and(|c| c.is_ascii_alphanumeric());
    let rest_ok = chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    if first_ok && rest_ok {
        Ok(())
    } else {
        Err(LabelError::UserProvidedRepoName(repo.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn canonical_repo_names_allow_plus_and_empty() {
        // The main repository, and the two shapes bzlmod generates.
        validate_repo_name("").unwrap();
        validate_repo_name("rules_foo+").unwrap();
        validate_repo_name("rules_foo+1.2.3").unwrap();
        validate_repo_name("rules_foo++ext+repo").unwrap();
        assert!(validate_repo_name("has space").is_err());
        assert!(validate_repo_name("has/slash").is_err());
        assert!(validate_repo_name("has@at").is_err());
    }

    #[test]
    fn user_provided_repo_names_are_narrower() {
        validate_user_provided_repo_name("foo").unwrap();
        validate_user_provided_repo_name("foo_bar-1.2").unwrap();
        validate_user_provided_repo_name("0abc").unwrap();
        // Empty, leading punctuation and '+' are all canonical-only.
        assert!(validate_user_provided_repo_name("").is_err());
        assert!(validate_user_provided_repo_name("_foo").is_err());
        assert!(validate_user_provided_repo_name("-foo").is_err());
        assert!(validate_user_provided_repo_name("rules_foo+").is_err());
    }

    use super::*;

    #[test]
    fn root_package_is_valid() {
        assert_eq!(validate_package_name(""), Ok(()));
    }

    #[test]
    fn ordinary_package_is_valid() {
        assert_eq!(validate_package_name("foo/bar"), Ok(()));
    }

    #[test]
    fn package_leading_slash_is_invalid() {
        assert_eq!(
            validate_package_name("/foo"),
            Err(LabelError::PackageStartsWithSlash)
        );
    }

    #[test]
    fn package_trailing_slash_is_invalid() {
        assert_eq!(
            validate_package_name("foo/"),
            Err(LabelError::PackageEndsWithSlash)
        );
    }

    #[test]
    fn package_double_slash_is_invalid() {
        assert_eq!(
            validate_package_name("foo//bar"),
            Err(LabelError::PackageDoubleSlash)
        );
    }

    #[test]
    fn package_dot_only_segment_is_invalid() {
        assert_eq!(
            validate_package_name("foo/./bar"),
            Err(LabelError::PackageDotOnlyComponent)
        );
        assert_eq!(
            validate_package_name("foo/../bar"),
            Err(LabelError::PackageDotOnlyComponent)
        );
    }

    #[test]
    fn package_non_ascii_is_invalid() {
        assert_eq!(
            validate_package_name("café"),
            Err(LabelError::PackageInvalidChar)
        );
    }

    #[test]
    fn package_colon_and_backslash_are_invalid() {
        assert_eq!(
            validate_package_name("foo:bar"),
            Err(LabelError::PackageInvalidChar)
        );
        assert_eq!(
            validate_package_name("foo\\bar"),
            Err(LabelError::PackageInvalidChar)
        );
    }

    #[test]
    fn target_non_ascii_is_valid() {
        // The point of buildfiji-mum.18: a source file named with
        // non-ASCII characters is a legal Bazel target name.
        assert_eq!(validate_target_name("café.txt"), Ok(()));
        assert_eq!(validate_target_name("测试.txt"), Ok(()));
    }

    #[test]
    fn target_empty_is_invalid() {
        assert_eq!(validate_target_name(""), Err(LabelError::TargetEmpty));
    }

    #[test]
    fn target_dot_alone_is_valid() {
        assert_eq!(validate_target_name("."), Ok(()));
    }

    #[test]
    fn target_leading_slash_is_invalid() {
        assert_eq!(
            validate_target_name("/foo"),
            Err(LabelError::TargetStartsWithSlash)
        );
    }

    #[test]
    fn target_trailing_slash_is_invalid() {
        assert_eq!(
            validate_target_name("foo/"),
            Err(LabelError::TargetEndsWithSlash)
        );
    }

    #[test]
    fn target_up_level_reference_is_invalid() {
        assert_eq!(
            validate_target_name(".."),
            Err(LabelError::TargetUpLevelReference)
        );
        assert_eq!(
            validate_target_name("../foo"),
            Err(LabelError::TargetUpLevelReference)
        );
        assert_eq!(
            validate_target_name("foo/../bar"),
            Err(LabelError::TargetUpLevelReference)
        );
        assert_eq!(
            validate_target_name("foo/.."),
            Err(LabelError::TargetUpLevelReference)
        );
    }

    #[test]
    fn target_dot_segment_is_invalid() {
        assert_eq!(
            validate_target_name("./foo"),
            Err(LabelError::TargetDotSegment)
        );
        assert_eq!(
            validate_target_name("foo/./bar"),
            Err(LabelError::TargetDotSegment)
        );
    }

    #[test]
    fn target_trailing_dot_segment_is_valid() {
        // Bazel special-cases this (data directories); see LabelValidator.
        assert_eq!(validate_target_name("foo/."), Ok(()));
    }

    #[test]
    fn target_double_slash_is_invalid() {
        assert_eq!(
            validate_target_name("foo//bar"),
            Err(LabelError::TargetDoubleSlash)
        );
    }

    #[test]
    fn target_trailing_cr_is_invalid() {
        assert_eq!(
            validate_target_name("foo\r"),
            Err(LabelError::TargetTrailingCr)
        );
    }

    #[test]
    fn target_control_char_is_invalid() {
        assert_eq!(
            validate_target_name("foo\x01bar"),
            Err(LabelError::TargetNonPrintable(0x01))
        );
    }

    #[test]
    fn target_colon_and_backslash_are_invalid() {
        assert_eq!(
            validate_target_name("foo:bar"),
            Err(LabelError::TargetInvalidChar(':'))
        );
        assert_eq!(
            validate_target_name("foo\\bar"),
            Err(LabelError::TargetInvalidChar('\\'))
        );
    }

    #[test]
    fn target_file_in_subdirectory_is_valid() {
        assert_eq!(validate_target_name("testdata/input.txt"), Ok(()));
    }
}
