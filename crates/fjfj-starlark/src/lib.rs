//! Starlark loading phase: evaluates `MODULE.bazel`, `.bzl` and `BUILD` files
//! using the `starlark` crate (the Buck2 implementation), which already
//! matches the Starlark spec and Bazel dialect closely.
//!
//! Bazel builtins (`rule`, `attr`, `aspect`, `provider`, `ctx.actions.*`,
//! `native.*`, `select`, ...) are supplied by this crate as globals.

mod args;
mod attr;
#[cfg(test)]
mod attr_matrix;
#[cfg(test)]
mod attr_tests;
#[cfg(test)]
mod builtins_tests;
#[cfg(test)]
mod conformance;
mod decl;
#[cfg(test)]
mod decl_matrix;
#[cfg(test)]
mod decl_tests;
mod depset;
#[cfg(test)]
mod depset_tests;
mod dialect;
mod exports;
mod ext;
#[cfg(test)]
mod ext_matrix;
#[cfg(test)]
mod ext_tests;
mod floats;
mod instantiate;
mod json;
mod label;
#[cfg(test)]
mod label_tests;
mod load_visibility;
#[cfg(test)]
mod load_visibility_matrix;
#[cfg(test)]
mod load_visibility_tests;
mod loader;
#[cfg(test)]
mod loader_tests;
mod macros;
#[cfg(test)]
mod macros_matrix;
#[cfg(test)]
mod macros_tests;
mod module_ctx;
mod native;
mod proto;
mod provider;
#[cfg(test)]
mod provider_tests;
mod repo_ctx;
#[cfg(test)]
mod repo_ctx_matrix;
#[cfg(test)]
mod repo_ctx_tests;
mod repo_download;
#[cfg(test)]
mod repo_download_matrix;
#[cfg(test)]
mod repo_download_tests;
mod rule;
#[cfg(test)]
mod rule_tests;
mod select;
#[cfg(test)]
mod select_matrix;
#[cfg(test)]
mod select_tests;
mod set;
mod structs;
#[cfg(test)]
mod test_support;

pub use depset::{Depset, DepsetGen, FrozenDepset, Order, depset_to_list, is_depset, new_depset};
pub use dialect::{FileKind, build_dialect, bzl_dialect, parse};
pub use label::{BzlFile, RepoMappings, bzl_name, evaluate_bzl};
pub use load_visibility::{LoadVisibility, check_load_visibility, load_visibility};
pub use loader::{BzlLoader, Importing};
pub use module_ctx::{
    ExtensionInput, ExtensionOutput, GeneratedRepo, ModuleUse, TagUse, TagValue,
    convert_repo_attrs, has_module_extension, has_repository_rule, run_module_extension,
};
pub use native::{
    BuildFile, BuildFileError, BuildFileOutput, build_globals, bzl_globals, evaluate_build_file,
};
pub use repo_ctx::{RepoAttr, RepoEnv, RepoError, repository_rule_defaults, run_repository_rule};
pub use repo_download::{Downloader, HttpRequest};
pub use structs::{FrozenStruct, Struct, StructGen};

use starlark::environment::{FrozenModule, Globals, Module};
use starlark::eval::{Evaluator, FileLoader};

/// Evaluate a BUILD/bzl source string with no `load()` support. Placeholder:
/// returns the module's result value as a string. The real implementation
/// records rule instantiations into a package.
pub fn eval_source(path: &str, src: &str, kind: FileKind) -> anyhow::Result<String> {
    eval_source_with_loader(path, src, kind, &NoLoads)
}

/// [`eval_source`], resolving `load()` statements through `loader`.
pub fn eval_source_with_loader(
    path: &str,
    src: &str,
    kind: FileKind,
    loader: &dyn FileLoader,
) -> anyhow::Result<String> {
    let ast = parse(path, src, kind)?;
    let globals = Globals::standard();
    Module::with_temp_heap(|module| {
        let mut eval = Evaluator::new(&module);
        eval.set_loader(loader);
        let v = eval
            .eval_module(ast, &globals)
            .map_err(|e| e.into_anyhow())?;
        Ok(v.to_string())
    })
}

/// A loader for a file that must not `load()` anything.
struct NoLoads;

