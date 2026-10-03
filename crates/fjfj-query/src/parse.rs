//! The lexer and parser of query expressions. The words it accepts and the
//! texts of its errors are Bazel 9.2.0's.

use crate::ast::{Arg, ArgKind, Call, Dialect, Expr, Function, Op};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Word(String),
    Int(i64),
    Let,
    In,
    Set,
    Equals,
    Plus,
    Caret,
    Minus,
    Open,
    Close,
    Comma,
}

impl Token {
    fn text(&self) -> String {
        match self {
            Token::Word(w) => w.clone(),
            Token::Int(n) => n.to_string(),
            Token::Let => "let".into(),
            Token::In => "in".into(),
            Token::Set => "set".into(),
            Token::Equals => "=".into(),
            Token::Plus => "+".into(),
            Token::Caret => "^".into(),
            Token::Minus => "-".into(),
            Token::Open => "(".into(),
            Token::Close => ")".into(),
            Token::Comma => ",".into(),
        }
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || "*/@.-_:$~[]".contains(c)
}

fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut at = 0;
    while at < chars.len() {
        let c = chars[at];
        match c {
            c if c.is_whitespace() => at += 1,
            '(' => {
                tokens.push(Token::Open);
                at += 1;
            }
            ')' => {
                tokens.push(Token::Close);
                at += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                at += 1;
            }
            '=' => {
                tokens.push(Token::Equals);
                at += 1;
            }
            '+' => {
                tokens.push(Token::Plus);
                at += 1;
            }
            '^' => {
                tokens.push(Token::Caret);
                at += 1;
            }
            // A leading dash is the operator; inside a word it is part of it.
            '-' => {
                tokens.push(Token::Minus);
                at += 1;
            }
            '\'' | '"' => {
                let quote = c;
                let mut word = String::new();
                at += 1;
                loop {
                    match chars.get(at) {
                        None => return Err("unclosed quotation".to_owned()),
                        // A backslash stays, for the regexes in `filter` and
                        // `attr`, unless it protects the quote.
                        Some('\\') if chars.get(at + 1) == Some(&quote) => {
                            word.push(quote);
                            at += 2;
                        }
                        Some(&q) if q == quote => {
                            at += 1;
                            break;
                        }
                        Some(&other) => {
                            word.push(other);
                            at += 1;
                        }
                    }
                }
                tokens.push(Token::Word(word));
            }
            c if is_word_char(c) => {
                let start = at;
                while at < chars.len() && is_word_char(chars[at]) {
                    at += 1;
                }
                let word: String = chars[start..at].iter().collect();
                tokens.push(match word.as_str() {
                    "let" => Token::Let,
                    "in" => Token::In,
                    "set" => Token::Set,
                    "union" => Token::Plus,
                    "intersect" => Token::Caret,
                    "except" => Token::Minus,
                    w if w.chars().all(|c| c.is_ascii_digit()) => {
                        Token::Int(w.parse().map_err(|_| format!("integer too large: '{w}'"))?)
                    }
                    _ => Token::Word(word),
                });
            }
            other => return Err(format!("syntax error at '{other}'")),
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
    dialect: Dialect,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.at).cloned();
        if t.is_some() {
            self.at += 1;
        }
        t
    }

    /// Up to three tokens from the current one, as Bazel shows the place of
    /// a syntax error.
    fn context(&self) -> String {
        self.tokens[self.at.min(self.tokens.len())..]
            .iter()
            .take(3)
            .map(Token::text)
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn premature() -> String {
        "premature end of input".to_owned()
    }

    fn syntax_error(&self) -> String {
        format!("syntax error at '{}'", self.context())
    }

    fn expect(&mut self, want: Token) -> Result<(), String> {
        match self.peek() {
            None => Err(Self::premature()),
            Some(t) if *t == want => {
                self.at += 1;
                Ok(())
            }
            Some(_) => Err(self.syntax_error()),
        }
    }

    fn expression(&mut self) -> Result<Expr, String> {
        let mut left = self.primary()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => Op::Union,
                Some(Token::Caret) => Op::Intersect,
                Some(Token::Minus) => Op::Except,
                _ => return Ok(left),
            };
            self.at += 1;
            let right = self.primary()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
    }

