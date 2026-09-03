//! Starlark loading phase: evaluates `MODULE.bazel`, `.bzl` and `BUILD` files
//! using the `starlark` crate (the Buck2 implementation), which already
//! matches the Starlark spec and Bazel dialect closely.
//!
//! Bazel builtins (`rule`, `attr`, `aspect`, `provider`, `ctx.actions.*`,
//! `native.*`, `select`, ...) are supplied by this crate as globals.

use starlark::environment::{Globals, Module};
use starlark::eval::Evaluator;
use starlark::syntax::{AstModule, Dialect};

/// Bazel's Starlark dialect settings for BUILD files (no `def`/`lambda`).
pub fn build_dialect() -> Dialect {
    Dialect {
        enable_def: false,
        enable_lambda: false,
        ..Dialect::Standard
    }
}

/// Dialect for `.bzl` files.
pub fn bzl_dialect() -> Dialect {
    Dialect::Standard
}

/// Evaluate a BUILD/bzl source string. Placeholder: returns the module's
/// result value as a string. The real implementation records rule
/// instantiations into a package.
pub fn eval_source(path: &str, src: &str, dialect: Dialect) -> anyhow::Result<String> {
    let ast = AstModule::parse(path, src.to_owned(), &dialect).map_err(|e| e.into_anyhow())?;
    let globals = Globals::standard();
    Module::with_temp_heap(|module| {
        let mut eval = Evaluator::new(&module);
        let v = eval
            .eval_module(ast, &globals)
            .map_err(|e| e.into_anyhow())?;
        Ok(v.to_string())
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn evaluates_expression() {
        assert_eq!(
            super::eval_source("t.bzl", "1 + 2", super::bzl_dialect()).unwrap(),
            "3"
        );
    }
}
