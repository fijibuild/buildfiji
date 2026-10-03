//! The query language (buildfiji-9s8.1): `bazel query`'s expressions parsed
//! and evaluated over a graph of targets.
//!
//! Every behaviour here was checked against Bazel 9.2.0 on a scratch
//! workspace; the tests carry the probed outputs. The parser is written by
//! hand because the lexer's word rules and the error texts are Bazel's own
//! and a combinator library would not give either.

pub mod ast;
pub mod eval;
pub mod graph;
pub mod output;
pub mod parse;
pub mod proto;
pub mod target_proto;

pub use ast::Dialect;
pub use ast::Expr;
pub use eval::{Evaluator, Options};
pub use graph::{Edge, Frame, Graph, Node, NodeAttr, NodeKind};
pub use parse::{parse, parse_in};
