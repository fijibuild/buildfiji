//! `--proto:rule_classes` (buildfiji-6gk): the `rule_class_key` of a rule and,
//! for the first rule of each class in an output, its `rule_class_info`, a
//! `stardoc_output.RuleInfo`.
//!
//! A class written in Starlark is described from its schema. A class Bazel
//! implements is described by a table recorded from Bazel 9.2.0
//! (`native_rule_classes.json`), as its attributes and their defaults come
//! from Java code that has no other record here.

use crate::graph::{Graph, Node, NodeKind};
use crate::proto::{Msg, Val};
use fjfj_graph::rule::{AttrFlag, AttrType, AttrValue};
use fjfj_graph::schema::{ProviderRef, RuleSchema, SchemaAttr};

/// What identifies a rule class: the `.bzl` and name of one written in
/// Starlark, the name of one Bazel implements.
pub fn class_key(node: &Node, graph: &dyn Graph) -> Option<String> {
    let NodeKind::Rule { class, .. } = &node.kind else {
        return None;
    };
    let schema = node.schema.as_ref()?;
    Some(match &schema.defined_in {
        Some(bzl) if schema.starlark => format!("{}%{class}", graph.display(bzl)),
        _ => class.clone(),
    })
}

/// `AttributeType`.
fn attribute_type(ty: AttrType) -> (&'static str, i32) {
    match ty {
        AttrType::Int | AttrType::Tristate => ("INT", 2),
        AttrType::Label => ("LABEL", 3),
        AttrType::String => ("STRING", 4),
        AttrType::StringList => ("STRING_LIST", 5),
        AttrType::IntList => ("INT_LIST", 6),
        AttrType::LabelList => ("LABEL_LIST", 7),
        AttrType::Bool => ("BOOLEAN", 8),
        AttrType::LabelKeyedStringDict => ("LABEL_STRING_DICT", 9),
        AttrType::StringDict => ("STRING_DICT", 10),
        AttrType::StringListDict => ("STRING_LIST_DICT", 11),
        AttrType::Output => ("OUTPUT", 12),
        AttrType::OutputList => ("OUTPUT_LIST", 13),
        AttrType::StringKeyedLabelDict => ("LABEL_DICT_UNARY", 14),
        AttrType::LabelListDict => ("LABEL_LIST_DICT", 15),
    }
}

