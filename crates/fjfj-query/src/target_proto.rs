//! Targets as `build.proto`'s `Target` message, for `--output=proto` and the
//! formats that print it (buildfiji-tle.4).

use crate::eval::Evaluator;
use crate::graph::{Graph, Node, NodeAttr, NodeKind};
use crate::proto::{Msg, Val};
use fjfj_graph::Label;
use fjfj_graph::rule::{AttrType, AttrValue, default_condition};
use std::collections::BTreeSet;

/// The `--proto:` flags that shape the `Target` messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtoOptions {
    /// `--[no]proto:flatten_selects` (default true): a `select()` shows every
    /// value it could take, joined, instead of as a `SELECTOR_LIST`.
    pub flatten_selects: bool,
    /// `--[no]proto:default_values` (default true): attributes the rule did
    /// not set are shown.
    pub default_values: bool,
    /// `--[no]proto:rule_inputs_and_outputs` (default true).
    pub rule_inputs_and_outputs: bool,
    /// `--[no]proto:locations` (default true).
    pub locations: bool,
    /// `--proto:output_rule_attrs`: only these attributes, if given.
    pub output_rule_attrs: Option<Vec<String>>,
    /// `--[no]proto:instantiation_stack` (default false).
    pub instantiation_stack: bool,
    /// `--[no]proto:definition_stack` (default false).
    pub definition_stack: bool,
    /// `--[no]proto:include_configurations` (default true, `cquery`): without
    /// them the output is that of `query`.
    pub include_configurations: bool,
    /// `--[no]proto:include_attribute_source_aspects` (default false).
    pub include_attribute_source_aspects: bool,
}

impl Default for ProtoOptions {
    fn default() -> ProtoOptions {
        ProtoOptions {
            flatten_selects: true,
            default_values: true,
            rule_inputs_and_outputs: true,
            locations: true,
            output_rule_attrs: None,
            instantiation_stack: false,
            definition_stack: false,
            include_configurations: true,
            include_attribute_source_aspects: false,
        }
    }
}

