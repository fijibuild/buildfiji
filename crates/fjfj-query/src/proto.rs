//! Protocol buffer output of `query` and `cquery` (buildfiji-tle.4): the
//! messages of Bazel's `build.proto` and `analysis_v2.proto` written as
//! binary, as text and as JSON, without a generated type for each.
//!
//! A [`Msg`] is the fields that were set, each with its number and name. The
//! writers sort them by number, as the protobuf libraries do, and spell them as
//! Bazel's Java ones do: the text format with a space before a nested message's
//! brace and two spaces of indent, JSON with a repeated message field as
//! `[{` ... `}, {` ... `}]`, a repeated scalar field on one line, and
//! `< > & = '` written as `\u00XX`. Everything here was compared with the output
//! of Bazel 9.2.0.

use std::fmt::Write as _;

/// A value of a field.
#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    /// An `int32`, `uint32` or `int64`.
    Int(i64),
    Bool(bool),
    Str(String),
    /// An enum value: its name, and its number.
    Enum(&'static str, i32),
    Msg(Msg),
}

impl From<&str> for Val {
    fn from(s: &str) -> Val {
        Val::Str(s.to_owned())
    }
}

impl From<String> for Val {
    fn from(s: String) -> Val {
        Val::Str(s)
    }
}

impl From<bool> for Val {
    fn from(b: bool) -> Val {
        Val::Bool(b)
    }
}

impl From<i64> for Val {
    fn from(i: i64) -> Val {
        Val::Int(i)
    }
}

impl From<Msg> for Val {
    fn from(m: Msg) -> Val {
        Val::Msg(m)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Field {
    number: u32,
    name: &'static str,
    repeated: bool,
    value: Val,
}

/// A message: the fields that are set.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Msg {
    fields: Vec<Field>,
}

impl Msg {
    pub fn new() -> Msg {
        Msg::default()
    }

    /// Set the singular field.
    pub fn one(mut self, number: u32, name: &'static str, value: impl Into<Val>) -> Msg {
        self.fields.push(Field {
            number,
            name,
            repeated: false,
            value: value.into(),
        });
        self
    }

    /// Add each of `values` to the repeated field.
    pub fn many<V: Into<Val>>(
        mut self,
        number: u32,
        name: &'static str,
        values: impl IntoIterator<Item = V>,
    ) -> Msg {
        for value in values {
            self.fields.push(Field {
                number,
                name,
                repeated: true,
                value: value.into(),
            });
        }
        self
    }

    /// The fields in the order the writers print them: by number, those of a
    /// repeated field in the order they were added.
    fn sorted(&self) -> Vec<&Field> {
        let mut fields: Vec<&Field> = self.fields.iter().collect();
        fields.sort_by_key(|f| f.number);
        fields
    }

    // ---- binary ----

    /// The message in the wire format.
    pub fn binary(&self) -> Vec<u8> {
        let mut out = Vec::new();
        self.encode(&mut out);
        out
    }

    /// The message preceded by its length, as `writeDelimitedTo` writes it.
    pub fn delimited(&self) -> Vec<u8> {
        let body = self.binary();
        let mut out = Vec::with_capacity(body.len() + 5);
        varint(&mut out, body.len() as u64);
        out.extend(body);
        out
    }

    fn encode(&self, out: &mut Vec<u8>) {
        for field in self.sorted() {
            let number = u64::from(field.number);
            match &field.value {
                Val::Int(i) => {
                    varint(out, number << 3);
                    varint(out, *i as u64);
                }
                Val::Bool(b) => {
                    varint(out, number << 3);
                    varint(out, u64::from(*b));
                }
                Val::Enum(_, n) => {
                    varint(out, number << 3);
                    varint(out, *n as i64 as u64);
                }
                Val::Str(s) => {
                    varint(out, (number << 3) | 2);
                    varint(out, s.len() as u64);
                    out.extend(s.as_bytes());
                }
                Val::Msg(m) => {
                    let body = m.binary();
                    varint(out, (number << 3) | 2);
                    varint(out, body.len() as u64);
                    out.extend(body);
                }
            }
        }
    }

    // ---- text ----

    /// The text format, which `--output=textproto` prints.
    pub fn text(&self) -> String {
        let mut out = String::new();
        self.write_text(&mut out, 0);
        out
    }