/// A string as Starlark writes it.
fn quote(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\x{:02x}", c as u32))
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn list<T>(items: &[T], one: impl Fn(&T) -> String) -> String {
    format!("[{}]", items.iter().map(one).collect::<Vec<_>>().join(", "))
}

fn dict<K, V>(
    items: &[(K, V)],
    key: impl Fn(&K) -> String,
    value: impl Fn(&V) -> String,
) -> String {
    format!(
        "{{{}}}",
        items
            .iter()
            .map(|(k, v)| format!("{}: {}", key(k), value(v)))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// A default as Starlark writes it; `None` for what has no such form.
fn repr(value: &AttrValue, graph: &dyn Graph) -> Option<String> {
    let label = |l: &fjfj_graph::Label| quote(&graph.display(l));
    Some(match value {
        AttrValue::Bool(true) => "True".to_owned(),
        AttrValue::Bool(false) => "False".to_owned(),
        AttrValue::Int(i) => i.to_string(),
        AttrValue::String(s) => quote(s),
        AttrValue::Label(l) => label(l),
        AttrValue::StringList(items) => list(items, |s| quote(s)),
        AttrValue::IntList(items) => list(items, |i| i.to_string()),
        AttrValue::LabelList(items) => list(items, label),
        AttrValue::StringDict(items) => dict(items, |k| quote(k), |v| quote(v)),
        AttrValue::StringListDict(items) => dict(items, |k| quote(k), |v| list(v, |s| quote(s))),
        AttrValue::LabelKeyedStringDict(items) => dict(items, label, |v| quote(v)),
        AttrValue::StringKeyedLabelDict(items) => dict(items, |k| quote(k), label),
        AttrValue::LabelListDict(items) => dict(items, |k| quote(k), |v| list(v, label)),
        AttrValue::Select(_) => return None,
    })
}

/// What a type has when nothing was given.
fn zero(ty: AttrType) -> &'static str {
    match ty {
        AttrType::Bool => "False",
        AttrType::Int | AttrType::Tristate => "0",
        AttrType::String => "\"\"",
        AttrType::Label => "None",
        AttrType::Output => "None",
        AttrType::StringList | AttrType::IntList | AttrType::LabelList | AttrType::OutputList => {
            "[]"
        }
        AttrType::StringDict
        | AttrType::StringListDict
        | AttrType::LabelKeyedStringDict
        | AttrType::StringKeyedLabelDict
        | AttrType::LabelListDict => "{}",
    }
}

fn origin(name: &str, file: &str) -> Msg {
    Msg::new().one(1, "name", name).one(2, "file", file)
}

/// A `ProviderNameGroup`.
fn provider_group(group: &[ProviderRef]) -> Msg {
    Msg::new()
        .many(1, "provider_name", group.iter().map(|p| p.name.as_str()))
        .many(
            2,
            "origin_key",
            group.iter().map(|p| origin(&p.name, &p.file)),
        )
}

fn attribute(attr: &SchemaAttr, graph: &dyn Graph) -> Msg {
    let (type_name, type_number) = attribute_type(attr.def.ty);
    let mut msg = Msg::new().one(1, "name", attr.name.as_str());
    if let Some(doc) = attr.def.doc.as_deref().filter(|d| !d.is_empty()) {
        msg = msg.one(2, "doc_string", doc);
    }
    msg = msg.one(3, "type", Val::Enum(type_name, type_number));
    let mandatory = attr.def.mandatory();
    if mandatory {
        msg = msg.one(4, "mandatory", true);
    }
    msg = msg.many(
        5,
        "provider_name_group",
        attr.info.providers.iter().map(|g| provider_group(g)),
    );
    if !mandatory {
        let default = attr
            .def
            .default
            .as_ref()
            .and_then(|d| repr(d, graph))
            .unwrap_or_else(|| zero(attr.def.ty).to_owned());
        msg = msg.one(6, "default_value", default);
    }
    if !attr.configurable || attr.def.configurable == Some(false) {
        msg = msg.one(7, "nonconfigurable", true);
    }
    msg.many(
        9,
        "values",
        attr.info.values_repr.iter().map(String::as_str),
    )
}

/// What Bazel 9.2.0 printed for the classes it implements and for the
/// attributes every rule has after `name`.
const NATIVE: &str = include_str!("native_rule_classes.json");

fn table() -> &'static serde_json::Value {
    static TABLE: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| serde_json::from_str(NATIVE).expect("a table of classes"))
}

/// The `RuleInfo` of the class of `node`.
pub fn class_info(node: &Node, graph: &dyn Graph) -> Option<Msg> {
    let NodeKind::Rule {
        class,
        test,
        executable,
        ..
    } = &node.kind
    else {
        return None;
    };
    let schema: &RuleSchema = node.schema.as_ref()?;
    if !schema.starlark {
        return native_class(class);
    }
    let mut info = Msg::new().one(1, "rule_name", class.as_str());
    if let Some(doc) = schema.doc.as_deref().filter(|d| !d.is_empty()) {
        info = info.one(2, "doc_string", doc);
    }
    let name = Msg::new()
        .one(1, "name", "name")
        .one(2, "doc_string", "A unique name for this target.")
        .one(3, "type", Val::Enum("NAME", 1))
        .one(4, "mandatory", true);
    let mut attrs = vec![name];
    attrs.extend(builtin_attributes(*test, *executable));
    // Its own, in the order they were declared; a private one is not shown.
    attrs.extend(
        schema
            .attrs
            .iter()
            .filter(|a| a.def.flags.contains(&AttrFlag::StarlarkDefined) && !a.private())
            .map(|a| attribute(a, graph)),
    );
    info = info.many(3, "attribute", attrs);
    if let Some(bzl) = &schema.defined_in {
        info = info.one(4, "origin_key", origin(class, &graph.display(bzl)));
    }
    if !schema.provides.is_empty() {
        info = info.one(5, "advertised_providers", provider_group(&schema.provides));
    }
    if *test {
        info = info.one(6, "test", true);
    }
    if *executable || *test {
        info = info.one(7, "executable", true);
    }
    Some(info)
}

/// A native attribute in the table: `[name, type, default, flags]`.
fn builtin_attributes(test: bool, executable: bool) -> Vec<Msg> {
    let table = table();
    let key = if test {
        "test"
    } else if executable {
        "executable"
    } else {
        "plain"
    };
    table["builtin"][key]
        .as_array()
        .map(|attrs| attrs.iter().map(from_json).collect())
        .unwrap_or_default()
}

