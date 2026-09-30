//! Bazel's Starlark dialects and the file-level rules the `starlark` crate
//! does not enforce on its own.
//!
//! Every setting here was checked against Bazel 9.2.0 (buildfiji-mum.2):
//! `Dialect::Standard` is not Bazel's dialect, and the two file kinds differ.

use anyhow::anyhow;
use starlark::codemap::{FileSpan, Span};
use starlark::syntax::{AstModule, Dialect, DialectTypes};
use starlark_syntax::error::ErrorKind;
use starlark_syntax::syntax::ast::{
    AssignTargetP, AstLiteral, AstNoPayload, AstStmt, ExprP, StmtP,
};
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
    if kind == FileKind::Bzl {
        check_bzl_top_level(&ast).map_err(|e| e.into_anyhow())?;
    }
    Ok(ast)
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
