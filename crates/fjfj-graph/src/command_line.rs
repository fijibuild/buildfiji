//! What a command line holds that is known only when the action runs
//! (buildfiji-136.37), and how a parameter file spells a command line.
//!
//! `Args.add_all` and `Args.add_joined` expand a tree artifact, a directory an
//! action fills, into one word for each file in it. Which files those are is
//! not known when the rule is analysed, so a call that reads a tree is kept
//! as it was made, with the tree as a name, and expanded when the action runs.
//! The words `argv` holds for it are what `aquery` shows: the tree as one
//! word, which is how Bazel shows it too.

use std::fmt::Write as _;

/// How a parameter file lists its arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ParamFormat {
    /// Each argument on a line, quoted as a shell would need.
    Shell,
    /// Each argument on a line, as it is.
    Multiline,
    /// `--flag=value` arguments split at the `=`, each half on a line.
    FlagPerLine,
}

/// One value of a call that reads a tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum LazyItem {
    /// A word that is known, `format_each` applied.
    Text(String),
    /// The tree at this exec path: a word for each file in it, `format_each`
    /// applied to each.
    Tree {
        dir: String,
        format_each: Option<String>,
    },
}

/// A call of `add_all` or `add_joined` that reads a tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum LazyCall {
    AddAll {
        /// The name that goes before the values, if they are not empty or
        /// `omit_if_empty` is off.
        name: Option<String>,
        items: Vec<LazyItem>,
        before_each: Option<String>,
        omit_if_empty: bool,
        uniquify: bool,
        terminate_with: Option<String>,
    },
    AddJoined {
        name: Option<String>,
        items: Vec<LazyItem>,
        join_with: String,
        format_joined: Option<String>,
        omit_if_empty: bool,
        uniquify: bool,
    },
}

/// A call in the words of a command line, which stand for it until the tree
/// is known.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct LazyArg {
    /// Where its words start in the command line.
    pub at: usize,
    /// How many words it made before the tree was known.
    pub len: usize,
    pub call: LazyCall,
}

/// `pattern` with `%s` replaced by `text`; `%%` is a `%`.
pub fn format_arg(pattern: &str, text: &str) -> String {
    let mut out = String::with_capacity(pattern.len() + text.len());
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, chars.peek()) {
            ('%', Some('s')) => {
                chars.next();
                out.push_str(text);
            }
            ('%', Some('%')) => {
                chars.next();
                out.push('%');
            }
            _ => out.push(c),
        }
    }
    out
}

impl LazyCall {
    /// The words the call makes once `files_of` says which files each tree
    /// holds, as exec paths in the order they are put on the command line.
    pub fn words(&self, files_of: &dyn Fn(&str) -> Vec<String>) -> Vec<String> {
        let (items, uniquify) = match self {
            LazyCall::AddAll {
                items, uniquify, ..
            }
            | LazyCall::AddJoined {
                items, uniquify, ..
            } => (items, *uniquify),
        };
        let mut values: Vec<String> = Vec::new();
        for item in items {
            match item {
                LazyItem::Text(text) => values.push(text.clone()),
                LazyItem::Tree { dir, format_each } => {
                    for file in files_of(dir) {
                        values.push(match format_each {
                            Some(pattern) => format_arg(pattern, &file),
                            None => file,
                        });
                    }
                }
            }
        }
        if uniquify {
            let mut seen: Vec<String> = Vec::new();
            values.retain(|v| {
                let new = !seen.contains(v);
                if new {
                    seen.push(v.clone());
                }
                new
            });
        }
        match self {
            LazyCall::AddAll {
                name,
                before_each,
                omit_if_empty,
                terminate_with,
                ..
            } => {
                if values.is_empty() && *omit_if_empty {
                    return Vec::new();
                }
                let mut out: Vec<String> = name.iter().cloned().collect();
                for value in values {
                    out.extend(before_each.iter().cloned());
                    out.push(value);
                }
                out.extend(terminate_with.iter().cloned());
                out
            }
            LazyCall::AddJoined {
                name,
                join_with,
                format_joined,
                omit_if_empty,
                ..
            } => {
                if values.is_empty() && *omit_if_empty {
                    return Vec::new();
                }
                let mut joined = values.join(join_with);
                if let Some(pattern) = format_joined {
                    joined = format_arg(pattern, &joined);
                }
                let mut out: Vec<String> = name.iter().cloned().collect();
                out.push(joined);
                out
            }
        }
    }
}

/// `words` with each call of `lazy` replaced by what it makes.
pub fn expand_words(
    words: &[String],
    lazy: &[LazyArg],
    files_of: &dyn Fn(&str) -> Vec<String>,
) -> Vec<String> {
    let mut out = Vec::with_capacity(words.len());
    let mut at = 0;
    for call in lazy {
        out.extend_from_slice(&words[at.min(call.at)..call.at]);
        out.extend(call.call.words(files_of));
        at = call.at + call.len;
    }
    out.extend_from_slice(&words[at.min(words.len())..]);
    out
}

