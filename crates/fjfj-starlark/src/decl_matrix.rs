//! Generated from Bazel 9.2.0 probes: what the declaration builtins print and refuse.

use crate::test_support::BuildRow;

pub(crate) const DECL_CASES: &[(&str, Result<&str, &str>)] = &[
    (
        r#"r=1
def f(target, ctx): return []
a=aspect(implementation=f)
print(a)
print(type(a))
print(repr(a))
print(str(a))"#,
        Ok(r#"<aspect>
Aspect
<aspect>
<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
a=aspect(f)
print(a)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
aspect()"#,
        Err(r#"aspect() missing 1 required positional argument: implementation"#),
    ),
    (
        r#"r=1
aspect(1)"#,
        Err(
            r#"in call to aspect(), parameter 'implementation' got value of type 'int', want 'function'"#,
        ),
    ),
    (
        r#"r=1
aspect(implementation=1)"#,
        Err(
            r#"in call to aspect(), parameter 'implementation' got value of type 'int', want 'function'"#,
        ),
    ),
    (
        r#"r=1
aspect(implementation=None)"#,
        Err(
            r#"in call to aspect(), parameter 'implementation' got value of type 'NoneType', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, 1))"#,
        Err(
            r#"in call to aspect(), parameter 'attr_aspects' got value of type 'int', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, [], {}, [], [], [], [], [], [], [], None, False, ""))"#,
        Err(
            r#"in call to aspect(), parameter 'toolchains_aspects' got value of type 'dict', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, [], {}, [], [], [], [], [], [], [], None, False, "", 1))"#,
        Err(
            r#"in call to aspect(), parameter 'toolchains_aspects' got value of type 'dict', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, foo=1))"#,
        Err(r#"aspect() got unexpected keyword argument 'foo'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
a=aspect(f)
b=aspect(f)
print(a==b)
print(a==a)"#,
        Ok(r#"False
True"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
a=aspect(f)
print({a:1})"#,
        Ok(r#"{<aspect>: 1}"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
a=aspect(f)
print(hash(a))"#,
        Err(r#"in call to hash(), parameter 'value' got value of type 'Aspect', want 'string'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
a=aspect(f)
print(dir(a))"#,
        Ok(r#"[]"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
a=aspect(f)
print(bool(a))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
a=aspect(f)
print(a.foo)"#,
        Err(r#"'Aspect' value has no field or method 'foo'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
A=aspect(f)
print(A)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
_A=aspect(f)
print(_A)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
A=aspect(f)
B=A
print(B)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
A=[aspect(f)]
print(A)"#,
        Ok(r#"[<aspect>]"#),
    ),
    (
        r#"r=1
def f(): return []
print(aspect(f))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(a,b,c): return []
print(aspect(f))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target): return []
print(aspect(f))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(*args): return []
print(aspect(f))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attr_aspects=["x"]))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attr_aspects="x"))"#,
        Err(
            r#"in call to aspect(), parameter 'attr_aspects' got value of type 'string', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attr_aspects=[1]))"#,
        Err(r#"at index 0 of attr_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attr_aspects=("x",)))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attr_aspects=["*"]))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attr_aspects=["_x"]))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attr_aspects=["x y"]))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attr_aspects=[""]))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attr_aspects=["x","x"]))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"_x":attr.label()}))"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"x":attr.label()}))"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"x":attr.string()}))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"_x":attr.string()}))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"_x":attr.int()}))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"_x":attr.label_list()}))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"_x":attr.string(values=["a"])}))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"_x":attr.string(default="a")}))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"_x":attr.string(default="a", values=["a"])}))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"_x":attr.int(default=1, values=[1])}))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"_x":attr.int(default=1)}))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"_x":attr.bool(default=True)}))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"_x":attr.label(default="//:x")}))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"_x":attr.label(default="//:x", allow_files=True)}))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"_x":attr.label(default="//:x", providers=[[]])}))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"name":attr.string()}))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"x y":attr.string()}))"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={1:attr.string()}))"#,
        Err(r#"got dict<int, Attribute> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"x":1}))"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs=[]))"#,
        Err(r#"in call to aspect(), parameter 'attrs' got value of type 'list', want 'dict'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs=None))"#,
        Err(r#"in call to aspect(), parameter 'attrs' got value of type 'NoneType', want 'dict'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"_x":attr.output()}))"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"_x":attr.label_list(mandatory=True)}))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
print(aspect(f, attrs={"_x":attr.string(mandatory=True)}))"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, implementation=1)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'implementation' got value of type 'int', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, implementation="s")
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'implementation' got value of type 'string', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, implementation=[])
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'implementation' got value of type 'list', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, implementation={})
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'implementation' got value of type 'dict', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, implementation=None)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'implementation' got value of type 'NoneType', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, implementation=True)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'implementation' got value of type 'bool', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, implementation=())
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'implementation' got value of type 'tuple', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, implementation=f)
print(x)"#,
        Err(r#"aspect() got multiple values for argument 'implementation'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, implementation=lambda t, c: [])
print(x)"#,
        Err(r#"aspect() got multiple values for argument 'implementation'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, implementation=S)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'implementation' got value of type 'struct', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, implementation=len)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'implementation' got value of type 'builtin_function_or_method', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, implementation=aspect(f))
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'implementation' got value of type 'Aspect', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attr_aspects=1)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'attr_aspects' got value of type 'int', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attr_aspects="s")
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'attr_aspects' got value of type 'string', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attr_aspects=[])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attr_aspects={})
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'attr_aspects' got value of type 'dict', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attr_aspects=None)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'attr_aspects' got value of type 'NoneType', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attr_aspects=True)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'attr_aspects' got value of type 'bool', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attr_aspects=())
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attr_aspects=f)
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attr_aspects=lambda t, c: [])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attr_aspects=["a"])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attr_aspects=[1])
print(x)"#,
        Err(r#"at index 0 of attr_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attr_aspects=[[]])
print(x)"#,
        Err(r#"at index 0 of attr_aspects, got element of type list, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attr_aspects=range(2))
print(x)"#,
        Err(r#"at index 0 of attr_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attr_aspects=depset([]))
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'attr_aspects' got value of type 'depset', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attr_aspects=set())
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'attr_aspects' got value of type 'set', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attr_aspects=S)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'attr_aspects' got value of type 'struct', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attr_aspects={"a":"b"})
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'attr_aspects' got value of type 'dict', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains_aspects=1)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'toolchains_aspects' got value of type 'int', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains_aspects="s")
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'toolchains_aspects' got value of type 'string', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains_aspects=[])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains_aspects={})
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'toolchains_aspects' got value of type 'dict', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains_aspects=None)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'toolchains_aspects' got value of type 'NoneType', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains_aspects=())
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains_aspects=f)
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains_aspects=lambda t, c: [])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains_aspects=["a"])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains_aspects=[1])
print(x)"#,
        Err(r#"at index 0 of toolchains_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains_aspects=["//x:y"])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains_aspects=["x y"])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains_aspects=[L])
print(x)"#,
        Err(r#"at index 0 of toolchains_aspects, got element of type Label, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains_aspects=S)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'toolchains_aspects' got value of type 'struct', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains_aspects=range(2))
print(x)"#,
        Err(r#"at index 0 of toolchains_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains_aspects=depset([]))
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'toolchains_aspects' got value of type 'depset', want 'sequence or function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attrs=1)
print(x)"#,
        Err(r#"in call to aspect(), parameter 'attrs' got value of type 'int', want 'dict'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attrs="s")
print(x)"#,
        Err(r#"in call to aspect(), parameter 'attrs' got value of type 'string', want 'dict'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attrs=[])
print(x)"#,
        Err(r#"in call to aspect(), parameter 'attrs' got value of type 'list', want 'dict'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attrs={})
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attrs=None)
print(x)"#,
        Err(r#"in call to aspect(), parameter 'attrs' got value of type 'NoneType', want 'dict'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attrs=S)
print(x)"#,
        Err(r#"in call to aspect(), parameter 'attrs' got value of type 'struct', want 'dict'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attrs={"a":"b"})
print(x)"#,
        Err(r#"got dict<string, string> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attrs={"a":1})
print(x)"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, attrs={1:2})
print(x)"#,
        Err(r#"got dict<int, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=1)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'required_providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers="s")
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'required_providers' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=[])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers={})
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'required_providers' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=None)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'required_providers' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=())
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=P)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'required_providers' got value of type 'Provider', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=[P])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=provider())
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'required_providers' got value of type 'Provider', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=_H)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'required_providers' got value of type 'Provider', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=[[P]])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=[[P, Q], [Q]])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=[P, [Q]])
print(x)"#,
        Err(r#"at index 0 of required_providers, got element of type Provider, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=[1])
print(x)"#,
        Err(r#"at index 0 of required_providers, got element of type int, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=[[]])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=[[1]])
print(x)"#,
        Err(r#"at index 0 of required_providers, got element of type int, want Provider"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=["a"])
print(x)"#,
        Err(r#"at index 0 of required_providers, got element of type string, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=range(2))
print(x)"#,
        Err(r#"at index 0 of required_providers, got element of type int, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=S)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'required_providers' got value of type 'struct', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_providers=aspect(f))
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'required_providers' got value of type 'Aspect', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_aspect_providers=1)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'required_aspect_providers' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_aspect_providers=[])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_aspect_providers=None)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'required_aspect_providers' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_aspect_providers=P)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'required_aspect_providers' got value of type 'Provider', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_aspect_providers=[P])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_aspect_providers=provider())
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'required_aspect_providers' got value of type 'Provider', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_aspect_providers=_H)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'required_aspect_providers' got value of type 'Provider', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_aspect_providers=[[P]])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_aspect_providers=[P, [Q]])
print(x)"#,
        Err(
            r#"at index 0 of required_aspect_providers, got element of type Provider, want sequence"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_aspect_providers=[1])
print(x)"#,
        Err(r#"at index 0 of required_aspect_providers, got element of type int, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_aspect_providers=[[]])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, required_aspect_providers=S)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'required_aspect_providers' got value of type 'struct', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, provides=1)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'provides' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, provides="s")
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'provides' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, provides=[])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, provides={})
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'provides' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, provides=None)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'provides' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, provides=())
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, provides=P)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'provides' got value of type 'Provider', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, provides=[P])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, provides=provider())
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'provides' got value of type 'Provider', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, provides=_H)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'provides' got value of type 'Provider', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, provides=[[P]])
print(x)"#,
        Err(r#"at index 0 of provides, got element of type list, want Provider"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, provides=[1])
print(x)"#,
        Err(r#"at index 0 of provides, got element of type int, want Provider"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, provides=["a"])
print(x)"#,
        Err(r#"at index 0 of provides, got element of type string, want Provider"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, provides=S)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'provides' got value of type 'struct', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, provides=aspect(f))
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'provides' got value of type 'Aspect', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, requires=1)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'requires' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, requires=[])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, requires=None)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'requires' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, requires=())
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, requires=aspect(f))
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'requires' got value of type 'Aspect', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, requires=S)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'requires' got value of type 'struct', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, requires=[1])
print(x)"#,
        Err(r#"at index 0 of requires, got element of type int, want Aspect"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, requires=P)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'requires' got value of type 'Provider', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, requires=f)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'requires' got value of type 'function', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, propagation_predicate=1)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, propagation_predicate="s")
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'string', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, propagation_predicate=[])
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'list', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, propagation_predicate=None)
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, propagation_predicate=f)
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, propagation_predicate=lambda t, c: [])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, propagation_predicate=len)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'builtin_function_or_method', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, propagation_predicate=S)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'struct', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, propagation_predicate=aspect(f))
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'Aspect', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, fragments=1)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, fragments="s")
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'fragments' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, fragments=[])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, fragments=None)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'fragments' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, fragments=())
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, fragments=["a"])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, fragments=[1])
print(x)"#,
        Err(r#"at index 0 of fragments, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, fragments=S)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'fragments' got value of type 'struct', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, fragments=range(2))
print(x)"#,
        Err(r#"at index 0 of fragments, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, host_fragments=1)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, host_fragments="s")
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, host_fragments=[])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, host_fragments=None)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, host_fragments=["a"])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, host_fragments=[1])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains=1)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'toolchains' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains="s")
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'toolchains' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains=[])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains=None)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'toolchains' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains=())
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains=["a"])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains=[1])
print(x)"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains=["//x:y"])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains=["x y"])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains=["//x:"])
print(x)"#,
        Err(
            r#"Unable to parse toolchain_type label '//x:': invalid target name '': empty target name"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains=[L])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains=S)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'toolchains' got value of type 'struct', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, toolchains={})
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'toolchains' got value of type 'dict', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, doc=1)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, doc="s")
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, doc=None)
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, doc=[])
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, doc=True)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_compatible_with=1)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'exec_compatible_with' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_compatible_with="s")
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'exec_compatible_with' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_compatible_with=[])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_compatible_with=None)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'exec_compatible_with' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_compatible_with=())
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_compatible_with=["a"])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_compatible_with=[1])
print(x)"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_compatible_with=["//x:y"])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_compatible_with=["x y"])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_compatible_with=["//x:"])
print(x)"#,
        Err(
            r#"Unable to parse label '//x:' in attribute 'exec_compatible_with': invalid target name '': empty target name"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_compatible_with=[L])
print(x)"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type Label, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_compatible_with=S)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'exec_compatible_with' got value of type 'struct', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_groups=1)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'exec_groups' got value of type 'int', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_groups="s")
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'exec_groups' got value of type 'string', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_groups=[])
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'exec_groups' got value of type 'list', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_groups={})
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_groups=None)
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_groups=S)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'exec_groups' got value of type 'struct', want 'dict or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_groups={'e':exec_group()})
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_groups={"a":"b"})
print(x)"#,
        Err(r#"got dict<string, string> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_groups={"a":1})
print(x)"#,
        Err(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, exec_groups={1:2})
print(x)"#,
        Err(r#"got dict<int, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, subrules=1)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'subrules' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, subrules=[])
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, subrules=None)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'subrules' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, subrules=())
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, subrules=S)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'subrules' got value of type 'struct', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, subrules=aspect(f))
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'subrules' got value of type 'Aspect', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, subrules=[1])
print(x)"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, apply_to_generating_rules=1)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, apply_to_generating_rules="s")
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'string', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, apply_to_generating_rules=[])
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'list', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, apply_to_generating_rules=None)
print(x)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(): return []
P=provider()
Q=provider(fields=['x'])
_H=provider()
S=struct()
L=Label('//x:y')
x=aspect(f, apply_to_generating_rules=True)
print(x)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
t=transition()
print(t)"#,
        Err(r#"transition() missing 3 required named arguments: implementation, inputs, outputs"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
t=transition(implementation=g)
print(t)"#,
        Err(r#"transition() missing 2 required named arguments: inputs, outputs"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
t=transition(implementation=g, inputs=[])
print(t)"#,
        Err(r#"transition() missing 1 required named argument: outputs"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
t=transition(implementation=g, outputs=[])
print(t)"#,
        Err(r#"transition() missing 1 required named argument: inputs"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
t=transition(implementation=g, inputs=[], outputs=[])
print(t)
print(type(t))
print(repr(t))
print(str(t))"#,
        Ok(r#"<transition object>
transition
<transition object>
<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
print(t==u)
print(t==t)"#,
        Ok(r#"True
True"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
t=transition(implementation=g, inputs=[], outputs=[])
print({t:1})"#,
        Err(r#"unhashable type: 'transition'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
t=transition(implementation=g, inputs=[], outputs=[])
print(dir(t))"#,
        Ok(r#"["and_then"]"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
t=transition(implementation=g, inputs=[], outputs=[])
print(t.foo)"#,
        Err(r#"'transition' value has no field or method 'foo'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
t=transition(implementation=g, inputs=[], outputs=[])
print(bool(t))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
T=transition(implementation=g, inputs=[], outputs=[])
print(T)"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu']))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=['//x:y'], outputs=['//x:y']))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=[':y'], outputs=[':y']))"#,
        Err(
            r#"invalid transition input ':y'. If this is intended as a native option, it must begin with //command_line_option: invalid label ':y': absolute label must begin with '@' or '//'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=['y'], outputs=['y']))"#,
        Err(
            r#"invalid transition input 'y'. If this is intended as a native option, it must begin with //command_line_option: invalid label 'y': absolute label must begin with '@' or '//'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=['x y'], outputs=['y']))"#,
        Err(
            r#"invalid transition input 'x y'. If this is intended as a native option, it must begin with //command_line_option: invalid label 'x y': absolute label must begin with '@' or '//'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=[''], outputs=['y']))"#,
        Err(
            r#"invalid transition input ''. If this is intended as a native option, it must begin with //command_line_option: invalid target name '': empty target name"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=[], outputs=['']))"#,
        Err(
            r#"invalid transition output ''. If this is intended as a native option, it must begin with //command_line_option: invalid target name '': empty target name"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=[], outputs=['//x:']))"#,
        Err(
            r#"invalid transition output '//x:'. If this is intended as a native option, it must begin with //command_line_option: invalid target name '': empty target name"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=[], outputs=[':']))"#,
        Err(
            r#"invalid transition output ':'. If this is intended as a native option, it must begin with //command_line_option: invalid target name '': empty target name"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=[], outputs=['@@//x:y']))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=[], outputs=['@foo//x:y']))"#,
        Err(
            r#"invalid transition output '@@[unknown repo 'foo' requested from @@]//x:y': no repo visible as @foo from main repository"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=[], outputs=['//command_line_option:']))"#,
        Err(r#"Malformed label in transition OUTPUTS parameter: '//command_line_option:'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=[], outputs=['//command_line_option:cpu','//command_line_option:cpu']))"#,
        Err(r#"duplicate transition output '//command_line_option:cpu'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=['//command_line_option:cpu','//command_line_option:cpu'], outputs=[]))"#,
        Err(r#"duplicate transition input '//command_line_option:cpu'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=[Label('//x:y')], outputs=[]))"#,
        Err(r#"at index 0 of inputs, got element of type Label, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=[1], outputs=[]))"#,
        Err(r#"at index 0 of inputs, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=[], outputs=[1]))"#,
        Err(r#"at index 0 of outputs, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=('a',), outputs=()))"#,
        Err(
            r#"invalid transition input 'a'. If this is intended as a native option, it must begin with //command_line_option: invalid label 'a': absolute label must begin with '@' or '//'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs='a', outputs=[]))"#,
        Err(
            r#"in call to transition(), parameter 'inputs' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=[], outputs='a'))"#,
        Err(
            r#"in call to transition(), parameter 'outputs' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=None, outputs=[]))"#,
        Err(
            r#"in call to transition(), parameter 'inputs' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=[], outputs=None))"#,
        Err(
            r#"in call to transition(), parameter 'outputs' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=1, inputs=[], outputs=[]))"#,
        Err(
            r#"in call to transition(), parameter 'implementation' got value of type 'int', want 'callable'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=None, inputs=[], outputs=[]))"#,
        Err(
            r#"in call to transition(), parameter 'implementation' got value of type 'NoneType', want 'callable'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=len, inputs=[], outputs=[]))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=lambda s, a: {}, inputs=[], outputs=[]))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=lambda: {}, inputs=[], outputs=[]))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=lambda s: {}, inputs=[], outputs=[]))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=lambda s, a, b: {}, inputs=[], outputs=[]))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=lambda *a: {}, inputs=[], outputs=[]))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=lambda **a: {}, inputs=[], outputs=[]))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=lambda s, a=1: {}, inputs=[], outputs=[]))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=aspect(f), inputs=[], outputs=[]))"#,
        Err(
            r#"in call to transition(), parameter 'implementation' got value of type 'Aspect', want 'callable'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(g, [], []))"#,
        Err(r#"transition() got unexpected positional argument"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(foo=1, implementation=g, inputs=[], outputs=[]))"#,
        Err(r#"transition() got unexpected keyword argument 'foo'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(transition(implementation=g, inputs=[], outputs=[], doc='x'))"#,
        Err(r#"transition() got unexpected keyword argument 'doc'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
x=[transition(implementation=g, inputs=[], outputs=[])]
print(x)"#,
        Ok(r#"[<transition object>]"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
def h(): return transition(implementation=g, inputs=[], outputs=[])"#,
        Ok(r#""#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
t=analysis_test_transition(settings={})
print(t)
print(type(t))"#,
        Ok(r#"<analysis_test_transition object>
transition"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
t=analysis_test_transition(settings={'//command_line_option:cpu':'x'})
print(t)"#,
        Ok(r#"<analysis_test_transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(analysis_test_transition())"#,
        Err(r#"analysis_test_transition() missing 1 required named argument: settings"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(analysis_test_transition({}))"#,
        Err(r#"analysis_test_transition() got unexpected positional argument"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(analysis_test_transition(settings=[]))"#,
        Err(
            r#"in call to analysis_test_transition(), parameter 'settings' got value of type 'list', want 'dict'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(analysis_test_transition(settings=None))"#,
        Err(
            r#"in call to analysis_test_transition(), parameter 'settings' got value of type 'NoneType', want 'dict'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(analysis_test_transition(settings={'x':'y'}))"#,
        Err(
            r#"invalid transition output 'x'. If this is intended as a native option, it must begin with //command_line_option: invalid label 'x': absolute label must begin with '@' or '//'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(analysis_test_transition(settings={'//x:y':1}))"#,
        Ok(r#"<analysis_test_transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(analysis_test_transition(settings={1:'y'}))"#,
        Err(r#"got dict<int, string> for 'changed_settings dict', want dict<string, unknown>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(analysis_test_transition(settings={'//x:y':[]}))"#,
        Ok(r#"<analysis_test_transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(analysis_test_transition(settings={'//x:y':None}))"#,
        Ok(r#"<analysis_test_transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(analysis_test_transition(settings={'//x:y':True}))"#,
        Ok(r#"<analysis_test_transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(analysis_test_transition(settings={'//x:y':'a'}, foo=1))"#,
        Err(r#"analysis_test_transition() got unexpected keyword argument 'foo'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
t=analysis_test_transition(settings={})
u=analysis_test_transition(settings={})
print(t==u)"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
t=analysis_test_transition(settings={})
print(dir(t))
print(bool(t))
print({t:1})"#,
        Err(r#"unhashable type: 'transition'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(config)
print(type(config))
print(dir(config))"#,
        Ok(r#"<config>
config
["bool", "exec", "int", "none", "string", "string_list", "string_set", "target"]"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(config.exec())
print(type(config.exec()))"#,
        Ok(
            r#"<unknown object com.google.devtools.build.lib.analysis.config.ExecutionTransitionFactory>
ExecTransitionFactory"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(config.exec('x'))
print(config.exec(exec_group='x'))
print(config.exec(None))"#,
        Ok(
            r#"<unknown object com.google.devtools.build.lib.analysis.config.ExecutionTransitionFactory>
<unknown object com.google.devtools.build.lib.analysis.config.ExecutionTransitionFactory>
<unknown object com.google.devtools.build.lib.analysis.config.ExecutionTransitionFactory>"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(config.exec(1))"#,
        Err(
            r#"in call to exec(), parameter 'exec_group' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(config.exec('x', 'y'))"#,
        Err(r#"exec() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(config.exec(foo=1))"#,
        Err(r#"exec() got unexpected keyword argument 'foo'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(config.target())
print(type(config.target()))"#,
        Ok(
            r#"<unknown object com.google.devtools.build.lib.analysis.config.transitions.NoTransition$Factory>
transition"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(config.none())
print(type(config.none()))"#,
        Ok(
            r#"<unknown object com.google.devtools.build.lib.analysis.config.transitions.NoConfigTransition$Factory>
transition"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(config.target(1))"#,
        Err(r#"target() got unexpected positional argument"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(config.none(1))"#,
        Err(r#"none() got unexpected positional argument"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(config.target(x=1))"#,
        Err(r#"target() got unexpected keyword argument 'x'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(config.exec()==config.exec())
print(config.target()==config.target())
print(config.none()==config.none())
print(config.target()==config.none())"#,
        Ok(r#"False
True
True
False"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(config.exec('a')==config.exec('a'))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print({config.target():1, config.none():2, config.exec():3})"#,
        Err(r#"unhashable type: 'transition'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(bool(config.target()), bool(config.exec()), bool(config.none()))"#,
        Ok(r#"True True True"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(dir(config.exec()), dir(config.target()), dir(config.none()))"#,
        Ok(r#"["and_then"] ["and_then"] ["and_then"]"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(repr(config.exec()), str(config.target()))"#,
        Ok(
            r#"<unknown object com.google.devtools.build.lib.analysis.config.ExecutionTransitionFactory> <unknown object com.google.devtools.build.lib.analysis.config.transitions.NoTransition$Factory>"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(config.foo)"#,
        Err(r#"'config' value has no field or method 'foo'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(config.exec)"#,
        Ok(r#"<built-in method exec of config value>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(config.target)"#,
        Ok(r#"<built-in method target of config value>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(type(config.target))"#,
        Ok(r#"builtin_function_or_method"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
config.x=1"#,
        Err(r#"cannot set .x field of config value"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t==u)
print(t==transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu']))"#,
        Ok(r#"False
True"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t==transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu']))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t==transition(implementation=g, inputs=[], outputs=['//command_line_option:cpu']))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t==transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=[]))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(transition(implementation=g, inputs=['//a:b','//a:c'], outputs=[])==transition(implementation=g, inputs=['//a:c','//a:b'], outputs=[]))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(transition(implementation=g, inputs=('//a:b',), outputs=[])==transition(implementation=g, inputs=['//a:b'], outputs=[]))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(transition(implementation=g, inputs=['//a:b'], outputs=[])==transition(implementation=g, inputs=['@@//a:b'], outputs=[]))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(transition(implementation=g, inputs=['//a:b'], outputs=[])==transition(implementation=g, inputs=['@//a:b'], outputs=[]))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t==1)
print(t==None)
print(t!=u)"#,
        Ok(r#"False
False
True"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then)"#,
        Ok(r#"<built-in method and_then of transition value>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then(u))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(type(t.and_then(u)))"#,
        Ok(r#"transition"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then(u)==t.and_then(u))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then())"#,
        Err(r#"and_then() missing 1 required positional argument: transition"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then(1))"#,
        Err(
            r#"in call to and_then(), parameter 'transition' got value of type 'int', want 'transition'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then(t))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then(u, u))"#,
        Err(r#"and_then() accepts no more than 1 positional argument but got 2"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then(other=u))"#,
        Err(r#"and_then() got unexpected keyword argument 'other'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then(config.target()))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then(config.none()))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then(config.exec()))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then(analysis_test_transition(settings={})))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(analysis_test_transition(settings={}).and_then(t))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(config.target().and_then(t))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(config.none().and_then(t))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(config.exec().and_then(t))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then(f))"#,
        Err(
            r#"in call to and_then(), parameter 'transition' got value of type 'function', want 'transition'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then('x'))"#,
        Err(
            r#"in call to and_then(), parameter 'transition' got value of type 'string', want 'transition'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(dir(t.and_then(u)))"#,
        Ok(r#"["and_then"]"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then(u).and_then(u))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then(u).and_then(t.and_then(u)))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then(u)==u.and_then(t))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t.and_then(u, extra=1))"#,
        Err(r#"and_then() got unexpected keyword argument 'extra'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print([t, u])"#,
        Ok(r#"[<transition object>, <transition object>]"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t in [t])"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(t in [u])"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print({'a':t})"#,
        Ok(r#"{"a": <transition object>}"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(config.exec().and_then)"#,
        Ok(r#"<built-in method and_then of ExecTransitionFactory value>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(config.target().and_then)"#,
        Ok(r#"<built-in method and_then of transition value>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(settings, attr): return {}
t=transition(implementation=g, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:cpu'])
u=transition(implementation=h, inputs=['//command_line_option:cpu'], outputs=['//command_line_option:compilation_mode'])
print(type(config.exec().and_then))"#,
        Ok(r#"builtin_function_or_method"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
e=exec_group()
print(e)
print(type(e))
print(repr(e))
print(str(e))"#,
        Ok(
            r#"<unknown object com.google.devtools.build.lib.packages.DeclaredExecGroup>
exec_group
<unknown object com.google.devtools.build.lib.packages.DeclaredExecGroup>
<unknown object com.google.devtools.build.lib.packages.DeclaredExecGroup>"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
e=exec_group(exec_compatible_with=['//x:y'], toolchains=['//x:y'])
print(e)"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.packages.DeclaredExecGroup>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(exec_group(1))"#,
        Err(r#"exec_group() got unexpected positional argument"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(exec_group(foo=1))"#,
        Err(r#"exec_group() got unexpected keyword argument 'foo'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(exec_group(exec_compatible_with=[1]))"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(exec_group(exec_compatible_with=['x y']))"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.packages.DeclaredExecGroup>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(exec_group(exec_compatible_with=['//x:']))"#,
        Err(
            r#"Unable to parse label '//x:' in attribute 'exec_compatible_with': invalid target name '': empty target name"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(exec_group(exec_compatible_with=[Label('//x:y')]))"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type Label, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(exec_group(exec_compatible_with=None))"#,
        Err(
            r#"in call to exec_group(), parameter 'exec_compatible_with' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(exec_group(exec_compatible_with='a'))"#,
        Err(
            r#"in call to exec_group(), parameter 'exec_compatible_with' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(exec_group(toolchains=None))"#,
        Err(
            r#"in call to exec_group(), parameter 'toolchains' got value of type 'NoneType', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(exec_group(toolchains='a'))"#,
        Err(
            r#"in call to exec_group(), parameter 'toolchains' got value of type 'string', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(exec_group(toolchains=[1]))"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(exec_group(toolchains=['//x:']))"#,
        Err(
            r#"Unable to parse toolchain_type label '//x:': invalid target name '': empty target name"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(exec_group(toolchains=['x y']))"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.packages.DeclaredExecGroup>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(exec_group(toolchains=[Label('//x:y')]))"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.packages.DeclaredExecGroup>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(exec_group(toolchains=['//x:y','//x:y']))"#,
        Ok(r#"<unknown object com.google.devtools.build.lib.packages.DeclaredExecGroup>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
e=exec_group()
print(e==e)
print(e==exec_group())
print({e:1})
print(dir(e))
print(bool(e))
print(e.foo)"#,
        Err(r#"unhashable type: 'exec_group'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
e=exec_group(toolchains=['//x:y'])
print(e==exec_group(toolchains=['//x:y']))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
e=exec_group(toolchains=['//x:y'])
print(e==exec_group(toolchains=[Label('//x:y')]))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
e=exec_group(exec_compatible_with=['//x:y'])
print(e==exec_group(exec_compatible_with=[Label('//x:y')]))"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type Label, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
c=configuration_field(fragment='cpp', name='cc_compiler')
print(c)
print(type(c))
print(repr(c))
print(str(c))"#,
        Err(r#"invalid configuration field name 'cc_compiler' on fragment 'cpp'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(configuration_field('cpp','cc_compiler'))"#,
        Err(r#"invalid configuration field name 'cc_compiler' on fragment 'cpp'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(configuration_field(fragment='cpp'))"#,
        Err(r#"configuration_field() missing 1 required positional argument: name"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(configuration_field(name='cc_compiler'))"#,
        Err(r#"configuration_field() missing 1 required positional argument: fragment"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(configuration_field())"#,
        Err(r#"configuration_field() missing 2 required positional arguments: fragment, name"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(configuration_field(1,2))"#,
        Err(
            r#"in call to configuration_field(), parameter 'fragment' got value of type 'int', want 'string'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(configuration_field('cpp','x'))"#,
        Err(r#"invalid configuration field name 'x' on fragment 'cpp'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(configuration_field('x','cc_compiler'))"#,
        Err(r#"invalid configuration fragment name 'x'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(configuration_field('x','y'))"#,
        Err(r#"invalid configuration fragment name 'x'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(configuration_field('','y'))"#,
        Err(r#"invalid configuration fragment name ''"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(configuration_field('cpp',''))"#,
        Err(r#"invalid configuration field name '' on fragment 'cpp'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(configuration_field('cpp','cc_compiler', 1))"#,
        Err(r#"configuration_field() accepts no more than 2 positional arguments but got 3"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(configuration_field(fragment='cpp', name='cc_compiler', foo=1))"#,
        Err(r#"configuration_field() got unexpected keyword argument 'foo'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
c=configuration_field('cpp','cc_compiler')
print(c==c)
print(c==configuration_field('cpp','cc_compiler'))
print({c:1})
print(dir(c))
print(bool(c))
print(c.foo)"#,
        Err(r#"invalid configuration field name 'cc_compiler' on fragment 'cpp'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(attr.label(default=configuration_field('cpp','cc_compiler')))"#,
        Err(r#"invalid configuration field name 'cc_compiler' on fragment 'cpp'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(attr.label(default=configuration_field('x','y')))"#,
        Err(r#"invalid configuration fragment name 'x'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(attr.string(default=configuration_field('cpp','cc_compiler')))"#,
        Err(r#"invalid configuration field name 'cc_compiler' on fragment 'cpp'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(attr.label(default=configuration_field('cpp','cc_compiler'), executable=True, cfg='exec'))"#,
        Err(r#"invalid configuration field name 'cc_compiler' on fragment 'cpp'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
P=provider()
print(attr.label_list(default=[configuration_field('cpp','cc_compiler')]))"#,
        Err(r#"invalid configuration field name 'cc_compiler' on fragment 'cpp'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
s=subrule(implementation=h)
print(s)
print(type(s))
print(repr(s))
print(str(s))"#,
        Ok(r#"<subrule s>
Subrule
<subrule s>
<subrule s>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule())"#,
        Err(r#"subrule() missing 1 required named argument: implementation"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(h))"#,
        Err(r#"subrule() got unexpected positional argument"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=1))"#,
        Err(
            r#"in call to subrule(), parameter 'implementation' got value of type 'int', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=None))"#,
        Err(
            r#"in call to subrule(), parameter 'implementation' got value of type 'NoneType', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=len))"#,
        Err(
            r#"in call to subrule(), parameter 'implementation' got value of type 'builtin_function_or_method', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=lambda: 1))"#,
        Ok(r#"<subrule unexported subrule>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs=1))"#,
        Err(r#"in call to subrule(), parameter 'attrs' got value of type 'int', want 'dict'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs=None))"#,
        Err(r#"in call to subrule(), parameter 'attrs' got value of type 'NoneType', want 'dict'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={}))"#,
        Ok(r#"<subrule unexported subrule>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'_x':attr.label(default='//x:y')}))"#,
        Ok(r#"<subrule unexported subrule>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'x':attr.label(default='//x:y')}))"#,
        Err(
            r#"illegal attribute name 'x': subrules may only define private attributes (whose names begin with '_')."#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'_x':attr.label()}))"#,
        Err(r#"for attribute '_x': no default value specified"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'_x':attr.string()}))"#,
        Err(
            r#"bad type for attribute '_x': subrule attributes may only be label or lists of labels."#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'_x':attr.string(default='a')}))"#,
        Err(
            r#"bad type for attribute '_x': subrule attributes may only be label or lists of labels."#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'x':attr.string(default='a')}))"#,
        Err(
            r#"illegal attribute name 'x': subrules may only define private attributes (whose names begin with '_')."#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'x':attr.string()}))"#,
        Err(
            r#"illegal attribute name 'x': subrules may only define private attributes (whose names begin with '_')."#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'_x':attr.label_list(default=['//x:y'])}))"#,
        Ok(r#"<subrule unexported subrule>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'_x':attr.int(default=1)}))"#,
        Err(
            r#"bad type for attribute '_x': subrule attributes may only be label or lists of labels."#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'_x':attr.bool(default=True)}))"#,
        Err(
            r#"bad type for attribute '_x': subrule attributes may only be label or lists of labels."#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'_x':attr.output()}))"#,
        Err(r#"for attribute '_x': no default value specified"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'_x':attr.label(default='//x:y', allow_files=True)}))"#,
        Ok(r#"<subrule unexported subrule>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'_x':attr.label(default='//x:y', mandatory=True)}))"#,
        Ok(r#"<subrule unexported subrule>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'name':attr.label(default='//x:y')}))"#,
        Err(
            r#"illegal attribute name 'name': subrules may only define private attributes (whose names begin with '_')."#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'x y':attr.label(default='//x:y')}))"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={1:attr.label(default='//x:y')}))"#,
        Err(r#"got dict<int, Attribute> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'_x':1}))"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'_x':attr.label(default='//x:y', cfg='exec', executable=True)}))"#,
        Ok(r#"<subrule unexported subrule>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, attrs={'_x':attr.label(default='//x:y', aspects=[aspect(f)])}))"#,
        Err(r#"Aspects should be top-level values in extension files that define them."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, toolchains=1))"#,
        Err(
            r#"in call to subrule(), parameter 'toolchains' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, toolchains=[]))"#,
        Ok(r#"<subrule unexported subrule>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, toolchains=['//x:y']))"#,
        Ok(r#"<subrule unexported subrule>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, toolchains=[1]))"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, toolchains=['//x:']))"#,
        Err(
            r#"Unable to parse toolchain_type label '//x:': invalid target name '': empty target name"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, toolchains=[Label('//x:y')]))"#,
        Ok(r#"<subrule unexported subrule>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, fragments=1))"#,
        Err(
            r#"in call to subrule(), parameter 'fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, fragments=['a']))"#,
        Ok(r#"<subrule unexported subrule>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, fragments=[1]))"#,
        Err(r#"at index 0 of fragments, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, subrules=1))"#,
        Err(
            r#"in call to subrule(), parameter 'subrules' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, subrules=[]))"#,
        Ok(r#"<subrule unexported subrule>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, subrules=[1]))"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
s=subrule(implementation=h)
print(subrule(implementation=h, subrules=[s]))"#,
        Ok(r#"<subrule unexported subrule>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
s=subrule(implementation=h)
print(s==s)
print(s==subrule(implementation=h))
print({s:1})
print(dir(s))
print(bool(s))
print(s.foo)"#,
        Err(r#"unhashable type: 'Subrule'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
s=subrule(implementation=h)
print(s('x'))"#,
        Err(r#"s can only be called from a rule or aspect implementation"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
s=subrule(implementation=h)
print(s())"#,
        Err(r#"s can only be called from a rule or aspect implementation"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
S=subrule(implementation=h)
print(S)"#,
        Ok(r#"<subrule S>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, doc='x'))"#,
        Err(r#"subrule() got unexpected keyword argument 'doc'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, foo=1))"#,
        Err(r#"subrule() got unexpected keyword argument 'foo'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, exec_compatible_with=[]))"#,
        Err(r#"subrule() got unexpected keyword argument 'exec_compatible_with'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, host_fragments=[]))"#,
        Err(r#"subrule() got unexpected keyword argument 'host_fragments'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(subrule(implementation=h, exec_groups={}))"#,
        Err(r#"subrule() got unexpected keyword argument 'exec_groups'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.bool())
print(type(config.bool()))"#,
        Ok(r#"<build_setting.boolean>
BuildSetting"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.bool(flag=True))
print(config.bool(flag=False))
print(config.bool(1))"#,
        Err(r#"bool() got unexpected positional argument"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.bool(flag=1))"#,
        Err(r#"in call to bool(), parameter 'flag' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.bool(flag=None))"#,
        Err(r#"in call to bool(), parameter 'flag' got value of type 'NoneType', want 'bool'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.bool(foo=1))"#,
        Err(r#"bool() got unexpected keyword argument 'foo'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.bool(True, 1))"#,
        Err(r#"bool() got unexpected positional argument"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.int())
print(config.int(flag=True))
print(config.int(flag=1))"#,
        Err(r#"in call to int(), parameter 'flag' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.string())
print(config.string(flag=True))
print(config.string(allow_multiple=True))
print(config.string(flag=True, allow_multiple=True))"#,
        Ok(r#"<build_setting.string>
<build_setting.string>
<build_setting.string>
<build_setting.string>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.string(allow_multiple=1))"#,
        Err(
            r#"in call to string(), parameter 'allow_multiple' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.string_list())
print(config.string_list(flag=True))
print(config.string_list(repeatable=True))"#,
        Err(r#"'repeatable' can only be set for a setting with 'flag = True'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.string_list(repeatable=1))"#,
        Err(
            r#"in call to string_list(), parameter 'repeatable' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.string_set())
print(config.string_set(flag=True))
print(config.string_set(repeatable=True))"#,
        Err(r#"'repeatable' can only be set for a setting with 'flag = True'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.string_set(foo=1))"#,
        Err(r#"string_set() got unexpected keyword argument 'foo'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.bool()==config.bool())
print(config.bool()==config.int())
print(config.bool(flag=True)==config.bool(flag=True))
print({config.bool():1})"#,
        Err(r#"unhashable type: 'BuildSetting'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
b=config.bool()
print(dir(b))
print(bool(b))
print(b.foo)"#,
        Err(r#"'BuildSetting' value has no field or method 'foo'"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.string(True))"#,
        Err(r#"string() got unexpected positional argument"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.string(True, True))"#,
        Err(r#"string() got unexpected positional argument"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.string(True, True, True))"#,
        Err(r#"string() got unexpected positional argument"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.string_list(True, True))"#,
        Err(r#"string_list() got unexpected positional argument"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.string_set(True, True))"#,
        Err(r#"string_set() got unexpected positional argument"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(config.int(True, True))"#,
        Err(r#"int() got unexpected positional argument"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(type(config.int()), type(config.string()), type(config.string_list()), type(config.string_set()))"#,
        Ok(r#"BuildSetting BuildSetting BuildSetting BuildSetting"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
print(repr(config.int()), repr(config.string()), repr(config.string_list()), repr(config.string_set()))"#,
        Ok(
            r#"<build_setting.int> <build_setting.string> <build_setting.list(string)> <build_setting.set(string)>"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
A=aspect(f)
x=attr.label(aspects=[A])
print(1)"#,
        Ok(r#"1"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
x=attr.label(aspects=[aspect(f)])
print(1)"#,
        Err(r#"Aspects should be top-level values in extension files that define them."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
A=aspect(f)
B=aspect(f, requires=[A])
print(B)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
B=aspect(f, requires=[aspect(f)])
print(B)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
A=aspect(f)
B=[aspect(f, requires=[A])]
print(B)"#,
        Ok(r#"[<aspect>]"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
_A=aspect(f)
x=attr.label(aspects=[_A])
print(1)"#,
        Ok(r#"1"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
A=aspect(f, provides=[provider()])
print(A)"#,
        Err(r#"Providers should be top-level values in extension files that define them."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
A=aspect(f, required_providers=[[provider()]])
print(A)"#,
        Err(r#"Providers should be top-level values in extension files that define them."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
A=aspect(f, required_aspect_providers=[[provider()]])
print(A)"#,
        Err(r#"Providers should be top-level values in extension files that define them."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
A=aspect(f, provides=[P])
print(A)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
A=[aspect(f)]
x=attr.label(aspects=A)
print(1)"#,
        Err(r#"Aspects should be top-level values in extension files that define them."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
A=aspect(f)
print(A)
x=attr.label(aspects=[A])
print(A)"#,
        Ok(r#"<aspect>
<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
A=aspect(f, attrs={'_a':attr.label(default='//x:y')})
print(A)"#,
        Ok(r#"<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
A=aspect(f)
B=A
print(A==B)
print(A)"#,
        Ok(r#"True
<aspect>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
A=aspect(f)
A2=A
x={A:1}
print(x)"#,
        Ok(r#"{<aspect>: 1}"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
A=aspect(f)
print({A:1}[A])"#,
        Ok(r#"1"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
def g(settings, attr): return {}
def h(ctx, x): return []
P=provider()
A=aspect(f)
print(A in [A])"#,
        Ok(r#"True"#),
    ),
    (
        r#"def f(t,c): return []
def r():
  aspect(f)
r()"#,
        Ok(r#""#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], toolchains_aspects=[1])"#,
        Err(r#"at index 0 of attr_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], attr_aspects=[1])"#,
        Err(r#"at index 0 of attr_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], attrs={'a':1})"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'a':1}, attr_aspects=[1])"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], required_providers=[1])"#,
        Err(r#"at index 0 of attr_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], attr_aspects=[1])"#,
        Err(r#"at index 0 of attr_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], required_aspect_providers=[1])"#,
        Err(r#"at index 0 of attr_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], attr_aspects=[1])"#,
        Err(r#"at index 0 of attr_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], provides=[1])"#,
        Err(r#"at index 0 of attr_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], attr_aspects=[1])"#,
        Err(r#"at index 0 of attr_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], requires=[1])"#,
        Err(r#"at index 0 of attr_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], attr_aspects=[1])"#,
        Err(r#"at index 0 of attr_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], fragments=[1])"#,
        Err(r#"at index 0 of attr_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], attr_aspects=[1])"#,
        Err(r#"at index 0 of attr_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], toolchains=[1])"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], attr_aspects=[1])"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], exec_compatible_with=[1])"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], attr_aspects=[1])"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], exec_groups={'a':1})"#,
        Err(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, attr_aspects=[1])"#,
        Err(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], subrules=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], attr_aspects=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], propagation_predicate=1)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, propagation_predicate=1, attr_aspects=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], doc=1)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, doc=1, attr_aspects=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], apply_to_generating_rules=1)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, apply_to_generating_rules=1, attr_aspects=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], host_fragments=1)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, host_fragments=1, attr_aspects=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], attrs={'a':1})"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'a':1}, toolchains_aspects=[1])"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], required_providers=[1])"#,
        Err(r#"at index 0 of toolchains_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], toolchains_aspects=[1])"#,
        Err(r#"at index 0 of toolchains_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], required_aspect_providers=[1])"#,
        Err(r#"at index 0 of toolchains_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], toolchains_aspects=[1])"#,
        Err(r#"at index 0 of toolchains_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], provides=[1])"#,
        Err(r#"at index 0 of toolchains_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], toolchains_aspects=[1])"#,
        Err(r#"at index 0 of toolchains_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], requires=[1])"#,
        Err(r#"at index 0 of toolchains_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], toolchains_aspects=[1])"#,
        Err(r#"at index 0 of toolchains_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], fragments=[1])"#,
        Err(r#"at index 0 of toolchains_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], toolchains_aspects=[1])"#,
        Err(r#"at index 0 of toolchains_aspects, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], toolchains=[1])"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], toolchains_aspects=[1])"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], exec_compatible_with=[1])"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], toolchains_aspects=[1])"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], exec_groups={'a':1})"#,
        Err(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, toolchains_aspects=[1])"#,
        Err(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], subrules=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], toolchains_aspects=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], propagation_predicate=1)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, propagation_predicate=1, toolchains_aspects=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], doc=1)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, doc=1, toolchains_aspects=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], apply_to_generating_rules=1)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, apply_to_generating_rules=1, toolchains_aspects=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], host_fragments=1)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, host_fragments=1, toolchains_aspects=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'a':1}, required_providers=[1])"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], attrs={'a':1})"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'a':1}, required_aspect_providers=[1])"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], attrs={'a':1})"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'a':1}, provides=[1])"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], attrs={'a':1})"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'a':1}, requires=[1])"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], attrs={'a':1})"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'a':1}, fragments=[1])"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], attrs={'a':1})"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'a':1}, toolchains=[1])"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], attrs={'a':1})"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'a':1}, exec_compatible_with=[1])"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], attrs={'a':1})"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'a':1}, exec_groups={'a':1})"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, attrs={'a':1})"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'a':1}, subrules=[1])"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], attrs={'a':1})"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'a':1}, propagation_predicate=1)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, propagation_predicate=1, attrs={'a':1})"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'a':1}, doc=1)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, doc=1, attrs={'a':1})"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'a':1}, apply_to_generating_rules=1)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, apply_to_generating_rules=1, attrs={'a':1})"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'a':1}, host_fragments=1)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, host_fragments=1, attrs={'a':1})"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], required_aspect_providers=[1])"#,
        Err(r#"at index 0 of required_providers, got element of type int, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], required_providers=[1])"#,
        Err(r#"at index 0 of required_providers, got element of type int, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], provides=[1])"#,
        Err(r#"at index 0 of required_providers, got element of type int, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], required_providers=[1])"#,
        Err(r#"at index 0 of required_providers, got element of type int, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], requires=[1])"#,
        Err(r#"at index 0 of required_providers, got element of type int, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], required_providers=[1])"#,
        Err(r#"at index 0 of required_providers, got element of type int, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], fragments=[1])"#,
        Err(r#"at index 0 of required_providers, got element of type int, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], required_providers=[1])"#,
        Err(r#"at index 0 of required_providers, got element of type int, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], toolchains=[1])"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], required_providers=[1])"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], exec_compatible_with=[1])"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], required_providers=[1])"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], exec_groups={'a':1})"#,
        Err(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, required_providers=[1])"#,
        Err(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], subrules=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], required_providers=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], propagation_predicate=1)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, propagation_predicate=1, required_providers=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], doc=1)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, doc=1, required_providers=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], apply_to_generating_rules=1)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, apply_to_generating_rules=1, required_providers=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], host_fragments=1)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, host_fragments=1, required_providers=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], provides=[1])"#,
        Err(r#"at index 0 of required_aspect_providers, got element of type int, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], required_aspect_providers=[1])"#,
        Err(r#"at index 0 of required_aspect_providers, got element of type int, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], requires=[1])"#,
        Err(r#"at index 0 of required_aspect_providers, got element of type int, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], required_aspect_providers=[1])"#,
        Err(r#"at index 0 of required_aspect_providers, got element of type int, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], fragments=[1])"#,
        Err(r#"at index 0 of required_aspect_providers, got element of type int, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], required_aspect_providers=[1])"#,
        Err(r#"at index 0 of required_aspect_providers, got element of type int, want sequence"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], toolchains=[1])"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], required_aspect_providers=[1])"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], exec_compatible_with=[1])"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], required_aspect_providers=[1])"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], exec_groups={'a':1})"#,
        Err(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, required_aspect_providers=[1])"#,
        Err(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], subrules=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], required_aspect_providers=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], propagation_predicate=1)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, propagation_predicate=1, required_aspect_providers=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], doc=1)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, doc=1, required_aspect_providers=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], apply_to_generating_rules=1)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, apply_to_generating_rules=1, required_aspect_providers=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], host_fragments=1)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, host_fragments=1, required_aspect_providers=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], requires=[1])"#,
        Err(r#"at index 0 of provides, got element of type int, want Provider"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], provides=[1])"#,
        Err(r#"at index 0 of provides, got element of type int, want Provider"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], fragments=[1])"#,
        Err(r#"at index 0 of provides, got element of type int, want Provider"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], provides=[1])"#,
        Err(r#"at index 0 of provides, got element of type int, want Provider"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], toolchains=[1])"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], provides=[1])"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], exec_compatible_with=[1])"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], provides=[1])"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], exec_groups={'a':1})"#,
        Err(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, provides=[1])"#,
        Err(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], subrules=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], provides=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], propagation_predicate=1)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, propagation_predicate=1, provides=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], doc=1)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, doc=1, provides=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], apply_to_generating_rules=1)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, apply_to_generating_rules=1, provides=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], host_fragments=1)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, host_fragments=1, provides=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], fragments=[1])"#,
        Err(r#"at index 0 of requires, got element of type int, want Aspect"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], requires=[1])"#,
        Err(r#"at index 0 of requires, got element of type int, want Aspect"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], toolchains=[1])"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], requires=[1])"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], exec_compatible_with=[1])"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], requires=[1])"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], exec_groups={'a':1})"#,
        Err(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, requires=[1])"#,
        Err(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], subrules=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], requires=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], propagation_predicate=1)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, propagation_predicate=1, requires=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], doc=1)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, doc=1, requires=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], apply_to_generating_rules=1)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, apply_to_generating_rules=1, requires=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], host_fragments=1)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, host_fragments=1, requires=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], toolchains=[1])"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], fragments=[1])"#,
        Err(r#"'toolchains' takes a toolchain_type, Label, or String, but instead got a Int32"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], exec_compatible_with=[1])"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], fragments=[1])"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], exec_groups={'a':1})"#,
        Err(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, fragments=[1])"#,
        Err(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], subrules=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], fragments=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], propagation_predicate=1)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, propagation_predicate=1, fragments=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], doc=1)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, doc=1, fragments=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], apply_to_generating_rules=1)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, apply_to_generating_rules=1, fragments=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], host_fragments=1)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, host_fragments=1, fragments=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], exec_compatible_with=[1])"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], toolchains=[1])"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], exec_groups={'a':1})"#,
        Err(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, toolchains=[1])"#,
        Err(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], subrules=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], toolchains=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], propagation_predicate=1)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, propagation_predicate=1, toolchains=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], doc=1)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, doc=1, toolchains=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], apply_to_generating_rules=1)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, apply_to_generating_rules=1, toolchains=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], host_fragments=1)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, host_fragments=1, toolchains=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], exec_groups={'a':1})"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, exec_compatible_with=[1])"#,
        Err(r#"at index 0 of exec_compatible_with, got element of type int, want string"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], subrules=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], exec_compatible_with=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], propagation_predicate=1)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, propagation_predicate=1, exec_compatible_with=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], doc=1)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, doc=1, exec_compatible_with=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], apply_to_generating_rules=1)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, apply_to_generating_rules=1, exec_compatible_with=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], host_fragments=1)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, host_fragments=1, exec_compatible_with=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, subrules=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], exec_groups={'a':1})"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, propagation_predicate=1)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, propagation_predicate=1, exec_groups={'a':1})"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, doc=1)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, doc=1, exec_groups={'a':1})"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, apply_to_generating_rules=1)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, apply_to_generating_rules=1, exec_groups={'a':1})"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, host_fragments=1)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, host_fragments=1, exec_groups={'a':1})"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], propagation_predicate=1)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, propagation_predicate=1, subrules=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], doc=1)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, doc=1, subrules=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], apply_to_generating_rules=1)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, apply_to_generating_rules=1, subrules=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], host_fragments=1)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, host_fragments=1, subrules=[1])"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, propagation_predicate=1, doc=1)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, doc=1, propagation_predicate=1)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, propagation_predicate=1, apply_to_generating_rules=1)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, apply_to_generating_rules=1, propagation_predicate=1)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, propagation_predicate=1, host_fragments=1)"#,
        Err(
            r#"in call to aspect(), parameter 'propagation_predicate' got value of type 'int', want 'function or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, host_fragments=1, propagation_predicate=1)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, doc=1, apply_to_generating_rules=1)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, apply_to_generating_rules=1, doc=1)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, doc=1, host_fragments=1)"#,
        Err(
            r#"in call to aspect(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, host_fragments=1, doc=1)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, apply_to_generating_rules=1, host_fragments=1)"#,
        Err(
            r#"in call to aspect(), parameter 'apply_to_generating_rules' got value of type 'int', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, host_fragments=1, apply_to_generating_rules=1)"#,
        Err(
            r#"in call to aspect(), parameter 'host_fragments' got value of type 'int', want 'sequence'"#,
        ),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'_x': attr.label()}, attr_aspects=[1])"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], attrs={'_x': attr.label()})"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'_x': attr.label()}, toolchains_aspects=[1])"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], attrs={'_x': attr.label()})"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'_x': attr.label()}, required_providers=[1])"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], attrs={'_x': attr.label()})"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'_x': attr.label()}, required_aspect_providers=[1])"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], attrs={'_x': attr.label()})"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'_x': attr.label()}, provides=[1])"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], attrs={'_x': attr.label()})"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'_x': attr.label()}, requires=[1])"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], attrs={'_x': attr.label()})"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'_x': attr.label()}, fragments=[1])"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], attrs={'_x': attr.label()})"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'_x': attr.label()}, toolchains=[1])"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], attrs={'_x': attr.label()})"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'_x': attr.label()}, exec_compatible_with=[1])"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], attrs={'_x': attr.label()})"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'_x': attr.label()}, exec_groups={'a':1})"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, attrs={'_x': attr.label()})"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'_x': attr.label()}, subrules=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], attrs={'_x': attr.label()})"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'_x': attr.label()}, exec_groups={'a b':exec_group()})"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a b':exec_group()}, attrs={'_x': attr.label()})"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'_x': attr.label()}, subrules=[subrule(implementation=f)])"#,
        Err(r#"Invalid subrule hasn't been exported by a bzl file"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[subrule(implementation=f)], attrs={'_x': attr.label()})"#,
        Err(r#"Invalid subrule hasn't been exported by a bzl file"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'_x': attr.label()}, provides=[provider()])"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[provider()], attrs={'_x': attr.label()})"#,
        Err(r#"Aspect attribute '_x' has no default value."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x': attr.label()}, attr_aspects=[1])"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], attrs={'x': attr.label()})"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x': attr.label()}, toolchains_aspects=[1])"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], attrs={'x': attr.label()})"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x': attr.label()}, required_providers=[1])"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], attrs={'x': attr.label()})"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x': attr.label()}, required_aspect_providers=[1])"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], attrs={'x': attr.label()})"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x': attr.label()}, provides=[1])"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], attrs={'x': attr.label()})"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x': attr.label()}, requires=[1])"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], attrs={'x': attr.label()})"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x': attr.label()}, fragments=[1])"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], attrs={'x': attr.label()})"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x': attr.label()}, toolchains=[1])"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], attrs={'x': attr.label()})"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x': attr.label()}, exec_compatible_with=[1])"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], attrs={'x': attr.label()})"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x': attr.label()}, exec_groups={'a':1})"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, attrs={'x': attr.label()})"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x': attr.label()}, subrules=[1])"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], attrs={'x': attr.label()})"#,
        Err(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x': attr.label()}, exec_groups={'a b':exec_group()})"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a b':exec_group()}, attrs={'x': attr.label()})"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x': attr.label()}, subrules=[subrule(implementation=f)])"#,
        Err(r#"Invalid subrule hasn't been exported by a bzl file"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[subrule(implementation=f)], attrs={'x': attr.label()})"#,
        Err(r#"Invalid subrule hasn't been exported by a bzl file"#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x': attr.label()}, provides=[provider()])"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[provider()], attrs={'x': attr.label()})"#,
        Err(r#"Aspect parameter attribute 'x' must have type 'bool', 'int' or 'string'."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x y': attr.string()}, attr_aspects=[1])"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attr_aspects=[1], attrs={'x y': attr.string()})"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x y': attr.string()}, toolchains_aspects=[1])"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains_aspects=[1], attrs={'x y': attr.string()})"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x y': attr.string()}, required_providers=[1])"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_providers=[1], attrs={'x y': attr.string()})"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x y': attr.string()}, required_aspect_providers=[1])"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, required_aspect_providers=[1], attrs={'x y': attr.string()})"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x y': attr.string()}, provides=[1])"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[1], attrs={'x y': attr.string()})"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x y': attr.string()}, requires=[1])"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, requires=[1], attrs={'x y': attr.string()})"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x y': attr.string()}, fragments=[1])"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, fragments=[1], attrs={'x y': attr.string()})"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x y': attr.string()}, toolchains=[1])"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, toolchains=[1], attrs={'x y': attr.string()})"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x y': attr.string()}, exec_compatible_with=[1])"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_compatible_with=[1], attrs={'x y': attr.string()})"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x y': attr.string()}, exec_groups={'a':1})"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a':1}, attrs={'x y': attr.string()})"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x y': attr.string()}, subrules=[1])"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[1], attrs={'x y': attr.string()})"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x y': attr.string()}, exec_groups={'a b':exec_group()})"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, exec_groups={'a b':exec_group()}, attrs={'x y': attr.string()})"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x y': attr.string()}, subrules=[subrule(implementation=f)])"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, subrules=[subrule(implementation=f)], attrs={'x y': attr.string()})"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, attrs={'x y': attr.string()}, provides=[provider()])"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(target, ctx): return []
P=provider()
aspect(f, provides=[provider()], attrs={'x y': attr.string()})"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['//x:y','@@//x:y'], outputs=[]))"#,
        Err(
            r#"Transition declares duplicate build setting '//x:y' in INPUTS (specified as '@@//x:y' and '//x:y')"#,
        ),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['//x:y','@//x:y'], outputs=[]))"#,
        Err(
            r#"Transition declares duplicate build setting '//x:y' in INPUTS (specified as '@//x:y' and '//x:y')"#,
        ),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=[], outputs=['//x:y','@//x:y']))"#,
        Err(
            r#"Transition declares duplicate build setting '//x:y' in OUTPUTS (specified as '@//x:y' and '//x:y')"#,
        ),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['//x:y', '//x:y'], outputs=['//x:']))"#,
        Err(r#"duplicate transition input '//x:y'"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['//command_line_option:'], outputs=[]))"#,
        Err(r#"Malformed label in transition INPUTS parameter: '//command_line_option:'"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['//command_line_option:a','//command_line_option:a'], outputs=['//command_line_option:']))"#,
        Err(r#"duplicate transition input '//command_line_option:a'"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['y'], outputs=['//command_line_option:']))"#,
        Err(
            r#"invalid transition input 'y'. If this is intended as a native option, it must begin with //command_line_option: invalid label 'y': absolute label must begin with '@' or '//'"#,
        ),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=[], outputs=['//command_line_option:cpu', 'x']))"#,
        Err(
            r#"invalid transition output 'x'. If this is intended as a native option, it must begin with //command_line_option: invalid label 'x': absolute label must begin with '@' or '//'"#,
        ),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['x'], outputs=[1]))"#,
        Err(r#"at index 0 of outputs, got element of type int, want string"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=[1], outputs=['x']))"#,
        Err(r#"at index 0 of inputs, got element of type int, want string"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['@foo//x:y'], outputs=[]))"#,
        Err(
            r#"invalid transition input '@@[unknown repo 'foo' requested from @@]//x:y': no repo visible as @foo from main repository"#,
        ),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['@@foo//x:y'], outputs=[]))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['//x:y//z'], outputs=[]))"#,
        Err(
            r#"invalid transition input '//x:y//z'. If this is intended as a native option, it must begin with //command_line_option: invalid target name 'y//z': target names may not contain '//' path separators"#,
        ),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['//x/:y'], outputs=[]))"#,
        Err(
            r#"invalid transition input '//x/:y'. If this is intended as a native option, it must begin with //command_line_option: invalid package name 'x/': package names may not end with '/'"#,
        ),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['//x:y:z'], outputs=[]))"#,
        Err(
            r#"invalid transition input '//x:y:z'. If this is intended as a native option, it must begin with //command_line_option: invalid target name 'y:z': target names may not contain ':'"#,
        ),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['//x...:y'], outputs=[]))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['@//x:y'], outputs=[]))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['//x'], outputs=[]))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['@foo'], outputs=[]))"#,
        Err(
            r#"invalid transition input '@@[unknown repo 'foo' requested from @@]//:foo': no repo visible as @foo from main repository"#,
        ),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['//command_line_option'], outputs=[]))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['//command_line_option:a:b'], outputs=[]))"#,
        Err(r#"Malformed label in transition INPUTS parameter: '//command_line_option:a:b'"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['//command_line_option:a b'], outputs=[]))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['//command_line_optionx:a'], outputs=[]))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['@@//command_line_option:a'], outputs=[]))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(transition(implementation=g, inputs=['@//command_line_option:a'], outputs=[]))"#,
        Ok(r#"<transition object>"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(analysis_test_transition(settings={'//command_line_option:': 1}))"#,
        Err(r#"Malformed label in transition OUTPUTS parameter: '//command_line_option:'"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(analysis_test_transition(settings={'//x:': 1}))"#,
        Err(
            r#"invalid transition output '//x:'. If this is intended as a native option, it must begin with //command_line_option: invalid target name '': empty target name"#,
        ),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(analysis_test_transition(settings={'//x:y': 1, '@@//x:y': 2}))"#,
        Err(
            r#"Transition declares duplicate build setting '//x:y' in OUTPUTS (specified as '@@//x:y' and '//x:y')"#,
        ),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(analysis_test_transition(settings={'//command_line_option:a': 1, '//command_line_option:b': 2}))"#,
        Ok(r#"<analysis_test_transition object>"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(analysis_test_transition(settings={'y': 1}))"#,
        Err(
            r#"invalid transition output 'y'. If this is intended as a native option, it must begin with //command_line_option: invalid label 'y': absolute label must begin with '@' or '//'"#,
        ),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(analysis_test_transition(settings={'': 1}))"#,
        Err(
            r#"invalid transition output ''. If this is intended as a native option, it must begin with //command_line_option: invalid target name '': empty target name"#,
        ),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(analysis_test_transition(settings={'//x:y': g}))"#,
        Ok(r#"<analysis_test_transition object>"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
print(analysis_test_transition(settings={'//x:y': analysis_test_transition(settings={})}))"#,
        Ok(r#"<analysis_test_transition object>"#),
    ),
    (
        r#"r=1
c=configuration_field('cpp','zipper')
print(c)
print(type(c))
print(c==c)
print(c==configuration_field('cpp','zipper'))
print(c==configuration_field('cpp','libc_top'))"#,
        Ok(r#"<late-bound default>
LateBoundDefault
True
True
False"#),
    ),
    (
        r#"r=1
c=configuration_field('cpp','zipper')
print({c:1})"#,
        Err(r#"unhashable type: 'LateBoundDefault'"#),
    ),
    (
        r#"r=1
c=configuration_field('cpp','zipper')
print(dir(c))
print(bool(c))
print(c.foo)"#,
        Err(r#"'LateBoundDefault' value has no field or method 'foo'"#),
    ),
    (
        r#"r=1
c=configuration_field('cpp','zipper')
print(hash(c))"#,
        Err(
            r#"in call to hash(), parameter 'value' got value of type 'LateBoundDefault', want 'string'"#,
        ),
    ),
    (
        r#"r=1
print(attr.label(default=configuration_field('cpp','zipper')))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"r=1
print(attr.label(default=configuration_field('cpp','zipper'), cfg='exec', executable=True))"#,
        Ok(r#"<attr.label>"#),
    ),
    (
        r#"r=1
print(attr.string(default=configuration_field('cpp','zipper')))"#,
        Err(
            r#"in call to string(), parameter 'default' got value of type 'LateBoundDefault', want 'string'"#,
        ),
    ),
    (
        r#"r=1
print(attr.label_list(default=[configuration_field('cpp','zipper')]))"#,
        Err(
            r#"expected value of type 'string' for element 0 of parameter 'default' of attribute 'label_list', but got <late-bound default> (LateBoundDefault)"#,
        ),
    ),
    (
        r#"r=1
a=attr.label(default=configuration_field('cpp','zipper'))
b=attr.label(default=configuration_field('cpp','zipper'))
print(a==b)"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
c=configuration_field('cpp','zipper')
a=attr.label(default=c)
b=attr.label(default=c)
print(a==b)"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
f=lambda: 1
print(aspect(f)==aspect(f))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
e=exec_group(exec_compatible_with=['//x:y'], toolchains=['//a:b', '//c:d'])
print(e==exec_group(exec_compatible_with=['//x:y'], toolchains=['//c:d','//a:b']))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
print(exec_group(exec_compatible_with=['//x:y','//z:w'])==exec_group(exec_compatible_with=['//z:w','//x:y']))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
print(exec_group(toolchains=['//a:b'])==exec_group(toolchains=['@@//a:b']))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
print(exec_group(toolchains=['//a:b'])==exec_group(toolchains=['@//a:b']))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
print(exec_group(toolchains=['//a:b'])==exec_group(toolchains=[':b']))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
print(exec_group(exec_compatible_with=['//a:b'])==exec_group(exec_compatible_with=[':b']))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
s=struct()
print(s)"#,
        Ok(r#"struct()"#),
    ),
    (
        r#"r=1
print(exec_group(toolchains=['//x:y'], exec_compatible_with=['//x:y'])==exec_group(toolchains=['//x:y'], exec_compatible_with=[]))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
print(exec_group(toolchains=[Label('//x:y')])==exec_group(toolchains=['//x:y']))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
print(exec_group(toolchains=['//a:b','//a:b'])==exec_group(toolchains=['//a:b']))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
print(attr.label(cfg=t)==attr.label(cfg=t))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
print(attr.label(cfg=t)==attr.label(cfg=u))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
print(attr.label(cfg=config.none())==attr.label(cfg=config.none()))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
print(attr.label(cfg=config.target())==attr.label(cfg='target'))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
print(attr.label(cfg=config.target())==attr.label())"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
print(attr.label(cfg=config.none())==attr.label(cfg=config.target()))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
print(attr.label(cfg=config.none())==attr.label())"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
a=analysis_test_transition(settings={})
print(attr.label(cfg=a)==attr.label(cfg=a))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
print(attr.label(cfg=analysis_test_transition(settings={}))==attr.label(cfg=analysis_test_transition(settings={})))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
print(attr.label(cfg=t.and_then(u))==attr.label(cfg=t.and_then(u)))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
c=t.and_then(u)
print(attr.label(cfg=c)==attr.label(cfg=c))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
e=config.exec()
print(attr.label(cfg=e)==attr.label(cfg=e))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
print(attr.label(cfg=config.target().and_then(t))==attr.label(cfg=config.target().and_then(t)))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
print(attr.label(cfg=t, executable=True)==attr.label(cfg=t, executable=True))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
print(attr.label_list(cfg=t)==attr.label_list(cfg=t))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
print(attr.label(cfg='exec')==attr.label(cfg='exec'))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
print(attr.label(cfg='host')==attr.label(cfg='host'))"#,
        Ok(r#"False"#),
    ),
    (
        r#"r=1
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
u=transition(implementation=g, inputs=[], outputs=[])
print(attr.label(cfg='target', executable=True)==attr.label(cfg='target', executable=True))"#,
        Ok(r#"True"#),
    ),
    (
        r#"r=1
print(configuration_field('apple','zz'))"#,
        Err(r#"invalid configuration field name 'zz' on fragment 'apple'"#),
    ),
    (
        r#"r=1
print(configuration_field('bazel_py','zz'))"#,
        Err(r#"invalid configuration field name 'zz' on fragment 'bazel_py'"#),
    ),
    (
        r#"r=1
print(configuration_field('coverage','zz'))"#,
        Err(r#"invalid configuration field name 'zz' on fragment 'coverage'"#),
    ),
    (
        r#"r=1
print(configuration_field('cpp','zz'))"#,
        Err(r#"invalid configuration field name 'zz' on fragment 'cpp'"#),
    ),
    (
        r#"r=1
print(configuration_field('j2objc','zz'))"#,
        Err(r#"invalid configuration field name 'zz' on fragment 'j2objc'"#),
    ),
    (
        r#"r=1
print(configuration_field('java','zz'))"#,
        Err(r#"invalid configuration field name 'zz' on fragment 'java'"#),
    ),
    (
        r#"r=1
print(configuration_field('objc','zz'))"#,
        Err(r#"invalid configuration field name 'zz' on fragment 'objc'"#),
    ),
    (
        r#"r=1
print(configuration_field('platform','zz'))"#,
        Err(r#"invalid configuration field name 'zz' on fragment 'platform'"#),
    ),
    (
        r#"r=1
print(configuration_field('proto','zz'))"#,
        Err(r#"invalid configuration field name 'zz' on fragment 'proto'"#),
    ),
    (
        r#"r=1
print(configuration_field('py','zz'))"#,
        Err(r#"invalid configuration field name 'zz' on fragment 'py'"#),
    ),
    (
        r#"r=1
print(configuration_field('android','zz'))"#,
        Err(r#"invalid configuration field name 'zz' on fragment 'android'"#),
    ),
    (
        r#"r=1
print(configuration_field('go','zz'))"#,
        Err(r#"invalid configuration fragment name 'go'"#),
    ),
    (
        r#"r=1
print(configuration_field('rust','zz'))"#,
        Err(r#"invalid configuration fragment name 'rust'"#),
    ),
    (
        r#"r=1
print(configuration_field('shell','zz'))"#,
        Err(r#"invalid configuration fragment name 'shell'"#),
    ),
    (
        r#"r=1
print(configuration_field('test','zz'))"#,
        Err(r#"invalid configuration fragment name 'test'"#),
    ),
    (
        r#"r=1
print(configuration_field('run','zz'))"#,
        Err(r#"invalid configuration fragment name 'run'"#),
    ),
    (
        r#"r=1
print(configuration_field('bazel_android','zz'))"#,
        Err(r#"invalid configuration field name 'zz' on fragment 'bazel_android'"#),
    ),
    (
        r#"r=1
print(configuration_field('bazel_java','zz'))"#,
        Err(r#"invalid configuration fragment name 'bazel_java'"#),
    ),
    (
        r#"r=1
print(configuration_field('core','zz'))"#,
        Err(r#"invalid configuration fragment name 'core'"#),
    ),
    (
        r#"r=1
print(configuration_field('build','zz'))"#,
        Err(r#"invalid configuration fragment name 'build'"#),
    ),
    (
        r#"r=1
print(configuration_field('sh','zz'))"#,
        Err(r#"invalid configuration fragment name 'sh'"#),
    ),
    (
        r#"r=1
print(configuration_field('dynamic_mode','zz'))"#,
        Err(r#"invalid configuration fragment name 'dynamic_mode'"#),
    ),
    (
        r#"r=1
print(configuration_field('bazel','zz'))"#,
        Err(r#"invalid configuration fragment name 'bazel'"#),
    ),
    (
        r#"r=1
print(configuration_field('jvm','zz'))"#,
        Err(r#"invalid configuration fragment name 'jvm'"#),
    ),
    (
        r#"r=1
print(configuration_field('xcode','zz'))"#,
        Err(r#"invalid configuration fragment name 'xcode'"#),
    ),
    (
        r#"r=1
print(configuration_field('cc','zz'))"#,
        Err(r#"invalid configuration fragment name 'cc'"#),
    ),
    (
        r#"r=1
print(configuration_field('config','zz'))"#,
        Err(r#"invalid configuration fragment name 'config'"#),
    ),
    (
        r#"r=1
print(configuration_field('genrule','zz'))"#,
        Err(r#"invalid configuration fragment name 'genrule'"#),
    ),
    (
        r#"r=1
print(configuration_field('google','zz'))"#,
        Err(r#"invalid configuration fragment name 'google'"#),
    ),
    (
        r#"r=1
print(configuration_field('bool','zz'))"#,
        Err(r#"invalid configuration fragment name 'bool'"#),
    ),
    (
        r#"r=1
print(configuration_field('python','zz'))"#,
        Err(r#"invalid configuration fragment name 'python'"#),
    ),
    (
        r#"r=1
print(configuration_field('windows','zz'))"#,
        Err(r#"invalid configuration fragment name 'windows'"#),
    ),
    (
        r#"r=1
print(configuration_field('fdo','zz'))"#,
        Err(r#"invalid configuration fragment name 'fdo'"#),
    ),
    (
        r#"r=1
print(configuration_field('lcov','zz'))"#,
        Err(r#"invalid configuration fragment name 'lcov'"#),
    ),
    (
        r#"r=1
print(configuration_field('jacoco','zz'))"#,
        Err(r#"invalid configuration fragment name 'jacoco'"#),
    ),
    (
        r#"r=1
print(configuration_field('shell_executable','zz'))"#,
        Err(r#"invalid configuration fragment name 'shell_executable'"#),
    ),
    (
        r#"r=1
print(configuration_field('default_shell_env','zz'))"#,
        Err(r#"invalid configuration fragment name 'default_shell_env'"#),
    ),
    (
        r#"r=1
print(configuration_field('stamp','zz'))"#,
        Err(r#"invalid configuration fragment name 'stamp'"#),
    ),
    (
        r#"r=1
print(configuration_field('options','zz'))"#,
        Err(r#"invalid configuration fragment name 'options'"#),
    ),
    (
        r#"r=1
print(configuration_field('apple','xcode_config_label'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('coverage','output_generator'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('cpp','zipper'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('cpp','libc_top'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('cpp','fdo_profile'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('cpp','fdo_prefetch_hints'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('cpp','memprof_profile'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('cpp','propeller_optimize'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('cpp','custom_malloc'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('cpp','cs_fdo_profile'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('java','bytecode_optimizer'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('java','local_java_optimization_configuration'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('java','launcher'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('proto','proto_compiler'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('proto','proto_toolchain_for_java'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('proto','proto_toolchain_for_cc'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('proto','proto_toolchain_for_java_lite'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('py','native_rules_allowlist'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('android','legacy_main_dex_list_generator'))"#,
        Ok(r#"<late-bound default>"#),
    ),
    (
        r#"r=1
print(configuration_field('cpp','java_toolchain_label'))"#,
        Err(r#"invalid configuration field name 'java_toolchain_label' on fragment 'cpp'"#),
    ),
    (
        r#"r=1
print(configuration_field('py','stl'))"#,
        Err(r#"invalid configuration field name 'stl' on fragment 'py'"#),
    ),
    (
        r#"r=1
print(configuration_field('proto','cc_test_toolchain'))"#,
        Err(r#"invalid configuration field name 'cc_test_toolchain' on fragment 'proto'"#),
    ),
    (
        r#"r=1
print(configuration_field('bazel_py','fake_default_crosstool'))"#,
        Err(r#"invalid configuration field name 'fake_default_crosstool' on fragment 'bazel_py'"#),
    ),
    (
        r#"r=1
print(configuration_field('java','test_filter'))"#,
        Err(r#"invalid configuration field name 'test_filter' on fragment 'java'"#),
    ),
    (
        r#"r=1
print(configuration_field('py','aapt'))"#,
        Err(r#"invalid configuration field name 'aapt' on fragment 'py'"#),
    ),
    (
        r#"r=1
print(configuration_field('platform','dex_splitter'))"#,
        Err(r#"invalid configuration field name 'dex_splitter' on fragment 'platform'"#),
    ),
    (
        r#"r=1
print(configuration_field('py','thinlto'))"#,
        Err(r#"invalid configuration field name 'thinlto' on fragment 'py'"#),
    ),
    (
        r#"r=1
print(configuration_field('proto','bootclasspath'))"#,
        Err(r#"invalid configuration field name 'bootclasspath' on fragment 'proto'"#),
    ),
    (
        r#"r=1
print(configuration_field('bazel_py','xcode_config_label'))"#,
        Err(r#"invalid configuration field name 'xcode_config_label' on fragment 'bazel_py'"#),
    ),
    (
        r#"r=1
print(configuration_field('py','j2objc_library_migration'))"#,
        Err(r#"invalid configuration field name 'j2objc_library_migration' on fragment 'py'"#),
    ),
    (
        r#"r=1
print(configuration_field('apple','jacocorunner'))"#,
        Err(r#"invalid configuration field name 'jacocorunner' on fragment 'apple'"#),
    ),
    (
        r#"r=1
print(configuration_field('platform','proto_compiler'))"#,
        Err(r#"invalid configuration field name 'proto_compiler' on fragment 'platform'"#),
    ),
    (
        r#"r=1
print(configuration_field('j2objc','xcode_config'))"#,
        Err(r#"invalid configuration field name 'xcode_config' on fragment 'j2objc'"#),
    ),
    (
        r#"r=1
print(configuration_field('proto','runtime'))"#,
        Err(r#"invalid configuration field name 'runtime' on fragment 'proto'"#),
    ),
    (
        r#"r=1
print(configuration_field('cpp','thinlto'))"#,
        Err(r#"invalid configuration field name 'thinlto' on fragment 'cpp'"#),
    ),
    (
        r#"r=1
print(configuration_field('coverage','idlclass'))"#,
        Err(r#"invalid configuration field name 'idlclass' on fragment 'coverage'"#),
    ),
    (
        r#"r=1
print(configuration_field('bazel_android','output_generator'))"#,
        Err(r#"invalid configuration field name 'output_generator' on fragment 'bazel_android'"#),
    ),
    (
        r#"r=1
print(configuration_field('platform','proto_toolchain_for_cc'))"#,
        Err(r#"invalid configuration field name 'proto_toolchain_for_cc' on fragment 'platform'"#),
    ),
    (
        r#"r=1
print(configuration_field('proto','j2objc_library_migration_label'))"#,
        Err(
            r#"invalid configuration field name 'j2objc_library_migration_label' on fragment 'proto'"#,
        ),
    ),
    (
        r#"r=1
print(configuration_field('proto','default_python_version'))"#,
        Err(r#"invalid configuration field name 'default_python_version' on fragment 'proto'"#),
    ),
    (
        r#"r=1
print(configuration_field('platform','j2objc_library_migration'))"#,
        Err(
            r#"invalid configuration field name 'j2objc_library_migration' on fragment 'platform'"#,
        ),
    ),
    (
        r#"r=1
print(configuration_field('objc','target_libc_top'))"#,
        Err(r#"invalid configuration field name 'target_libc_top' on fragment 'objc'"#),
    ),
    (
        r#"r=1
print(configuration_field('py','turbine'))"#,
        Err(r#"invalid configuration field name 'turbine' on fragment 'py'"#),
    ),
    (
        r#"r=1
print(configuration_field('coverage','aapt2'))"#,
        Err(r#"invalid configuration field name 'aapt2' on fragment 'coverage'"#),
    ),
    (
        r#"r=1
print(configuration_field('cpp','objdump'))"#,
        Err(r#"invalid configuration field name 'objdump' on fragment 'cpp'"#),
    ),
    (
        r#"r=1
print(configuration_field('py','jre'))"#,
        Err(r#"invalid configuration field name 'jre' on fragment 'py'"#),
    ),
    (
        r#"r=1
print(configuration_field('coverage','dx'))"#,
        Err(r#"invalid configuration field name 'dx' on fragment 'coverage'"#),
    ),
    (
        r#"r=1
print(configuration_field('proto','lcov_merger'))"#,
        Err(r#"invalid configuration field name 'lcov_merger' on fragment 'proto'"#),
    ),
    (
        r#"r=1
print(configuration_field('objc','xcode_config'))"#,
        Err(r#"invalid configuration field name 'xcode_config' on fragment 'objc'"#),
    ),
    (
        r#"r=1
print(configuration_field('bazel_android','test_runner'))"#,
        Err(r#"invalid configuration field name 'test_runner' on fragment 'bazel_android'"#),
    ),
    (
        r#"r=1
print(configuration_field('apple','proto_toolchain_for_java_lite'))"#,
        Err(
            r#"invalid configuration field name 'proto_toolchain_for_java_lite' on fragment 'apple'"#,
        ),
    ),
    (
        r#"r=1
print(configuration_field('android','j2objc_library_migration_label'))"#,
        Err(
            r#"invalid configuration field name 'j2objc_library_migration_label' on fragment 'android'"#,
        ),
    ),
    (
        r#"r=1
print(configuration_field('bazel_android','jar_filter'))"#,
        Err(r#"invalid configuration field name 'jar_filter' on fragment 'bazel_android'"#),
    ),
    (
        r#"r=1
print(configuration_field('apple','cc_binary'))"#,
        Err(r#"invalid configuration field name 'cc_binary' on fragment 'apple'"#),
    ),
    (
        r#"r=1
print(configuration_field('coverage','target_platform'))"#,
        Err(r#"invalid configuration field name 'target_platform' on fragment 'coverage'"#),
    ),
    (
        r#"r=1
print(configuration_field('bazel_android','stamp'))"#,
        Err(r#"invalid configuration field name 'stamp' on fragment 'bazel_android'"#),
    ),
    (
        r#"r=1
print(configuration_field('py','propeller_optimize'))"#,
        Err(r#"invalid configuration field name 'propeller_optimize' on fragment 'py'"#),
    ),
    (
        r#"r=1
print(configuration_field('apple','cpu'))"#,
        Err(r#"invalid configuration field name 'cpu' on fragment 'apple'"#),
    ),
    (
        r#"r=1
print(configuration_field('j2objc','stamp'))"#,
        Err(r#"invalid configuration field name 'stamp' on fragment 'j2objc'"#),
    ),
    (
        r#"r=1
print(configuration_field('bazel_android','cc_library'))"#,
        Err(r#"invalid configuration field name 'cc_library' on fragment 'bazel_android'"#),
    ),
    (
        r#"r=1
print(configuration_field('apple','launcher'))"#,
        Err(r#"invalid configuration field name 'launcher' on fragment 'apple'"#),
    ),
    (
        r#"r=1
print(configuration_field('j2objc','java_launcher'))"#,
        Err(r#"invalid configuration field name 'java_launcher' on fragment 'j2objc'"#),
    ),
    (
        r#"r=1
print(configuration_field('platform','legacy_main_dex_list_generator'))"#,
        Err(
            r#"invalid configuration field name 'legacy_main_dex_list_generator' on fragment 'platform'"#,
        ),
    ),
    (
        r#"r=1
print(configuration_field('py','host_java_launcher'))"#,
        Err(r#"invalid configuration field name 'host_java_launcher' on fragment 'py'"#),
    ),
    (
        r#"r=1
print(configuration_field('bazel_android','cc_toolchain'))"#,
        Err(r#"invalid configuration field name 'cc_toolchain' on fragment 'bazel_android'"#),
    ),
    (
        r#"r=1
print(configuration_field('java','cc_library'))"#,
        Err(r#"invalid configuration field name 'cc_library' on fragment 'java'"#),
    ),
    (
        r#"r=1
print(configuration_field('android','cc_library'))"#,
        Err(r#"invalid configuration field name 'cc_library' on fragment 'android'"#),
    ),
    (
        r#"r=1
print(configuration_field('objc','ar'))"#,
        Err(r#"invalid configuration field name 'ar' on fragment 'objc'"#),
    ),
    (
        r#"r=1
print(configuration_field('objc','memprof_profile'))"#,
        Err(r#"invalid configuration field name 'memprof_profile' on fragment 'objc'"#),
    ),
];

pub(crate) const DECL_BUILD_CASES: &[BuildRow] = &[
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return aspect(f)
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"aspect() can only be used during .bzl initialization (top-level evaluation)"#,
        ),
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return transition(implementation=g, inputs=[], outputs=[])
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return exec_group()
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return configuration_field('cpp','zipper')
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"configuration_field() can only be used during .bzl initialization (top-level evaluation)"#,
        ),
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return subrule(implementation=f)
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return analysis_test_transition(settings={})
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return config.exec()
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return config.target()
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return config.none()
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return config.bool()
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return config.string()
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return config.int()
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return config.string_list()
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return config.string_set()
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return attr.string()
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return provider()
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return rule(implementation=f)
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"rule() can only be used during .bzl initialization (top-level evaluation)"#),
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return struct()
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return Label('//x:y')
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return select({'a':1})
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return depset([])
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return set()
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return json.encode(1)
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(t,c): return []
def g(s,a): return {}
t=transition(implementation=g, inputs=[], outputs=[])
def r():
  return t.and_then(t)
"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool())"#,
        build: r#"r(name="a", build_setting_default=True)
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "build_setting_default": True, "help": ""}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool(flag=True))"#,
        build: r#"r(name="a", build_setting_default=True)
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "build_setting_default": True, "help": ""}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.int())"#,
        build: r#"r(name="a", build_setting_default=1)
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "build_setting_default": 1, "help": ""}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.int(flag=True))"#,
        build: r#"r(name="a", build_setting_default=1)
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "build_setting_default": 1, "help": ""}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string())"#,
        build: r#"r(name="a", build_setting_default="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "build_setting_default": "a", "help": ""}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string(flag=True))"#,
        build: r#"r(name="a", build_setting_default="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "build_setting_default": "a", "help": ""}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string(allow_multiple=True))"#,
        build: r#"r(name="a", build_setting_default="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "build_setting_default": "a", "help": ""}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string(flag=True, allow_multiple=True))"#,
        build: r#"r(name="a", build_setting_default="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "build_setting_default": "a", "help": ""}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string_list())"#,
        build: r#"r(name="a", build_setting_default=[])
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "build_setting_default": (), "help": ""}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string_list(flag=True))"#,
        build: r#"r(name="a", build_setting_default=[])
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "build_setting_default": (), "help": ""}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string_list(flag=True, repeatable=True))"#,
        build: r#"r(name="a", build_setting_default=[])
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "build_setting_default": (), "help": ""}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string_set(flag=True))"#,
        build: r#"r(name="a", build_setting_default=[])
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "help": ""}"#,
        ],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'set(string)' for attribute 'build_setting_default' of 'r', but got [] (list)"#,
            r#"BUILD.bazel:2:2: //:a: missing value for mandatory attribute 'build_setting_default' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool())"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: missing value for mandatory attribute 'build_setting_default' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool())"#,
        build: r#"r(name="a", build_setting_default=1)"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool())"#,
        build: r#"r(name="a", build_setting_default="x")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected one of [False, True, 0, 1] for attribute 'build_setting_default' of 'r', but got "x" (string)"#,
            r#"BUILD.bazel:2:2: //:a: missing value for mandatory attribute 'build_setting_default' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string())"#,
        build: r#"r(name="a", build_setting_default=1)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'build_setting_default' of 'r', but got 1 (int)"#,
            r#"BUILD.bazel:2:2: //:a: missing value for mandatory attribute 'build_setting_default' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.int())"#,
        build: r#"r(name="a", build_setting_default="x")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'int' for attribute 'build_setting_default' of 'r', but got "x" (string)"#,
            r#"BUILD.bazel:2:2: //:a: missing value for mandatory attribute 'build_setting_default' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string_list())"#,
        build: r#"r(name="a", build_setting_default="x")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'list(string)' for attribute 'build_setting_default' of 'r', but got "x" (string)"#,
            r#"BUILD.bazel:2:2: //:a: missing value for mandatory attribute 'build_setting_default' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string_list())"#,
        build: r#"r(name="a", build_setting_default=["x"])
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "build_setting_default": ("x",), "help": ""}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool(), attrs={'build_setting_default':attr.bool()})"#,
        build: r#"r(name="a", build_setting_default=True)"#,
        printed: &[],
        events: &[
            r#"u.bzl:2:7: There is already a built-in attribute 'build_setting_default' which cannot be overridden."#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool(), attrs={'scope':attr.string()})"#,
        build: r#"r(name="a", build_setting_default=True)"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool(), attrs={'x':attr.string()})"#,
        build: r#"r(name="a", build_setting_default=True, x="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool())"#,
        build: r#"r(name="a", build_setting_default=True, scope="universal")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "build_setting_default": True, "help": ""}"#,
        ],
        events: &[r#"BUILD.bazel:2:2: //:a: no such attribute 'scope' in 'r' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool(flag=True))"#,
        build: r#"r(name="a", build_setting_default=True, scope="universal")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": (), "build_setting_default": True, "help": ""}"#,
        ],
        events: &[r#"BUILD.bazel:2:2: //:a: no such attribute 'scope' in 'r' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool(flag=True))"#,
        build: r#"r(name="a", build_setting_default=True)
r(name="b", build_setting_default=True)
print(existing_rules().keys())"#,
        printed: &[r#"["a", "b"]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool(), executable=True)"#,
        build: r#"r(name="a", build_setting_default=True)"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool(), test=True)"#,
        build: r#"r(name="a", build_setting_default=True)"#,
        printed: &[],
        events: &[
            r#"u.bzl:2:7: Invalid rule class name 'r', test rule class names must end with '_test' and other rule classes must not"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool(), cfg=transition(implementation=lambda s,a:{}, inputs=[], outputs=[]))"#,
        build: r#"r(name="a", build_setting_default=True)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"Build setting rules cannot use the `cfg` param to apply transitions to themselves."#,
        ),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool(), outputs={'o':'%{name}.o'})"#,
        build: r#"r(name="a", build_setting_default=True)"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool(), analysis_test=True)"#,
        build: r#"r(name="a", build_setting_default=True)"#,
        printed: &[],
        events: &[
            r#"u.bzl:2:7: Invalid rule class name 'r', test rule class names must end with '_test' and other rule classes must not"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=1)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to rule(), parameter 'build_setting' got value of type 'int', want 'BuildSetting or NoneType'"#,
        ),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting='x')"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to rule(), parameter 'build_setting' got value of type 'string', want 'BuildSetting or NoneType'"#,
        ),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.exec())"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to rule(), parameter 'build_setting' got value of type 'ExecTransitionFactory', want 'BuildSetting or NoneType'"#,
        ),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.target())"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to rule(), parameter 'build_setting' got value of type 'transition', want 'BuildSetting or NoneType'"#,
        ),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to rule(), parameter 'build_setting' got value of type 'builtin_function_or_method', want 'BuildSetting or NoneType'"#,
        ),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, cfg=transition(implementation=lambda s,a:{}, inputs=[], outputs=[]))"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, cfg=config.target())"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, cfg=config.none())"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, cfg=config.exec())"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, cfg=analysis_test_transition(settings={}))"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, cfg='target')"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, cfg='exec')"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, cfg=None)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, cfg=1)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, cfg=transition(implementation=lambda s,a:{}, inputs=[], outputs=[]).and_then(transition(implementation=lambda s,a:{}, inputs=[], outputs=[])))"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, cfg=config.exec('x'))"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"`cfg` must be set to a transition object initialized by the transition() function."#,
        ),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, exec_groups={})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": ()}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, exec_groups={'a':exec_group()})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": ()}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, exec_groups={'a':exec_group(toolchains=['//x:y'])})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": ()}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, exec_groups={'a':1})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"got dict<string, int> for 'exec_group', want dict<string, exec_group>"#),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, exec_groups={1:exec_group()})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"got dict<int, exec_group> for 'exec_group', want dict<string, exec_group>"#),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, exec_groups={'':exec_group()})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"Exec group name '' is not a valid name."#),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, exec_groups={'x y':exec_group()})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"Exec group name 'x y' is not a valid name."#),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, exec_groups={'a':exec_group(), 'a b':exec_group()})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"Exec group name 'a b' is not a valid name."#),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, exec_groups={'_a':exec_group()})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": ()}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, exec_groups={'a':config.exec()})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"got dict<string, ExecTransitionFactory> for 'exec_group', want dict<string, exec_group>"#,
        ),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, exec_groups=None)"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[
            r#"{"name": "a", "kind": "r", "expect_failure": "", "visibility": (), "transitive_configs": (), "tags": (), "generator_name": "", "generator_function": "", "generator_location": "", "features": (), "compatible_with": (), "restricted_to": (), "aspect_hints": (), "toolchains": (), "exec_properties": {}, "exec_compatible_with": (), "exec_group_compatible_with": {}, "target_compatible_with": ()}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, exec_groups={'a':None})"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"got dict<string, NoneType> for 'exec_group', want dict<string, exec_group>"#,
        ),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, subrules=[])"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, subrules=[subrule(implementation=f)])"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[r#"u.bzl:2:7: Invalid subrule hasn't been exported by a bzl file"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, subrules=[1])"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"at index 0 of subrules, got element of type int, want Subrule"#),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, subrules=[None])"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"at index 0 of subrules, got element of type NoneType, want Subrule"#),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, subrules=(subrule(implementation=f),))"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[r#"u.bzl:2:7: Invalid subrule hasn't been exported by a bzl file"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, subrules=[subrule(implementation=f), subrule(implementation=f)])"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[r#"u.bzl:2:7: Invalid subrule hasn't been exported by a bzl file"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, analysis_test=True)"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[],
        events: &[
            r#"u.bzl:2:7: Invalid rule class name 'r', test rule class names must end with '_test' and other rule classes must not"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, analysis_test=True, cfg=analysis_test_transition(settings={}))"#,
        build: r#"r(name="a")
print(dict(existing_rule("a")))"#,
        printed: &[],
        events: &[
            r#"u.bzl:2:7: Invalid rule class name 'r', test rule class names must end with '_test' and other rule classes must not"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, test=True, analysis_test=True)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"u.bzl:2:7: Invalid rule class name 'r', test rule class names must end with '_test' and other rule classes must not"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, analysis_test=False)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool(), attrs={'help':attr.string()})"#,
        build: r#"r(name="a", build_setting_default=True)"#,
        printed: &[],
        events: &[
            r#"u.bzl:2:7: There is already a built-in attribute 'help' which cannot be overridden."#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool(), attrs={'z':attr.string()})"#,
        build: r#"r(name="a", build_setting_default=True, z="q")
print(dict(existing_rule("a")).keys())"#,
        printed: &[
            r#"["name", "kind", "expect_failure", "visibility", "transitive_configs", "tags", "generator_name", "generator_function", "generator_location", "features", "compatible_with", "restricted_to", "aspect_hints", "toolchains", "exec_properties", "exec_compatible_with", "exec_group_compatible_with", "target_compatible_with", "z", "build_setting_default", "help"]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, attrs={'build_setting_default':attr.string()})"#,
        build: r#"r(name="a", build_setting_default="x")
print(dict(existing_rule("a")).keys())"#,
        printed: &[
            r#"["name", "kind", "expect_failure", "visibility", "transitive_configs", "tags", "generator_name", "generator_function", "generator_location", "features", "compatible_with", "restricted_to", "aspect_hints", "toolchains", "exec_properties", "exec_compatible_with", "exec_group_compatible_with", "target_compatible_with", "build_setting_default"]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, attrs={'help':attr.string()})"#,
        build: r#"r(name="a", help="x")
print(dict(existing_rule("a")).keys())"#,
        printed: &[
            r#"["name", "kind", "expect_failure", "visibility", "transitive_configs", "tags", "generator_name", "generator_function", "generator_location", "features", "compatible_with", "restricted_to", "aspect_hints", "toolchains", "exec_properties", "exec_compatible_with", "exec_group_compatible_with", "target_compatible_with", "help"]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string(), attrs={'z':attr.string()})"#,
        build: r#"r(name="a", build_setting_default="x")
print(existing_rule("a")["z"])"#,
        printed: &[r#""#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string_set(flag=True))"#,
        build: r#"r(name="a", build_setting_default=set(["x"]))
print(dict(existing_rule("a")).keys())"#,
        printed: &[
            r#"["name", "kind", "expect_failure", "visibility", "transitive_configs", "tags", "generator_name", "generator_function", "generator_location", "features", "compatible_with", "restricted_to", "aspect_hints", "toolchains", "exec_properties", "exec_compatible_with", "exec_group_compatible_with", "target_compatible_with", "help"]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string_set(flag=True))"#,
        build: r#"r(name="a", build_setting_default=set())
print(existing_rule("a")["build_setting_default"])"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"key "build_setting_default" not found in view"#),
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string_set(flag=True))"#,
        build: r#"r(name="a", build_setting_default=["x"])"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'set(string)' for attribute 'build_setting_default' of 'r', but got ["x"] (list)"#,
            r#"BUILD.bazel:2:2: //:a: missing value for mandatory attribute 'build_setting_default' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string_set(flag=True))"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: missing value for mandatory attribute 'build_setting_default' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool())"#,
        build: r#"r(name="a", build_setting_default=True, help=1)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: expected value of type 'string' for attribute 'help' of 'r', but got 1 (int)"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool())"#,
        build: r#"r(name="a", build_setting_default=True, help="text")
print(existing_rule("a")["help"])"#,
        printed: &[r#"text"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool(flag=True))"#,
        build: r#"r(name="a", build_setting_default=True, help="text")
print(existing_rule("a")["help"])"#,
        printed: &[r#"text"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool())"#,
        build: r#"r(name="a", build_setting_default=None)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: missing value for mandatory attribute 'build_setting_default' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string())"#,
        build: r#"r(name="a", build_setting_default=None)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: //:a: missing value for mandatory attribute 'build_setting_default' in 'r' rule"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.bool())"#,
        build: r#"r(name="a", build_setting_default=False)
print(existing_rule("a")["build_setting_default"])"#,
        printed: &[r#"False"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def f(ctx): return []
r=rule(implementation=f, build_setting=config.string(), attrs={'a1':attr.string(), 'a2':attr.int()})"#,
        build: r#"r(name="a", build_setting_default="x")
print(dict(existing_rule("a")).keys())"#,
        printed: &[
            r#"["name", "kind", "expect_failure", "visibility", "transitive_configs", "tags", "generator_name", "generator_function", "generator_location", "features", "compatible_with", "restricted_to", "aspect_hints", "toolchains", "exec_properties", "exec_compatible_with", "exec_group_compatible_with", "target_compatible_with", "a1", "a2", "build_setting_default", "help"]"#,
        ],
        events: &[],
        fatal: None,
    },
];
