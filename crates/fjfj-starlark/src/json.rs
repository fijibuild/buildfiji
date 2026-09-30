//! The `json` module: `encode`, `decode`, `encode_indent` and `indent`
//! (buildfiji-mum.3.1).
//!
//! Bazel's encoder and decoder are its own, and every message here was read
//! off Bazel 9.2.0:
//!
//! - `encode` writes keys and struct fields sorted, has no whitespace, writes
//!   only control characters below U+0020 as escapes, and names the path to a
//!   value it cannot encode (`in struct field .a: at list index 0: ...`);
//! - `decode` reports byte offsets, accepts any value as a candidate object
//!   key and then complains that it is not a string, keeps the last of a
//!   repeated key, and turns a lone UTF-16 surrogate into U+FFFD;
//! - `indent` is a lenient reformatter rather than a validator: it only
//!   understands enough JSON to place newlines, stops after the first
//!   complete value, and copies `true`, `false` and `null` by length.
//!   `encode_indent` is `indent(encode(x))`, so a bare number, which `indent`
//!   refuses, is refused there too.

use crate::args::{Wording, bind, fatal, param, positional_only};
use crate::floats::format_float;
use crate::structs::fields_of;
use allocative::Allocative;
use num_bigint::BigInt;
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_module;
use starlark::starlark_simple_value;
use starlark::values::dict::DictRef;
use starlark::values::float::StarlarkFloat;
use starlark::values::{Heap, NoSerialize, ProvidesStaticType, StarlarkValue, Value, ValueLike};
use starlark_derive::starlark_value;
use std::fmt;

#[derive(Debug, ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct JsonModule;

starlark_simple_value!(JsonModule);

impl fmt::Display for JsonModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "json")
    }
}

#[starlark_value(type = "json")]
impl<'v> StarlarkValue<'v> for JsonModule {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("json", json_methods);
        Some(RES.methods())
    }
}

#[starlark_module]
fn json_methods(builder: &mut MethodsBuilder) {
    /// `json.encode(x)`.
    fn encode<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<String> {
        let _ = this;
        let bound = bind(
            "encode",
            Wording::Signature,
            &[positional_only("x", true)],
            args,
            eval,
        )?;
        encode(bound[0].expect("required"), eval.heap())
    }

    /// `json.decode(x, default = <error>)`.
    fn decode<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let _ = this;
        let bound = bind(
            "decode",
            Wording::Signature,
            &[positional_only("x", true), param("default", true, false)],
            args,
            eval,
        )?;
        let text = string_arg("decode", "x", bound[0].expect("required"))?;
        match decode(text, eval.heap()) {
            Ok(v) => Ok(v),
            Err(_) if bound[1].is_some() => Ok(bound[1].expect("checked")),
            Err(e) => Err(fatal(e)),
        }
    }

    /// `json.encode_indent(x, *, prefix = "", indent = "\t")`.
    fn encode_indent<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<String> {
        let _ = this;
        let bound = bind(
            "encode_indent",
            Wording::Signature,
            &[
                positional_only("x", true),
                param("prefix", false, false),
                param("indent", false, false),
            ],
            args,
            eval,
        )?;
        let prefix = optional_string("encode_indent", "prefix", bound[1], "")?;
        let indent_with = optional_string("encode_indent", "indent", bound[2], "\t")?;
        let encoded = encode(bound[0].expect("required"), eval.heap())?;
        indent(&encoded, prefix, indent_with).map_err(fatal)
    }

    /// `json.indent(s, *, prefix = "", indent = "\t")`.
    fn indent<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<String> {
        let _ = this;
        let bound = bind(
            "indent",
            Wording::Signature,
            &[
                positional_only("s", true),
                param("prefix", false, false),
                param("indent", false, false),
            ],
            args,
            eval,
        )?;
        let text = string_arg("indent", "s", bound[0].expect("required"))?;
        let prefix = optional_string("indent", "prefix", bound[1], "")?;
        let indent_with = optional_string("indent", "indent", bound[2], "\t")?;
        indent(text, prefix, indent_with).map_err(fatal)
    }
}

fn string_arg<'v>(function: &str, param: &str, value: Value<'v>) -> starlark::Result<&'v str> {
    value.unpack_str().ok_or_else(|| {
        fatal(format!(
            "in call to {function}(), parameter '{param}' got value of type '{}', want 'string'",
            value.get_type()
        ))
    })
}

fn optional_string<'v>(
    function: &str,
    param: &str,
    value: Option<Value<'v>>,
    default: &'static str,
) -> starlark::Result<&'v str> {
    match value {
        Some(v) => string_arg(function, param, v),
        None => Ok(default),
    }
}

