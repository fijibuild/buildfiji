//! `attr.*` against Bazel 9.2.0 (buildfiji-mum.3.3): the tables at the
//! bottom (and in `attr_matrix.rs`) are what Bazel printed and reported for
//! the same `.bzl` bodies, and the tests here are what those cannot say: what
//! a descriptor keeps, and that a label default is read where the call is
//! written.

use crate::attr::view;
use crate::attr_matrix::ATTR_MATRIX;
use crate::test_support::{module_in, replay, replay_in};
use crate::{build_globals, bzl_globals};
use fjfj_graph::Label;
use fjfj_graph::rule::{AttrFlag, AttrType, AttrValue, Cfg, FileTypes};
use starlark::values::Value;

/// Probes whose answer is the Starlark runtime's generic wording, which
/// belongs to buildfiji-v32.
const GENERIC: &[&str] = &[
    "has no field or method",
    "has no operator",
    "unsupported binary operation",
    "unsupported comparison",
    "unhashable type",
    "cannot set .",
    "is not callable",
    "in call to len()",
    "in call to list()",
    "in call to hash()",
    // Parse-time errors, and `print`'s own signature.
    "duplicate keyword argument",
    "positional argument may not follow keyword argument",
    "dictionary expression has duplicate key",
    "print() got unexpected keyword argument",
];

/// Probes that need something no bead has built yet, or print a builtin
/// (Bazel writes `<built-in method string of attr value>`, buildfiji-v32).
const SKIPPED: &[&str] = &[
    "DefaultInfo",
    "OutputGroupInfo",
    "print(attr.string)",
    "print(attr.label_list)",
    "repr(attr.label)",
    "str(attr.label)",
    "type(attr.string)",
    "dir(attr.string)",
    "attr[",
    // The crate calls a builtin a `function` and accepts it where Bazel does
    // not (buildfiji-v32).
    "=print)",
    "dir(config",
];

fn assert_replays(wrong: Vec<String>) {
    assert!(
        wrong.is_empty(),
        "{} differ:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

#[test]
fn attr_builders_are_bazels() {
    assert_replays(replay(ATTR_CASES, SKIPPED, GENERIC));
}

#[test]
fn every_keyword_takes_what_bazel_takes() {
    assert_replays(replay(ATTR_MATRIX, SKIPPED, GENERIC));
}

/// Rows taken in the workspace of the Label tests, in the package `sub` of
/// the main repo: a label default is read where the call is written.
const IN_MAIN: &[(&str, Result<&str, &str>)] = &[
    (
        r#"print(attr.label(default=":x") == attr.label(default="//sub:x"))"#,
        Ok("True"),
    ),
    (
        r#"print(attr.label(default=":x") == attr.label(default="//:x"))"#,
        Ok("False"),
    ),
    (
        r#"print(attr.label(default="@oth//x:y") == attr.label(default="@@other+//x:y"))"#,
        Ok("False"),
    ),
    (
        r#"print(attr.label(default="@mydep//x:y") == attr.label(default="@@dep+//x:y"))"#,
        Ok("True"),
    ),
    (
        r#"print(attr.label(default="//x:y") == attr.label(default="@@dep+//x:y"))"#,
        Ok("False"),
    ),
    (
        r#"print(attr.label(default="//x:y") == attr.label(default="@@//x:y"))"#,
        Ok("True"),
    ),
    (
        r#"print(attr.label(default="@//x:y") == attr.label(default="@@dep+//x:y"))"#,
        Ok("False"),
    ),
    (
        r#"print(attr.label(default="@//x:y") == attr.label(default="@@//x:y"))"#,
        Ok("True"),
    ),
    (
        r#"print(attr.label(default="@nope//x:y") == attr.label(default="@nope//x:y"))"#,
        Ok("True"),
    ),
    (
        r#"print(attr.label(default="@nope//x:y"))"#,
        Ok("<attr.label>"),
    ),
    (
        r#"print(attr.label_list(default=["@nope//x:y"]) == attr.label_list(default=["@nope//x:y"]))"#,
        Ok("True"),
    ),
];

#[test]
fn a_label_default_is_read_in_the_main_repo() {
    assert_replays(replay_in("", "sub", IN_MAIN, &[], &[]));
}

/// The same rows for a `.bzl` in `dep+`, which calls itself `dep` and
/// `other+` `oth`.
const IN_DEP: &[(&str, Result<&str, &str>)] = &[
    (
        r#"print(attr.label(default=":x") == attr.label(default="//sub:x"))"#,
        Ok("True"),
    ),
    (
        r#"print(attr.label(default=":x") == attr.label(default="//:x"))"#,
        Ok("False"),
    ),
    (
        r#"print(attr.label(default="@oth//x:y") == attr.label(default="@@other+//x:y"))"#,
        Ok("True"),
    ),
    (
        r#"print(attr.label(default="@mydep//x:y") == attr.label(default="@@dep+//x:y"))"#,
        Ok("False"),
    ),
    (
        r#"print(attr.label(default="//x:y") == attr.label(default="@@dep+//x:y"))"#,
        Ok("True"),
    ),
    (
        r#"print(attr.label(default="//x:y") == attr.label(default="@@//x:y"))"#,
        Ok("False"),
    ),
    (
        r#"print(attr.label(default="@//x:y") == attr.label(default="@@dep+//x:y"))"#,
        Ok("False"),
    ),
    (
        r#"print(attr.label(default="@//x:y") == attr.label(default="@@//x:y"))"#,
        Ok("False"),
    ),
    (
        r#"print(attr.label(default="@nope//x:y") == attr.label(default="@nope//x:y"))"#,
        Ok("True"),
    ),
    (
        r#"print(attr.label(default="@nope//x:y"))"#,
        Ok("<attr.label>"),
    ),
    (
        r#"print(attr.label_list(default=["@nope//x:y"]) == attr.label_list(default=["@nope//x:y"]))"#,
        Ok("True"),
    ),
];

#[test]
fn a_label_default_is_read_in_the_repo_of_the_bzl_that_wrote_it() {
    assert_replays(replay_in("dep+", "sub", IN_DEP, &[], &[]));
}

/// What `x` holds in `src`, a `.bzl` in `package` of `repo`.
fn kept<R>(
    repo: &str,
    package: &str,
    src: &str,
    check: impl FnOnce(crate::attr::AttributeView<'_, '_>) -> R,
) -> R {
    let module = module_in(repo, package, src).unwrap();
    let owned = module.get("x").unwrap();
    let x: Value = owned.value();
    check(view(x).expect("an attribute descriptor"))
}

fn label(repo: &str, package: &str, name: &str) -> Label {
    Label {
        repo: repo.into(),
        package: package.into(),
        name: name.into(),
    }
}

#[test]
fn a_label_attribute_keeps_what_it_was_given() {
    kept(
        "dep+",
        "sub",
        r#"x = attr.label(default=":d", doc="what", mandatory=True, allow_files=[".x", ".y"],
            allow_rules=["r", "q"], flags=["ORDER_INDEPENDENT"], cfg="exec",
            skip_validations=True, configurable=False, executable=True)"#,
        |a| {
            let def = a.def;
            assert_eq!(def.ty, AttrType::Label);
            assert_eq!(
                def.default,
                Some(AttrValue::Label(label("dep+", "sub", "d")))
            );
            assert_eq!(def.doc.as_deref(), Some("what"));
            assert!(def.mandatory() && def.executable() && !def.single_file());
            assert!(def.flags.contains(&AttrFlag::OrderIndependent));
            assert!(def.flags.contains(&AttrFlag::StarlarkDefined));
            assert_eq!(
                def.files,
                FileTypes::Suffixes(vec![".x".into(), ".y".into()])
            );
            assert_eq!(
                def.allow_rules,
                Some(["q".to_owned(), "r".to_owned()].into_iter().collect())
            );
            assert_eq!(def.cfg, Cfg::Exec);
            assert_eq!(def.configurable, Some(false));
            assert!(def.skip_validations && !def.computed_default);
            assert_eq!(a.builder, "label");
        },
    );
}

#[test]
fn a_keyword_and_its_flag_are_one_thing() {
    kept(
        "",
        "",
        r#"x = attr.label_list(allow_empty=False, flags=["MANDATORY"])"#,
        |a| {
            assert!(a.def.mandatory() && !a.def.allow_empty());
            assert!(!a.def.single_file());
        },
    );
    kept("", "", "x = attr.string_list(True, False)", |a| {
        assert!(a.def.mandatory() && !a.def.allow_empty());
    });
    kept("", "", r#"x = attr.label(allow_single_file=[".a"])"#, |a| {
        assert!(a.def.single_file());
        assert_eq!(a.def.files, FileTypes::Suffixes(vec![".a".into()]));
    });
    kept("", "", "x = attr.label(allow_single_file=False)", |a| {
        assert!(a.def.single_file());
        assert_eq!(a.def.files, FileTypes::None);
    });
    kept("", "", "x = attr.label(allow_single_file=True)", |a| {
        assert_eq!(a.def.files, FileTypes::Any);
    });
    kept("", "", "x = attr.label()", |a| {
        assert!(!a.def.single_file() && !a.def.mandatory() && a.def.allow_empty());
        assert_eq!(a.def.files, FileTypes::None);
        assert_eq!(a.def.cfg, Cfg::Target);
    });
}

#[test]
fn a_default_is_converted_to_what_the_attribute_holds() {
    kept("", "", r#"x = attr.int_list(default=(1, -2))"#, |a| {
        assert_eq!(a.def.default, Some(AttrValue::IntList(vec![1, -2])));
    });
    kept(
        "dep+",
        "sub",
        r#"x = attr.label_keyed_string_dict(default={"//x:y": "a", Label("@@//p:q"): "b", ":z": "c"})"#,
        |a| {
            assert_eq!(
                a.def.default,
                Some(AttrValue::LabelKeyedStringDict(vec![
                    (label("dep+", "x", "y"), "a".into()),
                    (label("", "p", "q"), "b".into()),
                    (label("dep+", "sub", "z"), "c".into()),
                ]))
            );
        },
    );
    kept(
        "",
        "",
        r#"x = attr.label_list_dict(default={"k": ["//a:b", Label("//c:d")], "j": depset(["//e:f"])})"#,
        |a| {
            assert_eq!(
                a.def.default,
                Some(AttrValue::LabelListDict(vec![
                    ("k".into(), vec![label("", "a", "b"), label("", "c", "d")]),
                    ("j".into(), vec![label("", "e", "f")]),
                ]))
            );
        },
    );
    kept(
        "",
        "",
        r#"x = attr.string_list_dict(default={"a": ("b", "c")})"#,
        |a| {
            assert_eq!(
                a.def.default,
                Some(AttrValue::StringListDict(vec![(
                    "a".into(),
                    vec!["b".into(), "c".into()]
                )]))
            );
        },
    );
    kept("", "", "x = attr.string(default='v')", |a| {
        assert_eq!(a.def.default_value(), Some(AttrValue::String("v".into())));
    });
    kept("", "", "x = attr.string()", |a| {
        assert_eq!(a.def.default, None);
        assert_eq!(a.def.default_value(), Some(AttrValue::String("".into())));
    });
    kept("", "", "x = attr.label()", |a| {
        assert_eq!(a.def.default_value(), None);
    });
}

#[test]
fn a_function_default_is_kept_and_not_called() {
    kept("", "", r#"x = attr.label(default=lambda a: a.nope)"#, |a| {
        assert!(a.def.computed_default);
        assert_eq!(a.def.default_value(), None);
        assert!(a.computed.is_some());
    });
    kept(
        "",
        "",
        r#"x = attr.label_list(default=lambda a: a.nope)"#,
        |a| {
            assert!(a.def.computed_default && a.computed.is_some());
        },
    );
}

#[test]
fn values_are_kept_as_given() {
    kept(
        "",
        "",
        r#"x = attr.string(values=["a", "b"], default="c")"#,
        |a| {
            let shown: Vec<String> = a.values.iter().map(|v| v.to_repr()).collect();
            assert_eq!(shown, ["\"a\"", "\"b\""]);
        },
    );
    kept("", "", "x = attr.int(values=[1, 2])", |a| {
        assert_eq!(a.values.len(), 2);
    });
}

#[test]
fn descriptors_freeze_and_keep_their_equality() {
    let module = module_in(
        "",
        "",
        r#"
x = attr.string(default = "a", doc = "d")
y = attr.string(default = "a", doc = "d")
z = attr.string(default = "b")
w = attr.string(values = ["a"])
l = [x, {"k": z}]
"#,
    )
    .unwrap();
    let get = |name: &str| module.get(name).unwrap();
    let (x, y, z, w, l) = (get("x"), get("y"), get("z"), get("w"), get("l"));
    let (x, y, z, w, l) = (x.value(), y.value(), z.value(), w.value(), l.value());
    assert!(x.equals(x).unwrap());
    assert!(x.equals(y).unwrap());
    assert!(!x.equals(z).unwrap());
    // A value list is compared by identity, so not even a copy is equal.
    assert!(w.equals(w).unwrap());
    assert!(!w.equals(x).unwrap());
    assert_eq!(x.to_string(), "<attr.string>");
    assert_eq!(x.get_type(), "Attribute");
    assert_eq!(l.to_repr(), "[<attr.string>, {\"k\": <attr.string>}]");
    assert!(view(x).is_some());
    assert!(view(l).is_none());
}

#[test]
fn attr_is_a_global_of_a_bzl_and_not_of_a_build_file() {
    assert!(bzl_globals().names().any(|n| n.as_str() == "attr"));
    assert!(!build_globals().names().any(|n| n.as_str() == "attr"));
    assert_eq!(
        bzl_globals()
            .iter()
            .find(|(name, _)| *name == "attr")
            .map(|(_, v)| v.to_value().get_type()),
        Some("attr")
    );
}

