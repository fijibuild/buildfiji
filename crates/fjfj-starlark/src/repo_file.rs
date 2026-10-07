//! `REPO.bazel`: the file at a repo's root that holds `repo()` and
//! `ignore_directories()` (buildfiji-e4r).
//!
//! Behaviour checked against Bazel 9.2.0:
//!
//! - the dialect is a BUILD file's with two functions, `ignore_directories`
//!   and `repo`, and no `load`; assignments and `print` work.
//! - `ignore_directories(dirs)` takes one positional list of strings, once.
//!   Each is a pattern over directory paths relative to the repo root, as
//!   `glob()` reads them except that `?` is a wildcard too; a pattern that
//!   is absolute, has `..`, ends in `/` or is empty matches nothing.
//! - `repo(...)` takes `package()`'s arguments, at least one, once, and
//!   before any other function. Only parsed here.
//! - a syntax error, a `load` or an undefined name is an event at its
//!   place; a failing call is a traceback, except an unknown keyword of
//!   `repo()`, which is a bare message.

use crate::FileKind;
use crate::args::{Wording, bind, describe, fatal, param, positional_only, sequence};
use crate::dialect::{parse_all, scope_errors};
use crate::syntax_event;
use starlark::environment::{Globals, GlobalsBuilder, Module};
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_module;
use starlark::values::none::NoneType;
use starlark::values::{ProvidesStaticType, Value};
use starlark_syntax::syntax::ast::{AstStmt, StmtP};
use std::cell::RefCell;

/// What a `REPO.bazel` says.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepoFile {
    /// The patterns of `ignore_directories()`.
    pub ignore_directories: Vec<String>,
    /// Whether `repo()` was called.
    pub repo_called: bool,
    /// What the file printed, as `DEBUG: file:line:col: text` events.
    pub printed: Vec<String>,
}

/// Why a `REPO.bazel` could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{summary}")]
pub struct RepoFileError {
    /// Events printed before the failure, without their `ERROR: ` prefix:
    /// `file:line:col: message`, a traceback, or a `DEBUG: ` line.
    pub events: Vec<String>,
    /// What Bazel says stopped, e.g. `error parsing REPO.bazel file for the
    /// main repo`.
    pub summary: String,
}

/// Run the `REPO.bazel` whose text is `source`, known as `path` in events.
/// `whose` completes Bazel's `for ...`: `the main repo`.
pub fn evaluate_repo_file(
    path: &str,
    source: &str,
    whose: &str,
) -> Result<RepoFile, RepoFileError> {
    let stop = |events: Vec<String>, summary: String| RepoFileError { events, summary };
    let parsed = parse_all(path, source, FileKind::Build);
    if !parsed.syntax.is_empty() {
        return Err(stop(
            parsed
                .syntax
                .iter()
                .map(|e| syntax_event(path, e))
                .collect(),
            format!("error parsing REPO.bazel file at {path}"),
        ));
    }
    let parsing = format!("error parsing REPO.bazel file for {whose}");
    // Every `load` is one event, at the statement.
    let top: &[AstStmt] = match &parsed.ast.statement().node {
        StmtP::Statements(v) => v,
        _ => std::slice::from_ref(parsed.ast.statement()),
    };
    let loads: Vec<String> = top
        .iter()
        .filter(|stmt| matches!(stmt.node, StmtP::Load(_)))
        .map(|stmt| {
            let at = parsed.ast.file_span(stmt.span).resolve_span().begin;
            format!(
                "{path}:{}:{}: `load` statements may not be used in REPO.bazel files",
                at.line + 1,
                at.column + 1
            )
        })
        .collect();
    if !loads.is_empty() {
        return Err(stop(loads, parsing));
    }
    let globals = repo_globals();
    let undefined = Module::with_temp_heap(|module| {
        Ok::<_, starlark::Error>(scope_errors(
            parse_all(path, source, FileKind::Build).ast,
            parsed.resolution,
            &module,
            &globals,
        ))
    })
    .unwrap_or_default();
    if !undefined.is_empty() {
        return Err(stop(
            undefined.iter().map(|e| syntax_event(path, e)).collect(),
            parsing,
        ));
    }
    let context = RepoContext::default();
    let outcome = Module::with_temp_heap(|module| {
        let mut eval = Evaluator::new(&module);
        eval.extra = Some(&context);
        eval.set_print_handler(&context);
        eval.eval_module(parsed.ast, &globals).map(|_| ())
    });
    let state = context.state.into_inner();
    let printed = state.printed;
    match outcome {
        Ok(()) => Ok(RepoFile {
            ignore_directories: state.ignored.unwrap_or_default(),
            repo_called: state.repo_called,
            printed,
        }),
        Err(error) => {
            let mut events = printed;
            events.push(match state.bare {
                Some(message) => message,
                None => crate::traceback(&error),
            });
            Err(stop(
                events,
                format!("error evaluating REPO.bazel file for {whose}"),
            ))
        }
    }
}

