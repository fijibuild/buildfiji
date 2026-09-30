//! Generated from Bazel 9.2.0 probes: what `select()` prints and refuses.

use crate::test_support::BuildRow;

pub(crate) const SELECT_CASES: &[(&str, Result<&str, &str>)] = &[
    (
        r#"s=select({"//conditions:default":[1]})
print(s)
print(type(s))"#,
        Ok(r#"select({"//conditions:default": [1]})
select"#),
    ),
    (
        r#"s=select({"//conditions:default":[1]})
print(repr(s))
print(str(s))"#,
        Ok(r#"select({"//conditions:default": [1]})
select({"//conditions:default": [1]})"#),
    ),
    (
        r#"print(select({"//conditions:default":[1], ":a":[2]}))"#,
        Ok(r#"select({"//conditions:default": [1], ":a": [2]})"#),
    ),
    (r#"print(select({":a":[2]}))"#, Ok(r#"select({":a": [2]})"#)),
    (
        r#"print(select({}))"#,
        Err(
            r#"select({}) with an empty dictionary can never resolve because it includes no conditions to match"#,
        ),
    ),
    (
        r#"print(select())"#,
        Err(r#"select() missing 1 required positional argument: x"#),
    ),
    (
        r#"print(select(1))"#,
        Err(r#"in call to select(), parameter 'x' got value of type 'int', want 'dict'"#),
    ),
    (
        r#"print(select([]))"#,
        Err(r#"in call to select(), parameter 'x' got value of type 'list', want 'dict'"#),
    ),
    (
        r#"print(select([("a",1)]))"#,
        Err(r#"in call to select(), parameter 'x' got value of type 'list', want 'dict'"#),
    ),
    (
        r#"print(select({1:2}))"#,
        Err(r#"select: got int for dict key, want a Label or label string"#),
    ),
    (
        r#"print(select({"a":1}, no_match_error="x"))"#,
        Ok(r#"select({"a": 1})"#),
    ),
    (
        r#"print(select({"a":1}, no_match_error=1))"#,
        Err(
            r#"in call to select(), parameter 'no_match_error' got value of type 'int', want 'string'"#,
        ),
    ),
    (
        r#"print(select({"a":1}, no_match_error=None))"#,
        Err(
            r#"in call to select(), parameter 'no_match_error' got value of type 'NoneType', want 'string'"#,
        ),
    ),
    (r#"print(select({"a":1}, "x"))"#, Ok(r#"select({"a": 1})"#)),
    (
        r#"print(select({"a":1}, foo=1))"#,
        Err(r#"select() got unexpected keyword argument 'foo'"#),
    ),
    (
        r#"print(select(x={"a":1}))"#,
        Err(r#"select() got named argument for positional-only parameter 'x'"#),
    ),
    (
        r#"print(select({"a":1}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of int, list)"#),
    ),
    (
        r#"print([1] + select({"a":[1]}))"#,
        Ok(r#"[1] + select({"a": [1]})"#),
    ),
    (
        r#"print(select({"a":[1]}) + [1])"#,
        Ok(r#"select({"a": [1]}) + [1]"#),
    ),
    (
        r#"print(select({"a":[1]}) + select({"b":[2]}))"#,
        Ok(r#"select({"a": [1]}) + select({"b": [2]})"#),
    ),
    (
        r#"print(select({"a":[1]}) + (1,))"#,
        Ok(r#"select({"a": [1]}) + (1,)"#),
    ),
    (
        r#"print((1,) + select({"a":[1]}))"#,
        Ok(r#"(1,) + select({"a": [1]})"#),
    ),
    (
        r#"print(select({"a":(1,)}) + [1])"#,
        Ok(r#"select({"a": (1,)}) + [1]"#),
    ),
    (
        r#"print(select({"a":1}) + 1)"#,
        Ok(r#"select({"a": 1}) + 1"#),
    ),
    (
        r#"print(1 + select({"a":1}))"#,
        Ok(r#"1 + select({"a": 1})"#),
    ),
    (
        r#"print(select({"a":1}) + "x")"#,
        Err(r#"Cannot combine incompatible types (select of int, string)"#),
    ),
    (
        r#"print(select({"a":"x"}) + "x")"#,
        Ok(r#"select({"a": "x"}) + "x""#),
    ),
    (
        r#"print("x" + select({"a":"x"}))"#,
        Ok(r#""x" + select({"a": "x"})"#),
    ),
    (
        r#"print(select({"a":{"k":1}}) + {"j":1})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"print({"j":1} + select({"a":{"k":1}}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"print(select({"a":{"k":1}}) | {"j":1})"#,
        Ok(r#"select({"a": {"k": 1}}) | {"j": 1}"#),
    ),
    (
        r#"print({"j":1} | select({"a":{"k":1}}))"#,
        Ok(r#"{"j": 1} | select({"a": {"k": 1}})"#),
    ),
    (
        r#"print(select({"a":{"k":1}}) | select({"b":{"j":1}}))"#,
        Ok(r#"select({"a": {"k": 1}}) | select({"b": {"j": 1}})"#),
    ),
    (
        r#"print(select({"a":[1]}) | select({"b":[2]}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"print(select({"a":[1]}) * 2)"#,
        Err(r#"unsupported binary operation: select * int"#),
    ),
    (
        r#"print(2 * select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: int * select"#),
    ),
    (
        r#"print(select({"a":[1]}) - select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select - select"#),
    ),
    (
        r#"print(len(select({"a":[1]})))"#,
        Err(
            r#"in call to len(), parameter 'x' got value of type 'select', want 'iterable or string'"#,
        ),
    ),
    (r#"print(bool(select({"a":[1]})))"#, Ok(r#"True"#)),
    (
        r#"print([x for x in select({"a":[1]})])"#,
        Err(r#"type 'select' is not iterable"#),
    ),
    (
        r#"print(select({"a":[1]})[0])"#,
        Err(r#"type 'select' has no operator [](int)"#),
    ),
    (
        r#"print(select({"a":[1]}).foo)"#,
        Err(r#"'select' value has no field or method 'foo'"#),
    ),
    (r#"print(dir(select({"a":[1]})))"#, Ok(r#"[]"#)),
    (
        r#"print(select({"a":[1]}) == select({"a":[1]}))"#,
        Ok(r#"True"#),
    ),
    (r#"print(select({"a":[1]}) == [1])"#, Ok(r#"False"#)),
    (
        r#"print(hash(select({"a":[1]})))"#,
        Err(r#"in call to hash(), parameter 'value' got value of type 'select', want 'string'"#),
    ),
    (
        r#"print({select({"a":[1]}):1})"#,
        Err(r#"unhashable type: 'select'"#),
    ),
    (
        r#"print(1 in select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: int in select"#),
    ),
    (
        r#"print(str(select({"a":[1]}) + [2]))"#,
        Ok(r#"select({"a": [1]}) + [2]"#),
    ),
    (
        r#"print(select({"a":[1]}) + select({"b":[2]}) + [3])"#,
        Ok(r#"select({"a": [1]}) + select({"b": [2]}) + [3]"#),
    ),
    (
        r#"print(select({"a":[1]}, no_match_error="m") + select({"b":[2]}, no_match_error="n"))"#,
        Ok(r#"select({"a": [1]}) + select({"b": [2]})"#),
    ),
    (
        r#"print(select({"//conditions:default":1}))"#,
        Ok(r#"select({"//conditions:default": 1})"#),
    ),
    (
        r#"print(select({":a":1, "//:a":2}))"#,
        Ok(r#"select({":a": 1, "//:a": 2})"#),
    ),
    (
        r#"print(select({"//a:b":1, "@//a:b":2}))"#,
        Ok(r#"select({"//a:b": 1, "@//a:b": 2})"#),
    ),
    (r#"print(select({"x y":1}))"#, Ok(r#"select({"x y": 1})"#)),
    (r#"print(select({"":1}))"#, Ok(r#"select({"": 1})"#)),
    (
        r#"print(select({"a":1}.items()))"#,
        Err(r#"in call to select(), parameter 'x' got value of type 'list', want 'dict'"#),
    ),
    (
        r#"print(select(dict=1))"#,
        Err(r#"select() got unexpected keyword argument 'dict'"#),
    ),
    (r#"print(select({"a":None}))"#, Ok(r#"select({"a": None})"#)),
    (
        r#"print(select({"a":select({"b":1})}))"#,
        Ok(r#"select({"a": select({"b": 1})})"#),
    ),
    (
        r#"print(select({"a":select({"b":1})}) + [])"#,
        Err(r#"Cannot combine incompatible types (select of select, list)"#),
    ),
    (
        r#"print(select({"a":[1]}) + [] )"#,
        Ok(r#"select({"a": [1]}) + []"#),
    ),
    (
        r#"print(select({"a":[1]}) + select({"a":{"k":1}}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"print(select({"a":[1]}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of string)"#),
    ),
    (
        r#"print(select({"a":[1], "b":{"k":1}}))"#,
        Ok(r#"select({"a": [1], "b": {"k": 1}})"#),
    ),
    (
        r#"print(select({"a":[1], "b":{"k":1}}) + [1])"#,
        Ok(r#"select({"a": [1], "b": {"k": 1}}) + [1]"#),
    ),
    (
        r#"print(select({"a":[1], "b":(1,)}) + [1])"#,
        Ok(r#"select({"a": [1], "b": (1,)}) + [1]"#),
    ),
    (
        r#"print(select({"a":1, "b":"s"}) + 1)"#,
        Ok(r#"select({"a": 1, "b": "s"}) + 1"#),
    ),
    (
        r#"print(select({"a":None, "b":[1]}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, list)"#),
    ),
    (
        r#"print(select({"a":[1], "b":None}) + [1])"#,
        Ok(r#"select({"a": [1], "b": None}) + [1]"#),
    ),
    (
        r#"print(select({"a":[1]}) + [1] + [2])"#,
        Ok(r#"select({"a": [1]}) + [1] + [2]"#),
    ),
    (
        r#"print([1] + select({"a":[1]}) + [2])"#,
        Ok(r#"[1] + select({"a": [1]}) + [2]"#),
    ),
    (
        r#"print([1] + [2] + select({"a":[1]}))"#,
        Ok(r#"[1, 2] + select({"a": [1]})"#),
    ),
    (
        r#"print(([1] + [2]) + (select({"a":[1]}) + [3]))"#,
        Ok(r#"[1, 2] + select({"a": [1]}) + [3]"#),
    ),
    (
        r#"print(select({"a":[1]}) + (select({"b":[1]}) + [3]))"#,
        Ok(r#"select({"a": [1]}) + select({"b": [1]}) + [3]"#),
    ),
    (
        r#"print((select({"a":[1]}) + select({"b":[1]})) + (select({"c":[1]}) + select({"d":[1]})))"#,
        Ok(r#"select({"a": [1]}) + select({"b": [1]}) + select({"c": [1]}) + select({"d": [1]})"#),
    ),
    (
        r#"print(select({"a":[1]}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"print(select({"a":[1]}) | {"k":1})"#,
        Err(r#"Cannot combine incompatible types (select of list, dict)"#),
    ),
    (
        r#"print({"k":1} | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of list)"#),
    ),
    (
        r#"print(select({"a":{"k":1}}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"print(select({"a":{"k":1}}) | 1)"#,
        Err(r#"unsupported binary operation: select | int"#),
    ),
    (
        r#"print(select({"a":1}) | select({"b":1}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"print(select({"a":1}) | 1)"#,
        Err(r#"unsupported binary operation: select | int"#),
    ),
    (
        r#"print(select({"a":{"k":1}}) | select({"b":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"print(select({"a":{"k":1}}) | select({"b":{"j":1}}) | {"x":1})"#,
        Ok(r#"select({"a": {"k": 1}}) | select({"b": {"j": 1}}) | {"x": 1}"#),
    ),
    (
        r#"print(select({"a":{"k":1}}) | select({"b":{"j":1}}) + select({"c":{"j":1}}))"#,
        Err(r#"unsupported binary operation: select + select"#),
    ),
    (
        r#"print(select({"a":{"k":1}}) + select({"b":{"j":1}}))"#,
        Err(r#"unsupported binary operation: select + select"#),
    ),
    (
        r#"print(select({"a":{"k":1}}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of dict, int)"#),
    ),
    (
        r#"print(select({"a":[1]}) + None)"#,
        Err(r#"Cannot combine incompatible types (select of list, NoneType)"#),
    ),
    (
        r#"print(None + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (NoneType, select of list)"#),
    ),
    (
        r#"print(select({"a":[1]}) + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of NoneType)"#),
    ),
    (
        r#"print(select({"a":[1]}) + True)"#,
        Err(r#"Cannot combine incompatible types (select of list, bool)"#),
    ),
    (
        r#"print(select({"a":True}) + True)"#,
        Ok(r#"select({"a": True}) + True"#),
    ),
    (
        r#"print(select({"a":True}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of bool, int)"#),
    ),
    (
        r#"print(select({"a":1.5}) + 1.5)"#,
        Ok(r#"select({"a": 1.5}) + 1.5"#),
    ),
    (
        r#"print(select({"a":1.5}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of float, int)"#),
    ),
    (
        r#"print(select({"a":depset([1])}) + depset([2]))"#,
        Ok(r#"select({"a": depset([1])}) + depset([2])"#),
    ),
    (
        r#"print(select({"a":depset([1])}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of depset, list)"#),
    ),
    (
        r#"print(select({"a":range(2)}) + [1])"#,
        Ok(r#"select({"a": range(0, 2)}) + [1]"#),
    ),
    (
        r#"print(select({"a":range(2)}) + range(2))"#,
        Ok(r#"select({"a": range(0, 2)}) + range(0, 2)"#),
    ),
    (
        r#"print(select({"a":[1]}) + range(2))"#,
        Ok(r#"select({"a": [1]}) + range(0, 2)"#),
    ),
    (
        r#"print(select({"a":1}) + select({"b":2}))"#,
        Ok(r#"select({"a": 1}) + select({"b": 2})"#),
    ),
    (
        r#"print(select({"a":1}) + select({"b":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of string)"#),
    ),
    (
        r#"print(select({"a":"s"}) + select({"b":"t"}))"#,
        Ok(r#"select({"a": "s"}) + select({"b": "t"})"#),
    ),
    (
        r#"print(select({"a":"s"}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of string, list)"#),
    ),
    (
        r#"print(select({"a":(1,)}) + select({"b":[1]}))"#,
        Ok(r#"select({"a": (1,)}) + select({"b": [1]})"#),
    ),
    (
        r#"print(select({"a":(1,)}) + select({"b":(1,)}))"#,
        Ok(r#"select({"a": (1,)}) + select({"b": (1,)})"#),
    ),
    (
        r#"print(select({"a":(1,)}) + (1,))"#,
        Ok(r#"select({"a": (1,)}) + (1,)"#),
    ),
    (
        r#"print(select({"a":(1,)}) + select({"b":[1]}) + (1,))"#,
        Ok(r#"select({"a": (1,)}) + select({"b": [1]}) + (1,)"#),
    ),
    (
        r#"print(str(select({"a":[1]}) + select({"b":{"k":1}})))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"print(select({"a":[1]}) + [] + [])"#,
        Ok(r#"select({"a": [1]}) + [] + []"#),
    ),
    (
        r#"print([] + select({"a":[1]}))"#,
        Ok(r#"[] + select({"a": [1]})"#),
    ),
    (
        r#"print([] + select({"a":[1]}) + [])"#,
        Ok(r#"[] + select({"a": [1]}) + []"#),
    ),
    (
        r#"print(select({"a":"x y", "b":'q"uote'}))"#,
        Ok(r#"select({"a": "x y", "b": "q\"uote"})"#),
    ),
    (
        r#"print(select({"a":[1,2,3], "b":{"k":[1]}}, no_match_error="m"))"#,
        Ok(r#"select({"a": [1, 2, 3], "b": {"k": [1]}})"#),
    ),
    (
        r#"print(select({"a":1, "a":2}))"#,
        Err(r#"dictionary expression has duplicate key: "a""#),
    ),
    (
        r#"print(select({"a":1}) if True else 0)"#,
        Ok(r#"select({"a": 1})"#),
    ),
    (r#"print(select({"a":1}) and 1)"#, Ok(r#"1"#)),
    (r#"print(not select({"a":1}))"#, Ok(r#"False"#)),
    (
        r#"print(select({"a":1}) < select({"a":1}))"#,
        Err(r#"unsupported comparison: select <=> select"#),
    ),
    (
        r#"print(select({"a":1}) != select({"a":2}))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(select({"a":1}) == select({"a":1}, no_match_error="x"))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(select({"a":1}) == select({"a":1}) + [])"#,
        Err(r#"Cannot combine incompatible types (select of int, list)"#),
    ),
    (
        r#"print(type(select({"a":1}) + [1]))"#,
        Err(r#"Cannot combine incompatible types (select of int, list)"#),
    ),
    (r#"print(type(select({"a":[1]}) + [1]))"#, Ok(r#"select"#)),
    (
        r#"print(type(select))"#,
        Ok(r#"builtin_function_or_method"#),
    ),
    (r#"print(select)"#, Ok(r#"<built-in function select>"#)),
    (
        r#"print([select({"a":[1]})])"#,
        Ok(r#"[select({"a": [1]})]"#),
    ),
    (
        r#"print({"k":select({"a":[1]})})"#,
        Ok(r#"{"k": select({"a": [1]})}"#),
    ),
    (
        r#"print(json.encode(select({"a":[1]})))"#,
        Err(r#"cannot encode select as JSON"#),
    ),
    (
        r#"print(sorted(select({"a":[1]})))"#,
        Err(
            r#"in call to sorted(), parameter 'iterable' got value of type 'select', want 'iterable'"#,
        ),
    ),
    (
        r#"print(list(select({"a":[1]})))"#,
        Err(r#"in call to list(), parameter 'x' got value of type 'select', want 'iterable'"#),
    ),
    (
        r#"print(str(select({"a":1}), 1))"#,
        Err(r#"str() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(select({"a":1}).select)"#,
        Err(r#"'select' value has no field or method 'select'"#),
    ),
    (
        r#"print(depset([select({"a":1})]))"#,
        Err(r#"depset elements must not be mutable values"#),
    ),
    (
        r#"print(depset(select({"a":[1]})))"#,
        Err(
            r#"in call to depset(), parameter 'direct' got value of type 'select', want 'sequence or NoneType'"#,
        ),
    ),
    (
        r#"print(select({"a":1}, no_match_error=""))"#,
        Ok(r#"select({"a": 1})"#),
    ),
    (
        r#"print(select({"a":1}, no_match_error="a" + "b"))"#,
        Ok(r#"select({"a": 1})"#),
    ),
    (
        r#"r=1
d={"a":1}
s=select(d)
d["b"]=2
print(s)"#,
        Ok(r#"select({"a": 1})"#),
    ),
    (
        r#"r=1
d={"a":[1]}
s=select(d)
d["a"].append(2)
print(s)"#,
        Ok(r#"select({"a": [1, 2]})"#),
    ),
    (
        r#"r=1
print(select({Label("//x:y"):1}))"#,
        Ok(r#"select({Label("//x:y"): 1})"#),
    ),
    (
        r#"r=1
print(select({Label("//x:y"):1, "//x:y":2}))"#,
        Ok(r#"select({Label("//x:y"): 1, "//x:y": 2})"#),
    ),
    (
        r#"r=1
print(select({Label("@foo//x:y"):1}))"#,
        Ok(r#"select({Label("@@[unknown repo 'foo' requested from @@]//x:y"): 1})"#),
    ),
    (
        r#"r=1
print(select({Label(":y"):1}))"#,
        Ok(r#"select({Label("//:y"): 1})"#),
    ),
    (
        r#"r=1
print(select({struct():1}))"#,
        Err(r#"select: got struct for dict key, want a Label or label string"#),
    ),
    (
        r#"r=1
print(select({1.5:1}))"#,
        Err(r#"select: got float for dict key, want a Label or label string"#),
    ),
    (
        r#"r=1
print(select({None:1}))"#,
        Err(r#"select: got NoneType for dict key, want a Label or label string"#),
    ),
    (
        r#"r=1
print(select({("a",):1}))"#,
        Err(r#"select: got tuple for dict key, want a Label or label string"#),
    ),
    (
        r#"r=1
print(struct(x=select({"a":[1]})))"#,
        Ok(r#"struct(x = select({"a": [1]}))"#),
    ),
    (
        r#"r=1
print(str(struct(x=select({"a":[1]}))))"#,
        Ok(r#"struct(x = select({"a": [1]}))"#),
    ),
    (
        r#"r=1
print(json.encode(struct(x=select({"a":[1]}))))"#,
        Err(r#"in struct field .x: cannot encode select as JSON"#),
    ),
    (
        r#"r=1
print(native.select)"#,
        Err(r#"no native function or rule 'select'"#),
    ),
    (
        r#"r=1
print(native.select({"a":1}))"#,
        Err(r#"no native function or rule 'select'"#),
    ),
    (
        r#"r=1
print(dir(native))"#,
        Ok(
            r#"["action_listener", "alias", "cc_binary", "cc_import", "cc_libc_top_alias", "cc_library", "cc_shared_library", "cc_static_library", "cc_test", "cc_toolchain", "cc_toolchain_alias", "cc_toolchain_suite", "config_feature_flag", "config_setting", "constraint_setting", "constraint_value", "environment", "existing_rule", "existing_rules", "exports_files", "extra_action", "fdo_prefetch_hints", "fdo_profile", "filegroup", "genquery", "genrule", "glob", "java_binary", "java_import", "java_library", "java_package_configuration", "java_plugin", "java_plugins_flag_alias", "java_runtime", "java_test", "java_toolchain", "label_flag", "label_setting", "legacy_globals", "memprof_profile", "module_name", "module_version", "objc_import", "objc_library", "package", "package_default_visibility", "package_group", "package_name", "package_relative_label", "platform", "propeller_optimize", "repo_name", "repository_name", "starlark_doc_extract", "subpackages", "test_suite", "toolchain", "toolchain_type"]"#,
        ),
    ),
    (
        r#"r=1
def f():
  return select({"a":1})
print(f())"#,
        Ok(r#"select({"a": 1})"#),
    ),
    (
        r#"r=1
def f(x):
  return select({"a":x})
print(f(1))"#,
        Ok(r#"select({"a": 1})"#),
    ),
    (
        r#"r=1
s=select({"a":1})
def f(): return s
print(f())"#,
        Ok(r#"select({"a": 1})"#),
    ),
    (
        r#"r=1
s=select({"a":[1]})
t = s + [2]
print(t)
print(s)"#,
        Ok(r#"select({"a": [1]}) + [2]
select({"a": [1]})"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + struct())"#,
        Err(r#"Cannot combine incompatible types (select of int, struct)"#),
    ),
    (
        r#"r=1
print(struct() + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (struct, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + struct())"#,
        Ok(r#"select({"a": struct()}) + struct()"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + select({"a":struct()}))"#,
        Ok(r#"select({"a": struct()}) + select({"a": struct()})"#),
    ),
    (
        r#"r=1
print(select({"a":1}, no_match_error=None))"#,
        Err(
            r#"in call to select(), parameter 'no_match_error' got value of type 'NoneType', want 'string'"#,
        ),
    ),
    (
        r#"r=1
print(select({"a":[1]}, "err") + select({"b":[1]}, "err2"))"#,
        Ok(r#"select({"a": [1]}) + select({"b": [1]})"#),
    ),
    (
        r#"r=1
s=select({"a":[1]}) + select({"b":[2]})
print(type(s))
print(s == s)
print(s == select({"a":[1]}) + select({"b":[2]}))"#,
        Ok(r#"select
True
True"#),
    ),
    (
        r#"r=1
print(select({"a":1}) == select({"a":1}))
print(select({"a":1}) == select({"a":1.0}))"#,
        Ok(r#"True
False"#),
    ),
    (
        r#"r=1
print(select({"a":1, "b":2}) == select({"b":2, "a":1}))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
print(select({"a":1, "b":2}))"#,
        Ok(r#"select({"a": 1, "b": 2})"#),
    ),
    (
        r#"r=1
print(select({"b":1, "a":2, "//conditions:default":3}))"#,
        Ok(r#"select({"b": 1, "a": 2, "//conditions:default": 3})"#),
    ),
    (
        r#"r=1
print(select({"//conditions:default":3, "a":2}) )"#,
        Ok(r#"select({"//conditions:default": 3, "a": 2})"#),
    ),
    (
        r#"r=1
print(repr(select({"a":1}) + [1]))"#,
        Err(r#"Cannot combine incompatible types (select of int, list)"#),
    ),
    (
        r#"r=1
print(str(select({"a":1})) + "x")"#,
        Ok(r#"select({"a": 1})x"#),
    ),
    (
        r#"r=1
print("%s" % select({"a":1}))"#,
        Ok(r#"select({"a": 1})"#),
    ),
    (
        r#"r=1
print("%r" % select({"a":1}))"#,
        Ok(r#"select({"a": 1})"#),
    ),
    (
        r#"r=1
print("{}".format(select({"a":1})))"#,
        Ok(r#"select({"a": 1})"#),
    ),
    (
        r#"r=1
print(select({"a":1}).__class__)"#,
        Err(r#"'select' value has no field or method '__class__'"#),
    ),
    (
        r#"r=1
print(hasattr(select({"a":1}), "x"))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
print(getattr(select({"a":1}), "x", 5))"#,
        Ok(r#"5"#),
    ),
    (
        r#"r=1
print(dir(select({"a":1}) + [1]))"#,
        Err(r#"Cannot combine incompatible types (select of int, list)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) in [1])"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
print("a" in select({"a":{"k":1}}))"#,
        Err(r#"unsupported binary operation: string in select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"b":[1]}) + [2] + select({"c":[1]}))"#,
        Ok(r#"select({"a": [1]}) + select({"b": [1]}) + [2] + select({"c": [1]})"#),
    ),
    (
        r#"r=1
print(([1] + select({"a":[1]})) + ([2] + select({"b":[1]})))"#,
        Ok(r#"[1] + select({"a": [1]}) + [2] + select({"b": [1]})"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"b":[1]}) == select({"b":[1]}) + select({"a":[1]}))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
print(((1,) + select({"a":[1]})))"#,
        Ok(r#"(1,) + select({"a": [1]})"#),
    ),
    (
        r#"r=1
print(((1,) + select({"a":(1,)})))"#,
        Ok(r#"(1,) + select({"a": (1,)})"#),
    ),
    (
        r#"r=1
print(list((1,)) + select({"a":(1,)}))"#,
        Ok(r#"[1] + select({"a": (1,)})"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + "t" + select({"b":"u"}))"#,
        Ok(r#"select({"a": "s"}) + "t" + select({"b": "u"})"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + 1 + select({"b":2}))"#,
        Ok(r#"select({"a": 1}) + 1 + select({"b": 2})"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + [1] + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + [1] + select({"b":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"b":[1]}) + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"b":{}}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print({} + select({"a":[1]}) + [1])"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print([1] + {} + select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: list + dict"#),
    ),
    (
        r#"r=1
print(1 + select({"a":1}))"#,
        Ok(r#"1 + select({"a": 1})"#),
    ),
    (
        r#"r=1
print(1 + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (int, select of string)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (int, select of list)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (int, select of tuple)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (int, select of dict)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (int, select of NoneType)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (int, select of bool)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (int, select of float)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (int, select of depset)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (int, select of range)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (int, select of struct)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (int, select of select)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (string, select of int)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":"s"}))"#,
        Ok(r#""s" + select({"a": "s"})"#),
    ),
    (
        r#"r=1
print("s" + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (string, select of list)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (string, select of tuple)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (string, select of dict)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (string, select of NoneType)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (string, select of bool)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (string, select of float)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (string, select of depset)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (string, select of range)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (string, select of struct)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (string, select of select)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (list, select of int)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (list, select of string)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":[1]}))"#,
        Ok(r#"[1] + select({"a": [1]})"#),
    ),
    (
        r#"r=1
print([1] + select({"a":(1,)}))"#,
        Ok(r#"[1] + select({"a": (1,)})"#),
    ),
    (
        r#"r=1
print([1] + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (list, select of dict)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (list, select of NoneType)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (list, select of bool)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (list, select of float)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (list, select of depset)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":range(2)}))"#,
        Ok(r#"[1] + select({"a": range(0, 2)})"#),
    ),
    (
        r#"r=1
print([1] + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (list, select of struct)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (list, select of select)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of int)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of string)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":[1]}))"#,
        Ok(r#"(1,) + select({"a": [1]})"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":(1,)}))"#,
        Ok(r#"(1,) + select({"a": (1,)})"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of dict)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of NoneType)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of bool)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of float)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of depset)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":range(2)}))"#,
        Ok(r#"(1,) + select({"a": range(0, 2)})"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of struct)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of select)"#),
    ),
    (
        r#"r=1
print({} + select({"a":1}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print({} + select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print({} + select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print({} + select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print({} + select({"a":{}}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print({} + select({"a":None}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print({} + select({"a":True}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print({} + select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print({} + select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print({} + select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print({} + select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print({} + select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print(None + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (NoneType, select of int)"#),
    ),
    (
        r#"r=1
print(None + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (NoneType, select of string)"#),
    ),
    (
        r#"r=1
print(None + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (NoneType, select of list)"#),
    ),
    (
        r#"r=1
print(None + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (NoneType, select of tuple)"#),
    ),
    (
        r#"r=1
print(None + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (NoneType, select of dict)"#),
    ),
    (
        r#"r=1
print(None + select({"a":None}))"#,
        Ok(r#"None + select({"a": None})"#),
    ),
    (
        r#"r=1
print(None + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (NoneType, select of bool)"#),
    ),
    (
        r#"r=1
print(None + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (NoneType, select of float)"#),
    ),
    (
        r#"r=1
print(None + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (NoneType, select of depset)"#),
    ),
    (
        r#"r=1
print(None + select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (NoneType, select of range)"#),
    ),
    (
        r#"r=1
print(None + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (NoneType, select of struct)"#),
    ),
    (
        r#"r=1
print(None + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (NoneType, select of select)"#),
    ),
    (
        r#"r=1
print(True + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (bool, select of int)"#),
    ),
    (
        r#"r=1
print(True + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (bool, select of string)"#),
    ),
    (
        r#"r=1
print(True + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (bool, select of list)"#),
    ),
    (
        r#"r=1
print(True + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (bool, select of tuple)"#),
    ),
    (
        r#"r=1
print(True + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (bool, select of dict)"#),
    ),
    (
        r#"r=1
print(True + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (bool, select of NoneType)"#),
    ),
    (
        r#"r=1
print(True + select({"a":True}))"#,
        Ok(r#"True + select({"a": True})"#),
    ),
    (
        r#"r=1
print(True + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (bool, select of float)"#),
    ),
    (
        r#"r=1
print(True + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (bool, select of depset)"#),
    ),
    (
        r#"r=1
print(True + select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (bool, select of range)"#),
    ),
    (
        r#"r=1
print(True + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (bool, select of struct)"#),
    ),
    (
        r#"r=1
print(True + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (bool, select of select)"#),
    ),
    (
        r#"r=1
print(1.5 + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (float, select of int)"#),
    ),
    (
        r#"r=1
print(1.5 + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (float, select of string)"#),
    ),
    (
        r#"r=1
print(1.5 + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (float, select of list)"#),
    ),
    (
        r#"r=1
print(1.5 + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (float, select of tuple)"#),
    ),
    (
        r#"r=1
print(1.5 + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (float, select of dict)"#),
    ),
    (
        r#"r=1
print(1.5 + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (float, select of NoneType)"#),
    ),
    (
        r#"r=1
print(1.5 + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (float, select of bool)"#),
    ),
    (
        r#"r=1
print(1.5 + select({"a":1.5}))"#,
        Ok(r#"1.5 + select({"a": 1.5})"#),
    ),
    (
        r#"r=1
print(1.5 + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (float, select of depset)"#),
    ),
    (
        r#"r=1
print(1.5 + select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (float, select of range)"#),
    ),
    (
        r#"r=1
print(1.5 + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (float, select of struct)"#),
    ),
    (
        r#"r=1
print(1.5 + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (float, select of select)"#),
    ),
    (
        r#"r=1
print(depset([1]) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (depset, select of int)"#),
    ),
    (
        r#"r=1
print(depset([1]) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (depset, select of string)"#),
    ),
    (
        r#"r=1
print(depset([1]) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (depset, select of list)"#),
    ),
    (
        r#"r=1
print(depset([1]) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (depset, select of tuple)"#),
    ),
    (
        r#"r=1
print(depset([1]) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (depset, select of dict)"#),
    ),
    (
        r#"r=1
print(depset([1]) + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (depset, select of NoneType)"#),
    ),
    (
        r#"r=1
print(depset([1]) + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (depset, select of bool)"#),
    ),
    (
        r#"r=1
print(depset([1]) + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (depset, select of float)"#),
    ),
    (
        r#"r=1
print(depset([1]) + select({"a":depset([1])}))"#,
        Ok(r#"depset([1]) + select({"a": depset([1])})"#),
    ),
    (
        r#"r=1
print(depset([1]) + select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (depset, select of range)"#),
    ),
    (
        r#"r=1
print(depset([1]) + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (depset, select of struct)"#),
    ),
    (
        r#"r=1
print(depset([1]) + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (depset, select of select)"#),
    ),
    (
        r#"r=1
print(range(2) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (range, select of int)"#),
    ),
    (
        r#"r=1
print(range(2) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (range, select of string)"#),
    ),
    (
        r#"r=1
print(range(2) + select({"a":[1]}))"#,
        Ok(r#"range(0, 2) + select({"a": [1]})"#),
    ),
    (
        r#"r=1
print(range(2) + select({"a":(1,)}))"#,
        Ok(r#"range(0, 2) + select({"a": (1,)})"#),
    ),
    (
        r#"r=1
print(range(2) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (range, select of dict)"#),
    ),
    (
        r#"r=1
print(range(2) + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (range, select of NoneType)"#),
    ),
    (
        r#"r=1
print(range(2) + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (range, select of bool)"#),
    ),
    (
        r#"r=1
print(range(2) + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (range, select of float)"#),
    ),
    (
        r#"r=1
print(range(2) + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (range, select of depset)"#),
    ),
    (
        r#"r=1
print(range(2) + select({"a":range(2)}))"#,
        Ok(r#"range(0, 2) + select({"a": range(0, 2)})"#),
    ),
    (
        r#"r=1
print(range(2) + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (range, select of struct)"#),
    ),
    (
        r#"r=1
print(range(2) + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (range, select of select)"#),
    ),
    (
        r#"r=1
print(struct() + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (struct, select of string)"#),
    ),
    (
        r#"r=1
print(struct() + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (struct, select of list)"#),
    ),
    (
        r#"r=1
print(struct() + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (struct, select of tuple)"#),
    ),
    (
        r#"r=1
print(struct() + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (struct, select of dict)"#),
    ),
    (
        r#"r=1
print(struct() + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (struct, select of NoneType)"#),
    ),
    (
        r#"r=1
print(struct() + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (struct, select of bool)"#),
    ),
    (
        r#"r=1
print(struct() + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (struct, select of float)"#),
    ),
    (
        r#"r=1
print(struct() + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (struct, select of depset)"#),
    ),
    (
        r#"r=1
print(struct() + select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (struct, select of range)"#),
    ),
    (
        r#"r=1
print(struct() + select({"a":struct()}))"#,
        Ok(r#"struct() + select({"a": struct()})"#),
    ),
    (
        r#"r=1
print(struct() + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (struct, select of select)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + 1)"#,
        Ok(r#"select({"a": 1}) + 1"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of int, string)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of int, list)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of int, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + None)"#,
        Err(r#"Cannot combine incompatible types (select of int, NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + True)"#,
        Err(r#"Cannot combine incompatible types (select of int, bool)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + 1.5)"#,
        Err(r#"Cannot combine incompatible types (select of int, float)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + depset([1]))"#,
        Err(r#"Cannot combine incompatible types (select of int, depset)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + range(2))"#,
        Err(r#"Cannot combine incompatible types (select of int, range)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":1}))"#,
        Ok(r#"select({"a": 1}) + select({"a": 1})"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of bool)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of float)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of depset)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of range)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of struct)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of select)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of string, int)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + "s")"#,
        Ok(r#"select({"a": "s"}) + "s""#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of string, list)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of string, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + None)"#,
        Err(r#"Cannot combine incompatible types (select of string, NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + True)"#,
        Err(r#"Cannot combine incompatible types (select of string, bool)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + 1.5)"#,
        Err(r#"Cannot combine incompatible types (select of string, float)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + depset([1]))"#,
        Err(r#"Cannot combine incompatible types (select of string, depset)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + range(2))"#,
        Err(r#"Cannot combine incompatible types (select of string, range)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + struct())"#,
        Err(r#"Cannot combine incompatible types (select of string, struct)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":"s"}))"#,
        Ok(r#"select({"a": "s"}) + select({"a": "s"})"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of bool)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of float)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of depset)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of range)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of struct)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of select)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of list, int)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of list, string)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + [1])"#,
        Ok(r#"select({"a": [1]}) + [1]"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + (1,))"#,
        Ok(r#"select({"a": [1]}) + (1,)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + None)"#,
        Err(r#"Cannot combine incompatible types (select of list, NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + True)"#,
        Err(r#"Cannot combine incompatible types (select of list, bool)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + 1.5)"#,
        Err(r#"Cannot combine incompatible types (select of list, float)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + depset([1]))"#,
        Err(r#"Cannot combine incompatible types (select of list, depset)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + range(2))"#,
        Ok(r#"select({"a": [1]}) + range(0, 2)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + struct())"#,
        Err(r#"Cannot combine incompatible types (select of list, struct)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":[1]}))"#,
        Ok(r#"select({"a": [1]}) + select({"a": [1]})"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":(1,)}))"#,
        Ok(r#"select({"a": [1]}) + select({"a": (1,)})"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of bool)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of float)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of depset)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":range(2)}))"#,
        Ok(r#"select({"a": [1]}) + select({"a": range(0, 2)})"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of struct)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of select)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of tuple, int)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of tuple, string)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + [1])"#,
        Ok(r#"select({"a": (1,)}) + [1]"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + (1,))"#,
        Ok(r#"select({"a": (1,)}) + (1,)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + None)"#,
        Err(r#"Cannot combine incompatible types (select of tuple, NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + True)"#,
        Err(r#"Cannot combine incompatible types (select of tuple, bool)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + 1.5)"#,
        Err(r#"Cannot combine incompatible types (select of tuple, float)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + depset([1]))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, depset)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + range(2))"#,
        Ok(r#"select({"a": (1,)}) + range(0, 2)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + struct())"#,
        Err(r#"Cannot combine incompatible types (select of tuple, struct)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":[1]}))"#,
        Ok(r#"select({"a": (1,)}) + select({"a": [1]})"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":(1,)}))"#,
        Ok(r#"select({"a": (1,)}) + select({"a": (1,)})"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of bool)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of float)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of depset)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":range(2)}))"#,
        Ok(r#"select({"a": (1,)}) + select({"a": range(0, 2)})"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of struct)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of select)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of dict, int)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of dict, string)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of dict, list)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of dict, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + None)"#,
        Err(r#"Cannot combine incompatible types (select of dict, NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + True)"#,
        Err(r#"Cannot combine incompatible types (select of dict, bool)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + 1.5)"#,
        Err(r#"Cannot combine incompatible types (select of dict, float)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + depset([1]))"#,
        Err(r#"Cannot combine incompatible types (select of dict, depset)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + range(2))"#,
        Err(r#"Cannot combine incompatible types (select of dict, range)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + struct())"#,
        Err(r#"Cannot combine incompatible types (select of dict, struct)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":{}}))"#,
        Err(r#"unsupported binary operation: select + select"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of bool)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of float)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of depset)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of range)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of struct)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of select)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, int)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, string)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, list)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + None)"#,
        Ok(r#"select({"a": None}) + None"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + True)"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, bool)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + 1.5)"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, float)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + depset([1]))"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, depset)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + range(2))"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, range)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + struct())"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, struct)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + select({"a":None}))"#,
        Ok(r#"select({"a": None}) + select({"a": None})"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, select of bool)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, select of float)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, select of depset)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, select of range)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, select of struct)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, select of select)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of bool, int)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of bool, string)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of bool, list)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of bool, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + None)"#,
        Err(r#"Cannot combine incompatible types (select of bool, NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + True)"#,
        Ok(r#"select({"a": True}) + True"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + 1.5)"#,
        Err(r#"Cannot combine incompatible types (select of bool, float)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + depset([1]))"#,
        Err(r#"Cannot combine incompatible types (select of bool, depset)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + range(2))"#,
        Err(r#"Cannot combine incompatible types (select of bool, range)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + struct())"#,
        Err(r#"Cannot combine incompatible types (select of bool, struct)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of bool, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of bool, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of bool, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of bool, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of bool, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (select of bool, select of NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + select({"a":True}))"#,
        Ok(r#"select({"a": True}) + select({"a": True})"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (select of bool, select of float)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (select of bool, select of depset)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (select of bool, select of range)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (select of bool, select of struct)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (select of bool, select of select)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of float, int)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of float, string)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of float, list)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of float, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + None)"#,
        Err(r#"Cannot combine incompatible types (select of float, NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + True)"#,
        Err(r#"Cannot combine incompatible types (select of float, bool)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + 1.5)"#,
        Ok(r#"select({"a": 1.5}) + 1.5"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + depset([1]))"#,
        Err(r#"Cannot combine incompatible types (select of float, depset)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + range(2))"#,
        Err(r#"Cannot combine incompatible types (select of float, range)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + struct())"#,
        Err(r#"Cannot combine incompatible types (select of float, struct)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of float, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of float, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of float, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of float, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of float, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (select of float, select of NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (select of float, select of bool)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + select({"a":1.5}))"#,
        Ok(r#"select({"a": 1.5}) + select({"a": 1.5})"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (select of float, select of depset)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (select of float, select of range)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (select of float, select of struct)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (select of float, select of select)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of depset, int)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of depset, string)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of depset, list)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of depset, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + None)"#,
        Err(r#"Cannot combine incompatible types (select of depset, NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + True)"#,
        Err(r#"Cannot combine incompatible types (select of depset, bool)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + 1.5)"#,
        Err(r#"Cannot combine incompatible types (select of depset, float)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + depset([1]))"#,
        Ok(r#"select({"a": depset([1])}) + depset([1])"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + range(2))"#,
        Err(r#"Cannot combine incompatible types (select of depset, range)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + struct())"#,
        Err(r#"Cannot combine incompatible types (select of depset, struct)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of depset, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of depset, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of depset, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of depset, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of depset, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (select of depset, select of NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (select of depset, select of bool)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (select of depset, select of float)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + select({"a":depset([1])}))"#,
        Ok(r#"select({"a": depset([1])}) + select({"a": depset([1])})"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (select of depset, select of range)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (select of depset, select of struct)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (select of depset, select of select)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of range, int)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of range, string)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + [1])"#,
        Ok(r#"select({"a": range(0, 2)}) + [1]"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + (1,))"#,
        Ok(r#"select({"a": range(0, 2)}) + (1,)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + None)"#,
        Err(r#"Cannot combine incompatible types (select of range, NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + True)"#,
        Err(r#"Cannot combine incompatible types (select of range, bool)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + 1.5)"#,
        Err(r#"Cannot combine incompatible types (select of range, float)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + depset([1]))"#,
        Err(r#"Cannot combine incompatible types (select of range, depset)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + range(2))"#,
        Ok(r#"select({"a": range(0, 2)}) + range(0, 2)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + struct())"#,
        Err(r#"Cannot combine incompatible types (select of range, struct)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of range, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of range, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + select({"a":[1]}))"#,
        Ok(r#"select({"a": range(0, 2)}) + select({"a": [1]})"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + select({"a":(1,)}))"#,
        Ok(r#"select({"a": range(0, 2)}) + select({"a": (1,)})"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of range, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (select of range, select of NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (select of range, select of bool)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (select of range, select of float)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (select of range, select of depset)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + select({"a":range(2)}))"#,
        Ok(r#"select({"a": range(0, 2)}) + select({"a": range(0, 2)})"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (select of range, select of struct)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (select of range, select of select)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of struct, int)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of struct, string)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of struct, list)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of struct, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + None)"#,
        Err(r#"Cannot combine incompatible types (select of struct, NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + True)"#,
        Err(r#"Cannot combine incompatible types (select of struct, bool)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + 1.5)"#,
        Err(r#"Cannot combine incompatible types (select of struct, float)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + depset([1]))"#,
        Err(r#"Cannot combine incompatible types (select of struct, depset)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + range(2))"#,
        Err(r#"Cannot combine incompatible types (select of struct, range)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of struct, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of struct, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of struct, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of struct, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of struct, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (select of struct, select of NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (select of struct, select of bool)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (select of struct, select of float)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (select of struct, select of depset)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (select of struct, select of range)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) + select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (select of struct, select of select)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of select, int)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of select, string)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of select, list)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of select, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + None)"#,
        Err(r#"Cannot combine incompatible types (select of select, NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + True)"#,
        Err(r#"Cannot combine incompatible types (select of select, bool)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + 1.5)"#,
        Err(r#"Cannot combine incompatible types (select of select, float)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + depset([1]))"#,
        Err(r#"Cannot combine incompatible types (select of select, depset)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + range(2))"#,
        Err(r#"Cannot combine incompatible types (select of select, range)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + struct())"#,
        Err(r#"Cannot combine incompatible types (select of select, struct)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of select, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of select, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of select, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of select, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of select, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (select of select, select of NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (select of select, select of bool)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (select of select, select of float)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (select of select, select of depset)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (select of select, select of range)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (select of select, select of struct)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) + select({"a":select({"b":1})}))"#,
        Ok(r#"select({"a": select({"b": 1})}) + select({"a": select({"b": 1})})"#),
    ),
    (
        r#"r=1
print(1 | select({"a":1}))"#,
        Err(r#"unsupported binary operation: int | select"#),
    ),
    (
        r#"r=1
print(1 | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: int | select"#),
    ),
    (
        r#"r=1
print(1 | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: int | select"#),
    ),
    (
        r#"r=1
print(1 | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: int | select"#),
    ),
    (
        r#"r=1
print(1 | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: int | select"#),
    ),
    (
        r#"r=1
print(1 | select({"a":None}))"#,
        Err(r#"unsupported binary operation: int | select"#),
    ),
    (
        r#"r=1
print(1 | select({"a":True}))"#,
        Err(r#"unsupported binary operation: int | select"#),
    ),
    (
        r#"r=1
print(1 | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: int | select"#),
    ),
    (
        r#"r=1
print(1 | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: int | select"#),
    ),
    (
        r#"r=1
print(1 | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: int | select"#),
    ),
    (
        r#"r=1
print(1 | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: int | select"#),
    ),
    (
        r#"r=1
print(1 | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: int | select"#),
    ),
    (
        r#"r=1
print("s" | select({"a":1}))"#,
        Err(r#"unsupported binary operation: string | select"#),
    ),
    (
        r#"r=1
print("s" | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: string | select"#),
    ),
    (
        r#"r=1
print("s" | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: string | select"#),
    ),
    (
        r#"r=1
print("s" | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: string | select"#),
    ),
    (
        r#"r=1
print("s" | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: string | select"#),
    ),
    (
        r#"r=1
print("s" | select({"a":None}))"#,
        Err(r#"unsupported binary operation: string | select"#),
    ),
    (
        r#"r=1
print("s" | select({"a":True}))"#,
        Err(r#"unsupported binary operation: string | select"#),
    ),
    (
        r#"r=1
print("s" | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: string | select"#),
    ),
    (
        r#"r=1
print("s" | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: string | select"#),
    ),
    (
        r#"r=1
print("s" | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: string | select"#),
    ),
    (
        r#"r=1
print("s" | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: string | select"#),
    ),
    (
        r#"r=1
print("s" | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: string | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":1}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":None}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":True}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print((1,) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: tuple | select"#),
    ),
    (
        r#"r=1
print((1,) | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: tuple | select"#),
    ),
    (
        r#"r=1
print((1,) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: tuple | select"#),
    ),
    (
        r#"r=1
print((1,) | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: tuple | select"#),
    ),
    (
        r#"r=1
print((1,) | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: tuple | select"#),
    ),
    (
        r#"r=1
print((1,) | select({"a":None}))"#,
        Err(r#"unsupported binary operation: tuple | select"#),
    ),
    (
        r#"r=1
print((1,) | select({"a":True}))"#,
        Err(r#"unsupported binary operation: tuple | select"#),
    ),
    (
        r#"r=1
print((1,) | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: tuple | select"#),
    ),
    (
        r#"r=1
print((1,) | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: tuple | select"#),
    ),
    (
        r#"r=1
print((1,) | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: tuple | select"#),
    ),
    (
        r#"r=1
print((1,) | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: tuple | select"#),
    ),
    (
        r#"r=1
print((1,) | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: tuple | select"#),
    ),
    (
        r#"r=1
print({} | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of int)"#),
    ),
    (
        r#"r=1
print({} | select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of string)"#),
    ),
    (
        r#"r=1
print({} | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of list)"#),
    ),
    (
        r#"r=1
print({} | select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of tuple)"#),
    ),
    (
        r#"r=1
print({} | select({"a":{}}))"#,
        Ok(r#"{} | select({"a": {}})"#),
    ),
    (
        r#"r=1
print({} | select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of NoneType)"#),
    ),
    (
        r#"r=1
print({} | select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of bool)"#),
    ),
    (
        r#"r=1
print({} | select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of float)"#),
    ),
    (
        r#"r=1
print({} | select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of depset)"#),
    ),
    (
        r#"r=1
print({} | select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of range)"#),
    ),
    (
        r#"r=1
print({} | select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of struct)"#),
    ),
    (
        r#"r=1
print({} | select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of select)"#),
    ),
    (
        r#"r=1
print(None | select({"a":1}))"#,
        Err(r#"unsupported binary operation: NoneType | select"#),
    ),
    (
        r#"r=1
print(None | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: NoneType | select"#),
    ),
    (
        r#"r=1
print(None | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: NoneType | select"#),
    ),
    (
        r#"r=1
print(None | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: NoneType | select"#),
    ),
    (
        r#"r=1
print(None | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: NoneType | select"#),
    ),
    (
        r#"r=1
print(None | select({"a":None}))"#,
        Err(r#"unsupported binary operation: NoneType | select"#),
    ),
    (
        r#"r=1
print(None | select({"a":True}))"#,
        Err(r#"unsupported binary operation: NoneType | select"#),
    ),
    (
        r#"r=1
print(None | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: NoneType | select"#),
    ),
    (
        r#"r=1
print(None | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: NoneType | select"#),
    ),
    (
        r#"r=1
print(None | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: NoneType | select"#),
    ),
    (
        r#"r=1
print(None | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: NoneType | select"#),
    ),
    (
        r#"r=1
print(None | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: NoneType | select"#),
    ),
    (
        r#"r=1
print(True | select({"a":1}))"#,
        Err(r#"unsupported binary operation: bool | select"#),
    ),
    (
        r#"r=1
print(True | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: bool | select"#),
    ),
    (
        r#"r=1
print(True | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: bool | select"#),
    ),
    (
        r#"r=1
print(True | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: bool | select"#),
    ),
    (
        r#"r=1
print(True | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: bool | select"#),
    ),
    (
        r#"r=1
print(True | select({"a":None}))"#,
        Err(r#"unsupported binary operation: bool | select"#),
    ),
    (
        r#"r=1
print(True | select({"a":True}))"#,
        Err(r#"unsupported binary operation: bool | select"#),
    ),
    (
        r#"r=1
print(True | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: bool | select"#),
    ),
    (
        r#"r=1
print(True | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: bool | select"#),
    ),
    (
        r#"r=1
print(True | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: bool | select"#),
    ),
    (
        r#"r=1
print(True | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: bool | select"#),
    ),
    (
        r#"r=1
print(True | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: bool | select"#),
    ),
    (
        r#"r=1
print(1.5 | select({"a":1}))"#,
        Err(r#"unsupported binary operation: float | select"#),
    ),
    (
        r#"r=1
print(1.5 | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: float | select"#),
    ),
    (
        r#"r=1
print(1.5 | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: float | select"#),
    ),
    (
        r#"r=1
print(1.5 | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: float | select"#),
    ),
    (
        r#"r=1
print(1.5 | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: float | select"#),
    ),
    (
        r#"r=1
print(1.5 | select({"a":None}))"#,
        Err(r#"unsupported binary operation: float | select"#),
    ),
    (
        r#"r=1
print(1.5 | select({"a":True}))"#,
        Err(r#"unsupported binary operation: float | select"#),
    ),
    (
        r#"r=1
print(1.5 | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: float | select"#),
    ),
    (
        r#"r=1
print(1.5 | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: float | select"#),
    ),
    (
        r#"r=1
print(1.5 | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: float | select"#),
    ),
    (
        r#"r=1
print(1.5 | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: float | select"#),
    ),
    (
        r#"r=1
print(1.5 | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: float | select"#),
    ),
    (
        r#"r=1
print(depset([1]) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: depset | select"#),
    ),
    (
        r#"r=1
print(depset([1]) | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: depset | select"#),
    ),
    (
        r#"r=1
print(depset([1]) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: depset | select"#),
    ),
    (
        r#"r=1
print(depset([1]) | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: depset | select"#),
    ),
    (
        r#"r=1
print(depset([1]) | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: depset | select"#),
    ),
    (
        r#"r=1
print(depset([1]) | select({"a":None}))"#,
        Err(r#"unsupported binary operation: depset | select"#),
    ),
    (
        r#"r=1
print(depset([1]) | select({"a":True}))"#,
        Err(r#"unsupported binary operation: depset | select"#),
    ),
    (
        r#"r=1
print(depset([1]) | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: depset | select"#),
    ),
    (
        r#"r=1
print(depset([1]) | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: depset | select"#),
    ),
    (
        r#"r=1
print(depset([1]) | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: depset | select"#),
    ),
    (
        r#"r=1
print(depset([1]) | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: depset | select"#),
    ),
    (
        r#"r=1
print(depset([1]) | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: depset | select"#),
    ),
    (
        r#"r=1
print(range(2) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: range | select"#),
    ),
    (
        r#"r=1
print(range(2) | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: range | select"#),
    ),
    (
        r#"r=1
print(range(2) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: range | select"#),
    ),
    (
        r#"r=1
print(range(2) | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: range | select"#),
    ),
    (
        r#"r=1
print(range(2) | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: range | select"#),
    ),
    (
        r#"r=1
print(range(2) | select({"a":None}))"#,
        Err(r#"unsupported binary operation: range | select"#),
    ),
    (
        r#"r=1
print(range(2) | select({"a":True}))"#,
        Err(r#"unsupported binary operation: range | select"#),
    ),
    (
        r#"r=1
print(range(2) | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: range | select"#),
    ),
    (
        r#"r=1
print(range(2) | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: range | select"#),
    ),
    (
        r#"r=1
print(range(2) | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: range | select"#),
    ),
    (
        r#"r=1
print(range(2) | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: range | select"#),
    ),
    (
        r#"r=1
print(range(2) | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: range | select"#),
    ),
    (
        r#"r=1
print(struct() | select({"a":1}))"#,
        Err(r#"unsupported binary operation: struct | select"#),
    ),
    (
        r#"r=1
print(struct() | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: struct | select"#),
    ),
    (
        r#"r=1
print(struct() | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: struct | select"#),
    ),
    (
        r#"r=1
print(struct() | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: struct | select"#),
    ),
    (
        r#"r=1
print(struct() | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: struct | select"#),
    ),
    (
        r#"r=1
print(struct() | select({"a":None}))"#,
        Err(r#"unsupported binary operation: struct | select"#),
    ),
    (
        r#"r=1
print(struct() | select({"a":True}))"#,
        Err(r#"unsupported binary operation: struct | select"#),
    ),
    (
        r#"r=1
print(struct() | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: struct | select"#),
    ),
    (
        r#"r=1
print(struct() | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: struct | select"#),
    ),
    (
        r#"r=1
print(struct() | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: struct | select"#),
    ),
    (
        r#"r=1
print(struct() | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: struct | select"#),
    ),
    (
        r#"r=1
print(struct() | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: struct | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | 1)"#,
        Err(r#"unsupported binary operation: select | int"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | "s")"#,
        Err(r#"unsupported binary operation: select | string"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | (1,))"#,
        Err(r#"unsupported binary operation: select | tuple"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of int, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | None)"#,
        Err(r#"unsupported binary operation: select | NoneType"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | True)"#,
        Err(r#"unsupported binary operation: select | bool"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | 1.5)"#,
        Err(r#"unsupported binary operation: select | float"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | depset([1]))"#,
        Err(r#"unsupported binary operation: select | depset"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | range(2))"#,
        Err(r#"unsupported binary operation: select | range"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | struct())"#,
        Err(r#"unsupported binary operation: select | struct"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":None}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":True}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | 1)"#,
        Err(r#"unsupported binary operation: select | int"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | "s")"#,
        Err(r#"unsupported binary operation: select | string"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | (1,))"#,
        Err(r#"unsupported binary operation: select | tuple"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of string, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | None)"#,
        Err(r#"unsupported binary operation: select | NoneType"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | True)"#,
        Err(r#"unsupported binary operation: select | bool"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | 1.5)"#,
        Err(r#"unsupported binary operation: select | float"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | depset([1]))"#,
        Err(r#"unsupported binary operation: select | depset"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | range(2))"#,
        Err(r#"unsupported binary operation: select | range"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | struct())"#,
        Err(r#"unsupported binary operation: select | struct"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | select({"a":None}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | select({"a":True}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | 1)"#,
        Err(r#"unsupported binary operation: select | int"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | "s")"#,
        Err(r#"unsupported binary operation: select | string"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | (1,))"#,
        Err(r#"unsupported binary operation: select | tuple"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of list, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | None)"#,
        Err(r#"unsupported binary operation: select | NoneType"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | True)"#,
        Err(r#"unsupported binary operation: select | bool"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | 1.5)"#,
        Err(r#"unsupported binary operation: select | float"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | depset([1]))"#,
        Err(r#"unsupported binary operation: select | depset"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | range(2))"#,
        Err(r#"unsupported binary operation: select | range"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | struct())"#,
        Err(r#"unsupported binary operation: select | struct"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":None}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":True}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | 1)"#,
        Err(r#"unsupported binary operation: select | int"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | "s")"#,
        Err(r#"unsupported binary operation: select | string"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | (1,))"#,
        Err(r#"unsupported binary operation: select | tuple"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of tuple, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | None)"#,
        Err(r#"unsupported binary operation: select | NoneType"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | True)"#,
        Err(r#"unsupported binary operation: select | bool"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | 1.5)"#,
        Err(r#"unsupported binary operation: select | float"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | depset([1]))"#,
        Err(r#"unsupported binary operation: select | depset"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | range(2))"#,
        Err(r#"unsupported binary operation: select | range"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | struct())"#,
        Err(r#"unsupported binary operation: select | struct"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | select({"a":None}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | select({"a":True}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | 1)"#,
        Err(r#"unsupported binary operation: select | int"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | "s")"#,
        Err(r#"unsupported binary operation: select | string"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | (1,))"#,
        Err(r#"unsupported binary operation: select | tuple"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | {})"#,
        Ok(r#"select({"a": {}}) | {}"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | None)"#,
        Err(r#"unsupported binary operation: select | NoneType"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | True)"#,
        Err(r#"unsupported binary operation: select | bool"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | 1.5)"#,
        Err(r#"unsupported binary operation: select | float"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | depset([1]))"#,
        Err(r#"unsupported binary operation: select | depset"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | range(2))"#,
        Err(r#"unsupported binary operation: select | range"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | struct())"#,
        Err(r#"unsupported binary operation: select | struct"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":{}}))"#,
        Ok(r#"select({"a": {}}) | select({"a": {}})"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":None}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of NoneType)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":True}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of bool)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":1.5}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of float)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":depset([1])}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of depset)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":range(2)}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of range)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":struct()}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of struct)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":select({"b":1})}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of select)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | 1)"#,
        Err(r#"unsupported binary operation: select | int"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | "s")"#,
        Err(r#"unsupported binary operation: select | string"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | (1,))"#,
        Err(r#"unsupported binary operation: select | tuple"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | None)"#,
        Err(r#"unsupported binary operation: select | NoneType"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | True)"#,
        Err(r#"unsupported binary operation: select | bool"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | 1.5)"#,
        Err(r#"unsupported binary operation: select | float"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | depset([1]))"#,
        Err(r#"unsupported binary operation: select | depset"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | range(2))"#,
        Err(r#"unsupported binary operation: select | range"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | struct())"#,
        Err(r#"unsupported binary operation: select | struct"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of NoneType, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | select({"a":None}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | select({"a":True}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":None}) | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | 1)"#,
        Err(r#"unsupported binary operation: select | int"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | "s")"#,
        Err(r#"unsupported binary operation: select | string"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | (1,))"#,
        Err(r#"unsupported binary operation: select | tuple"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of bool, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | None)"#,
        Err(r#"unsupported binary operation: select | NoneType"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | True)"#,
        Err(r#"unsupported binary operation: select | bool"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | 1.5)"#,
        Err(r#"unsupported binary operation: select | float"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | depset([1]))"#,
        Err(r#"unsupported binary operation: select | depset"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | range(2))"#,
        Err(r#"unsupported binary operation: select | range"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | struct())"#,
        Err(r#"unsupported binary operation: select | struct"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of bool, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | select({"a":None}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | select({"a":True}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":True}) | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | 1)"#,
        Err(r#"unsupported binary operation: select | int"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | "s")"#,
        Err(r#"unsupported binary operation: select | string"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | (1,))"#,
        Err(r#"unsupported binary operation: select | tuple"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of float, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | None)"#,
        Err(r#"unsupported binary operation: select | NoneType"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | True)"#,
        Err(r#"unsupported binary operation: select | bool"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | 1.5)"#,
        Err(r#"unsupported binary operation: select | float"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | depset([1]))"#,
        Err(r#"unsupported binary operation: select | depset"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | range(2))"#,
        Err(r#"unsupported binary operation: select | range"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | struct())"#,
        Err(r#"unsupported binary operation: select | struct"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of float, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | select({"a":None}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | select({"a":True}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1.5}) | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | 1)"#,
        Err(r#"unsupported binary operation: select | int"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | "s")"#,
        Err(r#"unsupported binary operation: select | string"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | (1,))"#,
        Err(r#"unsupported binary operation: select | tuple"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of depset, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | None)"#,
        Err(r#"unsupported binary operation: select | NoneType"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | True)"#,
        Err(r#"unsupported binary operation: select | bool"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | 1.5)"#,
        Err(r#"unsupported binary operation: select | float"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | depset([1]))"#,
        Err(r#"unsupported binary operation: select | depset"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | range(2))"#,
        Err(r#"unsupported binary operation: select | range"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | struct())"#,
        Err(r#"unsupported binary operation: select | struct"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of depset, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | select({"a":None}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | select({"a":True}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":depset([1])}) | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | 1)"#,
        Err(r#"unsupported binary operation: select | int"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | "s")"#,
        Err(r#"unsupported binary operation: select | string"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | (1,))"#,
        Err(r#"unsupported binary operation: select | tuple"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of range, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | None)"#,
        Err(r#"unsupported binary operation: select | NoneType"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | True)"#,
        Err(r#"unsupported binary operation: select | bool"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | 1.5)"#,
        Err(r#"unsupported binary operation: select | float"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | depset([1]))"#,
        Err(r#"unsupported binary operation: select | depset"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | range(2))"#,
        Err(r#"unsupported binary operation: select | range"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | struct())"#,
        Err(r#"unsupported binary operation: select | struct"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of range, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | select({"a":None}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | select({"a":True}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":range(2)}) | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | 1)"#,
        Err(r#"unsupported binary operation: select | int"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | "s")"#,
        Err(r#"unsupported binary operation: select | string"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | (1,))"#,
        Err(r#"unsupported binary operation: select | tuple"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of struct, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | None)"#,
        Err(r#"unsupported binary operation: select | NoneType"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | True)"#,
        Err(r#"unsupported binary operation: select | bool"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | 1.5)"#,
        Err(r#"unsupported binary operation: select | float"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | depset([1]))"#,
        Err(r#"unsupported binary operation: select | depset"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | range(2))"#,
        Err(r#"unsupported binary operation: select | range"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | struct())"#,
        Err(r#"unsupported binary operation: select | struct"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of struct, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | select({"a":None}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | select({"a":True}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":struct()}) | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | 1)"#,
        Err(r#"unsupported binary operation: select | int"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | "s")"#,
        Err(r#"unsupported binary operation: select | string"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | (1,))"#,
        Err(r#"unsupported binary operation: select | tuple"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of select, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | None)"#,
        Err(r#"unsupported binary operation: select | NoneType"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | True)"#,
        Err(r#"unsupported binary operation: select | bool"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | 1.5)"#,
        Err(r#"unsupported binary operation: select | float"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | depset([1]))"#,
        Err(r#"unsupported binary operation: select | depset"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | range(2))"#,
        Err(r#"unsupported binary operation: select | range"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | struct())"#,
        Err(r#"unsupported binary operation: select | struct"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of select, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | select({"a":None}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | select({"a":True}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | select({"a":1.5}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | select({"a":depset([1])}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | select({"a":range(2)}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | select({"a":struct()}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":select({"b":1})}) | select({"a":select({"b":1})}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":(1,)}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of string, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":(1,)}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of tuple, int)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":1}) + select({"a":1}))"#,
        Ok(r#"select({"a": 1}) + select({"a": 1}) + select({"a": 1})"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":1}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + 1 + 1)"#,
        Err(r#"Cannot combine incompatible types (select of list, int)"#),
    ),
    (
        r#"r=1
print((1,) + 1 + select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: tuple + int"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + {} + (1,))"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + 1 + 1)"#,
        Ok(r#"select({"a": 1}) + 1 + 1"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + 1 + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, int)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":{}}) + {})"#,
        Err(r#"Cannot combine incompatible types (int, select of dict)"#),
    ),
    (
        r#"r=1
print({} + select({"a":{}}) + "s")"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":[1]}) + select({"a":(1,)}))"#,
        Ok(r#"select({"a": [1]}) + select({"a": [1]}) + select({"a": (1,)})"#),
    ),
    (
        r#"r=1
print([1] + select({"a":(1,)}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + "s" + "s")"#,
        Err(r#"Cannot combine incompatible types (select of int, string)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":{}}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of string, select of dict)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":1}) + (1,))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of int)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":1}) + [1])"#,
        Err(r#"Cannot combine incompatible types (string, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + "s" + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of list, string)"#),
    ),
    (
        r#"r=1
print((1,) + "s" + select({"a":1}))"#,
        Err(r#"unsupported binary operation: tuple + string"#),
    ),
    (
        r#"r=1
print({} + select({"a":[1]}) + select({"a":{}}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + (1,) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of list, int)"#),
    ),
    (
        r#"r=1
print("s" + [1] + select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: string + list"#),
    ),
    (
        r#"r=1
print({} + select({"a":(1,)}) + (1,))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":[1]}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":1}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + "s" + {})"#,
        Err(r#"Cannot combine incompatible types (select of int, string)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":"s"}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + 1 + {})"#,
        Err(r#"Cannot combine incompatible types (select of list, int)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":1}) + 1)"#,
        Ok(r#"1 + select({"a": 1}) + 1"#),
    ),
    (
        r#"r=1
print([1] + "s" + select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: list + string"#),
    ),
    (
        r#"r=1
print(1 + (1,) + select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: int + tuple"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":[1]}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of tuple, int)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":"s"}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of string, int)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + {} + select({"a":{}}))"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print({} + (1,) + select({"a":1}))"#,
        Err(r#"unsupported binary operation: dict + tuple"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + "s" + "s")"#,
        Ok(r#"select({"a": "s"}) + "s" + "s""#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + (1,) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of string, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":(1,)}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + "s" + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of list, string)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":{}}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":"s"}) + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of string, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":1}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":1}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of int)"#),
    ),
    (
        r#"r=1
print({} + select({"a":(1,)}) + select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + "s" + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of dict)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":{}}) + 1)"#,
        Err(r#"Cannot combine incompatible types (string, select of dict)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":1}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":{}}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of dict)"#),
    ),
    (
        r#"r=1
print({} + select({"a":"s"}) + select({"a":1}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":{}}) + {})"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of dict)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":(1,)}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (int, select of tuple)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":(1,)}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (int, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":1}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of int)"#),
    ),
    (
        r#"r=1
print([1] + [1] + select({"a":(1,)}))"#,
        Ok(r#"[1, 1] + select({"a": (1,)})"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":{}}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":"s"}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":1}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of int)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":1}) + [1])"#,
        Err(r#"Cannot combine incompatible types (list, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + {} + select({"a":1}))"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":1}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":{}}) + select({"a":{}}))"#,
        Err(r#"unsupported binary operation: select + select"#),
    ),
    (
        r#"r=1
print("s" + select({"a":[1]}) + 1)"#,
        Err(r#"Cannot combine incompatible types (string, select of list)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":1}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (string, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + "s" + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of int, string)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":"s"}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of string)"#),
    ),
    (
        r#"r=1
print({} + select({"a":"s"}) + select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + {} + select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":[1]}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + (1,) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":1}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":"s"}) + "s")"#,
        Err(r#"Cannot combine incompatible types (list, select of string)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":(1,)}) + (1,))"#,
        Ok(r#"[1] + select({"a": (1,)}) + (1,)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":"s"}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (int, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":[1]}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":(1,)}) + select({"a":[1]}))"#,
        Ok(r#"(1,) + select({"a": (1,)}) + select({"a": [1]})"#),
    ),
    (
        r#"r=1
print({} + select({"a":{}}) + select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print(1 + (1,) + select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: int + tuple"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":1}) + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + [1] + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of int, list)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":"s"}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":"s"}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":"s"}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + "s" + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, string)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + "s" + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of int, string)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":"s"}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + (1,) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":"s"}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of string)"#),
    ),
    (
        r#"r=1
print({} + select({"a":"s"}) + (1,))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print((1,) + 1 + select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: tuple + int"#),
    ),
    (
        r#"r=1
print(1 + (1,) + select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: int + tuple"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":{}}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":(1,)}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + 1 + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, int)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":1}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of int, list)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + 1 + select({"a":1}))"#,
        Ok(r#"select({"a": 1}) + 1 + select({"a": 1})"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + [1] + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + {} + select({"a":{}}))"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + {} + select({"a":1}))"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":(1,)}) + [1])"#,
        Ok(r#"select({"a": [1]}) + select({"a": (1,)}) + [1]"#),
    ),
    (
        r#"r=1
print((1,) + (1,) + select({"a":[1]}))"#,
        Ok(r#"(1, 1) + select({"a": [1]})"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":{}}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + {} + "s")"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":{}}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + [1] + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of int, list)"#),
    ),
    (
        r#"r=1
print({} + [1] + select({"a":1}))"#,
        Err(r#"unsupported binary operation: dict + list"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":[1]}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":"s"}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of string)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":1}) + [1])"#,
        Err(r#"Cannot combine incompatible types (tuple, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":(1,)}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":{}}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of dict)"#),
    ),
    (
        r#"r=1
print({} + select({"a":1}) + [1])"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":(1,)}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of int, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":{}}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":{}}) + "s")"#,
        Err(r#"Cannot combine incompatible types (int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":(1,)}) + (1,))"#,
        Ok(r#"select({"a": (1,)}) + select({"a": (1,)}) + (1,)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + 1 + "s")"#,
        Err(r#"Cannot combine incompatible types (select of tuple, int)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":[1]}) + select({"a":[1]}))"#,
        Ok(r#"(1,) + select({"a": [1]}) + select({"a": [1]})"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":1}) + "s")"#,
        Err(r#"Cannot combine incompatible types (tuple, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":(1,)}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of tuple)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":"s"}) + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of string, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + "s" + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + {} + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print({} + select({"a":1}) + (1,))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + 1 + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":"s"}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of string)"#),
    ),
    (
        r#"r=1
print({} + select({"a":[1]}) + select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + [1] + select({"a":[1]}))"#,
        Ok(r#"select({"a": (1,)}) + [1] + select({"a": [1]})"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":1}) + {})"#,
        Err(r#"Cannot combine incompatible types (select of list, select of int)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":[1]}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (int, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":[1]}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of string)"#),
    ),
    (
        r#"r=1
print({} + (1,) + select({"a":{}}))"#,
        Err(r#"unsupported binary operation: dict + tuple"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":{}}) + select({"a":1}))"#,
        Err(r#"unsupported binary operation: select + select"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":1}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":"s"}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + {} + select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(1 + select({"a":{}}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + {} + 1)"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print([1] + select({"a":"s"}) + [1])"#,
        Err(r#"Cannot combine incompatible types (list, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":{}}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + 1 + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":"s"}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":(1,)}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":{}}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":[1]}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + (1,) + (1,))"#,
        Ok(r#"select({"a": (1,)}) + (1,) + (1,)"#),
    ),
    (
        r#"r=1
print({} + [1] + select({"a":{}}))"#,
        Err(r#"unsupported binary operation: dict + list"#),
    ),
    (
        r#"r=1
print("s" + (1,) + select({"a":1}))"#,
        Err(r#"unsupported binary operation: string + tuple"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + 1 + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, int)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":"s"}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of string, list)"#),
    ),
    (
        r#"r=1
print({} + select({"a":"s"}) + [1])"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":[1]}) + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(1 + select({"a":[1]}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (int, select of list)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":[1]}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of tuple, string)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":(1,)}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of int, select of tuple)"#),
    ),
    (
        r#"r=1
print([1] + [1] + select({"a":[1]}))"#,
        Ok(r#"[1, 1] + select({"a": [1]})"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + (1,) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + {} + [1])"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":1}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of int)"#),
    ),
    (
        r#"r=1
print({} + select({"a":1}) + select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + (1,) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of int, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":(1,)}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":[1]}) + [1])"#,
        Ok(r#"select({"a": (1,)}) + select({"a": [1]}) + [1]"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + [1] + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, list)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + "s" + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of tuple)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":1}) + {})"#,
        Err(r#"Cannot combine incompatible types (list, select of int)"#),
    ),
    (
        r#"r=1
print([1] + {} + select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: list + dict"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":(1,)}) + select({"a":[1]}))"#,
        Ok(r#"select({"a": [1]}) + select({"a": (1,)}) + select({"a": [1]})"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":"s"}) + 1)"#,
        Err(r#"Cannot combine incompatible types (tuple, select of string)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":(1,)}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of tuple, string)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + (1,) + {})"#,
        Err(r#"Cannot combine incompatible types (select of dict, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":"s"}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of list, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":{}}) + select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select + select"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":(1,)}) + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of tuple)"#),
    ),
    (
        r#"r=1
print([1] + 1 + select({"a":{}}))"#,
        Err(r#"unsupported binary operation: list + int"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":1}) + {})"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + (1,) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of int, tuple)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":(1,)}) + 1)"#,
        Err(r#"Cannot combine incompatible types (string, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":"s"}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of string)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":"s"}) + 1)"#,
        Err(r#"Cannot combine incompatible types (list, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + 1 + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, int)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + 1 + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of list, int)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":(1,)}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (string, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":[1]}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + "s" + {})"#,
        Err(r#"Cannot combine incompatible types (select of list, string)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":{}}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + 1 + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of list, int)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":"s"}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of string)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":"s"}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (int, select of string)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":[1]}) + (1,))"#,
        Ok(r#"(1,) + select({"a": [1]}) + (1,)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":(1,)}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of int, select of tuple)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":(1,)}) + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print({} + select({"a":{}}) + select({"a":(1,)}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print("s" + select({"a":{}}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (string, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + [1] + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of dict)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":1}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of int)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":(1,)}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":"s"}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of string)"#),
    ),
    (
        r#"r=1
print("s" + (1,) + select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: string + tuple"#),
    ),
    (
        r#"r=1
print([1] + (1,) + select({"a":{}}))"#,
        Err(r#"unsupported binary operation: list + tuple"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + [1] + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, list)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":"s"}) + {})"#,
        Err(r#"Cannot combine incompatible types (select of int, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":(1,)}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of tuple)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":(1,)}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (int, select of tuple)"#),
    ),
    (
        r#"r=1
print({} + select({"a":"s"}) + {})"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print((1,) + (1,) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + "s" + [1])"#,
        Err(r#"Cannot combine incompatible types (select of list, string)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + 1 + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of string, int)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":{}}) + (1,))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of dict)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":"s"}) + [1])"#,
        Err(r#"Cannot combine incompatible types (tuple, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":[1]}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of int, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + (1,) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":(1,)}) + {})"#,
        Err(r#"Cannot combine incompatible types (select of int, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + (1,) + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of string, tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + [1] + [1])"#,
        Ok(r#"select({"a": [1]}) + [1] + [1]"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":(1,)}) + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":[1]}) + {})"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":(1,)}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (int, select of tuple)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":[1]}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (string, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":[1]}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + [1] + {})"#,
        Err(r#"Cannot combine incompatible types (select of string, list)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":(1,)}) + {})"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print({} + select({"a":1}) + select({"a":"s"}))"#,
        Err(r#"unsupported binary operation: dict + select"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + "s" + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, string)"#),
    ),
    (
        r#"r=1
print(1 + [1] + select({"a":1}))"#,
        Err(r#"unsupported binary operation: int + list"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + "s" + 1)"#,
        Err(r#"Cannot combine incompatible types (select of tuple, string)"#),
    ),
    (
        r#"r=1
print(1 + select({"a":[1]}) + {})"#,
        Err(r#"Cannot combine incompatible types (int, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + [1] + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of string, list)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":{}}) + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + "s" + select({"a":"s"}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, string)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":[1]}) + select({"a":(1,)}))"#,
        Ok(r#"(1,) + select({"a": [1]}) + select({"a": (1,)})"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + "s" + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, string)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":1}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of int)"#),
    ),
    (
        r#"r=1
print((1,) + select({"a":(1,)}) + select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of dict)"#),
    ),
    (
        r#"r=1
print([1] + (1,) + select({"a":1}))"#,
        Err(r#"unsupported binary operation: list + tuple"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + 1 + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of string, int)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + 1 + (1,))"#,
        Err(r#"Cannot combine incompatible types (select of list, int)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + "s" + {})"#,
        Err(r#"Cannot combine incompatible types (select of tuple, string)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + [1] + {})"#,
        Err(r#"Cannot combine incompatible types (select of int, list)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":1}) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of string, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":(1,)}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of list, int)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + (1,) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of dict, tuple)"#),
    ),
    (
        r#"r=1
print((1,) + (1,) + select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (tuple, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":"s"}) + select({"a":[1]}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of string, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":[1]}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of tuple, string)"#),
    ),
    (
        r#"r=1
print(select({"a":(1,)}) + select({"a":{}}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of tuple, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + {} + [1])"#,
        Err(r#"unsupported binary operation: select + dict"#),
    ),
    (
        r#"r=1
print(1 + select({"a":[1]}) + [1])"#,
        Err(r#"Cannot combine incompatible types (int, select of list)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":(1,)}) + 1)"#,
        Err(r#"Cannot combine incompatible types (select of list, int)"#),
    ),
    (
        r#"r=1
print([1] + 1 + select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: list + int"#),
    ),
    (
        r#"r=1
print(1 + select({"a":"s"}) + (1,))"#,
        Err(r#"Cannot combine incompatible types (int, select of string)"#),
    ),
    (
        r#"r=1
print([1] + select({"a":(1,)}) + [1])"#,
        Ok(r#"[1] + select({"a": (1,)}) + [1]"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + 1 + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of list)"#),
    ),
    (
        r#"r=1
print("s" + select({"a":(1,)}) + [1])"#,
        Err(r#"Cannot combine incompatible types (string, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":"s"}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of int, select of string)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + select({"a":{}}) + select({"a":(1,)}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) + select({"a":{}}) + "s")"#,
        Err(r#"unsupported binary operation: select + select"#),
    ),
    (
        r#"r=1
print("s" + select({"a":(1,)}) + {})"#,
        Err(r#"Cannot combine incompatible types (string, select of tuple)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":1}) + select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + [1] + 1)"#,
        Err(r#"Cannot combine incompatible types (select of int, list)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + select({"a":1}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of int, string)"#),
    ),
    (
        r#"r=1
print({} | {} | select({"a":{}}))"#,
        Ok(r#"{} | select({"a": {}})"#),
    ),
    (
        r#"r=1
print({} | {} | select({"b":{"k":1}}))"#,
        Ok(r#"{} | select({"b": {"k": 1}})"#),
    ),
    (
        r#"r=1
print({} | {} | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of list)"#),
    ),
    (
        r#"r=1
print({} | {} | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of int)"#),
    ),
    (
        r#"r=1
print({} | select({"a":{}}) | {})"#,
        Ok(r#"{} | select({"a": {}}) | {}"#),
    ),
    (
        r#"r=1
print({} | select({"a":{}}) | select({"a":{}}))"#,
        Ok(r#"{} | select({"a": {}}) | select({"a": {}})"#),
    ),
    (
        r#"r=1
print({} | select({"a":{}}) | select({"b":{"k":1}}))"#,
        Ok(r#"{} | select({"a": {}}) | select({"b": {"k": 1}})"#),
    ),
    (
        r#"r=1
print({} | select({"a":{}}) | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print({} | select({"a":{}}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print({} | select({"a":{}}) | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print({} | select({"b":{"k":1}}) | {})"#,
        Ok(r#"{} | select({"b": {"k": 1}}) | {}"#),
    ),
    (
        r#"r=1
print({} | select({"b":{"k":1}}) | select({"a":{}}))"#,
        Ok(r#"{} | select({"b": {"k": 1}}) | select({"a": {}})"#),
    ),
    (
        r#"r=1
print({} | select({"b":{"k":1}}) | select({"b":{"k":1}}))"#,
        Ok(r#"{} | select({"b": {"k": 1}}) | select({"b": {"k": 1}})"#),
    ),
    (
        r#"r=1
print({} | select({"b":{"k":1}}) | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print({} | select({"b":{"k":1}}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print({} | select({"b":{"k":1}}) | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print({} | select({"a":[1]}) | {})"#,
        Err(r#"Cannot combine incompatible types (dict, select of list)"#),
    ),
    (
        r#"r=1
print({} | select({"a":[1]}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of list)"#),
    ),
    (
        r#"r=1
print({} | select({"a":[1]}) | select({"b":{"k":1}}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of list)"#),
    ),
    (
        r#"r=1
print({} | select({"a":[1]}) | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of list)"#),
    ),
    (
        r#"r=1
print({} | select({"a":[1]}) | [1])"#,
        Err(r#"Cannot combine incompatible types (dict, select of list)"#),
    ),
    (
        r#"r=1
print({} | select({"a":[1]}) | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of list)"#),
    ),
    (
        r#"r=1
print({} | [1] | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: dict | list"#),
    ),
    (
        r#"r=1
print({} | [1] | select({"b":{"k":1}}))"#,
        Err(r#"unsupported binary operation: dict | list"#),
    ),
    (
        r#"r=1
print({} | [1] | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: dict | list"#),
    ),
    (
        r#"r=1
print({} | [1] | select({"a":1}))"#,
        Err(r#"unsupported binary operation: dict | list"#),
    ),
    (
        r#"r=1
print({} | select({"a":1}) | {})"#,
        Err(r#"Cannot combine incompatible types (dict, select of int)"#),
    ),
    (
        r#"r=1
print({} | select({"a":1}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of int)"#),
    ),
    (
        r#"r=1
print({} | select({"a":1}) | select({"b":{"k":1}}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of int)"#),
    ),
    (
        r#"r=1
print({} | select({"a":1}) | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of int)"#),
    ),
    (
        r#"r=1
print({} | select({"a":1}) | [1])"#,
        Err(r#"Cannot combine incompatible types (dict, select of int)"#),
    ),
    (
        r#"r=1
print({} | select({"a":1}) | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | {} | {})"#,
        Ok(r#"select({"a": {}}) | {} | {}"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | {} | select({"a":{}}))"#,
        Ok(r#"select({"a": {}}) | {} | select({"a": {}})"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | {} | select({"b":{"k":1}}))"#,
        Ok(r#"select({"a": {}}) | {} | select({"b": {"k": 1}})"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | {} | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | {} | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | {} | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":{}}) | {})"#,
        Ok(r#"select({"a": {}}) | select({"a": {}}) | {}"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":{}}) | select({"a":{}}))"#,
        Ok(r#"select({"a": {}}) | select({"a": {}}) | select({"a": {}})"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":{}}) | select({"b":{"k":1}}))"#,
        Ok(r#"select({"a": {}}) | select({"a": {}}) | select({"b": {"k": 1}})"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":{}}) | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":{}}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":{}}) | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"b":{"k":1}}) | {})"#,
        Ok(r#"select({"a": {}}) | select({"b": {"k": 1}}) | {}"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"b":{"k":1}}) | select({"a":{}}))"#,
        Ok(r#"select({"a": {}}) | select({"b": {"k": 1}}) | select({"a": {}})"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"b":{"k":1}}) | select({"b":{"k":1}}))"#,
        Ok(r#"select({"a": {}}) | select({"b": {"k": 1}}) | select({"b": {"k": 1}})"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"b":{"k":1}}) | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"b":{"k":1}}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"b":{"k":1}}) | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":[1]}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":[1]}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":[1]}) | select({"b":{"k":1}}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":[1]}) | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":[1]}) | [1])"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":[1]}) | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | [1] | {})"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | [1] | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | [1] | select({"b":{"k":1}}))"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | [1] | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | [1] | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | [1] | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":1}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":1}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":1}) | select({"b":{"k":1}}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":1}) | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":1}) | [1])"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | select({"a":1}) | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | {} | {})"#,
        Ok(r#"select({"b": {"k": 1}}) | {} | {}"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | {} | select({"a":{}}))"#,
        Ok(r#"select({"b": {"k": 1}}) | {} | select({"a": {}})"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | {} | select({"b":{"k":1}}))"#,
        Ok(r#"select({"b": {"k": 1}}) | {} | select({"b": {"k": 1}})"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | {} | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | {} | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | {} | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":{}}) | {})"#,
        Ok(r#"select({"b": {"k": 1}}) | select({"a": {}}) | {}"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":{}}) | select({"a":{}}))"#,
        Ok(r#"select({"b": {"k": 1}}) | select({"a": {}}) | select({"a": {}})"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":{}}) | select({"b":{"k":1}}))"#,
        Ok(r#"select({"b": {"k": 1}}) | select({"a": {}}) | select({"b": {"k": 1}})"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":{}}) | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":{}}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":{}}) | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"b":{"k":1}}) | {})"#,
        Ok(r#"select({"b": {"k": 1}}) | select({"b": {"k": 1}}) | {}"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"b":{"k":1}}) | select({"a":{}}))"#,
        Ok(r#"select({"b": {"k": 1}}) | select({"b": {"k": 1}}) | select({"a": {}})"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"b":{"k":1}}) | select({"b":{"k":1}}))"#,
        Ok(r#"select({"b": {"k": 1}}) | select({"b": {"k": 1}}) | select({"b": {"k": 1}})"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"b":{"k":1}}) | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"b":{"k":1}}) | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"b":{"k":1}}) | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":[1]}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":[1]}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":[1]}) | select({"b":{"k":1}}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":[1]}) | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":[1]}) | [1])"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":[1]}) | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of list)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | [1] | {})"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | [1] | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | [1] | select({"b":{"k":1}}))"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | [1] | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | [1] | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | [1] | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":1}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":1}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":1}) | select({"b":{"k":1}}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":1}) | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":1}) | [1])"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"b":{"k":1}}) | select({"a":1}) | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of dict, select of int)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | {} | {})"#,
        Err(r#"Cannot combine incompatible types (select of list, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | {} | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of list, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | {} | select({"b":{"k":1}}))"#,
        Err(r#"Cannot combine incompatible types (select of list, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | {} | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of list, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | {} | [1])"#,
        Err(r#"Cannot combine incompatible types (select of list, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | {} | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of list, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":{}}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":{}}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":{}}) | select({"b":{"k":1}}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":{}}) | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":{}}) | [1])"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":{}}) | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"b":{"k":1}}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"b":{"k":1}}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"b":{"k":1}}) | select({"b":{"k":1}}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"b":{"k":1}}) | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"b":{"k":1}}) | [1])"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"b":{"k":1}}) | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":[1]}) | {})"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":[1]}) | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":[1]}) | select({"b":{"k":1}}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":[1]}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":[1]}) | [1])"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":[1]}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | [1] | {})"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | [1] | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | [1] | select({"b":{"k":1}}))"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | [1] | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | [1] | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | [1] | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":1}) | {})"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":1}) | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":1}) | select({"b":{"k":1}}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":1}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":1}) | [1])"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) | select({"a":1}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print([1] | {} | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: list | dict"#),
    ),
    (
        r#"r=1
print([1] | {} | select({"b":{"k":1}}))"#,
        Err(r#"unsupported binary operation: list | dict"#),
    ),
    (
        r#"r=1
print([1] | {} | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: list | dict"#),
    ),
    (
        r#"r=1
print([1] | {} | select({"a":1}))"#,
        Err(r#"unsupported binary operation: list | dict"#),
    ),
    (
        r#"r=1
print([1] | select({"a":{}}) | {})"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":{}}) | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":{}}) | select({"b":{"k":1}}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":{}}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":{}}) | [1])"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":{}}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"b":{"k":1}}) | {})"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"b":{"k":1}}) | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"b":{"k":1}}) | select({"b":{"k":1}}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"b":{"k":1}}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"b":{"k":1}}) | [1])"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"b":{"k":1}}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":[1]}) | {})"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":[1]}) | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":[1]}) | select({"b":{"k":1}}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":[1]}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":[1]}) | [1])"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":[1]}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | [1] | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: list | list"#),
    ),
    (
        r#"r=1
print([1] | [1] | select({"b":{"k":1}}))"#,
        Err(r#"unsupported binary operation: list | list"#),
    ),
    (
        r#"r=1
print([1] | [1] | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: list | list"#),
    ),
    (
        r#"r=1
print([1] | [1] | select({"a":1}))"#,
        Err(r#"unsupported binary operation: list | list"#),
    ),
    (
        r#"r=1
print([1] | select({"a":1}) | {})"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":1}) | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":1}) | select({"b":{"k":1}}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":1}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":1}) | [1])"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print([1] | select({"a":1}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: list | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | {} | {})"#,
        Err(r#"Cannot combine incompatible types (select of int, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | {} | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of int, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | {} | select({"b":{"k":1}}))"#,
        Err(r#"Cannot combine incompatible types (select of int, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | {} | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of int, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | {} | [1])"#,
        Err(r#"Cannot combine incompatible types (select of int, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | {} | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of int, dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":{}}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":{}}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":{}}) | select({"b":{"k":1}}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":{}}) | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":{}}) | [1])"#,
        Err(r#"Cannot combine incompatible types (select of int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":{}}) | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"b":{"k":1}}) | {})"#,
        Err(r#"Cannot combine incompatible types (select of int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"b":{"k":1}}) | select({"a":{}}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"b":{"k":1}}) | select({"b":{"k":1}}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"b":{"k":1}}) | select({"a":[1]}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"b":{"k":1}}) | [1])"#,
        Err(r#"Cannot combine incompatible types (select of int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"b":{"k":1}}) | select({"a":1}))"#,
        Err(r#"Cannot combine incompatible types (select of int, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":[1]}) | {})"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":[1]}) | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":[1]}) | select({"b":{"k":1}}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":[1]}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":[1]}) | [1])"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":[1]}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | [1] | {})"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | [1] | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | [1] | select({"b":{"k":1}}))"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | [1] | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | [1] | [1])"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | [1] | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | list"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":1}) | {})"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":1}) | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":1}) | select({"b":{"k":1}}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":1}) | select({"a":[1]}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":1}) | [1])"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":1}) | select({"a":1}) | select({"a":1}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(({} | select({"a":{}})) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of dict, list)"#),
    ),
    (
        r#"r=1
print(([1] + select({"a":[1]})) + ({} | select({"a":{}})))"#,
        Err(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | ({} | select({"b":{}})))"#,
        Ok(r#"select({"a": {}}) | {} | select({"b": {}})"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | ({"x":1} | {"y":2}))"#,
        Ok(r#"select({"a": {}}) | {"x": 1, "y": 2}"#),
    ),
    (
        r#"r=1
print(select({"a":{}}) | {"x":1} | {"y":2})"#,
        Ok(r#"select({"a": {}}) | {"x": 1} | {"y": 2}"#),
    ),
    (
        r#"r=1
print({"x":1} | {"y":2} | select({"a":{}}))"#,
        Ok(r#"{"x": 1, "y": 2} | select({"a": {}})"#),
    ),
    (
        r#"r=1
print(select({"a":1}) + 1 + 2)"#,
        Ok(r#"select({"a": 1}) + 1 + 2"#),
    ),
    (
        r#"r=1
print(1 + 2 + select({"a":1}))"#,
        Ok(r#"3 + select({"a": 1})"#),
    ),
    (
        r#"r=1
print(set([1]) | select({"a":{}}))"#,
        Err(r#"unsupported binary operation: set | select"#),
    ),
    (
        r#"r=1
print(select({"a":set([1])}) + select({"a":set([1])}))"#,
        Ok(r#"select({"a": set([1])}) + select({"a": set([1])})"#),
    ),
    (
        r#"r=1
print(select({"a":set([1])}) | select({"a":set([1])}))"#,
        Err(r#"unsupported binary operation: select | select"#),
    ),
    (
        r#"r=1
print(select({"a":set([1])}) | set([1]))"#,
        Err(r#"unsupported binary operation: select | set"#),
    ),
    (
        r#"r=1
print(select({"a":set([1])}) + [1])"#,
        Err(r#"Cannot combine incompatible types (select of set, list)"#),
    ),
    (
        r#"r=1
print(select({"a":[1]}) + set([1]))"#,
        Err(r#"Cannot combine incompatible types (select of list, set)"#),
    ),
    (
        r#"r=1
print(select({"a":Label("//x:y")}) + Label("//x:y"))"#,
        Ok(r#"select({"a": Label("//x:y")}) + Label("//x:y")"#),
    ),
    (
        r#"r=1
print(select({"a":Label("//x:y")}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of Label, string)"#),
    ),
    (
        r#"r=1
print(select({"a":"//x:y"}) + Label("//x:y"))"#,
        Err(r#"Cannot combine incompatible types (select of string, Label)"#),
    ),
    (
        r#"r=1
def f():
  pass
print(select({"a":f}) + f)"#,
        Ok(r#"select({"a": <function f from //:u.bzl>}) + <function f from //:u.bzl>"#),
    ),
    (
        r#"r=1
print(select({"a":len}) + len)"#,
        Ok(r#"select({"a": <built-in function len>}) + <built-in function len>"#),
    ),
    (
        r#"r=1
print(select({"a":[]}) + [])"#,
        Ok(r#"select({"a": []}) + []"#),
    ),
    (
        r#"r=1
print(select({"a":[]}) + select({"b":()}))"#,
        Ok(r#"select({"a": []}) + select({"b": ()})"#),
    ),
    (
        r#"r=1
print(select({"a":1, "b":"s"}) + select({"a":2}))"#,
        Ok(r#"select({"a": 1, "b": "s"}) + select({"a": 2})"#),
    ),
    (
        r#"r=1
print(select({"a":1, "b":"s"}) + "s")"#,
        Err(r#"Cannot combine incompatible types (select of int, string)"#),
    ),
];

pub(crate) const SELECT_BUILD_CASES: &[BuildRow] = &[
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"o":attr.output(),"ol":attr.output_list(),"ld":attr.label_list(allow_files=True),"lf":attr.label(allow_files=True),"sv":attr.string(values=["a","b"]),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",ll=select({"a":["f1.txt"]}) + 1)
print(existing_rule("t"))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"Cannot combine incompatible types (select of list, int)"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"o":attr.output(),"ol":attr.output_list(),"ld":attr.label_list(allow_files=True),"lf":attr.label(allow_files=True),"sv":attr.string(values=["a","b"]),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",ll=select({"a":["f1.txt"]}) + "x")
print(existing_rule("t"))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"Cannot combine incompatible types (select of list, string)"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"o":attr.output(),"ol":attr.output_list(),"ld":attr.label_list(allow_files=True),"lf":attr.label(allow_files=True),"sv":attr.string(values=["a","b"]),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name=select({"//conditions:default":"t"}),m="m")
print(existing_rule("t"))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"r 'name' attribute must be a string"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//conditions:default":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"("x",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"],"b":["y"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:a"): ("x",), Label("//:b"): ("y",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"],"//conditions:default":["z"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:a"): ("x",), Label("//conditions:default"): ("z",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//conditions:default":["z"],"a":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//conditions:default"): ("z",), Label("//:a"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=["x"] + select({"a":["y"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[
            r#"select({Label("//conditions:default"): ("x",)}) + select({Label("//:a"): ("y",)})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=["x"] + select({"//conditions:default":["y"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"("x", "y")"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//conditions:default":["y"]}) + ["x"])
print(existing_rule("t")["sl"])"#,
        printed: &[r#"("y", "x")"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//conditions:default":["y"]}) + select({"//conditions:default":["z"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"("y", "z")"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":["y"]}) + select({"b":["z"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:a"): ("y",)}) + select({Label("//:b"): ("z",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":["y"]}) + select({"//conditions:default":["z"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[
            r#"select({Label("//:a"): ("y",)}) + select({Label("//conditions:default"): ("z",)})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",s=select({"//conditions:default":"x"}))
print(existing_rule("t")["s"])"#,
        printed: &[r#"x"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",s=select({"a":"x"}))
print(existing_rule("t")["s"])"#,
        printed: &[r#"select({Label("//:a"): "x"})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",s=select({"//conditions:default":"x"}) + "y")
print(existing_rule("t")["s"])"#,
        printed: &[r#"xy"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",s=select({"//conditions:default":None}))
print(existing_rule("t")["s"])"#,
        printed: &[r#""#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",s=select({"a":None,"b":"x"}))
print(existing_rule("t")["s"])"#,
        printed: &[r#"select({Label("//:a"): "", Label("//:b"): "x"})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",i=select({"//conditions:default":3}))
print(existing_rule("t")["i"])"#,
        printed: &[r#"3"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",b=select({"//conditions:default":True}))
print(existing_rule("t")["b"])"#,
        printed: &[r#"True"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",b=select({"//conditions:default":1}))
print(existing_rule("t")["b"])"#,
        printed: &[r#"True"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",b=select({"a":True}))
print(existing_rule("t")["b"])"#,
        printed: &[r#"select({Label("//:a"): True})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",l=select({"//conditions:default":"f1.txt"}))
print(existing_rule("t")["l"])"#,
        printed: &[r#":f1.txt"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",l=select({"//conditions:default":None}))
print(existing_rule("t")["l"])"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"key "l" not found in view"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",l=select({"a":"f1.txt","b":None}))
print(existing_rule("t")["l"])"#,
        printed: &[r#"select({Label("//:a"): ":f1.txt", Label("//:b"): None})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",ll=select({"//conditions:default":["f1.txt"]}))
print(existing_rule("t")["ll"])"#,
        printed: &[r#"(":f1.txt",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",ll=select({"//conditions:default":["f1.txt"]}) + ["sub:x.txt"])
print(existing_rule("t")["ll"])"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label 'sub:x.txt' in element 0 of attribute 'll' of 'r': invalid label 'sub:x.txt': absolute label must begin with '@' or '//'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",ll=select({"a":["f1.txt"]}))
print(existing_rule("t")["ll"])"#,
        printed: &[r#"select({Label("//:a"): (":f1.txt",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sd=select({"//conditions:default":{"k":"v"}}))
print(existing_rule("t")["sd"])"#,
        printed: &[r#"{"k": "v"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sd=select({"a":{"k":"v"}}))
print(existing_rule("t")["sd"])"#,
        printed: &[r#"select({Label("//:a"): {"k": "v"}})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sd=select({"//conditions:default":{"k":"v"}}) | {"j":"w"})
print(existing_rule("t")["sd"])"#,
        printed: &[r#"{"k": "v", "j": "w"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sd={"j":"w"} | select({"//conditions:default":{"k":"v"}}))
print(existing_rule("t")["sd"])"#,
        printed: &[r#"{"j": "w", "k": "v"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sd={"j":"w"} | select({"a":{"k":"v"}}))
print(existing_rule("t")["sd"])"#,
        printed: &[
            r#"select({Label("//conditions:default"): {"j": "w"}}) | select({Label("//:a"): {"k": "v"}})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sd=select({"a":{"k":"v"}}) | select({"b":{"k":"w"}}))
print(existing_rule("t")["sd"])"#,
        printed: &[r#"select({Label("//:a"): {"k": "v"}}) | select({Label("//:b"): {"k": "w"}})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sld=select({"//conditions:default":{"k":["v"]}}))
print(existing_rule("t")["sld"])"#,
        printed: &[r#"{"k": ("v",)}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",lk=select({"//conditions:default":{"f1.txt":"v"}}))
print(existing_rule("t")["lk"])"#,
        printed: &[r#"{":f1.txt": "v"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",lk=select({"a":{"f1.txt":"v"}}))
print(existing_rule("t")["lk"])"#,
        printed: &[r#"select({Label("//:a"): {":f1.txt": "v"}})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",il=select({"//conditions:default":[1,2]}))
print(existing_rule("t")["il"])"#,
        printed: &[r#"(1, 2)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//conditions:default":("x","y")}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"("x", "y")"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":("x",)}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:a"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",ll=select({"a":("f1.txt",)}))
print(existing_rule("t")["ll"])"#,
        printed: &[r#"select({Label("//:a"): (":f1.txt",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"]}) + select({"//conditions:default":["z"]}) + ["q"])
print(existing_rule("t")["sl"])"#,
        printed: &[
            r#"select({Label("//:a"): ("x",)}) + select({Label("//conditions:default"): ("z",)}) + select({Label("//conditions:default"): ("q",)})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"]}, no_match_error="nm"))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:a"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",l=select({"a":":f1.txt", "b":"//:f1.txt"}))
print(existing_rule("t")["l"])"#,
        printed: &[r#"select({Label("//:a"): ":f1.txt", Label("//:b"): ":f1.txt"})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({":c":["x"], "//conditions:default":["z"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:c"): ("x",), Label("//conditions:default"): ("z",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//x:y":["x"], "@//x:y":["z"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//x:y"): ("z",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({":c":["x"], "//:c":["z"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:c"): ("z",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"], "//:a":["y"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:a"): ("y",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a b":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:a b"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label '' in attribute 'sl' of 'r': invalid target name '': empty target name"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//x:":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label '//x:' in attribute 'sl' of 'r': invalid target name '': empty target name"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"@@//x:y":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//x:y"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"@foo//x:y":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("@@[unknown repo 'foo' requested from @@]//x:y"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({":":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label ':' in attribute 'sl' of 'r': invalid target name '': empty target name"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//:":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label '//:' in attribute 'sl' of 'r': invalid target name '': empty target name"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//x:y:z":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label '//x:y:z' in attribute 'sl' of 'r': invalid target name 'y:z': target names may not contain ':'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"x:y:z":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label 'x:y:z' in attribute 'sl' of 'r': invalid target name 'y:z': target names may not contain ':'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//...":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label '//...' in attribute 'sl' of 'r': invalid label '//...': package name cannot contain '...'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"@//:x":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:x"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//conditions:other":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//conditions:other"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//x/:y":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label '//x/:y' in attribute 'sl' of 'r': invalid package name 'x/': package names may not end with '/'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"x":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:x"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"x/y":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:x/y"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//x:y/":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label '//x:y/' in attribute 'sl' of 'r': invalid target name 'y/': target names may not end with '/'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//x:../y":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label '//x:../y' in attribute 'sl' of 'r': invalid target name '../y': target names may not contain up-level references '..'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"@@foo//:x":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("@@foo//:x"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//:f1.txt":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:f1.txt"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"sub/x.txt":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:sub/x.txt"): ("x",)})"#],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:sub/x.txt' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:x.txt'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"], "a":["y"]}))
print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"dictionary expression has duplicate key: "a""#),
    },
    BuildRow {
        bzl: r#"def r():
  native.filegroup(name="g",srcs=select({Label("//x:y"):["f1.txt"]}))
  print(native.existing_rule("g")["srcs"])"#,
        build: r#"r()"#,
        printed: &[r#"select({Label("//x:y"): (":f1.txt",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r():
  native.filegroup(name="g",srcs=select({Label("//x:y"):["f1.txt"], "//x:y":["f1.txt"]}))
  print(native.existing_rule("g")["srcs"])"#,
        build: r#"r()"#,
        printed: &[r#"select({Label("//x:y"): (":f1.txt",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r():
  native.filegroup(name="g",srcs=select({Label("//conditions:default"):["f1.txt"]}))
  print(native.existing_rule("g")["srcs"])"#,
        build: r#"r()"#,
        printed: &[r#"(":f1.txt",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r():
  native.filegroup(name="g",srcs=select({Label("//conditions:default"):["f1.txt"], "//conditions:default":[]}))
  print(native.existing_rule("g")["srcs"])"#,
        build: r#"r()"#,
        printed: &[r#"select({Label("//conditions:default"): ()})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r():
  native.filegroup(name="g",srcs=select({Label("@foo//x:y"):["f1.txt"]}))
  print(native.existing_rule("g")["srcs"])"#,
        build: r#"r()"#,
        printed: &[
            r#"select({Label("@@[unknown repo 'foo' requested from @@]//x:y"): (":f1.txt",)})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r():
  native.filegroup(name="g",srcs=select({Label("//x:y"):["f1.txt"], Label("//x:y"):[]}))
  print(native.existing_rule("g")["srcs"])"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"dictionary expression has duplicate key: Label("//x:y")"#),
    },
    BuildRow {
        bzl: r#"def r():
  native.filegroup(name="g",srcs=select({Label("//:a"):["f1.txt"], "a":[]}))
  print(native.existing_rule("g")["srcs"])"#,
        build: r#"r()"#,
        printed: &[r#"select({Label("//:a"): ()})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r():
  native.filegroup(name="g",srcs=select({Label("//:a"):["f1.txt"], ":a":[]}))
  print(native.existing_rule("g")["srcs"])"#,
        build: r#"r()"#,
        printed: &[r#"select({Label("//:a"): ()})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r():
  native.filegroup(name="g",srcs=select({"@//:a":["f1.txt"], ":a":[]}))
  print(native.existing_rule("g")["srcs"])"#,
        build: r#"r()"#,
        printed: &[r#"select({Label("//:a"): ()})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def r():
  native.filegroup(name="g",srcs=select({"//:a":["f1.txt"], ":a":[]}))
  print(native.existing_rule("g")["srcs"])"#,
        build: r#"r()"#,
        printed: &[r#"select({Label("//:a"): ()})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",s=select({"a":None,"//conditions:default":None}))
print(existing_rule("t")["s"] if "s" in existing_rule("t") else "absent")"#,
        printed: &[r#"select({Label("//:a"): "", Label("//conditions:default"): ""})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",s=select({"a":None,"b":None}))
print(existing_rule("t").get("s","absent"))"#,
        printed: &[r#"select({Label("//:a"): "", Label("//:b"): ""})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",i=select({"a":None,"//conditions:default":None}))
print(existing_rule("t")["i"] if "i" in existing_rule("t") else "absent")"#,
        printed: &[r#"select({Label("//:a"): 0, Label("//conditions:default"): 0})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",i=select({"a":None,"b":None}))
print(existing_rule("t").get("i","absent"))"#,
        printed: &[r#"select({Label("//:a"): 0, Label("//:b"): 0})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",b=select({"a":None,"//conditions:default":None}))
print(existing_rule("t")["b"] if "b" in existing_rule("t") else "absent")"#,
        printed: &[r#"select({Label("//:a"): False, Label("//conditions:default"): False})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",b=select({"a":None,"b":None}))
print(existing_rule("t").get("b","absent"))"#,
        printed: &[r#"select({Label("//:a"): False, Label("//:b"): False})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",l=select({"a":None,"//conditions:default":None}))
print(existing_rule("t")["l"] if "l" in existing_rule("t") else "absent")"#,
        printed: &[r#"select({Label("//:a"): None, Label("//conditions:default"): None})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",l=select({"a":None,"b":None}))
print(existing_rule("t").get("l","absent"))"#,
        printed: &[r#"select({Label("//:a"): None, Label("//:b"): None})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",ll=select({"a":None,"//conditions:default":None}))
print(existing_rule("t")["ll"] if "ll" in existing_rule("t") else "absent")"#,
        printed: &[r#"select({Label("//:a"): (), Label("//conditions:default"): ()})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",ll=select({"a":None,"b":None}))
print(existing_rule("t").get("ll","absent"))"#,
        printed: &[r#"select({Label("//:a"): (), Label("//:b"): ()})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=select({"a":None,"//conditions:default":None}))
print(existing_rule("t")["sl"] if "sl" in existing_rule("t") else "absent")"#,
        printed: &[r#"select({Label("//:a"): (), Label("//conditions:default"): ()})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=select({"a":None,"b":None}))
print(existing_rule("t").get("sl","absent"))"#,
        printed: &[r#"select({Label("//:a"): (), Label("//:b"): ()})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sd=select({"a":None,"//conditions:default":None}))
print(existing_rule("t")["sd"] if "sd" in existing_rule("t") else "absent")"#,
        printed: &[r#"select({Label("//:a"): {}, Label("//conditions:default"): {}})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sd=select({"a":None,"b":None}))
print(existing_rule("t").get("sd","absent"))"#,
        printed: &[r#"select({Label("//:a"): {}, Label("//:b"): {}})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sld=select({"a":None,"//conditions:default":None}))
print(existing_rule("t")["sld"] if "sld" in existing_rule("t") else "absent")"#,
        printed: &[r#"select({Label("//:a"): {}, Label("//conditions:default"): {}})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sld=select({"a":None,"b":None}))
print(existing_rule("t").get("sld","absent"))"#,
        printed: &[r#"select({Label("//:a"): {}, Label("//:b"): {}})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",lk=select({"a":None,"//conditions:default":None}))
print(existing_rule("t")["lk"] if "lk" in existing_rule("t") else "absent")"#,
        printed: &[r#"select({Label("//:a"): {}, Label("//conditions:default"): {}})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",lk=select({"a":None,"b":None}))
print(existing_rule("t").get("lk","absent"))"#,
        printed: &[r#"select({Label("//:a"): {}, Label("//:b"): {}})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",il=select({"a":None,"//conditions:default":None}))
print(existing_rule("t")["il"] if "il" in existing_rule("t") else "absent")"#,
        printed: &[r#"select({Label("//:a"): (), Label("//conditions:default"): ()})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",il=select({"a":None,"b":None}))
print(existing_rule("t").get("il","absent"))"#,
        printed: &[r#"select({Label("//:a"): (), Label("//:b"): ()})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=select({"a":1}),i=select({"a":"x"}),s=select({"a":1}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:t: expected value of type 'int' for each branch in select expression of attribute 'i' of 'r' (including '//:a'), but got "x" (string)"#,
            r#"BUILD.bazel:2:2: //:t: expected value of type 'string' for each branch in select expression of attribute 's' of 'r' (including '//:a'), but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",s=select({"a":1}),sl=select({"a":1}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'string' for each branch in select expression of attribute 's' of 'r' (including '//:a'), but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",il=select({"a":["x"]}),sl=select({"a":1}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'int' for element 0 of each branch in select expression of attribute 'il' of 'r' (including '//:a'), but got "x" (string)"#,
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=select({"a":1}),il=select({"a":["x"]}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:t: expected value of type 'int' for element 0 of each branch in select expression of attribute 'il' of 'r' (including '//:a'), but got "x" (string)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",lk=select({"a":1}),sl=select({"a":1}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'dict(label, string)' for each branch in select expression of attribute 'lk' of 'r' (including '//:a'), but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=select({"a":1}),lk=select({"a":1}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:t: expected value of type 'dict(label, string)' for each branch in select expression of attribute 'lk' of 'r' (including '//:a'), but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=select({"a":1}),zzz=1)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:t: no such attribute 'zzz' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",zzz=1,sl=select({"a":1}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: no such attribute 'zzz' in 'r' rule"#,
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=select({"a":1}),ll=["sub/x.txt"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
            r#"BUILD.bazel:2:2: Label '//:sub/x.txt' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:x.txt'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",ll=["sub/x.txt"],sl=select({"a":1}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
            r#"BUILD.bazel:2:2: Label '//:sub/x.txt' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:x.txt'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",sl=select({"a":1}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:t: missing value for mandatory attribute 'm' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",sl=1)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for attribute 'sl' of 'r', but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:t: missing value for mandatory attribute 'm' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",visibility=select({"a":1}),sl=select({"a":1}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: attribute "visibility" is not configurable"#,
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=select({"a":1}),visibility=select({"a":1}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:t: attribute "visibility" is not configurable"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",tags=select({"a":1}))"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:t: attribute "tags" is not configurable"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",tags=select({"a":["x"]}),sl=select({"a":1}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: attribute "tags" is not configurable"#,
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",o=1,sl=select({"a":1}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: no such attribute 'o' in 'r' rule"#,
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",_p=select({"a":1}))"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:t: no such attribute '_p' in 'r' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",zzz=select({"a":1}))"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:t: no such attribute 'zzz' in 'r' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=select({"a":1, "b":2}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=select({"b":1, "a":2}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:b'), but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"], "b":2, "c":3}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:b'), but got 2 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=[select({"a":["x"]})])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'string' for element 0 of attribute 'sl' of 'r', but got select({"a": ["x"]}) (select)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sd={"k":select({"a":"v"})})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'string' for dict value element, but got select({"a": "v"}) (select)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=["x", select({"a":"v"})])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'string' for element 1 of attribute 'sl' of 'r', but got select({"a": "v"}) (select)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",s=select({"a":select({"b":"x"})}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'string' for each branch in select expression of attribute 's' of 'r' (including '//:a'), but got select({"b": "x"}) (select)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"]}) + select({"b":[1]}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'string' for element 0 of each branch in select expression of attribute 'sl' of 'r' (including '//:b'), but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"]}) + [1])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'string' for element 0 of attribute 'sl' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"]}) + select({"b":{"k":"v"}}))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"Cannot combine incompatible types (select of list, select of dict)"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",s=select({"a":"x"}) + select({"b":1}))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"Cannot combine incompatible types (select of string, select of int)"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",i=select({"a":1}) + select({"b":"x"}))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"Cannot combine incompatible types (select of int, select of string)"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",s=select({"a":1}) + "x")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"Cannot combine incompatible types (select of int, string)"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",s=select({"a":"x"}) + "y")
print(1)"#,
        printed: &[r#"1"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",s=select({"a":"x"}) + "y")
print(existing_rule("t")["s"])"#,
        printed: &[
            r#"select({Label("//:a"): "x"}) + select({Label("//conditions:default"): "y"})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",i=select({"a":1}) + 2)
print(existing_rule("t")["i"])"#,
        printed: &[r#"select({Label("//:a"): 1}) + select({Label("//conditions:default"): 2})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",b=select({"a":True}) + False)
print(existing_rule("t")["b"])"#,
        printed: &[r#"False"#],
        events: &[r#"BUILD.bazel:2:2: //:t: type 'boolean' doesn't support select concatenation"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",s=select({"a":"x"}) + select({"b":"y"}))
print(existing_rule("t")["s"])"#,
        printed: &[r#"select({Label("//:a"): "x"}) + select({Label("//:b"): "y"})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",s="q" + select({"a":"x"}))
print(existing_rule("t")["s"])"#,
        printed: &[
            r#"select({Label("//conditions:default"): "q"}) + select({Label("//:a"): "x"})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",l=select({"a":":f1.txt"}) + ":f1.txt")"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:t: type 'label' doesn't support select concatenation"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",l=select({"a":[":f1.txt"]}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'string' for each branch in select expression of attribute 'l' of 'r' (including '//:a'), but got [":f1.txt"] (list)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"]}),s=select({"a":"x"}),i=select({"a":1}))
print(existing_rule("t")["sl"])
print(existing_rule("t")["s"])
print(existing_rule("t")["i"])"#,
        printed: &[
            r#"select({Label("//:a"): ("x",)})"#,
            r#"select({Label("//:a"): "x"})"#,
            r#"select({Label("//:a"): 1})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"]}))
print(existing_rules()["t"]["sl"])"#,
        printed: &[r#"select({Label("//:a"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"]}))
x=existing_rule("t")
print(x["sl"])
print(type(x["sl"]))
print(x["sl"]==x["sl"])"#,
        printed: &[r#"select({Label("//:a"): ("x",)})"#, r#"select"#, r#"True"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"load(':u.bzl','x_test','xb')
x_test(name="t",size=select({"a":"small"}))"#,
        printed: &[],
        events: &[r#"BUILD.bazel:3:7: //:t: attribute "size" is not configurable"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"load(':u.bzl','x_test','xb')
x_test(name="t",timeout=select({"a":"short"}))"#,
        printed: &[],
        events: &[r#"BUILD.bazel:3:7: //:t: attribute "timeout" is not configurable"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"load(':u.bzl','x_test','xb')
x_test(name="t",flaky=select({"a":True}))"#,
        printed: &[],
        events: &[r#"BUILD.bazel:3:7: //:t: attribute "flaky" is not configurable"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"load(':u.bzl','x_test','xb')
x_test(name="t",shard_count=select({"a":2}))"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"load(':u.bzl','x_test','xb')
x_test(name="t",local=select({"a":True}))"#,
        printed: &[],
        events: &[r#"BUILD.bazel:3:7: //:t: attribute "local" is not configurable"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"load(':u.bzl','x_test','xb')
x_test(name="t",args=select({"a":["a"]}))"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"load(':u.bzl','x_test','xb')
x_test(name="t",env=select({"a":{"a":"b"}}))"#,
        printed: &[],
        events: &[r#"BUILD.bazel:3:7: //:t: no such attribute 'env' in 'x_test' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"load(':u.bzl','x_test','xb')
x_test(name="t",env_inherit=select({"a":["a"]}))"#,
        printed: &[],
        events: &[r#"BUILD.bazel:3:7: //:t: no such attribute 'env_inherit' in 'x_test' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"load(':u.bzl','x_test','xb')
xb(name="t",args=select({"a":["a"]}))"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"load(':u.bzl','x_test','xb')
xb(name="t",output_licenses=select({"a":["a"]}))"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"load(':u.bzl','x_test','xb')
xb(name="t",env=select({"a":{"a":"b"}}))"#,
        printed: &[],
        events: &[r#"BUILD.bazel:3:3: //:t: no such attribute 'env' in 'xb' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"load(':u.bzl','x_test','xb')
x_test(name="t",size=select({"a":"huge"}))"#,
        printed: &[],
        events: &[r#"BUILD.bazel:3:7: //:t: attribute "size" is not configurable"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"load(':u.bzl','x_test','xb')
x_test(name="t",args=select({"a":["x"]}))
print(existing_rule("t")["args"])"#,
        printed: &[r#"select({Label("//:a"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})
x_test=rule(implementation=lambda ctx: [],test=True)
xb=rule(implementation=lambda ctx: [],executable=True)"#,
        build: r#"load(':u.bzl','x_test','xb')
xb(name="t",args=select({"a":["x"]}))
print(existing_rule("t")["args"])"#,
        printed: &[r#"select({Label("//:a"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=1"#,
        build: r#"alias(name="g",actual=select({"a":"f1.txt","b":"sub/x.txt"}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:6: Label '//:sub/x.txt' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:x.txt'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=1"#,
        build: r#"alias(name="g",actual=select({"a":["f1.txt"]}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:6: //:g: expected value of type 'string' for each branch in select expression of attribute 'actual' of 'alias' (including '//:a'), but got ["f1.txt"] (list)"#,
            r#"BUILD.bazel:2:6: //:g: missing value for mandatory attribute 'actual' in 'alias' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=1"#,
        build: r#"alias(name="g",actual=select({"a":"f1.txt"}))
print(existing_rule("g")["actual"])"#,
        printed: &[r#"select({Label("//:a"): ":f1.txt"})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=1"#,
        build: r#"alias(name="g",actual=select({"//conditions:default":"f1.txt"}))
print(existing_rule("g")["actual"])"#,
        printed: &[r#":f1.txt"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=1"#,
        build: r#"alias(name="g",actual=None)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:6: //:g: missing value for mandatory attribute 'actual' in 'alias' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=1"#,
        build: r#"alias(name="g",actual=select({"a":None}))"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=1"#,
        build: r#"alias(name="g",actual=select({"//conditions:default":None}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:6: //:g: missing value for mandatory attribute 'actual' in 'alias' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=1"#,
        build: r#"filegroup(name="g",srcs=select({"a":["f1.txt"]}) + glob(["*.txt"]))
print(existing_rule("g")["srcs"])"#,
        printed: &[
            r#"select({Label("//:a"): (":f1.txt",)}) + select({Label("//conditions:default"): (":f1.txt",)})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=1"#,
        build: r#"filegroup(name="g",srcs=select({"a":[]}),data=select({"a":[]}))
print(existing_rule("g")["srcs"])
print(existing_rule("g")["data"])"#,
        printed: &[
            r#"select({Label("//:a"): ()})"#,
            r#"select({Label("//:a"): ()})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=1"#,
        build: r#"filegroup(name="g",output_group=select({"a":"x"}))
print(existing_rule("g")["output_group"])"#,
        printed: &[r#"select({Label("//:a"): "x"})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=1"#,
        build: r#"filegroup(name="g",features=select({"a":["x"]}))
print(existing_rule("g")["features"])"#,
        printed: &[r#"select({Label("//:a"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=1"#,
        build: r#"filegroup(name="g",target_compatible_with=select({"a":["f1.txt"]}))
print(existing_rule("g")["target_compatible_with"])"#,
        printed: &[r#"select({Label("//:a"): (":f1.txt",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=1"#,
        build: r#"filegroup(name="g",aspect_hints=select({"a":["f1.txt"]}))
print(existing_rule("g")["aspect_hints"])"#,
        printed: &[r#"select({Label("//:a"): (":f1.txt",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=1"#,
        build: r#"filegroup(name="g",srcs=select({"a":["f1.txt"]}))
print(existing_rule("g")["srcs"])"#,
        printed: &[r#"select({Label("//:a"): (":f1.txt",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f():
  return attr.string(default=select({"a":"x"}))
x=f()
r=1"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to string(), parameter 'default' got value of type 'select', want 'string'"#,
        ),
    },
    BuildRow {
        bzl: r#"def f():
  return attr.string_list(default=select({"a":["x"]}))
x=f()
r=1"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to string_list(), parameter 'default' got value of type 'select', want 'sequence'"#,
        ),
    },
    BuildRow {
        bzl: r#"def f():
  return attr.label(default=select({"a":"//:f1.txt"}))
x=f()
r=1"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to label(), parameter 'default' got value of type 'select', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    },
    BuildRow {
        bzl: r#"def f():
  return attr.int(default=select({"a":1}))
x=f()
r=1"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to int(), parameter 'default' got value of type 'select', want 'int'"#,
        ),
    },
    BuildRow {
        bzl: r#"def f():
  return attr.string(values=select({"a":["x"]}))
x=f()
r=1"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to string(), parameter 'values' got value of type 'select', want 'sequence'"#,
        ),
    },
    BuildRow {
        bzl: r#"def f():
  return attr.label(allow_files=select({"a":True}))
x=f()
r=1"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to label(), parameter 'allow_files' got value of type 'select', want 'bool, sequence, or NoneType'"#,
        ),
    },
    BuildRow {
        bzl: r#"def f():
  return attr.label_list(allow_files=[select({"a":".x"})])
x=f()
r=1"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"at index 0 of allow_files argument, got element of type select, want string"#,
        ),
    },
    BuildRow {
        bzl: r#"def f():
  return attr.string(doc=select({"a":"x"}))
x=f()
r=1"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to string(), parameter 'doc' got value of type 'select', want 'string or NoneType'"#,
        ),
    },
    BuildRow {
        bzl: r#"def f():
  return attr.string(mandatory=select({"a":True}))
x=f()
r=1"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to string(), parameter 'mandatory' got value of type 'select', want 'bool'"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string(default=select({"a":"x"}))})"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to string(), parameter 'default' got value of type 'select', want 'string'"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string_list(default=select({"a":["x"]}))})"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to string_list(), parameter 'default' got value of type 'select', want 'sequence'"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.label(default=select({"a":"//:f1.txt"}))})"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to label(), parameter 'default' got value of type 'select', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=select({"a":1}))
x=1"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to rule(), parameter 'implementation' got value of type 'select', want 'function'"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs=select({"a":{}}))"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to rule(), parameter 'attrs' got value of type 'select', want 'dict'"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],outputs=select({"a":{}}))"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to rule(), parameter 'outputs' got value of type 'select', want 'dict, NoneType, or function'"#,
        ),
    },
    BuildRow {
        bzl: r#"r=1"#,
        build: r#"print(select({"a":1}))"#,
        printed: &[r#"select({"a": 1})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=1"#,
        build: r#"filegroup(name="g",srcs=select({"a":["f1.txt"]}))
print(existing_rule("g")["srcs"])
filegroup(name="h",srcs=[":g"])"#,
        printed: &[r#"select({Label("//:a"): (":f1.txt",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",s=select({"a":"x"}) + select({"b":"x"}))
print(existing_rule("t")["s"])"#,
        printed: &[r#"select({Label("//:a"): "x"}) + select({Label("//:b"): "x"})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",s=select({"a":"x"}) + "x")
print(existing_rule("t")["s"])"#,
        printed: &[
            r#"select({Label("//:a"): "x"}) + select({Label("//conditions:default"): "x"})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",s=select({"a":"x"}) | "x")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"unsupported binary operation: select | string"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",i=select({"a":1}) + select({"b":1}))
print(existing_rule("t")["i"])"#,
        printed: &[r#"select({Label("//:a"): 1}) + select({Label("//:b"): 1})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",i=select({"a":1}) + 1)
print(existing_rule("t")["i"])"#,
        printed: &[r#"select({Label("//:a"): 1}) + select({Label("//conditions:default"): 1})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",i=select({"a":1}) | 1)"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"unsupported binary operation: select | int"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",b=select({"a":True}) + select({"b":True}))
print(existing_rule("t")["b"])"#,
        printed: &[r#"False"#],
        events: &[r#"BUILD.bazel:2:2: //:t: type 'boolean' doesn't support select concatenation"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",b=select({"a":True}) + True)
print(existing_rule("t")["b"])"#,
        printed: &[r#"False"#],
        events: &[r#"BUILD.bazel:2:2: //:t: type 'boolean' doesn't support select concatenation"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",b=select({"a":True}) | True)"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"unsupported binary operation: select | bool"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",l=select({"a":":f1.txt"}) + select({"b":":f1.txt"}))
print(existing_rule("t")["l"])"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:t: type 'label' doesn't support select concatenation"#],
        fatal: Some(r#"key "l" not found in view"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",l=select({"a":":f1.txt"}) + ":f1.txt")
print(existing_rule("t")["l"])"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:t: type 'label' doesn't support select concatenation"#],
        fatal: Some(r#"key "l" not found in view"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",l=select({"a":":f1.txt"}) | ":f1.txt")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"unsupported binary operation: select | string"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",ll=select({"a":["f1.txt"]}) + select({"b":["f1.txt"]}))
print(existing_rule("t")["ll"])"#,
        printed: &[
            r#"select({Label("//:a"): (":f1.txt",)}) + select({Label("//:b"): (":f1.txt",)})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",ll=select({"a":["f1.txt"]}) + ["f1.txt"])
print(existing_rule("t")["ll"])"#,
        printed: &[
            r#"select({Label("//:a"): (":f1.txt",)}) + select({Label("//conditions:default"): (":f1.txt",)})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",ll=select({"a":["f1.txt"]}) | ["f1.txt"])"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"unsupported binary operation: select | list"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"]}) + select({"b":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:a"): ("x",)}) + select({Label("//:b"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"]}) + ["x"])
print(existing_rule("t")["sl"])"#,
        printed: &[
            r#"select({Label("//:a"): ("x",)}) + select({Label("//conditions:default"): ("x",)})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"]}) | ["x"])"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"unsupported binary operation: select | list"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sd=select({"a":{"k":"v"}}) | select({"b":{"k":"v"}}))
print(existing_rule("t")["sd"])"#,
        printed: &[r#"select({Label("//:a"): {"k": "v"}}) | select({Label("//:b"): {"k": "v"}})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sd=select({"a":{"k":"v"}}) | {"k":"v"})
print(existing_rule("t")["sd"])"#,
        printed: &[
            r#"select({Label("//:a"): {"k": "v"}}) | select({Label("//conditions:default"): {"k": "v"}})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sd=select({"a":{"k":"v"}}) + select({"b":{"k":"v"}}))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"unsupported binary operation: select + select"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sld=select({"a":{"k":["v"]}}) | select({"b":{"k":["v"]}}))
print(existing_rule("t")["sld"])"#,
        printed: &[
            r#"select({Label("//:a"): {"k": ("v",)}}) | select({Label("//:b"): {"k": ("v",)}})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sld=select({"a":{"k":["v"]}}) | {"k":["v"]})
print(existing_rule("t")["sld"])"#,
        printed: &[
            r#"select({Label("//:a"): {"k": ("v",)}}) | select({Label("//conditions:default"): {"k": ("v",)}})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sld=select({"a":{"k":["v"]}}) + select({"b":{"k":["v"]}}))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"unsupported binary operation: select + select"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",lk=select({"a":{"f1.txt":"v"}}) | select({"b":{"f1.txt":"v"}}))
print(existing_rule("t")["lk"])"#,
        printed: &[
            r#"select({Label("//:a"): {":f1.txt": "v"}}) | select({Label("//:b"): {":f1.txt": "v"}})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",lk=select({"a":{"f1.txt":"v"}}) | {"f1.txt":"v"})
print(existing_rule("t")["lk"])"#,
        printed: &[
            r#"select({Label("//:a"): {":f1.txt": "v"}}) | select({Label("//conditions:default"): {":f1.txt": "v"}})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",lk=select({"a":{"f1.txt":"v"}}) + select({"b":{"f1.txt":"v"}}))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"unsupported binary operation: select + select"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",il=select({"a":[1]}) + select({"b":[1]}))
print(existing_rule("t")["il"])"#,
        printed: &[r#"select({Label("//:a"): (1,)}) + select({Label("//:b"): (1,)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",il=select({"a":[1]}) + [1])
print(existing_rule("t")["il"])"#,
        printed: &[
            r#"select({Label("//:a"): (1,)}) + select({Label("//conditions:default"): (1,)})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",il=select({"a":[1]}) | [1])"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"unsupported binary operation: select | list"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"o":attr.output()})"#,
        build: r#"r(name="t",o=select({"a":"x"}) + "y")"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:t: attribute "o" is not configurable"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"o":attr.output_list()})"#,
        build: r#"r(name="t",o=select({"a":["x"]}) + ["y"])"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:t: attribute "o" is not configurable"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",ll=["f1.txt","f1.txt"] + select({"a":["f1.txt"]}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:f1.txt' is duplicated in the 'll' attribute of rule 't'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",ll=["f1.txt","f1.txt"] + select({"//conditions:default":["f1.txt"]}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:f1.txt' is duplicated in the 'll' attribute of rule 't'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",ll=select({"//conditions:default":["f1.txt"]}) + ["f1.txt"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:f1.txt' is duplicated in the 'll' attribute of rule 't'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//conditions:default":["f1.txt"]}) + ["f1.txt"])
print(existing_rule("t")["sl"])"#,
        printed: &[r#"("f1.txt", "f1.txt")"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",lk=select({"//conditions:default":{"f1.txt":"a"}}) | {"f1.txt":"b"})
print(existing_rule("t")["lk"])"#,
        printed: &[r#"{":f1.txt": "b"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",lk=select({"a":{"f1.txt":"a"}}) | {"f1.txt":"b"})
print(existing_rule("t")["lk"])"#,
        printed: &[
            r#"select({Label("//:a"): {":f1.txt": "a"}}) | select({Label("//conditions:default"): {":f1.txt": "b"}})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",lk=select({"a":{"f1.txt":"a", ":f1.txt":"b"}}))
print(existing_rule("t")["lk"])"#,
        printed: &[r#"{}"#],
        events: &[
            r#"BUILD.bazel:2:2: //:t: duplicate labels in each branch in select expression of attribute 'lk' of 'r' (including '//:a'): //:f1.txt (as ["f1.txt", ":f1.txt"])"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",lk={"f1.txt":"a", ":f1.txt":"b"})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: duplicate labels in attribute 'lk' of 'r': //:f1.txt (as ["f1.txt", ":f1.txt"])"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",lk=select({"a":{"f1.txt":"a"}}) | select({"b":{"f1.txt":"b"}}))
print(existing_rule("t")["lk"])"#,
        printed: &[
            r#"select({Label("//:a"): {":f1.txt": "a"}}) | select({Label("//:b"): {":f1.txt": "b"}})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sd=select({"a":{"k":"v"}}) | {"k":"w"})
print(existing_rule("t")["sd"])"#,
        printed: &[
            r#"select({Label("//:a"): {"k": "v"}}) | select({Label("//conditions:default"): {"k": "w"}})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sd=select({"//conditions:default":{"k":"v"}}) | {"k":"w"})
print(existing_rule("t")["sd"])"#,
        printed: &[r#"{"k": "w"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":[]}) + [])
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:a"): ()}) + select({Label("//conditions:default"): ()})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//conditions:default":[]}) + [])
print(existing_rule("t")["sl"])"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=[] + select({"//conditions:default":[]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"]}) + select({"b":["x"]}) + select({"c":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[
            r#"select({Label("//:a"): ("x",)}) + select({Label("//:b"): ("x",)}) + select({Label("//:c"): ("x",)})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"]}) + (select({"b":["x"]}) + ["y"]))
print(existing_rule("t")["sl"])"#,
        printed: &[
            r#"select({Label("//:a"): ("x",)}) + select({Label("//:b"): ("x",)}) + select({Label("//conditions:default"): ("y",)})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",s=select({"a":"x","b":1}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'string' for each branch in select expression of attribute 's' of 'r' (including '//:b'), but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",i=select({"a":1,"b":True}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'int' for each branch in select expression of attribute 'i' of 'r' (including '//:b'), but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",b=select({"a":True,"b":2}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected one of [False, True, 0, 1] for each branch in select expression of attribute 'b' of 'r' (including '//:b'), but got 2 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",b=select({"a":True,"b":0}))
print(existing_rule("t")["b"])"#,
        printed: &[r#"select({Label("//:a"): True, Label("//:b"): False})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",il=select({"a":[1],"b":[True]}))
print(existing_rule("t")["il"])"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'int' for element 0 of each branch in select expression of attribute 'il' of 'r' (including '//:b'), but got True (bool)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",l=select({"a":"//x:y"}))
print(existing_rule("t")["l"])"#,
        printed: &[r#"select({Label("//:a"): "//x:y"})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",l=select({"a":"@foo//x:y"}))
print(existing_rule("t")["l"])"#,
        printed: &[r#"select({Label("//:a"): "@@[unknown repo 'foo' requested from @@]//x:y"})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",l=select({"a":"x y"}))
print(existing_rule("t")["l"])"#,
        printed: &[r#"select({Label("//:a"): ":x y"})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",l=select({"a":"//x:"}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label '//x:' in each branch in select expression of attribute 'l' of 'r' (including '//:a'): invalid target name '': empty target name"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",l=select({"a":""}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label '' in each branch in select expression of attribute 'l' of 'r' (including '//:a'): invalid target name '': empty target name"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",ll=select({"a":["//x:"]}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label '//x:' in element 0 of each branch in select expression of attribute 'll' of 'r' (including '//:a'): invalid target name '': empty target name"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",ll=select({"a":["//x:y:z"]}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label '//x:y:z' in element 0 of each branch in select expression of attribute 'll' of 'r' (including '//:a'): invalid target name 'y:z': target names may not contain ':'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"b":attr.bool(),"l":attr.label(),"ll":attr.label_list(),"sl":attr.string_list(),"sd":attr.string_dict(),"sld":attr.string_list_dict(),"lk":attr.label_keyed_string_dict(),"il":attr.int_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",lk=select({"a":{"//x:":"v"}}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label '//x:' in dict key element: invalid target name '': empty target name"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",i=select({"//conditions:default":1}) + 2)
print(existing_rule("t")["i"])"#,
        printed: &[r#"3"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",i=2 + select({"//conditions:default":1}) + 3)
print(existing_rule("t")["i"])"#,
        printed: &[r#"6"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",s=select({"//conditions:default":"a"}) + select({"//conditions:default":"b"}))
print(existing_rule("t")["s"])"#,
        printed: &[r#"ab"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"],"b":["y"],"//:a":["z"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:a"): ("z",), Label("//:b"): ("y",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":["x"],"b":["y"],":a":1}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"()"#],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//x:":1}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label '//x:' in attribute 'sl' of 'r': invalid target name '': empty target name"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//x:":["x"], "b":1}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label '//x:' in attribute 'sl' of 'r': invalid target name '': empty target name"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"a":1, "//x:":["x"]}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: expected value of type 'list(string)' for each branch in select expression of attribute 'sl' of 'r' (including '//:a'), but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({":":["x"], "//conditions:default":1}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:t: invalid label ':' in attribute 'sl' of 'r': invalid target name '': empty target name"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sd=select({"//conditions:default":{"a":"b"}}) | select({"//conditions:default":{"c":"d"}}))
print(existing_rule("t")["sd"])"#,
        printed: &[r#"{"a": "b", "c": "d"}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sd=select({"//conditions:default":{"a":"b"}}) | select({"x":{"c":"d"}}))
print(existing_rule("t")["sd"])"#,
        printed: &[
            r#"select({Label("//conditions:default"): {"a": "b"}}) | select({Label("//:x"): {"c": "d"}})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//conditions:default":["a"], "x":[]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//conditions:default"): ("a",), Label("//:x"): ()})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",ll=select({"x":["f1.txt"]}) + select({"x":["f1.txt","f1.txt"]}))"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:f1.txt' is duplicated in the 'll' attribute of rule 't'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",ll=select({"x":["f1.txt"]}) + select({"y":["sub/x.txt"]}))
print(existing_rule("t")["ll"])"#,
        printed: &[
            r#"select({Label("//:x"): (":f1.txt",)}) + select({Label("//:y"): (":sub/x.txt",)})"#,
        ],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:sub/x.txt' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:x.txt'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({":default":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"select({Label("//:default"): ("x",)})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"@//conditions:default":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"("x",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"@@//conditions:default":["x"]}))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"("x",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//conditions:default":["x"]}, no_match_error="m"))
print(existing_rule("t")["sl"])"#,
        printed: &[r#"("x",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"s":attr.string(),"i":attr.int(),"sl":attr.string_list(),"sd":attr.string_dict(),"lk":attr.label_keyed_string_dict(),"ll":attr.label_list(),"m":attr.string(mandatory=True)})"#,
        build: r#"r(name="t",m="m",sl=select({"//conditions:default":["x"]}, no_match_error="m") + select({"a":["x"]}, no_match_error="n"))
print(existing_rule("t")["sl"])"#,
        printed: &[
            r#"select({Label("//conditions:default"): ("x",)}) + select({Label("//:a"): ("x",)})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string(configurable=False)})"#,
        build: r#"r(name="t",a=select({"//conditions:default":"x"}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string(configurable=False)})"#,
        build: r#"r(name="t",a=select({"x":"x"}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string(configurable=False)})"#,
        build: r#"r(name="t",a="x")
print(existing_rule("t")["a"])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string(configurable=True)})"#,
        build: r#"r(name="t",a=select({"//conditions:default":"x"}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string(configurable=True)})"#,
        build: r#"r(name="t",a=select({"x":"x"}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string(configurable=True)})"#,
        build: r#"r(name="t",a="x")
print(existing_rule("t")["a"])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string(configurable=None)})"#,
        build: r#"r(name="t",a=select({"//conditions:default":"x"}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to string(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string(configurable=None)})"#,
        build: r#"r(name="t",a=select({"x":"x"}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to string(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string(configurable=None)})"#,
        build: r#"r(name="t",a="x")
print(existing_rule("t")["a"])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to string(), parameter 'configurable' got value of type 'NoneType', want 'bool or unbound'"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.output(configurable=True)})"#,
        build: r#"r(name="t",a=select({"//conditions:default":"x"}))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"output() got unexpected keyword argument 'configurable'"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.output(configurable=True)})"#,
        build: r#"r(name="t",a=select({"x":"x"}))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"output() got unexpected keyword argument 'configurable'"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.output(configurable=True)})"#,
        build: r#"r(name="t",a="x")
print(existing_rule("t")["a"])"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"output() got unexpected keyword argument 'configurable'"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.output(configurable=False)})"#,
        build: r#"r(name="t",a=select({"//conditions:default":"x"}))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"output() got unexpected keyword argument 'configurable'"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.output(configurable=False)})"#,
        build: r#"r(name="t",a=select({"x":"x"}))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"output() got unexpected keyword argument 'configurable'"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.output(configurable=False)})"#,
        build: r#"r(name="t",a="x")
print(existing_rule("t")["a"])"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"output() got unexpected keyword argument 'configurable'"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.output_list(configurable=True)})"#,
        build: r#"r(name="t",a=select({"//conditions:default":["x"]}))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"output_list() got unexpected keyword argument 'configurable'"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.output_list(configurable=True)})"#,
        build: r#"r(name="t",a=select({"x":["x"]}))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"output_list() got unexpected keyword argument 'configurable'"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.output_list(configurable=True)})"#,
        build: r#"r(name="t",a=["x"])
print(existing_rule("t")["a"])"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"output_list() got unexpected keyword argument 'configurable'"#),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.label_list(configurable=False)})"#,
        build: r#"r(name="t",a=select({"//conditions:default":["f1.txt"]}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.label_list(configurable=False)})"#,
        build: r#"r(name="t",a=select({"x":["f1.txt"]}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.label_list(configurable=False)})"#,
        build: r#"r(name="t",a=["f1.txt"])
print(existing_rule("t")["a"])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.int(configurable=False)})"#,
        build: r#"r(name="t",a=select({"//conditions:default":1}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.int(configurable=False)})"#,
        build: r#"r(name="t",a=select({"x":1}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.int(configurable=False)})"#,
        build: r#"r(name="t",a=1)
print(existing_rule("t")["a"])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string_list(configurable=False)})"#,
        build: r#"r(name="t",a=select({"//conditions:default":["x"]}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string_list(configurable=False)})"#,
        build: r#"r(name="t",a=select({"x":["x"]}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string_list(configurable=False)})"#,
        build: r#"r(name="t",a=["x"])
print(existing_rule("t")["a"])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string_dict(configurable=False)})"#,
        build: r#"r(name="t",a=select({"//conditions:default":{"k":"v"}}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string_dict(configurable=False)})"#,
        build: r#"r(name="t",a=select({"x":{"k":"v"}}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string_dict(configurable=False)})"#,
        build: r#"r(name="t",a={"k":"v"})
print(existing_rule("t")["a"])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.label(configurable=False)})"#,
        build: r#"r(name="t",a=select({"//conditions:default":"f1.txt"}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.label(configurable=False)})"#,
        build: r#"r(name="t",a=select({"x":"f1.txt"}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.label(configurable=False)})"#,
        build: r#"r(name="t",a="f1.txt")
print(existing_rule("t")["a"])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string(configurable=1)})"#,
        build: r#"1"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to string(), parameter 'configurable' got value of type 'int', want 'bool or unbound'"#,
        ),
    },
    BuildRow {
        bzl: r#"r=rule(implementation=lambda ctx: [],attrs={"a":attr.string(configurable=False)})"#,
        build: r#"r(name="t",a=select({"x":"y"}) + select({"z":"w"}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"attribute 'a' has the 'configurable' argument set, which is not allowed in rule definitions"#,
        ),
    },
];
