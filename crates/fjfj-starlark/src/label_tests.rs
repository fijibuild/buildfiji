//! `Label` against Bazel 9.2.0 (buildfiji-mum.3.2): the table at the bottom
//! is what it printed and reported for the same `.bzl` bodies in the main
//! repo of a workspace with no dependencies, and the tests above it cover
//! what needs a dependency module, taken from probes in a workspace where
//! `probe` depends on `dep` (as `mydep`) and `other`, and `dep+` depends on
//! `other` (as `oth`).

use crate::test_support::{replay, replay_in};

/// Probes whose answer is the Starlark runtime's generic wording, which
/// belongs to buildfiji-v32.
const GENERIC: &[&str] = &[
    "has no field or method",
    "unsupported binary operation",
    "cannot set .name field",
    "in call to len()",
];

/// Probes with a known difference: `.name()` calls the attribute as if it
/// were a method (buildfiji-v32), and `%s` and an unnumbered `{}` format a
/// label with `repr` (buildfiji-b9c).
const SKIPPED: &[&str] = &[".name()", ".package()", "\"{}\".format(Label"];

fn assert_replays(wrong: Vec<String>) {
    assert!(
        wrong.is_empty(),
        "{} differ:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

#[test]
fn label_in_the_main_repo_is_bazels() {
    assert_replays(replay(LABEL_CASES, SKIPPED, GENERIC));
}

/// Every row was printed by Bazel 9.2.0 for a `.bzl` in the main repo of the
/// workspace described at the top, which names `dep` `mydep` and `other`
/// `other`, so `@mydep` and `@other` map and `@dep` and `@oth` do not.
const MAIN_MAPPING: &[(&str, Result<&str, &str>)] = &[
    (
        r#"print(str(Label("@mydep//a:b")), str(Label("@other//a:b")), str(Label("@probe//a:b")))"#,
        Ok("@@dep+//a:b @@other+//a:b @@//a:b"),
    ),
    (
        r#"print(str(Label("@dep//a:b")), str(Label("@oth//a:b")), str(Label("@oth")))"#,
        Ok(
            "@@[unknown repo 'dep' requested from @@]//a:b @@[unknown repo 'oth' requested from @@]//a:b @@[unknown repo 'oth' requested from @@]//:oth",
        ),
    ),
    (
        r#"print(str(Label("@mydep")), str(Label("@@dep+")), str(Label("@other")))"#,
        Ok("@@dep+//:mydep @@dep+//:dep+ @@other+//:other"),
    ),
    (
        r#"l = Label("@mydep//a:b");print(l.repo_name, l.workspace_name, l.workspace_root)"#,
        Ok("dep+ dep+ external/dep+"),
    ),
    (
        r#"l = Label("@other//a:b");print(l.repo_name, l.workspace_root)"#,
        Ok("other+ external/other+"),
    ),
    (
        r#"print(Label("@oth//a:b").repo_name)"#,
        Err(
            "'repo_name' is not allowed on invalid Label @@[unknown repo 'oth' requested from @@]//a:b",
        ),
    ),
    (
        r#"print(Label("@oth//a:b").workspace_root)"#,
        Err(
            "'workspace_root' is not allowed on invalid Label @@[unknown repo 'oth' requested from @@]//a:b",
        ),
    ),
    // `relative` names repos through the mapping of the file that calls it,
    // whichever repo the label is in.
    (
        r#"l = Label("//a:b");print(str(l.relative("@mydep//x:y")), str(l.relative("@oth//x:y")), str(l.relative("@probe//x:y")))"#,
        Ok("@@dep+//x:y @@[unknown repo 'oth' requested from @@]//x:y @@//x:y"),
    ),
    (
        r#"l = Label("@@dep+//a:b");print(str(l.relative("@oth//x:y")), str(l.relative("@mydep//x:y")), str(l.relative("@dep//x:y")))"#,
        Ok(
            "@@[unknown repo 'oth' requested from @@]//x:y @@dep+//x:y @@[unknown repo 'dep' requested from @@]//x:y",
        ),
    ),
    (
        r#"l = Label("@@dep+//a:b");print(str(l.relative(":c")), str(l.relative("c")), str(l.relative("//c")), str(l.relative("@@dep+//c")))"#,
        Ok("@@dep+//a:c @@dep+//a:c @@dep+//c:c @@dep+//c:c"),
    ),
    (
        r#"print(Label("@mydep//a:b") == Label("@@dep+//a:b"), Label("@mydep//a:b") == Label("@dep//a:b"))"#,
        Ok("True False"),
    ),
    (
        r#"print(repr(Label("@mydep//a:b")), repr(Label("@other//a:b")))"#,
        Ok(r#"Label("@@dep+//a:b") Label("@@other+//a:b")"#),
    ),
];

#[test]
fn a_repo_is_named_through_the_mapping_of_the_main_repo() {
    assert_replays(replay(MAIN_MAPPING, &[], &[]));
}

/// The same, for a `.bzl` in `dep+`, which calls itself `dep` and `other+`
/// `oth`, and cannot name `mydep`, `other` or the main repo's `probe`.
const DEP_MAPPING: &[(&str, Result<&str, &str>)] = &[
    (
        r#"print(str(Label("//a:b")), str(Label(":x")), str(Label("x")), str(Label("@@//a:b")))"#,
        Ok("@@dep+//a:b @@dep+//:x @@dep+//:x @@//a:b"),
    ),
    (
        // `@` is an apparent name like another, and `dep+` does not have it.
        r#"print(str(Label("@//a:b")))"#,
        Ok("@@[unknown repo '' requested from @@dep+]//a:b"),
    ),
    (
        r#"print(str(Label("@dep//a:b")), str(Label("@oth//a:b")), str(Label("@oth")))"#,
        Ok("@@dep+//a:b @@other+//a:b @@other+//:oth"),
    ),
    (
        // A near miss is suggested.
        r#"print(str(Label("@mydep//a:b")), str(Label("@other//a:b")))"#,
        Ok(
            "@@[unknown repo 'mydep' requested from @@dep+ (did you mean 'dep'?)]//a:b @@[unknown repo 'other' requested from @@dep+ (did you mean 'oth'?)]//a:b",
        ),
    ),
    (
        r#"print(str(Label("@probe//a:b")), str(Label("@x//a:b")))"#,
        Ok(
            "@@[unknown repo 'probe' requested from @@dep+]//a:b @@[unknown repo 'x' requested from @@dep+]//a:b",
        ),
    ),
    (
        r#"l = Label("//a:b");print(l.repo_name, l.workspace_name, l.workspace_root, l.package, l.name)"#,
        Ok("dep+ dep+ external/dep+ a b"),
    ),
    (
        r#"l = Label("@oth//a:b");print(l.repo_name, l.workspace_root)"#,
        Ok("other+ external/other+"),
    ),
    (
        r#"print(Label("@x//a:b").repo_name)"#,
        Err(
            "'repo_name' is not allowed on invalid Label @@[unknown repo 'x' requested from @@dep+]//a:b",
        ),
    ),
    (
        r#"l = Label("//a:b");print(str(l.relative("@oth//x:y")), str(l.relative("@dep//x:y")), str(l.relative(":z")), str(l.relative("z")))"#,
        Ok("@@other+//x:y @@dep+//x:y @@dep+//a:z @@dep+//a:z"),
    ),
    (
        r#"print(str(Label("@oth//a:b").relative("@dep//x:y")), str(Label("@@other+//a:b").relative("@oth//x:y")))"#,
        Ok("@@dep+//x:y @@other+//x:y"),
    ),
    (
        r#"print(repr(Label("//a:b")), repr(Label("@oth//a:b")), repr(Label("@@//a:b")))"#,
        Ok(r#"Label("@@dep+//a:b") Label("@@other+//a:b") Label("//a:b")"#),
    ),
    (
        r#"print(Label("//a:b") == Label("@dep//a:b"), Label("//a:b") == Label("@@dep+//a:b"), str(Label("//a:b")) == "@@dep+//a:b")"#,
        Ok("True True True"),
    ),
    (r#"print(Label("@@//a:b").repo_name == "")"#, Ok("True")),
];

#[test]
fn a_repo_is_named_through_the_mapping_of_the_repo_the_file_is_in() {
    assert_replays(replay_in("dep+", "", DEP_MAPPING, &[], &[]));
}

/// `:x` and `x` are in the package of the `.bzl`, wherever the BUILD file
/// that loaded it is.
const IN_A_PACKAGE: &[(&str, Result<&str, &str>)] = &[
    (
        r#"print(str(Label(":x")), str(Label("x")), str(Label("y/z")), str(Label("//a")), Label(":x").package, Label("x").name)"#,
        Ok("@@//sub/pkg:x @@//sub/pkg:x @@//sub/pkg:y/z @@//a:a sub/pkg x"),
    ),
    (
        r#"print(str(Label("//sub/pkg:b").relative(":c")), repr(Label(":x")))"#,
        Ok(r#"@@//sub/pkg:c Label("//sub/pkg:x")"#),
    ),
];

#[test]
fn a_bzl_in_a_package_resolves_relative_labels_there() {
    assert_replays(replay_in("", "sub/pkg", IN_A_PACKAGE, &[], &[]));
    let wrong = replay_in(
        "dep+",
        "sub/pkg",
        &[
            (
                r#"print(str(Label(":x")), str(Label("x")), str(Label("y/z")), str(Label("//a")), Label(":x").package)"#,
                Ok("@@dep+//sub/pkg:x @@dep+//sub/pkg:x @@dep+//sub/pkg:y/z @@dep+//a:a sub/pkg"),
            ),
            (
                r#"print(str(Label("//sub/pkg:b").relative(":c")), repr(Label("x")))"#,
                Ok(r#"@@dep+//sub/pkg:c Label("@@dep+//sub/pkg:x")"#),
            ),
        ],
        &[],
        &[],
    );
    assert_replays(wrong);
}

const LABEL_CASES: &[(&str, Result<&str, &str>)] = &[
    (
        r#"print(str(Label("//a:b")), repr(Label("//a:b")), type(Label("//a:b")))"#,
        Ok(r#"@@//a:b Label("//a:b") Label"#),
    ),
    (
        r#"print(str(Label("//a/b")), str(Label(":x")), str(Label("x")), str(Label("y/z")), str(Label("//:t")), str(Label("//a")))"#,
        Ok(r#"@@//a/b:b @@//:x @@//:x @@//:y/z @@//:t @@//a:a"#),
    ),
    (
        r#"print(str(Label("@//a:b")), str(Label("@@//a:b")), str(Label("@probe//a:b")), str(Label("@@probe//a:b")))"#,
        Ok(r#"@@//a:b @@//a:b @@//a:b @@probe//a:b"#),
    ),
    (
        r#"print(str(Label("@r//a:b")), str(Label("@@r//a:b")), str(Label("@r")), str(Label("@@r")), str(Label("@@r+x//a:b")))"#,
        Ok(
            r#"@@[unknown repo 'r' requested from @@]//a:b @@r//a:b @@[unknown repo 'r' requested from @@]//:r @@r//:r @@r+x//a:b"#,
        ),
    ),
    (
        r#"print(repr(Label("@r//a:b")), repr(Label("@@r//a:b")), repr(Label("@@//a:b")), repr(Label("@@probe//a:b")))"#,
        Ok(
            r#"Label("@@[unknown repo 'r' requested from @@]//a:b") Label("@@r//a:b") Label("//a:b") Label("@@probe//a:b")"#,
        ),
    ),
    (
        r#"print(repr(Label('//a:b"c')), str(Label('//a:b"c')))"#,
        Ok(r#"Label("//a:b\"c") @@//a:b"c"#),
    ),
    (
        r#"print(repr(Label('//a:é')), str(Label('//a:é')))"#,
        Ok(r#"Label("//a:é") @@//a:é"#),
    ),
    (
        r#"l = Label("//a/b:c")
print(l.name, l.package, repr(l.repo_name), repr(l.workspace_name), repr(l.workspace_root))"#,
        Ok(r#"c a/b "" "" """#),
    ),
    (
        r#"l = Label("@@r+//a/b:c")
print(l.name, l.package, l.repo_name, l.workspace_name, repr(l.workspace_root))"#,
        Ok(r#"c a/b r+ r+ "external/r+""#),
    ),
    (
        r#"l = Label("//:c")
print(repr(l.package), l.name)"#,
        Ok(r#""" c"#),
    ),
    (
        r#"print(type(Label("//a:b").name), type(Label("//a:b").package), type(Label("//a:b").repo_name))"#,
        Ok(r#"string string string"#),
    ),
    (
        r#"print(str(Label("@@r+//a/b:c").relative(":d")))"#,
        Ok(r#"@@r+//a/b:d"#),
    ),
    (
        r#"l = Label("//a/b:c")
print(str(l.relative(":d")), str(l.relative("//x:y")), str(l.relative("@@r//x:y")), str(l.relative("d")))"#,
        Ok(r#"@@//a/b:d @@//x:y @@r//x:y @@//a/b:d"#),
    ),
    (
        r#"l = Label("//a/b:c")
print(str(l.relative("@r//x:y")), str(l.relative("@probe//x:y")), str(l.relative("@//x:y")), str(l.relative("@@//x:y")))"#,
        Ok(r#"@@[unknown repo 'r' requested from @@]//x:y @@//x:y @@//x:y @@//x:y"#),
    ),
    (
        r#"l = Label("//a/b:c")
print(str(l.same_package_label("d")), str(l.same_package_label("d/e")))"#,
        Ok(r#"@@//a/b:d @@//a/b:d/e"#),
    ),
    (
        r#"l = Label("@@r+//a/b:c")
print(str(l.same_package_label("d")))"#,
        Ok(r#"@@r+//a/b:d"#),
    ),
    (
        r#"l = Label("//a/b:c")
print(Label(l) == l)"#,
        Ok(r#"True"#),
    ),
    (
        r#"l = Label("//a/b:c")
print(l == Label("//a/b:c"), l == "//a/b:c", l != Label("//a/b:d"), l == None, l != 1)"#,
        Ok(r#"True False True False True"#),
    ),
    (
        r#"print(Label("@@r//a:b") == Label("@r//a:b"), Label("//a:b") == Label("@@probe//a:b"), Label("//a:b") == Label("@@//a:b"), Label("@r//a:b") == Label("@r//a:b"), Label("@r//a:b") == Label("@x//a:b"))"#,
        Ok(r#"False False True True False"#),
    ),
    (r#"print({Label("//a:b"): 1}[Label("//a:b")])"#, Ok(r#"1"#)),
    (
        r#"print({Label("//a:b"): 1, Label("//a:c"): 2})"#,
        Ok(r#"{Label("//a:b"): 1, Label("//a:c"): 2}"#),
    ),
    (
        r#"print(depset([Label("//a:b"), Label("//a:b"), Label("//a:c")]))"#,
        Ok(r#"depset([Label("//a:b"), Label("//a:c")])"#),
    ),
    (
        r#"print([Label("//a:b"), Label("//a:c")], (Label("//a:b"),), str([Label("//a:b")]))"#,
        Ok(r#"[Label("//a:b"), Label("//a:c")] (Label("//a:b"),) [Label("//a:b")]"#),
    ),
    (
        r#"print(Label("//a:b") in [Label("//a:b")], Label("//a:b") in {Label("//a:b"): 1}, Label("//a:x") in [Label("//a:b")])"#,
        Ok(r#"True True False"#),
    ),
    (
        r#"print([Label("//a:b")] == [Label("//a:b")])"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(Label("//a:b") < Label("//a:c"), Label("//a:c") < Label("//a:b"), Label("//a:b") < Label("//a:b"), Label("//a:b") <= Label("//a:b"), Label("//a:b") > Label("//a:b"), Label("//a:b") >= Label("//a:a"))"#,
        Ok(r#"True False False True False True"#),
    ),
    (
        r#"print(sorted([Label("@@z//a:b"), Label("//z:b"), Label("//a:z"), Label("//a:b"), Label("//a:a"), Label("//a/b:a"), Label("//a-b:a"), Label("//a:B"), Label("@@dep+//a:b")]))"#,
        Ok(
            r#"[Label("//a:B"), Label("//a:a"), Label("//a:b"), Label("//a:z"), Label("//a-b:a"), Label("//a/b:a"), Label("//z:b"), Label("@@dep+//a:b"), Label("@@z//a:b")]"#,
        ),
    ),
    (
        r#"print(str(max([Label("//a:b"), Label("//a:c")])), str(min([Label("//a:b"), Label("//a:c")])))"#,
        Ok(r#"@@//a:c @@//a:b"#),
    ),
    (
        r#"print(Label("//a:b") < "x")"#,
        Err(r#"unsupported comparison: Label <=> string"#),
    ),
    (
        r#"print(Label("//a:b") < 1)"#,
        Err(r#"unsupported comparison: Label <=> int"#),
    ),
    (
        r#"print(dir(Label("//a:b")))"#,
        Ok(
            r#"["name", "package", "relative", "repo_name", "same_package_label", "workspace_name", "workspace_root"]"#,
        ),
    ),
    (
        r#"print(dir(Label("@r//a:b")))"#,
        Ok(
            r#"["name", "package", "relative", "repo_name", "same_package_label", "workspace_name", "workspace_root"]"#,
        ),
    ),
    (
        r#"print(hasattr(Label("//a:b"), "name"), hasattr(Label("//a:b"), "foo"), getattr(Label("//a:b"), "foo", 1), getattr(Label("//a:b"), "name"))"#,
        Ok(r#"True False 1 b"#),
    ),
    (
        r#"print(hasattr(Label("@x//a:b"), "repo_name"))"#,
        Ok(r#"True"#),
    ),
    (
        r#"print(Label("//a:b").foo)"#,
        Err(r#"'Label' value has no field or method 'foo'"#),
    ),
    (
        r#"print(Label("//a:b") + "x")"#,
        Err(r#"unsupported binary operation: Label + string"#),
    ),
    (
        r#"print(Label("//a:b").name())"#,
        Err(r#"'string' object is not callable"#),
    ),
    (
        r#"print(Label("//a:b").package())"#,
        Err(r#"'string' object is not callable"#),
    ),
    (
        r#"l = Label("//a:b")
l.name = "x""#,
        Err(r#"cannot set .name field of Label value"#),
    ),
    (r#"print(bool(Label("//a:b")))"#, Ok(r#"True"#)),
    (
        r#"print(len(Label("//a:b")))"#,
        Err(
            r#"in call to len(), parameter 'x' got value of type 'Label', want 'iterable or string'"#,
        ),
    ),
    (
        r#"print(json.encode(Label("//a:b")))"#,
        Err(r#"cannot encode Label as JSON"#),
    ),
    (
        r#"print(proto.encode_text(struct(a = Label("//a:b"))))"#,
        Err(r#"in struct field .a: got Label, want string, int, float, bool, or struct"#),
    ),
    (
        r#"print(struct(a = Label("//a:b")))"#,
        Ok(r#"struct(a = Label("//a:b"))"#),
    ),
    (
        r#"print("%s %r" % (Label("//a:b"), Label("//a:b")))"#,
        Ok(r#"@@//a:b Label("//a:b")"#),
    ),
    (
        r#"print("{}".format(Label("//a:b")), "{}".format([Label("//a:b")]))"#,
        Ok(r#"@@//a:b [Label("//a:b")]"#),
    ),
    (
        r#"print(Label("@r//a:b").repo_name)"#,
        Err(
            r#"'repo_name' is not allowed on invalid Label @@[unknown repo 'r' requested from @@]//a:b"#,
        ),
    ),
    (
        r#"print(Label("@r//a:b").workspace_name)"#,
        Err(
            r#"'workspace_name' is not allowed on invalid Label @@[unknown repo 'r' requested from @@]//a:b"#,
        ),
    ),
    (
        r#"print(Label("@r//a:b").workspace_root)"#,
        Err(
            r#"'workspace_root' is not allowed on invalid Label @@[unknown repo 'r' requested from @@]//a:b"#,
        ),
    ),
    (
        r#"print(Label("@r//a:b").name, Label("@r//a:b").package)"#,
        Ok(r#"b a"#),
    ),
    (
        r#"print(getattr(Label("@x//a:b"), "repo_name", 1))"#,
        Err(
            r#"'repo_name' is not allowed on invalid Label @@[unknown repo 'x' requested from @@]//a:b"#,
        ),
    ),
    (
        r#"print(Label(1))"#,
        Err(
            r#"in call to Label(), parameter 'input' got value of type 'int', want 'string or Label'"#,
        ),
    ),
    (
        r#"print(Label(None))"#,
        Err(
            r#"in call to Label(), parameter 'input' got value of type 'NoneType', want 'string or Label'"#,
        ),
    ),
    (
        r#"print(Label([]))"#,
        Err(
            r#"in call to Label(), parameter 'input' got value of type 'list', want 'string or Label'"#,
        ),
    ),
    (
        r#"print(Label())"#,
        Err(r#"Label() missing 1 required positional argument: input"#),
    ),
    (
        r#"print(Label("//a:b", "//c:d"))"#,
        Err(r#"Label() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(Label(x = "//a:b"))"#,
        Err(r#"Label() got unexpected keyword argument 'x'"#),
    ),
    (
        r#"print(Label(input = "//a:b"))"#,
        Err(r#"Label() got named argument for positional-only parameter 'input'"#),
    ),
    (
        r#"print(Label(**{"input": "//a:b"}))"#,
        Err(r#"Label() got named argument for positional-only parameter 'input'"#),
    ),
    (r#"print(str(Label(*["//a:b"])))"#, Ok(r#"@@//a:b"#)),
    (r#"print(str(Label("//a:b", )))"#, Ok(r#"@@//a:b"#)),
    (
        r#"print(Label("//a:b").relative(1))"#,
        Err(r#"in call to relative(), parameter 'relName' got value of type 'int', want 'string'"#),
    ),
    (
        r#"print(Label("//a:b").relative(Label("//c:d")))"#,
        Err(
            r#"in call to relative(), parameter 'relName' got value of type 'Label', want 'string'"#,
        ),
    ),
    (
        r#"print(Label("//a:b").relative())"#,
        Err(r#"relative() missing 1 required positional argument: relName"#),
    ),
    (
        r#"print(Label("//a:b").relative("a", "b"))"#,
        Err(r#"relative() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(Label("//a:b").relative(x = "a"))"#,
        Err(r#"relative() got unexpected keyword argument 'x'"#),
    ),
    (
        r#"print(Label("//a:b").relative(relName = ":c"))"#,
        Err(r#"relative() got named argument for positional-only parameter 'relName'"#),
    ),
    (
        r#"print(Label("//a:b").same_package_label(1))"#,
        Err(
            r#"in call to same_package_label(), parameter 'target_name' got value of type 'int', want 'string'"#,
        ),
    ),
    (
        r#"print(Label("//a:b").same_package_label(Label("//c:d")))"#,
        Err(
            r#"in call to same_package_label(), parameter 'target_name' got value of type 'Label', want 'string'"#,
        ),
    ),
    (
        r#"print(Label("//a:b").same_package_label())"#,
        Err(r#"same_package_label() missing 1 required positional argument: target_name"#),
    ),
    (
        r#"print(Label("//a:b").same_package_label("a", "b"))"#,
        Err(r#"same_package_label() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"print(Label("//a:b").same_package_label(target_name = "c"))"#,
        Err(
            r#"same_package_label() got named argument for positional-only parameter 'target_name'"#,
        ),
    ),
    (
        r#"print(Label("a:b"))"#,
        Err(
            r#"invalid label in Label(): invalid label 'a:b': absolute label must begin with '@' or '//'"#,
        ),
    ),
    (
        r#"print(Label(""))"#,
        Err(r#"invalid label in Label(): invalid target name '': empty target name"#),
    ),
    (r#"print(str(Label("//a b:c")))"#, Ok(r#"@@//a b:c"#)),
    (
        r#"print(Label("//a:"))"#,
        Err(r#"invalid label in Label(): invalid target name '': empty target name"#),
    ),
    (
        r#"print(Label("//a::b"))"#,
        Err(
            r#"invalid label in Label(): invalid target name ':b': target names may not contain ':'"#,
        ),
    ),
    (
        r#"print(Label("//..."))"#,
        Err(
            r#"invalid label in Label(): invalid label '//...': package name cannot contain '...'"#,
        ),
    ),
    (
        r#"print(Label("//a/..."))"#,
        Err(
            r#"invalid label in Label(): invalid label '//a/...': package name cannot contain '...'"#,
        ),
    ),
    (
        r#"print(Label("@@"))"#,
        Err(r#"invalid label in Label(): invalid target name '': empty target name"#),
    ),
    (
        r#"print(Label("@"))"#,
        Err(r#"invalid label in Label(): invalid target name '': empty target name"#),
    ),
    (
        r#"print(Label("@r//"))"#,
        Err(r#"invalid label in Label(): invalid target name '': empty target name"#),
    ),
    (
        r#"print(Label("//:"))"#,
        Err(r#"invalid label in Label(): invalid target name '': empty target name"#),
    ),
    (
        r#"print(Label("//"))"#,
        Err(r#"invalid label in Label(): invalid target name '': empty target name"#),
    ),
    (r#"print(str(Label("//:a")))"#, Ok(r#"@@//:a"#)),
    (
        r#"print(Label(" //a:b"))"#,
        Err(
            r#"invalid label in Label(): invalid package name ' //a': package names may not contain '//' path separators"#,
        ),
    ),
    (
        r#"print(Label("//a:b/../c"))"#,
        Err(
            r#"invalid label in Label(): invalid target name 'b/../c': target names may not contain up-level references '..'"#,
        ),
    ),
    (
        r#"print(Label("//a//b:c"))"#,
        Err(
            r#"invalid label in Label(): invalid package name 'a//b': package names may not contain '//' path separators"#,
        ),
    ),
    (
        r#"print(Label("@r/x//a:b"))"#,
        Err(
            r#"invalid label in Label(): invalid repository name 'r/x': repo names may contain only A-Z, a-z, 0-9, '-', '_', '.' and '+'"#,
        ),
    ),
    (
        r#"print(Label("@@r~x//a:b"))"#,
        Err(
            r#"invalid label in Label(): invalid repository name 'r~x': repo names may contain only A-Z, a-z, 0-9, '-', '_', '.' and '+'"#,
        ),
    ),
    (
        r#"print(Label("@@r/x//a:b"))"#,
        Err(
            r#"invalid label in Label(): invalid repository name 'r/x': repo names may contain only A-Z, a-z, 0-9, '-', '_', '.' and '+'"#,
        ),
    ),
    (
        r#"print(Label("@r x//a:b"))"#,
        Err(
            r#"invalid label in Label(): invalid repository name 'r x': repo names may contain only A-Z, a-z, 0-9, '-', '_', '.' and '+'"#,
        ),
    ),
    (
        r#"print(Label("//a:b\\c"))"#,
        Err(
            r#"invalid label in Label(): invalid target name 'b\c': target names may not contain '\'"#,
        ),
    ),
    (
        r#"print(Label("//a:b/"))"#,
        Err(
            r#"invalid label in Label(): invalid target name 'b/': target names may not end with '/'"#,
        ),
    ),
    (
        r#"print(Label("//a:/b"))"#,
        Err(
            r#"invalid label in Label(): invalid target name '/b': target names may not start with '/'"#,
        ),
    ),
    (
        r#"print(Label("//a:b//c"))"#,
        Err(
            r#"invalid label in Label(): invalid target name 'b//c': target names may not contain '//' path separators"#,
        ),
    ),
    (
        r#"print(Label("//a:./b"))"#,
        Err(
            r#"invalid label in Label(): invalid target name './b': target names may not contain '.' as a path segment"#,
        ),
    ),
    (r#"print(str(Label("//a:b/.")))"#, Ok(r#"@@//a:b"#)),
    (
        r#"print(Label("//a/./b:c"))"#,
        Err(
            r#"invalid label in Label(): invalid package name 'a/./b': package name component contains only '.' characters"#,
        ),
    ),
    (
        r#"print(Label("//a/../b:c"))"#,
        Err(
            r#"invalid label in Label(): invalid package name 'a/../b': package name component contains only '.' characters"#,
        ),
    ),
    (
        r#"print(Label("//a:b:c"))"#,
        Err(
            r#"invalid label in Label(): invalid target name 'b:c': target names may not contain ':'"#,
        ),
    ),
    (r#"print(str(Label("//a b")))"#, Ok(r#"@@//a b:a b"#)),
    (r#"print(Label("//a b:c ").name)"#, Ok(r#"c "#)),
    (r#"print(Label("//a:é").name)"#, Ok(r#"é"#)),
    (r#"print(str(Label("//a:.")))"#, Ok(r#"@@//a:."#)),
    (
        r#"print(str(Label("//a:..")))"#,
        Err(
            r#"invalid label in Label(): invalid target name '..': target names may not contain up-level references '..'"#,
        ),
    ),
    (
        r#"print(str(Label("//a/b:c").relative(".")))"#,
        Ok(r#"@@//a/b:."#),
    ),
    (
        r#"print(Label("//a:b").same_package_label("//c:d"))"#,
        Err(r#"invalid target name '//c:d': target names may not start with '/'"#),
    ),
    (
        r#"print(Label("//a:b").same_package_label(":d"))"#,
        Err(r#"invalid target name ':d': target names may not contain ':'"#),
    ),
    (
        r#"print(Label("//a:b").same_package_label("@r//a:d"))"#,
        Err(r#"invalid target name '@r//a:d': target names may not contain '//' path separators"#),
    ),
    (
        r#"print(Label("//a:b").same_package_label(""))"#,
        Err(r#"invalid target name '': empty target name"#),
    ),
    (
        r#"print(Label("//a:b").relative(""))"#,
        Err(r#"invalid target name '': empty target name"#),
    ),
    (
        r#"print(Label("//a:b").relative("..."))"#,
        Err(r#"invalid label '...': package name cannot contain '...'"#),
    ),
    (
        r#"print(Label("//a:b").relative("//..."))"#,
        Err(r#"invalid label '//...': package name cannot contain '...'"#),
    ),
    (
        r#"print(str(Label("//a:b").relative("@r")), str(Label("//a:b").relative("@@r")), str(Label("//a:b").relative("@@r+//p")))"#,
        Ok(r#"@@[unknown repo 'r' requested from @@]//:r @@r//:r @@r+//p:p"#),
    ),
    (
        r#"print(Label("//a:b").relative("a:b:c"))"#,
        Err(r#"invalid target name 'b:c': target names may not contain ':'"#),
    ),
    (
        r#"print(Label("//a:b").relative("x:y"))"#,
        Err(r#"invalid label 'x:y': absolute label must begin with '@' or '//'"#),
    ),
    (
        r#"print(Label("//a/b:c").relative("../x"))"#,
        Err(r#"invalid target name '../x': target names may not contain up-level references '..'"#),
    ),
    (
        r#"print(Label("//a/b:c").relative("x/../y"))"#,
        Err(
            r#"invalid target name 'x/../y': target names may not contain up-level references '..'"#,
        ),
    ),
    (
        r#"print(Label("//a/b:c").relative(":x/../y"))"#,
        Err(
            r#"invalid target name 'x/../y': target names may not contain up-level references '..'"#,
        ),
    ),
    (
        r#"print(Label("//a/b:c").relative("//"))"#,
        Err(r#"invalid target name '': empty target name"#),
    ),
    (
        r#"print(str(Label("//a/b:c").relative("//:x")), str(Label("//a:b").relative("//c")), str(Label("//a:b").relative("//c/d")))"#,
        Ok(r#"@@//:x @@//c:c @@//c/d:d"#),
    ),
    (
        r#"print(str(Label("//a:b").relative("d/e")))"#,
        Ok(r#"@@//a:d/e"#),
    ),
    (
        r#"print(str(Label("//a:b").relative("c").relative("d")))"#,
        Ok(r#"@@//a:d"#),
    ),
    (
        r#"print(str(Label("@@r+//a:b").relative("@@//c:d")), str(Label("//a:b").relative("@@r//c:d")))"#,
        Ok(r#"@@//c:d @@r//c:d"#),
    ),
    (
        r#"print(str(Label("@@r//a:b").relative("//c:d")))"#,
        Ok(r#"@@r//c:d"#),
    ),
    (
        r#"print(Label("//a:b").relative("@r//x:y").repo_name)"#,
        Err(
            r#"'repo_name' is not allowed on invalid Label @@[unknown repo 'r' requested from @@]//x:y"#,
        ),
    ),
    (r#"print(Label("//a:b").name.upper())"#, Ok(r#"B"#)),
    (
        r#"x = Label("//a:b")
y = x.relative(":c")
print(str(x), str(y))"#,
        Ok(r#"@@//a:b @@//a:c"#),
    ),
    (
        r#"print(str(Label("@@dep+//a:b")), str(Label("@@dep+")), repr(Label("@@dep+")), str(Label("@@dep+//a:dep+")))"#,
        Ok(r#"@@dep+//a:b @@dep+//:dep+ Label("@@dep+//:dep+") @@dep+//a:dep+"#),
    ),
];

#[test]
fn mappings_of_a_module_graph_name_repos_as_its_modules_do() {
    // What `fjfj_bzlmod::Resolution::repo_mappings` gives for the workspace
    // `bazel mod dump_repo_mapping` was probed on (buildfiji-mum.15).
    let rows = |rows: &[(&str, &str)]| {
        rows.iter()
            .map(|(a, c)| (a.to_string(), c.to_string()))
            .collect::<Vec<_>>()
    };
    let mappings = crate::RepoMappings::from_repos([
        (
            String::new(),
            rows(&[
                ("", ""),
                ("me", ""),
                ("mydep", "dep+"),
                ("bazel_tools", "bazel_tools"),
            ]),
        ),
        (
            "dep+".to_owned(),
            rows(&[("dep_alias", "dep+"), ("bazel_tools", "bazel_tools")]),
        ),
    ]);
    assert_eq!(mappings.resolve_apparent("", "me"), "");
    assert_eq!(mappings.resolve_apparent("", "mydep"), "dep+");
    assert_eq!(mappings.resolve_apparent("dep+", "dep_alias"), "dep+");
    assert_eq!(
        mappings.resolve_apparent("dep+", "mydep"),
        "[unknown repo 'mydep' requested from @@dep+]"
    );
}