    fn write_text(&self, out: &mut String, indent: usize) {
        let pad = " ".repeat(indent);
        for field in self.sorted() {
            match &field.value {
                Val::Msg(m) => {
                    let _ = writeln!(out, "{pad}{} {{", field.name);
                    m.write_text(out, indent + 2);
                    let _ = writeln!(out, "{pad}}}");
                }
                Val::Int(i) => {
                    let _ = writeln!(out, "{pad}{}: {i}", field.name);
                }
                Val::Bool(b) => {
                    let _ = writeln!(out, "{pad}{}: {b}", field.name);
                }
                Val::Enum(name, _) => {
                    let _ = writeln!(out, "{pad}{}: {name}", field.name);
                }
                Val::Str(s) => {
                    let _ = writeln!(out, "{pad}{}: \"{}\"", field.name, text_escape(s));
                }
            }
        }
    }

    // ---- JSON ----

    /// The JSON `--output=jsonproto` prints: indented, a repeated message
    /// field as `[{` ... `}]`.
    pub fn json(&self) -> String {
        let mut out = String::new();
        self.write_json(&mut out, Some(0));
        out.push('\n');
        out
    }

    /// The JSON of `streamed_jsonproto`: one line, no spaces.
    pub fn json_compact(&self) -> String {
        let mut out = String::new();
        self.write_json(&mut out, None);
        out
    }