fn native_class(class: &str) -> Option<Msg> {
    let info = table()["classes"].get(class)?;
    Some(from_json_info(info))
}

/// An `AttributeInfo` recorded as the JSON Bazel printed it.
fn from_json(value: &serde_json::Value) -> Msg {
    let text = |k: &str| value.get(k).and_then(|v| v.as_str());
    let flag = |k: &str| value.get(k).and_then(|v| v.as_bool()).unwrap_or(false);
    let mut msg = Msg::new().one(1, "name", text("name").unwrap_or_default());
    if let Some(doc) = text("docString") {
        msg = msg.one(2, "doc_string", doc);
    }
    let (type_name, number) = json_type(text("type").unwrap_or_default());
    msg = msg.one(3, "type", Val::Enum(type_name, number));
    if flag("mandatory") {
        msg = msg.one(4, "mandatory", true);
    }
    if let Some(groups) = value.get("providerNameGroup").and_then(|g| g.as_array()) {
        msg = msg.many(5, "provider_name_group", groups.iter().map(json_group));
    }
    if let Some(default) = text("defaultValue") {
        msg = msg.one(6, "default_value", default);
    }
    if flag("nonconfigurable") {
        msg = msg.one(7, "nonconfigurable", true);
    }
    if flag("nativelyDefined") {
        msg = msg.one(8, "natively_defined", true);
    }
    if let Some(values) = value.get("values").and_then(|g| g.as_array()) {
        msg = msg.many(9, "values", values.iter().filter_map(|v| v.as_str()));
    }
    msg
}

fn json_group(group: &serde_json::Value) -> Msg {
    let names = group["providerName"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let keys = group["originKey"].as_array().cloned().unwrap_or_default();
    Msg::new()
        .many(
            1,
            "provider_name",
            names.iter().filter_map(|n| n.as_str().map(str::to_owned)),
        )
        .many(
            2,
            "origin_key",
            keys.iter().map(|k| {
                origin(
                    k["name"].as_str().unwrap_or_default(),
                    k["file"].as_str().unwrap_or_default(),
                )
            }),
        )
}

fn json_type(name: &str) -> (&'static str, i32) {
    match name {
        "NAME" => ("NAME", 1),
        "INT" => ("INT", 2),
        "LABEL" => ("LABEL", 3),
        "STRING" => ("STRING", 4),
        "STRING_LIST" => ("STRING_LIST", 5),
        "INT_LIST" => ("INT_LIST", 6),
        "LABEL_LIST" => ("LABEL_LIST", 7),
        "BOOLEAN" => ("BOOLEAN", 8),
        "LABEL_STRING_DICT" => ("LABEL_STRING_DICT", 9),
        "STRING_DICT" => ("STRING_DICT", 10),
        "STRING_LIST_DICT" => ("STRING_LIST_DICT", 11),
        "OUTPUT" => ("OUTPUT", 12),
        "OUTPUT_LIST" => ("OUTPUT_LIST", 13),
        "LABEL_DICT_UNARY" => ("LABEL_DICT_UNARY", 14),
        "LABEL_LIST_DICT" => ("LABEL_LIST_DICT", 15),
        _ => ("UNKNOWN", 0),
    }
}

/// A `RuleInfo` recorded as the JSON Bazel printed it.
fn from_json_info(info: &serde_json::Value) -> Msg {
    let text = |k: &str| info.get(k).and_then(|v| v.as_str());
    let mut msg = Msg::new().one(1, "rule_name", text("ruleName").unwrap_or_default());
    if let Some(doc) = text("docString") {
        msg = msg.one(2, "doc_string", doc);
    }
    if let Some(attrs) = info.get("attribute").and_then(|a| a.as_array()) {
        msg = msg.many(3, "attribute", attrs.iter().map(from_json));
    }
    if let Some(key) = info.get("originKey") {
        msg = msg.one(
            4,
            "origin_key",
            origin(
                key["name"].as_str().unwrap_or_default(),
                key["file"].as_str().unwrap_or_default(),
            ),
        );
    }
    if let Some(group) = info.get("advertisedProviders") {
        msg = msg.one(5, "advertised_providers", json_group(group));
    }
    if info.get("test").and_then(|v| v.as_bool()) == Some(true) {
        msg = msg.one(6, "test", true);
    }
    if info.get("executable").and_then(|v| v.as_bool()) == Some(true) {
        msg = msg.one(7, "executable", true);
    }
    msg
}