const ATTR_CASES: &[(&str, Result<&str, &str>)] = &[
    (r#"print(attr)"#, Ok(r#"<attr>"#)),
    (r#"print(type(attr))"#, Ok(r#"attr"#)),
    (
        r#"print(dir(attr))"#,
        Ok(
            r#"["bool", "int", "int_list", "label", "label_keyed_string_dict", "label_list", "label_list_dict", "output", "output_list", "string", "string_dict", "string_keyed_label_dict", "string_list", "string_list_dict"]"#,
        ),
    ),
    (r#"print(attr.string())"#, Ok(r#"<attr.string>"#)),
    (r#"print(type(attr.string()))"#, Ok(r#"Attribute"#)),
    (r#"print(repr(attr.string()))"#, Ok(r#"<attr.string>"#)),
    (r#"print(str(attr.string()))"#, Ok(r#"<attr.string>"#)),
    (r#"print(attr.string() == attr.string())"#, Ok(r#"True"#)),
    (
        r#"x = attr.string()
print(x == x)"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(hash(attr.string()) != 0)"#,
        Err(r#"in call to hash(), parameter 'value' got value of type 'Attribute', want 'string'"#),
    ),
    (r#"print(dir(attr.string()))"#, Ok(r#"[]"#)),
    (
        r#"print(hasattr(attr.string(), "default"))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print([attr.int(), attr.bool()])"#,
        Ok(r#"[<attr.int>, <attr.bool>]"#),
    ),
    (
        r#"print({attr.int(): 1})"#,
        Err(r#"unhashable type: 'Attribute'"#),
    ),
    (r#"print(bool(attr.int()))"#, Ok(r#"True"#)),
    (
        r#"print(attr.int() + 1)"#,
        Err(r#"unsupported binary operation: Attribute + int"#),
    ),
    (
        r#"print(attr.int()[0])"#,
        Err(r#"type 'Attribute' has no operator [](int)"#),
    ),
    (
        r#"print(attr.int().foo)"#,
        Err(r#"'Attribute' value has no field or method 'foo'"#),
    ),
    (
        r#"print(attr.string().default)"#,
        Err(r#"'Attribute' value has no field or method 'default'"#),
    ),
    (
        r#"print(attr.string(1))"#,
        Err(r#"string() got unexpected positional argument"#),
    ),
    (
        r#"print(attr.string("a"))"#,
        Err(r#"string() got unexpected positional argument"#),
    ),
    (
        r#"print(attr.string(default=1))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'int', want 'string'"#),
    ),
    (
        r#"print(attr.string(default=None))"#,
        Err(
            r#"in call to string(), parameter 'default' got value of type 'NoneType', want 'string'"#,
        ),
    ),
    (r#"print(attr.string(default=""))"#, Ok(r#"<attr.string>"#)),
    (r#"print(attr.string(default="a"))"#, Ok(r#"<attr.string>"#)),
    (
        r#"print(attr.string(foo=1))"#,
        Err(r#"string() got unexpected keyword argument 'foo'"#),
    ),
    (
        r#"print(attr.string(defualt="a"))"#,
        Err(r#"string() got unexpected keyword argument 'defualt' (did you mean 'default'?)"#),
    ),
    (
        r#"print(attr.string(doc=1))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (r#"print(attr.string(doc=None))"#, Ok(r#"<attr.string>"#)),
    (r#"print(attr.string(doc="x"))"#, Ok(r#"<attr.string>"#)),
    (
        r#"print(attr.string(mandatory=1))"#,
        Err(r#"in call to string(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.string(mandatory=None))"#,
        Err(
            r#"in call to string(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string(mandatory=True))"#,
        Ok(r#"<attr.string>"#),
    ),
    (
        r#"print(attr.string(mandatory=True, default="a"))"#,
        Ok(r#"<attr.string>"#),
    ),
    (
        r#"print(attr.string(values=1))"#,
        Err(r#"in call to string(), parameter 'values' got value of type 'int', want 'sequence'"#),
    ),
    (r#"print(attr.string(values=[]))"#, Ok(r#"<attr.string>"#)),
    (
        r#"print(attr.string(values=["a"]))"#,
        Ok(r#"<attr.string>"#),
    ),
    (
        r#"print(attr.string(values=["a"], default="b"))"#,
        Ok(r#"<attr.string>"#),
    ),
    (
        r#"print(attr.string(values=["a"], default="a"))"#,
        Ok(r#"<attr.string>"#),
    ),
    (r#"print(attr.string(values=[1]))"#, Ok(r#"<attr.string>"#)),
    (
        r#"print(attr.string(values=("a","b")))"#,
        Ok(r#"<attr.string>"#),
    ),
    (
        r#"print(attr.string(values=depset(["a"])))"#,
        Err(
            r#"in call to string(), parameter 'values' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string(values=["a","a"]))"#,
        Ok(r#"<attr.string>"#),
    ),
    (
        r#"print(attr.string(values={"a":1}))"#,
        Err(r#"in call to string(), parameter 'values' got value of type 'dict', want 'sequence'"#),
    ),
    (
        r#"print(attr.string(values="a"))"#,
        Err(
            r#"in call to string(), parameter 'values' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string(configurable=False))"#,
        Ok(r#"<attr.string>"#),
    ),
    (
        r#"print(attr.string(configurable=1))"#,
        Err(
            r#"in call to string(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string(configurable=True))"#,
        Ok(r#"<attr.string>"#),
    ),
    (
        r#"print(attr.string(default=select({"//conditions:default": "a"})))"#,
        Err(
            r#"in call to string(), parameter 'default' got value of type 'select', want 'string'"#,
        ),
    ),
    (
        r#"print(attr.string(default=select({"//conditions:default": "a"}), configurable=False))"#,
        Err(
            r#"in call to string(), parameter 'default' got value of type 'select', want 'string'"#,
        ),
    ),
    (
        r#"print(attr.string(default=lambda: "a"))"#,
        Err(
            r#"in call to string(), parameter 'default' got value of type 'function', want 'string'"#,
        ),
    ),
    (
        r#"print(attr.string(default=lambda x: "a"))"#,
        Err(
            r#"in call to string(), parameter 'default' got value of type 'function', want 'string'"#,
        ),
    ),
    (
        r#"print(attr.string(default=lambda **k: "a"))"#,
        Err(
            r#"in call to string(), parameter 'default' got value of type 'function', want 'string'"#,
        ),
    ),
    (
        r#"print(attr.string(default=print))"#,
        Err(
            r#"in call to string(), parameter 'default' got value of type 'builtin_function_or_method', want 'string'"#,
        ),
    ),
    (
        r#"print(attr.string(default=["a"]))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'list', want 'string'"#),
    ),
    (
        r#"print(attr.string(default=Label("//a:b")))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'Label', want 'string'"#),
    ),
    (
        r#"print(attr.string(default=1.5))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'float', want 'string'"#),
    ),
    (
        r#"print(attr.string(default=True))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'bool', want 'string'"#),
    ),
    (
        r#"print(attr.string(flags=["x"]))"#,
        Err(r#"string() got unexpected keyword argument 'flags'"#),
    ),
    (
        r#"print(attr.string(flags=[]))"#,
        Err(r#"string() got unexpected keyword argument 'flags'"#),
    ),
    (
        r#"print(attr.string(cfg="exec"))"#,
        Err(r#"string() got unexpected keyword argument 'cfg'"#),
    ),
    (
        r#"print(attr.string(executable=True))"#,
        Err(r#"string() got unexpected keyword argument 'executable'"#),
    ),
    (
        r#"print(attr.string(allow_files=True))"#,
        Err(r#"string() got unexpected keyword argument 'allow_files'"#),
    ),
    (
        r#"print(attr.string(providers=[]))"#,
        Err(r#"string() got unexpected keyword argument 'providers'"#),
    ),
    (
        r#"print(attr.string(aspects=[]))"#,
        Err(r#"string() got unexpected keyword argument 'aspects'"#),
    ),
    (
        r#"print(attr.string(allow_empty=False))"#,
        Err(r#"string() got unexpected keyword argument 'allow_empty'"#),
    ),
    (
        r#"print(attr.string(allow_single_file=True))"#,
        Err(r#"string() got unexpected keyword argument 'allow_single_file'"#),
    ),
    (
        r#"print(attr.bool(1))"#,
        Err(r#"bool() got unexpected positional argument"#),
    ),
    (
        r#"print(attr.bool(1, 2))"#,
        Err(r#"bool() got unexpected positional argument"#),
    ),
    (
        r#"print(attr.bool(default=None))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'NoneType', want 'bool'"#),
    ),
    (r#"print(attr.bool(doc=None))"#, Ok(r#"<attr.bool>"#)),
    (
        r#"print(attr.bool(mandatory=None))"#,
        Err(
            r#"in call to bool(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.bool(values=None))"#,
        Err(r#"bool() got unexpected keyword argument 'values'"#),
    ),
    (
        r#"print(attr.bool(configurable=None))"#,
        Err(
            r#"in call to bool(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.bool(allow_empty=None))"#,
        Err(r#"bool() got unexpected keyword argument 'allow_empty'"#),
    ),
    (
        r#"print(attr.bool(allow_files=None))"#,
        Err(r#"bool() got unexpected keyword argument 'allow_files'"#),
    ),
    (
        r#"print(attr.bool(allow_single_file=None))"#,
        Err(r#"bool() got unexpected keyword argument 'allow_single_file'"#),
    ),
    (
        r#"print(attr.bool(allow_rules=None))"#,
        Err(r#"bool() got unexpected keyword argument 'allow_rules'"#),
    ),
    (
        r#"print(attr.bool(providers=None))"#,
        Err(r#"bool() got unexpected keyword argument 'providers'"#),
    ),
    (
        r#"print(attr.bool(flags=None))"#,
        Err(r#"bool() got unexpected keyword argument 'flags'"#),
    ),
    (
        r#"print(attr.bool(cfg=None))"#,
        Err(r#"bool() got unexpected keyword argument 'cfg'"#),
    ),
    (
        r#"print(attr.bool(aspects=None))"#,
        Err(r#"bool() got unexpected keyword argument 'aspects'"#),
    ),
    (
        r#"print(attr.bool(executable=None))"#,
        Err(r#"bool() got unexpected keyword argument 'executable'"#),
    ),
    (
        r#"print(attr.bool(skip_validations=None))"#,
        Err(r#"bool() got unexpected keyword argument 'skip_validations'"#),
    ),
    (
        r#"print(attr.bool(materializer=None))"#,
        Err(r#"bool() got unexpected keyword argument 'materializer'"#),
    ),
    (
        r#"print(attr.bool(for_dependency_resolution=None))"#,
        Err(r#"bool() got unexpected keyword argument 'for_dependency_resolution'"#),
    ),
    (
        r#"print(attr.bool(name=None))"#,
        Err(r#"bool() got unexpected keyword argument 'name'"#),
    ),
    (
        r#"print(attr.bool(allowlist=None))"#,
        Err(r#"bool() got unexpected keyword argument 'allowlist'"#),
    ),
    (
        r#"print(attr.bool(nonempty=None))"#,
        Err(r#"bool() got unexpected keyword argument 'nonempty'"#),
    ),
    (
        r#"print(attr.bool(single_file=None))"#,
        Err(r#"bool() got unexpected keyword argument 'single_file'"#),
    ),
    (
        r#"print(attr.bool(non_empty=None))"#,
        Err(r#"bool() got unexpected keyword argument 'non_empty'"#),
    ),
    (
        r#"print(attr.bool(order=None))"#,
        Err(r#"bool() got unexpected keyword argument 'order'"#),
    ),
    (
        r#"print(attr.bool(output_to_genfiles=None))"#,
        Err(r#"bool() got unexpected keyword argument 'output_to_genfiles'"#),
    ),
    (
        r#"print(attr.bool(default_provider=None))"#,
        Err(r#"bool() got unexpected keyword argument 'default_provider'"#),
    ),
    (
        r#"print(attr.bool(container=None))"#,
        Err(r#"bool() got unexpected keyword argument 'container'"#),
    ),
    (
        r#"print(attr.bool(exec_group=None))"#,
        Err(r#"bool() got unexpected keyword argument 'exec_group'"#),
    ),
    (
        r#"print(attr.bool(kind=None))"#,
        Err(r#"bool() got unexpected keyword argument 'kind'"#),
    ),
    (
        r#"print(attr.bool(visibility=None))"#,
        Err(r#"bool() got unexpected keyword argument 'visibility'"#),
    ),
    (
        r#"print(attr.bool(tags=None))"#,
        Err(r#"bool() got unexpected keyword argument 'tags'"#),
    ),
    (
        r#"print(attr.int(1))"#,
        Err(r#"int() got unexpected positional argument"#),
    ),
    (
        r#"print(attr.int(1, 2))"#,
        Err(r#"int() got unexpected positional argument"#),
    ),
    (
        r#"print(attr.int(default=None))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'NoneType', want 'int'"#),
    ),
    (r#"print(attr.int(doc=None))"#, Ok(r#"<attr.int>"#)),
    (
        r#"print(attr.int(mandatory=None))"#,
        Err(r#"in call to int(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#),
    ),
    (
        r#"print(attr.int(values=None))"#,
        Err(
            r#"in call to int(), parameter 'values' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int(configurable=None))"#,
        Err(
            r#"in call to int(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int(allow_empty=None))"#,
        Err(r#"int() got unexpected keyword argument 'allow_empty'"#),
    ),
    (
        r#"print(attr.int(allow_files=None))"#,
        Err(r#"int() got unexpected keyword argument 'allow_files'"#),
    ),
    (
        r#"print(attr.int(allow_single_file=None))"#,
        Err(r#"int() got unexpected keyword argument 'allow_single_file'"#),
    ),
    (
        r#"print(attr.int(allow_rules=None))"#,
        Err(r#"int() got unexpected keyword argument 'allow_rules'"#),
    ),
    (
        r#"print(attr.int(providers=None))"#,
        Err(r#"int() got unexpected keyword argument 'providers'"#),
    ),
    (
        r#"print(attr.int(flags=None))"#,
        Err(r#"int() got unexpected keyword argument 'flags'"#),
    ),
    (
        r#"print(attr.int(cfg=None))"#,
        Err(r#"int() got unexpected keyword argument 'cfg'"#),
    ),
    (
        r#"print(attr.int(aspects=None))"#,
        Err(r#"int() got unexpected keyword argument 'aspects'"#),
    ),
    (
        r#"print(attr.int(executable=None))"#,
        Err(r#"int() got unexpected keyword argument 'executable'"#),
    ),
    (
        r#"print(attr.int(skip_validations=None))"#,
        Err(r#"int() got unexpected keyword argument 'skip_validations'"#),
    ),
    (
        r#"print(attr.int(materializer=None))"#,
        Err(r#"int() got unexpected keyword argument 'materializer'"#),
    ),
    (
        r#"print(attr.int(for_dependency_resolution=None))"#,
        Err(r#"int() got unexpected keyword argument 'for_dependency_resolution'"#),
    ),
    (
        r#"print(attr.int(name=None))"#,
        Err(r#"int() got unexpected keyword argument 'name'"#),
    ),
    (
        r#"print(attr.int(allowlist=None))"#,
        Err(r#"int() got unexpected keyword argument 'allowlist'"#),
    ),
    (
        r#"print(attr.int(nonempty=None))"#,
        Err(r#"int() got unexpected keyword argument 'nonempty'"#),
    ),
    (
        r#"print(attr.int(single_file=None))"#,
        Err(r#"int() got unexpected keyword argument 'single_file'"#),
    ),
    (
        r#"print(attr.int(non_empty=None))"#,
        Err(r#"int() got unexpected keyword argument 'non_empty'"#),
    ),
    (
        r#"print(attr.int(order=None))"#,
        Err(r#"int() got unexpected keyword argument 'order'"#),
    ),
    (
        r#"print(attr.int(output_to_genfiles=None))"#,
        Err(r#"int() got unexpected keyword argument 'output_to_genfiles'"#),
    ),
    (
        r#"print(attr.int(default_provider=None))"#,
        Err(r#"int() got unexpected keyword argument 'default_provider'"#),
    ),
    (
        r#"print(attr.int(container=None))"#,
        Err(r#"int() got unexpected keyword argument 'container'"#),
    ),
    (
        r#"print(attr.int(exec_group=None))"#,
        Err(r#"int() got unexpected keyword argument 'exec_group'"#),
    ),
    (
        r#"print(attr.int(kind=None))"#,
        Err(r#"int() got unexpected keyword argument 'kind'"#),
    ),
    (
        r#"print(attr.int(visibility=None))"#,
        Err(r#"int() got unexpected keyword argument 'visibility'"#),
    ),
    (
        r#"print(attr.int(tags=None))"#,
        Err(r#"int() got unexpected keyword argument 'tags'"#),
    ),
    (
        r#"print(attr.int_list(1))"#,
        Err(r#"in call to int_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.int_list(1, 2))"#,
        Err(r#"in call to int_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.int_list(default=None))"#,
        Err(
            r#"in call to int_list(), parameter 'default' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int_list(doc=None))"#,
        Ok(r#"<attr.int_list>"#),
    ),
    (
        r#"print(attr.int_list(mandatory=None))"#,
        Err(
            r#"in call to int_list(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(values=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'values'"#),
    ),
    (
        r#"print(attr.int_list(configurable=None))"#,
        Err(
            r#"in call to int_list(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.int_list(allow_empty=None))"#,
        Err(
            r#"in call to int_list(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(allow_files=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'allow_files'"#),
    ),
    (
        r#"print(attr.int_list(allow_single_file=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'allow_single_file'"#),
    ),
    (
        r#"print(attr.int_list(allow_rules=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'allow_rules'"#),
    ),
    (
        r#"print(attr.int_list(providers=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'providers'"#),
    ),
    (
        r#"print(attr.int_list(flags=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'flags'"#),
    ),
    (
        r#"print(attr.int_list(cfg=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'cfg'"#),
    ),
    (
        r#"print(attr.int_list(aspects=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'aspects'"#),
    ),
    (
        r#"print(attr.int_list(executable=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'executable'"#),
    ),
    (
        r#"print(attr.int_list(skip_validations=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'skip_validations'"#),
    ),
    (
        r#"print(attr.int_list(materializer=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'materializer'"#),
    ),
    (
        r#"print(attr.int_list(for_dependency_resolution=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'for_dependency_resolution'"#),
    ),
    (
        r#"print(attr.int_list(name=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'name'"#),
    ),
    (
        r#"print(attr.int_list(allowlist=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'allowlist'"#),
    ),
    (
        r#"print(attr.int_list(nonempty=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'nonempty'"#),
    ),
    (
        r#"print(attr.int_list(single_file=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'single_file'"#),
    ),
    (
        r#"print(attr.int_list(non_empty=None))"#,
        Err(
            r#"int_list() got unexpected keyword argument 'non_empty' (did you mean 'allow_empty'?)"#,
        ),
    ),
    (
        r#"print(attr.int_list(order=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'order'"#),
    ),
    (
        r#"print(attr.int_list(output_to_genfiles=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'output_to_genfiles'"#),
    ),
    (
        r#"print(attr.int_list(default_provider=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'default_provider'"#),
    ),
    (
        r#"print(attr.int_list(container=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'container'"#),
    ),
    (
        r#"print(attr.int_list(exec_group=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'exec_group'"#),
    ),
    (
        r#"print(attr.int_list(kind=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'kind'"#),
    ),
    (
        r#"print(attr.int_list(visibility=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'visibility'"#),
    ),
    (
        r#"print(attr.int_list(tags=None))"#,
        Err(r#"int_list() got unexpected keyword argument 'tags'"#),
    ),
    (
        r#"print(attr.label(1))"#,
        Err(r#"label() got unexpected positional argument"#),
    ),
    (
        r#"print(attr.label(1, 2))"#,
        Err(r#"label() got unexpected positional argument"#),
    ),
    (r#"print(attr.label(default=None))"#, Ok(r#"<attr.label>"#)),
    (r#"print(attr.label(doc=None))"#, Ok(r#"<attr.label>"#)),
    (
        r#"print(attr.label(mandatory=None))"#,
        Err(
            r#"in call to label(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(values=None))"#,
        Err(r#"label() got unexpected keyword argument 'values'"#),
    ),
    (
        r#"print(attr.label(configurable=None))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_empty=None))"#,
        Err(r#"label() got unexpected keyword argument 'allow_empty'"#),
    ),
    (
        r#"print(attr.label(allow_files=None))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_single_file=None))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_rules=None))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(providers=None))"#,
        Err(
            r#"in call to label(), parameter 'providers' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(flags=None))"#,
        Err(
            r#"in call to label(), parameter 'flags' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (r#"print(attr.label(cfg=None))"#, Ok(r#"<attr.label>"#)),
    (
        r#"print(attr.label(aspects=None))"#,
        Err(
            r#"in call to label(), parameter 'aspects' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(executable=None))"#,
        Err(
            r#"in call to label(), parameter 'executable' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(skip_validations=None))"#,
        Err(
            r#"in call to label(), parameter 'skip_validations' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label(materializer=None))"#,
        Err(
            r#"in call to label(), parameter 'materializer' is experimental and thus unavailable with the current flags. It may be enabled by setting --experimental_dormant_deps"#,
        ),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=None))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(name=None))"#,
        Err(r#"label() got unexpected keyword argument 'name'"#),
    ),
    (
        r#"print(attr.label(allowlist=None))"#,
        Err(r#"label() got unexpected keyword argument 'allowlist'"#),
    ),
    (
        r#"print(attr.label(nonempty=None))"#,
        Err(r#"label() got unexpected keyword argument 'nonempty'"#),
    ),
    (
        r#"print(attr.label(single_file=None))"#,
        Err(r#"label() got unexpected keyword argument 'single_file'"#),
    ),
    (
        r#"print(attr.label(non_empty=None))"#,
        Err(r#"label() got unexpected keyword argument 'non_empty'"#),
    ),
    (
        r#"print(attr.label(order=None))"#,
        Err(r#"label() got unexpected keyword argument 'order'"#),
    ),
    (
        r#"print(attr.label(output_to_genfiles=None))"#,
        Err(r#"label() got unexpected keyword argument 'output_to_genfiles'"#),
    ),
    (
        r#"print(attr.label(default_provider=None))"#,
        Err(r#"label() got unexpected keyword argument 'default_provider'"#),
    ),
    (
        r#"print(attr.label(container=None))"#,
        Err(r#"label() got unexpected keyword argument 'container'"#),
    ),
    (
        r#"print(attr.label(exec_group=None))"#,
        Err(r#"label() got unexpected keyword argument 'exec_group'"#),
    ),
    (
        r#"print(attr.label(kind=None))"#,
        Err(r#"label() got unexpected keyword argument 'kind'"#),
    ),
    (
        r#"print(attr.label(visibility=None))"#,
        Err(r#"label() got unexpected keyword argument 'visibility'"#),
    ),
    (
        r#"print(attr.label(tags=None))"#,
        Err(r#"label() got unexpected keyword argument 'tags'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(1, 2))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=None))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'NoneType', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=None))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=None))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(values=None))"#,
        Err(r#"label_keyed_string_dict() got unexpected keyword argument 'values'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=None))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=None))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=None))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_single_file=None))"#,
        Err(r#"label_keyed_string_dict() got unexpected keyword argument 'allow_single_file'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=None))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers=None))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'providers' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags=None))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'flags' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg=None))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects=None))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'aspects' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(executable=None))"#,
        Err(r#"label_keyed_string_dict() got unexpected keyword argument 'executable'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(skip_validations=None))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'skip_validations' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(materializer=None))"#,
        Err(r#"label_keyed_string_dict() got unexpected keyword argument 'materializer'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(for_dependency_resolution=None))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(name=None))"#,
        Err(r#"label_keyed_string_dict() got unexpected keyword argument 'name'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allowlist=None))"#,
        Err(r#"label_keyed_string_dict() got unexpected keyword argument 'allowlist'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(nonempty=None))"#,
        Err(r#"label_keyed_string_dict() got unexpected keyword argument 'nonempty'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(single_file=None))"#,
        Err(r#"label_keyed_string_dict() got unexpected keyword argument 'single_file'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(non_empty=None))"#,
        Err(
            r#"label_keyed_string_dict() got unexpected keyword argument 'non_empty' (did you mean 'allow_empty'?)"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(order=None))"#,
        Err(r#"label_keyed_string_dict() got unexpected keyword argument 'order'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(output_to_genfiles=None))"#,
        Err(r#"label_keyed_string_dict() got unexpected keyword argument 'output_to_genfiles'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default_provider=None))"#,
        Err(r#"label_keyed_string_dict() got unexpected keyword argument 'default_provider'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(container=None))"#,
        Err(r#"label_keyed_string_dict() got unexpected keyword argument 'container'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(exec_group=None))"#,
        Err(r#"label_keyed_string_dict() got unexpected keyword argument 'exec_group'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(kind=None))"#,
        Err(r#"label_keyed_string_dict() got unexpected keyword argument 'kind'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(visibility=None))"#,
        Err(r#"label_keyed_string_dict() got unexpected keyword argument 'visibility'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(tags=None))"#,
        Err(r#"label_keyed_string_dict() got unexpected keyword argument 'tags'"#),
    ),
    (
        r#"print(attr.label_list(1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(1, 2))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=None))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'NoneType', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=None))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(mandatory=None))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(values=None))"#,
        Err(r#"label_list() got unexpected keyword argument 'values'"#),
    ),
    (
        r#"print(attr.label_list(configurable=None))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=None))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files=None))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(allow_single_file=None))"#,
        Err(r#"label_list() got unexpected keyword argument 'allow_single_file'"#),
    ),
    (
        r#"print(attr.label_list(allow_rules=None))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(providers=None))"#,
        Err(
            r#"in call to label_list(), parameter 'providers' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(flags=None))"#,
        Err(
            r#"in call to label_list(), parameter 'flags' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(cfg=None))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(aspects=None))"#,
        Err(
            r#"in call to label_list(), parameter 'aspects' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(executable=None))"#,
        Err(r#"label_list() got unexpected keyword argument 'executable'"#),
    ),
    (
        r#"print(attr.label_list(skip_validations=None))"#,
        Err(
            r#"in call to label_list(), parameter 'skip_validations' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(materializer=None))"#,
        Err(
            r#"in call to label_list(), parameter 'materializer' is experimental and thus unavailable with the current flags. It may be enabled by setting --experimental_dormant_deps"#,
        ),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution=None))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(name=None))"#,
        Err(r#"label_list() got unexpected keyword argument 'name'"#),
    ),
    (
        r#"print(attr.label_list(allowlist=None))"#,
        Err(r#"label_list() got unexpected keyword argument 'allowlist'"#),
    ),
    (
        r#"print(attr.label_list(nonempty=None))"#,
        Err(r#"label_list() got unexpected keyword argument 'nonempty'"#),
    ),
    (
        r#"print(attr.label_list(single_file=None))"#,
        Err(r#"label_list() got unexpected keyword argument 'single_file'"#),
    ),
    (
        r#"print(attr.label_list(non_empty=None))"#,
        Err(
            r#"label_list() got unexpected keyword argument 'non_empty' (did you mean 'allow_empty'?)"#,
        ),
    ),
    (
        r#"print(attr.label_list(order=None))"#,
        Err(r#"label_list() got unexpected keyword argument 'order'"#),
    ),
    (
        r#"print(attr.label_list(output_to_genfiles=None))"#,
        Err(r#"label_list() got unexpected keyword argument 'output_to_genfiles'"#),
    ),
    (
        r#"print(attr.label_list(default_provider=None))"#,
        Err(r#"label_list() got unexpected keyword argument 'default_provider'"#),
    ),
    (
        r#"print(attr.label_list(container=None))"#,
        Err(r#"label_list() got unexpected keyword argument 'container'"#),
    ),
    (
        r#"print(attr.label_list(exec_group=None))"#,
        Err(r#"label_list() got unexpected keyword argument 'exec_group'"#),
    ),
    (
        r#"print(attr.label_list(kind=None))"#,
        Err(r#"label_list() got unexpected keyword argument 'kind'"#),
    ),
    (
        r#"print(attr.label_list(visibility=None))"#,
        Err(r#"label_list() got unexpected keyword argument 'visibility'"#),
    ),
    (
        r#"print(attr.label_list(tags=None))"#,
        Err(r#"label_list() got unexpected keyword argument 'tags'"#),
    ),
    (
        r#"print(attr.label_list_dict(1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(1, 2))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=None))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'NoneType', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=None))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=None))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(values=None))"#,
        Err(r#"label_list_dict() got unexpected keyword argument 'values'"#),
    ),
    (
        r#"print(attr.label_list_dict(configurable=None))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=None))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=None))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(allow_single_file=None))"#,
        Err(r#"label_list_dict() got unexpected keyword argument 'allow_single_file'"#),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=None))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(providers=None))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'providers' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(flags=None))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'flags' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(cfg=None))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(aspects=None))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'aspects' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(executable=None))"#,
        Err(r#"label_list_dict() got unexpected keyword argument 'executable'"#),
    ),
    (
        r#"print(attr.label_list_dict(skip_validations=None))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'skip_validations' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(materializer=None))"#,
        Err(r#"label_list_dict() got unexpected keyword argument 'materializer'"#),
    ),
    (
        r#"print(attr.label_list_dict(for_dependency_resolution=None))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(name=None))"#,
        Err(r#"label_list_dict() got unexpected keyword argument 'name'"#),
    ),
    (
        r#"print(attr.label_list_dict(allowlist=None))"#,
        Err(r#"label_list_dict() got unexpected keyword argument 'allowlist'"#),
    ),
    (
        r#"print(attr.label_list_dict(nonempty=None))"#,
        Err(r#"label_list_dict() got unexpected keyword argument 'nonempty'"#),
    ),
    (
        r#"print(attr.label_list_dict(single_file=None))"#,
        Err(r#"label_list_dict() got unexpected keyword argument 'single_file'"#),
    ),
    (
        r#"print(attr.label_list_dict(non_empty=None))"#,
        Err(
            r#"label_list_dict() got unexpected keyword argument 'non_empty' (did you mean 'allow_empty'?)"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(order=None))"#,
        Err(r#"label_list_dict() got unexpected keyword argument 'order'"#),
    ),
    (
        r#"print(attr.label_list_dict(output_to_genfiles=None))"#,
        Err(r#"label_list_dict() got unexpected keyword argument 'output_to_genfiles'"#),
    ),
    (
        r#"print(attr.label_list_dict(default_provider=None))"#,
        Err(r#"label_list_dict() got unexpected keyword argument 'default_provider'"#),
    ),
    (
        r#"print(attr.label_list_dict(container=None))"#,
        Err(r#"label_list_dict() got unexpected keyword argument 'container'"#),
    ),
    (
        r#"print(attr.label_list_dict(exec_group=None))"#,
        Err(r#"label_list_dict() got unexpected keyword argument 'exec_group'"#),
    ),
    (
        r#"print(attr.label_list_dict(kind=None))"#,
        Err(r#"label_list_dict() got unexpected keyword argument 'kind'"#),
    ),
    (
        r#"print(attr.label_list_dict(visibility=None))"#,
        Err(r#"label_list_dict() got unexpected keyword argument 'visibility'"#),
    ),
    (
        r#"print(attr.label_list_dict(tags=None))"#,
        Err(r#"label_list_dict() got unexpected keyword argument 'tags'"#),
    ),
    (
        r#"print(attr.output(1))"#,
        Err(r#"output() got unexpected positional argument"#),
    ),
    (
        r#"print(attr.output(1, 2))"#,
        Err(r#"output() got unexpected positional argument"#),
    ),
    (
        r#"print(attr.output(default=None))"#,
        Err(r#"output() got unexpected keyword argument 'default'"#),
    ),
    (r#"print(attr.output(doc=None))"#, Ok(r#"<attr.output>"#)),
    (
        r#"print(attr.output(mandatory=None))"#,
        Err(
            r#"in call to output(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output(values=None))"#,
        Err(r#"output() got unexpected keyword argument 'values'"#),
    ),
    (
        r#"print(attr.output(configurable=None))"#,
        Err(r#"output() got unexpected keyword argument 'configurable'"#),
    ),
    (
        r#"print(attr.output(allow_empty=None))"#,
        Err(r#"output() got unexpected keyword argument 'allow_empty'"#),
    ),
    (
        r#"print(attr.output(allow_files=None))"#,
        Err(r#"output() got unexpected keyword argument 'allow_files'"#),
    ),
    (
        r#"print(attr.output(allow_single_file=None))"#,
        Err(r#"output() got unexpected keyword argument 'allow_single_file'"#),
    ),
    (
        r#"print(attr.output(allow_rules=None))"#,
        Err(r#"output() got unexpected keyword argument 'allow_rules'"#),
    ),
    (
        r#"print(attr.output(providers=None))"#,
        Err(r#"output() got unexpected keyword argument 'providers'"#),
    ),
    (
        r#"print(attr.output(flags=None))"#,
        Err(r#"output() got unexpected keyword argument 'flags'"#),
    ),
    (
        r#"print(attr.output(cfg=None))"#,
        Err(r#"output() got unexpected keyword argument 'cfg'"#),
    ),
    (
        r#"print(attr.output(aspects=None))"#,
        Err(r#"output() got unexpected keyword argument 'aspects'"#),
    ),
    (
        r#"print(attr.output(executable=None))"#,
        Err(r#"output() got unexpected keyword argument 'executable'"#),
    ),
    (
        r#"print(attr.output(skip_validations=None))"#,
        Err(r#"output() got unexpected keyword argument 'skip_validations'"#),
    ),
    (
        r#"print(attr.output(materializer=None))"#,
        Err(r#"output() got unexpected keyword argument 'materializer'"#),
    ),
    (
        r#"print(attr.output(for_dependency_resolution=None))"#,
        Err(r#"output() got unexpected keyword argument 'for_dependency_resolution'"#),
    ),
    (
        r#"print(attr.output(name=None))"#,
        Err(r#"output() got unexpected keyword argument 'name'"#),
    ),
    (
        r#"print(attr.output(allowlist=None))"#,
        Err(r#"output() got unexpected keyword argument 'allowlist'"#),
    ),
    (
        r#"print(attr.output(nonempty=None))"#,
        Err(r#"output() got unexpected keyword argument 'nonempty'"#),
    ),
    (
        r#"print(attr.output(single_file=None))"#,
        Err(r#"output() got unexpected keyword argument 'single_file'"#),
    ),
    (
        r#"print(attr.output(non_empty=None))"#,
        Err(r#"output() got unexpected keyword argument 'non_empty'"#),
    ),
    (
        r#"print(attr.output(order=None))"#,
        Err(r#"output() got unexpected keyword argument 'order'"#),
    ),
    (
        r#"print(attr.output(output_to_genfiles=None))"#,
        Err(r#"output() got unexpected keyword argument 'output_to_genfiles'"#),
    ),
    (
        r#"print(attr.output(default_provider=None))"#,
        Err(r#"output() got unexpected keyword argument 'default_provider'"#),
    ),
    (
        r#"print(attr.output(container=None))"#,
        Err(r#"output() got unexpected keyword argument 'container'"#),
    ),
    (
        r#"print(attr.output(exec_group=None))"#,
        Err(r#"output() got unexpected keyword argument 'exec_group'"#),
    ),
    (
        r#"print(attr.output(kind=None))"#,
        Err(r#"output() got unexpected keyword argument 'kind'"#),
    ),
    (
        r#"print(attr.output(visibility=None))"#,
        Err(r#"output() got unexpected keyword argument 'visibility'"#),
    ),
    (
        r#"print(attr.output(tags=None))"#,
        Err(r#"output() got unexpected keyword argument 'tags'"#),
    ),
    (
        r#"print(attr.output_list(1))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(1, 2))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(default=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'default'"#),
    ),
    (
        r#"print(attr.output_list(doc=None))"#,
        Ok(r#"<attr.output_list>"#),
    ),
    (
        r#"print(attr.output_list(mandatory=None))"#,
        Err(
            r#"in call to output_list(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(values=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'values'"#),
    ),
    (
        r#"print(attr.output_list(configurable=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'configurable'"#),
    ),
    (
        r#"print(attr.output_list(allow_empty=None))"#,
        Err(
            r#"in call to output_list(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.output_list(allow_files=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'allow_files'"#),
    ),
    (
        r#"print(attr.output_list(allow_single_file=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'allow_single_file'"#),
    ),
    (
        r#"print(attr.output_list(allow_rules=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'allow_rules'"#),
    ),
    (
        r#"print(attr.output_list(providers=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'providers'"#),
    ),
    (
        r#"print(attr.output_list(flags=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'flags'"#),
    ),
    (
        r#"print(attr.output_list(cfg=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'cfg'"#),
    ),
    (
        r#"print(attr.output_list(aspects=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'aspects'"#),
    ),
    (
        r#"print(attr.output_list(executable=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'executable'"#),
    ),
    (
        r#"print(attr.output_list(skip_validations=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'skip_validations'"#),
    ),
    (
        r#"print(attr.output_list(materializer=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'materializer'"#),
    ),
    (
        r#"print(attr.output_list(for_dependency_resolution=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'for_dependency_resolution'"#),
    ),
    (
        r#"print(attr.output_list(name=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'name'"#),
    ),
    (
        r#"print(attr.output_list(allowlist=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'allowlist'"#),
    ),
    (
        r#"print(attr.output_list(nonempty=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'nonempty'"#),
    ),
    (
        r#"print(attr.output_list(single_file=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'single_file'"#),
    ),
    (
        r#"print(attr.output_list(non_empty=None))"#,
        Err(
            r#"output_list() got unexpected keyword argument 'non_empty' (did you mean 'allow_empty'?)"#,
        ),
    ),
    (
        r#"print(attr.output_list(order=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'order'"#),
    ),
    (
        r#"print(attr.output_list(output_to_genfiles=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'output_to_genfiles'"#),
    ),
    (
        r#"print(attr.output_list(default_provider=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'default_provider'"#),
    ),
    (
        r#"print(attr.output_list(container=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'container'"#),
    ),
    (
        r#"print(attr.output_list(exec_group=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'exec_group'"#),
    ),
    (
        r#"print(attr.output_list(kind=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'kind'"#),
    ),
    (
        r#"print(attr.output_list(visibility=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'visibility'"#),
    ),
    (
        r#"print(attr.output_list(tags=None))"#,
        Err(r#"output_list() got unexpected keyword argument 'tags'"#),
    ),
    (
        r#"print(attr.string(1, 2))"#,
        Err(r#"string() got unexpected positional argument"#),
    ),
    (
        r#"print(attr.string(values=None))"#,
        Err(
            r#"in call to string(), parameter 'values' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string(configurable=None))"#,
        Err(
            r#"in call to string(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string(allow_empty=None))"#,
        Err(r#"string() got unexpected keyword argument 'allow_empty'"#),
    ),
    (
        r#"print(attr.string(allow_files=None))"#,
        Err(r#"string() got unexpected keyword argument 'allow_files'"#),
    ),
    (
        r#"print(attr.string(allow_single_file=None))"#,
        Err(r#"string() got unexpected keyword argument 'allow_single_file'"#),
    ),
    (
        r#"print(attr.string(allow_rules=None))"#,
        Err(r#"string() got unexpected keyword argument 'allow_rules'"#),
    ),
    (
        r#"print(attr.string(providers=None))"#,
        Err(r#"string() got unexpected keyword argument 'providers'"#),
    ),
    (
        r#"print(attr.string(flags=None))"#,
        Err(r#"string() got unexpected keyword argument 'flags'"#),
    ),
    (
        r#"print(attr.string(cfg=None))"#,
        Err(r#"string() got unexpected keyword argument 'cfg'"#),
    ),
    (
        r#"print(attr.string(aspects=None))"#,
        Err(r#"string() got unexpected keyword argument 'aspects'"#),
    ),
    (
        r#"print(attr.string(executable=None))"#,
        Err(r#"string() got unexpected keyword argument 'executable'"#),
    ),
    (
        r#"print(attr.string(skip_validations=None))"#,
        Err(r#"string() got unexpected keyword argument 'skip_validations'"#),
    ),
    (
        r#"print(attr.string(materializer=None))"#,
        Err(r#"string() got unexpected keyword argument 'materializer'"#),
    ),
    (
        r#"print(attr.string(for_dependency_resolution=None))"#,
        Err(r#"string() got unexpected keyword argument 'for_dependency_resolution'"#),
    ),
    (
        r#"print(attr.string(name=None))"#,
        Err(r#"string() got unexpected keyword argument 'name'"#),
    ),
    (
        r#"print(attr.string(allowlist=None))"#,
        Err(r#"string() got unexpected keyword argument 'allowlist'"#),
    ),
    (
        r#"print(attr.string(nonempty=None))"#,
        Err(r#"string() got unexpected keyword argument 'nonempty'"#),
    ),
    (
        r#"print(attr.string(single_file=None))"#,
        Err(r#"string() got unexpected keyword argument 'single_file'"#),
    ),
    (
        r#"print(attr.string(non_empty=None))"#,
        Err(r#"string() got unexpected keyword argument 'non_empty'"#),
    ),
    (
        r#"print(attr.string(order=None))"#,
        Err(r#"string() got unexpected keyword argument 'order'"#),
    ),
    (
        r#"print(attr.string(output_to_genfiles=None))"#,
        Err(r#"string() got unexpected keyword argument 'output_to_genfiles'"#),
    ),
    (
        r#"print(attr.string(default_provider=None))"#,
        Err(r#"string() got unexpected keyword argument 'default_provider'"#),
    ),
    (
        r#"print(attr.string(container=None))"#,
        Err(r#"string() got unexpected keyword argument 'container'"#),
    ),
    (
        r#"print(attr.string(exec_group=None))"#,
        Err(r#"string() got unexpected keyword argument 'exec_group'"#),
    ),
    (
        r#"print(attr.string(kind=None))"#,
        Err(r#"string() got unexpected keyword argument 'kind'"#),
    ),
    (
        r#"print(attr.string(visibility=None))"#,
        Err(r#"string() got unexpected keyword argument 'visibility'"#),
    ),
    (
        r#"print(attr.string(tags=None))"#,
        Err(r#"string() got unexpected keyword argument 'tags'"#),
    ),
    (
        r#"print(attr.string_dict(1))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(1, 2))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default=None))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'NoneType', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc=None))"#,
        Ok(r#"<attr.string_dict>"#),
    ),
    (
        r#"print(attr.string_dict(mandatory=None))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(values=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'values'"#),
    ),
    (
        r#"print(attr.string_dict(configurable=None))"#,
        Err(
            r#"in call to string_dict(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(allow_empty=None))"#,
        Err(
            r#"in call to string_dict(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(allow_files=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'allow_files'"#),
    ),
    (
        r#"print(attr.string_dict(allow_single_file=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'allow_single_file'"#),
    ),
    (
        r#"print(attr.string_dict(allow_rules=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'allow_rules'"#),
    ),
    (
        r#"print(attr.string_dict(providers=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'providers'"#),
    ),
    (
        r#"print(attr.string_dict(flags=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'flags'"#),
    ),
    (
        r#"print(attr.string_dict(cfg=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'cfg'"#),
    ),
    (
        r#"print(attr.string_dict(aspects=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'aspects'"#),
    ),
    (
        r#"print(attr.string_dict(executable=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'executable'"#),
    ),
    (
        r#"print(attr.string_dict(skip_validations=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'skip_validations'"#),
    ),
    (
        r#"print(attr.string_dict(materializer=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'materializer'"#),
    ),
    (
        r#"print(attr.string_dict(for_dependency_resolution=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'for_dependency_resolution'"#),
    ),
    (
        r#"print(attr.string_dict(name=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'name'"#),
    ),
    (
        r#"print(attr.string_dict(allowlist=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'allowlist'"#),
    ),
    (
        r#"print(attr.string_dict(nonempty=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'nonempty'"#),
    ),
    (
        r#"print(attr.string_dict(single_file=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'single_file'"#),
    ),
    (
        r#"print(attr.string_dict(non_empty=None))"#,
        Err(
            r#"string_dict() got unexpected keyword argument 'non_empty' (did you mean 'allow_empty'?)"#,
        ),
    ),
    (
        r#"print(attr.string_dict(order=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'order'"#),
    ),
    (
        r#"print(attr.string_dict(output_to_genfiles=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'output_to_genfiles'"#),
    ),
    (
        r#"print(attr.string_dict(default_provider=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'default_provider'"#),
    ),
    (
        r#"print(attr.string_dict(container=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'container'"#),
    ),
    (
        r#"print(attr.string_dict(exec_group=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'exec_group'"#),
    ),
    (
        r#"print(attr.string_dict(kind=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'kind'"#),
    ),
    (
        r#"print(attr.string_dict(visibility=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'visibility'"#),
    ),
    (
        r#"print(attr.string_dict(tags=None))"#,
        Err(r#"string_dict() got unexpected keyword argument 'tags'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(1, 2))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=None))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'NoneType', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=None))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=None))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(values=None))"#,
        Err(r#"string_keyed_label_dict() got unexpected keyword argument 'values'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=None))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=None))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=None))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_single_file=None))"#,
        Err(r#"string_keyed_label_dict() got unexpected keyword argument 'allow_single_file'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules=None))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers=None))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'providers' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags=None))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'flags' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg=None))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(aspects=None))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'aspects' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(executable=None))"#,
        Err(r#"string_keyed_label_dict() got unexpected keyword argument 'executable'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(skip_validations=None))"#,
        Err(r#"string_keyed_label_dict() got unexpected keyword argument 'skip_validations'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(materializer=None))"#,
        Err(r#"string_keyed_label_dict() got unexpected keyword argument 'materializer'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(for_dependency_resolution=None))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(name=None))"#,
        Err(r#"string_keyed_label_dict() got unexpected keyword argument 'name'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allowlist=None))"#,
        Err(r#"string_keyed_label_dict() got unexpected keyword argument 'allowlist'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(nonempty=None))"#,
        Err(r#"string_keyed_label_dict() got unexpected keyword argument 'nonempty'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(single_file=None))"#,
        Err(r#"string_keyed_label_dict() got unexpected keyword argument 'single_file'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(non_empty=None))"#,
        Err(
            r#"string_keyed_label_dict() got unexpected keyword argument 'non_empty' (did you mean 'allow_empty'?)"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(order=None))"#,
        Err(r#"string_keyed_label_dict() got unexpected keyword argument 'order'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(output_to_genfiles=None))"#,
        Err(r#"string_keyed_label_dict() got unexpected keyword argument 'output_to_genfiles'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default_provider=None))"#,
        Err(r#"string_keyed_label_dict() got unexpected keyword argument 'default_provider'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(container=None))"#,
        Err(r#"string_keyed_label_dict() got unexpected keyword argument 'container'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(exec_group=None))"#,
        Err(r#"string_keyed_label_dict() got unexpected keyword argument 'exec_group'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(kind=None))"#,
        Err(r#"string_keyed_label_dict() got unexpected keyword argument 'kind'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(visibility=None))"#,
        Err(r#"string_keyed_label_dict() got unexpected keyword argument 'visibility'"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(tags=None))"#,
        Err(r#"string_keyed_label_dict() got unexpected keyword argument 'tags'"#),
    ),
    (
        r#"print(attr.string_list(1))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(1, 2))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=None))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(doc=None))"#,
        Ok(r#"<attr.string_list>"#),
    ),
    (
        r#"print(attr.string_list(mandatory=None))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(values=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'values'"#),
    ),
    (
        r#"print(attr.string_list(configurable=None))"#,
        Err(
            r#"in call to string_list(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list(allow_empty=None))"#,
        Err(
            r#"in call to string_list(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(allow_files=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'allow_files'"#),
    ),
    (
        r#"print(attr.string_list(allow_single_file=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'allow_single_file'"#),
    ),
    (
        r#"print(attr.string_list(allow_rules=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'allow_rules'"#),
    ),
    (
        r#"print(attr.string_list(providers=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'providers'"#),
    ),
    (
        r#"print(attr.string_list(flags=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'flags'"#),
    ),
    (
        r#"print(attr.string_list(cfg=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'cfg'"#),
    ),
    (
        r#"print(attr.string_list(aspects=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'aspects'"#),
    ),
    (
        r#"print(attr.string_list(executable=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'executable'"#),
    ),
    (
        r#"print(attr.string_list(skip_validations=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'skip_validations'"#),
    ),
    (
        r#"print(attr.string_list(materializer=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'materializer'"#),
    ),
    (
        r#"print(attr.string_list(for_dependency_resolution=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'for_dependency_resolution'"#),
    ),
    (
        r#"print(attr.string_list(name=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'name'"#),
    ),
    (
        r#"print(attr.string_list(allowlist=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'allowlist'"#),
    ),
    (
        r#"print(attr.string_list(nonempty=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'nonempty'"#),
    ),
    (
        r#"print(attr.string_list(single_file=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'single_file'"#),
    ),
    (
        r#"print(attr.string_list(non_empty=None))"#,
        Err(
            r#"string_list() got unexpected keyword argument 'non_empty' (did you mean 'allow_empty'?)"#,
        ),
    ),
    (
        r#"print(attr.string_list(order=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'order'"#),
    ),
    (
        r#"print(attr.string_list(output_to_genfiles=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'output_to_genfiles'"#),
    ),
    (
        r#"print(attr.string_list(default_provider=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'default_provider'"#),
    ),
    (
        r#"print(attr.string_list(container=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'container'"#),
    ),
    (
        r#"print(attr.string_list(exec_group=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'exec_group'"#),
    ),
    (
        r#"print(attr.string_list(kind=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'kind'"#),
    ),
    (
        r#"print(attr.string_list(visibility=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'visibility'"#),
    ),
    (
        r#"print(attr.string_list(tags=None))"#,
        Err(r#"string_list() got unexpected keyword argument 'tags'"#),
    ),
    (
        r#"print(attr.string_list_dict(1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(1, 2))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default=None))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'NoneType', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc=None))"#,
        Ok(r#"<attr.string_list_dict>"#),
    ),
    (
        r#"print(attr.string_list_dict(mandatory=None))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(values=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'values'"#),
    ),
    (
        r#"print(attr.string_list_dict(configurable=None))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(allow_empty=None))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'allow_empty' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(allow_files=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'allow_files'"#),
    ),
    (
        r#"print(attr.string_list_dict(allow_single_file=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'allow_single_file'"#),
    ),
    (
        r#"print(attr.string_list_dict(allow_rules=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'allow_rules'"#),
    ),
    (
        r#"print(attr.string_list_dict(providers=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'providers'"#),
    ),
    (
        r#"print(attr.string_list_dict(flags=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'flags'"#),
    ),
    (
        r#"print(attr.string_list_dict(cfg=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'cfg'"#),
    ),
    (
        r#"print(attr.string_list_dict(aspects=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'aspects'"#),
    ),
    (
        r#"print(attr.string_list_dict(executable=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'executable'"#),
    ),
    (
        r#"print(attr.string_list_dict(skip_validations=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'skip_validations'"#),
    ),
    (
        r#"print(attr.string_list_dict(materializer=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'materializer'"#),
    ),
    (
        r#"print(attr.string_list_dict(for_dependency_resolution=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'for_dependency_resolution'"#),
    ),
    (
        r#"print(attr.string_list_dict(name=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'name'"#),
    ),
    (
        r#"print(attr.string_list_dict(allowlist=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'allowlist'"#),
    ),
    (
        r#"print(attr.string_list_dict(nonempty=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'nonempty'"#),
    ),
    (
        r#"print(attr.string_list_dict(single_file=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'single_file'"#),
    ),
    (
        r#"print(attr.string_list_dict(non_empty=None))"#,
        Err(
            r#"string_list_dict() got unexpected keyword argument 'non_empty' (did you mean 'allow_empty'?)"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(order=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'order'"#),
    ),
    (
        r#"print(attr.string_list_dict(output_to_genfiles=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'output_to_genfiles'"#),
    ),
    (
        r#"print(attr.string_list_dict(default_provider=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'default_provider'"#),
    ),
    (
        r#"print(attr.string_list_dict(container=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'container'"#),
    ),
    (
        r#"print(attr.string_list_dict(exec_group=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'exec_group'"#),
    ),
    (
        r#"print(attr.string_list_dict(kind=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'kind'"#),
    ),
    (
        r#"print(attr.string_list_dict(visibility=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'visibility'"#),
    ),
    (
        r#"print(attr.string_list_dict(tags=None))"#,
        Err(r#"string_list_dict() got unexpected keyword argument 'tags'"#),
    ),
    (
        r#"print(attr.int_list(True, 1))"#,
        Err(
            r#"in call to int_list(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int_list(True, True, 1))"#,
        Err(r#"int_list() accepts no more than 2 positional arguments but got 3"#),
    ),
    (
        r#"print(attr.int_list(True, True, True))"#,
        Err(r#"int_list() accepts no more than 2 positional arguments but got 3"#),
    ),
    (
        r#"print(attr.int_list(True, True, True, True))"#,
        Err(r#"int_list() accepts no more than 2 positional arguments but got 4"#),
    ),
    (
        r#"print(attr.int_list(True, True, True, True, True, True, True, True, True, True, True, True, True))"#,
        Err(r#"int_list() accepts no more than 2 positional arguments but got 13"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(True, 1))"#,
        Err(r#"label_keyed_string_dict() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(True, True, 1))"#,
        Err(r#"label_keyed_string_dict() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(True, True, True))"#,
        Err(r#"label_keyed_string_dict() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(True, True, True, True))"#,
        Err(r#"label_keyed_string_dict() accepts no more than 1 positional argument but got 4"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(True, True, True, True, True, True, True, True, True, True, True, True, True))"#,
        Err(r#"label_keyed_string_dict() accepts no more than 1 positional argument but got 13"#),
    ),
    (
        r#"print(attr.label_list(True, 1))"#,
        Err(r#"label_list() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(attr.label_list(True, True, 1))"#,
        Err(r#"label_list() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"print(attr.label_list(True, True, True))"#,
        Err(r#"label_list() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"print(attr.label_list(True, True, True, True))"#,
        Err(r#"label_list() accepts no more than 1 positional argument but got 4"#),
    ),
    (
        r#"print(attr.label_list(True, True, True, True, True, True, True, True, True, True, True, True, True))"#,
        Err(r#"label_list() accepts no more than 1 positional argument but got 13"#),
    ),
    (
        r#"print(attr.label_list_dict(True, 1))"#,
        Err(r#"label_list_dict() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(attr.label_list_dict(True, True, 1))"#,
        Err(r#"label_list_dict() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"print(attr.label_list_dict(True, True, True))"#,
        Err(r#"label_list_dict() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"print(attr.label_list_dict(True, True, True, True))"#,
        Err(r#"label_list_dict() accepts no more than 1 positional argument but got 4"#),
    ),
    (
        r#"print(attr.label_list_dict(True, True, True, True, True, True, True, True, True, True, True, True, True))"#,
        Err(r#"label_list_dict() accepts no more than 1 positional argument but got 13"#),
    ),
    (
        r#"print(attr.output_list(True, 1))"#,
        Err(r#"output_list() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(attr.output_list(True, True, 1))"#,
        Err(r#"output_list() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"print(attr.output_list(True, True, True))"#,
        Err(r#"output_list() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"print(attr.output_list(True, True, True, True))"#,
        Err(r#"output_list() accepts no more than 1 positional argument but got 4"#),
    ),
    (
        r#"print(attr.output_list(True, True, True, True, True, True, True, True, True, True, True, True, True))"#,
        Err(r#"output_list() accepts no more than 1 positional argument but got 13"#),
    ),
    (
        r#"print(attr.string_dict(True, 1))"#,
        Err(r#"string_dict() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(attr.string_dict(True, True, 1))"#,
        Err(r#"string_dict() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"print(attr.string_dict(True, True, True))"#,
        Err(r#"string_dict() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"print(attr.string_dict(True, True, True, True))"#,
        Err(r#"string_dict() accepts no more than 1 positional argument but got 4"#),
    ),
    (
        r#"print(attr.string_dict(True, True, True, True, True, True, True, True, True, True, True, True, True))"#,
        Err(r#"string_dict() accepts no more than 1 positional argument but got 13"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(True, 1))"#,
        Err(r#"string_keyed_label_dict() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(True, True, 1))"#,
        Err(r#"string_keyed_label_dict() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(True, True, True))"#,
        Err(r#"string_keyed_label_dict() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(True, True, True, True))"#,
        Err(r#"string_keyed_label_dict() accepts no more than 1 positional argument but got 4"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(True, True, True, True, True, True, True, True, True, True, True, True, True))"#,
        Err(r#"string_keyed_label_dict() accepts no more than 1 positional argument but got 13"#),
    ),
    (
        r#"print(attr.string_list(True, 1))"#,
        Err(
            r#"in call to string_list(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(True, True, 1))"#,
        Err(r#"string_list() accepts no more than 2 positional arguments but got 3"#),
    ),
    (
        r#"print(attr.string_list(True, True, True))"#,
        Err(r#"string_list() accepts no more than 2 positional arguments but got 3"#),
    ),
    (
        r#"print(attr.string_list(True, True, True, True))"#,
        Err(r#"string_list() accepts no more than 2 positional arguments but got 4"#),
    ),
    (
        r#"print(attr.string_list(True, True, True, True, True, True, True, True, True, True, True, True, True))"#,
        Err(r#"string_list() accepts no more than 2 positional arguments but got 13"#),
    ),
    (
        r#"print(attr.string_list_dict(True, 1))"#,
        Err(r#"string_list_dict() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(attr.string_list_dict(True, True, 1))"#,
        Err(r#"string_list_dict() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"print(attr.string_list_dict(True, True, True))"#,
        Err(r#"string_list_dict() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"print(attr.string_list_dict(True, True, True, True))"#,
        Err(r#"string_list_dict() accepts no more than 1 positional argument but got 4"#),
    ),
    (
        r#"print(attr.string_list_dict(True, True, True, True, True, True, True, True, True, True, True, True, True))"#,
        Err(r#"string_list_dict() accepts no more than 1 positional argument but got 13"#),
    ),
    (r#"print(attr.int(values=["a"]))"#, Ok(r#"<attr.int>"#)),
    (
        r#"print(attr.int(values=[1,2], default=3))"#,
        Ok(r#"<attr.int>"#),
    ),
    (
        r#"print(attr.int(values=[1,2], default=1))"#,
        Ok(r#"<attr.int>"#),
    ),
    (
        r#"print(attr.string(values=["a"], mandatory=True))"#,
        Ok(r#"<attr.string>"#),
    ),
    (
        r#"print(attr.string_dict(default={"a":1}))"#,
        Err(r#"expected value of type 'string' for dict value element, but got 1 (int)"#),
    ),
    (
        r#"print(attr.string_dict(default={1:"a"}))"#,
        Err(r#"expected value of type 'string' for dict key element, but got 1 (int)"#),
    ),
    (
        r#"print(attr.string_dict(default={"a":None}))"#,
        Err(r#"expected value of type 'string' for dict value element, but got None (NoneType)"#),
    ),
    (
        r#"print(attr.string_dict(default={"a":["b"]}))"#,
        Err(r#"expected value of type 'string' for dict value element, but got ["b"] (list)"#),
    ),
    (
        r#"print(attr.string_dict(default=[("a","b")]))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'list', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default={"a":["b"]}))"#,
        Ok(r#"<attr.string_list_dict>"#),
    ),
    (
        r#"print(attr.string_list_dict(default={"a":["b", 1]}))"#,
        Err(
            r#"expected value of type 'string' for element 1 of dict value element, but got 1 (int)"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default={1:["b"]}))"#,
        Err(r#"expected value of type 'string' for dict key element, but got 1 (int)"#),
    ),
    (
        r#"print(attr.string_list_dict(default={"a":("b",)}))"#,
        Ok(r#"<attr.string_list_dict>"#),
    ),
    (
        r#"print(attr.string_list_dict(default={"a":[]}))"#,
        Ok(r#"<attr.string_list_dict>"#),
    ),
    (
        r#"print(attr.string_list_dict(default={"a":[]}, allow_empty=False))"#,
        Ok(r#"<attr.string_list_dict>"#),
    ),
    (
        r#"print(attr.string_list_dict(default={}, allow_empty=False))"#,
        Ok(r#"<attr.string_list_dict>"#),
    ),
    (
        r#"print(attr.string_list_dict(default={"a": "b"}))"#,
        Err(
            r#"expected value of type 'list(string)' for dict value element, but got "b" (string)"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default={"a":["//a:b"]}))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(default={"a":[Label("//a:b")]}))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(default={"a":[1]}))"#,
        Err(
            r#"expected value of type 'string' for element 0 of dict value element, but got 1 (int)"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default={"a":["not a label"]}))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(default={1:["//a:b"]}))"#,
        Err(r#"expected value of type 'string' for dict key element, but got 1 (int)"#),
    ),
    (
        r#"print(attr.label_list_dict(default={"a":("//a:b",)}))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(default={"a":"//a:b"}))"#,
        Err(
            r#"expected value of type 'list(label)' for dict value element, but got "//a:b" (string)"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=lambda: {}))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'function', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={"//a:b":"x"}))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={Label("//a:b"):"x"}))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={"//a:b":1}))"#,
        Err(r#"expected value of type 'string' for dict value element, but got 1 (int)"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={1:"x"}))"#,
        Err(r#"expected value of type 'string' for dict key element, but got 1 (int)"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={"bad label":"x"}))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={"//a:b":"x"}, allow_empty=False))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={}, allow_empty=False))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={"x":"//a:b"}))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={"x":Label("//a:b")}))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={"x":1}))"#,
        Err(r#"expected value of type 'string' for dict value element, but got 1 (int)"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={1:"//a:b"}))"#,
        Err(r#"expected value of type 'string' for dict key element, but got 1 (int)"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={"x":"bad label"}))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={"x":None}))"#,
        Err(r#"expected value of type 'string' for dict value element, but got None (NoneType)"#),
    ),
    (
        r#"print(attr.label_list(default=["//a:b"]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(default=[Label("//a:b")]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(default=["bad label"]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(default=[None]))"#,
        Err(
            r#"expected value of type 'string' for element 0 of parameter 'default' of attribute 'label_list', but got None (NoneType)"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=[":x", "//a:b", "@r//a:b", "//a"]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(default=[":x:y"]))"#,
        Err(
            r#"invalid label ':x:y' in element 0 of parameter 'default' of attribute 'label_list': invalid target name 'x:y': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=[]), allow_empty=False)"#,
        Err(r#"print() got unexpected keyword argument 'allow_empty'"#),
    ),
    (
        r#"print(attr.label_list(default=[], allow_empty=False))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(default=["//a:b"], allow_empty=False))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(default=lambda x: [], allow_empty=False))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.string_list(default=[], allow_empty=False))"#,
        Ok(r#"<attr.string_list>"#),
    ),
    (
        r#"print(attr.string_list(default=["a"], allow_empty=False))"#,
        Ok(r#"<attr.string_list>"#),
    ),
    (
        r#"print(attr.int_list(default=[], allow_empty=False))"#,
        Ok(r#"<attr.int_list>"#),
    ),
    (
        r#"print(attr.string_list(default=[], mandatory=True))"#,
        Ok(r#"<attr.string_list>"#),
    ),
    (
        r#"print(attr.output_list(allow_empty=False))"#,
        Ok(r#"<attr.output_list>"#),
    ),
    (
        r#"print(attr.label(default="//a:b"))"#,
        Ok(r#"<attr.label>"#),
    ),
    (r#"print(attr.label(default=":x"))"#, Ok(r#"<attr.label>"#)),
    (r#"print(attr.label(default="x"))"#, Ok(r#"<attr.label>"#)),
    (
        r#"print(attr.label(default="bad label"))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(default=""))"#,
        Err(
            r#"invalid label '' in parameter 'default' of attribute 'label': invalid target name '': empty target name"#,
        ),
    ),
    (
        r#"print(attr.label(default="@r//a:b"))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(default="@@r//a:b"))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(default="//a:b:c"))"#,
        Err(
            r#"invalid label '//a:b:c' in parameter 'default' of attribute 'label': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label(default=lambda x: 1))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(default=lambda **k: 1))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(default=lambda x, y: 1))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(default=lambda *a: 1))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(default=lambda x=1: 1))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(default=lambda *, x: 1))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(default=lambda: Label("//a:b"), mandatory=True))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(default="//a:b", mandatory=True))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(default="//a:b", executable=True))"#,
        Err(
            r#"cfg parameter is mandatory when executable=True is provided. Please see https://bazel.build/extending/rules#configurations for more details."#,
        ),
    ),
    (
        r#"print(attr.label(default="//a:b", executable=True, cfg="exec"))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(default="//a:b", executable=True, cfg="target"))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(executable=False))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(executable=True, cfg="exec", allow_files=True))"#,
        Ok(r#"<attr.label>"#),
    ),
    (r#"print(attr.label(cfg="exec"))"#, Ok(r#"<attr.label>"#)),
    (r#"print(attr.label(cfg="target"))"#, Ok(r#"<attr.label>"#)),
    (r#"print(attr.label(cfg="host"))"#, Ok(r#"<attr.label>"#)),
    (
        r#"print(attr.label(cfg="Exec"))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(cfg="data"))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(cfg="")) "#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=True, allow_single_file=True))"#,
        Err(r#"Cannot specify both allow_files and allow_single_file"#),
    ),
    (
        r#"print(attr.label(allow_files=False, allow_single_file=False))"#,
        Err(r#"Cannot specify both allow_files and allow_single_file"#),
    ),
    (
        r#"print(attr.label(allow_files=True, allow_single_file=False))"#,
        Err(r#"Cannot specify both allow_files and allow_single_file"#),
    ),
    (
        r#"print(attr.label(allow_files=False, allow_single_file=True))"#,
        Err(r#"Cannot specify both allow_files and allow_single_file"#),
    ),
    (
        r#"print(attr.label(allow_files=[".x"], allow_single_file=[".y"]))"#,
        Err(r#"Cannot specify both allow_files and allow_single_file"#),
    ),
    (
        r#"print(attr.label(allow_files=None, allow_single_file=None))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_files=[".x"], allow_rules=["a"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_files=True, allow_rules=["a"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_rules=["a"], providers=[[]]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_rules=["a"], providers=[]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_rules=[]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_files=["x"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_files=[".x", 1]))"#,
        Err(r#"at index 1 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(allow_single_file=[".x", 1]))"#,
        Err(r#"at index 1 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(allow_single_file=["x", "y"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_files=[""]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_files=[".a.b", "*", "x.y"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(providers=[[]]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(providers=[[], []]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(providers=[[1]]))"#,
        Err(r#"at index 0 of providers, got element of type int, want Provider"#),
    ),
    (
        r#"print(attr.label(providers=[["a"]]))"#,
        Err(r#"at index 0 of providers, got element of type string, want Provider"#),
    ),
    (
        r#"print(attr.label(providers=[DefaultInfo]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(providers=[[DefaultInfo]]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(providers=[[DefaultInfo], [OutputGroupInfo]]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(providers=[DefaultInfo, [DefaultInfo]]))"#,
        Err(r#"at index 0 of providers, got element of type Provider, want sequence"#),
    ),
    (
        r#"print(attr.label(providers=[[DefaultInfo], DefaultInfo]))"#,
        Err(r#"at index 1 of providers, got element of type Provider, want sequence"#),
    ),
    (
        r#"print(attr.label(providers=[DefaultInfo, OutputGroupInfo]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(providers=[[DefaultInfo, 1]]))"#,
        Err(r#"at index 1 of providers, got element of type int, want Provider"#),
    ),
    (
        r#"print(attr.label(providers=[[[]]]))"#,
        Err(r#"at index 0 of providers, got element of type list, want Provider"#),
    ),
    (
        r#"print(attr.label(providers=[provider()]))"#,
        Err(r#"Providers should be top-level values in extension files that define them."#),
    ),
    (
        r#"print(attr.label(providers=[[provider()]]))"#,
        Err(r#"Providers should be top-level values in extension files that define them."#),
    ),
    (
        r#"print(attr.label(providers=[provider(), 1]))"#,
        Err(r#"at index 0 of providers, got element of type Provider, want sequence"#),
    ),
    (
        r#"print(attr.label(providers=[[provider()], 1]))"#,
        Err(r#"at index 1 of providers, got element of type int, want sequence"#),
    ),
    (
        r#"print(attr.label(providers=[[provider()], []]))"#,
        Err(r#"Providers should be top-level values in extension files that define them."#),
    ),
    (
        r#"print(attr.label(providers=[[], DefaultInfo]))"#,
        Err(r#"at index 1 of providers, got element of type Provider, want sequence"#),
    ),
    (
        r#"print(attr.label(providers=[None]))"#,
        Err(r#"at index 0 of providers, got element of type NoneType, want sequence"#),
    ),
    (
        r#"print(attr.label(providers=[[None]]))"#,
        Err(r#"at index 0 of providers, got element of type NoneType, want Provider"#),
    ),
    (
        r#"print(attr.label(providers=[struct()]))"#,
        Err(r#"at index 0 of providers, got element of type struct, want sequence"#),
    ),
    (
        r#"print(attr.label(providers=[[struct()]]))"#,
        Err(r#"at index 0 of providers, got element of type struct, want Provider"#),
    ),
    (
        r#"print(attr.label(providers=([DefaultInfo],)))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(providers=[(DefaultInfo,)]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(aspects=[aspect(implementation=lambda t, c: [])]))"#,
        Err(r#"Aspects should be top-level values in extension files that define them."#),
    ),
    (
        r#"print(attr.label(aspects=[aspect(implementation=lambda t, c: []), 1]))"#,
        Err(r#"at index 1 of aspects, got element of type int, want Aspect"#),
    ),
    (
        r#"print(attr.label(aspects=[[]]))"#,
        Err(r#"at index 0 of aspects, got element of type list, want Aspect"#),
    ),
    (
        r#"print(attr.label(flags=["SINGLE_ARTIFACT"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(flags=["DIRECT_COMPILE_TIME_INPUT"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(flags=["ORDER_INDEPENDENT_DATA"]))"#,
        Err(r#"unknown attribute flag 'ORDER_INDEPENDENT_DATA'"#),
    ),
    (
        r#"print(attr.label(flags=["CHECK_CONSTRAINTS"]))"#,
        Err(r#"unknown attribute flag 'CHECK_CONSTRAINTS'"#),
    ),
    (
        r#"print(attr.label(flags=["TREAT_AS_DEP_FOR_VISIBILITY_ONLY"]))"#,
        Err(r#"unknown attribute flag 'TREAT_AS_DEP_FOR_VISIBILITY_ONLY'"#),
    ),
    (
        r#"print(attr.label(flags=["single_artifact"]))"#,
        Err(r#"unknown attribute flag 'single_artifact'"#),
    ),
    (
        r#"print(attr.label(flags=["NON_EMPTY"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(flags=["MANDATORY"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(flags=["EXECUTABLE"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(flags=["UNDOCUMENTED"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(flags=["SKIP_ANALYSIS_TIME_FILETYPE_CHECK"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(flags=["SKIP_CONSTRAINTS_OVERRIDE"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(flags=["HIDDEN"]))"#,
        Err(r#"unknown attribute flag 'HIDDEN'"#),
    ),
    (
        r#"print(attr.label(flags=["DO_NOT_CHECK_MANDATORY"]))"#,
        Err(r#"unknown attribute flag 'DO_NOT_CHECK_MANDATORY'"#),
    ),
    (
        r#"print(attr.label(flags=["OUTPUT_LICENSES"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(flags=["ORDER_INDEPENDENT"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(flags=["BUILD_SETTING_DEFAULT_CONVENTION"]))"#,
        Err(r#"unknown attribute flag 'BUILD_SETTING_DEFAULT_CONVENTION'"#),
    ),
    (
        r#"print(attr.label(flags=["SINGLE_ARTIFACT", "SINGLE_ARTIFACT"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(flags=["a", 1]))"#,
        Err(r#"at index 1 of flags, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(flags=[1, "a"]))"#,
        Err(r#"at index 0 of flags, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(materializer=lambda x: 1))"#,
        Err(
            r#"in call to label(), parameter 'materializer' is experimental and thus unavailable with the current flags. It may be enabled by setting --experimental_dormant_deps"#,
        ),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=True))"#,
        Ok(r#"<attr.label>"#),
    ),
    (r#"print(attr.output())"#, Ok(r#"<attr.output>"#)),
    (
        r#"print(attr.output(doc="d", mandatory=True))"#,
        Ok(r#"<attr.output>"#),
    ),
    (r#"print(attr.output_list())"#, Ok(r#"<attr.output_list>"#)),
    (
        r#"print(attr.output_list(doc="d", mandatory=True, allow_empty=False))"#,
        Ok(r#"<attr.output_list>"#),
    ),
    (r#"print(attr.bool())"#, Ok(r#"<attr.bool>"#)),
    (
        r#"print(attr.bool(default=False, doc="d", mandatory=True, configurable=False))"#,
        Ok(r#"<attr.bool>"#),
    ),
    (
        r#"print(attr.int_list(True, False))"#,
        Ok(r#"<attr.int_list>"#),
    ),
    (
        r#"print(attr.string_list(True, False))"#,
        Ok(r#"<attr.string_list>"#),
    ),
    (
        r#"print(attr.int_list(mandatory=True, allow_empty=False, default=[1], doc="d", configurable=True))"#,
        Ok(r#"<attr.int_list>"#),
    ),
    (
        r#"print(attr.string_list(True, False, default=["a"], allow_empty=True))"#,
        Err(r#"string_list() got multiple values for argument 'allow_empty'"#),
    ),
    (
        r#"print(attr.string_list(True, mandatory=True))"#,
        Err(r#"string_list() got multiple values for argument 'mandatory'"#),
    ),
    (
        r#"print(attr.string_list(True, False, mandatory=True))"#,
        Err(r#"string_list() got multiple values for argument 'mandatory'"#),
    ),
    (
        r#"print(attr.string_list(True, False, allow_empty=True))"#,
        Err(r#"string_list() got multiple values for argument 'allow_empty'"#),
    ),
    (
        r#"print(attr.label_list(True, allow_empty=True))"#,
        Err(r#"label_list() got multiple values for argument 'allow_empty'"#),
    ),
    (
        r#"print(attr.label_list(True, True))"#,
        Err(r#"label_list() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(attr.label_list(True, default=[]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.string_dict(True))"#,
        Ok(r#"<attr.string_dict>"#),
    ),
    (
        r#"print(attr.string_dict(allow_empty=True, default={}))"#,
        Ok(r#"<attr.string_dict>"#),
    ),
    (
        r#"print(attr.string_dict(True, allow_empty=True))"#,
        Err(r#"string_dict() got multiple values for argument 'allow_empty'"#),
    ),
    (
        r#"print(attr.int(default=1<<31))"#,
        Err(
            r#"for parameter 'default' of attribute '', got 2147483648, want value in signed 32-bit range"#,
        ),
    ),
    (r#"print(attr.int(default=(1<<31)-1))"#, Ok(r#"<attr.int>"#)),
    (r#"print(attr.int(default=-(1<<31)))"#, Ok(r#"<attr.int>"#)),
    (
        r#"print(attr.int(default=-(1<<31)-1))"#,
        Err(
            r#"for parameter 'default' of attribute '', got -2147483649, want value in signed 32-bit range"#,
        ),
    ),
    (
        r#"print(attr.int(default=1<<63))"#,
        Err(
            r#"for parameter 'default' of attribute '', got 9223372036854775808, want value in signed 32-bit range"#,
        ),
    ),
    (
        r#"print(attr.int(default=1<<100))"#,
        Err(
            r#"for parameter 'default' of attribute '', got 1267650600228229401496703205376, want value in signed 32-bit range"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=[1<<31]))"#,
        Err(
            r#"for element 0 of parameter 'default' of attribute '', got 2147483648, want value in signed 32-bit range"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=[-(1<<31)-1]))"#,
        Err(
            r#"for element 0 of parameter 'default' of attribute '', got -2147483649, want value in signed 32-bit range"#,
        ),
    ),
    (r#"print(attr.int(values=[1<<40]))"#, Ok(r#"<attr.int>"#)),
    (r#"print(attr.int(values=[1<<31]))"#, Ok(r#"<attr.int>"#)),
    (
        r#"print(attr.int(values=["a"], default=1))"#,
        Ok(r#"<attr.int>"#),
    ),
    (
        r#"print(attr.int_list(default=[1.5]))"#,
        Err(
            r#"expected value of type 'int' for element 0 of parameter 'default' of attribute '', but got 1.5 (float)"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=[True]))"#,
        Err(
            r#"expected value of type 'int' for element 0 of parameter 'default' of attribute '', but got True (bool)"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=[None]))"#,
        Err(
            r#"expected value of type 'int' for element 0 of parameter 'default' of attribute '', but got None (NoneType)"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=[None]))"#,
        Err(
            r#"expected value of type 'string' for element 0 of parameter 'default' of attribute '', but got None (NoneType)"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=[["a"]]))"#,
        Err(
            r#"expected value of type 'string' for element 0 of parameter 'default' of attribute '', but got ["a"] (list)"#,
        ),
    ),
    (
        r#"print(attr.string_list(default="a"))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=depset(["a"])))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'depset', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default={"a":1}))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=[1, "a"]))"#,
        Err(
            r#"expected value of type 'int' for element 1 of parameter 'default' of attribute '', but got "a" (string)"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=["a", 1]))"#,
        Err(
            r#"expected value of type 'string' for element 1 of parameter 'default' of attribute '', but got 1 (int)"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=["//a:b", 1]))"#,
        Err(
            r#"expected value of type 'string' for element 1 of parameter 'default' of attribute 'label_list', but got 1 (int)"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=["//a:b", "//a:b:c"]))"#,
        Err(
            r#"invalid label '//a:b:c' in element 1 of parameter 'default' of attribute 'label_list': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=["//a:b:c"]))"#,
        Err(
            r#"invalid label '//a:b:c' in element 0 of parameter 'default' of attribute 'label_list': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=["//a:b:c", 1]))"#,
        Err(
            r#"invalid label '//a:b:c' in element 0 of parameter 'default' of attribute 'label_list': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=[1, "//a:b:c"]))"#,
        Err(
            r#"expected value of type 'string' for element 0 of parameter 'default' of attribute 'label_list', but got 1 (int)"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=depset()))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'depset', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default={}))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'dict', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label(default=1))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default="//a:b:c", mandatory=1))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.label(default="//a:b:c", cfg="x"))"#,
        Err(
            r#"invalid label '//a:b:c' in parameter 'default' of attribute 'label': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label(default="//a:b:c", allow_files=1))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(mandatory=1, default=1))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.label(allow_files=1, mandatory=1))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(cfg="x", executable=True))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(cfg="x", allow_files=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(allow_files=[1], cfg="x"))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(flags=["a"], cfg="x"))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label(cfg="x", flags=["a"]))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label(cfg="x", allow_rules=[1]))"#,
        Err(
            r#"at index 0 of allowed rule classes for attribute definition, got element of type int, want string"#,
        ),
    ),
    (
        r#"print(attr.label(cfg="x", providers=[1]))"#,
        Err(r#"at index 0 of providers, got element of type int, want sequence"#),
    ),
    (
        r#"print(attr.label(providers=[1], cfg="x"))"#,
        Err(r#"at index 0 of providers, got element of type int, want sequence"#),
    ),
    (
        r#"print(attr.label(aspects=[1], cfg="x"))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(cfg="x", aspects=[1]))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(aspects=[1], providers=[1]))"#,
        Err(r#"at index 0 of providers, got element of type int, want sequence"#),
    ),
    (
        r#"print(attr.label(providers=[1], aspects=[1]))"#,
        Err(r#"at index 0 of providers, got element of type int, want sequence"#),
    ),
    (
        r#"print(attr.label(allow_files=[1], allow_single_file=[1]))"#,
        Err(r#"Cannot specify both allow_files and allow_single_file"#),
    ),
    (
        r#"print(attr.label(allow_files=[1], allow_rules=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(allow_rules=[1], allow_files=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(allow_single_file=[1], allow_rules=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(allow_files=[1], providers=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(allow_single_file=1, allow_files=[1]))"#,
        Err(r#"Cannot specify both allow_files and allow_single_file"#),
    ),
    (
        r#"print(attr.label(allow_single_file=[1], allow_files=True))"#,
        Err(r#"Cannot specify both allow_files and allow_single_file"#),
    ),
    (
        r#"print(attr.label(allow_single_file=True, allow_files=None))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_single_file=None, allow_files=True))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_single_file=[], allow_files=None))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_single_file=False))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_files=False))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_single_file=[".x"], executable=True))"#,
        Err(
            r#"cfg parameter is mandatory when executable=True is provided. Please see https://bazel.build/extending/rules#configurations for more details."#,
        ),
    ),
    (
        r#"print(attr.label(executable=True, default=1))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(executable=True, cfg="x"))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(default="//a:b:c", executable=True))"#,
        Err(
            r#"invalid label '//a:b:c' in parameter 'default' of attribute 'label': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label(default=1, executable=True))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(flags=["a"], executable=True))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label(flags=["a"], allow_files=[1]))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label(flags=["a"], allow_rules=[1]))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label(flags=["a"], providers=[1]))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label(flags=["a"], aspects=[1]))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label(flags=["a"], allow_files=True, allow_single_file=True))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label(allow_files=[1], flags=[1]))"#,
        Err(r#"at index 0 of flags, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(cfg=1, flags=[1]))"#,
        Err(r#"at index 0 of flags, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(cfg="x", executable=True, allow_files=True, allow_single_file=True))"#,
        Err(r#"Cannot specify both allow_files and allow_single_file"#),
    ),
    (
        r#"print(attr.label_list(default=["//a:b:c"], allow_files=[1]))"#,
        Err(
            r#"invalid label '//a:b:c' in element 0 of parameter 'default' of attribute 'label_list': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=["//a:b:c"], cfg="x"))"#,
        Err(
            r#"invalid label '//a:b:c' in element 0 of parameter 'default' of attribute 'label_list': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=["//a:b:c"], flags=["a"]))"#,
        Err(
            r#"invalid label '//a:b:c' in element 0 of parameter 'default' of attribute 'label_list': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files=[1], cfg="x"))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label_list(default=["//a:b:c"], allow_empty=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=1, mandatory=1))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'int', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default={1:1}))"#,
        Err(r#"expected value of type 'string' for dict key element, but got 1 (int)"#),
    ),
    (
        r#"print(attr.string_list_dict(default={"a":[1]}))"#,
        Err(
            r#"expected value of type 'string' for element 0 of dict value element, but got 1 (int)"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default={"a":1}))"#,
        Err(r#"expected value of type 'list(string)' for dict value element, but got 1 (int)"#),
    ),
    (
        r#"print(attr.string_list_dict(default={"a":["b"], 1:["c"]}))"#,
        Err(r#"expected value of type 'string' for dict key element, but got 1 (int)"#),
    ),
    (
        r#"print(attr.string_list_dict(default={"a":[1], 1:["c"]}))"#,
        Err(
            r#"expected value of type 'string' for element 0 of dict value element, but got 1 (int)"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default={1:1}))"#,
        Err(r#"expected value of type 'string' for dict key element, but got 1 (int)"#),
    ),
    (
        r#"print(attr.string_dict(default={"a":"b", 1:"c"}))"#,
        Err(r#"expected value of type 'string' for dict key element, but got 1 (int)"#),
    ),
    (
        r#"print(attr.string_dict(default={"a":1, 1:"c"}))"#,
        Err(r#"expected value of type 'string' for dict value element, but got 1 (int)"#),
    ),
    (
        r#"print(attr.label_list_dict(default={"a":["//a:b:c"]}))"#,
        Err(
            r#"invalid label '//a:b:c' in element 0 of dict value element: invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default={"a":[Label("//a:b"), "//a:b:c"]}))"#,
        Err(
            r#"invalid label '//a:b:c' in element 1 of dict value element: invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default={"a":[Label("//a:b"), 1]}))"#,
        Err(
            r#"expected value of type 'string' for element 1 of dict value element, but got 1 (int)"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default={"//a:b:c":["//a:b"]}))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(default={"a":None}))"#,
        Err(
            r#"expected value of type 'list(label)' for dict value element, but got None (NoneType)"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default={"a":depset()}))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={"//a:b:c":"x"}))"#,
        Err(
            r#"invalid label '//a:b:c' in dict key element: invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={"//a:b:c":1}))"#,
        Err(
            r#"invalid label '//a:b:c' in dict key element: invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={1:1}))"#,
        Err(r#"expected value of type 'string' for dict key element, but got 1 (int)"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={None:"x"}))"#,
        Err(r#"expected value of type 'string' for dict key element, but got None (NoneType)"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={Label("//a:b"):"x", "//a:b":"y"}))"#,
        Err(
            r#"duplicate labels in parameter 'default' of attribute 'label_keyed_string_dict': //a:b (as [Label("//a:b"), "//a:b"])"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={"x":"//a:b:c"}))"#,
        Err(
            r#"invalid label '//a:b:c' in dict value element: invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={1:1}))"#,
        Err(r#"expected value of type 'string' for dict key element, but got 1 (int)"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={"x":"//a:b:c", 1:"//a:b"}))"#,
        Err(
            r#"invalid label '//a:b:c' in dict value element: invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={"x":"", "y":"//a:b:c"}))"#,
        Err(r#"invalid label '' in dict value element: invalid target name '': empty target name"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={"x":"//a:b:c"}, allow_files=[1]))"#,
        Err(
            r#"invalid label '//a:b:c' in dict value element: invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={"x":"//a:b"}, allow_empty=False))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={}, allow_empty=False))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={"x":"//a:b"}, default=1))"#,
        Err(r#"duplicate keyword argument: default"#),
    ),
    (
        r#"print(attr.string_dict(default={}, allow_empty=False))"#,
        Ok(r#"<attr.string_dict>"#),
    ),
    (
        r#"print(attr.string_dict(default={"a":"b"}, allow_empty=False))"#,
        Ok(r#"<attr.string_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(default={}, allow_empty=False))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.config)"#,
        Err(r#"'attr' value has no field or method 'config'"#),
    ),
    (
        r#"print(dir(config))"#,
        Ok(r#"["bool", "exec", "int", "none", "string", "string_list", "string_set", "target"]"#),
    ),
    (
        r#"print(type(config.exec()))"#,
        Ok(r#"ExecTransitionFactory"#),
    ),
    (
        r#"print(type(config.exec("x")))"#,
        Ok(r#"ExecTransitionFactory"#),
    ),
    (r#"print(type(config.target()))"#, Ok(r#"transition"#)),
    (r#"print(type(config.none()))"#, Ok(r#"transition"#)),
    (
        r#"print(config.exec())"#,
        Ok(
            r#"<unknown object com.google.devtools.build.lib.analysis.config.ExecutionTransitionFactory>"#,
        ),
    ),
    (
        r#"print(attr.label(cfg=config.exec()))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(cfg=config.target()))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(cfg=config.none()))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(cfg=config.exec(), executable=True))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(cfg=config.exec("x"), executable=True))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(cfg=config.exec(exec_group="x")))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(cfg=analysis_test_transition(settings={})))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(type(analysis_test_transition(settings={})))"#,
        Ok(r#"transition"#),
    ),
    (
        r#"def _i(settings, attr): return {}
t = transition(implementation=_i, inputs=[], outputs=[])
print(type(t))"#,
        Ok(r#"transition"#),
    ),
    (
        r#"def _i(settings, attr): return {}
t = transition(implementation=_i, inputs=[], outputs=[])
print(attr.label(cfg=t))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"def _i(settings, attr): return {}
t = transition(implementation=_i, inputs=[], outputs=[])
print(attr.label(cfg=t, executable=True))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"def _i(settings, attr): return {}
t = transition(implementation=_i, inputs=[], outputs=[])
print(attr.label_list(cfg=t))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"def _i(settings, attr): return {}
t = transition(implementation=_i, inputs=[], outputs=[])
print(attr.label_keyed_string_dict(cfg=t))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_list(cfg="exec"))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(cfg="host"))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(cfg="target"))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg="exec"))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(cfg="x"))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(cfg="x"))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(cfg="x"))"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (r#"print(attr.string() == attr.int())"#, Ok(r#"False"#)),
    (
        r#"print(attr.string(default="a") == attr.string(default="b"))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.string(default="a") == attr.string(default="a"))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.string(doc="a") == attr.string(doc="b"))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.string(mandatory=True) == attr.string())"#,
        Ok(r#"False"#),
    ),
    (r#"print(attr.label() == attr.label())"#, Ok(r#"True"#)),
    (
        r#"print(attr.label(default="//a:b") == attr.label(default="//a:c"))"#,
        Ok(r#"False"#),
    ),
    (r#"print(attr.string() != attr.string())"#, Ok(r#"False"#)),
    (
        r#"print(attr.string() < attr.string())"#,
        Err(r#"unsupported comparison: Attribute <=> Attribute"#),
    ),
    (r#"print(attr.string() in [attr.string()])"#, Ok(r#"True"#)),
    (
        r#"x = attr.string()
print(x in [x])"#,
        Ok(r#"True"#),
    ),
    (
        r#"x = attr.string()
print([x] == [attr.string()])"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.bool(default=1, doc=1))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(default=1, mandatory=1))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(default=1, configurable=1))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(doc=1, mandatory=1))"#,
        Err(
            r#"in call to bool(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.bool(doc=1, configurable=1))"#,
        Err(
            r#"in call to bool(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.bool(mandatory=1, configurable=1))"#,
        Err(r#"in call to bool(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.int(default="a", doc=1))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'string', want 'int'"#),
    ),
    (
        r#"print(attr.int(default="a", mandatory=1))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'string', want 'int'"#),
    ),
    (
        r#"print(attr.int(default="a", values=1))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'string', want 'int'"#),
    ),
    (
        r#"print(attr.int(default="a", configurable=1))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'string', want 'int'"#),
    ),
    (
        r#"print(attr.int(doc=1, mandatory=1))"#,
        Err(
            r#"in call to int(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int(doc=1, values=1))"#,
        Err(
            r#"in call to int(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int(doc=1, configurable=1))"#,
        Err(
            r#"in call to int(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int(mandatory=1, values=1))"#,
        Err(r#"in call to int(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.int(mandatory=1, configurable=1))"#,
        Err(r#"in call to int(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.int(values=1, configurable=1))"#,
        Err(r#"in call to int(), parameter 'values' got value of type 'int', want 'sequence'"#),
    ),
    (
        r#"print(attr.int_list(default=1, doc=1))"#,
        Err(
            r#"in call to int_list(), parameter 'default' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=1, mandatory=1))"#,
        Err(
            r#"in call to int_list(), parameter 'default' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=1, configurable=1))"#,
        Err(
            r#"in call to int_list(), parameter 'default' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int_list(default=1, allow_empty=1))"#,
        Err(
            r#"in call to int_list(), parameter 'default' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.int_list(doc=1, mandatory=1))"#,
        Err(
            r#"in call to int_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int_list(doc=1, configurable=1))"#,
        Err(
            r#"in call to int_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int_list(doc=1, allow_empty=1))"#,
        Err(
            r#"in call to int_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.int_list(mandatory=1, configurable=1))"#,
        Err(r#"in call to int_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.int_list(mandatory=1, allow_empty=1))"#,
        Err(r#"in call to int_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.int_list(configurable=1, allow_empty=1))"#,
        Err(
            r#"in call to int_list(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(default=1, doc=1))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=1, mandatory=1))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=1, configurable=1))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=1, allow_files=1))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=1, allow_rules=1))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=1, providers=1))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=1, flags=1))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=1, aspects=1))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=1, executable=1))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=1, skip_validations=1))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=1, mandatory=1))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=1, configurable=1))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=1, allow_files=1))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=1, allow_rules=1))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=1, providers=1))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=1, flags=1))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=1, aspects=1))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=1, executable=1))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(doc=1, skip_validations=1))"#,
        Err(
            r#"in call to label(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(mandatory=1, configurable=1))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.label(mandatory=1, allow_files=1))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.label(mandatory=1, allow_rules=1))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.label(mandatory=1, providers=1))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.label(mandatory=1, flags=1))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.label(mandatory=1, aspects=1))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.label(mandatory=1, executable=1))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.label(mandatory=1, skip_validations=1))"#,
        Err(r#"in call to label(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.label(configurable=1, allow_files=1))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable=1, allow_rules=1))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable=1, providers=1))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable=1, flags=1))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable=1, aspects=1))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable=1, executable=1))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(configurable=1, skip_validations=1))"#,
        Err(
            r#"in call to label(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=1, allow_rules=1))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=1, providers=1))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=1, flags=1))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=1, aspects=1))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=1, executable=1))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=1, skip_validations=1))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=1, providers=1))"#,
        Err(
            r#"in call to label(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=1, flags=1))"#,
        Err(
            r#"in call to label(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=1, aspects=1))"#,
        Err(
            r#"in call to label(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=1, executable=1))"#,
        Err(
            r#"in call to label(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=1, skip_validations=1))"#,
        Err(
            r#"in call to label(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(providers=1, flags=1))"#,
        Err(
            r#"in call to label(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(providers=1, aspects=1))"#,
        Err(
            r#"in call to label(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(providers=1, executable=1))"#,
        Err(
            r#"in call to label(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(providers=1, skip_validations=1))"#,
        Err(
            r#"in call to label(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label(flags=1, aspects=1))"#,
        Err(r#"in call to label(), parameter 'flags' got value of type 'int', want 'sequence'"#),
    ),
    (
        r#"print(attr.label(flags=1, executable=1))"#,
        Err(r#"in call to label(), parameter 'flags' got value of type 'int', want 'sequence'"#),
    ),
    (
        r#"print(attr.label(flags=1, skip_validations=1))"#,
        Err(r#"in call to label(), parameter 'flags' got value of type 'int', want 'sequence'"#),
    ),
    (
        r#"print(attr.label(aspects=1, executable=1))"#,
        Err(r#"in call to label(), parameter 'aspects' got value of type 'int', want 'sequence'"#),
    ),
    (
        r#"print(attr.label(aspects=1, skip_validations=1))"#,
        Err(r#"in call to label(), parameter 'aspects' got value of type 'int', want 'sequence'"#),
    ),
    (
        r#"print(attr.label(executable=1, skip_validations=1))"#,
        Err(r#"in call to label(), parameter 'executable' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=1, doc=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=1, mandatory=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=1, configurable=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=1, allow_empty=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=1, allow_files=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=1, allow_rules=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=1, providers=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=1, flags=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=1, aspects=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default=1, skip_validations=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=1, mandatory=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=1, configurable=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=1, allow_empty=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=1, allow_files=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=1, allow_rules=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=1, providers=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=1, flags=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=1, aspects=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(doc=1, skip_validations=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=1, configurable=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=1, allow_empty=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=1, allow_files=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=1, allow_rules=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=1, providers=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=1, flags=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=1, aspects=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(mandatory=1, skip_validations=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=1, allow_empty=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=1, allow_files=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=1, allow_rules=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=1, providers=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=1, flags=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=1, aspects=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(configurable=1, skip_validations=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=1, allow_files=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=1, allow_rules=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=1, providers=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=1, flags=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=1, aspects=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_empty=1, skip_validations=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=1, allow_rules=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=1, providers=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=1, flags=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=1, aspects=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_files=1, skip_validations=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=1, providers=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=1, flags=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=1, aspects=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(allow_rules=1, skip_validations=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers=1, flags=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers=1, aspects=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(providers=1, skip_validations=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags=1, aspects=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'flags' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(flags=1, skip_validations=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'flags' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_keyed_string_dict(aspects=1, skip_validations=1))"#,
        Err(
            r#"in call to label_keyed_string_dict(), parameter 'aspects' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=1, doc=1))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'int', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=1, configurable=1))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'int', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=1, allow_empty=1))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'int', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=1, allow_files=1))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'int', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=1, allow_rules=1))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'int', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=1, providers=1))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'int', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=1, flags=1))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'int', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=1, aspects=1))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'int', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list(), parameter 'default' got value of type 'int', want 'sequence or function'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=1, mandatory=1))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=1, configurable=1))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=1, allow_empty=1))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=1, allow_files=1))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=1, allow_rules=1))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=1, providers=1))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=1, flags=1))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=1, aspects=1))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(doc=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=1, configurable=1))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=1, allow_empty=1))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=1, allow_files=1))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=1, allow_rules=1))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=1, providers=1))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=1, flags=1))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=1, aspects=1))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(mandatory=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=1, allow_empty=1))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=1, allow_files=1))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=1, allow_rules=1))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=1, providers=1))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=1, flags=1))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=1, aspects=1))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(configurable=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=1, allow_files=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=1, allow_rules=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=1, providers=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=1, flags=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=1, aspects=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_empty=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files=1, allow_rules=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files=1, providers=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files=1, flags=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files=1, aspects=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_files=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_rules=1, providers=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_rules=1, flags=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_rules=1, aspects=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(allow_rules=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list(providers=1, flags=1))"#,
        Err(
            r#"in call to label_list(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(providers=1, aspects=1))"#,
        Err(
            r#"in call to label_list(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(providers=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(flags=1, aspects=1))"#,
        Err(
            r#"in call to label_list(), parameter 'flags' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(flags=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list(), parameter 'flags' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list(aspects=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list(), parameter 'aspects' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=1, doc=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=1, mandatory=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=1, configurable=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=1, allow_empty=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=1, allow_files=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=1, allow_rules=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=1, providers=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=1, flags=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=1, aspects=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(default=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=1, mandatory=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=1, configurable=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=1, allow_empty=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=1, allow_files=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=1, allow_rules=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=1, providers=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=1, flags=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=1, aspects=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(doc=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=1, configurable=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=1, allow_empty=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=1, allow_files=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=1, allow_rules=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=1, providers=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=1, flags=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=1, aspects=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(mandatory=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=1, allow_empty=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=1, allow_files=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=1, allow_rules=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=1, providers=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=1, flags=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=1, aspects=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(configurable=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=1, allow_files=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=1, allow_rules=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=1, providers=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=1, flags=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=1, aspects=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_empty=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=1, allow_rules=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=1, providers=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=1, flags=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=1, aspects=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_files=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=1, providers=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=1, flags=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=1, aspects=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(allow_rules=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(providers=1, flags=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(providers=1, aspects=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(providers=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(flags=1, aspects=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'flags' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(flags=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'flags' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.label_list_dict(aspects=1, skip_validations=1))"#,
        Err(
            r#"in call to label_list_dict(), parameter 'aspects' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.output(doc=1, mandatory=1))"#,
        Err(
            r#"in call to output(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output_list(doc=1, mandatory=1))"#,
        Err(
            r#"in call to output_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output_list(doc=1, allow_empty=1))"#,
        Err(
            r#"in call to output_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.output_list(mandatory=1, allow_empty=1))"#,
        Err(
            r#"in call to output_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string(default=1, doc=1))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'int', want 'string'"#),
    ),
    (
        r#"print(attr.string(default=1, mandatory=1))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'int', want 'string'"#),
    ),
    (
        r#"print(attr.string(default=1, values=1))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'int', want 'string'"#),
    ),
    (
        r#"print(attr.string(default=1, configurable=1))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'int', want 'string'"#),
    ),
    (
        r#"print(attr.string(doc=1, mandatory=1))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(doc=1, values=1))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(doc=1, configurable=1))"#,
        Err(
            r#"in call to string(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(mandatory=1, values=1))"#,
        Err(r#"in call to string(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.string(mandatory=1, configurable=1))"#,
        Err(r#"in call to string(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.string(values=1, configurable=1))"#,
        Err(r#"in call to string(), parameter 'values' got value of type 'int', want 'sequence'"#),
    ),
    (
        r#"print(attr.string_dict(default=1, doc=1))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default=1, mandatory=1))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default=1, configurable=1))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(default=1, allow_empty=1))"#,
        Err(
            r#"in call to string_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc=1, mandatory=1))"#,
        Err(
            r#"in call to string_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc=1, configurable=1))"#,
        Err(
            r#"in call to string_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(doc=1, allow_empty=1))"#,
        Err(
            r#"in call to string_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(mandatory=1, configurable=1))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(mandatory=1, allow_empty=1))"#,
        Err(
            r#"in call to string_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_dict(configurable=1, allow_empty=1))"#,
        Err(
            r#"in call to string_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=1, doc=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=1, mandatory=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=1, configurable=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=1, allow_empty=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=1, allow_files=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=1, allow_rules=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=1, providers=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=1, flags=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default=1, aspects=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'default' got value of type 'int', want 'dict or function'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=1, mandatory=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=1, configurable=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=1, allow_empty=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=1, allow_files=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=1, allow_rules=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=1, providers=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=1, flags=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(doc=1, aspects=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=1, configurable=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=1, allow_empty=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=1, allow_files=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=1, allow_rules=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=1, providers=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=1, flags=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(mandatory=1, aspects=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=1, allow_empty=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=1, allow_files=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=1, allow_rules=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=1, providers=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=1, flags=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(configurable=1, aspects=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=1, allow_files=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=1, allow_rules=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=1, providers=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=1, flags=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_empty=1, aspects=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_empty' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=1, allow_rules=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=1, providers=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=1, flags=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_files=1, aspects=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules=1, providers=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules=1, flags=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(allow_rules=1, aspects=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'allow_rules' got value of type 'int', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers=1, flags=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(providers=1, aspects=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(flags=1, aspects=1))"#,
        Err(
            r#"in call to string_keyed_label_dict(), parameter 'flags' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=1, doc=1))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=1, mandatory=1))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=1, configurable=1))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=1, allow_empty=1))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(doc=1, mandatory=1))"#,
        Err(
            r#"in call to string_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list(doc=1, configurable=1))"#,
        Err(
            r#"in call to string_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list(doc=1, allow_empty=1))"#,
        Err(
            r#"in call to string_list(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory=1, configurable=1))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory=1, allow_empty=1))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(configurable=1, allow_empty=1))"#,
        Err(
            r#"in call to string_list(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default=1, doc=1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default=1, mandatory=1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default=1, configurable=1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default=1, allow_empty=1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'default' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc=1, mandatory=1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc=1, configurable=1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(doc=1, allow_empty=1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(mandatory=1, configurable=1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(mandatory=1, allow_empty=1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(configurable=1, allow_empty=1))"#,
        Err(
            r#"in call to string_list_dict(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    ),
    (
        r#"print(attr.label(default="//a:b:c", flags=["a"]))"#,
        Err(
            r#"invalid label '//a:b:c' in parameter 'default' of attribute 'label': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label(default="//a:b:c", allow_files=[1]))"#,
        Err(
            r#"invalid label '//a:b:c' in parameter 'default' of attribute 'label': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label(default="//a:b:c", allow_single_file=[1]))"#,
        Err(
            r#"invalid label '//a:b:c' in parameter 'default' of attribute 'label': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label(default="//a:b:c", allow_rules=[1]))"#,
        Err(
            r#"invalid label '//a:b:c' in parameter 'default' of attribute 'label': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label(default="//a:b:c", providers=[1]))"#,
        Err(
            r#"invalid label '//a:b:c' in parameter 'default' of attribute 'label': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label(default="//a:b:c", aspects=[1]))"#,
        Err(
            r#"invalid label '//a:b:c' in parameter 'default' of attribute 'label': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label(flags=["a"], default="//a:b:c"))"#,
        Err(
            r#"invalid label '//a:b:c' in parameter 'default' of attribute 'label': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label(flags=["a"], allow_single_file=[1]))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label(allow_files=[1], default="//a:b:c"))"#,
        Err(
            r#"invalid label '//a:b:c' in parameter 'default' of attribute 'label': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=[1], flags=["a"]))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label(allow_files=[1], aspects=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(allow_files=[1], executable=True))"#,
        Err(
            r#"cfg parameter is mandatory when executable=True is provided. Please see https://bazel.build/extending/rules#configurations for more details."#,
        ),
    ),
    (
        r#"print(attr.label(allow_single_file=[1], default="//a:b:c"))"#,
        Err(
            r#"invalid label '//a:b:c' in parameter 'default' of attribute 'label': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_single_file=[1], flags=["a"]))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label(allow_single_file=[1], allow_files=[1]))"#,
        Err(r#"Cannot specify both allow_files and allow_single_file"#),
    ),
    (
        r#"print(attr.label(allow_single_file=[1], providers=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(allow_single_file=[1], cfg="x"))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(allow_single_file=[1], aspects=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(allow_single_file=[1], executable=True))"#,
        Err(
            r#"cfg parameter is mandatory when executable=True is provided. Please see https://bazel.build/extending/rules#configurations for more details."#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=[1], default="//a:b:c"))"#,
        Err(
            r#"invalid label '//a:b:c' in parameter 'default' of attribute 'label': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=[1], flags=["a"]))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label(allow_rules=[1], allow_single_file=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(allow_rules=[1], providers=[1]))"#,
        Err(
            r#"at index 0 of allowed rule classes for attribute definition, got element of type int, want string"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=[1], cfg="x"))"#,
        Err(
            r#"at index 0 of allowed rule classes for attribute definition, got element of type int, want string"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=[1], aspects=[1]))"#,
        Err(
            r#"at index 0 of allowed rule classes for attribute definition, got element of type int, want string"#,
        ),
    ),
    (
        r#"print(attr.label(allow_rules=[1], executable=True))"#,
        Err(
            r#"cfg parameter is mandatory when executable=True is provided. Please see https://bazel.build/extending/rules#configurations for more details."#,
        ),
    ),
    (
        r#"print(attr.label(providers=[1], default="//a:b:c"))"#,
        Err(
            r#"invalid label '//a:b:c' in parameter 'default' of attribute 'label': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label(providers=[1], flags=["a"]))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label(providers=[1], allow_files=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(providers=[1], allow_single_file=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(providers=[1], allow_rules=[1]))"#,
        Err(
            r#"at index 0 of allowed rule classes for attribute definition, got element of type int, want string"#,
        ),
    ),
    (
        r#"print(attr.label(providers=[1], executable=True))"#,
        Err(
            r#"cfg parameter is mandatory when executable=True is provided. Please see https://bazel.build/extending/rules#configurations for more details."#,
        ),
    ),
    (
        r#"print(attr.label(cfg="x", default="//a:b:c"))"#,
        Err(
            r#"invalid label '//a:b:c' in parameter 'default' of attribute 'label': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label(cfg="x", allow_single_file=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(aspects=[1], default="//a:b:c"))"#,
        Err(
            r#"invalid label '//a:b:c' in parameter 'default' of attribute 'label': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label(aspects=[1], flags=["a"]))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label(aspects=[1], allow_files=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(aspects=[1], allow_single_file=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(aspects=[1], allow_rules=[1]))"#,
        Err(
            r#"at index 0 of allowed rule classes for attribute definition, got element of type int, want string"#,
        ),
    ),
    (
        r#"print(attr.label(aspects=[1], executable=True))"#,
        Err(
            r#"cfg parameter is mandatory when executable=True is provided. Please see https://bazel.build/extending/rules#configurations for more details."#,
        ),
    ),
    (
        r#"print(attr.label(executable=True, default="//a:b:c"))"#,
        Err(
            r#"invalid label '//a:b:c' in parameter 'default' of attribute 'label': invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(attr.label(executable=True, flags=["a"]))"#,
        Err(r#"unknown attribute flag 'a'"#),
    ),
    (
        r#"print(attr.label(executable=True, allow_files=[1]))"#,
        Err(
            r#"cfg parameter is mandatory when executable=True is provided. Please see https://bazel.build/extending/rules#configurations for more details."#,
        ),
    ),
    (
        r#"print(attr.label(executable=True, allow_single_file=[1]))"#,
        Err(
            r#"cfg parameter is mandatory when executable=True is provided. Please see https://bazel.build/extending/rules#configurations for more details."#,
        ),
    ),
    (
        r#"print(attr.label(executable=True, allow_rules=[1]))"#,
        Err(
            r#"cfg parameter is mandatory when executable=True is provided. Please see https://bazel.build/extending/rules#configurations for more details."#,
        ),
    ),
    (
        r#"print(attr.label(executable=True, providers=[1]))"#,
        Err(
            r#"cfg parameter is mandatory when executable=True is provided. Please see https://bazel.build/extending/rules#configurations for more details."#,
        ),
    ),
    (
        r#"print(attr.label(executable=True, aspects=[1]))"#,
        Err(
            r#"cfg parameter is mandatory when executable=True is provided. Please see https://bazel.build/extending/rules#configurations for more details."#,
        ),
    ),
    (
        r#"print(attr.label(flags=["ALLOW_EMPTY"]))"#,
        Err(r#"unknown attribute flag 'ALLOW_EMPTY'"#),
    ),
    (
        r#"print(attr.label(flags=["NON_CONFIGURABLE"]))"#,
        Err(r#"unknown attribute flag 'NON_CONFIGURABLE'"#),
    ),
    (
        r#"print(attr.label(flags=["STARLARK_DEFINED"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(flags=["HOST_DEP_PUBLIC"]))"#,
        Err(r#"unknown attribute flag 'HOST_DEP_PUBLIC'"#),
    ),
    (
        r#"print(attr.label(flags=["LABEL_LIST"]))"#,
        Err(r#"unknown attribute flag 'LABEL_LIST'"#),
    ),
    (
        r#"print(attr.label(flags=["CONFIGURABLE"]))"#,
        Err(r#"unknown attribute flag 'CONFIGURABLE'"#),
    ),
    (
        r#"print(attr.label(flags=["SKIP_ANALYSIS"]))"#,
        Err(r#"unknown attribute flag 'SKIP_ANALYSIS'"#),
    ),
    (
        r#"print(attr.label(flags=["DEPRECATED"]))"#,
        Err(r#"unknown attribute flag 'DEPRECATED'"#),
    ),
    (
        r#"print(attr.label(flags=["PRIVATE"]))"#,
        Err(r#"unknown attribute flag 'PRIVATE'"#),
    ),
    (
        r#"print(attr.label(flags=["REQUIRED"]))"#,
        Err(r#"unknown attribute flag 'REQUIRED'"#),
    ),
    (
        r#"print(attr.label(flags=["OPTIONAL"]))"#,
        Err(r#"unknown attribute flag 'OPTIONAL'"#),
    ),
    (
        r#"print(attr.label(flags=["NONEMPTY"]))"#,
        Err(r#"unknown attribute flag 'NONEMPTY'"#),
    ),
    (
        r#"print(attr.label(flags=["DIRECT_COMPILE_TIME_INPUTS"]))"#,
        Err(r#"unknown attribute flag 'DIRECT_COMPILE_TIME_INPUTS'"#),
    ),
    (
        r#"print(attr.label(flags=["SINGLE"]))"#,
        Err(r#"unknown attribute flag 'SINGLE'"#),
    ),
    (
        r#"print(attr.label(flags=["ORDER_INDEPENDENT_ALLOW_FILES"]))"#,
        Err(r#"unknown attribute flag 'ORDER_INDEPENDENT_ALLOW_FILES'"#),
    ),
    (
        r#"print(attr.label(flags=["TREAT_AS_DEP"]))"#,
        Err(r#"unknown attribute flag 'TREAT_AS_DEP'"#),
    ),
    (
        r#"print(attr.string(mandatory=1, default=1))"#,
        Err(r#"in call to string(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.string(foo=1, default=1))"#,
        Err(r#"string() got unexpected keyword argument 'foo'"#),
    ),
    (
        r#"print(attr.string(default=1, foo=1))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'int', want 'string'"#),
    ),
    (
        r#"print(attr.string(foo=1, bar=1))"#,
        Err(r#"string() got unexpected keyword argument 'foo'"#),
    ),
    (
        r#"print(attr.string(bar=1, foo=1))"#,
        Err(r#"string() got unexpected keyword argument 'bar'"#),
    ),
    (
        r#"print(attr.string(1, default=1))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'int', want 'string'"#),
    ),
    (
        r#"print(attr.string_list(1, default=1))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(True, default=1, allow_empty=1))"#,
        Err(
            r#"in call to string_list(), parameter 'default' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=1, allow_empty=1, True))"#,
        Err(r#"positional argument may not follow keyword argument"#),
    ),
    (
        r#"print(attr.string_list(mandatory=1, default=1))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=[1], mandatory=1))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.string_list(mandatory=1, default=[1]))"#,
        Err(
            r#"in call to string_list(), parameter 'mandatory' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"print(attr.int(mandatory=1, default="a"))"#,
        Err(r#"in call to int(), parameter 'mandatory' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.label(executable=True, allow_files=True, allow_single_file=True))"#,
        Err(
            r#"cfg parameter is mandatory when executable=True is provided. Please see https://bazel.build/extending/rules#configurations for more details."#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=True, allow_single_file=True, executable=True))"#,
        Err(
            r#"cfg parameter is mandatory when executable=True is provided. Please see https://bazel.build/extending/rules#configurations for more details."#,
        ),
    ),
    (
        r#"print(attr.label(allow_files=1, default=1))"#,
        Err(
            r#"in call to label(), parameter 'allow_files' got value of type 'int', want 'bool, sequence, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(cfg="x", default=1))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(default=1, cfg="x"))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.label(flags=["a"], default=1))"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"print(attr.string(**{"default": 1}))"#,
        Err(r#"in call to string(), parameter 'default' got value of type 'int', want 'string'"#),
    ),
    (
        r#"print(attr.string(**{"foo": 1}))"#,
        Err(r#"string() got unexpected keyword argument 'foo'"#),
    ),
    (
        r#"print(attr.string(**{"default": "a"}))"#,
        Ok(r#"<attr.string>"#),
    ),
    (
        r#"print(attr.string(*[1]))"#,
        Err(r#"string() got unexpected positional argument"#),
    ),
    (r#"print(attr.string(*[]))"#, Ok(r#"<attr.string>"#)),
    (
        r#"print(attr.string_list(*[True, False]))"#,
        Ok(r#"<attr.string_list>"#),
    ),
    (
        r#"print(attr.label_list(default=["//a:b"], allow_files=True, cfg="exec", providers=[[]], allow_rules=["x"], aspects=[], flags=["MANDATORY"], doc="d", mandatory=False, allow_empty=False, skip_validations=True, for_dependency_resolution=1, configurable=False))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label(default=None, doc=None, executable=False, allow_files=None, allow_single_file=None, mandatory=False, skip_validations=False, providers=[], allow_rules=None, cfg=None, aspects=[], flags=[], configurable=True))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_files=[".a"], allow_single_file=None))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_files=None, allow_single_file=[".a"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_files=(".a",".b")))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_rules=("a","b")))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_rules=["a", "a"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(allow_rules=["nonexistent rule"]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(providers=[[DefaultInfo, OutputGroupInfo]]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(providers=[[DefaultInfo], []]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(providers=([], [DefaultInfo])))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(attr.label(providers=[DefaultInfo, DefaultInfo]))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"print(DefaultInfo)
print(type(DefaultInfo))
print(type(provider()))"#,
        Ok(r#"<function DefaultInfo>
Provider
Provider"#),
    ),
    (
        r#"print(attr.label(providers=[[DefaultInfo, 1], 1]))"#,
        Err(r#"at index 1 of providers, got element of type int, want sequence"#),
    ),
    (
        r#"print(attr.label(providers=[[1], 1]))"#,
        Err(r#"at index 1 of providers, got element of type int, want sequence"#),
    ),
    (
        r#"print(attr.label(providers=[[DefaultInfo, ("a",)]]))"#,
        Err(r#"at index 1 of providers, got element of type tuple, want Provider"#),
    ),
    (
        r#"print(attr.label(providers=[[DefaultInfo, "a"]]))"#,
        Err(r#"at index 1 of providers, got element of type string, want Provider"#),
    ),
    (
        r#"print(attr.label(providers=[[DefaultInfo, "a"]], allow_files=[1]))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(providers=["a"]))"#,
        Err(r#"at index 0 of providers, got element of type string, want sequence"#),
    ),
    (
        r#"print(attr.string(configurable=True) == attr.string())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.string(configurable=False) == attr.string())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(default="//a:b") == attr.label(default=Label("//a:b")))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(default="//a:b") == attr.label(default=":b"))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(default="@nope//a:b") == attr.label(default="@nope2//a:b"))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(default=lambda: 1) == attr.label(default=lambda: 1))"#,
        Ok(r#"False"#),
    ),
    (
        r#"f = lambda: 1
print(attr.label(default=f) == attr.label(default=f))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(allow_files=True) == attr.label(allow_files=[]))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(allow_files=[".a"]) == attr.label(allow_files=(".a",)))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(allow_files=[".a"]) == attr.label(allow_single_file=[".a"]))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(allow_files=True) == attr.label(allow_single_file=True))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(allow_files=False) == attr.label())"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(allow_files=None) == attr.label())"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(providers=[DefaultInfo]) == attr.label(providers=[[DefaultInfo]]))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(providers=[DefaultInfo]) == attr.label(providers=[[DefaultInfo],[]]))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(providers=[]) == attr.label(providers=[[]]))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(cfg="target") == attr.label())"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(cfg="exec") == attr.label(cfg="host"))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(flags=["MANDATORY"]) == attr.label(mandatory=True))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(flags=["MANDATORY"]) == attr.label(flags=["MANDATORY","MANDATORY"]))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(flags=["SINGLE_ARTIFACT"]) == attr.label(allow_single_file=True))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(allow_rules=[]) == attr.label())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(allow_rules=["a","b"]) == attr.label(allow_rules=["b","a"]))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=True) == attr.label())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(skip_validations=True) == attr.label())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(doc="") == attr.label())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(doc=None) == attr.label())"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.string(values=[]) == attr.string())"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.string(values=["a"]) == attr.string(values=("a",)))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.string(values=["a", "b"]) == attr.string(values=["b", "a"]))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.string(default="") == attr.string())"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.string_list(default=[]) == attr.string_list())"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.string_list(default=["a"]) == attr.string_list(default=("a",)))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.string_list(allow_empty=True) == attr.string_list())"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.string_list(True, True) == attr.string_list(mandatory=True, allow_empty=True))"#,
        Ok(r#"True"#),
    ),
    (r#"print(attr.int() == attr.int(default=0))"#, Ok(r#"True"#)),
    (
        r#"print(attr.bool() == attr.bool(default=False))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.string_dict(default={"a":"b","c":"d"}) == attr.string_dict(default={"c":"d","a":"b"}))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.string_dict() == attr.string_dict(default={}))"#,
        Ok(r#"True"#),
    ),
    (r#"print(attr.output() == attr.output())"#, Ok(r#"True"#)),
    (
        r#"print(attr.output_list() == attr.output_list(allow_empty=True))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label_list(default=["//a:b"]) == attr.label_list(default=[Label("//a:b")]))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label_list(default=["//a:b", "//a:c"]) == attr.label_list(default=["//a:c", "//a:b"]))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={"//a:b":"x"}) == attr.label_keyed_string_dict(default={Label("//a:b"):"x"}))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={"x":"//a:b"}) == attr.string_keyed_label_dict(default={"x":Label("//a:b")}))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label_list_dict(default={"a":["//a:b"]}) == attr.label_list_dict(default={"a":[Label("//a:b")]}))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label_list_dict(default={"a":("//a:b",)}) == attr.label_list_dict(default={"a":["//a:b"]}))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.string_list_dict(default={"a":("b",)}) == attr.string_list_dict(default={"a":["b"]}))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(default=None) == attr.label())"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(default=None, mandatory=True) == attr.label(mandatory=True))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.string(default="a", mandatory=True) == attr.string(default="a", mandatory=False))"#,
        Ok(r#"False"#),
    ),
    (r#"print(bool(attr.string()))"#, Ok(r#"True"#)),
    (
        r#"print(str([attr.string(doc="x")]))"#,
        Ok(r#"[<attr.string>]"#),
    ),
    (
        r#"print(attr.string().__class__)"#,
        Err(r#"'Attribute' value has no field or method '__class__'"#),
    ),
    (
        r#"print(type(attr.string) )"#,
        Ok(r#"builtin_function_or_method"#),
    ),
    (
        r#"print(attr.string)"#,
        Ok(r#"<built-in method string of attr value>"#),
    ),
    (
        r#"print(attr.label_list)"#,
        Ok(r#"<built-in method label_list of attr value>"#),
    ),
    (
        r#"print(repr(attr.label))"#,
        Ok(r#"<built-in method label of attr value>"#),
    ),
    (
        r#"print(str(attr.label))"#,
        Ok(r#"<built-in method label of attr value>"#),
    ),
    (
        r#"print(attr.string.__name__)"#,
        Err(r#"'builtin_function_or_method' value has no field or method '__name__'"#),
    ),
    (r#"print(hasattr(attr, "string"))"#, Ok(r#"True"#)),
    (r#"print(hasattr(attr, "nope"))"#, Ok(r#"False"#)),
    (r#"print(getattr(attr, "nope", 1))"#, Ok(r#"1"#)),
    (
        r#"print(attr.nope)"#,
        Err(r#"'attr' value has no field or method 'nope'"#),
    ),
    (
        r#"print(attr.stringg)"#,
        Err(r#"'attr' value has no field or method 'stringg' (did you mean 'string'?)"#),
    ),
    (
        r#"print(attr.strin)"#,
        Err(r#"'attr' value has no field or method 'strin' (did you mean 'string'?)"#),
    ),
    (
        r#"print(attr.label_lis)"#,
        Err(r#"'attr' value has no field or method 'label_lis' (did you mean 'label_list'?)"#),
    ),
    (
        r#"print(attr["string"])"#,
        Err(r#"type 'attr' has no operator [](string)"#),
    ),
    (
        r#"attr.foo = 1"#,
        Err(r#"cannot set .foo field of attr value"#),
    ),
    (
        r#"x = attr.string
print(x == attr.string)"#,
        Ok(r#"False"#),
    ),
    (r#"print(attr == attr)"#, Ok(r#"True"#)),
    (r#"print(bool(attr))"#, Ok(r#"True"#)),
    (r#"print(attr())"#, Err(r#"'attr' object is not callable"#)),
    (
        r#"print(attr.string(default="a").default)"#,
        Err(r#"'Attribute' value has no field or method 'default'"#),
    ),
    (
        r#"print(attr.string().mandatory)"#,
        Err(r#"'Attribute' value has no field or method 'mandatory'"#),
    ),
    (
        r#"print(getattr(attr.string(), "doc", None))"#,
        Ok(r#"None"#),
    ),
    (
        r#"attr.string(doc="x").doc = 1"#,
        Err(r#"cannot set .doc field of Attribute value"#),
    ),
    (
        r#"print(attr.string()())"#,
        Err(r#"'Attribute' object is not callable"#),
    ),
    (
        r#"print(len(attr.string()))"#,
        Err(
            r#"in call to len(), parameter 'x' got value of type 'Attribute', want 'iterable or string'"#,
        ),
    ),
    (
        r#"print(list(attr.string()))"#,
        Err(r#"in call to list(), parameter 'x' got value of type 'Attribute', want 'iterable'"#),
    ),
    (r#"print(dir(attr.string))"#, Ok(r#"[]"#)),
    (
        r#"print(attr.label(allow_files=[".a"]) == attr.label(allow_files=[".a"]))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(allow_files=[".a", ".b"]) == attr.label(allow_files=[".b", ".a"]))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.string(values=["a"]) == attr.string(values=["a"]))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.int(values=[1]) == attr.int(values=[1]))"#,
        Ok(r#"False"#),
    ),
    (
        r#"x = attr.string(values=["a"])
print(x == x)"#,
        Ok(r#"True"#),
    ),
    (
        r#"x = attr.label(default=lambda: 1)
print(x == x)"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(allow_single_file=[".a"]) == attr.label(allow_single_file=[".a"]))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(providers=[[DefaultInfo, OutputGroupInfo]]) == attr.label(providers=[[OutputGroupInfo, DefaultInfo]]))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(providers=[[DefaultInfo], [OutputGroupInfo]]) == attr.label(providers=[[OutputGroupInfo], [DefaultInfo]]))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(cfg=config.exec()) == attr.label(cfg=config.exec()))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(cfg=config.target()) == attr.label(cfg=config.target()))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(cfg="exec") == attr.label(cfg="exec"))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(cfg="exec") == attr.label(cfg=config.exec()))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(aspects=[]) == attr.label())"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(executable=True, cfg="exec") == attr.label(executable=True, cfg="exec"))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(executable=True, cfg="exec") == attr.label(cfg="exec"))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(default="//a:b") == attr.label(default="//a"))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(default="//a:b", allow_rules=["x"]) == attr.label(default="//a:b", allow_rules=["x"]))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.string(doc="x") == attr.string(doc="x"))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.string(configurable=False) == attr.string(configurable=False))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(flags=["SINGLE_ARTIFACT"]) == attr.label(flags=["SINGLE_ARTIFACT"]))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(flags=["NON_EMPTY"]) == attr.label_list(allow_empty=False))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label_list(flags=["NON_EMPTY"]) == attr.label_list(allow_empty=False))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label_list(flags=["NON_EMPTY"]) == attr.label_list())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label_list(flags=["MANDATORY"]) == attr.label_list(mandatory=True))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label_list(flags=["EXECUTABLE"]) == attr.label_list())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label_list(flags=["UNDOCUMENTED"]) == attr.label_list())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label_list(flags=["OUTPUT_LICENSES"]) == attr.label_list())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label_list(flags=["STARLARK_DEFINED"]) == attr.label_list())"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label_list(flags=["ORDER_INDEPENDENT"]) == attr.label_list())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label_list(flags=["DIRECT_COMPILE_TIME_INPUT"]) == attr.label_list())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label_list(flags=["SKIP_ANALYSIS_TIME_FILETYPE_CHECK"]) == attr.label_list())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label_list(flags=["SKIP_CONSTRAINTS_OVERRIDE"]) == attr.label_list())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label_list(flags=["ORDER_INDEPENDENT", "UNDOCUMENTED"]) == attr.label_list(flags=["UNDOCUMENTED", "ORDER_INDEPENDENT"]))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label_list(allow_files=True, flags=["SINGLE_ARTIFACT"]) == attr.label_list(allow_files=True))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label_list(skip_validations=False) == attr.label_list())"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label_list(allow_empty=False) == attr.label_list(allow_empty=False))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=None) == attr.label())"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=False) == attr.label())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=1) == attr.label())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution="a") == attr.label())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=True) == attr.label(for_dependency_resolution=True))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=[]) == attr.label())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=0) == attr.label())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label_list(for_dependency_resolution=True) == attr.label_list())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(for_dependency_resolution=lambda x: 1) == attr.label())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(executable=True, cfg="exec") == attr.label(executable=True, cfg="exec", flags=["EXECUTABLE"]))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(allow_single_file=True) == attr.label(allow_single_file=True))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(allow_single_file=False) == attr.label())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(allow_single_file=None) == attr.label())"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(allow_files=True, flags=["SINGLE_ARTIFACT"]) == attr.label(allow_single_file=True))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(allow_files=True) == attr.label(allow_files=True))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(allow_files=True, allow_rules=[]) == attr.label(allow_files=True, allow_rules=[]))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(cfg="target", executable=True) == attr.label(cfg="target", executable=True))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(cfg="none") )"#,
        Err(
            r#"cfg must be either 'target', 'exec' or a starlark defined transition defined by the exec() or transition() functions."#,
        ),
    ),
    (
        r#"print(attr.label(cfg="host") == attr.label(cfg="host"))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(providers=[DefaultInfo]) == attr.label(providers=[DefaultInfo]))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(providers=[[DefaultInfo, DefaultInfo]]) == attr.label(providers=[[DefaultInfo]]))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(default=Label("//a:b")) == attr.label(default=Label("//a:b")))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(default=lambda: 1, mandatory=True) == attr.label(mandatory=True))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label(default="//a:b", flags=["MANDATORY"]) == attr.label(default="//a:b", mandatory=True))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label(mandatory=False, flags=["MANDATORY"]) == attr.label(mandatory=True))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.string_list(mandatory=False, flags=["MANDATORY"]) == attr.string_list())"#,
        Err(r#"string_list() got unexpected keyword argument 'flags'"#),
    ),
    (
        r#"print(attr.label_list(allow_empty=True, flags=["NON_EMPTY"]) == attr.label_list())"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(attr.label_list(allow_empty=True, flags=["NON_EMPTY"]) == attr.label_list(allow_empty=False))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(attr.label_list_dict(default={"a":depset(["//a:b"])}))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.label_list_dict(default={"a":depset([1])}))"#,
        Err(
            r#"expected value of type 'string' for element 0 of dict value element, but got 1 (int)"#,
        ),
    ),
    (
        r#"print(attr.string_list_dict(default={"a":depset()}))"#,
        Ok(r#"<attr.string_list_dict>"#),
    ),
    (
        r#"print(attr.string_list_dict(default={"a":depset(["x"])}))"#,
        Ok(r#"<attr.string_list_dict>"#),
    ),
    (
        r#"print(attr.string_list(default=range(2)))"#,
        Err(
            r#"expected value of type 'string' for element 0 of parameter 'default' of attribute '', but got 0 (int)"#,
        ),
    ),
    (
        r#"print(attr.string(values=range(2)))"#,
        Ok(r#"<attr.string>"#),
    ),
    (
        r#"print(attr.int_list(default=range(2)))"#,
        Ok(r#"<attr.int_list>"#),
    ),
    (
        r#"print(attr.label(allow_files=range(2)))"#,
        Err(r#"at index 0 of allow_files argument, got element of type int, want string"#),
    ),
    (
        r#"print(attr.label(providers=range(2)))"#,
        Err(r#"at index 0 of providers, got element of type int, want sequence"#),
    ),
    (
        r#"print(attr.label_list(default=range(2)))"#,
        Err(
            r#"expected value of type 'string' for element 0 of parameter 'default' of attribute 'label_list', but got 0 (int)"#,
        ),
    ),
    (
        r#"print(attr.string_list(default=[""]))"#,
        Ok(r#"<attr.string_list>"#),
    ),
    (
        r#"print(attr.label_list(default=[""]))"#,
        Err(
            r#"invalid label '' in element 0 of parameter 'default' of attribute 'label_list': invalid target name '': empty target name"#,
        ),
    ),
    (
        r#"print(attr.label_list(default=[Label("//a:b"), Label("//a:b")]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list(default=["//a:b", "//a:b"]))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.label_list_dict(default={"a":["//a:b", "//a:b"]}))"#,
        Ok(r#"<attr.label_list_dict>"#),
    ),
    (
        r#"print(attr.string_dict(default={"a":"b", "a":"c"}))"#,
        Err(r#"dictionary expression has duplicate key: "a""#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={Label("//a:b"):"x", Label("//a:b"):"y"}))"#,
        Err(r#"dictionary expression has duplicate key: Label("//a:b")"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={"//a:b":"x", ":b":"y"}))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={"//a:b":"x", "//a:c":"y", "//a:b ":"z"}))"#,
        Ok(r#"<attr.label_keyed_string_dict>"#),
    ),
    (
        r#"print(attr.label_keyed_string_dict(default={Label("//a:b"):"x", "//a:c":"y", "//a:b":"z", ":c": "w"}))"#,
        Err(
            r#"duplicate labels in parameter 'default' of attribute 'label_keyed_string_dict': //a:b (as [Label("//a:b"), "//a:b"])"#,
        ),
    ),
    (
        r#"print(attr.string_keyed_label_dict(default={"x":"//a:b", "y":"//a:b"}))"#,
        Ok(r#"<attr.string_keyed_label_dict>"#),
    ),
    (
        r#"print(attr.label_list(default=["//a:b"], doc=None, mandatory=True, allow_empty=False))"#,
        Ok(r#"<attr.label_list>"#),
    ),
    (
        r#"print(attr.int(default=True))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'bool', want 'int'"#),
    ),
    (
        r#"print(attr.int(default=1.0))"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'float', want 'int'"#),
    ),
    (
        r#"print(attr.bool(default=1))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.bool(default=0))"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"print(attr.int_list(default=(1,2)))"#,
        Ok(r#"<attr.int_list>"#),
    ),
    (
        r#"print(attr.output(mandatory=True, doc="x", configurable=False))"#,
        Err(r#"output() got unexpected keyword argument 'configurable'"#),
    ),
    (
        r#"print(attr.output_list(configurable=False))"#,
        Err(r#"output_list() got unexpected keyword argument 'configurable'"#),
    ),
    (
        r#"print(attr.string(configurable=False, default="a"))"#,
        Ok(r#"<attr.string>"#),
    ),
    (
        r#"print(attr.label(default=Label("//a:b"), configurable=False))"#,
        Ok(r#"<attr.label>"#),
    ),
];
