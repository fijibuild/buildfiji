//! Bazel's Starlark dialects and the file-level rules the `starlark` crate
//! does not enforce on its own.
//!
//! Every setting here was checked against Bazel 9.2.0 (buildfiji-mum.2):
//! `Dialect::Standard` is not Bazel's dialect, and the two file kinds differ.

use anyhow::anyhow;
use starlark::codemap::{FileSpan, Span};
use starlark::syntax::{AstModule, Dialect, DialectTypes};
use starlark_syntax::codemap::Pos;
use starlark_syntax::error::ErrorKind;
use starlark_syntax::syntax::ast::{
    AssignTargetP, AstExpr, AstLiteral, AstNoPayload, AstStmt, ExprP, StmtP,
};
use starlark_syntax::syntax::uniplate::Visit;
use std::collections::HashMap;

/// Which Bazel file dialect a source is parsed and checked under.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FileKind {
    /// `BUILD` / `BUILD.bazel`: no `def`, no `lambda`, no `if`/`for` statements.
    Build,
    /// `.bzl`: `def`, `lambda` and keyword-only parameters, loads first.
    Bzl,
}

impl FileKind {
    /// The `starlark` crate dialect for this kind of file.
    pub fn dialect(self) -> Dialect {
        match self {
            FileKind::Build => build_dialect(),
            FileKind::Bzl => bzl_dialect(),
        }
    }
}

/// Bazel's Starlark dialect settings for BUILD files.
///
/// `def` and `lambda` are both "functions" to Bazel and both rejected. `if`
/// and `for` statements are rejected by `enable_top_level_stmt: false`, which
/// is what BUILD files need (comprehensions and `if` expressions still work).
/// A BUILD file may `load()` anywhere and rebind a loaded name.
pub fn build_dialect() -> Dialect {
    Dialect {
        enable_def: false,
        enable_lambda: false,
        enable_load: true,
        enable_keyword_only_arguments: false,
        enable_positional_only_arguments: false,
        enable_types: DialectTypes::Disable,
        enable_load_reexport: false,
        enable_top_level_stmt: false,
        enable_f_strings: false,
        ..Dialect::Standard
    }
}

/// Dialect for `.bzl` files.
///
/// Bazel 9 accepts `lambda` (only BUILD files reject it), keyword-only
/// parameters after `*` (its own `@_builtins` use them), and parses type
/// annotations without acting on them. It rejects positional-only `/`,
/// f-strings and top-level `if`/`for`, and a `.bzl`'s own `load()` bindings
/// are private to it: another file loading the same name gets an error.
pub fn bzl_dialect() -> Dialect {
    Dialect {
        enable_def: true,
        enable_lambda: true,
        enable_load: true,
        enable_keyword_only_arguments: true,
        enable_positional_only_arguments: false,
        enable_types: DialectTypes::ParseOnly,
        enable_load_reexport: false,
        enable_top_level_stmt: false,
        enable_f_strings: false,
        ..Dialect::Standard
    }
}

/// Parse `src` under `kind`'s dialect and apply the file-level rules the
/// parser does not: in a `.bzl`, `load()` statements come before every other
/// statement and no top-level name is declared twice.
pub fn parse(path: &str, src: &str, kind: FileKind) -> anyhow::Result<AstModule> {
    let ast =
        AstModule::parse(path, src.to_owned(), &kind.dialect()).map_err(|e| e.into_anyhow())?;
    check_string_escapes(&ast).map_err(|e| e.into_anyhow())?;
    if kind == FileKind::Bzl {
        check_bzl_top_level(&ast).map_err(|e| e.into_anyhow())?;
    }
    Ok(ast)
}