fn repo_globals() -> Globals {
    crate::native::bazel_standard().with(repo_functions).build()
}

#[derive(Default)]
struct RepoState {
    ignored: Option<Vec<String>>,
    repo_called: bool,
    /// Another function ran, so `repo()` is too late.
    other_called: bool,
    printed: Vec<String>,
    /// A failure Bazel reports as a line, with no traceback.
    bare: Option<String>,
}

#[derive(Default, ProvidesStaticType)]
struct RepoContext {
    state: RefCell<RepoState>,
}

impl starlark::PrintHandler for RepoContext {
    fn println(&self, text: &str) -> starlark::Result<()> {
        self.state
            .borrow_mut()
            .printed
            .push(format!("DEBUG: {text}"));
        Ok(())
    }

    fn println_at(
        &self,
        location: Option<&starlark::codemap::FileSpan>,
        text: &str,
    ) -> starlark::Result<()> {
        self.println(&crate::print_line(location, text))
    }
}

fn context<'a>(eval: &Evaluator<'_, 'a, '_>) -> &'a RepoContext {
    eval.extra
        .and_then(|e| e.downcast_ref::<RepoContext>())
        .expect("evaluated by evaluate_repo_file")
}

/// `repo()` takes what `package()` does.
const REPO_PARAMS: [&str; 10] = [
    "default_visibility",
    "default_testonly",
    "default_deprecation",
    "default_compatible_with",
    "default_restricted_to",
    "default_hdrs_check",
    "licenses",
    "default_applicable_licenses",
    "default_package_metadata",
    "features",
];

