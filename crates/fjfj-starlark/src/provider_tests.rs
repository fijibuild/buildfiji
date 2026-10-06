//! `provider()` against Bazel 9.2.0 (buildfiji-mum.3.4): the table at the
//! bottom is what Bazel printed and reported for the same `.bzl` bodies, and
//! the tests here are what a table cannot say: that a provider is the same
//! provider once frozen and loaded.

use crate::test_support::{replay, run_files};

/// Probes whose answer is the Starlark runtime's generic wording, which
/// belongs to buildfiji-v32.
const GENERIC: &[&str] = &[];

/// Probes that need something no bead has built yet, or differ in the
/// crate.
const SKIPPED: &[&str] = &[
    "rule(",
    "aspect(",
    "DefaultInfo",
    // Bazel's `struct` is a callable `Provider`, so it is not `callable` and
    // cannot be an `init`; the crate's is a function.
    "init=struct",
    // The harness reads the first line of what `print` writes, and strips
    // the last newline.
    "encode_indent",
    "proto.encode_text",
    // Bazel lets `*["x"]` and `doc = "y"` both bind `doc`, and the crate
    // refuses a `**` that repeats a keyword before the call is made.
    "*[\"x\"]",
    "**{\"fields\"",
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
fn provider_is_bazels() {
    assert_replays(replay(PROVIDER_CASES, SKIPPED, GENERIC));
}

/// The error `main` ends in, or what it printed.
fn outcome(files: &[(&str, &str)], main: &str) -> String {
    match run_files(files, main) {
        Ok(printed) => printed.join("\n"),
        Err(e) => e,
    }
}

#[test]
fn a_provider_is_itself_once_frozen_and_loaded() {
    let a = "P = provider(fields = ['a'])\nI = P(a = 1)\nd = {P: 'p', I: 'i'}\n";
    let main = "load(':a.bzl', 'P', 'I', 'd')
print(I == P(a = 1))
print(d[P], d[P(a = 1)])
print(type(I), dir(I))
print(P == provider(fields = ['a']))
print(I == struct(a = 1))
";
    assert_eq!(
        outcome(&[("a.bzl", a)], main),
        "True\np i\nstruct [\"a\"]\nFalse\nFalse"
    );
}

#[test]
fn two_loads_of_a_file_are_one_provider() {
    let a = "P = provider()\n";
    let b = "load(':a.bzl', 'P')\nQ = P\n";
    let main = "load(':a.bzl', 'P')\nload(':b.bzl', 'Q')\nprint(P == Q, P(x = 1) == Q(x = 1))\n";
    assert_eq!(outcome(&[("a.bzl", a), ("b.bzl", b)], main), "True True");
}

#[test]
fn a_provider_is_named_by_the_first_top_level_name_it_is_bound_to() {
    let fields = |bind: &str, call: &str| {
        let a = format!("{bind}\ndef f():\n    return {call}\n");
        outcome(&[("a.bzl", &a)], "load(':a.bzl', 'f')\nf()\n")
    };
    let says = |out: String, name: &str| {
        assert!(
            out.contains(&format!("in call to instantiate provider {name}")),
            "{out}"
        );
    };
    says(fields("P = provider(fields = [])", "P(b = 1)"), "P");
    says(fields("Q = provider(fields = [])\nP = Q", "P(b = 1)"), "Q");
    says(fields("_P = provider(fields = [])", "_P(b = 1)"), "_P");
    says(
        fields(
            "P, R = provider(fields = [], init = lambda **k: k)",
            "P(b = 1)",
        ),
        "P",
    );
    says(
        fields(
            "P, R = provider(fields = [], init = lambda **k: k)",
            "R(b = 1)",
        ),
        "P",
    );
    says(
        fields(
            "def mk():\n    return provider(fields = [])\nP = mk()",
            "P(b = 1)",
        ),
        "P",
    );
    // Only a value bound to a name is named: not one in a container, nor a
    // provider made and dropped.
    says(
        fields("d = {'P': provider(fields = [])}", "d['P'](b = 1)"),
        "<no name>",
    );
    says(
        fields("l = [provider(fields = [])]", "l[0](b = 1)"),
        "<no name>",
    );
    says(fields("", "provider(fields = [])(b = 1)"), "<no name>");
}

#[test]
fn a_provider_is_named_where_it_is_bound_even_while_its_module_runs() {
    let out = outcome(&[], "P = provider(fields = [])\nP(b = 1)\n");
    assert!(out.contains("in call to instantiate provider P"), "{out}");
    // So is a `_P`, which a native function cannot read by name: the name is
    // taken from the assignment being run.
    let out = outcome(&[], "_P = provider(fields = [])\n_P(b = 1)\n");
    assert!(out.contains("in call to instantiate provider _P"), "{out}");
    // A value made by a function and bound to a `_Q` is named when the module
    // is done.
    let out = outcome(
        &[],
        "def f():\n    return provider(fields = [])\n_Q = f()\n_Q(b = 1)\n",
    );
    assert!(
        out.contains("in call to instantiate provider <no name>"),
        "{out}"
    );
}

#[test]
fn only_a_named_provider_can_be_required() {
    let out = |src: &str| outcome(&[], src);
    assert_eq!(
        out("P = provider()\nprint(attr.label(providers = [P]))"),
        "<attr.label>"
    );
    // A `_P` is not visible to a native function until the module is done,
    // so a module that has one gives every provider the benefit of the doubt.
    assert_eq!(
        out("_P = provider()\nprint(attr.label(providers = [[_P]]))"),
        "<attr.label>"
    );
    assert!(
        out("def f():\n    return provider()\nprint(attr.label(providers = [f()]))")
            .contains("Providers should be top-level values in extension files that define them.")
    );
    assert_eq!(
        out("P = provider()\ndef f():\n    return attr.label(providers = [P])\nprint(f())"),
        "<attr.label>"
    );
}

#[test]
fn a_provider_of_one_file_is_named_in_the_file_that_loads_it_from_a_container() {
    // Bazel names a provider at the first top-level binding of it, in
    // whichever file makes it: a loaded one that was only in a dict has
    // none until a file binds it.
    let a = "d = {'P': provider(fields = [])}\n";
    let main = "load(':a.bzl', 'd')\nP = d['P']\n";
    assert_eq!(outcome(&[("a.bzl", a)], main), "");
}

const PROVIDER_CASES: &[(&str, Result<&str, &str>)] = &[
    (
        r#"P = provider()
def X():
    print(P)
X()"#,
        Ok(r#"<provider>"#),
    ),
    (
        r#"P = provider()
def X():
    print(type(P))
X()"#,
        Ok(r#"Provider"#),
    ),
    (
        r#"P = provider()
def X():
    print(repr(P))
X()"#,
        Ok(r#"<provider>"#),
    ),
    (
        r#"P = provider()
def X():
    print(str(P))
X()"#,
        Ok(r#"<provider>"#),
    ),
    (
        r#"P = provider()
def X():
    print(dir(P))
X()"#,
        Ok(r#"[]"#),
    ),
    (
        r#"P = provider()
def X():
    print(P == P)
X()"#,
        Ok(r#"True"#),
    ),
    (
        r#"P = provider()
def X():
    print(hash(P) == hash(P))
X()"#,
        Err(r#"in call to hash(), parameter 'value' got value of type 'Provider', want 'string'"#),
    ),
    (
        r#"P = provider()
def X():
    print(bool(P))
X()"#,
        Ok(r#"True"#),
    ),
    (
        r#"P = provider()
def X():
    print(P())
X()"#,
        Ok(r#"struct()"#),
    ),
    (
        r#"P = provider()
def X():
    print(type(P()))
X()"#,
        Ok(r#"struct"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=1))
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"P = provider()
def X():
    print(repr(P(a=1)))
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"P = provider()
def X():
    print(str(P(a=1)))
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"P = provider()
def X():
    print(dir(P(a=1)))
X()"#,
        Ok(r#"["a"]"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=1) == P(a=1))
X()"#,
        Ok(r#"True"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=1) == P(a=2))
X()"#,
        Ok(r#"False"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=1) == P(b=1))
X()"#,
        Ok(r#"False"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=1) != P(a=1))
X()"#,
        Ok(r#"False"#),
    ),
    (
        r#"P = provider()
def X():
    print(hash(P(a=1)) == hash(P(a=1)))
X()"#,
        Err(r#"in call to hash(), parameter 'value' got value of type 'struct', want 'string'"#),
    ),
    (
        r#"P = provider()
def X():
    print(hash(P(a=[1])))
X()"#,
        Err(r#"in call to hash(), parameter 'value' got value of type 'struct', want 'string'"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=1).a)
X()"#,
        Ok(r#"1"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=1).b)
X()"#,
        Err(r#"'P' value has no field or method 'b'
Available attributes: a"#),
    ),
    (
        r#"P = provider()
def X():
    print(getattr(P(a=1), "a"))
X()"#,
        Ok(r#"1"#),
    ),
    (
        r#"P = provider()
def X():
    print(getattr(P(a=1), "b", 5))
X()"#,
        Ok(r#"5"#),
    ),
    (
        r#"P = provider()
def X():
    print(hasattr(P(a=1), "a"))
X()"#,
        Ok(r#"True"#),
    ),
    (
        r#"P = provider()
def X():
    print(hasattr(P(a=1), "b"))
X()"#,
        Ok(r#"False"#),
    ),
    (
        r#"P = provider()
def X():
    P(1)
X()"#,
        Err(r#"P: unexpected positional arguments"#),
    ),
    (
        r#"P = provider()
def X():
    P(1, a=2)
X()"#,
        Err(r#"P: unexpected positional arguments"#),
    ),
    (
        r#"P = provider()
def X():
    x = P(a=1)
    x.a = 2
X()"#,
        Err(r#"struct value does not support field assignment"#),
    ),
    (
        r#"P = provider()
def X():
    x = P(a=1)
    x.b = 2
X()"#,
        Err(r#"struct value does not support field assignment"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=1) < P(a=2))
X()"#,
        Err(r#"unsupported comparison: struct <=> struct"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=1) + P(b=2))
X()"#,
        Ok(r#"struct(a = 1, b = 2)"#),
    ),
    (
        r#"P = provider()
def X():
    print(len(P(a=1)))
X()"#,
        Err(
            r#"in call to len(), parameter 'x' got value of type 'struct', want 'iterable or string'"#,
        ),
    ),
    (
        r#"P = provider()
def X():
    print(list(P(a=1)))
X()"#,
        Err(r#"in call to list(), parameter 'x' got value of type 'struct', want 'iterable'"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=1)["a"])
X()"#,
        Err(r#"type 'struct' has no operator [](string)"#),
    ),
    (
        r#"P = provider()
def X():
    print("a" in P(a=1))
X()"#,
        Err(r#"unsupported binary operation: string in struct"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=1)(2))
X()"#,
        Err(r#"'struct' object is not callable"#),
    ),
    (
        r#"P = provider()
def X():
    print(bool(P()))
X()"#,
        Ok(r#"True"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=P(b=2)))
X()"#,
        Ok(r#"struct(a = struct(b = 2))"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=[1,2], b={"x":1}))
X()"#,
        Ok(r#"struct(a = [1, 2], b = {"x": 1})"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a="s"))
X()"#,
        Ok(r#"struct(a = "s")"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=None))
X()"#,
        Ok(r#"struct(a = None)"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=None) == P())
X()"#,
        Ok(r#"False"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(**{"a":1}))
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(**{"1a":1}))
X()"#,
        Ok(r#"struct(1a = 1)"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(**{"a b":1}))
X()"#,
        Ok(r#"struct(a b = 1)"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(**{"":1}))
X()"#,
        Ok(r#"struct( = 1)"#),
    ),
    (
        r#"p = provider()
def X():
    print(p(a=1))
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"P = provider()
def X():
    print(P.a)
X()"#,
        Err(r#"'Provider' value has no field or method 'a'"#),
    ),
    (
        r#"P = provider()
def X():
    P.x = 1
X()"#,
        Err(r#"cannot set .x field of Provider value"#),
    ),
    (
        r#"P = provider()
def X():
    print(P[0])
X()"#,
        Err(r#"type 'Provider' has no operator [](int)"#),
    ),
    (
        r#"P = provider()
def X():
    print(len(P))
X()"#,
        Err(
            r#"in call to len(), parameter 'x' got value of type 'Provider', want 'iterable or string'"#,
        ),
    ),
    (
        r#"P = provider()
def X():
    print({P: 1})
X()"#,
        Ok(r#"{<provider>: 1}"#),
    ),
    (
        r#"P = provider()
def X():
    print({P(a=1): 1})
X()"#,
        Ok(r#"{struct(a = 1): 1}"#),
    ),
    (
        r#"P = provider()
def X():
    print({P(a=[1]): 1})
X()"#,
        Err(r#"unhashable type: 'struct'"#),
    ),
    (
        r#"P = provider()
def X():
    print(json.encode(P(a=1)))
X()"#,
        Ok(r#"{"a":1}"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=1).to_json())
X()"#,
        Err(r#"'P' value has no field or method 'to_json'
Available attributes: a"#),
    ),
    (
        r#"P = provider()
def X():
    print(struct(a=P(a=1)))
X()"#,
        Ok(r#"struct(a = struct(a = 1))"#),
    ),
    (
        r#"P = provider()
def X():
    print(struct(a=P))
X()"#,
        Ok(r#"struct(a = <provider>)"#),
    ),
    (
        r#"P = provider()
def X():
    print(depset([P(a=1)]))
X()"#,
        Ok(r#"depset([struct(a = 1)])"#),
    ),
    (
        r#"P = provider()
def X():
    print(depset([P]))
X()"#,
        Ok(r#"depset([<provider>])"#),
    ),
    (
        r#"P = provider()
def X():
    print([P, P()])
X()"#,
        Ok(r#"[<provider>, struct()]"#),
    ),
    (
        r#"P = provider()
def X():
    print(str([P, P(a=1)]))
X()"#,
        Ok(r#"[<provider>, struct(a = 1)]"#),
    ),
    (
        r#"P = provider()
def X():
    print(provider()(a=1))
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"P = provider()
def X():
    print(provider())
X()"#,
        Ok(r#"<provider>"#),
    ),
    (
        r#"P = provider()
def X():
    print(bool(P(a=1)))
X()"#,
        Ok(r#"True"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=1) == struct(a=1))
X()"#,
        Ok(r#"False"#),
    ),
    (
        r#"P = provider()
def X():
    print(struct(a=1) == P(a=1))
X()"#,
        Ok(r#"False"#),
    ),
    (
        r#"P = provider()
Q = provider()
def X():
    print(P(a=1) == Q(a=1))
X()"#,
        Ok(r#"False"#),
    ),
    (
        r#"P = provider()
def X():
    print(hash(P(a=1)) == hash(P(a=2)))
X()"#,
        Err(r#"in call to hash(), parameter 'value' got value of type 'struct', want 'string'"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(b=1, a=2))
X()"#,
        Ok(r#"struct(a = 2, b = 1)"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=1, a=2))
X()"#,
        Err(r#"duplicate keyword argument: a"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(*[1]))
X()"#,
        Err(r#"P: unexpected positional arguments"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(**1))
X()"#,
        Err(r#"argument after ** must be a dict, not int"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(**{1:2}))
X()"#,
        Err(r#"keywords must be strings, not int"#),
    ),
    (
        r#"P = provider(fields=["a"])
def X():
    P(a=1, b=2)
X()"#,
        Err(r#"got unexpected field 'b' in call to instantiate provider P"#),
    ),
    (
        r#"P = provider(fields=["a"])
def X():
    P(b=2)
X()"#,
        Err(r#"got unexpected field 'b' in call to instantiate provider P"#),
    ),
    (
        r#"P = provider(fields=["a"])
def X():
    P(b=2, c=3)
X()"#,
        Err(r#"got unexpected fields 'b', 'c' in call to instantiate provider P"#),
    ),
    (
        r#"P = provider(fields=["a"])
def X():
    P(c=1, b=2, d=3)
X()"#,
        Err(r#"got unexpected fields 'c', 'b', 'd' in call to instantiate provider P"#),
    ),
    (
        r#"P = provider(fields=["a"])
def X():
    print(P(a=1))
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"P = provider(fields=["a"])
def X():
    print(P().a)
X()"#,
        Err(r#"'P' value has no field or method 'a'
Available attributes: "#),
    ),
    (
        r#"P = provider(fields=["a"])
def X():
    print(P(a=1).b)
X()"#,
        Err(r#"'P' value has no field or method 'b'
Available attributes: a"#),
    ),
    (
        r#"P = provider(fields=["a"])
def X():
    P(1)
X()"#,
        Err(r#"P: unexpected positional arguments"#),
    ),
    (
        r#"P = provider(fields=["a"])
def X():
    x = P(a=1)
    x.a = 2
X()"#,
        Err(r#"struct value does not support field assignment"#),
    ),
    (
        r#"P = provider(fields=["a"])
def X():
    print(P)
X()"#,
        Ok(r#"<provider>"#),
    ),
    (
        r#"P = provider(fields=["a"])
def X():
    print(P(a=1) == P(a=1))
X()"#,
        Ok(r#"True"#),
    ),
    (
        r#"P = provider(fields=["a"])
def X():
    print(P(a=1) + P(a=2))
X()"#,
        Err(r#"cannot add struct instances with common field 'a'"#),
    ),
    (
        r#"P = provider(fields=["a"])
def X():
    print(P(a=1) + P())
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"P = provider(fields=["a"])
Q = provider(fields=["b"])
def X():
    print(P(a=1) + Q(b=1))
X()"#,
        Err(r#"Cannot use '+' operator on instances of different providers (P and Q)"#),
    ),
    (
        r#"P = provider()
def X():
    print(P(a=1) + struct(b=1))
X()"#,
        Err(r#"Cannot use '+' operator on instances of different providers (P and struct)"#),
    ),
    (
        r#"P = provider()
def X():
    print(struct(b=1) + P(a=1))
X()"#,
        Err(r#"Cannot use '+' operator on instances of different providers (struct and P)"#),
    ),
    (
        r#"P = provider()
def X():
    print(struct(a=1) + P(a=1))
X()"#,
        Err(r#"Cannot use '+' operator on instances of different providers (struct and P)"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(P(1))
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(R(a=1))
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(R(1))
X()"#,
        Err(r#"<raw constructor for P>: unexpected positional arguments"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(P(1) == R(a=1))
X()"#,
        Ok(r#"True"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(P(x=1))
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(P())
X()"#,
        Err(r#"lambda() missing 1 required positional argument: x"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(P(1, 2))
X()"#,
        Err(r#"lambda() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(P(y=2))
X()"#,
        Err(r#"lambda() got unexpected keyword argument: y"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(R)
X()"#,
        Ok(
            r#"<unknown object com.google.devtools.build.lib.packages.StarlarkProvider$RawConstructor>"#,
        ),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(type(R))
X()"#,
        Ok(r#"RawConstructor"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(dir(R))
X()"#,
        Ok(r#"[]"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(R == R)
X()"#,
        Ok(r#"True"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(P == R)
X()"#,
        Ok(r#"False"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(P)
X()"#,
        Ok(r#"<provider>"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(R(b=1))
X()"#,
        Ok(r#"struct(b = 1)"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(R())
X()"#,
        Ok(r#"struct()"#),
    ),
    (
        r#"P, R = provider(init=lambda x: 1)
def X():
    print(P(1))
X()"#,
        Err(r#"got int for 'return value of provider init()', want dict"#),
    ),
    (
        r#"P, R = provider(init=lambda x: None)
def X():
    print(P(1))
X()"#,
        Err(r#"got NoneType for 'return value of provider init()', want dict"#),
    ),
    (
        r#"P, R = provider(init=lambda x: [])
def X():
    print(P(1))
X()"#,
        Err(r#"got list for 'return value of provider init()', want dict"#),
    ),
    (
        r#"P, R = provider(init=lambda x: ())
def X():
    print(P(1))
X()"#,
        Err(r#"got tuple for 'return value of provider init()', want dict"#),
    ),
    (
        r#"P, R = provider(init=lambda x: (1,))
def X():
    print(P(1))
X()"#,
        Err(r#"got tuple for 'return value of provider init()', want dict"#),
    ),
    (
        r#"P, R = provider(init=lambda x: struct(a=x))
def X():
    print(P(1))
X()"#,
        Err(r#"got struct for 'return value of provider init()', want dict"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {1: x})
def X():
    print(P(1))
X()"#,
        Err(
            r#"got dict<int, int> for 'return value of provider init()', want dict<string, unknown>"#,
        ),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(P(x=[1]))
X()"#,
        Ok(r#"struct(a = [1])"#),
    ),
    (
        r#"P, R = provider(init=lambda *a, **k: k)
def X():
    print(P(a=1, b=2))
X()"#,
        Ok(r#"struct(a = 1, b = 2)"#),
    ),
    (
        r#"P, R = provider(init=lambda *a, **k: k)
def X():
    print(P(1, a=1))
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"P, R = provider(init=lambda *a, **k: {"a": a})
def X():
    print(P(1, 2))
X()"#,
        Ok(r#"struct(a = (1, 2))"#),
    ),
    (
        r#"P, R = provider(fields=["a"], init=lambda x: {"a": x})
def X():
    print(P(1))
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"P, R = provider(fields=["a"], init=lambda x: {"b": x})
def X():
    print(P(1))
X()"#,
        Err(r#"got unexpected field 'b' in call to instantiate provider P"#),
    ),
    (
        r#"P, R = provider(fields=["a"], init=lambda x: {"b": x})
def X():
    print(R(b=1))
X()"#,
        Err(r#"got unexpected field 'b' in call to instantiate provider P"#),
    ),
    (
        r#"P, R = provider(fields=["a"], init=lambda x: {"a": x})
def X():
    print(R(a=1))
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"P, R = provider(fields=["a"], init=lambda x: {"a": x})
def X():
    print(R(a=1, b=2))
X()"#,
        Err(r#"got unexpected field 'b' in call to instantiate provider P"#),
    ),
    (
        r#"P, R = provider(fields=["a"], init=lambda x: {"a": x})
def X():
    print(P(1, b=2))
X()"#,
        Err(r#"lambda() got unexpected keyword argument: b"#),
    ),
    (
        r#"P, R = provider(init=lambda **k: k)
def X():
    print(P(a=1))
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"P, R = provider(init=lambda **k: k)
def X():
    print(P(**{"a": 1}))
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"P, R = provider(init=lambda: {"a": 1})
def X():
    print(P())
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"P, R = provider(init=lambda: {})
def X():
    print(P())
X()"#,
        Ok(r#"struct()"#),
    ),
    (
        r#"P, R = provider(init=lambda: {})
def X():
    print(P(a=1))
X()"#,
        Err(r#"lambda() got unexpected keyword argument: a"#),
    ),
    (
        r#"P, R = provider(init=lambda: fail("boom"))
def X():
    print(P())
X()"#,
        Err(r#"boom"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(P(1).a)
X()"#,
        Ok(r#"1"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(R(a=1) + P(1))
X()"#,
        Err(r#"cannot add struct instances with common field 'a'"#),
    ),
    (
        r#"P, R = provider(init=lambda x, *, y=2: {"a": x, "b": y})
def X():
    print(P(1))
X()"#,
        Ok(r#"struct(a = 1, b = 2)"#),
    ),
    (
        r#"P, R = provider(init=lambda x, *, y=2: {"a": x, "b": y})
def X():
    print(P(1, y=5))
X()"#,
        Ok(r#"struct(a = 1, b = 5)"#),
    ),
    (
        r#"P, R = provider(init=lambda x, *, y=2: {"a": x, "b": y})
def X():
    print(P(1, z=5))
X()"#,
        Err(r#"lambda() got unexpected keyword argument: z"#),
    ),
    (
        r#"P, R = provider(init=lambda x: {"a": x})
def X():
    print(type(P))
X()"#,
        Ok(r#"Provider"#),
    ),
    (
        r#"P = provider(init=print)
def X():
    print(P)
X()"#,
        Ok(
            r#"(<provider>, <unknown object com.google.devtools.build.lib.packages.StarlarkProvider$RawConstructor>)"#,
        ),
    ),
    (
        r#"P = provider(init=lambda: {})
def X():
    print(P)
X()"#,
        Ok(
            r#"(<provider>, <unknown object com.google.devtools.build.lib.packages.StarlarkProvider$RawConstructor>)"#,
        ),
    ),
    (
        r#"Q = provider(init=lambda: {})
def X():
    print(Q)
X()"#,
        Ok(
            r#"(<provider>, <unknown object com.google.devtools.build.lib.packages.StarlarkProvider$RawConstructor>)"#,
        ),
    ),
    (
        r#"P = provider(init=lambda: {})[0]
def X():
    print(P)
X()"#,
        Ok(r#"<provider>"#),
    ),
    (
        r#"P = provider(init=lambda: {})[0]
def X():
    print(P())
X()"#,
        Ok(r#"struct()"#),
    ),
    (
        r#"P, R = provider(init=lambda: {})
def X():
    print(P.__init__)
X()"#,
        Err(r#"'Provider' value has no field or method '__init__'"#),
    ),
    (
        r#"P = provider()
print(P)
print(type(P))
print(P(a=1))
print(P == P)
print(dir(P))"#,
        Ok(r#"<provider>
Provider
struct(a = 1)
True
[]"#),
    ),
    (r#"print(provider())"#, Ok(r#"<provider>"#)),
    (r#"print(provider(doc="x"))"#, Ok(r#"<provider>"#)),
    (r#"print(provider("x"))"#, Ok(r#"<provider>"#)),
    (
        r#"print(provider(1))"#,
        Err(
            r#"in call to provider(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(doc=1))"#,
        Err(
            r#"in call to provider(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (r#"print(provider(doc=None))"#, Ok(r#"<provider>"#)),
    (
        r#"print(provider(fields=1))"#,
        Err(
            r#"in call to provider(), parameter 'fields' got value of type 'int', want 'sequence, dict, or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(fields="a"))"#,
        Err(
            r#"in call to provider(), parameter 'fields' got value of type 'string', want 'sequence, dict, or NoneType'"#,
        ),
    ),
    (r#"print(provider(fields=None))"#, Ok(r#"<provider>"#)),
    (r#"print(provider(fields=[]))"#, Ok(r#"<provider>"#)),
    (r#"print(provider(fields=["a"]))"#, Ok(r#"<provider>"#)),
    (r#"print(provider(fields=("a",)))"#, Ok(r#"<provider>"#)),
    (r#"print(provider(fields={"a":"d"}))"#, Ok(r#"<provider>"#)),
    (
        r#"print(provider(fields={"a":1}))"#,
        Err(r#"got dict<string, int> for 'fields', want dict<string, string>"#),
    ),
    (
        r#"print(provider(fields={1:"a"}))"#,
        Err(r#"got dict<int, string> for 'fields', want dict<string, string>"#),
    ),
    (
        r#"print(provider(fields=[1]))"#,
        Err(r#"at index 0 of fields, got element of type int, want string"#),
    ),
    (
        r#"print(provider(fields=["a", 1]))"#,
        Err(r#"at index 1 of fields, got element of type int, want string"#),
    ),
    (
        r#"print(provider(fields=depset(["a"])))"#,
        Err(
            r#"in call to provider(), parameter 'fields' got value of type 'depset', want 'sequence, dict, or NoneType'"#,
        ),
    ),
    (r#"print(provider(fields=["a b"]))"#, Ok(r#"<provider>"#)),
    (r#"print(provider(fields=[""]))"#, Ok(r#"<provider>"#)),
    (r#"print(provider(fields=["1a"]))"#, Ok(r#"<provider>"#)),
    (r#"print(provider(fields={}))"#, Ok(r#"<provider>"#)),
    (
        r#"print(provider(fields=set(["a"])))"#,
        Err(
            r#"in call to provider(), parameter 'fields' got value of type 'set', want 'sequence, dict, or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(fields=range(3)))"#,
        Err(r#"at index 0 of fields, got element of type int, want string"#),
    ),
    (
        r#"print(provider(init=1))"#,
        Err(
            r#"in call to provider(), parameter 'init' got value of type 'int', want 'callable or NoneType'"#,
        ),
    ),
    (r#"print(provider(init=None))"#, Ok(r#"<provider>"#)),
    (
        r#"print(provider(init=print))"#,
        Ok(
            r#"(<provider>, <unknown object com.google.devtools.build.lib.packages.StarlarkProvider$RawConstructor>)"#,
        ),
    ),
    (
        r#"print(provider(init=lambda: {}))"#,
        Ok(
            r#"(<provider>, <unknown object com.google.devtools.build.lib.packages.StarlarkProvider$RawConstructor>)"#,
        ),
    ),
    (
        r#"print(provider(1, 2))"#,
        Err(
            r#"in call to provider(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(1, 2, 3))"#,
        Err(
            r#"in call to provider(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(doc="x", fields=["a"], init=lambda: {}))"#,
        Ok(
            r#"(<provider>, <unknown object com.google.devtools.build.lib.packages.StarlarkProvider$RawConstructor>)"#,
        ),
    ),
    (
        r#"print(provider(unknown=1))"#,
        Err(r#"provider() got unexpected keyword argument 'unknown'"#),
    ),
    (
        r#"print(provider(doc="a", doc="b"))"#,
        Err(r#"duplicate keyword argument: doc"#),
    ),
    (
        r#"print(provider(fielsd=[]))"#,
        Err(r#"provider() got unexpected keyword argument 'fielsd' (did you mean 'fields'?)"#),
    ),
    (
        r#"print(provider(fields=["a"], fields=[]))"#,
        Err(r#"duplicate keyword argument: fields"#),
    ),
    (r#"print(type(provider(init=lambda: {})))"#, Ok(r#"tuple"#)),
    (r#"print(len(provider(init=lambda: {})))"#, Ok(r#"2"#)),
    (
        r#"print(provider(init=lambda: {})[0])"#,
        Ok(r#"<provider>"#),
    ),
    (
        r#"print(provider(init=lambda: {})[1])"#,
        Ok(
            r#"<unknown object com.google.devtools.build.lib.packages.StarlarkProvider$RawConstructor>"#,
        ),
    ),
    (
        r#"print(provider(fields=["a"])(a=1, b=2))"#,
        Err(r#"got unexpected field 'b' in call to instantiate provider <no name>"#),
    ),
    (
        r#"print(provider(fields=["a"])(b=2))"#,
        Err(r#"got unexpected field 'b' in call to instantiate provider <no name>"#),
    ),
    (r#"print(provider(fields=["a"])())"#, Ok(r#"struct()"#)),
    (
        r#"print(provider(fields=["a"])(a=1))"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (r#"print(provider(fields=["a"])(a=1).a)"#, Ok(r#"1"#)),
    (
        r#"print(provider(fields=["a"])().a)"#,
        Err(r#"'struct' value has no field or method 'a'
Available attributes: "#),
    ),
    (
        r#"print(provider(fields=["a"])(a=1) == provider(fields=["a"])(a=1))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(provider(fields=["a", "b"])(b=1, a=2))"#,
        Ok(r#"struct(a = 2, b = 1)"#),
    ),
    (
        r#"print(dir(provider(fields=["a", "b"])(a=1)))"#,
        Ok(r#"["a"]"#),
    ),
    (r#"print(dir(provider(fields=["a", "b"])()))"#, Ok(r#"[]"#)),
    (
        r#"print(hasattr(provider(fields=["a"])(), "a"))"#,
        Ok(r#"False"#),
    ),
    (
        r#"print(getattr(provider(fields=["a"])(), "a", 7))"#,
        Ok(r#"7"#),
    ),
    (
        r#"print(provider(fields={"a": "doc"})(a=1))"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"print(provider(fields={"a": "doc"})(b=1))"#,
        Err(r#"got unexpected field 'b' in call to instantiate provider <no name>"#),
    ),
    (
        r#"print(provider(fields=["a"])(b=1, c=2))"#,
        Err(r#"got unexpected fields 'b', 'c' in call to instantiate provider <no name>"#),
    ),
    (
        r#"print(provider(fields=["a"])(c=1, b=2, d=3))"#,
        Err(r#"got unexpected fields 'c', 'b', 'd' in call to instantiate provider <no name>"#),
    ),
    (
        r#"print(provider(fields=["a"])(a=1, a=2))"#,
        Err(r#"duplicate keyword argument: a"#),
    ),
    (
        r#"print(provider(fields=["a"])(a=None))"#,
        Ok(r#"struct(a = None)"#),
    ),
    (
        r#"print(provider(fields=["a"])(a=None) == provider(fields=["a"])())"#,
        Ok(r#"False"#),
    ),
    (
        r#"P = provider()
A = attr.label(providers=[P])
def X(): print(A)
X()"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"P = provider()
A = attr.label(providers=[[P]])
def X(): print(A)
X()"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"P = provider()
print(P(a=1))
print(P(a=1).b)
def X(): pass
X()"#,
        Err(r#"'P' value has no field or method 'b'
Available attributes: a"#),
    ),
    (
        r#"P = provider(fields=["a"])
print(P(b=1))
def X(): pass
X()"#,
        Err(r#"got unexpected field 'b' in call to instantiate provider P"#),
    ),
    (
        r#"P = provider(fields=["a"])
I = P(a=1)
def X(): print(I.b)
X()"#,
        Err(r#"'P' value has no field or method 'b'
Available attributes: a"#),
    ),
    (
        r#"P = provider(fields=["a"])
I = P(a=1)
print(I.b)
def X(): pass
X()"#,
        Err(r#"'P' value has no field or method 'b'
Available attributes: a"#),
    ),
    (
        r#"def f(): return provider()
def X(): print(attr.label(providers=[f()]))
X()"#,
        Err(r#"Providers should be top-level values in extension files that define them."#),
    ),
    (
        r#"P = provider()
print(attr.label(providers=[P]))
def X(): pass
X()"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"P = provider()
print(attr.label(providers=[[P]]))
def X(): pass
X()"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"P = provider()
R = rule(implementation=lambda ctx: [P()], attrs={"d": attr.label(providers=[P])})
def X(): pass
X()"#,
        Ok(r#""#),
    ),
    (
        r#"P = provider()
R = rule(implementation=lambda ctx: [], provides=[P])
def X(): pass
X()"#,
        Ok(r#""#),
    ),
    (
        r#"R = rule(implementation=lambda ctx: [], provides=[provider()])
def X(): pass
X()"#,
        Err(r#"Providers should be top-level values in extension files that define them."#),
    ),
    (
        r#"P = provider(fields=["a"])
print(P)
print(P(a=1))
def X(): print(P)
X()"#,
        Ok(r#"<provider>
struct(a = 1)
<provider>"#),
    ),
    (
        r#"P = provider()
P2 = P
def X(): print(P2 == P)
X()"#,
        Ok(r#"True"#),
    ),
    (
        r#"P = provider()
print(type(P))
print(str(P))
print(repr(P))
print(P == P)
print(P != P)
print(P < P)
def X(): pass
X()"#,
        Err(r#"unsupported comparison: Provider <=> Provider"#),
    ),
    (
        r#"P = provider()
print(P == 1)
print(P == None)
print(1 == P)
print(P in [P])
print(P in {P: 1})
print({P: 1}[P])
def X(): pass
X()"#,
        Ok(r#"False
False
False
True
True
1"#),
    ),
    (
        r#"P = provider()
print(P(a=1) in [P(a=1)])
print(P(a=1) in {P(a=1): 1})
def X(): pass
X()"#,
        Ok(r#"True
True"#),
    ),
    (
        r#"P = provider()
print(P(a=1) == P(a=1.0))
print(P(a=1) == P(a=[1]))
print(P(a=[1]) == P(a=[1]))
print(P(a=[1]) == P(a=(1,)))
def X(): pass
X()"#,
        Ok(r#"True
False
True
False"#),
    ),
    (
        r#"P = provider()
print(P(a=1, b=2) == P(b=2, a=1))
print(P() == P())
def X(): pass
X()"#,
        Ok(r#"True
True"#),
    ),
    (
        r#"P = provider()
print(P(a=1) != P(a=2))
print(P(a=1) != P(a=1))
print(P(a=1) != 1)
def X(): pass
X()"#,
        Ok(r#"True
False
True"#),
    ),
    (
        r#"P = provider()
print(P(a=1) == 1)
print(1 == P(a=1))
print(P(a=1) == None)
print(P(a=1) == struct(a=1))
def X(): pass
X()"#,
        Ok(r#"False
False
False
False"#),
    ),
    (
        r#"P = provider()
print(P(a=1) + 1)
def X(): pass
X()"#,
        Err(r#"unsupported binary operation: struct + int"#),
    ),
    (
        r#"P = provider()
print(1 + P(a=1))
def X(): pass
X()"#,
        Err(r#"unsupported binary operation: int + struct"#),
    ),
    (
        r#"P = provider()
print(P(a=1) - P(a=1))
def X(): pass
X()"#,
        Err(r#"unsupported binary operation: struct - struct"#),
    ),
    (
        r#"P = provider()
print(P(a=1) * 2)
def X(): pass
X()"#,
        Err(r#"unsupported binary operation: struct * int"#),
    ),
    (
        r#"P = provider()
print(P(a=1) | P(b=1))
def X(): pass
X()"#,
        Err(r#"unsupported binary operation: struct | struct"#),
    ),
    (
        r#"P = provider()
print(-P(a=1))
def X(): pass
X()"#,
        Err(r#"unsupported unary operation: -struct"#),
    ),
    (
        r#"P = provider()
print(P(a=1)[0])
def X(): pass
X()"#,
        Err(r#"type 'struct' has no operator [](int)"#),
    ),
    (
        r#"P = provider()
print(P(a=1)[:])
def X(): pass
X()"#,
        Err(r#"invalid slice operand: struct"#),
    ),
    (
        r#"P = provider()
print(getattr(P(a=1), "b"))
def X(): pass
X()"#,
        Err(r#"'P' value has no field or method 'b'
Available attributes: a"#),
    ),
    (
        r#"P = provider()
print(getattr(P, "b"))
def X(): pass
X()"#,
        Err(r#"'Provider' value has no field or method 'b'"#),
    ),
    (
        r#"P = provider()
print(getattr(P, "b", 1))
def X(): pass
X()"#,
        Ok(r#"1"#),
    ),
    (
        r#"P = provider()
print(hasattr(P, "b"))
print(dir(P))
def X(): pass
X()"#,
        Ok(r#"False
[]"#),
    ),
    (
        r#"P = provider()
print(str(P(a="x")))
print(repr(P(a="x")))
print(str(P(a=P)))
print(P(a=[P()]))
def X(): pass
X()"#,
        Ok(r#"struct(a = "x")
struct(a = "x")
struct(a = <provider>)
struct(a = [struct()])"#),
    ),
    (
        r#"P = provider()
print(P(a=1).to_proto())
def X(): pass
X()"#,
        Err(r#"'P' value has no field or method 'to_proto'
Available attributes: a"#),
    ),
    (
        r#"P = provider()
print(json.encode(P(a=1, b=[1])))
print(json.encode(P()))
print(json.encode_indent(P(a=1)))
def X(): pass
X()"#,
        Ok(r#"{"a":1,"b":[1]}
{}
{"#),
    ),
    (
        r#"P = provider()
print(json.encode(P(a=P(b=1))))
def X(): pass
X()"#,
        Ok(r#"{"a":{"b":1}}"#),
    ),
    (
        r#"P = provider()
print(json.encode(P))
def X(): pass
X()"#,
        Err(r#"cannot encode Provider as JSON"#),
    ),
    (
        r#"P = provider()
print(proto.encode_text(P(a=1)))
def X(): pass
X()"#,
        Ok(r#"a: 1"#),
    ),
    (
        r#"P = provider()
print(P(x=1, y=2, z=3))
print(P(z=1, y=2, x=3))
print(P(b=1, a=2))
def X(): pass
X()"#,
        Ok(r#"struct(x = 1, y = 2, z = 3)
struct(x = 3, y = 2, z = 1)
struct(a = 2, b = 1)"#),
    ),
    (
        r#"P = provider()
print(bool(P()))
print(not P())
print(not P)
def X(): pass
X()"#,
        Ok(r#"True
False
False"#),
    ),
    (
        r#"P = provider()
print(P.__class__)
def X(): pass
X()"#,
        Err(r#"'Provider' value has no field or method '__class__'"#),
    ),
    (
        r#"P = provider()
print(dir(P()))
print(dir(P(a=1, b=2)))
def X(): pass
X()"#,
        Ok(r#"[]
["a", "b"]"#),
    ),
    (
        r#"print(provider("x", ["a"]))
def X(): pass
X()"#,
        Err(r#"provider() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(provider("x", fields=["a"]))
def X(): pass
X()"#,
        Ok(r#"<provider>"#),
    ),
    (
        r#"print(provider("x", ["a"], lambda: {}))
def X(): pass
X()"#,
        Err(r#"provider() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"print(provider(fields=["a"], init=None)(a=1))
def X(): pass
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"print(provider()(1))
def X(): pass
X()"#,
        Err(r#"<no name>: unexpected positional arguments"#),
    ),
    (
        r#"print(provider() + provider())
def X(): pass
X()"#,
        Err(r#"unsupported binary operation: Provider + Provider"#),
    ),
    (
        r#"print(provider()(a=1) + provider()(b=1))
def X(): pass
X()"#,
        Err(
            r#"Cannot use '+' operator on instances of different providers (<no name> and <no name>)"#,
        ),
    ),
    (
        r#"P = provider()
print(P(a=1) + provider()(b=1))
def X(): pass
X()"#,
        Err(r#"Cannot use '+' operator on instances of different providers (P and <no name>)"#),
    ),
    (
        r#"P = provider()
print(provider()(b=1) + P(a=1))
def X(): pass
X()"#,
        Err(r#"Cannot use '+' operator on instances of different providers (<no name> and P)"#),
    ),
    (
        r#"print(provider()(b=1) + struct(a=1))
def X(): pass
X()"#,
        Err(
            r#"Cannot use '+' operator on instances of different providers (<no name> and struct)"#,
        ),
    ),
    (
        r#"print(struct(a=1) + provider()(b=1))
def X(): pass
X()"#,
        Err(
            r#"Cannot use '+' operator on instances of different providers (struct and <no name>)"#,
        ),
    ),
    (
        r#"p, r = provider(init=lambda: {})
print(r(1))
def X(): pass
X()"#,
        Err(r#"<raw constructor for p>: unexpected positional arguments"#),
    ),
    (
        r#"print(provider(init=lambda: {})[1](1))
def X(): pass
X()"#,
        Err(r#"<raw constructor>: unexpected positional arguments"#),
    ),
    (
        r#"print(provider(fields=["a"], init=lambda: {})[1](b=1))
def X(): pass
X()"#,
        Err(r#"got unexpected field 'b' in call to instantiate provider <no name>"#),
    ),
    (
        r#"print(provider(fields=["a"], init=lambda: {})[0](b=1))
def X(): pass
X()"#,
        Err(r#"lambda() got unexpected keyword argument: b"#),
    ),
    (
        r#"print(provider(init=lambda: {"a": 1})[0]().a)
def X(): pass
X()"#,
        Ok(r#"1"#),
    ),
    (
        r#"print(provider(init=lambda x: x)[0]({}))
def X(): pass
X()"#,
        Ok(r#"struct()"#),
    ),
    (
        r#"print(provider(init=lambda x: x)[0]({"a": 1}))
def X(): pass
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"print(provider(init=lambda x: x)[0]({"a": 1, "b": 2}))
def X(): pass
X()"#,
        Ok(r#"struct(a = 1, b = 2)"#),
    ),
    (
        r#"print(provider(init=lambda x: x)[0]({"b": 1, "a": 2}))
def X(): pass
X()"#,
        Ok(r#"struct(a = 2, b = 1)"#),
    ),
    (
        r#"print(provider(init=lambda x: x)[0]({"a b": 1}))
def X(): pass
X()"#,
        Ok(r#"struct(a b = 1)"#),
    ),
    (
        r#"print(provider(init=lambda x: x)[0]({1: 1}))
def X(): pass
X()"#,
        Err(
            r#"got dict<int, int> for 'return value of provider init()', want dict<string, unknown>"#,
        ),
    ),
    (
        r#"print(provider(init=lambda x: x)[0]({"a": 1}.items()))
def X(): pass
X()"#,
        Err(r#"got list for 'return value of provider init()', want dict"#),
    ),
    (
        r#"print(provider(init=lambda x: x)[0](struct(a=1)))
def X(): pass
X()"#,
        Err(r#"got struct for 'return value of provider init()', want dict"#),
    ),
    (
        r#"print(provider(init=lambda x: x)[0](None))
def X(): pass
X()"#,
        Err(r#"got NoneType for 'return value of provider init()', want dict"#),
    ),
    (
        r#"print(provider(init=lambda x: {"a": x})[0](x=1, y=2))
def X(): pass
X()"#,
        Err(r#"lambda() got unexpected keyword argument: y"#),
    ),
    (
        r#"print(provider(init=len)[0]([1]))
def X(): pass
X()"#,
        Err(r#"got int for 'return value of provider init()', want dict"#),
    ),
    (
        r#"print(provider(init=dict)[0](a=1))
def X(): pass
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"print(provider(init=dict)[0]())
def X(): pass
X()"#,
        Ok(r#"struct()"#),
    ),
    (
        r#"print(provider(init=struct)[0]())
def X(): pass
X()"#,
        Err(
            r#"in call to provider(), parameter 'init' got value of type 'Provider', want 'callable or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(init=provider())[0]())
def X(): pass
X()"#,
        Err(r#"got struct for 'return value of provider init()', want dict"#),
    ),
    (
        r#"Q = provider()
print(provider(init=Q)[0]())
def X(): pass
X()"#,
        Err(r#"got struct for 'return value of provider init()', want dict"#),
    ),
    (
        r#"Q = provider(init=lambda: {})
print(provider(init=Q)[0]())
def X(): pass
X()"#,
        Err(
            r#"in call to provider(), parameter 'init' got value of type 'tuple', want 'callable or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(init=struct(a=1)))
def X(): pass
X()"#,
        Err(
            r#"in call to provider(), parameter 'init' got value of type 'struct', want 'callable or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(init=[]))
def X(): pass
X()"#,
        Err(
            r#"in call to provider(), parameter 'init' got value of type 'list', want 'callable or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(init=""))
def X(): pass
X()"#,
        Err(
            r#"in call to provider(), parameter 'init' got value of type 'string', want 'callable or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(init=1, fields=1))
def X(): pass
X()"#,
        Err(
            r#"in call to provider(), parameter 'init' got value of type 'int', want 'callable or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(fields=1, doc=1))
def X(): pass
X()"#,
        Err(
            r#"in call to provider(), parameter 'fields' got value of type 'int', want 'sequence, dict, or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(doc=1, fielsd=1))
def X(): pass
X()"#,
        Err(
            r#"in call to provider(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(fielsd=1, doc=1))
def X(): pass
X()"#,
        Err(r#"provider() got unexpected keyword argument 'fielsd' (did you mean 'fields'?)"#),
    ),
    (
        r#"print(provider(init=None, fields=None, doc=None))
def X(): pass
X()"#,
        Ok(r#"<provider>"#),
    ),
    (
        r#"print(provider(*["x"]))
def X(): pass
X()"#,
        Ok(r#"<provider>"#),
    ),
    (
        r#"print(provider(**{"doc": "x"}))
def X(): pass
X()"#,
        Ok(r#"<provider>"#),
    ),
    (
        r#"print(provider(**{"docs": "x"}))
def X(): pass
X()"#,
        Err(r#"provider() got unexpected keyword argument 'docs' (did you mean 'doc'?)"#),
    ),
    (
        r#"print(provider(1, 2, 3, 4))
def X(): pass
X()"#,
        Err(
            r#"in call to provider(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(fields=["a", "b"], init=lambda: {})[0](a=1))
def X(): pass
X()"#,
        Err(r#"lambda() got unexpected keyword argument: a"#),
    ),
    (
        r#"print(provider(fields={"a": "x", "b": "y"})(b=1))
def X(): pass
X()"#,
        Ok(r#"struct(b = 1)"#),
    ),
    (
        r#"print(provider(fields=["a"])(**{"a": 1}))
def X(): pass
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"print(provider(fields=["a"])(**{"b c": 1}))
def X(): pass
X()"#,
        Err(r#"got unexpected field 'b c' in call to instantiate provider <no name>"#),
    ),
    (
        r#"print(provider(fields=["a"])(**{"": 1}))
def X(): pass
X()"#,
        Err(r#"got unexpected field '' in call to instantiate provider <no name>"#),
    ),
    (
        r#"print(provider(fields=["", "a b"])(**{"": 1, "a b": 2}))
def X(): pass
X()"#,
        Ok(r#"struct( = 1, a b = 2)"#),
    ),
    (
        r#"print(provider()(**{"_x": 1, "X": 2, "a.b": 3}))
def X(): pass
X()"#,
        Ok(r#"struct(X = 2, _x = 1, a.b = 3)"#),
    ),
    (
        r#"print(provider()(**{"a": 1, "A": 2, "_": 3, "b": 4}))
def X(): pass
X()"#,
        Ok(r#"struct(A = 2, _ = 3, a = 1, b = 4)"#),
    ),
    (
        r#"print(provider)
print(type(provider))
print(provider.__name__)
def X(): pass
X()"#,
        Err(r#"'builtin_function_or_method' value has no field or method '__name__'"#),
    ),
    (
        r#"print(dir(provider))
def X(): pass
X()"#,
        Ok(r#"[]"#),
    ),
    (
        r#"P = provider()
print(P)
print(P(a=1))
def X(): pass
X()"#,
        Ok(r#"<provider>
struct(a = 1)"#),
    ),
    (
        r#"P = provider()
print(P(a=1) < P(a=2))
def X(): pass
X()"#,
        Err(r#"unsupported comparison: struct <=> struct"#),
    ),
    (
        r#"P = provider()
x = P(a=1)
print(x.a)
print(getattr(x, "a"))
print(hasattr(x, "a"))
print(dir(x))
print(hasattr(x, "to_json"))
def X(): pass
X()"#,
        Ok(r#"1
1
True
["a"]
False"#),
    ),
    (
        r#"struct(a=1).b = 2
def X(): pass
X()"#,
        Err(r#"struct value does not support field assignment"#),
    ),
    (
        r#"x = struct(a=1)
x.a = 2
def X(): pass
X()"#,
        Err(r#"struct value does not support field assignment"#),
    ),
    (
        r#"P = provider()
print(P.__doc__)
def X(): pass
X()"#,
        Err(r#"'Provider' value has no field or method '__doc__'"#),
    ),
    (
        r#"P = provider()
print(str(P))
print("%s" % P)
print("%r" % P)
print("%s" % P(a=1))
print("%r" % P(a=1))
def X(): pass
X()"#,
        Ok(r#"<provider>
<provider>
<provider>
struct(a = 1)
struct(a = 1)"#),
    ),
    (
        r#"P = provider()
print("%s" % struct(a=P))
def X(): pass
X()"#,
        Ok(r#"struct(a = <provider>)"#),
    ),
    (
        r#"P = provider()
print(P(a=1) == P(a=1))
print(P(a={"x":1}) == P(a={"x":1}))
print(P(a=P(b=1)) == P(a=P(b=1)))
def X(): pass
X()"#,
        Ok(r#"True
True
True"#),
    ),
    (
        r#"P = provider()
print(sorted([P(a=2), P(a=1)]))
def X(): pass
X()"#,
        Err(r#"unsupported comparison: struct <=> struct"#),
    ),
    (
        r#"P = provider()
print(P(a=depset([1])))
print(P(a=depset([1])) == P(a=depset([1])))
def X(): pass
X()"#,
        Ok(r#"struct(a = depset([1]))
False"#),
    ),
    (
        r#"P = provider()
print(max(P(a=1), P(a=2)))
def X(): pass
X()"#,
        Err(r#"unsupported comparison: struct <=> struct"#),
    ),
    (
        r#"P = provider()
print(list(P(a=1).__dict__))
def X(): pass
X()"#,
        Err(r#"'P' value has no field or method '__dict__'
Available attributes: a"#),
    ),
    (
        r#"P = provider(fields=["a"])
I = P(a=1)
print(I.a)
J = P()
print(hasattr(J, "a"))
print(getattr(J, "a", "d"))
def X(): pass
X()"#,
        Ok(r#"1
False
d"#),
    ),
    (
        r#"P = provider()
print(P(a=1, b=None))
print(dir(P(a=1, b=None)))
print(hasattr(P(b=None), "b"))
print(P(b=None).b)
def X(): pass
X()"#,
        Ok(r#"struct(a = 1, b = None)
["a", "b"]
True
None"#),
    ),
    (
        r#"print(provider(fields={"a": 1, "b": "x"}))
def X(): pass
X()"#,
        Err(r#"got dict<string, int> for 'fields', want dict<string, string>"#),
    ),
    (
        r#"print(provider(fields={"a": "x", "b": 1}))
def X(): pass
X()"#,
        Err(r#"got dict<string, int> for 'fields', want dict<string, string>"#),
    ),
    (
        r#"print(provider(fields={1: "x", "b": "y"}))
def X(): pass
X()"#,
        Err(r#"got dict<int, string> for 'fields', want dict<string, string>"#),
    ),
    (
        r#"print(provider(fields={"a": 1, "b": None}))
def X(): pass
X()"#,
        Err(r#"got dict<string, int> for 'fields', want dict<string, string>"#),
    ),
    (
        r#"print(provider(fields={"a": [1]}))
def X(): pass
X()"#,
        Err(r#"got dict<string, list> for 'fields', want dict<string, string>"#),
    ),
    (
        r#"print(provider(fields={"a": {"x": 1}}))
def X(): pass
X()"#,
        Err(r#"got dict<string, dict> for 'fields', want dict<string, string>"#),
    ),
    (
        r#"print(provider(fields={1: 1, "a": 1.5}))
def X(): pass
X()"#,
        Err(r#"got dict<int, int> for 'fields', want dict<string, string>"#),
    ),
    (
        r#"print(provider(fields={"a": None}))
def X(): pass
X()"#,
        Err(r#"got dict<string, NoneType> for 'fields', want dict<string, string>"#),
    ),
    (
        r#"print(provider(fields={None: "a"}))
def X(): pass
X()"#,
        Err(r#"got dict<NoneType, string> for 'fields', want dict<string, string>"#),
    ),
    (
        r#"print(provider(init=lambda: {1: 1, "a": 2}))[0]()
def X(): pass
X()"#,
        Err(r#"type 'NoneType' has no operator [](int)"#),
    ),
    (
        r#"print(provider(init=lambda: {1: 1, "a": 2})[0]())
def X(): pass
X()"#,
        Err(
            r#"got dict<int, int> for 'return value of provider init()', want dict<string, unknown>"#,
        ),
    ),
    (
        r#"print(provider(init=lambda: {1: 1, 2: "a"})[0]())
def X(): pass
X()"#,
        Err(
            r#"got dict<int, int> for 'return value of provider init()', want dict<string, unknown>"#,
        ),
    ),
    (
        r#"print(provider(init=lambda: {None: 1})[0]())
def X(): pass
X()"#,
        Err(
            r#"got dict<NoneType, int> for 'return value of provider init()', want dict<string, unknown>"#,
        ),
    ),
    (
        r#"print(provider(init=lambda: {(1,): 1})[0]())
def X(): pass
X()"#,
        Err(
            r#"got dict<tuple, int> for 'return value of provider init()', want dict<string, unknown>"#,
        ),
    ),
    (
        r#"print(provider(init=lambda: {"a": 1, 2: 1, "b": 3})[0]())
def X(): pass
X()"#,
        Err(
            r#"got dict<int, int> for 'return value of provider init()', want dict<string, unknown>"#,
        ),
    ),
    (
        r#"print(provider(fields=["a"], init=lambda: {"a": 1})[0](1))
def X(): pass
X()"#,
        Err(r#"lambda() does not accept positional arguments, but got 1"#),
    ),
    (
        r#"print(provider(fields=["a"])(1, b=2))
def X(): pass
X()"#,
        Err(r#"<no name>: unexpected positional arguments"#),
    ),
    (
        r#"print(provider(fields=["a"])(b=2, *[1]))
def X(): pass
X()"#,
        Err(r#"<no name>: unexpected positional arguments"#),
    ),
    (
        r#"print(provider(fields=[])(a=1))
def X(): pass
X()"#,
        Err(r#"got unexpected field 'a' in call to instantiate provider <no name>"#),
    ),
    (
        r#"print(provider(fields={})(a=1))
def X(): pass
X()"#,
        Err(r#"got unexpected field 'a' in call to instantiate provider <no name>"#),
    ),
    (
        r#"print(provider(fields=None)(a=1))
def X(): pass
X()"#,
        Ok(r#"struct(a = 1)"#),
    ),
    (
        r#"print(provider(fields=range(0))(a=1))
def X(): pass
X()"#,
        Err(r#"got unexpected field 'a' in call to instantiate provider <no name>"#),
    ),
    (
        r#"print(provider(fields=[""])(a=1))
def X(): pass
X()"#,
        Err(r#"got unexpected field 'a' in call to instantiate provider <no name>"#),
    ),
    (
        r#"print(provider("x", doc="y"))
def X(): pass
X()"#,
        Err(r#"provider() got multiple values for argument 'doc'"#),
    ),
    (
        r#"print(provider(1, doc="y"))
def X(): pass
X()"#,
        Err(
            r#"in call to provider(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(provider("x", doc=1))
def X(): pass
X()"#,
        Err(
            r#"in call to provider(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(doc="y", *["x"]))
def X(): pass
X()"#,
        Ok(r#"<provider>"#),
    ),
    (
        r#"print(provider(fields=["a"], init=lambda *a: {})[0](1, b=2))
def X(): pass
X()"#,
        Err(r#"lambda() got unexpected keyword argument: b"#),
    ),
    (
        r#"print(provider(fields=["a"], init=lambda *a: {"b": 1})[0](1))
def X(): pass
X()"#,
        Err(r#"got unexpected field 'b' in call to instantiate provider <no name>"#),
    ),
    (
        r#"print(provider(init=lambda *a, **k: {"a": (a, k)})[0](1, b=2))
def X(): pass
X()"#,
        Ok(r#"struct(a = ((1,), {"b": 2}))"#),
    ),
    (
        r#"print(provider(fields=[1, 1]))
def X(): pass
X()"#,
        Err(r#"at index 0 of fields, got element of type int, want string"#),
    ),
    (
        r#"print(provider(fields={"a": "x"}, doc=1))
def X(): pass
X()"#,
        Err(
            r#"in call to provider(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(fields=[1], init=1))
def X(): pass
X()"#,
        Err(
            r#"in call to provider(), parameter 'init' got value of type 'int', want 'callable or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(fields=[1], doc=1))
def X(): pass
X()"#,
        Err(
            r#"in call to provider(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"print(provider(fields=["a"], fields=["b"]))
def X(): pass
X()"#,
        Err(r#"duplicate keyword argument: fields"#),
    ),
    (
        r#"print(provider(fields=["a"], **{"fields": ["b"]}))
def X(): pass
X()"#,
        Err(r#"provider() got multiple values for argument 'fields'"#),
    ),
];