/// Bazel's lexer accepts only `\\ \' \" \a \b \f \n \r \t \v`, an octal `\ooo` and
/// a backslash before a line break in a string that is not raw; the crate's also
/// takes `\x`, `\u` and `\U` (buildfiji-8q5). This looks at each string literal's text.
fn check_string_escapes(ast: &AstModule) -> Result<(), starlark::Error> {
    fn expr(ast: &AstModule, e: &AstExpr, found: &mut Option<starlark::Error>) {
        if found.is_some() {
            return;
        }
        if let ExprP::Literal(AstLiteral::String(_)) = &e.node {
            let text = ast.file_span(e.span).source_span().to_owned();
            if let Some((at, len, message)) = invalid_escape(&text) {
                let begin = e.span.begin().get() + at as u32;
                let span = Span::new(Pos::new(begin), Pos::new(begin + len as u32));
                *found = Some(error_at(ast, span, message));
            }
        }
        e.node.visit_expr(|child| expr(ast, child, found));
    }
    fn walk(ast: &AstModule, v: Visit<'_, AstNoPayload>, found: &mut Option<starlark::Error>) {
        match v {
            Visit::Expr(e) => expr(ast, e, found),
            Visit::Stmt(s) => s.node.visit_children(|v| walk(ast, v, found)),
        }
    }
    let mut found = None;
    ast.statement()
        .node
        .visit_children(|v| walk(ast, v, &mut found));
    found.map_or(Ok(()), Err)
}

/// The first escape in the source text of a string literal that Bazel does
/// not accept: the byte offset it is reported at (the character after the
/// backslash, or the last digit of an octal escape), that character's length and
/// Bazel's message.
fn invalid_escape(literal: &str) -> Option<(usize, usize, String)> {
    if literal.starts_with(['r', 'R']) {
        return None;
    }
    let bytes = literal.as_bytes();
    let mut chars = literal.char_indices();
    while let Some((at, c)) = chars.next() {
        if c != '\\' {
            continue;
        }
        let (next_at, next) = chars.next()?;
        match next {
            '\\' | '\'' | '"' | 'a' | 'b' | 'f' | 'n' | 'r' | 't' | 'v' | '\n' | '\r' => {}
            '0'..='7' => {
                // Up to three octal digits, no more than `\377`.
                let mut value = next as u32 - '0' as u32;
                let mut last = next_at;
                for _ in 0..2 {
                    match bytes.get(last + 1) {
                        Some(d @ b'0'..=b'7') => {
                            value = value * 8 + u32::from(d - b'0');
                            last += 1;
                            chars.next();
                        }
                        _ => break,
                    }
                }
                if value > 0o377 {
                    return Some((
                        last,
                        1,
                        "octal escape sequence out of range (maximum is \\377)".to_owned(),
                    ));
                }
            }
            other => {
                return Some((
                    next_at,
                    other.len_utf8(),
                    format!("invalid escape sequence: \\{other}. Use '\\\\' to insert '\\'."),
                ));
            }
        }
        let _ = at;
    }
    None
}

/// The names a file assigns at its top level (not `def`s or `load`s), in
/// the order it assigns them.
pub(crate) fn assigned_names(ast: &AstModule) -> Vec<String> {
    let stmts: &[AstStmt] = match &ast.statement().node {
        StmtP::Statements(v) => v,
        _ => std::slice::from_ref(ast.statement()),
    };
    let mut names = Vec::new();
    for stmt in stmts {
        if let StmtP::Assign(a) = &stmt.node {
            target_names(&a.lhs.node, &mut names);
        }
    }
    names.into_iter().map(|(name, _)| name.to_owned()).collect()
}

fn error_at(ast: &AstModule, span: Span, msg: String) -> starlark::Error {
    let FileSpan { file, span } = ast.file_span(span);
    starlark::Error::new_spanned(ErrorKind::Parser(anyhow!(msg)), span, &file)
}

/// How a top-level name came to be bound.
#[derive(Clone, Copy, PartialEq)]
enum Binding {
    Load,
    Global,
}

fn target_names<'a>(t: &'a AssignTargetP<AstNoPayload>, out: &mut Vec<(&'a str, Span)>) {
    match t {
        AssignTargetP::Identifier(id) => out.push((id.node.ident.as_str(), id.span)),
        AssignTargetP::Tuple(ts) => ts.iter().for_each(|t| target_names(&t.node, out)),
        AssignTargetP::Index(_) | AssignTargetP::Dot(..) => {}
    }
}

