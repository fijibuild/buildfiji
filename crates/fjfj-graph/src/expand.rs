//! Expanding `$(location x)` and `$(VARIABLE)` in a rule's command
//! (buildfiji-136.19), as Bazel 9.2.0 does it for `genrule`.
//!
//! Each behaviour was read off `bazel aquery` of a genrule:
//!
//! - `$$` is a `$`; `$@` and `$<` are the single output and the single input;
//!   `$(SRCS)`, `$(OUTS)`, `$(@D)`, `$(RULEDIR)`, `$(GENDIR)`, `$(BINDIR)`,
//!   `$(TARGET_CPU)`, `$(COMPILATION_MODE)` and the `--define`s are variables;
//! - `$(location x)`, `$(execpath x)`, `$(rootpath x)`, `$(rlocationpath x)`
//!   and their plural forms name a label that must be one of the rule's
//!   prerequisites. A path with no `/` in it gets `./` in front, so that a
//!   shell does not look for it on `PATH`;
//! - an error names the attribute and the rule:
//!   `in cmd attribute of genrule rule //:t: $(foo) not defined`.

use crate::{Artifact, Label, LabelContext};
use std::collections::BTreeMap;

/// A target the rule depends on and the files it gave.
#[derive(Debug, Clone)]
pub struct Prerequisite {
    pub label: Label,
    pub files: Vec<Artifact>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("in {attribute} attribute of {rule_class} rule {label}: {message}")]
pub struct ExpandError {
    pub attribute: String,
    pub rule_class: String,
    pub label: String,
    pub message: String,
}

/// `ctx.expand_make_variables`: `$(NAME)` from `lookup`, `$$` as `$`, and
/// `$@`, `$<` and `$^` as the variables of those names. Bazel 9.2.0's
/// reasons for refusing, in its words, without the `in <attribute> attribute
/// of <rule>` that precedes them:
///
/// - `$(name args)` is a function call, and no function is known here, so
///   its first word is reported as undefined;
/// - `$NAME` and `${NAME}` are not supported and say what to write instead;
///   a lone `$` at the end is unterminated, as is a `$(` with no `)`.
pub fn expand_make_variables(
    text: &str,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        if c != '$' {
            out.push(c);
            continue;
        }
        let Some((_, next)) = chars.next() else {
            return Err("unterminated $".to_owned());
        };
        match next {
            '$' => out.push('$'),
            '(' | '{' => {
                let close = if next == '(' { ')' } else { '}' };
                let start = at + 2;
                let Some(len) = text[start..].find(close) else {
                    return Err("unterminated variable reference".to_owned());
                };
                let name = &text[start..start + len];
                while chars.peek().is_some_and(|&(i, _)| i < start + len + 1) {
                    chars.next();
                }
                if next == '{' {
                    return Err(not_supported(&format!("{{{name}}}"), name));
                }
                match name.split_once(char::is_whitespace) {
                    Some((function, _)) => return Err(format!("$({function}) not defined")),
                    None => match lookup(name) {
                        Some(value) => out.push_str(&value),
                        None => return Err(format!("$({name}) not defined")),
                    },
                }
            }
            '@' | '<' | '^' => match lookup(&next.to_string()) {
                Some(value) => out.push_str(&value),
                None => return Err(format!("$({next}) not defined")),
            },
            first => {
                let mut name = first.to_string();
                while let Some(&(_, w)) = chars.peek() {
                    if !(w.is_ascii_alphanumeric() || w == '_') {
                        break;
                    }
                    name.push(w);
                    chars.next();
                }
                return Err(not_supported(&name, &name));
            }
        }
    }
    Ok(out)
}

fn not_supported(written: &str, name: &str) -> String {
    format!(
        "'${written}' syntax is not supported; use '$({name})' instead for \"Make\" variables, \
         or escape the '$' as '$$' if you intended this for the shell"
    )
}

/// What a command may refer to.
pub struct Expander<'a> {
    pub rule_class: &'a str,
    pub attribute: &'a str,
    pub label: &'a Label,
    /// The rule's source files, `$(SRCS)`.
    pub srcs: &'a [Artifact],
    pub outs: &'a [Artifact],
    pub prerequisites: &'a [Prerequisite],
    pub bin_dir: &'a str,
    pub target_cpu: &'a str,
    pub compilation_mode: &'a str,
    pub defines: &'a BTreeMap<String, String>,
    /// What the targets of the `toolchains` attribute give as Make variables,
    /// in the order the attribute lists them: the first to name one wins.
    pub toolchain_variables: &'a [Vec<(String, String)>],
    /// The runfiles name of the main repository, `_main`.
    pub main_repo_name: &'a str,
    /// Where relative labels in `$(location ...)` are read.
    pub context: LabelContext<'a>,
}