    fn primary(&mut self) -> Result<Expr, String> {
        match self.peek().cloned() {
            None => Err(Self::premature()),
            Some(Token::Word(w)) => {
                // A word followed by `(` is a function.
                if self.tokens.get(self.at + 1) == Some(&Token::Open) {
                    return self.call(&w);
                }
                self.at += 1;
                Ok(match w.strip_prefix('$') {
                    Some(name) if !name.is_empty() => Expr::Variable(name.to_owned()),
                    _ => Expr::Word(w),
                })
            }
            Some(Token::Int(n)) => {
                self.at += 1;
                Ok(Expr::Word(n.to_string()))
            }
            Some(Token::Open) => {
                self.at += 1;
                let inner = self.expression()?;
                self.expect(Token::Close)?;
                Ok(inner)
            }
            Some(Token::Let) => {
                self.at += 1;
                let name = match self.next() {
                    Some(Token::Word(w)) => w,
                    None => return Err(Self::premature()),
                    Some(_) => {
                        self.at -= 1;
                        return Err(self.syntax_error());
                    }
                };
                self.expect(Token::Equals)?;
                let value = self.expression()?;
                self.expect(Token::In)?;
                let body = self.expression()?;
                Ok(Expr::Let {
                    name,
                    value: Box::new(value),
                    body: Box::new(body),
                })
            }
            Some(Token::Set) => {
                self.at += 1;
                self.expect(Token::Open)?;
                let mut words = Vec::new();
                loop {
                    match self.next() {
                        Some(Token::Close) => break,
                        Some(Token::Word(w)) => words.push(w),
                        Some(Token::Int(n)) => words.push(n.to_string()),
                        None => return Err(Self::premature()),
                        Some(_) => {
                            self.at -= 1;
                            return Err(self.syntax_error());
                        }
                    }
                }
                Ok(Expr::Set(words))
            }
            Some(_) => Err(self.syntax_error()),
        }
    }

