//! The syntax of a query expression.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    /// A target pattern, as written (quotes removed).
    Word(String),
    /// `$name`.
    Variable(String),
    /// `set(a b c)`: the patterns it names.
    Set(Vec<String>),
    /// `e1 + e2`, `e1 ^ e2`, `e1 - e2`.
    Binary(Op, Box<Expr>, Box<Expr>),
    /// `let name = value in body`.
    Let {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    /// `deps(x, 2)`.
    Call(Call),
}

impl Expr {
    /// The target patterns the expression names, in the order written: what
    /// a query over analysed targets must analyse before it can run.
    pub fn patterns(&self) -> Vec<&str> {
        let mut out = Vec::new();
        self.collect_patterns(&mut out);
        out
    }

    fn collect_patterns<'a>(&'a self, out: &mut Vec<&'a str>) {
        match self {
            Expr::Word(w) => out.push(w),
            Expr::Variable(_) => {}
            Expr::Set(words) => out.extend(words.iter().map(String::as_str)),
            Expr::Binary(_, l, r) => {
                l.collect_patterns(out);
                r.collect_patterns(out);
            }
            Expr::Let { value, body, .. } => {
                value.collect_patterns(out);
                body.collect_patterns(out);
            }
            Expr::Call(call) => {
                for arg in &call.args {
                    if let Arg::Expr(e) = arg {
                        e.collect_patterns(out);
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Union,
    Intersect,
    Except,
}

/// A function of the query language and its arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub function: Function,
    pub args: Vec<Arg>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arg {
    Expr(Expr),
    Word(String),
    Int(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Function {
    AllPaths,
    Attr,
    BuildFiles,
    Deps,
    Executables,
    Filter,
    Kind,
    Labels,
    LoadFiles,
    RDeps,
    SamePkgDirectRDeps,
    Siblings,
    Some,
    SomePath,
    Tests,
    Visible,
}

/// The shape of an argument of a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgKind {
    Expr,
    Word,
    Int,
}

impl Function {
    /// Every function, in the alphabetical order Bazel lists them.
    pub const ALL: [Function; 16] = [
        Function::AllPaths,
        Function::Attr,
        Function::BuildFiles,
        Function::Deps,
        Function::Executables,
        Function::Filter,
        Function::Kind,
        Function::Labels,
        Function::LoadFiles,
        Function::RDeps,
        Function::SamePkgDirectRDeps,
        Function::Siblings,
        Function::Some,
        Function::SomePath,
        Function::Tests,
        Function::Visible,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Function::AllPaths => "allpaths",
            Function::Attr => "attr",
            Function::BuildFiles => "buildfiles",
            Function::Deps => "deps",
            Function::Executables => "executables",
            Function::Filter => "filter",
            Function::Kind => "kind",
            Function::Labels => "labels",
            Function::LoadFiles => "loadfiles",
            Function::RDeps => "rdeps",
            Function::SamePkgDirectRDeps => "same_pkg_direct_rdeps",
            Function::Siblings => "siblings",
            Function::Some => "some",
            Function::SomePath => "somepath",
            Function::Tests => "tests",
            Function::Visible => "visible",
        }
    }

    pub fn named(name: &str) -> Option<Function> {
        Function::ALL.into_iter().find(|f| f.name() == name)
    }

    /// The arguments it must have, and those it may add.
    pub fn signature(self) -> (&'static [ArgKind], &'static [ArgKind]) {
        use ArgKind::{Expr, Int, Word};
        match self {
            Function::AllPaths | Function::SomePath => (&[Expr, Expr], &[]),
            Function::Attr => (&[Word, Word, Expr], &[]),
            Function::BuildFiles
            | Function::Executables
            | Function::LoadFiles
            | Function::SamePkgDirectRDeps
            | Function::Siblings
            | Function::Some
            | Function::Tests => (&[Expr], &[]),
            Function::Deps => (&[Expr], &[Int]),
            Function::Filter | Function::Kind | Function::Labels => (&[Word, Expr], &[]),
            Function::RDeps => (&[Expr, Expr], &[Int]),
            Function::Visible => (&[Expr, Expr], &[]),
        }
    }
}

fn quote_if_needed(word: &str) -> String {
    if word
        .chars()
        .all(|c| c.is_alphanumeric() || "*/@.-_:$~[]".contains(c))
        && !word.is_empty()
    {
        word.to_owned()
    } else {
        format!("\"{word}\"")
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Word(w) => write!(f, "{}", quote_if_needed(w)),
            Expr::Variable(v) => write!(f, "${v}"),
            Expr::Set(words) => {
                write!(f, "set(")?;
                for (i, w) in words.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", quote_if_needed(w))?;
                }
                write!(f, ")")
            }
            Expr::Binary(op, l, r) => {
                let op = match op {
                    Op::Union => "+",
                    Op::Intersect => "^",
                    Op::Except => "-",
                };
                write!(f, "{l} {op} {r}")
            }
            Expr::Let { name, value, body } => write!(f, "let {name} = {value} in {body}"),
            Expr::Call(call) => {
                write!(f, "{}(", call.function.name())?;
                for (i, a) in call.args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    match a {
                        Arg::Expr(e) => write!(f, "{e}")?,
                        Arg::Word(w) => write!(f, "{}", quote_if_needed(w))?,
                        Arg::Int(n) => write!(f, "{n}")?,
                    }
                }
                write!(f, ")")
            }
        }
    }
}
