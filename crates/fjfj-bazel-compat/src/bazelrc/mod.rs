//! `.bazelrc` lexing, parsing, and resolution: discovery order,
//! `import`/`try-import`, `--config` expansion, `command:config` sections
//! and `common`. See `docs/design/cli-compat.md` for the design decision
//! (Chumsky + Ariadne) and bead `buildfiji-gwl.1`.

pub mod ast;
pub mod diagnostics;
pub mod parse;
pub mod resolve;

pub use ast::{Directive, Line, RcFile};
pub use diagnostics::render_parse_errors;
pub use parse::{ParseError, parse_rc_file};
pub use resolve::{
    DiscoveryOptions, ResolveError, ResolvedLine, discover_and_parse, resolve_command,
};