#[starlark_module]
fn repo_functions(builder: &mut GlobalsBuilder) {
    fn ignore_directories<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let ctx = context(eval);
        let bound = bind(
            "ignore_directories",
            Wording::Signature,
            &[positional_only("dirs", true)],
            args,
            eval,
        )?;
        let dirs = bound[0].expect("required");
        let items = sequence(dirs).ok_or_else(|| {
            fatal(format!(
                "in call to ignore_directories(), parameter 'dirs' got value of type '{}', \
                 want 'sequence'",
                dirs.get_type()
            ))
        })?;
        let mut patterns = Vec::new();
        for (i, item) in items.iter().enumerate() {
            let Some(text) = item.unpack_str() else {
                return Err(fatal(format!(
                    "at index {i} of dirs, got element of type {}, want string",
                    item.get_type()
                )));
            };
            patterns.push(text.to_owned());
        }
        let mut state = ctx.state.borrow_mut();
        if state.ignored.is_some() {
            return Err(fatal("'ignored_directories()' can only be called once"));
        }
        state.ignored = Some(patterns);
        state.other_called = true;
        Ok(NoneType)
    }

    fn repo<'v>(
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let ctx = context(eval);
        let params: Vec<_> = REPO_PARAMS.iter().map(|n| param(n, false, false)).collect();
        let bound = match bind("repo", Wording::Package, &params, args, eval) {
            Ok(bound) => bound,
            Err(error) => {
                // Bazel gives an unknown keyword as a bare line.
                if error.to_string().starts_with("unexpected keyword argument") {
                    ctx.state.borrow_mut().bare = Some(error.to_string());
                }
                return Err(error);
            }
        };
        {
            let state = ctx.state.borrow();
            if state.other_called {
                return Err(fatal(
                    "if repo() is called, it must be called before any other functions",
                ));
            }
            if state.repo_called {
                return Err(fatal(
                    "'repo' can only be called once in the REPO.bazel file",
                ));
            }
        }
        if bound.iter().all(Option::is_none) {
            return Err(fatal(
                "at least one argument must be given to the 'repo' function",
            ));
        }
        if let Some(visibility) = bound[0] {
            let Some(items) = sequence(visibility) else {
                ctx.state.borrow_mut().bare = Some(format!(
                    "expected value of type 'list(label)' for repo() argument \
                     'default_visibility', but got {}",
                    describe(visibility)
                ));
                return Err(fatal("repo() argument"));
            };
            let _: Vec<Value<'v>> = items;
        }
        ctx.state.borrow_mut().repo_called = true;
        Ok(NoneType)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(source: &str) -> Result<RepoFile, RepoFileError> {
        evaluate_repo_file("REPO.bazel", source, "the main repo")
    }

    fn failure(source: &str) -> (Vec<String>, String) {
        let e = read(source).unwrap_err();
        (e.events, e.summary)
    }

    /// The last line of the traceback: what Bazel says went wrong.
    fn said(source: &str) -> String {
        let (events, summary) = failure(source);
        assert_eq!(
            summary,
            "error evaluating REPO.bazel file for the main repo"
        );
        events.last().unwrap().lines().last().unwrap().to_owned()
    }

    #[test]
    fn ignore_directories_and_repo_are_read() {
        let file = read("x = 1\nprint(\"hi\")\nignore_directories([\"c\", \"**/gen\"])").unwrap();
        assert_eq!(file.ignore_directories, ["c", "**/gen"]);
        assert_eq!(file.printed, ["DEBUG: REPO.bazel:2:6: hi"]);
        assert!(!file.repo_called);
        let file = read("repo(default_visibility = [\"//visibility:public\"], features = [\"a\"])\nignore_directories([])").unwrap();
        assert!(file.repo_called);
        assert_eq!(file.ignore_directories, Vec::<String>::new());
        assert_eq!(read("").unwrap(), RepoFile::default());
    }

    #[test]
    fn a_call_that_fails_says_what_bazel_says() {
        for (source, want) in [
            (
                "ignore_directories(\"c\")",
                "Error in ignore_directories: in call to ignore_directories(), parameter 'dirs' got value of type 'string', want 'sequence'",
            ),
            (
                "ignore_directories()",
                "Error in ignore_directories: ignore_directories() missing 1 required positional argument: dirs",
            ),
            (
                "ignore_directories([\"c\"], x = 1)",
                "Error in ignore_directories: ignore_directories() got unexpected keyword argument 'x'",
            ),
            (
                "ignore_directories(dirs = [\"c\"])",
                "Error in ignore_directories: ignore_directories() got named argument for positional-only parameter 'dirs'",
            ),
            (
                "ignore_directories([1])",
                "Error in ignore_directories: at index 0 of dirs, got element of type int, want string",
            ),
            (
                "ignore_directories([\"c\"])\nignore_directories([\"x\"])",
                "Error in ignore_directories: 'ignored_directories()' can only be called once",
            ),
            (
                "ignore_directories([\"c\"])\nrepo(default_testonly = True)",
                "Error in repo: if repo() is called, it must be called before any other functions",
            ),
            (
                "repo(default_testonly = True)\nrepo(default_testonly = True)",
                "Error in repo: 'repo' can only be called once in the REPO.bazel file",
            ),
            (
                "repo()",
                "Error in repo: at least one argument must be given to the 'repo' function",
            ),
            (
                "repo(1)",
                "Error in repo: repo() got unexpected positional argument",
            ),
        ] {
            assert_eq!(said(source), want, "{source}");
        }
    }

    #[test]
    fn a_bare_message_has_no_traceback() {
        assert_eq!(
            failure("repo(foo = 1)"),
            (
                vec!["unexpected keyword argument: foo".to_owned()],
                "error evaluating REPO.bazel file for the main repo".to_owned()
            )
        );
        let (events, _) = failure("repo(default_visibility = \"x\")");
        assert_eq!(
            events,
            [
                "expected value of type 'list(label)' for repo() argument 'default_visibility', but got \"x\" (string)"
            ]
        );
    }

    #[test]
    fn static_errors_are_events_at_their_place() {
        assert_eq!(
            failure("load(\"//:x.bzl\", \"y\")"),
            (
                vec![
                    "REPO.bazel:1:1: `load` statements may not be used in REPO.bazel files"
                        .to_owned()
                ],
                "error parsing REPO.bazel file for the main repo".to_owned()
            )
        );
        assert_eq!(
            failure("package(default_visibility = [])"),
            (
                vec!["REPO.bazel:1:1: name 'package' is not defined".to_owned()],
                "error parsing REPO.bazel file for the main repo".to_owned()
            )
        );
        assert_eq!(
            failure("native.repo()"),
            (
                vec!["REPO.bazel:1:1: name 'native' is not defined".to_owned()],
                "error parsing REPO.bazel file for the main repo".to_owned()
            )
        );
        assert_eq!(
            failure("ignore_directories([\"c\"\n"),
            (
                vec![
                    "REPO.bazel:2:1: syntax error at 'newline': expected ',', 'for' or ']'"
                        .to_owned()
                ],
                "error parsing REPO.bazel file at REPO.bazel".to_owned()
            )
        );
    }
}