impl FileLoader for NoLoads {
    fn load(&self, path: &str) -> starlark::Result<FrozenModule> {
        Err(starlark::Error::new_other(anyhow::anyhow!(
            "cannot load '{path}': no loader"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Parse only: `Ok` if the file is accepted under `kind`'s rules.
    fn accepts(kind: FileKind, src: &str) -> bool {
        parse("t", src, kind).is_ok()
    }

    /// The parse error's message, panicking if the file is accepted.
    fn rejection(kind: FileKind, src: &str) -> String {
        match parse("t", src, kind) {
            Ok(_) => panic!("accepted under {kind:?}:\n{src}"),
            Err(e) => format!("{e:#}"),
        }
    }

    /// Resolves `load()` paths to in-memory `.bzl` sources, evaluating each
    /// under the `.bzl` rules.
    struct MapLoader(HashMap<&'static str, &'static str>);

    impl FileLoader for MapLoader {
        fn load(&self, path: &str) -> starlark::Result<FrozenModule> {
            let src = self.0.get(path).ok_or_else(|| {
                starlark::Error::new_other(anyhow::anyhow!("no such file {path}"))
            })?;
            let ast = parse(path, src, FileKind::Bzl).map_err(starlark::Error::new_other)?;
            Module::with_temp_heap(|module| {
                {
                    let mut eval = Evaluator::new(&module);
                    eval.set_loader(self);
                    eval.eval_module(ast, &Globals::standard())?;
                }
                Ok(module.freeze()?)
            })
        }
    }

    /// Evaluate a BUILD file against the given `.bzl` files.
    fn build_with(files: &[(&'static str, &'static str)], src: &str) -> anyhow::Result<String> {
        let loader = MapLoader(files.iter().copied().collect());
        eval_source_with_loader("BUILD", src, FileKind::Build, &loader)
    }

    #[test]
    fn evaluates_expression() {
        assert_eq!(eval_source("t.bzl", "1 + 2", FileKind::Bzl).unwrap(), "3");
    }

    #[test]
    fn bzl_accepts_what_bazel_accepts() {
        for src in [
            "def f(*, a):\n    return a\n",
            "def f(a, *, b):\n    pass\n",
            "def f(*args, k):\n    pass\n",
            "f = lambda x, *a, **k: x\n",
            "def f(a: int = 1, *, b: str = \"x\") -> None:\n    pass\n",
            "x: int = 1\n",
            "y = [i for i in range(3) if i]\n",
        ] {
            assert!(accepts(FileKind::Bzl, src), "{src}");
        }
    }

    #[test]
    fn bzl_ignores_type_annotations_at_runtime() {
        let src = "def f(a: int = 1) -> str:\n    return a\nf()\n";
        assert_eq!(eval_source("t.bzl", src, FileKind::Bzl).unwrap(), "1");
    }

    #[test]
    fn bzl_rejects_what_bazel_rejects() {
        for (src, why) in [
            ("def f(a, /, b):\n    pass\n", "positional-only parameters"),
            ("if True:\n    x = 1\n", "top-level if"),
            ("for i in [1]:\n    pass\n", "top-level for"),
            ("y = 1\nx = f\"{y}\"\n", "f-strings"),
        ] {
            assert!(!accepts(FileKind::Bzl, src), "{why}");
        }
    }

    #[test]
    fn bzl_loads_come_before_other_statements() {
        let msg = rejection(FileKind::Bzl, "x = 1\nload(\":b.bzl\", \"q\")\n");
        assert!(
            msg.contains("load statements must appear before any other statement"),
            "{msg}"
        );
        // A `def` counts as a statement too.
        rejection(
            FileKind::Bzl,
            "def f():\n    pass\nload(\":b.bzl\", \"q\")\n",
        );
        // String expressions do not: a docstring, or any run of them.
        assert!(accepts(
            FileKind::Bzl,
            "\"\"\"doc\"\"\"\nload(\":b.bzl\", \"q\")\nx = q\n"
        ));
        assert!(accepts(
            FileKind::Bzl,
            "\"a\"\n\"b\"\nload(\":b.bzl\", \"q\")\nx = q\n"
        ));
    }

    #[test]
    fn bzl_declares_each_top_level_name_once() {
        for src in [
            "x = 1\nx = 2\n",
            "x = 1\nx += 1\n",
            "def f(): pass\ndef f(): pass\n",
            "x = 1\ndef x(): pass\n",
            "x, (y, x) = 1, (2, 3)\n",
            "load(\":a.bzl\", \"q\")\nload(\":b.bzl\", \"q\")\n",
        ] {
            let msg = rejection(FileKind::Bzl, src);
            assert!(msg.contains("redeclared at top level"), "{src}: {msg}");
        }
        // Assigning to a loaded name is a different error.
        let msg = rejection(FileKind::Bzl, "load(\":b.bzl\", \"q\")\nq = 2\n");
        assert!(
            msg.contains("conflicting global declaration of 'q'"),
            "{msg}"
        );
        // Element assignment and method calls declare nothing.
        assert!(accepts(FileKind::Bzl, "x = [1]\nx[0] = 2\nx.append(3)\n"));
    }

    #[test]
    fn build_accepts_what_bazel_accepts() {
        for src in [
            // A BUILD file may load after other statements ...
            "x = 1\nload(\":a.bzl\", \"q\")\n",
            // ... rebind a name, loaded or not, ...
            "x = 1\nx = 2\n",
            "load(\":a.bzl\", \"q\")\nq = 2\n",
            // ... load the same name twice, ...
            "load(\":a.bzl\", \"q\")\nload(\":a.bzl\", \"q\")\n",
            // ... and use comprehensions and `if` expressions.
            "x = [i for i in range(3) if i]\ny = 1 if x else 2\n",
        ] {
            assert!(accepts(FileKind::Build, src), "{src}");
        }
    }

    #[test]
    fn build_rejects_what_bazel_rejects() {
        for (src, why) in [
            ("def f():\n    pass\n", "def"),
            ("x = lambda: 1\n", "lambda"),
            ("if True:\n    x = 1\n", "if statement"),
            ("for i in [1]:\n    pass\n", "for statement"),
            ("x: int = 1\n", "type annotation"),
        ] {
            assert!(!accepts(FileKind::Build, src), "{why}");
        }
    }

    #[test]
    fn load_binds_under_an_alias() {
        let files = [(":a.bzl", "x = 41\n")];
        assert_eq!(
            build_with(&files, "load(\":a.bzl\", y = \"x\")\ny + 1\n").unwrap(),
            "42"
        );
    }

    #[test]
    fn load_of_a_private_symbol_is_an_error() {
        let files = [(":a.bzl", "_x = 1\n")];
        for src in [
            "load(\":a.bzl\", \"_x\")\n",
            "load(\":a.bzl\", y = \"_x\")\n",
        ] {
            let msg = format!("{:#}", build_with(&files, src).unwrap_err());
            assert!(msg.contains("private") && msg.contains("_x"), "{msg}");
        }
    }

    #[test]
    fn load_of_a_missing_symbol_is_an_error() {
        let files = [(":a.bzl", "x = 1\n")];
        let msg = format!(
            "{:#}",
            build_with(&files, "load(\":a.bzl\", \"nope\")\n").unwrap_err()
        );
        assert!(msg.contains("nope"), "{msg}");
    }

    #[test]
    fn loaded_symbols_are_not_reexported() {
        // c.bzl loads x from b.bzl; loading x from c.bzl must fail.
        let files = [
            (":b.bzl", "x = 1\n"),
            (":c.bzl", "load(\":b.bzl\", \"x\")\n"),
        ];
        let msg = format!(
            "{:#}",
            build_with(&files, "load(\":c.bzl\", \"x\")\n").unwrap_err()
        );
        assert!(msg.contains("x"), "{msg}");
        // Not even under an underscore-free alias.
        let files = [
            (":b.bzl", "x = 1\n"),
            (":c.bzl", "load(\":b.bzl\", y = \"x\")\n"),
        ];
        build_with(&files, "load(\":c.bzl\", \"y\")\n").unwrap_err();
    }

    #[test]
    fn a_bzl_can_reexport_a_loaded_value_by_assigning_it() {
        let files = [
            (":b.bzl", "x = 1\n"),
            (":c.bzl", "load(\":b.bzl\", _x = \"x\")\nx = _x\n"),
        ];
        assert_eq!(
            build_with(&files, "load(\":c.bzl\", \"x\")\nx\n").unwrap(),
            "1"
        );
    }
}