// ---------------------------------------------------------------- encode

fn encode<'v>(value: Value<'v>, heap: Heap<'v>) -> starlark::Result<String> {
    let mut out = String::new();
    encode_into(value, heap, &mut out).map_err(fatal)?;
    Ok(out)
}

fn encode_into<'v>(value: Value<'v>, heap: Heap<'v>, out: &mut String) -> Result<(), String> {
    // Nesting is as deep as the value; grow the stack rather than cap it.
    stacker::maybe_grow(256 * 1024, 4 * 1024 * 1024, || {
        encode_value(value, heap, out)
    })
}

fn encode_value<'v>(value: Value<'v>, heap: Heap<'v>, out: &mut String) -> Result<(), String> {
    if value.is_none() {
        out.push_str("null");
    } else if let Some(b) = value.unpack_bool() {
        out.push_str(if b { "true" } else { "false" });
    } else if let Some(s) = value.unpack_str() {
        write_string(s, out);
    } else if value.get_type() == "int" {
        out.push_str(&value.to_repr());
    } else if let Some(f) = value.downcast_ref::<StarlarkFloat>() {
        if !f.0.is_finite() {
            return Err(format!(
                "cannot encode non-finite float {}",
                if f.0.is_nan() {
                    "nan"
                } else if f.0 > 0.0 {
                    "+inf"
                } else {
                    "-inf"
                }
            ));
        }
        out.push_str(&format_float(f.0));
    } else if let Some(dict) = DictRef::from_value(value) {
        let mut entries: Vec<(&str, Value<'v>)> = Vec::new();
        for (k, v) in dict.iter() {
            let key = k
                .unpack_str()
                .ok_or_else(|| format!("dict has {} key, want string", k.get_type()))?;
            entries.push((key, v));
        }
        entries.sort_by(|a, b| a.0.cmp(b.0));
        out.push('{');
        for (i, (key, v)) in entries.into_iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            write_string(key, out);
            out.push(':');
            encode_into(v, heap, out).map_err(|e| format!("in dict key \"{key}\": {e}"))?;
        }
        out.push('}');
    } else if let Some(fields) = fields_of(value) {
        out.push('{');
        for (i, (name, v)) in fields.into_iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            write_string(name, out);
            out.push(':');
            encode_into(v, heap, out).map_err(|e| format!("in struct field .{name}: {e}"))?;
        }
        out.push('}');
    } else if matches!(value.get_type(), "list" | "tuple" | "range" | "set") {
        out.push('[');
        let items = value.iterate(heap).map_err(|e| e.to_string())?;
        for (i, v) in items.enumerate() {
            if i > 0 {
                out.push(',');
            }
            encode_into(v, heap, out).map_err(|e| format!("at list index {i}: {e}"))?;
        }
        out.push(']');
    } else {
        return Err(format!("cannot encode {} as JSON", value.get_type()));
    }
    Ok(())
}

fn write_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

// ---------------------------------------------------------------- decode

struct Decoder<'a, 'v> {
    text: &'a str,
    at: usize,
    heap: Heap<'v>,
}

fn decode<'v>(text: &str, heap: Heap<'v>) -> Result<Value<'v>, String> {
    let mut d = Decoder { text, at: 0, heap };
    d.skip_space();
    let value = d.value()?;
    d.skip_space();
    if let Some(c) = d.peek() {
        return Err(format!(
            "at offset {}, unexpected character \"{c}\" after value",
            d.at
        ));
    }
    Ok(value)
}

impl<'a, 'v> Decoder<'a, 'v> {
    fn peek(&self) -> Option<char> {
        self.text[self.at..].chars().next()
    }