const LOCATION_FUNCTIONS: [&str; 8] = [
    "location",
    "locations",
    "execpath",
    "execpaths",
    "rootpath",
    "rootpaths",
    "rlocationpath",
    "rlocationpaths",
];

fn with_dot_slash(path: String) -> String {
    if path.contains('/') {
        path
    } else {
        format!("./{path}")
    }
}

impl Expander<'_> {
    fn error(&self, message: impl Into<String>) -> ExpandError {
        ExpandError {
            attribute: self.attribute.to_owned(),
            rule_class: self.rule_class.to_owned(),
            label: label_text(self.label),
            message: message.into(),
        }
    }

    pub fn expand(&self, text: &str) -> Result<String, ExpandError> {
        let mut out = String::with_capacity(text.len());
        let mut chars = text.char_indices().peekable();
        while let Some((at, c)) = chars.next() {
            if c != '$' {
                out.push(c);
                continue;
            }
            match chars.peek().copied() {
                Some((_, '$')) => {
                    chars.next();
                    out.push('$');
                }
                Some((_, '(')) => {
                    let start = at + 2;
                    let Some(len) = text[start..].find(')') else {
                        return Err(self.error("unterminated variable reference"));
                    };
                    let inner = &text[start..start + len];
                    // Skip what was read.
                    while chars.peek().is_some_and(|&(i, _)| i < start + len + 1) {
                        chars.next();
                    }
                    out.push_str(&self.reference(inner)?);
                }
                Some((_, single)) => {
                    chars.next();
                    out.push_str(&self.variable(&single.to_string())?);
                }
                None => return Err(self.error("unterminated variable reference")),
            }
        }
        Ok(out)
    }

    /// Only the `$(location ...)` family, as `ctx.expand_location` does:
    /// every other `$` stays as written.
    pub fn expand_locations(&self, text: &str) -> Result<String, ExpandError> {
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(at) = rest.find("$(") {
            out.push_str(&rest[..at]);
            let after = &rest[at + 2..];
            let Some(len) = after.find(')') else {
                return Err(self.error("unterminated variable reference"));
            };
            let inner = &after[..len];
            match inner.split_once(char::is_whitespace) {
                Some((word, label)) if LOCATION_FUNCTIONS.contains(&word) => {
                    out.push_str(&self.location(word, label.trim_start())?);
                }
                _ => {
                    out.push_str("$(");
                    out.push_str(inner);
                    out.push(')');
                }
            }
            rest = &after[len + 1..];
        }
        out.push_str(rest);
        Ok(out)
    }

    /// What is inside `$( )`: a location function or a variable.
    fn reference(&self, inner: &str) -> Result<String, ExpandError> {
        let (word, rest) = match inner.split_once(char::is_whitespace) {
            Some((word, rest)) => (word, Some(rest.trim_start())),
            None => (inner, None),
        };
        match (LOCATION_FUNCTIONS.contains(&word), rest) {
            (true, Some(label)) => self.location(word, label),
            _ => self.variable(inner),
        }
    }

    fn variable(&self, name: &str) -> Result<String, ExpandError> {
        let join = |files: &[Artifact]| {
            files
                .iter()
                .map(Artifact::exec_path)
                .collect::<Vec<_>>()
                .join(" ")
        };
        Ok(match name {
            "SRCS" => join(self.srcs),
            "OUTS" => join(self.outs),
            "<" => match self.srcs {
                [one] => one.exec_path(),
                [] => return Err(self.error("variable '$<' : no input file")),
                _ => return Err(self.error("variable '$<' : more than one input file")),
            },
            "@" => match self.outs {
                [one] => one.exec_path(),
                _ => return Err(self.error("variable '$@' : more than one output file")),
            },
            "@D" => match self.outs {
                [one] => dirname(&one.exec_path()),
                _ => self.rule_dir(),
            },
            "RULEDIR" => self.rule_dir(),
            // A `--define` is read before a toolchain's variable, and that
            // before the configuration's own.
            other => {
                if let Some(value) = self.defines.get(other) {
                    return Ok(value.clone());
                }
                let from_toolchain = self.toolchain_variables.iter().find_map(|variables| {
                    variables
                        .iter()
                        .rev()
                        .find(|(k, _)| k == other)
                        .map(|(_, v)| v.clone())
                });
                if let Some(value) = from_toolchain {
                    return Ok(value);
                }
                match other {
                    "GENDIR" | "BINDIR" => self.bin_dir.to_owned(),
                    "TARGET_CPU" => self.target_cpu.to_owned(),
                    "COMPILATION_MODE" => self.compilation_mode.to_owned(),
                    _ => return Err(self.error(format!("$({other}) not defined"))),
                }
            }
        })
    }

    fn rule_dir(&self) -> String {
        let repo = if self.label.repo.is_empty() {
            String::new()
        } else {
            format!("external/{}", self.label.repo)
        };
        [self.bin_dir, repo.as_str(), self.label.package.as_str()]
            .iter()
            .filter(|part| !part.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join("/")
    }

    fn location(&self, function: &str, written: &str) -> Result<String, ExpandError> {
        let label = Label::parse(written, self.context)
            .map_err(|e| self.error(format!("invalid label in {function} expression: {e}")))?;
        let Some(found) = self.prerequisites.iter().find(|p| p.label == label) else {
            return Err(self.error(format!(
                "label '{}' in $({function}) expression is not a declared prerequisite of this rule",
                label_text(&label)
            )));
        };
        let plural = function.ends_with('s');
        if !plural && found.files.len() != 1 {
            return Err(self.error(format!(
                "label '{}' in $({function}) expression expands to {} files, which is not one; \
                 use $({function}s) if you want all of them",
                label_text(&label),
                found.files.len()
            )));
        }
        let kind = function.trim_end_matches('s');
        Ok(found
            .files
            .iter()
            .map(|f| self.path_of(kind, f))
            .collect::<Vec<_>>()
            .join(" "))
    }

    fn path_of(&self, kind: &str, file: &Artifact) -> String {
        match kind {
            // In an action, a location is where the file is in the execroot.
            "location" | "execpath" => with_dot_slash(file.exec_path()),
            "rootpath" => with_dot_slash(self.root_path(file)),
            _ => self.rlocation_path(file),
        }
    }

    /// Relative to the runfiles root of the main repository.
    fn root_path(&self, file: &Artifact) -> String {
        match self.repo_of(file) {
            Some(repo) => format!("../{repo}/{}", self.package_path(file, &repo)),
            None => self.package_path(file, ""),
        }
    }

    fn rlocation_path(&self, file: &Artifact) -> String {
        match self.repo_of(file) {
            Some(repo) => format!("{repo}/{}", self.package_path(file, &repo)),
            None => format!("{}/{}", self.main_repo_name, self.package_path(file, "")),
        }
    }

    /// The external repository a file belongs to, if it does.
    fn repo_of(&self, file: &Artifact) -> Option<String> {
        let rest = if file.is_source() {
            file.root.prefix.strip_prefix("external/")?
        } else {
            file.path.strip_prefix("external/")?
        };
        Some(rest.split('/').next()?.to_owned())
    }

    /// The path under the repository's root.
    fn package_path(&self, file: &Artifact, repo: &str) -> String {
        if repo.is_empty() {
            return file.path.clone();
        }
        let path = file
            .path
            .strip_prefix(&format!("external/{repo}/"))
            .unwrap_or(&file.path);
        path.to_owned()
    }
}

