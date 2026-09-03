//! Attribute values carried by module-file directives.
//!
//! `archive_override(...)`, `git_override(...)` and module-extension tags
//! all forward arbitrary keyword arguments to a repo rule or tag class
//! that fjfj has not looked at yet — the repo rule runs in the fetch phase
//! (buildfiji-mum.8), the tag class is defined by the extension. So the
//! module file records them as plain data rather than as live Starlark
//! values: nothing here borrows the evaluator's heap, so a resolved module
//! graph outlives the evaluation that produced it.
//!
//! This mirrors Bazel's `AttributeValues`, which exists for the same
//! reason.

use starlark::values::Value;
use starlark::values::dict::DictRef;
use starlark::values::list::ListRef;

/// A Starlark value as it appears in a module-file directive: the subset
/// of the language that can cross out of the evaluator into the module
/// graph.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum AttrValue {
    None,
    Bool(bool),
    Int(i64),
    String(String),
    List(Vec<AttrValue>),
    /// Insertion-ordered, because Starlark dicts are and repo rules can
    /// see the difference.
    Dict(Vec<(AttrValue, AttrValue)>),
}

/// A value in a directive that fjfj cannot represent as data.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unsupported attribute value of type '{0}' (expected a string, int, bool, list or dict)")]
pub struct UnsupportedAttrValue(pub String);

impl AttrValue {
    /// Converts a Starlark value into recorded data, rejecting anything
    /// that would keep the evaluator's heap alive (functions, structs,
    /// providers, extension proxies).
    pub fn from_value(value: Value<'_>) -> Result<AttrValue, UnsupportedAttrValue> {
        if value.is_none() {
            return Ok(AttrValue::None);
        }
        if let Some(b) = value.unpack_bool() {
            return Ok(AttrValue::Bool(b));
        }
        if let Some(s) = value.unpack_str() {
            return Ok(AttrValue::String(s.to_owned()));
        }
        if let Some(i) = value.unpack_i32() {
            return Ok(AttrValue::Int(i.into()));
        }
        if let Some(list) = ListRef::from_value(value) {
            return list
                .iter()
                .map(AttrValue::from_value)
                .collect::<Result<Vec<_>, _>>()
                .map(AttrValue::List);
        }
        if let Some(dict) = DictRef::from_value(value) {
            return dict
                .iter()
                .map(|(k, v)| Ok((AttrValue::from_value(k)?, AttrValue::from_value(v)?)))
                .collect::<Result<Vec<_>, _>>()
                .map(AttrValue::Dict);
        }
        Err(UnsupportedAttrValue(value.get_type().to_owned()))
    }

    /// The value as a string, for the directives that require one.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            AttrValue::String(s) => Some(s),
            _ => None,
        }
    }
}

/// An insertion-ordered list of keyword arguments, as written in the
/// module file.
pub type Attrs = Vec<(String, AttrValue)>;