    fn skip_space(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.at += 1;
        }
    }

    fn eof(&self) -> String {
        format!("at offset {}, unexpected end of file", self.at)
    }

    fn value(&mut self) -> Result<Value<'v>, String> {
        stacker::maybe_grow(256 * 1024, 4 * 1024 * 1024, || self.value_here())
    }

    fn value_here(&mut self) -> Result<Value<'v>, String> {
        let Some(c) = self.peek() else {
            return Err(self.eof());
        };
        match c {
            '{' => self.object(),
            '[' => self.array(),
            '"' => {
                let s = self.string()?;
                Ok(self.heap.alloc_str(&s).to_value())
            }
            't' => self.literal("true", Value::new_bool(true)),
            'f' => self.literal("false", Value::new_bool(false)),
            'n' => self.literal("null", Value::new_none()),
            '-' | '0'..='9' => self.number(),
            c => Err(format!(
                "at offset {}, unexpected character \"{c}\"",
                self.at
            )),
        }
    }

    fn literal(&mut self, word: &str, value: Value<'v>) -> Result<Value<'v>, String> {
        if self.text[self.at..].starts_with(word) {
            self.at += word.len();
            Ok(value)
        } else {
            Err(format!(
                "at offset {}, unexpected character \"{}\"",
                self.at,
                self.peek().expect("a literal starts with a character")
            ))
        }
    }

    fn number(&mut self) -> Result<Value<'v>, String> {
        let start = self.at;
        let bytes = self.text.as_bytes();
        let mut end = start;
        while end < bytes.len()
            && matches!(bytes[end], b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
        {
            end += 1;
        }
        let token = &self.text[start..end];
        let invalid_at = |at: usize| format!("at offset {at}, invalid number: {token}");
        // The shapes Bazel refuses before it tries to convert the token.
        let digits = token.strip_prefix('-').unwrap_or(token).as_bytes();
        if !digits.first().is_some_and(u8::is_ascii_digit)
            || (digits[0] == b'0' && digits.get(1).is_some_and(u8::is_ascii_digit))
        {
            return Err(invalid_at(start));
        }
        let int_len = digits.iter().take_while(|b| b.is_ascii_digit()).count();
        if digits.get(int_len) == Some(&b'.')
            && !digits.get(int_len + 1).is_some_and(u8::is_ascii_digit)
        {
            return Err(invalid_at(start));
        }
        self.at = end;
        if token.contains(['.', 'e', 'E']) {
            let f: f64 = token.parse().map_err(|_| invalid_at(end))?;
            Ok(self.heap.alloc(f))
        } else {
            let n: BigInt = token.parse().map_err(|_| invalid_at(end))?;
            Ok(self.heap.alloc(n))
        }
    }

    fn string(&mut self) -> Result<String, String> {
        // The opening quote.
        self.at += 1;
        let mut units: Vec<u16> = Vec::new();
        loop {
            let Some(c) = self.peek() else {
                return Err(format!(
                    "at offset {}, unclosed string literal",
                    self.text.len()
                ));
            };
            match c {
                '"' => {
                    self.at += 1;
                    return Ok(String::from_utf16_lossy(&units));
                }
                '\\' => {
                    self.at += 1;
                    let Some(e) = self.peek() else {
                        return Err(format!("at offset {}, incomplete escape", self.at));
                    };
                    self.at += e.len_utf8();
                    let simple = match e {
                        '"' => Some('"'),
                        '\\' => Some('\\'),
                        '/' => Some('/'),
                        'b' => Some('\u{8}'),
                        'f' => Some('\u{c}'),
                        'n' => Some('\n'),
                        'r' => Some('\r'),
                        't' => Some('\t'),
                        _ => None,
                    };
                    if let Some(ch) = simple {
                        let mut buf = [0u16; 2];
                        units.extend_from_slice(ch.encode_utf16(&mut buf));
                    } else if e == 'u' {
                        let hex = &self.text[self.at..];
                        if hex.len() < 4 || !hex.is_char_boundary(4) {
                            return Err(format!(
                                "at offset {}, incomplete \\uXXXX escape",
                                self.at
                            ));
                        }
                        let mut unit = 0u16;
                        for h in hex[..4].chars() {
                            let Some(digit) = h.to_digit(16) else {
                                return Err(format!(
                                    "at offset {}, invalid hex char \"{h}\" in \\uXXXX escape",
                                    self.at
                                ));
                            };
                            unit = unit * 16 + digit as u16;
                        }
                        self.at += 4;
                        units.push(unit);
                    } else {
                        return Err(format!("at offset {}, invalid escape '\\{e}'", self.at));
                    }
                }
                c if (c as u32) < 0x20 => {
                    return Err(format!(
                        "at offset {}, invalid character '\\x{:02x}' in string literal",
                        self.at, c as u32
                    ));
                }
                c => {
                    let mut buf = [0u16; 2];
                    units.extend_from_slice(c.encode_utf16(&mut buf));
                    self.at += c.len_utf8();
                }
            }
        }
    }

    fn array(&mut self) -> Result<Value<'v>, String> {
        self.at += 1;
        let mut items = Vec::new();
        self.skip_space();
        if self.peek() == Some(']') {
            self.at += 1;
            return Ok(self.heap.alloc(items));
        }
        loop {
            self.skip_space();
            items.push(self.value()?);
            self.skip_space();
            match self.peek() {
                Some(',') => self.at += 1,
                Some(']') => {
                    self.at += 1;
                    return Ok(self.heap.alloc(items));
                }
                Some(c) => {
                    return Err(format!(
                        "at offset {}, got \"{c}\", want ',' or ']'",
                        self.at
                    ));
                }
                None => return Err(self.eof()),
            }
        }
    }

    fn object(&mut self) -> Result<Value<'v>, String> {
        self.at += 1;
        let mut entries: Vec<(String, Value<'v>)> = Vec::new();
        self.skip_space();
        if self.peek() == Some('}') {
            self.at += 1;
            return Ok(self.heap.alloc(starlark::values::dict::AllocDict(entries)));
        }
        loop {
            self.skip_space();
            let key = self.value()?;
            let Some(key) = key.unpack_str() else {
                return Err(format!(
                    "at offset {}, got {} for object key, want string",
                    self.at,
                    key.get_type()
                ));
            };
            let key = key.to_owned();
            self.skip_space();
            match self.peek() {
                Some(':') => self.at += 1,
                Some(c) => {
                    return Err(format!(
                        "at offset {}, after object key, got \"{c}\", want ':' ",
                        self.at
                    ));
                }
                None => return Err(self.eof()),
            }
            self.skip_space();
            let value = self.value()?;
            // A repeated key keeps its first place and its last value.
            match entries.iter_mut().find(|(k, _)| *k == key) {
                Some(entry) => entry.1 = value,
                None => entries.push((key, value)),
            }
            self.skip_space();
            match self.peek() {
                Some(',') => self.at += 1,
                Some('}') => {
                    self.at += 1;
                    return Ok(self.heap.alloc(starlark::values::dict::AllocDict(entries)));
                }
                Some(c) => {
                    return Err(format!(
                        "at offset {}, in object, got \"{c}\", want ',' or '}}'",
                        self.at
                    ));
                }
                None => return Err(self.eof()),
            }
        }
    }
}