    fn call(&mut self, name: &str) -> Result<Expr, String> {
        let Some(function) = Function::named(name, self.dialect) else {
            return Err(format!(
                "unknown function '{name}' at '{}'; expected one of [{}]",
                self.context(),
                Function::all(self.dialect)
                    .map(|f| format!("'{}'", f.name()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        };
        self.at += 2;
        let (required, optional) = function.signature();
        let kinds: Vec<ArgKind> = required.iter().chain(optional).copied().collect();
        let mut args: Vec<Arg> = Vec::new();
        if self.peek() != Some(&Token::Close) {
            loop {
                let kind = kinds[args.len()];
                args.push(self.argument(kind)?);
                if self.peek() != Some(&Token::Comma) {
                    break;
                }
                if args.len() == kinds.len() {
                    return Err(format!(
                        "too many arguments to function '{name}' at '{}'",
                        self.context()
                    ));
                }
                self.at += 1;
            }
        }
        if args.len() < required.len() && self.peek() == Some(&Token::Close) {
            return Err(format!(
                "too few arguments to function '{name}' at '{}'",
                self.context()
            ));
        }
        self.expect(Token::Close)?;
        Ok(Expr::Call(Call { function, args }))
    }

    fn argument(&mut self, kind: ArgKind) -> Result<Arg, String> {
        match kind {
            ArgKind::Expr => Ok(Arg::Expr(self.expression()?)),
            ArgKind::Word => match self.next() {
                Some(Token::Word(w)) => Ok(Arg::Word(w)),
                Some(Token::Int(n)) => Ok(Arg::Word(n.to_string())),
                None => Err(Self::premature()),
                Some(_) => {
                    self.at -= 1;
                    Err(self.syntax_error())
                }
            },
            ArgKind::Int => match self.next() {
                Some(Token::Int(n)) => Ok(Arg::Int(n)),
                Some(Token::Word(w)) => Err(format!("expected an integer literal: '{w}'")),
                None => Err(Self::premature()),
                Some(_) => {
                    self.at -= 1;
                    Err(self.syntax_error())
                }
            },
        }
    }
}

/// Parse `input`. The error is the text after `Error while parsing '<input>': `.
pub fn parse(input: &str) -> Result<Expr, String> {
    parse_in(input, Dialect::Query)
}

/// Parse `input` as an expression of `dialect`'s command.
pub fn parse_in(input: &str, dialect: Dialect) -> Result<Expr, String> {
    let tokens = lex(input)?;
    let mut parser = Parser {
        tokens,
        at: 0,
        dialect,
    };
    let expr = parser.expression()?;
    if let Some(extra) = parser.peek() {
        return Err(format!(
            "unexpected token '{}' after query expression '{expr}'",
            extra.text()
        ));
    }
    Ok(expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err(input: &str) -> String {
        parse(input).unwrap_err()
    }

    /// Each error text was produced by `bazel query` 9.2.0.
    #[test]
    fn the_wrong_number_of_arguments_is_bazels_error() {
        assert_eq!(err("deps()"), "too few arguments to function 'deps' at ')'");
        assert_eq!(
            err("deps(//a, 1, 2)"),
            "too many arguments to function 'deps' at ', 2 )'"
        );
        assert_eq!(
            err("kind(r)"),
            "too few arguments to function 'kind' at ')'"
        );
    }

    #[test]
    fn cquery_adds_config_to_the_functions() {
        assert!(parse("config(//a, target)").is_err());
        let e = parse_in("config(//a, target)", Dialect::Cquery).unwrap();
        assert_eq!(e.to_string(), "config(//a, target)");
        let list = parse_in("foo(//a)", Dialect::Cquery).unwrap_err();
        assert!(list.contains("'buildfiles', 'config', 'deps'"), "{list}");
        assert!(!list.contains("'inputs'"), "{list}");
    }

    #[test]
    fn aquery_adds_the_action_filters_to_the_functions() {
        assert!(parse("inputs(x, //a)").is_err());
        let e = parse_in("inputs(x, //a)", Dialect::Aquery).unwrap();
        assert_eq!(e.to_string(), "inputs(x, //a)");
        let list = parse_in("foo(//a)", Dialect::Aquery).unwrap_err();
        assert!(
            list.ends_with("'filter', 'inputs', 'kind', 'labels', 'loadfiles', 'mnemonic', 'outputs', 'rdeps', 'same_pkg_direct_rdeps', 'siblings', 'some', 'somepath', 'tests', 'visible']"),
            "{list}"
        );
    }

    #[test]
    fn errors_read_as_bazels_do() {
        assert_eq!(err("deps("), "premature end of input");
        assert_eq!(err("deps(//a:lib,"), "premature end of input");
        assert_eq!(err("//a:lib +"), "premature end of input");
        assert_eq!(err(""), "premature end of input");
        assert_eq!(
            err("//a:lib //a:gen"),
            "unexpected token '//a:gen' after query expression '//a:lib'"
        );
        assert_eq!(err("deps(//a:lib, x)"), "expected an integer literal: 'x'");
        assert_eq!(err("deps(//a:lib, -1)"), "syntax error at '- 1 )'");
        assert!(err("foo(").starts_with("unknown function 'foo' at 'foo ('; expected one of ['allpaths', 'attr', 'buildfiles', 'deps', 'executables', 'filter', 'kind', 'labels', 'loadfiles', 'rdeps', 'same_pkg_direct_rdeps', 'siblings', 'some', 'somepath', 'tests', 'visible']"));
        assert!(err("rbuildfiles(//rules.bzl)").starts_with(
            "unknown function 'rbuildfiles' at 'rbuildfiles ( //rules.bzl'; expected one of"
        ));
    }

    #[test]
    fn operators_have_equal_precedence_and_associate_left() {
        let e = parse("//a:lib + //a:gen ^ //a:gen").unwrap();
        assert_eq!(e.to_string(), "//a:lib + //a:gen ^ //a:gen");
        let Expr::Binary(Op::Intersect, left, _) = e else {
            panic!("{e:?}")
        };
        assert!(matches!(*left, Expr::Binary(Op::Union, _, _)));
    }

    #[test]
    fn words_quotes_keywords_and_functions_parse() {
        assert_eq!(parse("'//a:lib'").unwrap(), Expr::Word("//a:lib".into()));
        assert!(matches!(
            parse("\"//a:lib\" union \"//a:gen\"").unwrap(),
            Expr::Binary(Op::Union, _, _)
        ));
        assert_eq!(
            parse("set(//a:lib //a:gen)").unwrap(),
            Expr::Set(vec!["//a:lib".into(), "//a:gen".into()])
        );
        assert!(matches!(
            parse("let x = //a:lib in $x + $x").unwrap(),
            Expr::Let { .. }
        ));
        let Expr::Call(call) = parse("deps(//a:lib, 1)").unwrap() else {
            panic!()
        };
        assert_eq!(call.function, Function::Deps);
        assert_eq!(call.args[1], Arg::Int(1));
        assert!(matches!(
            parse("//a:all - //a:lib").unwrap(),
            Expr::Binary(Op::Except, _, _)
        ));
        // A dash inside a word stays in the word.
        assert_eq!(parse("//a:x-y").unwrap(), Expr::Word("//a:x-y".into()));
    }
}