fn dirname(path: &str) -> String {
    path.rsplit_once('/')
        .map(|(dir, _)| dir.to_owned())
        .unwrap_or_default()
}

/// `//p:n` for the main repository, `@@repo//p:n` for another.
pub fn label_text(label: &Label) -> String {
    if label.repo.is_empty() {
        format!("//{}:{}", label.package, label.name)
    } else {
        format!("@@{}//{}:{}", label.repo, label.package, label.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn label(package: &str, name: &str) -> Label {
        Label {
            repo: String::new(),
            package: package.into(),
            name: name.into(),
        }
    }

    const BIN: &str = "bazel-out/k8-fastbuild/bin";

    fn expand(
        srcs: &[Artifact],
        outs: &[Artifact],
        prerequisites: &[Prerequisite],
        text: &str,
    ) -> Result<String, String> {
        let me = label("", "t");
        let defines = BTreeMap::from([("K".to_owned(), "v".to_owned())]);
        Expander {
            rule_class: "genrule",
            attribute: "cmd",
            label: &me,
            srcs,
            outs,
            prerequisites,
            bin_dir: BIN,
            target_cpu: "x86_64",
            compilation_mode: "fastbuild",
            defines: &defines,
            toolchain_variables: &[],
            main_repo_name: "_main",
            context: LabelContext {
                repo: "",
                package: "",
            },
        }
        .expand(text)
        .map_err(|e| e.to_string())
    }

    fn out(name: &str) -> Artifact {
        Artifact::derived(BIN, "", "", name)
    }

    /// Each row is a command and what `bazel aquery` showed for a genrule
    /// `t` in the root package with `srcs = ["in.txt"]`, `outs = ["o.txt"]`.
    #[test]
    fn variables_and_locations_expand_as_bazel_does() {
        let src = Artifact::source("", "", "in.txt");
        let pre = [Prerequisite {
            label: label("", "in.txt"),
            files: vec![src.clone()],
        }];
        for (cmd, want) in [
            (
                "echo $(SRCS) $(OUTS)",
                "echo in.txt bazel-out/k8-fastbuild/bin/o.txt",
            ),
            (
                "echo $< $@ $(@D)",
                "echo in.txt bazel-out/k8-fastbuild/bin/o.txt bazel-out/k8-fastbuild/bin",
            ),
            (
                "echo $(RULEDIR) $(GENDIR) $(BINDIR) $(TARGET_CPU) $(COMPILATION_MODE)",
                "echo bazel-out/k8-fastbuild/bin bazel-out/k8-fastbuild/bin bazel-out/k8-fastbuild/bin x86_64 fastbuild",
            ),
            (
                "echo $(location in.txt) $(execpath in.txt) $(rootpath in.txt) $(rlocationpath in.txt)",
                "echo ./in.txt ./in.txt ./in.txt _main/in.txt",
            ),
            ("echo $$HOME $${X} $$$$", "echo $HOME ${X} $$"),
            (
                "echo $(locations in.txt) $(execpaths in.txt)",
                "echo ./in.txt ./in.txt",
            ),
            ("echo $(K)", "echo v"),
        ] {
            assert_eq!(
                expand(std::slice::from_ref(&src), &[out("o.txt")], &pre, cmd).unwrap(),
                want,
                "{cmd}"
            );
        }
    }

    #[test]
    fn a_path_in_a_package_has_no_dot_slash_and_an_output_directory_is_its_own() {
        let f = Artifact::source("", "a", "f.txt");
        let pre = [Prerequisite {
            label: label("a", "f.txt"),
            files: vec![f.clone()],
        }];
        assert_eq!(
            expand(
                std::slice::from_ref(&f),
                &[out("x/o.txt")],
                &pre,
                "echo $< $(location //a:f.txt) $@ $(@D)"
            )
            .unwrap(),
            "echo a/f.txt a/f.txt bazel-out/k8-fastbuild/bin/x/o.txt bazel-out/k8-fastbuild/bin/x"
        );
        // With several outputs `$(@D)` is the rule's directory, not their common one.
        assert_eq!(
            expand(&[f], &[out("x/o1"), out("x/o2")], &[], "echo $(@D)").unwrap(),
            "echo bazel-out/k8-fastbuild/bin"
        );
    }

    #[test]
    fn what_cannot_be_expanded_is_reported_in_bazels_words() {
        let src = Artifact::source("", "", "in.txt");
        let two = [src.clone(), Artifact::source("", "a", "f.txt")];
        let t = |srcs: &[Artifact], outs: &[Artifact], cmd: &str| {
            expand(srcs, outs, &[], cmd).unwrap_err()
        };
        let prefix = "in cmd attribute of genrule rule //:t: ";
        for (srcs, outs, cmd, want) in [
            (
                &two[..1],
                &[out("o")][..],
                "echo $(location nonexistent)",
                "label '//:nonexistent' in $(location) expression is not a declared prerequisite of this rule",
            ),
            (
                &two[..1],
                &[out("o")][..],
                "echo $(foo)",
                "$(foo) not defined",
            ),
            (
                &two[..],
                &[out("o")][..],
                "echo $<",
                "variable '$<' : more than one input file",
            ),
            (
                &two[..1],
                &[out("o1"), out("o2")][..],
                "echo $@",
                "variable '$@' : more than one output file",
            ),
            (
                &[][..],
                &[out("o")][..],
                "echo $<",
                "variable '$<' : no input file",
            ),
            (
                &two[..1],
                &[out("o")][..],
                "echo $(location)",
                "$(location) not defined",
            ),
            (
                &two[..1],
                &[out("o")][..],
                "echo $(SRCS",
                "unterminated variable reference",
            ),
            (
                &two[..1],
                &[out("o")][..],
                "echo $(JAVA)",
                "$(JAVA) not defined",
            ),
        ] {
            assert_eq!(t(srcs, outs, cmd), format!("{prefix}{want}"), "{cmd}");
        }
    }

    #[test]
    fn a_label_in_another_repository_is_a_path_into_external() {
        let dep = Artifact::source("dep+", "p", "x.txt");
        let pre = [Prerequisite {
            label: Label {
                repo: "dep+".into(),
                package: "p".into(),
                name: "x.txt".into(),
            },
            files: vec![dep],
        }];
        assert_eq!(
            expand(&[], &[out("o")], &pre, "echo $(execpath @@dep+//p:x.txt) $(rootpath @@dep+//p:x.txt) $(rlocationpath @@dep+//p:x.txt)").unwrap(),
            "echo external/dep+/p/x.txt ../dep+/p/x.txt dep+/p/x.txt"
        );
    }

    /// Every row is what Bazel 9.2.0's `ctx.expand_make_variables` gave with
    /// `FOO = bar` and `"" = empty` as the variables.
    #[test]
    fn make_variables_expand_as_bazel_does() {
        let lookup = |name: &str| match name {
            "FOO" => Some("bar".to_owned()),
            "" => Some("empty".to_owned()),
            _ => None,
        };
        let not_supported = |written: &str, name: &str| {
            format!(
                "'${written}' syntax is not supported; use '$({name})' instead for \"Make\" variables, or escape the '$' as '$$' if you intended this for the shell"
            )
        };
        for (text, want) in [
            ("plain", Ok("plain")),
            ("$(FOO) $(FOO)", Ok("bar bar")),
            ("$(FOO)$(FOO)", Ok("barbar")),
            ("$$FOO", Ok("$FOO")),
            ("$$", Ok("$")),
            ("$$(FOO)", Ok("$(FOO)")),
            ("$$$(FOO)", Ok("$bar")),
            ("$(FOO))", Ok("bar)")),
            ("é$(FOO)é", Ok("ébaré")),
            ("$()", Ok("empty")),
            ("$(FOO", Err("unterminated variable reference".to_owned())),
            ("${", Err("unterminated variable reference".to_owned())),
            ("a$", Err("unterminated $".to_owned())),
            ("$(NOPE)", Err("$(NOPE) not defined".to_owned())),
            ("$(foo)", Err("$(foo) not defined".to_owned())),
            ("$(FOO )", Err("$(FOO) not defined".to_owned())),
            ("$(location :t)", Err("$(location) not defined".to_owned())),
            ("$(FOO$(FOO))", Err("$(FOO$(FOO) not defined".to_owned())),
            ("$@", Err("$(@) not defined".to_owned())),
            ("$<", Err("$(<) not defined".to_owned())),
            ("$^", Err("$(^) not defined".to_owned())),
            ("$FOO-bar", Err(not_supported("FOO", "FOO"))),
            ("$1", Err(not_supported("1", "1"))),
            ("$-", Err(not_supported("-", "-"))),
            ("$ x", Err(not_supported(" x", " x"))),
            ("$A.B", Err(not_supported("A", "A"))),
            ("$*", Err(not_supported("*", "*"))),
            ("${FOO}x", Err(not_supported("{FOO}", "FOO"))),
            ("${}", Err(not_supported("{}", ""))),
            ("${F OO}", Err(not_supported("{F OO}", "F OO"))),
        ] {
            assert_eq!(
                expand_make_variables(text, &lookup),
                want.map(str::to_owned),
                "{text}"
            );
        }
        // `$@` is a variable when something defines it.
        let at = |name: &str| (name == "@").then(|| "out".to_owned());
        assert_eq!(expand_make_variables("a $@ b", &at), Ok("a out b".into()));
    }
}