/// The text of a parameter file for `items`.
pub fn param_file_contents(items: &[String], format: ParamFormat) -> String {
    let mut out = String::new();
    for item in items {
        match format {
            ParamFormat::Shell => {
                out.push_str(&shell_quote(item));
                out.push('\n');
            }
            ParamFormat::Multiline => {
                out.push_str(item);
                out.push('\n');
            }
            ParamFormat::FlagPerLine => {
                match item
                    .strip_prefix("--")
                    .and_then(|rest| rest.split_once('='))
                {
                    Some((flag, value)) => {
                        let _ = write!(out, "--{flag}\n{value}\n");
                    }
                    None => {
                        out.push_str(item);
                        out.push('\n');
                    }
                }
            }
        }
    }
    out
}

/// `arg` as a shell would need it written.
pub fn shell_quote(arg: &str) -> String {
    let plain = !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./:,+@%".contains(c));
    if plain {
        arg.to_owned()
    } else {
        format!("'{}'", arg.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(dir: &str) -> Vec<String> {
        match dir {
            "t" => vec!["t/b".to_owned(), "t/x/a".to_owned()],
            _ => Vec::new(),
        }
    }

    fn tree(dir: &str, format: Option<&str>) -> LazyItem {
        LazyItem::Tree {
            dir: dir.to_owned(),
            format_each: format.map(str::to_owned),
        }
    }

    fn all(name: Option<&str>, items: Vec<LazyItem>) -> LazyCall {
        LazyCall::AddAll {
            name: name.map(str::to_owned),
            items,
            before_each: None,
            omit_if_empty: true,
            uniquify: false,
            terminate_with: None,
        }
    }

    /// Probed on Bazel 9.2.0 with a tree of `b` and `x/a` and an empty one.
    #[test]
    fn a_tree_is_a_word_for_each_file_and_an_empty_one_leaves_out_its_name() {
        let call = LazyCall::AddAll {
            name: Some("--name".into()),
            items: vec![tree("t", Some("<%s>"))],
            before_each: Some("-I".into()),
            omit_if_empty: true,
            uniquify: false,
            terminate_with: Some("END".into()),
        };
        assert_eq!(
            call.words(&files),
            ["--name", "-I", "<t/b>", "-I", "<t/x/a>", "END"]
        );
        let empty = LazyCall::AddAll {
            name: Some("--name".into()),
            items: vec![tree("e", None)],
            before_each: Some("-I".into()),
            omit_if_empty: true,
            uniquify: false,
            terminate_with: Some("END".into()),
        };
        assert!(empty.words(&files).is_empty());
        let kept = LazyCall::AddAll {
            name: Some("--empty2".into()),
            items: vec![tree("e", None)],
            before_each: None,
            omit_if_empty: false,
            uniquify: false,
            terminate_with: None,
        };
        assert_eq!(kept.words(&files), ["--empty2"]);
    }

    #[test]
    fn values_of_a_call_with_a_tree_keep_their_place_and_uniquify_looks_at_all_of_them() {
        let call = LazyCall::AddAll {
            name: None,
            items: vec![
                LazyItem::Text("pre".into()),
                tree("t", None),
                LazyItem::Text("t/b".into()),
                LazyItem::Text("post".into()),
            ],
            before_each: None,
            omit_if_empty: true,
            uniquify: true,
            terminate_with: None,
        };
        assert_eq!(call.words(&files), ["pre", "t/b", "t/x/a", "post"]);
    }

    #[test]
    fn a_joined_call_is_one_word_that_may_be_empty() {
        let joined = |items, omit_if_empty| LazyCall::AddJoined {
            name: Some("--j".into()),
            items,
            join_with: ",".into(),
            format_joined: Some("{%s}".into()),
            omit_if_empty,
            uniquify: false,
        };
        assert_eq!(
            joined(
                vec![tree("t", Some("[%s]")), LazyItem::Text("[x]".into())],
                true
            )
            .words(&files),
            ["--j", "{[t/b],[t/x/a],[x]}"]
        );
        assert!(joined(vec![tree("e", None)], true).words(&files).is_empty());
        assert_eq!(
            joined(vec![tree("e", None)], false).words(&files),
            ["--j", "{}"]
        );
    }

    #[test]
    fn calls_replace_the_words_they_stood_for() {
        let words: Vec<String> = ["a", "--name", "t", "b", "t", "c"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let lazy = [
            LazyArg {
                at: 1,
                len: 2,
                call: all(Some("--name"), vec![tree("t", None)]),
            },
            LazyArg {
                at: 4,
                len: 1,
                call: all(None, vec![tree("e", None)]),
            },
        ];
        assert_eq!(
            expand_words(&words, &lazy, &files),
            ["a", "--name", "t/b", "t/x/a", "b", "c"]
        );
    }

    #[test]
    fn a_parameter_file_quotes_as_its_format_says() {
        let items: Vec<String> = ["--a=b c", "it's", "plain"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(
            param_file_contents(&items, ParamFormat::Shell),
            "'--a=b c'\n'it'\\''s'\nplain\n"
        );
        assert_eq!(
            param_file_contents(&items, ParamFormat::Multiline),
            "--a=b c\nit's\nplain\n"
        );
        assert_eq!(
            param_file_contents(&items, ParamFormat::FlagPerLine),
            "--a\nb c\nit's\nplain\n"
        );
    }
}
