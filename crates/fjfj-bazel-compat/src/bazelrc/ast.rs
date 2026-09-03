//! Parsed representation of a single `.bazelrc` file, before discovery-order
//! merging or `import`/`--config` expansion (see `resolve`).

use std::ops::Range;

/// A `.bazelrc` line, after shell-word splitting. Blank lines and full-line
/// `#` comments never produce a `Directive` (the lexer drops them).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Directive {
    /// `import <path>`: the target file must exist.
    Import { path: String },
    /// `try-import <path>`: a missing file is silently skipped.
    TryImport { path: String },
    /// `<command>[:<config>] <flag> <flag> ...`, e.g.
    /// `build:asan --copt=-fsanitize=address`. `command == "common"` applies
    /// the flags to every command; `config` is `None` for a plain line.
    CommandFlags {
        command: String,
        config: Option<String>,
        flags: Vec<String>,
    },
}

/// One logical `.bazelrc` line (after splicing `\`-continuations) with its
/// directive and source span, for diagnostics and for `resolve` to report
/// which file/line a flag came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub directive: Directive,
    /// 1-based physical line number the logical line started on.
    pub line_no: usize,
    /// Byte span into the original (unspliced) source text.
    pub span: Range<usize>,
}

/// A parsed `.bazelrc` file: its lines in source order, `import`/
/// `try-import` directives not yet expanded.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RcFile {
    pub lines: Vec<Line>,
}