    /// `indent` is the indent of the line the message's `{` is on, or `None`
    /// for no whitespace at all.
    fn write_json(&self, out: &mut String, indent: Option<usize>) {
        // The fields, a repeated one gathered into an array.
        let mut entries: Vec<(&'static str, bool, Vec<&Val>)> = Vec::new();
        for field in self.sorted() {
            match entries.last_mut() {
                Some((name, _, values)) if *name == field.name => values.push(&field.value),
                _ => entries.push((field.name, field.repeated, vec![&field.value])),
            }
        }
        out.push('{');
        let inner = indent.map(|i| i + 2);
        for (at, (name, repeated, values)) in entries.iter().enumerate() {
            if at > 0 {
                out.push(',');
            }
            if let Some(i) = inner {
                out.push('\n');
                out.push_str(&" ".repeat(i));
            }
            let _ = write!(out, "\"{}\":", json_name(name));
            if indent.is_some() {
                out.push(' ');
            }
            if *repeated {
                out.push('[');
                for (n, value) in values.iter().enumerate() {
                    if n > 0 {
                        out.push(',');
                        if indent.is_some() {
                            out.push(' ');
                        }
                    }
                    write_json_value(out, value, inner);
                }
                out.push(']');
            } else {
                write_json_value(out, values[0], inner);
            }
        }
        if let Some(i) = indent
            && !entries.is_empty()
        {
            out.push('\n');
            out.push_str(&" ".repeat(i));
        }
        out.push('}');
    }
}

fn write_json_value(out: &mut String, value: &Val, indent: Option<usize>) {
    match value {
        Val::Int(i) => {
            let _ = write!(out, "{i}");
        }
        Val::Bool(b) => {
            let _ = write!(out, "{b}");
        }
        Val::Enum(name, _) => {
            let _ = write!(out, "\"{name}\"");
        }
        Val::Str(s) => {
            let _ = write!(out, "\"{}\"", json_escape(s));
        }
        Val::Msg(m) => m.write_json(out, indent),
    }
}

fn varint(out: &mut Vec<u8>, mut n: u64) {
    while n >= 0x80 {
        out.push((n & 0x7f) as u8 | 0x80);
        n >>= 7;
    }
    out.push(n as u8);
}

/// `string_list_value` as JSON names it: `stringListValue`.
fn json_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut upper = false;
    for c in name.chars() {
        if c == '_' {
            upper = true;
        } else if upper {
            out.extend(c.to_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

/// A string as the Java text printer writes it: quotes, backslashes and
/// control characters escaped, every byte over 127 as an octal escape.
fn text_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &byte in s.as_bytes() {
        match byte {
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            b'"' => out.push_str("\\\""),
            b'\'' => out.push_str("\\'"),
            b'\\' => out.push_str("\\\\"),
            0x20..=0x7e => out.push(byte as char),
            _ => {
                let _ = write!(out, "\\{byte:03o}");
            }
        }
    }
    out
}

/// A string as the JSON printer writes it: the HTML-safe set (`< > & = '`)
/// as `\u00XX` besides the usual escapes.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '<' | '>' | '&' | '=' | '\'' => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c if (c as u32) < 0x20 || c == '\u{2028}' || c == '\u{2029}' => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shape of `bazel cquery --output=jsonproto` for a source file.
    fn source_file() -> Msg {
        Msg::new().one(
            1,
            "results",
            Msg::new()
                .one(
                    1,
                    "target",
                    Msg::new().one(1, "type", Val::Enum("SOURCE_FILE", 2)).one(
                        3,
                        "source_file",
                        Msg::new()
                            .one(1, "name", "//:s.txt")
                            .one(2, "location", "/tmp/fx/s.txt:1:1")
                            .many(5, "visibility_label", ["//visibility:private"]),
                    ),
                )
                .one(2, "configuration", Msg::new().one(4, "checksum", "null")),
        )
    }

    #[test]
    fn json_is_javas_with_the_array_brackets_hugging_the_braces() {
        let msg = Msg::new().many(
            1,
            "results",
            [
                Msg::new().one(1, "name", "a").many(2, "list", ["x", "y"]),
                Msg::new().one(1, "name", "b"),
            ],
        );
        assert_eq!(
            msg.json(),
            "{\n  \"results\": [{\n    \"name\": \"a\",\n    \"list\": [\"x\", \"y\"]\n  }, {\n    \"name\": \"b\"\n  }]\n}\n"
        );
        assert_eq!(
            msg.json_compact(),
            r#"{"results":[{"name":"a","list":["x","y"]},{"name":"b"}]}"#
        );
    }

    #[test]
    fn json_of_a_nested_target_matches_bazels() {
        // Verbatim from `bazel cquery //:s.txt --output=jsonproto`, whose
        // `results` is the field of a one-field message.
        let expected = "{\n  \"results\": {\n    \"target\": {\n      \"type\": \"SOURCE_FILE\",\n      \"sourceFile\": {\n        \"name\": \"//:s.txt\",\n        \"location\": \"/tmp/fx/s.txt:1:1\",\n        \"visibilityLabel\": [\"//visibility:private\"]\n      }\n    },\n    \"configuration\": {\n      \"checksum\": \"null\"\n    }\n  }\n}\n";
        assert_eq!(source_file().json(), expected);
    }

    #[test]
    fn text_nests_with_two_spaces_and_sorts_by_number() {
        let msg = Msg::new()
            .one(3, "c", "late")
            .one(1, "a", Msg::new().one(1, "x", true).one(2, "y", 7))
            .one(2, "b", Val::Enum("RULE", 1));
        assert_eq!(
            msg.text(),
            "a {\n  x: true\n  y: 7\n}\nb: RULE\nc: \"late\"\n"
        );
    }

    #[test]
    fn strings_are_escaped_as_javas_printers_do() {
        assert_eq!(text_escape("a\"b\n\u{e9}'"), "a\\\"b\\n\\303\\251\\'");
        assert_eq!(
            json_escape("cat $@ > x=1 & 'y'"),
            "cat $@ \\u003e x\\u003d1 \\u0026 \\u0027y\\u0027"
        );
    }

    #[test]
    fn binary_is_the_wire_format() {
        // Field 1 varint 150, field 2 string "testing": the protobuf docs' example.
        let msg = Msg::new().one(1, "a", 150).one(2, "b", "testing");
        assert_eq!(
            msg.binary(),
            [
                0x08, 0x96, 0x01, 0x12, 0x07, b't', b'e', b's', b't', b'i', b'n', b'g'
            ]
        );
        assert_eq!(msg.delimited()[0], 12);
        let nested = Msg::new().one(7, "m", Msg::new().one(1, "id", 1));
        assert_eq!(nested.binary(), [0x3a, 0x02, 0x08, 0x01]);
        let negative = Msg::new().one(1, "n", -1);
        assert_eq!(negative.binary().len(), 11);
    }

    #[test]
    fn json_names_are_lower_camel() {
        assert_eq!(json_name("string_list_value"), "stringListValue");
        assert_eq!(json_name("name"), "name");
    }
}
