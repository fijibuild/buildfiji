//! `proto.encode_text(struct)` (buildfiji-mum.3.1): a struct written in
//! protobuf text format, as Bazel 9.2.0 does.
//!
//! - fields come out sorted (a struct keeps them so), a `None` field is left
//!   out, and a list or tuple repeats its field, an empty one writing
//!   nothing;
//! - a nested struct is `name {` ... `}` in two-space steps, an empty one
//!   `name {` `}`;
//! - a dict is a repeated `name { key: ... value: ... }`, with int or string
//!   keys only;
//! - strings escape only `"`, `\` and newline; a float is written as
//!   Starlark's `repr` does, except that infinities are `inf` and `-inf`.

use crate::args::{Wording, bind, fatal, positional_only};
use crate::floats::format_float;
use crate::structs::fields_of;
use allocative::Allocative;
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
pub(crate) struct ProtoModule;

starlark_simple_value!(ProtoModule);

impl fmt::Display for ProtoModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "proto")
    }
}

#[starlark_value(type = "proto")]
impl<'v> StarlarkValue<'v> for ProtoModule {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("proto", proto_methods);
        Some(RES.methods())
    }
}

#[starlark_module]
fn proto_methods(builder: &mut MethodsBuilder) {
    /// `proto.encode_text(x)`.
    fn encode_text<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<String> {
        let _ = this;
        let bound = bind(
            "encode_text",
            Wording::Signature,
            &[positional_only("x", true)],
            args,
            eval,
        )?;
        let x = bound[0].expect("required");
        let fields = fields_of(x).ok_or_else(|| {
            fatal(format!(
                "in call to encode_text(), parameter 'x' got value of type '{}', want 'structure \
                 or NativeInfo'",
                x.get_type()
            ))
        })?;
        let mut out = String::new();
        message(&fields, 0, eval.heap(), &mut out).map_err(fatal)?;
        Ok(out)
    }
}

const WANT: &str = "want string, int, float, bool, or struct";

fn message<'v>(
    fields: &[(&str, Value<'v>)],
    level: usize,
    heap: Heap<'v>,
    out: &mut String,
) -> Result<(), String> {
    for (name, value) in fields {
        if value.is_none() {
            continue;
        }
        field(name, *value, level, true, heap, out)
            .map_err(|e| format!("in struct field .{name}: {e}"))?;
    }
    Ok(())
}

fn pad(level: usize, out: &mut String) {
    for _ in 0..level {
        out.push_str("  ");
    }
}

/// Write one field. `repeated` is whether a list or dict is allowed here:
/// at the field itself, but not inside one.
fn field<'v>(
    name: &str,
    value: Value<'v>,
    level: usize,
    repeated: bool,
    heap: Heap<'v>,
    out: &mut String,
) -> Result<(), String> {
    if let Some(scalar) = scalar(value) {
        pad(level, out);
        out.push_str(&format!("{name}: {scalar}\n"));
    } else if let Some(fields) = fields_of(value) {
        pad(level, out);
        out.push_str(&format!("{name} {{\n"));
        message(&fields, level + 1, heap, out)?;
        pad(level, out);
        out.push_str("}\n");
    } else if repeated && matches!(value.get_type(), "list" | "tuple" | "range") {
        let items = value.iterate(heap).map_err(|e| e.to_string())?;
        for (i, item) in items.enumerate() {
            field(name, item, level, false, heap, out)
                .map_err(|e| format!("at list index {i}: {e}"))?;
        }
    } else if repeated && let Some(dict) = DictRef::from_value(value) {
        for (k, v) in dict.iter() {
            let key = scalar(k)
                .filter(|_| matches!(k.get_type(), "int" | "string"))
                .ok_or_else(|| {
                    format!("invalid dict key: got {}, want int or string", k.get_type())
                })?;
            pad(level, out);
            out.push_str(&format!("{name} {{\n"));
            pad(level + 1, out);
            out.push_str(&format!("key: {key}\n"));
            field("value", v, level + 1, false, heap, out)
                .map_err(|e| format!("in value for dict key {}: {e}", k.to_repr()))?;
            pad(level, out);
            out.push_str("}\n");
        }
    } else {
        return Err(format!("got {}, {WANT}", value.get_type()));
    }
    Ok(())
}

/// A string, int, float or bool as text format writes it.
fn scalar(value: Value<'_>) -> Option<String> {
    if let Some(b) = value.unpack_bool() {
        Some(b.to_string())
    } else if let Some(s) = value.unpack_str() {
        let mut out = String::from("\"");
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                c => out.push(c),
            }
        }
        out.push('"');
        Some(out)
    } else if value.get_type() == "int" {
        Some(value.to_repr())
    } else {
        value.downcast_ref::<StarlarkFloat>().map(|f| {
            if f.0.is_nan() {
                "nan".to_owned()
            } else if f.0.is_infinite() {
                (if f.0 > 0.0 { "inf" } else { "-inf" }).to_owned()
            } else {
                format_float(f.0)
            }
        })
    }
}
