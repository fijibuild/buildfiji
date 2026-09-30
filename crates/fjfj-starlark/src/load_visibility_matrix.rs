//! Generated from Bazel 9.2.0 probes: what `visibility()` accepts and refuses.

pub(crate) const VISIBILITY_CALLS: &[(&str, Result<&str, &str>)] = &[
    (
        r#"visibility()
X = 1"#,
        Err(r#"visibility() missing 1 required positional argument: value"#),
    ),
    (
        r#"visibility(1)
X = 1"#,
        Err(r#"Invalid visibility: got 'int', want string or list of strings"#),
    ),
    (
        r#"visibility(None)
X = 1"#,
        Err(r#"Invalid visibility: got 'NoneType', want string or list of strings"#),
    ),
    (
        r#"visibility("x")
X = 1"#,
        Err(r#"invalid package name 'x': must start with '//', '@', or be 'public' or 'private'"#),
    ),
    (
        r#"visibility("//a")
X = 1"#,
        Ok(""),
    ),
    (
        r#"visibility([1])
X = 1"#,
        Err(r#"at index 0 of visibility list, got element of type int, want string"#),
    ),
    (
        r#"visibility(["x"])
X = 1"#,
        Err(r#"invalid package name 'x': must start with '//', '@', or be 'public' or 'private'"#),
    ),
    (
        r#"visibility(["//a:b"])
X = 1"#,
        Err(
            r#"invalid package name '//a:b': invalid target name 'b:__pkg__': target names may not contain ':'"#,
        ),
    ),
    (
        r#"visibility(["//a:__pkg__"])
X = 1"#,
        Ok(""),
    ),
    (
        r#"visibility(["//a:__subpackages__"])
X = 1"#,
        Ok(""),
    ),
    (
        r#"visibility(["@foo//x"])
X = 1"#,
        Ok(""),
    ),
    (
        r#"visibility(["-//b"])
X = 1"#,
        Err(r#"Cannot use negative package patterns here"#),
    ),
    (
        r#"visibility(["public","private"])
X = 1"#,
        Ok(""),
    ),
    (
        r#"visibility(["//b", "public"])
X = 1"#,
        Ok(""),
    ),
    (
        r#"visibility(["//b", "private"])
X = 1"#,
        Ok(""),
    ),
    (
        r#"visibility(("//b",))
X = 1"#,
        Err(r#"Invalid visibility: got 'tuple', want string or list of strings"#),
    ),
    (
        r#"visibility(["//b"], 1)
X = 1"#,
        Err(r#"visibility() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"visibility(x=["//b"])
X = 1"#,
        Err(r#"visibility() got unexpected keyword argument 'x'"#),
    ),
    (
        r#"visibility(value="public")
X = 1"#,
        Err(r#"visibility() got named argument for positional-only parameter 'value'"#),
    ),
    (
        r#"visibility(["//b"])
visibility(["//b"])
X = 1"#,
        Err(r#"load visibility may not be set more than once"#),
    ),
    (
        r#"visibility("public", "x")
X = 1"#,
        Err(r#"visibility() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"visibility(["//b:group"])
X = 1"#,
        Err(
            r#"invalid package name '//b:group': invalid target name 'group:__pkg__': target names may not contain ':'"#,
        ),
    ),
    (
        r#"visibility([" //b"])
X = 1"#,
        Err(
            r#"invalid package name ' //b': must start with '//', '@', or be 'public' or 'private'"#,
        ),
    ),
    (
        r#"visibility(["//b/"])
X = 1"#,
        Err(
            r#"invalid package name '//b/': invalid package name 'b/': package names may not end with '/'"#,
        ),
    ),
    (
        r#"visibility(["//b/.."])
X = 1"#,
        Err(
            r#"invalid package name '//b/..': invalid package name 'b/..': package name component contains only '.' characters"#,
        ),
    ),
    (
        r#"visibility(["b"])
X = 1"#,
        Err(r#"invalid package name 'b': must start with '//', '@', or be 'public' or 'private'"#),
    ),
    (
        r#"visibility(["@@//b"])
X = 1"#,
        Ok(""),
    ),
    (
        r#"visibility(["@//b"])
X = 1"#,
        Ok(""),
    ),
    (
        r#"visibility(["//..."])
visibility("private")
X = 1"#,
        Err(r#"load visibility may not be set more than once"#),
    ),
    (
        r#"visibility("public")
X = 1"#,
        Ok(""),
    ),
    (
        r#"visibility("private")
X = 1"#,
        Ok(""),
    ),
    (
        r#"visibility([])
X = 1"#,
        Ok(""),
    ),
    (
        r#"visibility(["public"])
X = 1"#,
        Ok(""),
    ),
    (
        r#"visibility(["//b/..."])
X = 1"#,
        Ok(""),
    ),
    (
        r#"visibility(["//a/", "//b"])
X = 1"#,
        Err(
            r#"invalid package name '//a/': invalid package name 'a/': package names may not end with '/'"#,
        ),
    ),
    (
        r#"visibility("//b")
X = 1"#,
        Ok(""),
    ),
    (
        r#"visibility(["//b:__pkg__", "-//c"])
X = 1"#,
        Err(r#"Cannot use negative package patterns here"#),
    ),
    (
        r#"visibility([None])
X = 1"#,
        Err(r#"at index 0 of visibility list, got element of type NoneType, want string"#),
    ),
    (
        r#"visibility([["//b"]])
X = 1"#,
        Err(r#"at index 0 of visibility list, got element of type list, want string"#),
    ),
    (
        r#"visibility({"a": 1})
X = 1"#,
        Err(r#"Invalid visibility: got 'dict', want string or list of strings"#),
    ),
    (
        r#"visibility(True)
X = 1"#,
        Err(r#"Invalid visibility: got 'bool', want string or list of strings"#),
    ),
    (
        r#"visibility(["//"])
X = 1"#,
        Ok(""),
    ),
    (
        r#"visibility(["//:x"])
X = 1"#,
        Err(
            r#"invalid package name '//:x': invalid target name 'x:__pkg__': target names may not contain ':'"#,
        ),
    ),
    (
        r#"visibility(["@//"])
X = 1"#,
        Ok(""),
    ),
];