/// Bazel's `.bzl` top-level checks, in the order it reports them.
fn check_bzl_top_level(ast: &AstModule) -> Result<(), starlark::Error> {
    let stmts: &[AstStmt] = match &ast.statement().node {
        StmtP::Statements(v) => v,
        _ => std::slice::from_ref(ast.statement()),
    };

    // Loads first. A bare string literal (a docstring, or any other string
    // expression) does not count as a statement for this purpose.
    let mut seen_non_load = false;
    for stmt in stmts {
        match &stmt.node {
            StmtP::Load(_) if seen_non_load => {
                return Err(error_at(
                    ast,
                    stmt.span,
                    "load statements must appear before any other statement".to_owned(),
                ));
            }
            StmtP::Load(_) => {}
            StmtP::Expression(e) if matches!(&e.node, ExprP::Literal(AstLiteral::String(_))) => {}
            _ => seen_non_load = true,
        }
    }

    // A name is declared once at the top level. A `load()` binding is
    // file-local, so assigning to a loaded name is a different error from
    // declaring a name twice.
    let mut seen: HashMap<&str, Binding> = HashMap::new();
    for stmt in stmts {
        let mut names = Vec::new();
        let how = match &stmt.node {
            StmtP::Load(l) => {
                names.extend(
                    l.args
                        .iter()
                        .map(|a| (a.local.node.ident.as_str(), a.local.span)),
                );
                Binding::Load
            }
            StmtP::Def(d) => {
                names.push((d.name.node.ident.as_str(), d.name.span));
                Binding::Global
            }
            StmtP::Assign(a) => {
                target_names(&a.lhs.node, &mut names);
                Binding::Global
            }
            StmtP::AssignModify(t, _, _) => {
                target_names(&t.node, &mut names);
                Binding::Global
            }
            _ => continue,
        };
        for (name, span) in names {
            let msg = match (seen.insert(name, how), how) {
                (None, _) => continue,
                (Some(Binding::Load), Binding::Global) => {
                    format!("conflicting global declaration of '{name}'")
                }
                (Some(_), _) => format!("'{name}' redeclared at top level"),
            };
            return Err(error_at(ast, span, msg));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refused(src: &str) -> Option<String> {
        parse("t.bzl", src, FileKind::Bzl)
            .err()
            .map(|e| e.to_string())
    }

    #[test]
    fn an_octal_escape_is_at_most_377_and_the_error_is_where_bazel_puts_it() {
        let error = refused("X = \"a\\400 b\"\n").unwrap();
        assert!(
            error.contains("octal escape sequence out of range (maximum is \\377)"),
            "{error}"
        );
        assert!(error.contains("1:10"), "{error}");
        let error = refused("X = \"a\\q b\"\n").unwrap();
        assert!(error.contains("1:8"), "{error}");
        assert_eq!(refused("X = \"\\377\\0\\12\\7\""), None);
    }

    #[test]
    fn bazel_takes_only_its_escapes() {
        for ok in [
            r#"x = "\n\t\\\"\'\a\b\f\v\r\101\0 \7""#,
            "x = 'it\\'s'",
            "x = r'\\q \\u \\x'",
            "x = \"\"\"a\\\nb\"\"\"",
            r#"x = ["a", {"b\n": "c"}]"#,
        ] {
            assert_eq!(refused(ok), None, "{ok}");
        }
        for (src, bad) in [
            (r#"x = "\u0001""#, "\\u"),
            (r#"x = "\U0001F600""#, "\\U"),
            (r#"x = "\x7f""#, "\\x"),
            (r#"x = "\q""#, "\\q"),
            (r#"x = "ok" + "a\ eb""#, "\\ "),
            (r#"def f(): return {"k": ["\8"]}"#, "\\8"),
        ] {
            let error = refused(src).unwrap_or_else(|| panic!("{src} is accepted"));
            assert!(
                error.contains(&format!(
                    "invalid escape sequence: {bad}. Use '\\\\' to insert '\\'."
                )),
                "{src}: {error}"
            );
        }
    }
}
