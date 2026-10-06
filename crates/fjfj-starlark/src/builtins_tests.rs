//! `struct`, `json`, `proto.encode_text` and `set` against Bazel 9.2.0
//! (buildfiji-mum.3.1): the tables at the bottom are what it printed and
//! reported for the same `.bzl` bodies, generated from probe runs, so a
//! disagreement is fjfj's to explain.

use crate::test_support::replay;

/// Probes whose answer is the Starlark runtime's generic wording (how an
/// unsupported operator, iteration, indexing, `hash()` or `len()` fails),
/// which belongs to buildfiji-v32.
const RUNTIME_WORDING: &[&str] = &[
    // How a builtin, a lambda and a builtin's type print and are named.
    "print(struct)",
    "type(struct)",
    "struct.__name__",
    "struct(a=print)",
    "struct(x=struct)",
    "lambda: 1",
    "print(set)",
    "print(set([1]).add)",
    ".union.__name__",
    // Java class names.
    "print(json)",
    "print(proto)",
    "proto.encode_text)",
    "hash(",
    // A row whose output spans lines, and rows that `repr` a non-ASCII string
    // (buildfiji-s9u).
    "u00e9",
    "é",
    "\\177",
    // `repr` with three arguments, and providers (buildfiji-mum.3.4).
    "repr(json.indent('[]')",
    "provider()",
    "print(proto.encode_text(struct(a=1)))",
    // In-place set operators (buildfiji-tg2).
    "s |= ",
    "s &= ",
    "s -= ",
    "s ^= ",
    // A row that never defined `X`, and builtins as set elements, which
    // Bazel calls unhashable (the same gap as buildfiji-ahp), and method
    // names.
    "def f(): X.add(2)",
    "set([print])",
    "print(set(set([1,2])).add)",
    "type(set([1]).union)",
];

/// Messages the runtime words the same way for every type.
const GENERIC: &[&str] = &["cannot encode Provider"];

