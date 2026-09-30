//! Binding a call's arguments to a native function's parameters the way
//! Bazel does, with its wording (buildfiji-mum.4, buildfiji-mum.14).
//!
//! Bazel's natives do not word their argument errors alike, so [`bind`] takes
//! the wording per function. Everything here was checked against Bazel 9.2.0.

use fjfj_graph::rule::suggest;
use starlark::eval::{Arguments, Evaluator};
use starlark::values::Value;
use starlark::values::list::ListRef;
use starlark::values::tuple::TupleRef;

pub(crate) fn fatal(message: impl Into<String>) -> starlark::Error {
    starlark::Error::new_other(anyhow::anyhow!(message.into()))
}

/// Whose wording an argument error takes: Bazel's natives are not uniform.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Wording {
    /// A function with a declared signature: `glob`, `exports_files`,
    /// `existing_rule`.
    Signature,
    /// `package()`, which does its own checking.
    Package,
    /// `package_group()`.
    Group,
}

#[derive(Clone, Copy)]
pub(crate) struct Param {
    pub(crate) name: &'static str,
    /// Can be given by position (in the order listed).
    pub(crate) positional: bool,
    pub(crate) required: bool,
}

pub(crate) const fn param(name: &'static str, positional: bool, required: bool) -> Param {
    Param {
        name,
        positional,
        required,
    }
}

/// Match `args` to `params` the way Bazel does, with its errors.
pub(crate) fn bind<'v>(
    function: &str,
    wording: Wording,
    params: &[Param],
    args: &Arguments<'v, '_>,
    eval: &Evaluator<'v, '_, '_>,
) -> starlark::Result<Vec<Option<Value<'v>>>> {
    let mut bound: Vec<Option<Value<'v>>> = vec![None; params.len()];
    let slots: Vec<usize> = (0..params.len())
        .filter(|&i| params[i].positional)
        .collect();
    let positions: Vec<Value<'v>> = args.positions(eval.heap())?.collect();
    if positions.len() > slots.len() {
        return Err(fatal(match wording {
            Wording::Signature => format!(
                "{function}() accepts no more than {} positional arguments but got {}",
                slots.len(),
                positions.len()
            ),
            Wording::Package | Wording::Group => {
                format!("{function}() got unexpected positional argument")
            }
        }));
    }
    for (value, &slot) in positions.iter().zip(&slots) {
        bound[slot] = Some(*value);
    }
    for (key, value) in args.names_map()?.iter() {
        let Some(slot) = params.iter().position(|p| p.name == key.as_str()) else {
            return Err(fatal(match wording {
                Wording::Package => format!("unexpected keyword argument: {}", key.as_str()),
                _ => format!(
                    "{function}() got unexpected keyword argument '{}'{}",
                    key.as_str(),
                    suggest(key.as_str(), params.iter().map(|p| p.name))
                        .map(|s| format!(" (did you mean '{s}'?)"))
                        .unwrap_or_default()
                ),
            }));
        };
        if bound[slot].is_some() {
            return Err(fatal(format!(
                "{function}() got multiple values for argument '{}'",
                key.as_str()
            )));
        }
        bound[slot] = Some(*value);
    }
    for (p, value) in params.iter().zip(&bound) {
        if p.required && value.is_none() {
            return Err(fatal(format!(
                "{function}() missing 1 required {} argument: {}",
                if p.positional { "positional" } else { "named" },
                p.name
            )));
        }
    }
    Ok(bound)
}

pub(crate) fn describe(value: Value<'_>) -> String {
    format!("{} ({})", value.to_repr(), value.get_type())
}

/// The items of a list or tuple.
pub(crate) fn sequence<'v>(value: Value<'v>) -> Option<Vec<Value<'v>>> {
    if let Some(list) = ListRef::from_value(value) {
        Some(list.iter().collect())
    } else {
        TupleRef::from_value(value).map(|tuple| tuple.iter().collect())
    }
}

/// A signature-checked sequence parameter.
pub(crate) fn want_sequence<'v>(
    function: &str,
    param: &str,
    value: Value<'v>,
    allow_none: bool,
) -> starlark::Result<Option<Vec<Value<'v>>>> {
    if allow_none && value.is_none() {
        return Ok(None);
    }
    sequence(value).map(Some).ok_or_else(|| {
        fatal(format!(
            "in call to {function}(), parameter '{param}' got value of type '{}', want '{}'",
            value.get_type(),
            if allow_none {
                "sequence or NoneType"
            } else {
                "sequence"
            }
        ))
    })
}