impl ProtoOptions {
    /// Read the flag `name` (without its dashes) with its `value` if it is a
    /// `--proto:` one. `Ok(false)` if it is not.
    pub fn flag(&mut self, name: &str, value: Option<&str>) -> Result<bool, String> {
        let (base, negated) = match name.strip_prefix("noproto:") {
            Some(base) => (base, true),
            None => match name.strip_prefix("proto:") {
                Some(base) => (base, false),
                None => return Ok(false),
            },
        };
        let on = match value {
            Some("false" | "0" | "no") => false,
            Some(_) | None => !negated,
        };
        match base {
            "flatten_selects" => self.flatten_selects = on,
            "default_values" => self.default_values = on,
            "rule_inputs_and_outputs" => self.rule_inputs_and_outputs = on,
            "locations" => self.locations = on,
            "instantiation_stack" => self.instantiation_stack = on,
            "definition_stack" => self.definition_stack = on,
            "include_configurations" => self.include_configurations = on,
            "include_attribute_source_aspects" => self.include_attribute_source_aspects = on,
            // Bazel shows the hash whatever this says.
            "include_synthetic_attribute_hash" => {}
            "output_rule_attrs" => {
                self.output_rule_attrs = Some(
                    value
                        .unwrap_or("")
                        .split(',')
                        .filter(|a| !a.is_empty())
                        .map(str::to_owned)
                        .collect(),
                );
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

/// `Attribute.Discriminator`.
fn discriminator(ty: AttrType) -> Val {
    let (name, number) = match ty {
        AttrType::Int => ("INTEGER", 1),
        AttrType::String => ("STRING", 2),
        AttrType::Label => ("LABEL", 3),
        AttrType::Output => ("OUTPUT", 4),
        AttrType::StringList => ("STRING_LIST", 5),
        AttrType::LabelList => ("LABEL_LIST", 6),
        AttrType::OutputList => ("OUTPUT_LIST", 7),
        AttrType::StringDict => ("STRING_DICT", 10),
        AttrType::LabelListDict => ("LABEL_LIST_DICT", 12),
        AttrType::StringListDict => ("STRING_LIST_DICT", 13),
        AttrType::Bool => ("BOOLEAN", 14),
        AttrType::Tristate => ("TRISTATE", 15),
        AttrType::IntList => ("INTEGER_LIST", 16),
        AttrType::StringKeyedLabelDict => ("LABEL_DICT_UNARY", 19),
        AttrType::LabelKeyedStringDict => ("LABEL_KEYED_STRING_DICT", 21),
    };
    Val::Enum(name, number)
}

/// The field numbers a value of an attribute has in `Attribute` and in
/// `Attribute.SelectorEntry`.
struct Numbers {
    int: u32,
    string: u32,
    boolean: u32,
    tristate: u32,
    string_list: u32,
    string_dict: u32,
    label_list_dict: u32,
    string_list_dict: u32,
    int_list: u32,
    label_dict_unary: u32,
    label_keyed_string_dict: u32,
}

const ATTRIBUTE: Numbers = Numbers {
    int: 3,
    string: 5,
    boolean: 14,
    tristate: 15,
    string_list: 6,
    string_dict: 8,
    label_list_dict: 10,
    string_list_dict: 11,
    int_list: 17,
    label_dict_unary: 19,
    label_keyed_string_dict: 22,
};

const SELECTOR_ENTRY: Numbers = Numbers {
    int: 2,
    string: 3,
    boolean: 4,
    tristate: 5,
    string_list: 6,
    string_dict: 8,
    label_list_dict: 10,
    string_list_dict: 11,
    int_list: 13,
    label_dict_unary: 15,
    label_keyed_string_dict: 17,
};

fn pair(key: &str, value: Val) -> Msg {
    Msg::new().one(1, "key", key).one(2, "value", value)
}

fn pair_of_list(key: &str, values: Vec<String>) -> Msg {
    Msg::new().one(1, "key", key).many(2, "value", values)
}

/// The value fields of `value` (not a `select()`), added to `msg`.
fn add_value(graph: &dyn Graph, ty: AttrType, value: &AttrValue, n: &Numbers, mut msg: Msg) -> Msg {
    let label = |l: &Label| graph.display(l);
    match value {
        AttrValue::Bool(b) => {
            msg = msg.one(n.int, "int_value", i64::from(*b)).one(
                n.string,
                "string_value",
                b.to_string(),
            );
            msg = msg.one(n.boolean, "boolean_value", *b);
        }
        AttrValue::Int(i) if ty == AttrType::Tristate => {
            let (name, number, text) = match i {
                -1 => ("AUTO", 2, "auto"),
                0 => ("NO", 0, "no"),
                _ => ("YES", 1, "yes"),
            };
            msg = msg
                .one(n.int, "int_value", i64::from(*i))
                .one(n.string, "string_value", text)
                .one(n.tristate, "tristate_value", Val::Enum(name, number));
        }
        AttrValue::Int(i) => msg = msg.one(n.int, "int_value", i64::from(*i)),
        AttrValue::String(s) => msg = msg.one(n.string, "string_value", s.as_str()),
        AttrValue::Label(l) => msg = msg.one(n.string, "string_value", label(l)),
        AttrValue::StringList(items) => {
            msg = msg.many(n.string_list, "string_list_value", items.iter().cloned());
        }
        AttrValue::LabelList(items) => {
            msg = msg.many(n.string_list, "string_list_value", items.iter().map(label));
        }
        AttrValue::IntList(items) => {
            msg = msg.many(
                n.int_list,
                "int_list_value",
                items.iter().map(|i| i64::from(*i)),
            );
        }
        AttrValue::StringDict(items) => {
            msg = msg.many(
                n.string_dict,
                "string_dict_value",
                items.iter().map(|(k, v)| pair(k, v.as_str().into())),
            );
        }
        AttrValue::StringListDict(items) => {
            msg = msg.many(
                n.string_list_dict,
                "string_list_dict_value",
                items.iter().map(|(k, v)| pair_of_list(k, v.clone())),
            );
        }
        AttrValue::LabelListDict(items) => {
            msg = msg.many(
                n.label_list_dict,
                "label_list_dict_value",
                items
                    .iter()
                    .map(|(k, v)| pair_of_list(k, v.iter().map(label).collect())),
            );
        }
        AttrValue::LabelKeyedStringDict(items) => {
            msg = msg.many(
                n.label_keyed_string_dict,
                "label_keyed_string_dict_value",
                items
                    .iter()
                    .map(|(k, v)| pair(&label(k), v.as_str().into())),
            );
        }
        AttrValue::StringKeyedLabelDict(items) => {
            msg = msg.many(
                n.label_dict_unary,
                "label_dict_unary_value",
                items.iter().map(|(k, v)| pair(k, label(v).into())),
            );
        }
        AttrValue::Select(_) => {}
    }
    msg
}

/// Attributes whose labels make no dependency.
const NODEP: [&str; 2] = ["transitive_configs", "visibility"];

/// An attribute that has no value, as the attributes every rule has but this
/// module cannot read from a class's schema: Bazel's own, named `$x` or `:x`.
fn internal(name: &'static str, ty: AttrType, value: Option<AttrValue>) -> NodeAttr {
    NodeAttr {
        name: name.to_owned(),
        text: String::new(),
        labels: Vec::new(),
        explicit: false,
        ty,
        unset: value.is_none(),
        value: value.unwrap_or(AttrValue::StringList(Vec::new())),
    }
}

fn attribute(graph: &dyn Graph, attr: &NodeAttr, native: bool, options: &ProtoOptions) -> Msg {
    let mut msg = Msg::new().one(1, "name", attr.name.as_str());
    // Bazel types these two as the lists of strings they are.
    let ty_of = match attr.name.as_str() {
        "visibility" | "transitive_configs" => AttrType::StringList,
        _ => attr.ty,
    };
    let mut ty = discriminator(ty_of);
    let mut nodep = true;
    if native && attr.name == "licenses" && attr.ty == AttrType::StringList {
        // `licenses` is a `LICENSE`: its kinds, `NONE` when there are none.
        ty = Val::Enum("LICENSE", 9);
        nodep = false;
        let AttrValue::StringList(kinds) = &attr.value else {
            unreachable!("licenses is a list of strings")
        };
        let mut kinds: Vec<String> = kinds.iter().map(|k| k.to_uppercase()).collect();
        if kinds.is_empty() {
            kinds.push("NONE".to_owned());
        }
        msg = msg.one(7, "license", Msg::new().many(1, "license_type", kinds));
    } else if attr.unset || (attr.name == "deprecation" && !attr.explicit) {
        // No value to give.
    } else if let (AttrValue::Select(list), false) = (&attr.value, options.flatten_selects) {
        ty = Val::Enum("SELECTOR_LIST", 20);
        let default = default_condition();
        let elements = list.elements.iter().map(|selector| {
            let entries = selector.branches.iter().map(|(condition, value)| {
                let entry = Msg::new().one(1, "label", graph.display(condition)).one(
                    16,
                    "is_default_value",
                    false,
                );
                match value {
                    Some(value) => add_value(graph, ty_of, value, &SELECTOR_ENTRY, entry),
                    None => entry,
                }
            });
            Msg::new()
                .many(1, "entries", entries)
                .one(
                    2,
                    "has_default_value",
                    selector.branches.iter().any(|(c, _)| *c == default),
                )
                .one(3, "no_match_error", selector.no_match_error.as_str())
        });
        msg = msg.one(
            21,
            "selector_list",
            Msg::new()
                .one(1, "type", discriminator(attr.ty))
                .many(2, "elements", elements),
        );
    } else {
        // What `--output=proto` shows of a `select()` is every value it could
        // take, joined.
        let value = crate::output::flatten(&attr.value);
        msg = add_value(graph, ty_of, &value, &ATTRIBUTE, msg);
    }
    msg = msg
        .one(2, "type", ty)
        .one(13, "explicitly_specified", attr.explicit);
    if nodep
        && matches!(
            ty_of,
            AttrType::Label
                | AttrType::LabelList
                | AttrType::LabelKeyedStringDict
                | AttrType::Output
                | AttrType::OutputList
                | AttrType::String
                | AttrType::StringList
        )
    {
        msg = msg.one(20, "nodep", NODEP.contains(&attr.name.as_str()));
    }
    msg
}

/// The attributes of a rule as Bazel lists them: its own and the common ones
/// by name, with the few it has that are not in a class's schema.
fn attributes<'a>(class: &str, node: &'a Node) -> Vec<std::borrow::Cow<'a, NodeAttr>> {
    use std::borrow::Cow;
    let mut attrs: Vec<Cow<'a, NodeAttr>> = node
        .attrs
        .iter()
        // The licence-making attribute Bazel 9 no longer has.
        .filter(|a| a.name != "applicable_licenses")
        .map(Cow::Borrowed)
        .collect();
    attrs.push(Cow::Owned(internal(
        "$config_dependencies",
        AttrType::LabelList,
        Some(AttrValue::LabelList(node.config_deps.clone())),
    )));
    if class != "alias" {
        attrs.push(Cow::Owned(internal(
            ":action_listener",
            AttrType::LabelList,
            Some(AttrValue::LabelList(Vec::new())),
        )));
    }
    if class == "genrule" {
        attrs.push(Cow::Owned(internal(
            "$is_executable",
            AttrType::Bool,
            Some(AttrValue::Bool(false)),
        )));
        attrs.push(Cow::Owned(internal(
            "$genrule_setup",
            AttrType::Label,
            Some(AttrValue::Label(Label {
                repo: "bazel_tools".to_owned(),
                package: "tools/genrule".to_owned(),
                name: "genrule-setup.sh".to_owned(),
            })),
        )));
    }
    attrs.sort_by(|a, b| a.name.cmp(&b.name));
    attrs
}

/// The files a rule makes, as Bazel lists them: those of its output
/// attributes in the order of the attributes' names, then the others.
fn outputs(node: &Node, graph: &dyn Graph) -> Vec<String> {
    let mut by_name: Vec<&NodeAttr> = node.attrs.iter().collect();
    by_name.sort_by(|a, b| a.name.cmp(&b.name));
    let mut out: Vec<String> = Vec::new();
    for attr in by_name {
        if !matches!(attr.ty, AttrType::Output | AttrType::OutputList) {
            continue;
        }
        match &attr.value {
            AttrValue::Label(l) => out.push(graph.display(l)),
            AttrValue::LabelList(ls) => out.extend(ls.iter().map(|l| graph.display(l))),
            _ => {}
        }
    }
    for file in &node.outputs {
        let shown = graph.display(file);
        if !out.contains(&shown) {
            out.push(shown);
        }
    }
    out
}

/// The `Target` for `label`.
pub fn target(ev: &Evaluator<'_>, label: &Label, options: &ProtoOptions) -> Result<Msg, String> {
    let graph = ev.graph();
    let node: std::sync::Arc<Node> = ev.node(label)?;
    let name = graph.display(label);
    Ok(match &node.kind {
        NodeKind::Rule { class, native, .. } => {
            let native = *native;
            let attrs: Vec<_> = attributes(class, &node)
                .into_iter()
                .filter(|a| options.default_values || a.explicit || a.name == "name")
                .filter(|a| {
                    options
                        .output_rule_attrs
                        .as_ref()
                        .is_none_or(|only| only.contains(&a.name))
                })
                .collect();
            let mut rule = Msg::new()
                .one(1, "name", name)
                .one(2, "rule_class", class.as_str());
            if options.locations {
                rule = rule.one(3, "location", node.shown_location(graph));
            }
            let mut shown: Vec<Msg> = attrs
                .iter()
                .map(|a| {
                    let message = attribute(graph, a, native, options);
                    if options.include_attribute_source_aspects {
                        message.one(23, "source_aspect_name", "")
                    } else {
                        message
                    }
                })
                .collect();
            // Last, after the attributes by name and whatever is asked for: a
            // digest of the rule class.
            if let Some(hash) = &node.implementation_hash {
                shown.push(
                    Msg::new()
                        .one(1, "name", "$rule_implementation_hash")
                        .one(2, "type", Val::Enum("STRING", 2))
                        .one(5, "string_value", hash.as_str()),
                );
            }
            rule = rule.many(4, "attribute", shown);
            if options.rule_inputs_and_outputs {
                // In label order, which compares the canonical names of the
                // repositories.
                let inputs: Vec<String> = ev
                    .edges(&*graph.declared_node(label)?)
                    .into_iter()
                    .filter(|e| !e.visibility)
                    .map(|e| e.to)
                    .collect::<BTreeSet<Label>>()
                    .iter()
                    .map(|l| graph.display(l))
                    .collect();
                rule = rule.many(5, "rule_input", inputs).many(
                    6,
                    "rule_output",
                    outputs(&node, graph),
                );
            }
            let shown = |frames: &[crate::graph::Frame]| -> Vec<String> {
                frames
                    .iter()
                    .map(|f| format!("{}: {}", f.relative, f.function))
                    .collect()
            };
            if options.instantiation_stack {
                rule = rule.many(13, "instantiation_stack", shown(&node.stack));
            }
            if options.definition_stack {
                rule = rule.many(14, "definition_stack", shown(&node.definition_stack));
            }
            Msg::new()
                .one(1, "type", Val::Enum("RULE", 1))
                .one(2, "rule", rule)
        }
        NodeKind::SourceFile => {
            let mut file = Msg::new().one(1, "name", name);
            if options.locations {
                file = file.one(2, "location", node.shown_location(graph));
            }
            let mut loads: Vec<&Label> = node.loads.iter().collect();
            loads.sort();
            file = file.many(3, "subinclude", loads.iter().map(|l| graph.display(l)));
            file = file.many(
                5,
                "visibility_label",
                node.visibility.iter().map(String::as_str),
            );
            if node.build_file {
                file = file.one(9, "package_contains_errors", false);
            }
            Msg::new()
                .one(1, "type", Val::Enum("SOURCE_FILE", 2))
                .one(3, "source_file", file)
        }
        NodeKind::GeneratedFile { rule } => {
            let mut file =
                Msg::new()
                    .one(1, "name", name)
                    .one(2, "generating_rule", graph.display(rule));
            if options.locations {
                file = file.one(3, "location", node.shown_location(graph));
            }
            Msg::new()
                .one(1, "type", Val::Enum("GENERATED_FILE", 3))
                .one(4, "generated_file", file)
        }
        NodeKind::PackageGroup => {
            let (includes, packages) = node.group.clone().unwrap_or_default();
            Msg::new()
                .one(1, "type", Val::Enum("PACKAGE_GROUP", 4))
                .one(
                    5,
                    "package_group",
                    Msg::new()
                        .one(1, "name", name)
                        .many(2, "contained_package", packages)
                        .many(
                            3,
                            "included_package_group",
                            includes.iter().map(|i| graph.display(i)),
                        ),
                )
        }
    })
}
