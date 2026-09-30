//! Generated from Bazel 9.2.0 probes: what `repository_rule()`, `module_extension()` and
//! `tag_class()` print, compare as and refuse.

use crate::test_support::BuildRow;

pub(crate) const EXT_CASES: &[(&str, Result<&str, &str>)] = &[
    (
        r#"def f(ctx): pass
x=repository_rule(f)
r=1
print(x)
print(type(x))
print(repr(x))
print(str(x))"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%x>
repository_rule
<starlark repository rule @@//:t.bzl%x>
<starlark repository rule @@//:t.bzl%x>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
print(r)
print(type(r))"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>
repository_rule"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(implementation=f)
print(r)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"r=repository_rule()"#,
        Err(r#"repository_rule() missing 1 required positional argument: implementation"#),
    ),
    (
        r#"r=repository_rule(1)"#,
        Err(
            r#"in call to repository_rule(), parameter 'implementation' got value of type 'int', want 'callable'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, attrs=1)"#,
        Err(
            r#"in call to repository_rule(), parameter 'attrs' got value of type 'int', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, attrs={"a": attr.string()})
print(r)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, attrs={"a": 1})"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, attrs={"name": attr.string()})"#,
        Err(r#"There is already a built-in attribute 'name' which cannot be overridden"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, attrs={"a": attr.label()})"#,
        Ok(r#""#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, attrs={"a": attr.label(cfg="exec")})"#,
        Ok(r#""#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, attrs={"a": attr.label(default="//:x")})"#,
        Ok(r#""#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, attrs={"a": attr.label(default="//:x", allow_files=True)})"#,
        Ok(r#""#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, attrs={"a": attr.output()})"#,
        Ok(r#""#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, attrs={"a": attr.output_list()})"#,
        Ok(r#""#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, attrs={"a": attr.label_list(providers=[])})"#,
        Ok(r#""#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, attrs={"a": attr.int(), "b": attr.bool(), "c": attr.string_list(), "d": attr.string_dict(), "e": attr.string_list_dict(), "f": attr.int_list(), "g": attr.label_keyed_string_dict(allow_files=True), "h": attr.label_list(allow_files=True)})
print(r)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, attrs={"a": attr.string(mandatory=True)})
print(r)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, attrs={"a": attr.string(configurable=False)})"#,
        Ok(r#""#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, local=True)
print(r)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, local=1)"#,
        Err(
            r#"in call to repository_rule(), parameter 'local' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, environ=["A","B"])
print(r)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, environ="A")"#,
        Err(
            r#"in call to repository_rule(), parameter 'environ' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, environ=[1])"#,
        Err(r#"at index 0 of repository_rule, got element of type int, want string"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, configure=True, remotable=True)
print(r)"#,
        Err(
            r#"in call to repository_rule(), parameter 'remotable' is experimental and thus unavailable with the current flags. It may be enabled by setting --experimental_repo_remote_exec"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, configure=1)"#,
        Err(
            r#"in call to repository_rule(), parameter 'configure' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, remotable=1)"#,
        Err(
            r#"in call to repository_rule(), parameter 'remotable' is experimental and thus unavailable with the current flags. It may be enabled by setting --experimental_repo_remote_exec"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, doc="d")
print(r)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, doc=1)"#,
        Err(
            r#"in call to repository_rule(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, doc=None)
print(r)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, bogus=1)"#,
        Err(r#"repository_rule() got unexpected keyword argument 'bogus'"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, 1)"#,
        Err(r#"repository_rule() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, {})"#,
        Err(r#"repository_rule() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"def f(): pass
r=repository_rule(f)
print(r)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"def f(a,b): pass
r=repository_rule(f)
print(r)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
r(name="x")"#,
        Err(r#"repo rules can only be called from within module extension impl functions"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
r()"#,
        Err(r#"repo rules can only be called from within module extension impl functions"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
r(1)"#,
        Err(r#"unexpected positional arguments"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
print(r(name="x"))"#,
        Err(r#"repo rules can only be called from within module extension impl functions"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
print(r == r)
print(r == repository_rule(f))"#,
        Ok(r#"True
False"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
print(bool(r))
print(hash(r) == hash(r))"#,
        Err(
            r#"in call to hash(), parameter 'value' got value of type 'repository_rule', want 'string'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
print(dir(r))"#,
        Ok(r#"[]"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
print(r.foo)"#,
        Err(r#"'repository_rule' value has no field or method 'foo'"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
x=[r]
print(x)"#,
        Ok(r#"[<starlark repository rule @@//:t.bzl%r>]"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
x={"k":r}
print(x)
print(x["k"])"#,
        Ok(r#"{"k": <starlark repository rule @@//:t.bzl%r>}
<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
s=struct(a=r)
print(s)"#,
        Ok(r#"struct(a = <starlark repository rule @@//:t.bzl%r>)"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
print(r)
r2=r
print(r2)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>
<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"def f(ctx): pass
x=repository_rule(f)
y=x
r=1
print(x)
print(y)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%x>
<starlark repository rule @@//:t.bzl%x>"#),
    ),
    (
        r#"def f(ctx): pass
_x=repository_rule(f)
r=1
print(_x)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%_x>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
print(repository_rule(f))"#,
        Ok(r#"<anonymous starlark repository rule>"#),
    ),
    (
        r#"def f(ctx): pass
r=[repository_rule(f)]
print(r)"#,
        Ok(r#"[<anonymous starlark repository rule>]"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f,attrs={})
print(r)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f,attrs=None)
print(r)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f,attrs=None)"#,
        Ok(r#""#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f,environ=None)
print(r)"#,
        Err(
            r#"in call to repository_rule(), parameter 'environ' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f,local=None)
print(r)"#,
        Err(
            r#"in call to repository_rule(), parameter 'local' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f,configure=None)
print(r)"#,
        Err(
            r#"in call to repository_rule(), parameter 'configure' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f,remotable=None)
print(r)"#,
        Err(
            r#"in call to repository_rule(), parameter 'remotable' is experimental and thus unavailable with the current flags. It may be enabled by setting --experimental_repo_remote_exec"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f,environ=("A",))
print(r)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f,environ=[])
print(r)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"r=repository_rule(lambda ctx: None)
print(r)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f,attrs={"a": attr.string(), "a": attr.int()})"#,
        Err(r#"dictionary expression has duplicate key: "a""#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f,attrs={1: attr.string()})"#,
        Err(r#"got dict<int, Attribute> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f,attrs={"1a": attr.string()})"#,
        Ok(r#""#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f,attrs={"a b": attr.string()})"#,
        Ok(r#""#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f,attrs={"_a": attr.string()})
print(r)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f,attrs={"_a": attr.label(default="//:x")})
print(r)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%r>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f,attrs={"a": attr.string(default="x", values=["y"])})"#,
        Ok(r#""#),
    ),
    (
        r#"def f(ctx): pass
x=module_extension(f)
r=1
print(x)
print(type(x))
print(repr(x))
print(str(x))"#,
        Ok(
            r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>
ModuleExtension
<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>
<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(implementation=f)
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>"#),
    ),
    (
        r#"r=module_extension()"#,
        Err(r#"module_extension() missing 1 required positional argument: implementation"#),
    ),
    (
        r#"r=module_extension(1)"#,
        Err(
            r#"in call to module_extension(), parameter 'implementation' got value of type 'int', want 'callable'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, tag_classes=1)"#,
        Err(
            r#"in call to module_extension(), parameter 'tag_classes' got value of type 'int', want 'dict'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, tag_classes={"t": 1})"#,
        Err(r#"got dict<string, int> for 'tag_classes', want dict<string, tag_class>"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, tag_classes={"t": tag_class()})
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, tag_classes={1: tag_class()})"#,
        Err(r#"got dict<int, tag_class> for 'tag_classes', want dict<string, tag_class>"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, tag_classes={"": tag_class()})"#,
        Ok(r#""#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, tag_classes={"a b": tag_class()})
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, tag_classes={"_t": tag_class()})
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, tag_classes=None)
print(r)"#,
        Err(
            r#"in call to module_extension(), parameter 'tag_classes' got value of type 'NoneType', want 'dict'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, tag_classes={})
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, environ=["A"])
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, environ="A")"#,
        Err(
            r#"in call to module_extension(), parameter 'environ' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, environ=[1])"#,
        Err(r#"at index 0 of environ, got element of type int, want string"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, environ=None)"#,
        Err(
            r#"in call to module_extension(), parameter 'environ' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, os_dependent=True, arch_dependent=True)
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, os_dependent=1)"#,
        Err(
            r#"in call to module_extension(), parameter 'os_dependent' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, arch_dependent=1)"#,
        Err(
            r#"in call to module_extension(), parameter 'arch_dependent' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, os_dependent=None)"#,
        Err(
            r#"in call to module_extension(), parameter 'os_dependent' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, doc="d")
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, doc=1)"#,
        Err(
            r#"in call to module_extension(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, bogus=1)"#,
        Err(r#"module_extension() got unexpected keyword argument 'bogus'"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, {})"#,
        Err(r#"module_extension() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"def f(): pass
r=module_extension(f)
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
r()"#,
        Err(r#"'ModuleExtension' object is not callable"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
print(r == r)
print(r == module_extension(f))"#,
        Ok(r#"True
False"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
print(dir(r))"#,
        Ok(r#"[]"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
print(r.foo)"#,
        Err(r#"'ModuleExtension' value has no field or method 'foo'"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
print(hash(r))"#,
        Err(
            r#"in call to hash(), parameter 'value' got value of type 'ModuleExtension', want 'string'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
print(module_extension(f))
r=1"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>"#),
    ),
    (
        r#"def f(ctx): pass
r=[module_extension(f)]
print(r)"#,
        Ok(r#"[<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>]"#),
    ),
    (
        r#"def f(ctx): pass
x=module_extension(f)
y=x
r=1
print(y)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, tag_classes={"t": tag_class(attrs={"a": attr.string()})})
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(lambda ctx: 0)
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, arch_dependent=False, os_dependent=False)
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.ModuleExtension>"#),
    ),
    (
        r#"x=tag_class()
r=1
print(x)
print(type(x))
print(repr(x))
print(str(x))"#,
        Ok(
            r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>
tag_class
<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>
<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>"#,
        ),
    ),
    (
        r#"r=tag_class()
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>"#),
    ),
    (
        r#"r=tag_class(attrs={})
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>"#),
    ),
    (
        r#"r=tag_class(attrs=1)"#,
        Err(r#"in call to tag_class(), parameter 'attrs' got value of type 'int', want 'dict'"#),
    ),
    (
        r#"r=tag_class(attrs={"a": attr.string()})
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>"#),
    ),
    (
        r#"r=tag_class(attrs={"a": 1})"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=tag_class(attrs={"name": attr.string()})
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>"#),
    ),
    (
        r#"r=tag_class(attrs={"a": attr.label()})
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>"#),
    ),
    (
        r#"r=tag_class(attrs={"a": attr.label(default="//:x")})
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>"#),
    ),
    (r#"r=tag_class(attrs={"a": attr.output()})"#, Ok(r#""#)),
    (
        r#"r=tag_class(attrs={"a": attr.label(cfg="exec")})"#,
        Ok(r#""#),
    ),
    (
        r#"r=tag_class(attrs={"a": attr.string(mandatory=True)})
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>"#),
    ),
    (
        r#"r=tag_class(attrs={"a": attr.string(configurable=False)})
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>"#),
    ),
    (
        r#"r=tag_class(attrs={"a": attr.label_list(allow_files=True)})
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>"#),
    ),
    (
        r#"r=tag_class(doc="d")
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>"#),
    ),
    (
        r#"r=tag_class(doc=1)"#,
        Err(
            r#"in call to tag_class(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=tag_class(attrs=None)
print(r)"#,
        Err(
            r#"in call to tag_class(), parameter 'attrs' got value of type 'NoneType', want 'dict'"#,
        ),
    ),
    (
        r#"r=tag_class(bogus=1)"#,
        Err(r#"tag_class() got unexpected keyword argument 'bogus'"#),
    ),
    (r#"r=tag_class({})"#, Ok(r#""#)),
    (
        r#"r=tag_class({}, "d")
print(r)"#,
        Err(r#"tag_class() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"r=tag_class({}, "d", 3)"#,
        Err(r#"tag_class() accepts no more than 1 positional argument but got 3"#),
    ),
    (
        r#"r=tag_class()
r()
print(1)"#,
        Err(r#"'tag_class' object is not callable"#),
    ),
    (
        r#"r=tag_class()
print(r == r)
print(r == tag_class())"#,
        Ok(r#"True
True"#),
    ),
    (
        r#"r=tag_class()
print(dir(r))"#,
        Ok(r#"[]"#),
    ),
    (
        r#"r=tag_class()
print(r.foo)"#,
        Err(r#"'tag_class' value has no field or method 'foo'"#),
    ),
    (
        r#"r=tag_class()
print(hash(r))"#,
        Err(r#"in call to hash(), parameter 'value' got value of type 'tag_class', want 'string'"#),
    ),
    (
        r#"r=tag_class()
x=[r,r]
print(x)"#,
        Ok(
            r#"[<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>, <unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>]"#,
        ),
    ),
    (
        r#"r=tag_class(attrs={"_a": attr.string()})
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>"#),
    ),
    (
        r#"r=tag_class(attrs={"1": attr.string()})
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>"#),
    ),
    (
        r#"r=tag_class(attrs={"a": attr.label_keyed_string_dict(allow_files=True)})
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>"#),
    ),
    (
        r#"r=tag_class(attrs={"a": attr.string(values=["x"], default="y")})
print(r)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.TagClass>"#),
    ),
    (
        r#"r=tag_class(attrs={"a": attr.string()})
print(r == tag_class(attrs={"a": attr.string()}))
print(r == tag_class(attrs={"a": attr.int()}))
print(r == tag_class(attrs={"b": attr.string()}))
print(r == tag_class(attrs={"a": attr.string()}, doc="d"))"#,
        Ok(r#"True
False
False
False"#),
    ),
    (
        r#"r=tag_class(doc="a")
print(r == tag_class(doc="b"))
print(r == tag_class(doc="a"))"#,
        Ok(r#"False
True"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
print(r == module_extension(f, doc="x"))"#,
        Ok(r#"False"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
print(r == r, r != r)"#,
        Ok(r#"True False"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
print(r == repository_rule(f, doc="x"))"#,
        Ok(r#"False"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
print(str(r)[:60])
print(repr(r)[:60])
print(str(r) == str(r))"#,
        Ok(
            r#"<unknown object com.google.devtools.build.lib.bazel.bzlmod.M
<unknown object com.google.devtools.build.lib.bazel.bzlmod.M
True"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
print(bool(r))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=tag_class()
print(bool(r))
print(str(type(r)))"#,
        Ok(r#"True
tag_class"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
print([r] == [r])
print({"a": r} == {"a": r})"#,
        Ok(r#"True
True"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
print(r < r)"#,
        Err(r#"unsupported comparison: ModuleExtension <=> ModuleExtension"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
print(len(r))"#,
        Err(
            r#"in call to len(), parameter 'x' got value of type 'ModuleExtension', want 'iterable or string'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
print(r + 1)"#,
        Err(r#"unsupported binary operation: ModuleExtension + int"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
print(r[0])"#,
        Err(r#"type 'ModuleExtension' has no operator [](int)"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
print(1 in r)"#,
        Err(r#"unsupported binary operation: int in ModuleExtension"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
print([x for x in r])"#,
        Err(r#"type 'ModuleExtension' is not iterable"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
print(json.encode(r))"#,
        Err(r#"cannot encode ModuleExtension as JSON"#),
    ),
    (
        r#"def f(ctx): pass
r=tag_class()
print(json.encode(r))"#,
        Err(r#"cannot encode tag_class as JSON"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
print(json.encode(r))"#,
        Err(r#"cannot encode repository_rule as JSON"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f)
print(type(r.foo))"#,
        Err(r#"'repository_rule' value has no field or method 'foo'"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f)
print(hasattr(r, "x"))
print(getattr(r, "x", 1))"#,
        Ok(r#"False
1"#),
    ),
    (
        r#"def f(ctx): pass
x=repository_rule(f, attrs={"a": attr.string()})
print(repr(x))
r=1"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%x>"#),
    ),
    (
        r#"def f(ctx): pass
x=repository_rule(f, attrs={"a": attr.string(), "b": attr.label(), "c": attr.int()}, doc="d", local=True, environ=["E"], configure=True)
print(x)
r=1"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%x>"#),
    ),
    (
        r#"def f(ctx): pass
attrs={"a": attr.string()}
x=repository_rule(f, attrs=attrs)
attrs["b"]=attr.int()
r=1
print(x)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%x>"#),
    ),
    (
        r#"def f(ctx): pass
e=["A"]
x=repository_rule(f, environ=e)
e.append("B")
r=1
print(x)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%x>"#),
    ),
    (
        r#"def f(ctx): pass
x=repository_rule(f, attrs={"a": attr.label(providers=[["x"]])})
r=1
print(x)"#,
        Err(r#"at index 0 of providers, got element of type string, want Provider"#),
    ),
    (
        r#"def f(ctx): pass
x=repository_rule(f, attrs={"a": attr.label(aspects=[])})
r=1
print(x)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%x>"#),
    ),
    (
        r#"def f(ctx): pass
x=repository_rule(f, attrs={"a": attr.label(executable=True, cfg="exec")})
r=1
print(x)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%x>"#),
    ),
    (
        r#"def f(ctx): pass
x=repository_rule(f, attrs={"a": attr.string(doc="d")})
r=1
print(x)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%x>"#),
    ),
    (
        r#"def f(ctx): pass
x=repository_rule(f, attrs={"a": attr.int(default=1, values=[2])})
r=1
print(x)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%x>"#),
    ),
    (
        r#"def f(ctx): pass
x=repository_rule(f, attrs={"a": attr.int(default="x")})
r=1
print(x)"#,
        Err(r#"in call to int(), parameter 'default' got value of type 'string', want 'int'"#),
    ),
    (
        r#"def f(ctx): pass
x=repository_rule(f, attrs={"a": attr.string_list(default=[1])})
r=1
print(x)"#,
        Err(
            r#"expected value of type 'string' for element 0 of parameter 'default' of attribute '', but got 1 (int)"#,
        ),
    ),
    (
        r#"def f(ctx): pass
x=repository_rule(f, attrs={"a": attr.label(default=Label("//:x"))})
r=1
print(x)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%x>"#),
    ),
    (
        r#"def f(ctx): pass
x=repository_rule(f, attrs={"a": attr.label(default=1)})
r=1
print(x)"#,
        Err(
            r#"in call to label(), parameter 'default' got value of type 'int', want 'Label, string, LateBoundDefault, function, or NoneType'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
x=repository_rule(f, attrs={"a": attr.label_list(default=["//:x"])})
r=1
print(x)"#,
        Ok(r#"<starlark repository rule @@//:t.bzl%x>"#),
    ),
    (
        r#"def f(ctx): pass
x=repository_rule(f, attrs={"a": attr.label_list(default=[1])})
r=1
print(x)"#,
        Err(
            r#"expected value of type 'string' for element 0 of parameter 'default' of attribute 'label_list', but got 1 (int)"#,
        ),
    ),
    (
        r#"def f(ctx): pass
x=repository_rule(f, attrs={"a": attr.string_dict(default={"a": 1})})
r=1
print(x)"#,
        Err(r#"expected value of type 'string' for dict value element, but got 1 (int)"#),
    ),
    (
        r#"def f(ctx): pass
x=repository_rule(f, attrs={"a": attr.bool(default=1)})
r=1
print(x)"#,
        Err(r#"in call to bool(), parameter 'default' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"def f(ctx): pass
x=repository_rule(f, attrs={"a": attr.output(default="x")})
r=1
print(x)"#,
        Err(r#"output() got unexpected keyword argument 'default'"#),
    ),
    (
        r#"def f(ctx): pass
r=1
s=struct(n=repository_rule(f), m=module_extension(f))
print(s.n)"#,
        Ok(r#"<anonymous starlark repository rule>"#),
    ),
    (
        r#"def f(ctx): pass
def g(): return repository_rule(f)
r=1
print(g())"#,
        Ok(r#"<anonymous starlark repository rule>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, local=True)
print(r.local)"#,
        Err(r#"'repository_rule' value has no field or method 'local'"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, implementation=f)"#,
        Err(r#"repository_rule() got multiple values for argument 'implementation'"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(doc=1, implementation=f)"#,
        Err(
            r#"in call to repository_rule(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(remotable=1, doc=1, implementation=f)"#,
        Err(
            r#"in call to repository_rule(), parameter 'remotable' is experimental and thus unavailable with the current flags. It may be enabled by setting --experimental_repo_remote_exec"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(doc=1, remotable=1, implementation=f)"#,
        Err(
            r#"in call to repository_rule(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, attrs={"name": attr.string()}, environ=[1])"#,
        Err(r#"at index 0 of repository_rule, got element of type int, want string"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, environ=[1], attrs={"name": attr.string()})"#,
        Err(r#"at index 0 of repository_rule, got element of type int, want string"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, attrs={"name": 1})"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"def f(ctx): pass
r=repository_rule(f, attrs={"a": attr.string()}, configure=True, local=True)"#,
        Ok(r#""#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, tag_classes={"t": 1}, environ=[1])"#,
        Err(r#"got dict<string, int> for 'tag_classes', want dict<string, tag_class>"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, environ=[1], tag_classes={"t": 1})"#,
        Err(r#"got dict<string, int> for 'tag_classes', want dict<string, tag_class>"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, tag_classes={"t": tag_class(attrs=1)})"#,
        Err(r#"in call to tag_class(), parameter 'attrs' got value of type 'int', want 'dict'"#),
    ),
    (
        r#"def f(ctx): pass
r=module_extension(f, tag_classes={"t": tag_class()}, os_dependent=True)
print(type(r))"#,
        Ok(r#"ModuleExtension"#),
    ),
];

pub(crate) const EXT_BUILD_CASES: &[BuildRow] = &[
    BuildRow {
        bzl: r#"def f(ctx): pass
r=1
x=repository_rule(f)
y=x"#,
        build: r#"print(1)"#,
        printed: &[r#"1"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): pass
r=repository_rule(f)"#,
        build: r#"r(name="x")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"repo rules can only be called from within module extension impl functions"#),
    },
    BuildRow {
        bzl: r#"def f(ctx): pass
r=repository_rule(f, attrs={"a": attr.string()})"#,
        build: r#"r(name="x", a="y")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"repo rules can only be called from within module extension impl functions"#),
    },
    BuildRow {
        bzl: r#"def f(ctx): pass
r=repository_rule(f)"#,
        build: r#"r(1, 2)"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"unexpected positional arguments"#),
    },
    BuildRow {
        bzl: r#"def f(ctx): pass
r=repository_rule(f)"#,
        build: r#"r(name=1)"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"repo rules can only be called from within module extension impl functions"#),
    },
    BuildRow {
        bzl: r#"def f(ctx): pass
r=repository_rule(f)"#,
        build: r#"r(bogus=1)"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"repo rules can only be called from within module extension impl functions"#),
    },
    BuildRow {
        bzl: r#"def f(ctx): pass
r=repository_rule(f, attrs={"a": attr.string(mandatory=True)})"#,
        build: r#"r(name="x")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"repo rules can only be called from within module extension impl functions"#),
    },
    BuildRow {
        bzl: r#"def f(ctx): pass
r=repository_rule(f)"#,
        build: r#"r(*[1])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:3: *args arguments are not allowed in BUILD files. Pass the arguments in explicitly."#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): pass
r=repository_rule(f)"#,
        build: r#"r(**{"name": "x"})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:3: **kwargs arguments are not allowed in BUILD files. Pass the arguments in explicitly."#,
        ],
        fatal: None,
    },
];