fn check(rows: &[(&str, Result<&str, &str>)]) {
    let wrong = replay(rows, RUNTIME_WORDING, GENERIC);
    assert!(
        wrong.is_empty(),
        "{} differ:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

/// `struct` is a `.bzl` builtin; `set`, `json` and `proto` are in both.
#[test]
fn scopes_are_bazels() {
    use crate::{build_globals, bzl_globals};
    let has = |globals: &starlark::environment::Globals, name: &str| {
        globals.names().any(|n| n.as_str() == name)
    };
    let (build, bzl) = (build_globals(), bzl_globals());
    assert!(!has(&build, "struct"));
    assert!(has(&bzl, "struct"));
    for name in ["set", "json", "proto"] {
        assert!(has(&build, name), "{name} in BUILD");
        assert!(has(&bzl, name), "{name} in .bzl");
    }
}

/// Sets and structs survive freezing, still work from another file, and a
/// frozen set refuses to change.
#[test]
fn sets_and_structs_survive_freezing() {
    use crate::test_support::Capture;
    use crate::{FileKind, bzl_globals, parse};
    use starlark::environment::Module;
    use starlark::eval::{Evaluator, FileLoader};
    use std::cell::RefCell;

    struct Loader;
    impl FileLoader for Loader {
        fn load(&self, _path: &str) -> starlark::Result<starlark::environment::FrozenModule> {
            let src = "S = set([1, 2])\nT = struct(b = set([3]), a = [1])\nE = set()\n";
            let ast = parse("lib.bzl", src, FileKind::Bzl).map_err(starlark::Error::new_other)?;
            Module::with_temp_heap(|module| {
                {
                    let mut eval = Evaluator::new(&module);
                    eval.eval_module(ast, &bzl_globals())?;
                }
                Ok(module.freeze()?)
            })
        }
    }
    let run_main = |body: &str| {
        let src = format!("load(\":lib.bzl\", \"S\", \"T\", \"E\")\n{body}");
        let ast = parse("t.bzl", &src, FileKind::Bzl).unwrap();
        let capture = Capture(RefCell::new(Vec::new()));
        let result = Module::with_temp_heap(|module| {
            let mut eval = Evaluator::new(&module);
            eval.set_loader(&Loader);
            eval.set_print_handler(&capture);
            eval.eval_module(ast, &bzl_globals())
                .map(|_| ())
                .map_err(|e| format!("{:#}", e.into_anyhow()))
        });
        result.map(|()| capture.0.into_inner())
    };
    assert_eq!(
        run_main(
            "print(S, E, T, len(S), 2 in S)\nprint(S | set([9]), S.union([5]), T.b.union([4]))\nprint(S == set([2, 1]), T == struct(a = [1], b = set([3])))\nprint(json.encode(T), proto.encode_text(struct(t = struct(a = T.a))))\nlive = set([0])\nlive.update(S)\nprint(live)\n"
        )
        .unwrap(),
        [
            "set([1, 2]) set() struct(a = [1], b = set([3])) 2 True",
            "set([1, 2, 9]) set([1, 2, 5]) set([3, 4])",
            "True True",
            "{\"a\":[1],\"b\":[3]} t {\n  a: 1\n}\n",
            "set([0, 1, 2])",
        ]
    );
    for call in [
        "S.add(3)",
        "S.clear()",
        "S.discard(9)",
        "S.update([])",
        "S.pop()",
        "T.b.add(1)",
    ] {
        let err = run_main(call).unwrap_err();
        assert!(
            err.contains("trying to mutate a frozen set value"),
            "{call}: {err}"
        );
    }
}

#[test]
fn struct_matches_bazel() {
    check(STRUCT_CASES);
}

#[test]
fn json_matches_bazel() {
    check(JSON_CASES);
}

#[test]
fn proto_encode_text_matches_bazel() {
    check(PROTO_CASES);
}

#[test]
fn set_matches_bazel() {
    check(SET_CASES);
}

const STRUCT_CASES: &[(&str, Result<&str, &str>)] = &[
    (r#"print(struct(a=1))"#, Ok(r#"struct(a = 1)"#)),
    (r#"print(struct())"#, Ok(r#"struct()"#)),
    (
        r#"print(struct(a=1,b="x",c=[1,2],d={"k":struct(z=1)}))"#,
        Ok(r#"struct(a = 1, b = "x", c = [1, 2], d = {"k": struct(z = 1)})"#),
    ),
    (r#"print(type(struct(a=1)))"#, Ok(r#"struct"#)),
    (r#"print(dir(struct(a=1,b=2)))"#, Ok(r#"["a", "b"]"#)),
    (r#"print(dir(struct()))"#, Ok(r#"[]"#)),
    (
        r#"print(struct(a=1)==struct(a=1), struct(a=1)==struct(a=2), struct(a=1,b=2)==struct(b=2,a=1), struct(a=1)!=struct(a=1))"#,
        Ok(r#"True False True False"#),
    ),
    (
        r#"print(hash(struct(a=1)) == hash(struct(a=1)))"#,
        Err(r#"in call to hash(), parameter 'value' got value of type 'struct', want 'string'"#),
    ),
    (
        r#"print(hash(struct(a=[1])))"#,
        Err(r#"in call to hash(), parameter 'value' got value of type 'struct', want 'string'"#),
    ),
    (
        r#"print(hash(struct()))"#,
        Err(r#"in call to hash(), parameter 'value' got value of type 'struct', want 'string'"#),
    ),
    (
        r#"s=struct(a=1)
s.a=2"#,
        Err(r#"struct value does not support field assignment"#),
    ),
    (
        r#"s=struct(a=1)
print(s.b)"#,
        Err(r#"'struct' value has no field or method 'b'
Available attributes: a"#),
    ),
    (
        r#"s=struct(a=1)
print(getattr(s,"b",None), hasattr(s,"a"), hasattr(s,"b"), getattr(s,"a"))"#,
        Ok(r#"None True False 1"#),
    ),
    (
        r#"s=struct(a=1)
print(getattr(s,"b"))"#,
        Err(r#"'struct' value has no field or method 'b'
Available attributes: a"#),
    ),
    (
        r#"print(struct(a=1)+struct(b=2))"#,
        Ok(r#"struct(a = 1, b = 2)"#),
    ),
    (
        r#"print(struct(a=1)<struct(a=2))"#,
        Err(r#"unsupported comparison: struct <=> struct"#),
    ),
    (
        r#"print(len(struct(a=1)))"#,
        Err(
            r#"in call to len(), parameter 'x' got value of type 'struct', want 'iterable or string'"#,
        ),
    ),
    (
        r#"print([x for x in struct(a=1)])"#,
        Err(r#"type 'struct' is not iterable"#),
    ),
    (
        r#"print(struct(a=1)["a"])"#,
        Err(r#"type 'struct' has no operator [](string)"#),
    ),
    (
        r#"print("a" in struct(a=1))"#,
        Err(r#"unsupported binary operation: string in struct"#),
    ),
    (
        r#"print(struct(1))"#,
        Err(r#"struct() got unexpected positional argument"#),
    ),
    (r#"print(struct(**{"a":1}))"#, Ok(r#"struct(a = 1)"#)),
    (r#"print(struct(**{"a b":1}))"#, Ok(r#"struct(a b = 1)"#)),
    (r#"print(struct(**{"1":1}))"#, Ok(r#"struct(1 = 1)"#)),
    (
        r#"print(struct(**{1:1}))"#,
        Err(r#"keywords must be strings, not int"#),
    ),
    (
        r#"print(struct(a=1,a=2))"#,
        Err(r#"duplicate keyword argument: a"#),
    ),
    (r#"print(struct(to_json=1))"#, Ok(r#"struct(to_json = 1)"#)),
    (
        r#"print(struct(a=1).to_json())"#,
        Err(r#"'struct' value has no field or method 'to_json'
Available attributes: a"#),
    ),
    (
        r#"print(struct(a=1).to_proto())"#,
        Err(r#"'struct' value has no field or method 'to_proto'
Available attributes: a"#),
    ),
    (
        r#"print(bool(struct()), bool(struct(a=1)))"#,
        Ok(r#"True True"#),
    ),
    (
        r#"print(str(struct(a=1)), repr(struct(a="x")))"#,
        Ok(r#"struct(a = 1) struct(a = "x")"#),
    ),
    (
        r#"print(struct(a=struct(b=None,c=True,d=1.5,e=(1,2),f=(1,),g=[],h={})))"#,
        Ok(
            r#"struct(a = struct(b = None, c = True, d = 1.5, e = (1, 2), f = (1,), g = [], h = {}))"#,
        ),
    ),
    (
        r#"print(struct(a=depset([1])))"#,
        Ok(r#"struct(a = depset([1]))"#),
    ),
    (
        r#"print(struct(a=1)==1, struct(a=1)=={"a":1})"#,
        Ok(r#"False False"#),
    ),
    (
        r#"print(struct(a=lambda: 1))"#,
        Ok(r#"struct(a = <function lambda from //:u.bzl>)"#),
    ),
    (
        r#"print(struct(a=print))"#,
        Ok(r#"struct(a = <built-in function print>)"#),
    ),
    (r#"print(struct(b=1,a=2))"#, Ok(r#"struct(a = 2, b = 1)"#)),
    (r#"print(struct(self=1))"#, Ok(r#"struct(self = 1)"#)),
    (r#"print(struct(_x=1))"#, Ok(r#"struct(_x = 1)"#)),
    (r#"print(struct(name=1).name)"#, Ok(r#"1"#)),
    (
        r#"print("%s" % struct(a=1), "%r" % struct(a=1))"#,
        Ok(r#"struct(a = 1) struct(a = 1)"#),
    ),
    (
        r#"print(hash(struct(a=depset([1]))))"#,
        Err(r#"in call to hash(), parameter 'value' got value of type 'struct', want 'string'"#),
    ),
    (r#"print({struct(a=1):1})"#, Ok(r#"{struct(a = 1): 1}"#)),
    (
        r#"print({struct(a=[]):1})"#,
        Err(r#"unhashable type: 'struct'"#),
    ),
    (
        r#"print(struct(a=1).a.b)"#,
        Err(r#"'int' value has no field or method 'b'"#),
    ),
    (r#"print(struct(a=struct(b=2)).a.b)"#, Ok(r#"2"#)),
    (
        r#"print(struct(a=1)+struct(a=2))"#,
        Err(r#"cannot add struct instances with common field 'a'"#),
    ),
    (
        r#"print(struct(a=1)+1)"#,
        Err(r#"unsupported binary operation: struct + int"#),
    ),
    (
        r#"print(1+struct(a=1))"#,
        Err(r#"unsupported binary operation: int + struct"#),
    ),
    (
        r#"print(struct(a=1)*2)"#,
        Err(r#"unsupported binary operation: struct * int"#),
    ),
    (
        r#"print(struct(a=1)|struct(b=1))"#,
        Err(r#"unsupported binary operation: struct | struct"#),
    ),
    (
        r#"print({struct(a=(1,[])):1})"#,
        Err(r#"unhashable type: 'struct'"#),
    ),
    (
        r#"print({struct(a=struct(b=[])):1})"#,
        Err(r#"unhashable type: 'struct'"#),
    ),
    (
        r#"print({struct(a=struct(b=1)):1})"#,
        Ok(r#"{struct(a = struct(b = 1)): 1}"#),
    ),
    (r#"print(struct(a=1) in [struct(a=1)])"#, Ok(r#"True"#)),
    (r#"print(struct(a=[1])==struct(a=[1]))"#, Ok(r#"True"#)),
    (
        r#"print(struct(a=struct(b=1))==struct(a=struct(b=1)))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(dir(struct(a=1)) == ["a"], type(dir(struct(a=1))))"#,
        Ok(r#"True list"#),
    ),
    (
        r#"print(struct(a=1).__class__)"#,
        Err(r#"'struct' value has no field or method '__class__'
Available attributes: a"#),
    ),
    (
        r#"print(struct.__name__)"#,
        Err(r#"'Provider' value has no field or method '__name__'"#),
    ),
    (r#"print(type(struct))"#, Ok(r#"Provider"#)),
    (r#"print(struct)"#, Ok(r#"<function struct>"#)),
    (r#"print(struct(a=1) == struct(a=1.0))"#, Ok(r#"True"#)),
    (r#"print(struct(a=None))"#, Ok(r#"struct(a = None)"#)),
    (
        r#"print(struct(a="it's \"q\"\n"))"#,
        Ok(r#"struct(a = "it's \"q\"\n")"#),
    ),
    (
        r#"print(struct(a=1,**{"b":2}))"#,
        Ok(r#"struct(a = 1, b = 2)"#),
    ),
    (
        r#"print(struct(**{"a":1},a=2))"#,
        Err(r#"keyword argument a may not follow **kwargs"#),
    ),
    (
        r#"print(struct(x=struct))"#,
        Ok(r#"struct(x = <function struct>)"#),
    ),
    (
        r#"print(struct(a=1) if struct(a=1) else 0)"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (r#"print(sorted([struct(a=1)]))"#, Ok(r#"[struct(a = 1)]"#)),
    (
        r#"print(str(struct(a=struct(b=1))))"#,
        Ok(r#"struct(a = struct(b = 1))"#),
    ),
    (
        r#"print(struct(a=1).a == 1, getattr(struct(a=1), "a"))"#,
        Ok(r#"True 1"#),
    ),
    (r#"print(json.encode(struct(a=1)))"#, Ok(r#"{"a":1}"#)),
    (
        r#"print(json)"#,
        Ok(r#"<unknown object net.starlark.java.lib.json.Json>"#),
    ),
    (r#"print(type(json))"#, Ok(r#"json"#)),
    (
        r#"print(dir(json))"#,
        Ok(r#"["decode", "encode", "encode_indent", "indent"]"#),
    ),
    (
        r#"print(proto)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.packages.Proto>"#),
    ),
    (r#"print(dir(proto))"#, Ok(r#"["encode_text"]"#)),
    (r#"print(type(proto))"#, Ok(r#"proto"#)),
    (
        r#"print(dir(set([1])))"#,
        Ok(
            r#"["add", "clear", "difference", "difference_update", "discard", "intersection", "intersection_update", "isdisjoint", "issubset", "issuperset", "pop", "remove", "symmetric_difference", "symmetric_difference_update", "union", "update"]"#,
        ),
    ),
    (r#"print(type(set([1])))"#, Ok(r#"set"#)),
    (
        r#"print(set([1,2]), set(), set([1,1,2]))"#,
        Ok(r#"set([1, 2]) set() set([1, 2])"#),
    ),
    (r#"print(set)"#, Ok(r#"<built-in function set>"#)),
];

const JSON_CASES: &[(&str, Result<&str, &str>)] = &[
    (
        r#"print(json.encode(1), json.encode("a"), json.encode(None), json.encode(True), json.encode(1.5), json.encode([1,"a"]), json.encode((1,2)), json.encode({"a":1}))"#,
        Ok(r#"1 "a" null true 1.5 [1,"a"] [1,2] {"a":1}"#),
    ),
    (
        r#"print(json.encode({}), json.encode([]), json.encode(struct()))"#,
        Ok(r#"{} [] {}"#),
    ),
    (
        r#"print(json.encode({"b":1,"a":2}))"#,
        Ok(r#"{"a":2,"b":1}"#),
    ),
    (
        r#"print(json.encode(struct(b=1,a=2)))"#,
        Ok(r#"{"a":2,"b":1}"#),
    ),
    (
        r#"print(json.encode("a\"b\\c\n\t\ré\u0001/"))"#,
        Err(r#"invalid escape sequence: \u. Use '\\' to insert '\'."#),
    ),
    (
        r#"print(json.encode("\U0001F600   \x7f"))"#,
        Err(r#"invalid escape sequence: \U. Use '\\' to insert '\'."#),
    ),
    (
        r#"print(json.encode(1.0), json.encode(1e20), json.encode(1e-7), json.encode(-0.0), json.encode(123456789012))"#,
        Ok(r#"1.0 1e+20 1e-07 -0.0 123456789012"#),
    ),
    (
        r#"print(json.encode(float("inf")))"#,
        Err(r#"cannot encode non-finite float +inf"#),
    ),
    (
        r#"print(json.encode(float("nan")))"#,
        Err(r#"cannot encode non-finite float nan"#),
    ),
    (
        r#"print(json.encode({1:2}))"#,
        Err(r#"dict has int key, want string"#),
    ),
    (
        r#"print(json.encode({"a":{1:2}}))"#,
        Err(r#"in dict key "a": dict has int key, want string"#),
    ),
    (
        r#"print(json.encode(depset([1])))"#,
        Err(r#"cannot encode depset as JSON"#),
    ),
    (
        r#"print(json.encode(print))"#,
        Err(r#"cannot encode builtin_function_or_method as JSON"#),
    ),
    (
        r#"print(json.encode(lambda: 1))"#,
        Err(r#"cannot encode function as JSON"#),
    ),
    (r#"print(json.encode(set([1])))"#, Ok(r#"[1]"#)),
    (
        r#"print(json.encode(1<<70))"#,
        Ok(r#"1180591620717411303424"#),
    ),
    (r#"print(json.encode(range(3)))"#, Ok(r#"[0,1,2]"#)),
    (
        r#"print(json.encode({"a":[{"b":struct(c=1)}]}))"#,
        Ok(r#"{"a":[{"b":{"c":1}}]}"#),
    ),
    (
        r#"print(json.encode(1, 2))"#,
        Err(r#"encode() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(json.encode())"#,
        Err(r#"encode() missing 1 required positional argument: x"#),
    ),
    (
        r#"print(json.encode(x=1))"#,
        Err(r#"encode() got named argument for positional-only parameter 'x'"#),
    ),
    (
        r#"print(json.encode(value=1))"#,
        Err(r#"encode() got unexpected keyword argument 'value'"#),
    ),
    (
        r#"print(repr(json.encode_indent({"a":[1,2],"b":{"c":1},"e":[],"f":{}})))"#,
        Ok(
            r#""{\n\t\"a\": [\n\t\t1,\n\t\t2\n\t],\n\t\"b\": {\n\t\t\"c\": 1\n\t},\n\t\"e\": [],\n\t\"f\": {}\n}""#,
        ),
    ),
    (
        r#"print(repr(json.encode_indent([1,[2,[3]]], prefix=">", indent="  ")))"#,
        Ok(r#""[\n>  1,\n>  [\n>    2,\n>    [\n>      3\n>    ]\n>  ]\n>]""#),
    ),
    (
        r#"print(repr(json.encode_indent({"a":1}, indent="\t")))"#,
        Ok(r#""{\n\t\"a\": 1\n}""#),
    ),
    (
        r#"print(repr(json.encode_indent({"a":1}, prefix="", indent="")))"#,
        Ok(r#""{\n\"a\": 1\n}""#),
    ),
    (
        r#"print(repr(json.encode_indent(1)))"#,
        Err(r#"input is not valid JSON"#),
    ),
    (r#"print(repr(json.encode_indent([])))"#, Ok(r#""[]""#)),
    (
        r#"print(repr(json.encode_indent(struct(a=1,b=[1]))))"#,
        Ok(r#""{\n\t\"a\": 1,\n\t\"b\": [\n\t\t1\n\t]\n}""#),
    ),
    (
        r#"print(repr(json.encode_indent({"a":1}, indent=1)))"#,
        Err(
            r#"in call to encode_indent(), parameter 'indent' got value of type 'int', want 'string'"#,
        ),
    ),
    (
        r#"print(repr(json.encode_indent({"a":1}, "x")))"#,
        Err(r#"encode_indent() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(repr(json.encode_indent({"a":1}, "x", "y")))"#,
        Err(r#"encode_indent() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"print(repr(json.indent('{"a":[1,2],"b":{}}')))"#,
        Ok(r#""{\n\t\"a\": [\n\t\t1,\n\t\t2\n\t],\n\t\"b\": {}\n}""#),
    ),
    (
        r##"print(repr(json.indent('{"a":[1,2],"b":{}}', prefix="#", indent="    ")))"##,
        Ok(r#""{\n#    \"a\": [\n#        1,\n#        2\n#    ],\n#    \"b\": {}\n#}""#),
    ),
    (
        r#"print(repr(json.indent('  [ 1 , 2 ]  ')))"#,
        Ok(r#""[\n\t1,\n\t2\n]""#),
    ),
    (
        r#"print(repr(json.indent('[1,')))"#,
        Err(r#"unexpected end of file"#),
    ),
    (
        r#"print(repr(json.indent('')))"#,
        Err(r#"unexpected end of file"#),
    ),
    (r#"print(repr(json.indent('nope')))"#, Ok(r#""nope""#)),
    (
        r#"print(repr(json.indent(1)))"#,
        Err(r#"in call to indent(), parameter 's' got value of type 'int', want 'string'"#),
    ),
    (
        r#"print(repr(json.indent('{"a":1} x')))"#,
        Ok(r#""{\n\t\"a\": 1\n}""#),
    ),
    (
        r#"print(repr(json.indent('[]'), json.indent('{}'), json.indent('[[]]')))"#,
        Err(r#"repr() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"print(repr(json.indent('{"a" : "b\\n"}')))"#,
        Ok(r#""{\n\t\"a\": \"b\\n\"\n}""#),
    ),
    (
        r#"print(json.decode('{"a":[1,2.5,"x",null,true,false,{"b":{}}]}'))"#,
        Ok(r#"{"a": [1, 2.5, "x", None, True, False, {"b": {}}]}"#),
    ),
    (
        r#"print(json.decode('1'), json.decode('"a"'), json.decode('null'), json.decode('true'), json.decode('[]'), json.decode('{}'))"#,
        Ok(r#"1 a None True [] {}"#),
    ),
    (
        r#"print(json.decode('1.0'), json.decode('1e2'), json.decode('-0'), json.decode('12345678901234567890'), json.decode('1E+2'))"#,
        Ok(r#"1.0 100.0 0 12345678901234567890 100.0"#),
    ),
    (
        r#"print(type(json.decode('1')), type(json.decode('1.0')), type(json.decode('1e2')), type(json.decode('{}')), type(json.decode('[]')))"#,
        Ok(r#"int float float dict list"#),
    ),
    (
        r#"print(json.decode(''))"#,
        Err(r#"at offset 0, unexpected end of file"#),
    ),
    (
        r#"print(json.decode('{'))"#,
        Err(r#"at offset 1, unexpected end of file"#),
    ),
    (
        r#"print(json.decode('nul'))"#,
        Err(r#"at offset 0, unexpected character "n""#),
    ),
    (
        r#"print(json.decode('[1,]'))"#,
        Err(r#"at offset 3, unexpected character "]""#),
    ),
    (
        r#"print(json.decode('{"a":1,}'))"#,
        Err(r#"at offset 7, unexpected character "}""#),
    ),
    (r#"print(json.decode('{"a":1,"a":2}'))"#, Ok(r#"{"a": 2}"#)),
    (
        r#"print(json.decode("{'a':1}"))"#,
        Err(r#"at offset 1, unexpected character "'""#),
    ),
    (
        r#"print(json.decode('[1] x'))"#,
        Err(r#"at offset 4, unexpected character "x" after value"#),
    ),
    (r#"print(json.decode('"\\u00e9\\n\\/"'))"#, Ok(r#"é"#)),
    (
        r#"print(json.decode('"\\x"'))"#,
        Err(r#"at offset 3, invalid escape '\x'"#),
    ),
    (
        r#"print(json.decode('"a\nb"'))"#,
        Err(r#"at offset 2, invalid character '\x0a' in string literal"#),
    ),
    (
        r#"print(json.decode('01'))"#,
        Err(r#"at offset 0, invalid number: 01"#),
    ),
    (
        r#"print(json.decode('+1'))"#,
        Err(r#"at offset 0, unexpected character "+""#),
    ),
    (
        r#"print(json.decode('1.'))"#,
        Err(r#"at offset 0, invalid number: 1."#),
    ),
    (
        r#"print(json.decode('.5'))"#,
        Err(r#"at offset 0, unexpected character ".""#),
    ),
    (
        r#"print(json.decode('NaN'))"#,
        Err(r#"at offset 0, unexpected character "N""#),
    ),
    (
        r#"print(json.decode('Infinity'))"#,
        Err(r#"at offset 0, unexpected character "I""#),
    ),
    (r#"print(json.decode(' [ 1 ] '))"#, Ok(r#"[1]"#)),
    (r#"print(json.decode('{"a":1}', None))"#, Ok(r#"{"a": 1}"#)),
    (
        r#"print(json.decode('{"a":1}', default=5))"#,
        Ok(r#"{"a": 1}"#),
    ),
    (r#"print(json.decode('x', default=5))"#, Ok(r#"5"#)),
    (r#"print(json.decode('x', 5))"#, Ok(r#"5"#)),
    (
        r#"print(json.decode(1))"#,
        Err(r#"in call to decode(), parameter 'x' got value of type 'int', want 'string'"#),
    ),
    (
        r#"print(json.decode('{1:2}'))"#,
        Err(r#"at offset 2, got int for object key, want string"#),
    ),
    (r#"print(json.decode('"\\ud83d\\ude00"'))"#, Ok(r#"😀"#)),
    (r#"print(json.decode('"\\ud83d"'))"#, Ok(r#"�"#)),
    (
        r#"print(json.decode('{"a":1}').a)"#,
        Err(r#"'dict' value has no field or method 'a'"#),
    ),
    (r#"print(json.decode('[1e400]'))"#, Ok(r#"[+inf]"#)),
    (
        r#"print(json.decode('[99999999999999999999999999]'))"#,
        Ok(r#"[99999999999999999999999999]"#),
    ),
    (
        r#"print(json.decode('-'))"#,
        Err(r#"at offset 0, invalid number: -"#),
    ),
    (
        r#"print(json.decode('tru'))"#,
        Err(r#"at offset 0, unexpected character "t""#),
    ),
    (
        r#"print(json.decode('[1 2]'))"#,
        Err(r#"at offset 3, got "2", want ',' or ']'"#),
    ),
    (
        r#"print(json.decode('{"a" 1}'))"#,
        Err(r#"at offset 5, after object key, got "1", want ':' "#),
    ),
    (
        r#"print(json.decode('{"a":}'))"#,
        Err(r#"at offset 5, unexpected character "}""#),
    ),
    (
        r#"print(json.decode('[' * 5 + ']' * 5))"#,
        Ok(r#"[[[[[]]]]]"#),
    ),
    (
        r#"print(json.encode("a\"b\\c\n\t\r\001\002\037\177/ é 😀 \033"))"#,
        Ok(r#""a\"b\\c\n\t\r\u0001\u0002\u001f/ é 😀 \u001b""#),
    ),
    (
        r#"print(json.encode("\b\f\v\a"))"#,
        Ok(r#""\b\f\u000b\u0007""#),
    ),
    (r#"print(json.encode("<>&'"))"#, Ok(r#""<>&'""#)),
    (r#"print(json.encode("\0"))"#, Ok(r#""\u0000""#)),
    (
        r#"print(json.decode('"\\b\\f\\r\\t\\"\\\\"') == "\b\f\r\t\"\\")"#,
        Ok(r#"True"#),
    ),
    (r#"print(json.decode('"\\u0000"') == "\0")"#, Ok(r#"True"#)),
    (
        r#"print(json.decode('"\\u12"'))"#,
        Err(r#"at offset 3, incomplete \uXXXX escape"#),
    ),
    (
        r#"print(json.decode('"\\uzzzz"'))"#,
        Err(r#"at offset 3, invalid hex char "z" in \uXXXX escape"#),
    ),
    (
        r#"print(json.decode('"abc'))"#,
        Err(r#"at offset 4, unclosed string literal"#),
    ),
    (
        r#"print(json.decode('"\\'))"#,
        Err(r#"at offset 2, incomplete escape"#),
    ),
    (
        r#"print(json.decode('"a\tb"'))"#,
        Err(r#"at offset 2, invalid character '\x09' in string literal"#),
    ),
    (r#"print(json.decode('"é😀"'))"#, Ok(r#"é😀"#)),
    (
        r#"print(json.encode(struct(a=struct())))"#,
        Ok(r#"{"a":{}}"#),
    ),
    (r#"print(json.encode({"a":None}))"#, Ok(r#"{"a":null}"#)),
    (
        r#"print(json.encode(struct(a=print)))"#,
        Err(r#"in struct field .a: cannot encode builtin_function_or_method as JSON"#),
    ),
    (
        r#"print(json.encode([print]))"#,
        Err(r#"at list index 0: cannot encode builtin_function_or_method as JSON"#),
    ),
    (
        r#"print(json.encode({"a":[depset([1])]}))"#,
        Err(r#"in dict key "a": at list index 0: cannot encode depset as JSON"#),
    ),
    (
        r#"print(json.encode([1,[2,{3:4}]]))"#,
        Err(r#"at list index 1: at list index 1: dict has int key, want string"#),
    ),
    (
        r#"print(json.encode(struct(a=1,b=[depset()])))"#,
        Err(r#"in struct field .b: at list index 0: cannot encode depset as JSON"#),
    ),
    (
        r#"print(json.encode(json))"#,
        Err(r#"cannot encode json as JSON"#),
    ),
    (
        r#"print(json.encode(struct))"#,
        Err(r#"cannot encode Provider as JSON"#),
    ),
    (
        r#"print(json.encode(provider()))"#,
        Err(r#"cannot encode Provider as JSON"#),
    ),
    (r#"print(json.decode('{"a":1}') == {"a":1})"#, Ok(r#"True"#)),
    (
        r#"print(json.decode('{"b":1,"a":2}').keys())"#,
        Ok(r#"["b", "a"]"#),
    ),
    (r#"print(json.decode('[1,2]')[0])"#, Ok(r#"1"#)),
    (
        r#"print(json.decode('  '))"#,
        Err(r#"at offset 2, unexpected end of file"#),
    ),
    (r#"print(json.decode('\n[\n1\n]\n'))"#, Ok(r#"[1]"#)),
    (r#"print(json.decode('[1,2', default=[]))"#, Ok(r#"[]"#)),
    (r#"print(json.decode('[', default=7))"#, Ok(r#"7"#)),
    (r#"print(json.decode('', default=None))"#, Ok(r#"None"#)),
    (
        r#"print(json.decode(x='[]'))"#,
        Err(r#"decode() got named argument for positional-only parameter 'x'"#),
    ),
    (
        r#"print(json.decode('[]', default=1, x=2))"#,
        Err(r#"decode() got named argument for positional-only parameter 'x'"#),
    ),
    (
        r#"print(json.decode())"#,
        Err(r#"decode() missing 1 required positional argument: x"#),
    ),
    (
        r#"print(json.decode('1','2','3'))"#,
        Err(r#"decode() accepts no more than 2 positional arguments but got 3"#),
    ),
    (
        r#"print(repr(json.indent('{"a":1}', 1)))"#,
        Err(r#"indent() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(repr(json.indent('[1]', prefix=1)))"#,
        Err(r#"in call to indent(), parameter 'prefix' got value of type 'int', want 'string'"#),
    ),
    (
        r#"print(repr(json.indent('[1]', indent=None)))"#,
        Err(
            r#"in call to indent(), parameter 'indent' got value of type 'NoneType', want 'string'"#,
        ),
    ),
    (
        r#"print(repr(json.indent(s='[1]')))"#,
        Err(r#"indent() got named argument for positional-only parameter 's'"#),
    ),
    (
        r#"print(repr(json.indent('{"a":1,"b":[]}', indent="")))"#,
        Ok(r#""{\n\"a\": 1,\n\"b\": []\n}""#),
    ),
    (
        r#"print(repr(json.indent('{"a":1', indent=" ")))"#,
        Err(r#"input is not valid JSON"#),
    ),
    (
        r#"print(repr(json.indent('{"a":1}}', indent=" ")))"#,
        Ok(r#""{\n \"a\": 1\n}""#),
    ),
    (
        r#"print(repr(json.indent('[1,2}', indent=" ")))"#,
        Ok(r#""[\n 1,\n 2\n}""#),
    ),
    (r#"print(repr(json.indent('"a"')))"#, Ok(r#""\"a\"""#)),
    (
        r#"print(repr(json.indent('1')))"#,
        Err(r#"input is not valid JSON"#),
    ),
    (
        r#"print(repr(json.indent('{"a"}')))"#,
        Ok(r#""{\n\t\"a\"\n}""#),
    ),
    (
        r#"print(repr(json.indent('{')))"#,
        Err(r#"unexpected end of file"#),
    ),
    (
        r#"print(repr(json.indent('[1 2]')))"#,
        Ok(r#""[\n\t12\n]""#),
    ),
    (
        r#"print(repr(json.indent('  ')))"#,
        Err(r#"unexpected end of file"#),
    ),
    (
        r#"print(repr(json.indent('{"a":[]} {"b":1}')))"#,
        Ok(r#""{\n\t\"a\": []\n}""#),
    ),
    (r#"print(repr(json.indent('{}x')))"#, Ok(r#""{}""#)),
    (
        r#"print(repr(json.indent('[,]')))"#,
        Ok(r#""[\n\t,\n\t\n]""#),
    ),
    (
        r#"print(repr(json.indent('nope', "x")))"#,
        Err(r#"indent() accepts no more than 1 positional argument but got 2"#),
    ),
    (r#"print(repr(json.indent('true false')))"#, Ok(r#""true""#)),
    (
        r#"print(repr(json.indent('{"a":1e5,"b":-0.5E-3}')))"#,
        Ok(r#""{\n\t\"a\": 1e5,\n\t\"b\": -0.5E-3\n}""#),
    ),
    (
        r#"print(repr(json.indent('{"a":\n1}')))"#,
        Ok(r#""{\n\t\"a\": 1\n}""#),
    ),
    (
        r#"print(repr(json.indent('[1,\n\n2]')))"#,
        Ok(r#""[\n\t1,\n\t2\n]""#),
    ),
    (
        r#"print(repr(json.indent('"a\\"')))"#,
        Err(r#"input is not valid JSON"#),
    ),
    (
        r#"print(repr(json.encode_indent({"a":1}, prefix="P")))"#,
        Ok(r#""{\nP\t\"a\": 1\nP}""#),
    ),
    (
        r#"print(repr(json.encode_indent([1], prefix="P", indent="I")))"#,
        Ok(r#""[\nPI1\nP]""#),
    ),
    (
        r#"print(repr(json.encode_indent(depset())))"#,
        Err(r#"cannot encode depset as JSON"#),
    ),
    (
        r#"print(repr(json.encode_indent(float("inf"))))"#,
        Err(r#"cannot encode non-finite float +inf"#),
    ),
    (
        r#"print(repr(json.encode_indent({1:2})))"#,
        Err(r#"dict has int key, want string"#),
    ),
    (
        r#"print(repr(json.encode_indent(x=1)))"#,
        Err(r#"encode_indent() got named argument for positional-only parameter 'x'"#),
    ),
    (
        r#"print(repr(json.encode_indent()))"#,
        Err(r#"encode_indent() missing 1 required positional argument: x"#),
    ),
    (
        r#"print(repr(json.encode_indent(1, prefix=1)))"#,
        Err(
            r#"in call to encode_indent(), parameter 'prefix' got value of type 'int', want 'string'"#,
        ),
    ),
    (
        r#"print(repr(json.encode_indent([[]])))"#,
        Ok(r#""[\n\t[]\n]""#),
    ),
    (
        r#"print(repr(json.encode_indent({"a":{}})))"#,
        Ok(r#""{\n\t\"a\": {}\n}""#),
    ),
    (r#"print(repr(json.encode_indent("a")))"#, Ok(r#""\"a\"""#)),
    (r#"print(repr(json.encode_indent(None)))"#, Ok(r#""null""#)),
    (
        r#"print(repr(json.encode_indent(1.5)))"#,
        Err(r#"input is not valid JSON"#),
    ),
    (r#"print(repr(json.indent('1 ')))"#, Ok(r#""1""#)),
    (r#"print(repr(json.indent('1,')))"#, Ok(r#""1""#)),
    (
        r#"print(repr(json.indent('-')))"#,
        Err(r#"input is not valid JSON"#),
    ),
    (
        r#"print(repr(json.indent('-1')))"#,
        Err(r#"input is not valid JSON"#),
    ),
    (r#"print(repr(json.indent('1]')))"#, Ok(r#""1""#)),
    (
        r#"print(repr(json.indent('[1')))"#,
        Err(r#"input is not valid JSON"#),
    ),
    (r#"print(repr(json.indent('[1]')))"#, Ok(r#""[\n\t1\n]""#)),
    (
        r#"print(repr(json.indent('"a')))"#,
        Err(r#"input is not valid JSON"#),
    ),
    (r#"print(repr(json.indent('"a"x')))"#, Ok(r#""\"a\"""#)),
    (
        r#"print(repr(json.indent('"')))"#,
        Err(r#"input is not valid JSON"#),
    ),
    (r#"print(repr(json.indent('""')))"#, Ok(r#""\"\"""#)),
    (
        r#"print(repr(json.indent('[""]')))"#,
        Ok(r#""[\n\t\"\"\n]""#),
    ),
    (
        r#"print(repr(json.indent('["a\\"]')))"#,
        Err(r#"input is not valid JSON"#),
    ),
    (
        r#"print(repr(json.indent('["a\\')))"#,
        Err(r#"input is not valid JSON"#),
    ),
    (r#"print(repr(json.indent('true')))"#, Ok(r#""true""#)),
    (r#"print(repr(json.indent('trux')))"#, Ok(r#""trux""#)),
    (r#"print(repr(json.indent('false')))"#, Ok(r#""false""#)),
    (r#"print(repr(json.indent('null')))"#, Ok(r#""null""#)),
    (r#"print(repr(json.indent('nulx')))"#, Ok(r#""nulx""#)),
    (
        r#"print(repr(json.indent('[nope]')))"#,
        Ok(r#""[\n\tnope\n]""#),
    ),
    (r#"print(repr(json.indent('}')))"#, Ok(r#""\n}""#)),
    (r#"print(repr(json.indent(']')))"#, Ok(r#""\n]""#)),
    (r#"print(repr(json.indent('{]')))"#, Ok(r#""{]""#)),
    (r#"print(repr(json.indent('[}')))"#, Ok(r#""[}""#)),
    (r#"print(repr(json.indent(']]')))"#, Ok(r#""\n]""#)),
    (r#"print(repr(json.indent('[]]')))"#, Ok(r#""[]""#)),
    (r#"print(repr(json.indent(':')))"#, Ok(r#"": ""#)),
    (r#"print(repr(json.indent(',')))"#, Ok(r#"",\n""#)),
    (
        r#"print(repr(json.indent('{"a":}')))"#,
        Ok(r#""{\n\t\"a\": \n}""#),
    ),
    (
        r#"print(repr(json.indent('{,}')))"#,
        Ok(r#""{\n\t,\n\t\n}""#),
    ),
    (
        r#"print(repr(json.indent('{"a": [ ]}')))"#,
        Ok(r#""{\n\t\"a\": []\n}""#),
    ),
    (r#"print(repr(json.indent('{ }')))"#, Ok(r#""{}""#)),
    (r#"print(repr(json.indent('[ ]')))"#, Ok(r#""[]""#)),
    (r#"print(repr(json.indent('[\n]')))"#, Ok(r#""[]""#)),
    (
        r#"print(repr(json.indent('[[ ], [ ]]')))"#,
        Ok(r#""[\n\t[],\n\t[]\n]""#),
    ),
    (
        r#"print(repr(json.indent('{"a":[{}]}')))"#,
        Ok(r#""{\n\t\"a\": [\n\t\t{}\n\t]\n}""#),
    ),
    (
        r#"print(repr(json.indent('x')))"#,
        Err(r#"unexpected character "x""#),
    ),
    (
        r#"print(repr(json.indent('@')))"#,
        Err(r#"unexpected character "@""#),
    ),
    (r#"print(repr(json.indent('  {}')))"#, Ok(r#""{}""#)),
    (r#"print(repr(json.indent('{}  ')))"#, Ok(r#""{}""#)),
    (
        r#"print(repr(json.indent('[,,]')))"#,
        Ok(r#""[\n\t,\n\t,\n\t\n]""#),
    ),
    (r#"print(repr(json.indent('[:]')))"#, Ok(r#""[\n\t: \n]""#)),
    (r#"print(repr(json.indent('{:}')))"#, Ok(r#""{\n\t: \n}""#)),
    (
        r#"print(repr(json.indent('{"a":1:2}')))"#,
        Ok(r#""{\n\t\"a\": 1: 2\n}""#),
    ),
    (
        r#"print(repr(json.indent('{"a"::1}')))"#,
        Ok(r#""{\n\t\"a\": : 1\n}""#),
    ),
    (
        r#"print(repr(json.indent('{"a":1,"b":[true,false,null]}')))"#,
        Ok(r#""{\n\t\"a\": 1,\n\t\"b\": [\n\t\ttrue,\n\t\tfalse,\n\t\tnull\n\t]\n}""#),
    ),
    (
        r#"print(repr(json.indent('[1e5]')))"#,
        Ok(r#""[\n\t1e5\n]""#),
    ),
    (r#"print(repr(json.indent('[-]')))"#, Ok(r#""[\n\t-\n]""#)),
    (
        r#"print(repr(json.indent('[1.5.5]')))"#,
        Ok(r#""[\n\t1.5.5\n]""#),
    ),
    (
        r#"print(repr(json.indent('[+1]')))"#,
        Err(r#"unexpected character "+""#),
    ),
    (
        r#"print(repr(json.indent('[.]')))"#,
        Err(r#"unexpected character ".""#),
    ),
    (
        r#"print(repr(json.indent('[1a]')))"#,
        Err(r#"unexpected character "a""#),
    ),
    (
        r#"print(repr(json.indent('[a]')))"#,
        Err(r#"unexpected character "a""#),
    ),
    (
        r#"print(repr(json.indent('[t,f]')))"#,
        Err(r#"unexpected end of file"#),
    ),
    (
        r#"print(repr(json.indent('["a", "b"]')))"#,
        Ok(r#""[\n\t\"a\",\n\t\"b\"\n]""#),
    ),
    (
        r#"print(repr(json.indent('[ "a" , "b" ]')))"#,
        Ok(r#""[\n\t\"a\",\n\t\"b\"\n]""#),
    ),
    (
        r#"print(repr(json.indent('[\t1\t]')))"#,
        Ok(r#""[\n\t1\n]""#),
    ),
    (
        r#"print(repr(json.indent('[\r\n1]')))"#,
        Ok(r#""[\n\t1\n]""#),
    ),
    (r#"print(repr(json.indent('[{}]')))"#, Ok(r#""[\n\t{}\n]""#)),
    (
        r#"print(repr(json.indent('[{},{}]')))"#,
        Ok(r#""[\n\t{},\n\t{}\n]""#),
    ),
    (r#"print(repr(json.indent('[[]]')))"#, Ok(r#""[\n\t[]\n]""#)),
    (
        r#"print(repr(json.indent('{"a":{}}')))"#,
        Ok(r#""{\n\t\"a\": {}\n}""#),
    ),
    (
        r#"print(repr(json.indent('{"a":{"b":{}}}')))"#,
        Ok(r#""{\n\t\"a\": {\n\t\t\"b\": {}\n\t}\n}""#),
    ),
    (
        r#"print(repr(json.indent('[1] [2]')))"#,
        Ok(r#""[\n\t1\n]""#),
    ),
    (r#"print(repr(json.indent('"a"  ')))"#, Ok(r#""\"a\"""#)),
    (r#"print(repr(json.indent('  "a"')))"#, Ok(r#""\"a\"""#)),
    (
        r#"print(repr(json.indent('[1,]')))"#,
        Ok(r#""[\n\t1,\n\t\n]""#),
    ),
    (
        r#"print(repr(json.indent('[,1]')))"#,
        Ok(r#""[\n\t,\n\t1\n]""#),
    ),
    (
        r#"print(repr(json.indent('{"a":1,}')))"#,
        Ok(r#""{\n\t\"a\": 1,\n\t\n}""#),
    ),
    (r#"print(repr(json.indent(']1')))"#, Ok(r#""\n]""#)),
    (r#"print(repr(json.indent('}1')))"#, Ok(r#""\n}""#)),
    (r#"print(repr(json.indent('[]1')))"#, Ok(r#""[]""#)),
    (r#"print(repr(json.indent('-1 ')))"#, Ok(r#""-1""#)),
    (r#"print(repr(json.indent('1.5 ')))"#, Ok(r#""1.5""#)),
    (
        r#"print(repr(json.indent('0')))"#,
        Err(r#"input is not valid JSON"#),
    ),
    (r#"print(repr(json.indent('[0]')))"#, Ok(r#""[\n\t0\n]""#)),
    (r#"print(repr(json.indent('[-0]')))"#, Ok(r#""[\n\t-0\n]""#)),
];

const PROTO_CASES: &[(&str, Result<&str, &str>)] = &[
    (r#"print(proto.encode_text(struct(a=1)))"#, Ok(r#"a: 1"#)),
    (
        r#"print(repr(proto.encode_text(struct(a=1))))"#,
        Ok(r#""a: 1\n""#),
    ),
    (r#"print(repr(proto.encode_text(struct())))"#, Ok(r#""""#)),
    (
        r#"print(repr(proto.encode_text(struct(a=1,b="x",c=True,d=1.5))))"#,
        Ok(r#""a: 1\nb: \"x\"\nc: true\nd: 1.5\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(b=1,a=2))))"#,
        Ok(r#""a: 2\nb: 1\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=[1,2]))))"#,
        Ok(r#""a: 1\na: 2\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=[]))))"#,
        Ok(r#""""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=struct(b=1)))))"#,
        Ok(r#""a {\n  b: 1\n}\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=struct()))))"#,
        Ok(r#""a {\n}\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=[struct(b=1),struct(b=2)]))))"#,
        Ok(r#""a {\n  b: 1\n}\na {\n  b: 2\n}\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=None))))"#,
        Ok(r#""""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=(1,2)))))"#,
        Ok(r#""a: 1\na: 2\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a="q\"\n\\é\001"))))"#,
        Ok(r#""a: \"q\\\"\\n\\\\é\x01\"\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a={"k":1}))))"#,
        Ok(r#""a {\n  key: \"k\"\n  value: 1\n}\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=depset([1])))))"#,
        Err(r#"in struct field .a: got depset, want string, int, float, bool, or struct"#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=print))))"#,
        Err(
            r#"in struct field .a: got builtin_function_or_method, want string, int, float, bool, or struct"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text({"a":1})))"#,
        Err(
            r#"in call to encode_text(), parameter 'x' got value of type 'dict', want 'structure or NativeInfo'"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text(1)))"#,
        Err(
            r#"in call to encode_text(), parameter 'x' got value of type 'int', want 'structure or NativeInfo'"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text("a")))"#,
        Err(
            r#"in call to encode_text(), parameter 'x' got value of type 'string', want 'structure or NativeInfo'"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text(None)))"#,
        Err(
            r#"in call to encode_text(), parameter 'x' got value of type 'NoneType', want 'structure or NativeInfo'"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text([1])))"#,
        Err(
            r#"in call to encode_text(), parameter 'x' got value of type 'list', want 'structure or NativeInfo'"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=[[1]]))))"#,
        Err(
            r#"in struct field .a: at list index 0: got list, want string, int, float, bool, or struct"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=[1,"x"]))))"#,
        Ok(r#""a: 1\na: \"x\"\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=[struct(b=1),1]))))"#,
        Ok(r#""a {\n  b: 1\n}\na: 1\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=1.0,b=1e20,c=-1,d=1<<70))))"#,
        Ok(r#""a: 1.0\nb: 1e+20\nc: -1\nd: 1180591620717411303424\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=float("inf")))))"#,
        Ok(r#""a: inf\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=float("nan")))))"#,
        Ok(r#""a: nan\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=False))))"#,
        Ok(r#""a: false\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=struct(b=struct(c=[1,2]))))))"#,
        Ok(r#""a {\n  b {\n    c: 1\n    c: 2\n  }\n}\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=[struct(b=[1,2])]))))"#,
        Ok(r#""a {\n  b: 1\n  b: 2\n}\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=[[]]))))"#,
        Err(
            r#"in struct field .a: at list index 0: got list, want string, int, float, bool, or struct"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=[struct()]))))"#,
        Ok(r#""a {\n}\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=[None]))))"#,
        Err(
            r#"in struct field .a: at list index 0: got NoneType, want string, int, float, bool, or struct"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text()))"#,
        Err(r#"encode_text() missing 1 required positional argument: x"#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=1), 2)))"#,
        Err(r#"encode_text() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(repr(proto.encode_text(x=struct(a=1))))"#,
        Err(r#"encode_text() got named argument for positional-only parameter 'x'"#),
    ),
    (
        r#"print(repr(proto.encode_text(x=struct(a=1)) ))"#,
        Err(r#"encode_text() got named argument for positional-only parameter 'x'"#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(**{"a b":1}))))"#,
        Ok(r#""a b: 1\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(**{"1":1}))))"#,
        Ok(r#""1: 1\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(**{"":1}))))"#,
        Ok(r#"": 1\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=1<<40))))"#,
        Ok(r#""a: 1099511627776\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a="x"))))"#,
        Ok(r#""a: \"x\"\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=1,b=struct(c=2),d=[3,4]))))"#,
        Ok(r#""a: 1\nb {\n  c: 2\n}\nd: 3\nd: 4\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=set([1])))))"#,
        Err(r#"in struct field .a: got set, want string, int, float, bool, or struct"#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=range(2)))))"#,
        Ok(r#""a: 0\na: 1\n""#),
    ),
    (
        r#"print(proto.encode_text == proto.encode_text)"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(type(proto.encode_text))"#,
        Ok(r#"builtin_function_or_method"#),
    ),
    (
        r#"print(proto.encode_text)"#,
        Ok(r#"<built-in method encode_text of proto value>"#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a="\t\r'\0\177\033\b\a\f\v<>&é😀"))))"#,
        Ok(r#""a: \"\t\r'\x00\x1b\x08\x07\x0c\x0b<>&é😀\"\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=""))))"#,
        Ok(r#""a: \"\"\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a={}))))"#,
        Ok(r#""""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a={"k":struct(z=1)}))))"#,
        Ok(r#""a {\n  key: \"k\"\n  value {\n    z: 1\n  }\n}\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a={1:2}))))"#,
        Ok(r#""a {\n  key: 1\n  value: 2\n}\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a={"k":[1,2]}))))"#,
        Err(
            r#"in struct field .a: in value for dict key "k": got list, want string, int, float, bool, or struct"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a={"k":{"j":1}}))))"#,
        Err(
            r#"in struct field .a: in value for dict key "k": got dict, want string, int, float, bool, or struct"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a={"k":None}))))"#,
        Err(
            r#"in struct field .a: in value for dict key "k": got NoneType, want string, int, float, bool, or struct"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a={"b":1,"a":2}))))"#,
        Ok(r#""a {\n  key: \"b\"\n  value: 1\n}\na {\n  key: \"a\"\n  value: 2\n}\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=[{"k":1}]))))"#,
        Err(
            r#"in struct field .a: at list index 0: got dict, want string, int, float, bool, or struct"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=1.5,b=0.1,c=1e-7,d=123456789.123,e=-0.0))))"#,
        Ok(r#""a: 1.5\nb: 0.1\nc: 1e-07\nd: 123456789.123\ne: -0.0\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=1e15,b=1e16,c=1e100,d=100.0,e=3.14159265358979))))"#,
        Ok(
            r#""a: 1000000000000000.0\nb: 10000000000000000.0\nc: 1e+100\nd: 100.0\ne: 3.14159265358979\n""#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=float("-inf")))))"#,
        Ok(r#""a: -inf\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a={"k":depset()}))))"#,
        Err(
            r#"in struct field .a: in value for dict key "k": got depset, want string, int, float, bool, or struct"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=[struct(b=[[1]])]))))"#,
        Err(
            r#"in struct field .a: at list index 0: in struct field .b: at list index 0: got list, want string, int, float, bool, or struct"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=struct(b=None)))))"#,
        Ok(r#""a {\n}\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=[struct(b=None)]))))"#,
        Ok(r#""a {\n}\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=struct(b=print)))))"#,
        Err(
            r#"in struct field .a: in struct field .b: got builtin_function_or_method, want string, int, float, bool, or struct"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=[struct(b=depset())]))))"#,
        Err(
            r#"in struct field .a: at list index 0: in struct field .b: got depset, want string, int, float, bool, or struct"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=struct(b=[depset()])))))"#,
        Err(
            r#"in struct field .a: in struct field .b: at list index 0: got depset, want string, int, float, bool, or struct"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a={"k":print}))))"#,
        Err(
            r#"in struct field .a: in value for dict key "k": got builtin_function_or_method, want string, int, float, bool, or struct"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a={"a":{}}))))"#,
        Err(
            r#"in struct field .a: in value for dict key "a": got dict, want string, int, float, bool, or struct"#,
        ),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a={1.5:2}))))"#,
        Err(r#"in struct field .a: invalid dict key: got float, want int or string"#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a={True:2}))))"#,
        Err(r#"in struct field .a: invalid dict key: got bool, want int or string"#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a={None:2}))))"#,
        Err(r#"in struct field .a: invalid dict key: got NoneType, want int or string"#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a={(1,2):2}))))"#,
        Err(r#"in struct field .a: invalid dict key: got tuple, want int or string"#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a={"k":"v"},b={"k":"v"}))))"#,
        Ok(r#""a {\n  key: \"k\"\n  value: \"v\"\n}\nb {\n  key: \"k\"\n  value: \"v\"\n}\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=1<<63,b=-(1<<63),c=1<<64))))"#,
        Ok(r#""a: 9223372036854775808\nb: -9223372036854775808\nc: 18446744073709551616\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(a=struct(b=struct())))))"#,
        Ok(r#""a {\n  b {\n  }\n}\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(**{"a.b":1}))))"#,
        Ok(r#""a.b: 1\n""#),
    ),
    (
        r#"print(repr(proto.encode_text(struct(**{"é":1}))))"#,
        Ok(r#""é: 1\n""#),
    ),
];

const SET_CASES: &[(&str, Result<&str, &str>)] = &[
    (
        r#"print(set(), set([]), set([1,2,3]), set((1,2)), set("abc"), set({"a":1,"b":2}), set(range(3)), set(set([1])))"#,
        Err(
            r#"in call to set(), parameter 'elements' got value of type 'string', want 'iterable'"#,
        ),
    ),
    (
        r#"print(set(1))"#,
        Err(r#"in call to set(), parameter 'elements' got value of type 'int', want 'iterable'"#),
    ),
    (r#"print(set([[]]))"#, Err(r#"unhashable type: 'list'"#)),
    (r#"print(set([{}]))"#, Err(r#"unhashable type: 'dict'"#)),
    (
        r#"print(set([1],[2]))"#,
        Err(r#"set() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(set(x=[1]))"#,
        Err(r#"set() got unexpected keyword argument 'x'"#),
    ),
    (
        r#"print(set(elements=[1]))"#,
        Err(r#"set() got named argument for positional-only parameter 'elements'"#),
    ),
    (
        r#"print(set(None))"#,
        Err(
            r#"in call to set(), parameter 'elements' got value of type 'NoneType', want 'iterable'"#,
        ),
    ),
    (
        r#"print(set([struct(a=1)]))"#,
        Ok(r#"set([struct(a = 1)])"#),
    ),
    (
        r#"print(set([struct(a=[])]))"#,
        Err(r#"unhashable type: 'struct'"#),
    ),
    (r#"print(set([1,1.0,True]))"#, Ok(r#"set([1, True])"#)),
    (
        r#"print(set(["b","a","c"]), set([3,1,2]))"#,
        Ok(r#"set(["b", "a", "c"]) set([3, 1, 2])"#),
    ),
    (
        r#"print(str(set([1])), repr(set(["a"])), type(set()))"#,
        Ok(r#"set([1]) set(["a"]) set"#),
    ),
    (r#"print(bool(set()), bool(set([1])))"#, Ok(r#"False True"#)),
    (r#"print(len(set([1,2,2])))"#, Ok(r#"2"#)),
    (r#"print(len(set()))"#, Ok(r#"0"#)),
    (r#"print([x for x in set([3,1,2])])"#, Ok(r#"[3, 1, 2]"#)),
    (
        r#"print(1 in set([1]), 2 in set([1]), [] in set([1]))"#,
        Ok(r#"True False False"#),
    ),
    (
        r#"print(set([1,2]) == set([2,1]), set([1]) == set([1,2]), set([1]) == [1], set() == set())"#,
        Ok(r#"True False False True"#),
    ),
    (r#"print(set([1,2]) != set([2,1]))"#, Ok(r#"False"#)),
    (
        r#"print(set([1]) < set([1,2]), set([1]) <= set([1]), set([1,2]) > set([1]), set([1]) >= set([2]))"#,
        Err(r#"unsupported comparison: set <=> set"#),
    ),
    (
        r#"print(set([1]) < 1)"#,
        Err(r#"unsupported comparison: set <=> int"#),
    ),
    (r#"print(set([1,2]) | set([2,3]))"#, Ok(r#"set([1, 2, 3])"#)),
    (r#"print(set([1,2]) & set([2,3]))"#, Ok(r#"set([2])"#)),
    (r#"print(set([1,2]) - set([2,3]))"#, Ok(r#"set([1])"#)),
    (r#"print(set([1,2]) ^ set([2,3]))"#, Ok(r#"set([1, 3])"#)),
    (
        r#"print(set([1,2]) + set([2,3]))"#,
        Err(r#"unsupported binary operation: set + set"#),
    ),
    (
        r#"print(set([1,2]) | [3])"#,
        Err(r#"unsupported binary operation: set | list"#),
    ),
    (
        r#"print(set([1,2]) & [2])"#,
        Err(r#"unsupported binary operation: set & list"#),
    ),
    (
        r#"print(set([1,2]) - [2])"#,
        Err(r#"unsupported binary operation: set - list"#),
    ),
    (
        r#"print(set([1,2]) ^ [2])"#,
        Err(r#"unsupported binary operation: set ^ list"#),
    ),
    (
        r#"print([1] | set([1]))"#,
        Err(r#"unsupported binary operation: list | set"#),
    ),
    (
        r#"print(set([1]) * 2)"#,
        Err(r#"unsupported binary operation: set * int"#),
    ),
    (
        r#"print(set([1])[0])"#,
        Err(r#"type 'set' has no operator [](int)"#),
    ),
    (
        r#"print(set([1])["a"])"#,
        Err(r#"type 'set' has no operator [](string)"#),
    ),
    (
        r#"print(hash(set([1])))"#,
        Err(r#"in call to hash(), parameter 'value' got value of type 'set', want 'string'"#),
    ),
    (r#"print({set([1]):1})"#, Err(r#"unhashable type: 'set'"#)),
    (
        r#"print(set([set([1])]))"#,
        Err(r#"unhashable type: 'set'"#),
    ),
    (r#"print(set([1]).add(2))"#, Ok(r#"None"#)),
    (
        r#"s=set([1])
s.add(2)
print(s)"#,
        Ok(r#"set([1, 2])"#),
    ),
    (
        r#"s=set([1])
print(s.add(1), s)"#,
        Ok(r#"None set([1])"#),
    ),
    (
        r#"s=set([1])
print(s.add([]))"#,
        Err(r#"unhashable type: 'list'"#),
    ),
    (
        r#"s=set([1])
print(s.add())"#,
        Err(r#"add() missing 1 required positional argument: element"#),
    ),
    (
        r#"s=set([1])
print(s.add(1,2))"#,
        Err(r#"add() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"s=set([1])
print(s.add(x=2))"#,
        Err(r#"add() got unexpected keyword argument 'x'"#),
    ),
    (
        r#"s=set([1,2,3])
print(s.pop(), s)"#,
        Ok(r#"1 set([2, 3])"#),
    ),
    (
        r#"s=set([3,1,2])
print(s.pop(), s.pop(), s.pop(), s)"#,
        Ok(r#"3 1 2 set()"#),
    ),
    (
        r#"s=set()
print(s.pop())"#,
        Err(r#"set is empty"#),
    ),
    (
        r#"s=set([1,2])
print(s.remove(1), s)"#,
        Ok(r#"None set([2])"#),
    ),
    (
        r#"s=set([1,2])
print(s.remove(3))"#,
        Err(r#"element 3 not found in set"#),
    ),
    (
        r#"s=set([1,2])
print(s.remove([]))"#,
        Err(r#"unhashable type: 'list'"#),
    ),
    (
        r#"s=set([1,2])
print(s.discard(1), s.discard(3), s)"#,
        Ok(r#"None None set([2])"#),
    ),
    (
        r#"s=set([1,2])
print(s.discard([]))"#,
        Err(r#"unhashable type: 'list'"#),
    ),
    (
        r#"s=set([1,2])
print(s.clear(), s)"#,
        Ok(r#"None set()"#),
    ),
    (
        r#"s=set([1,2])
print(s.clear(1))"#,
        Err(r#"clear() got unexpected positional argument"#),
    ),
    (
        r#"s=set([1,2])
print(s.update([3,4]), s)"#,
        Ok(r#"None set([1, 2, 3, 4])"#),
    ),
    (
        r#"s=set([1,2])
print(s.update([3],[4]), s)"#,
        Ok(r#"None set([1, 2, 3, 4])"#),
    ),
    (
        r#"s=set([1,2])
print(s.update(), s)"#,
        Ok(r#"None set([1, 2])"#),
    ),
    (
        r#"s=set([1,2])
print(s.update(5))"#,
        Err(
            r#"for update argument got value of type 'int', want a collection of hashable elements"#,
        ),
    ),
    (
        r#"s=set([1,2])
print(s.update([[]]))"#,
        Err(r#"unhashable type: 'list'"#),
    ),
    (
        r#"s=set([1,2])
print(s.update(set([9])), s)"#,
        Ok(r#"None set([1, 2, 9])"#),
    ),
    (
        r#"s=set([1,2])
print(s.update({"a":1}), s)"#,
        Ok(r#"None set([1, 2, "a"])"#),
    ),
    (
        r#"s=set([1,2])
print(s.update("ab"), s)"#,
        Err(
            r#"for update argument got value of type 'string', want a collection of hashable elements"#,
        ),
    ),
    (
        r#"s=set([1,2])
print(s.update(x=[3]))"#,
        Err(r#"update() got unexpected keyword argument 'x'"#),
    ),
    (
        r#"s=set([1,2])
print(s.update([3],x=[4]))"#,
        Err(r#"update() got unexpected keyword argument 'x'"#),
    ),
    (
        r#"print(set([1,2]).union([3]), set([1,2]).union([3],[4]), set([1,2]).union(), set([1,2]).union(set([5])))"#,
        Ok(r#"set([1, 2, 3]) set([1, 2, 3, 4]) set([1, 2]) set([1, 2, 5])"#),
    ),
    (
        r#"print(set([1,2]).union(5))"#,
        Err(
            r#"for union argument got value of type 'int', want a collection of hashable elements"#,
        ),
    ),
    (
        r#"print(set([1,2]).union([[]]))"#,
        Err(r#"unhashable type: 'list'"#),
    ),
    (
        r#"print(set([1,2,3]).intersection([2,3,4]), set([1,2,3]).intersection([2,3,4],[3]), set([1,2,3]).intersection())"#,
        Ok(r#"set([2, 3]) set([3]) set([1, 2, 3])"#),
    ),
    (
        r#"print(set([1,2,3]).difference([2]), set([1,2,3]).difference([2],[3]), set([1,2,3]).difference())"#,
        Ok(r#"set([1, 3]) set([1]) set([1, 2, 3])"#),
    ),
    (
        r#"print(set([1,2,3]).symmetric_difference([2,4]))"#,
        Ok(r#"set([1, 3, 4])"#),
    ),
    (
        r#"print(set([1,2,3]).symmetric_difference([2],[3]))"#,
        Err(r#"symmetric_difference() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(set([1,2,3]).symmetric_difference())"#,
        Err(r#"symmetric_difference() missing 1 required positional argument: other"#),
    ),
    (
        r#"print(set([1,2,3]).symmetric_difference(5))"#,
        Err(
            r#"for symmetric_difference argument got value of type 'int', want a collection of hashable elements"#,
        ),
    ),
    (
        r#"print(set([1,2,3]).isdisjoint([4]), set([1,2,3]).isdisjoint([3]), set().isdisjoint([]))"#,
        Ok(r#"True False True"#),
    ),
    (
        r#"print(set([1,2,3]).isdisjoint(5))"#,
        Err(
            r#"for isdisjoint argument got value of type 'int', want a collection of hashable elements"#,
        ),
    ),
    (
        r#"print(set([1,2]).issubset([1,2,3]), set([1,4]).issubset([1,2,3]), set().issubset([]))"#,
        Ok(r#"True False True"#),
    ),
    (
        r#"print(set([1,2]).issuperset([1]), set([1]).issuperset([1,2]))"#,
        Ok(r#"True False"#),
    ),
    (
        r#"print(set([1,2]).issubset(5))"#,
        Err(
            r#"for issubset argument got value of type 'int', want a collection of hashable elements"#,
        ),
    ),
    (
        r#"print(set([1,2]).issubset(1,2))"#,
        Err(r#"issubset() accepts no more than 1 positional argument but got 2"#),
    ),
    (r#"print(set([1,2]).issuperset(set([1])))"#, Ok(r#"True"#)),
    (r#"print(set([1,2]).issubset({1:2,2:3}))"#, Ok(r#"True"#)),
    (
        r#"print(set([1,2]).issubset("ab"))"#,
        Err(
            r#"for issubset argument got value of type 'string', want a collection of hashable elements"#,
        ),
    ),
    (
        r#"s=set([1,2,3])
print(s.intersection_update([2,3,4]), s)"#,
        Ok(r#"None set([2, 3])"#),
    ),
    (
        r#"s=set([1,2,3])
print(s.intersection_update(), s)"#,
        Ok(r#"None set([1, 2, 3])"#),
    ),
    (
        r#"s=set([1,2,3])
print(s.intersection_update([2],[3]), s)"#,
        Ok(r#"None set()"#),
    ),
    (
        r#"s=set([1,2,3])
print(s.intersection_update(5))"#,
        Err(
            r#"for intersection_update argument got value of type 'int', want a collection of hashable elements"#,
        ),
    ),
    (
        r#"s=set([1,2,3])
print(s.difference_update([2]), s)"#,
        Ok(r#"None set([1, 3])"#),
    ),
    (
        r#"s=set([1,2,3])
print(s.difference_update([2],[3]), s)"#,
        Ok(r#"None set([1])"#),
    ),
    (
        r#"s=set([1,2,3])
print(s.difference_update(), s)"#,
        Ok(r#"None set([1, 2, 3])"#),
    ),
    (
        r#"s=set([1,2,3])
print(s.symmetric_difference_update([2,4]), s)"#,
        Ok(r#"None set([1, 3, 4])"#),
    ),
    (
        r#"s=set([1,2,3])
print(s.symmetric_difference_update([2],[3]), s)"#,
        Err(
            r#"symmetric_difference_update() accepts no more than 1 positional argument but got 2"#,
        ),
    ),
    (
        r#"s=set([1,2,3])
print(s.symmetric_difference_update(), s)"#,
        Err(r#"symmetric_difference_update() missing 1 required positional argument: other"#),
    ),
    (
        r#"s=set([1,2,3])
print(s.symmetric_difference_update(5))"#,
        Err(
            r#"for symmetric_difference_update argument got value of type 'int', want a collection of hashable elements"#,
        ),
    ),
    (
        r#"s=set([1,2])
s.remove(1)
s.add(1)
print(s)"#,
        Ok(r#"set([2, 1])"#),
    ),
    (
        r#"s=set([1,2,3])
s.discard(1)
s.add(1)
print(s)"#,
        Ok(r#"set([2, 3, 1])"#),
    ),
    (r#"s=set([1])"#, Ok(r#""#)),
    (r#"s=set([1,2])"#, Ok(r#""#)),
    (
        r#"s=set([1,2])
print(s.union([2,9]) == set([1,2,9]), s)"#,
        Ok(r#"True set([1, 2])"#),
    ),
    (
        r#"s=set([1,2])
print(dir(s) == dir(set()))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(set([1]).nope)"#,
        Err(r#"'set' value has no field or method 'nope'"#),
    ),
    (
        r#"print(set([1]).add)"#,
        Ok(r#"<built-in method add of set value>"#),
    ),
    (
        r#"print(getattr(set([1]),"add",None) != None, hasattr(set([1]),"nope"))"#,
        Ok(r#"True False"#),
    ),
    (r#"print(sorted(set([3,1,2])))"#, Ok(r#"[1, 2, 3]"#)),
    (
        r#"print(list(set([3,1,2])), tuple(set([3,1,2])), dict([(k,1) for k in set(["a"])]))"#,
        Ok(r#"[3, 1, 2] (3, 1, 2) {"a": 1}"#),
    ),
    (
        r#"print(str(set([set()])))"#,
        Err(r#"unhashable type: 'set'"#),
    ),
    (r#"print(set([(1,2),(1,2)]))"#, Ok(r#"set([(1, 2)])"#)),
    (r#"print(set([1,2]).union(set()))"#, Ok(r#"set([1, 2])"#)),
    (
        r#"print(set([None]), set([None,None]), set([1,"a",None,(1,)]))"#,
        Ok(r#"set([None]) set([None]) set([1, "a", None, (1,)])"#),
    ),
    (
        r#"print(set([1,2,3]).intersection([3,2,1,1]))"#,
        Ok(r#"set([1, 2, 3])"#),
    ),
    (r#"print(reversed(set([1,2])))"#, Ok(r#"[2, 1]"#)),
    (r#"print(enumerate(set([1,2])))"#, Ok(r#"[(0, 1), (1, 2)]"#)),
    (
        r#"print(list(zip(set([1,2]),set([3,4]))))"#,
        Ok(r#"[(1, 3), (2, 4)]"#),
    ),
    (r#"print(set([1,2]) in [set([1,2])])"#, Ok(r#"True"#)),
    (r#"print(set([1,2]) in set([1,2]))"#, Ok(r#"False"#)),
    (
        r#"print(struct(a=set([1])))"#,
        Ok(r#"struct(a = set([1]))"#),
    ),
    (
        r#"print(struct(a=set([1]))==struct(a=set([1])))"#,
        Ok(r#"True"#),
    ),
    (r#"print({"a":set([1,2])})"#, Ok(r#"{"a": set([1, 2])}"#)),
    (
        r#"print(depset([set([1])]))"#,
        Err(r#"depset elements must not be mutable values"#),
    ),
    (r#"print(set([depset([1])]))"#, Ok(r#"set([depset([1])])"#)),
    (
        r#"print(set([depset([1])]) == set([depset([1])]))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print("%s %r" % (set([1]), set(["a"])))"#,
        Ok(r#"set([1]) set(["a"])"#),
    ),
    (
        r#"print(set([1,2]).union.__name__)"#,
        Err(r#"'builtin_function_or_method' value has no field or method '__name__'"#),
    ),
    (
        r#"def f():
  s=set([1,2,3])
  s |= set([4])
  print(s)
f()"#,
        Ok(r#"set([1, 2, 3, 4])"#),
    ),
    (
        r#"def f():
  s=set([1,2,3])
  t=s
  s |= set([4])
  print(t, s == t)
f()"#,
        Ok(r#"set([1, 2, 3, 4]) True"#),
    ),
    (
        r#"def f():
  s=set([1,2,3])
  t=s
  s &= set([2])
  print(t)
f()"#,
        Ok(r#"set([2])"#),
    ),
    (
        r#"def f():
  s=set([1,2,3])
  t=s
  s -= set([2])
  print(t)
f()"#,
        Ok(r#"set([1, 3])"#),
    ),
    (
        r#"def f():
  s=set([1,2,3])
  t=s
  s ^= set([2,9])
  print(t)
f()"#,
        Ok(r#"set([1, 3, 9])"#),
    ),
    (
        r#"def f():
  s=set([1,2,3])
  s |= [4]
f()"#,
        Err(r#"unsupported binary operation: set | list"#),
    ),
    (
        r#"def f():
  s=set([1,2,3])
  s &= [4]
f()"#,
        Err(r#"unsupported binary operation: set & list"#),
    ),
    (
        r#"def f():
  s=set([1,2,3])
  s -= [4]
f()"#,
        Err(r#"unsupported binary operation: set - list"#),
    ),
    (
        r#"def f():
  s=set([1,2,3])
  s ^= [4]
f()"#,
        Err(r#"unsupported binary operation: set ^ list"#),
    ),
    (
        r#"def f():
  s=set([1,2,3])
  s += set([4])
f()"#,
        Err(r#"unsupported binary operation: set + set"#),
    ),
    (
        r#"def f():
  s=set([1])
  t=s
  s = s | set([2])
  print(t, s)
f()"#,
        Ok(r#"set([1]) set([1, 2])"#),
    ),
    (
        r#"def f():
  s=set([1])
  for x in s:
    s.add(x+10)
f()"#,
        Err(r#"set value is temporarily immutable due to active for-loop iteration"#),
    ),
    (
        r#"def f():
  s=set([1])
  for x in s:
    s.remove(x)
f()"#,
        Err(r#"set value is temporarily immutable due to active for-loop iteration"#),
    ),
    (
        r#"def f():
  s=set([1,2])
  for x in s:
    s.add(1)
  print(s)
f()"#,
        Err(r#"set value is temporarily immutable due to active for-loop iteration"#),
    ),
    (
        r#"def f():
  s=set([1,2])
  for x in s:
    s.discard(9)
  print(s)
f()"#,
        Err(r#"set value is temporarily immutable due to active for-loop iteration"#),
    ),
    (
        r#"def f():
  s=set([1,2])
  for x in s:
    s.clear()
f()"#,
        Err(r#"set value is temporarily immutable due to active for-loop iteration"#),
    ),
    (
        r#"def f():
  s=set([1,2])
  for x in s:
    s.pop()
f()"#,
        Err(r#"set value is temporarily immutable due to active for-loop iteration"#),
    ),
    (
        r#"def f():
  s=set([1,2])
  for x in s:
    s.update([])
  print(s)
f()"#,
        Err(r#"set value is temporarily immutable due to active for-loop iteration"#),
    ),
    (
        r#"def f():
  s=set([1,2])
  for x in s:
    s.update([5])
f()"#,
        Err(r#"set value is temporarily immutable due to active for-loop iteration"#),
    ),
    (
        r#"def f():
  s=set([1,2])
  for x in s:
    s.intersection_update([1])
f()"#,
        Err(r#"set value is temporarily immutable due to active for-loop iteration"#),
    ),
    (
        r#"def f():
  s=set([1,2])
  for x in s:
    s.difference_update([])
  print(s)
f()"#,
        Err(r#"set value is temporarily immutable due to active for-loop iteration"#),
    ),
    (
        r#"def f():
  s=set([1,2])
  for x in s:
    s |= set([1])
  print(s)
f()"#,
        Err(r#"set value is temporarily immutable due to active for-loop iteration"#),
    ),
    (
        r#"def f():
  s=set([1,2])
  for x in s:
    s |= set([7])
f()"#,
        Err(r#"set value is temporarily immutable due to active for-loop iteration"#),
    ),
    (
        r#"def f():
  s=set([1,2])
  print([s.add(9) for x in s])
f()"#,
        Err(r#"set value is temporarily immutable due to active for-loop iteration"#),
    ),
    (
        r#"def f():
  s=set([1,2])
  print([x for x in s if s.discard(x) == None])
f()"#,
        Err(r#"set value is temporarily immutable due to active for-loop iteration"#),
    ),
    (
        r#"def f():
  s=set([1,2,3])
  print(s.intersection([1]) == set([1]), s)
f()"#,
        Ok(r#"True set([1, 2, 3])"#),
    ),
    (
        r#"Y=set([1])
Y.add(2)
print(Y)"#,
        Ok(r#"set([1, 2])"#),
    ),
    (r#"def f(): X.add(2)"#, Ok(r#""#)),
    (r#"S=set([1])"#, Ok(r#""#)),
    (
        r#"print(set([1]) == set([1.0]), set([1]) == set([True]))"#,
        Ok(r#"True False"#),
    ),
    (r#"print(set([[1]]))"#, Err(r#"unhashable type: 'list'"#)),
    (r#"print(set([[]]), 1)"#, Err(r#"unhashable type: 'list'"#)),
    (
        r#"print(set([print]))"#,
        Err(r#"unhashable type: 'builtin_function_or_method'"#),
    ),
    (
        r#"print(set([lambda: 1]))"#,
        Ok(r#"set([<function lambda from //:u.bzl>])"#),
    ),
    (
        r#"print(set([struct(a=lambda: 1)]))"#,
        Ok(r#"set([struct(a = <function lambda from //:u.bzl>)])"#),
    ),
    (r#"print(set([json]))"#, Err(r#"unhashable type: 'json'"#)),
    (
        r#"print(set([struct(a=depset())]))"#,
        Ok(r#"set([struct(a = depset([]))])"#),
    ),
    (
        r#"print(set(depset([1])))"#,
        Err(
            r#"in call to set(), parameter 'elements' got value of type 'depset', want 'iterable'"#,
        ),
    ),
    (
        r#"print(set(struct(a=1)))"#,
        Err(
            r#"in call to set(), parameter 'elements' got value of type 'struct', want 'iterable'"#,
        ),
    ),
    (
        r#"print(set(set([1,2])).add)"#,
        Ok(r#"<built-in method add of set value>"#),
    ),
    (
        r#"print(set(range(1000)) == set(range(1000)))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(set([1,2,3]).pop(), set([1,2,3]).pop())"#,
        Ok(r#"1 1"#),
    ),
    (
        r#"print(len(set(range(10)).union(range(5,15))))"#,
        Ok(r#"15"#),
    ),
    (r#"print(set([3,1]) | set([2,1]))"#, Ok(r#"set([3, 1, 2])"#)),
    (r#"print(set([3,1]) & set([1,3,2]))"#, Ok(r#"set([3, 1])"#)),
    (r#"print(set([3,1,2]) - set([1]))"#, Ok(r#"set([3, 2])"#)),
    (
        r#"print(set([3,1,2]) ^ set([1,5]))"#,
        Ok(r#"set([3, 2, 5])"#),
    ),
    (
        r#"print(set([1,2]).symmetric_difference({2:0,5:0}))"#,
        Ok(r#"set([1, 5])"#),
    ),
    (r#"print(set([1,2]).issubset(set([1,2])))"#, Ok(r#"True"#)),
    (r#"print(set([1]).union((2,3)))"#, Ok(r#"set([1, 2, 3])"#)),
    (
        r#"print(set([1]).union(range(3)))"#,
        Ok(r#"set([1, 0, 2])"#),
    ),
    (
        r#"print(set([1]).union(depset([2])))"#,
        Err(
            r#"for union argument got value of type 'depset', want a collection of hashable elements"#,
        ),
    ),
    (
        r#"print(set([1]).union(struct(a=1)))"#,
        Err(
            r#"for union argument got value of type 'struct', want a collection of hashable elements"#,
        ),
    ),
    (
        r#"print(set([1]).union(None))"#,
        Err(
            r#"for union argument got value of type 'NoneType', want a collection of hashable elements"#,
        ),
    ),
    (
        r#"print(set([1]).union("abc"))"#,
        Err(
            r#"for union argument got value of type 'string', want a collection of hashable elements"#,
        ),
    ),
    (
        r#"print(set([1]).union(1, 2))"#,
        Err(
            r#"for union argument got value of type 'int', want a collection of hashable elements"#,
        ),
    ),
    (
        r#"print(set([1]).isdisjoint(None))"#,
        Err(
            r#"for isdisjoint argument got value of type 'NoneType', want a collection of hashable elements"#,
        ),
    ),
    (
        r#"print(set([1]).update(None))"#,
        Err(
            r#"for update argument got value of type 'NoneType', want a collection of hashable elements"#,
        ),
    ),
    (
        r#"print(set.union)"#,
        Err(r#"'builtin_function_or_method' value has no field or method 'union'"#),
    ),
    (
        r#"print(type(set.union))"#,
        Err(r#"'builtin_function_or_method' value has no field or method 'union'"#),
    ),
    (r#"print(set([1]).union == set([1]).union)"#, Ok(r#"False"#)),
    (
        r#"print(type(set([1]).union))"#,
        Ok(r#"builtin_function_or_method"#),
    ),
    (r#"print(dir(set))"#, Ok(r#"[]"#)),
];

/// The analysis namespaces have the names Bazel 9.2.0's `dir()` printed for
/// them (buildfiji-b0s), and a function that is not there yet says so.
#[test]
fn analysis_namespaces_have_bazels_names() {
    use crate::test_support::run;
    let printed = run(r#"
def f():
    for ns in [platform_common, config_common, coverage_common, testing, cc_common, java_common, apple_common, android_common]:
        print(dir(ns))
f()
"#)
    .expect("the namespaces exist");
    assert_eq!(
        printed,
        [
            r#"["ConstraintSettingInfo", "ConstraintValueInfo", "PlatformInfo", "TemplateVariableInfo", "ToolchainInfo"]"#,
            r#"["FeatureFlagInfo", "config_feature_flag_transition", "toolchain_type"]"#,
            r#"["instrumented_files_info"]"#,
            r#"["ExecutionInfo", "TestEnvironment", "analysis_test"]"#,
            r#"["action_is_enabled", "add_go_exec_groups_to_binary_rules", "check_experimental_cc_shared_library", "do_not_use_tools_cpp_compiler_present", "empty_variables", "get_environment_variables", "get_execution_requirements", "get_memory_inefficient_command_line", "get_tool_for_action", "get_tool_requirement_for_action", "implementation_deps_allowed_by_allowlist", "incompatible_disable_objc_library_transition", "internal_DO_NOT_USE", "legacy_cc_flags_make_variable_do_not_use"]"#,
            r#"["internal_DO_NOT_USE"]"#,
            r#"["Objc", "XcodeProperties", "XcodeVersionConfig", "apple_host_system_env", "apple_toolchain", "dotted_version", "new_objc_provider", "platform", "platform_type", "target_apple_env"]"#,
            r#"["create_dex_merger_actions", "resource_source_directory"]"#,
        ]
    );
    assert_eq!(
        run("print(cc_common.internal_DO_NOT_USE().freeze([1]))").unwrap(),
        ["[1]"]
    );
    assert_eq!(
        run("print(java_common.internal_DO_NOT_USE().google_legacy_api_enabled())").unwrap(),
        ["False"]
    );
    let err = run("cc_common.legacy_cc_flags_make_variable_do_not_use()")
        .expect_err("not implemented yet");
    assert!(
        err.contains("cc_common.legacy_cc_flags_make_variable_do_not_use is not implemented yet"),
        "{err}"
    );
    // toolchain_type(), as Bazel 9.2.0 answered it.
    let t = |body: &str| {
        run(&format!(
            "x = config_common.toolchain_type({body})\nprint(x.toolchain_type == Label('//a:b'), x.toolchain_type.name, x.mandatory)\nprint(type(x), dir(x))"
        ))
    };
    assert_eq!(
        t("'//a:b', mandatory = False").unwrap(),
        [
            "True b False",
            "toolchain_type [\"mandatory\", \"toolchain_type\"]"
        ]
    );
    assert_eq!(t("'!!'").unwrap()[0], "False !! True");
    assert_eq!(t("'@@x//a:b'").unwrap()[0], "False b True");
    assert_eq!(t("Label('//a:b')").unwrap()[0], "True b True");
    for (body, want) in [
        (
            "1",
            "parameter 'name' got value of type 'int', want 'string or Label'",
        ),
        (
            "None",
            "parameter 'name' got value of type 'NoneType', want 'string or Label'",
        ),
        (
            "'//a:b', mandatory = 1",
            "parameter 'mandatory' got value of type 'int', want 'bool'",
        ),
        (
            "'//a:b', True",
            "toolchain_type() accepts no more than 1 positional argument but got 2",
        ),
        (
            "'//a:b', x = 1",
            "toolchain_type() got unexpected keyword argument 'x'",
        ),
        (
            "",
            "toolchain_type() missing 1 required positional argument: name",
        ),
    ] {
        let err = t(body).expect_err(body);
        assert!(err.contains(want), "{body}: {err}");
    }
    assert_eq!(
        run("print(apple_common.platform.ios_device)\nprint(apple_common.platform_type.ios)")
            .unwrap(),
        [
            r#"struct(is_device = True, name = "ios_device", name_in_plist = "iPhoneOS", platform_type = "ios")"#,
            "ios"
        ]
    );
    // Both a rule and an exec group take it.
    assert_eq!(
        run("t = config_common.toolchain_type('//a:b')\nprint(exec_group(toolchains = [t, '//c:d']))\nr = rule(implementation = lambda ctx: [], toolchains = [t])\nprint(t == config_common.toolchain_type('//a:b'), t == config_common.toolchain_type('//a:b', mandatory = False))").unwrap(),
        [
            "<unknown object com.google.devtools.build.lib.packages.DeclaredExecGroup>",
            "True False"
        ]
    );
}

/// Names that rules_java 9.1.0 and rules_cc pass to `configuration_field`, which
/// Bazel 9.2.0 accepts (probed one by one; `java_launcher` and `cc_compiler`
/// it refuses).
#[test]
fn configuration_fields_of_the_rulesets() {
    use crate::test_support::run;
    for (fragment, name) in [
        ("java", "java_toolchain_bytecode_optimizer"),
        ("cpp", "fdo_optimize"),
        ("cpp", "xbinary_fdo"),
        ("cpp", "proto_profile_path"),
    ] {
        run(&format!(
            "configuration_field(fragment = '{fragment}', name = '{name}')"
        ))
        .unwrap_or_else(|e| panic!("{fragment} {name}: {e}"));
    }
}

/// `default` of `dict.get` and `dict.setdefault` can be named (Bazel 9.2.0; patch
/// 0008 to the starlark crate), and `key` of `get` cannot.
#[test]
fn dict_get_and_setdefault_take_default_by_name() {
    use crate::test_support::run;
    assert_eq!(
        run("d = {'a': 1}\nprint(d.get('b', default = 5), d.get('a', default = 5))\nprint(dict(d).setdefault('b', default = 6))").unwrap(),
        ["5 1", "6"]
    );
}