// ---------------------------------------------------------------- indent

/// Bazel's `json.indent`. It places newlines by the brackets, commas and
/// colons it walks past and stops when the first value is complete, which
/// is why it accepts input that is not JSON.
fn indent(text: &str, prefix: &str, unit: &str) -> Result<String, String> {
    let c: Vec<char> = text.chars().collect();
    let n = c.len();
    let mut out = String::new();
    let mut depth: isize = 0;
    let mut i = 0;
    let newline = |out: &mut String, depth: isize| {
        out.push('\n');
        out.push_str(prefix);
        for _ in 0..depth.max(0) {
            out.push_str(unit);
        }
    };
    let skip_space = |i: &mut usize| {
        while *i < n && matches!(c[*i], ' ' | '\t' | '\n' | '\r') {
            *i += 1;
        }
    };
    let invalid = || "input is not valid JSON".to_owned();
    skip_space(&mut i);
    if i >= n {
        return Err("unexpected end of file".to_owned());
    }
    loop {
        let ch = c[i];
        match ch {
            '"' => {
                out.push('"');
                i += 1;
                loop {
                    let Some(&s) = c.get(i) else {
                        return Err(invalid());
                    };
                    out.push(s);
                    i += 1;
                    if s == '\\' {
                        let Some(&e) = c.get(i) else {
                            return Err(invalid());
                        };
                        out.push(e);
                        i += 1;
                    } else if s == '"' {
                        break;
                    }
                }
            }
            '{' | '[' => {
                out.push(ch);
                i += 1;
                depth += 1;
                let mut j = i;
                skip_space(&mut j);
                if j < n && matches!(c[j], '}' | ']') {
                    out.push(c[j]);
                    i = j + 1;
                    depth -= 1;
                } else {
                    newline(&mut out, depth);
                }
            }
            '}' | ']' => {
                depth -= 1;
                newline(&mut out, depth);
                out.push(ch);
                i += 1;
            }
            ',' => {
                out.push(',');
                newline(&mut out, depth);
                i += 1;
            }
            ':' => {
                out.push_str(": ");
                i += 1;
            }
            't' | 'n' | 'f' => {
                let len = if ch == 'f' { 5 } else { 4 };
                if i + len > n {
                    return Err("unexpected end of file".to_owned());
                }
                out.extend(&c[i..i + len]);
                i += len;
            }
            '-' | '0'..='9' => {
                while i < n && matches!(c[i], '0'..='9' | '.' | 'e' | 'E' | '+' | '-') {
                    out.push(c[i]);
                    i += 1;
                }
                if i >= n {
                    return Err(invalid());
                }
            }
            other => return Err(format!("unexpected character \"{other}\"")),
        }
        if depth <= 0 {
            return Ok(out);
        }
        skip_space(&mut i);
        if i >= n {
            return Err("unexpected end of file".to_owned());
        }
    }
}
