//! Generated from Bazel 9.2.0 probes: what macro() prints and refuses.

use crate::test_support::BuildRow;

pub(crate) const MACRO_CASES: &[(&str, Result<&str, &str>)] = &[
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=1)
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'implementation' got value of type 'int', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation="s")
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'implementation' got value of type 'string', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=[])
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'implementation' got value of type 'list', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation={})
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'implementation' got value of type 'dict', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=None)
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'implementation' got value of type 'NoneType', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=True)
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'implementation' got value of type 'bool', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=lambda name: None)
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f)
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=struct())
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'implementation' got value of type 'struct', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=R)
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'implementation' got value of type 'rule', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=macro(implementation=f))
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'implementation' got value of type 'macro', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=len)
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'implementation' got value of type 'builtin_function_or_method', want 'function'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs=1)
print(m)"#,
        Err(r#"in call to macro(), parameter 'attrs' got value of type 'int', want 'dict'"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs="s")
print(m)"#,
        Err(r#"in call to macro(), parameter 'attrs' got value of type 'string', want 'dict'"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs=[])
print(m)"#,
        Err(r#"in call to macro(), parameter 'attrs' got value of type 'list', want 'dict'"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs=None)
print(m)"#,
        Err(r#"in call to macro(), parameter 'attrs' got value of type 'NoneType', want 'dict'"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs=struct())
print(m)"#,
        Err(r#"in call to macro(), parameter 'attrs' got value of type 'struct', want 'dict'"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, inherit_attrs=1)
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'inherit_attrs' got value of type 'int', want 'rule, macro, string, or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, inherit_attrs="s")
print(m)"#,
        Err(r#"Invalid 'inherit_attrs' value "s"; expected a rule, a macro, or "common""#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, inherit_attrs=[])
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'inherit_attrs' got value of type 'list', want 'rule, macro, string, or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, inherit_attrs={})
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'inherit_attrs' got value of type 'dict', want 'rule, macro, string, or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, inherit_attrs=None)
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, inherit_attrs=True)
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'inherit_attrs' got value of type 'bool', want 'rule, macro, string, or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, inherit_attrs=struct())
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'inherit_attrs' got value of type 'struct', want 'rule, macro, string, or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, inherit_attrs=R)
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, inherit_attrs=macro(implementation=f))
print(m)"#,
        Err(
            r#"Invalid 'inherit_attrs' value: a rule or macro callable must be assigned to a global variable in a .bzl file before it can be inherited from"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, inherit_attrs="common")
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, inherit_attrs="other")
print(m)"#,
        Err(r#"Invalid 'inherit_attrs' value "other"; expected a rule, a macro, or "common""#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, inherit_attrs="")
print(m)"#,
        Err(r#"Invalid 'inherit_attrs' value ""; expected a rule, a macro, or "common""#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, inherit_attrs=f)
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'inherit_attrs' got value of type 'function', want 'rule, macro, string, or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, inherit_attrs=aspect(f))
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'inherit_attrs' got value of type 'Aspect', want 'rule, macro, string, or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, finalizer=1)
print(m)"#,
        Err(r#"in call to macro(), parameter 'finalizer' got value of type 'int', want 'bool'"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, finalizer="s")
print(m)"#,
        Err(r#"in call to macro(), parameter 'finalizer' got value of type 'string', want 'bool'"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, finalizer=[])
print(m)"#,
        Err(r#"in call to macro(), parameter 'finalizer' got value of type 'list', want 'bool'"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, finalizer=None)
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'finalizer' got value of type 'NoneType', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, finalizer=True)
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, finalizer=False)
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, finalizer=f)
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'finalizer' got value of type 'function', want 'bool'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, doc=1)
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'doc' got value of type 'int', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, doc="s")
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, doc=None)
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, doc=[])
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'doc' got value of type 'list', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, doc=True)
print(m)"#,
        Err(
            r#"in call to macro(), parameter 'doc' got value of type 'bool', want 'string or NoneType'"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.int()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.bool()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.label()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.label_list()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.string_list()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.string_dict()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.string_list_dict()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.label_keyed_string_dict()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.int_list()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.output()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.output_list()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.label_list(allow_files=True)})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.label(executable=True, cfg='exec')})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.label(cfg='exec')})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.string(configurable=False)})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.label(providers=[P])})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.label(aspects=[aspect(f)])})
print(m)"#,
        Err(r#"Aspects should be top-level values in extension files that define them."#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.string(mandatory=True)})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.string(default='a', values=['a','b'])})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.label(default='//x:y')})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.label(allow_single_file=True)})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.int(default=3)})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.bool(default=True)})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.string_list(allow_empty=False)})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.label_list(allow_empty=False)})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.string_dict(allow_empty=False)})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.string(doc='x')})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.label_list(cfg=config.target())})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.label(cfg=transition(implementation=lambda s,a:{}, inputs=[], outputs=[]))})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.label(default=lambda: '//x:y')})
print(m)"#,
        Err(
            r#"In macro attribute 'x': Macros do not support computed defaults or late-bound defaults"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.label(default=configuration_field('cpp','zipper'))})
print(m)"#,
        Err(
            r#"In macro attribute 'x': Macros do not support computed defaults or late-bound defaults"#,
        ),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'name': attr.string()})
print(m)"#,
        Err(r#"Cannot declare a macro attribute named 'name'"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'visibility': attr.string()})
print(m)"#,
        Err(r#"Cannot declare a macro attribute named 'visibility'"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'_x': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x y': attr.string()})
print(m)"#,
        Err(r#"attribute name `x y` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'1x': attr.string()})
print(m)"#,
        Err(r#"attribute name `1x` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'': attr.string()})
print(m)"#,
        Err(r#"attribute name `` is not a valid identifier."#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'tags': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'testonly': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'features': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'compatible_with': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'deprecation': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'exec_compatible_with': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'target_compatible_with': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'generator_name': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'aspect_hints': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'package_metadata': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'applicable_licenses': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'restricted_to': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'toolchains': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'exec_properties': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'transitive_configs': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'expect_failure': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'kind': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'self': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'ctx': attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': None})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': 1})
print(m)"#,
        Err(r#"got dict<string, int> for 'attrs', want dict<string, Attribute|None>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={1: attr.string()})
print(m)"#,
        Err(r#"got dict<int, Attribute> for 'attrs', want dict<string, Attribute|None>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x': attr.string(), 'x': attr.int()})
print(m)"#,
        Err(r#"dictionary expression has duplicate key: "x""#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=g, attrs={'x':attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=g)
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=h, attrs={'x':attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=h)
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=k, attrs={'x':attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=k)
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f, attrs={'x':attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=lambda: 1, attrs={'x':attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=lambda: 1)
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=lambda name: 1, attrs={'x':attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=lambda name: 1)
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=lambda name, **kw: 1, attrs={'x':attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=lambda name, **kw: 1)
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=lambda *a: 1, attrs={'x':attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=lambda *a: 1)
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=lambda **kw: 1, attrs={'x':attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=lambda **kw: 1)
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=lambda name, x=1: 1, attrs={'x':attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=lambda name, x=1: 1)
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=lambda x: 1, attrs={'x':attr.string()})
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=lambda x: 1)
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
M=macro(implementation=f)
print(M)"#,
        Ok(r#"<macro M>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
_M=macro(implementation=f)
print(_M)"#,
        Ok(r#"<macro _M>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
x=[macro(implementation=f)]
print(x)"#,
        Ok(r#"[<macro>]"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f)
n=m
print(n)"#,
        Ok(r#"<macro m>"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f)
print(m==m)
print(m==macro(implementation=f))
print({m:1})
print(dir(m))
print(bool(m))
print(m.foo)"#,
        Err(r#"'macro' value has no field or method 'foo'"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f)
print(m.name)"#,
        Err(r#"'macro' value has no field or method 'name'"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
m=macro(implementation=f)
print(hash(m))"#,
        Err(r#"in call to hash(), parameter 'value' got value of type 'macro', want 'string'"#),
    ),
    (
        r#"r=1
def f(name, **kw): pass
def g(name, x): pass
def h(name): pass
def k(): pass
P=provider()
R=rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), '_z':attr.string(default='d')})
def mk():
  return macro(implementation=f)
m=mk()
print(m)"#,
        Ok(r#"<macro m>"#),
    ),
];

pub(crate) const MACRO_BUILD_CASES: &[BuildRow] = &[
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
r(name="b")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"unexpected positional arguments"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r()"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"missing value for mandatory attribute 'name' in 'r' macro"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name=1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'string' for attribute 'name' of 'r', but got 1 (int)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"invalid target name '': empty target name"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="x y")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a/b")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a", foo=1)"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"no such attribute 'foo' in 'r' macro"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//visibility:public"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility="//visibility:public")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'list(label)' for attribute 'visibility' of 'r', but got "//visibility:public" (string)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'list(label)' for attribute 'visibility' of 'r', but got 1 (int)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=None)"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=[":x"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//x:y"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//visibility:private"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//visibility:nope"])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"Invalid visibility label '//visibility:nope'; did you mean //visibility:public or //visibility:private?"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"print(r)
print(type(r))"#,
        printed: &[r#"<macro r>"#, r#"macro"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"print(r(name="a"))"#,
        printed: &[r#"None"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"x=r(name="a")
print(x)"#,
        printed: &[r#"None"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name)
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_x')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'x')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'.x')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'-x')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name='other')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name[:-1])
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"illegal rule name: : invalid target name '': empty target name"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name='x'+name)
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'/x')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name.upper())
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name='')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"illegal rule name: : invalid target name '': empty target name"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+' ')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name)
  native.filegroup(name=name)
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"filegroup rule 'a' conflicts with existing filegroup rule, defined at BUILD.bazel:2:2"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_x')
  native.filegroup(name=name+'_x')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"filegroup rule 'a_x' conflicts with existing filegroup rule, defined at BUILD.bazel:2:2"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"filegroup(name="a_g")
r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"filegroup rule 'a_g' conflicts with existing filegroup rule, defined at BUILD.bazel:2:10"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
filegroup(name="a_g")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"filegroup rule 'a_g' conflicts with existing filegroup rule, defined at BUILD.bazel:2:2"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"macro 'a' conflicts with an existing macro (and was not created by it)"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a_g")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
r(name="a_g")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"macro 'a_g' conflicts with an existing target."#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="f1.txt")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name, srcs=['f1.txt'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
print(existing_rules().keys())"#,
        printed: &[r#"["a"]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"x_rule = rule(implementation=lambda ctx: [])
def _i(name, **kw):
  x_rule(name=name)
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"x_rule = rule(implementation=lambda ctx: [])
def _i(name, **kw):
  x_rule(name=name+'_x')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  fail('boom')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"boom"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  return 1
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"macro 'a' may not return a non-None value (got 1)"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  return None
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  return []
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"macro 'a' may not return a non-None value (got [])"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('hi', name)
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[r#"hi a"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(kw)
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[r#"{"visibility": [Label("//:__pkg__")]}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, visibility, **kw):
  print(visibility)
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[r#"[Label("//:__pkg__")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, visibility, **kw):
  print(visibility)
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//visibility:public"])"#,
        printed: &[r#"[Label("//visibility:public")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, visibility, **kw):
  print(visibility)
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//foo:bar", ":x"])"#,
        printed: &[r#"[Label("//:__pkg__"), Label("//:x"), Label("//foo:bar")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(name, type(name))
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[r#"a string"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name):
  pass
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"_i() got unexpected keyword argument: visibility"#),
    },
    BuildRow {
        bzl: r#"def _i(name, visibility):
  pass
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i():
  pass
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"_i() got unexpected keyword arguments: name, visibility"#),
    },
    BuildRow {
        bzl: r#"def _i(*, name):
  pass
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"_i() got unexpected keyword argument: visibility"#),
    },
    BuildRow {
        bzl: r#"def _i(n, **kw):
  pass
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"_i() missing 1 required positional argument: n"#),
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  pass
r = macro(implementation=_i, attrs={'x':attr.string()})"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string()})"#,
        build: r#"r(name="a", x="v")"#,
        printed: &[r#"select({"//conditions:default": "v"})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string(default='d')})"#,
        build: r#"r(name="a")"#,
        printed: &[r#"select({"//conditions:default": "d"})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string(mandatory=True)})"#,
        build: r#"r(name="a")"#,
        printed: &[r#"select({"//conditions:default": ""})"#],
        events: &[r#"//:a: missing value for mandatory attribute 'x' in 'r' macro"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string(mandatory=True)})"#,
        build: r#"r(name="a", x=None)"#,
        printed: &[r#"select({"//conditions:default": ""})"#],
        events: &[r#"//:a: missing value for mandatory attribute 'x' in 'r' macro"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string()})"#,
        build: r#"r(name="a", x=None)"#,
        printed: &[r#"select({"//conditions:default": ""})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string()})"#,
        build: r#"r(name="a", x=1)"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"expected value of type 'string' for attribute 'x' of 'r', but got 1 (int)"#),
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.int()})"#,
        build: r#"r(name="a", x="s")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'int' for attribute 'x' of 'r', but got "s" (string)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.int()})"#,
        build: r#"r(name="a", x=1)"#,
        printed: &[r#"select({"//conditions:default": 1})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.int()})"#,
        build: r#"r(name="a")"#,
        printed: &[r#"select({"//conditions:default": 0})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.bool()})"#,
        build: r#"r(name="a")"#,
        printed: &[r#"select({"//conditions:default": False})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.bool()})"#,
        build: r#"r(name="a", x=1)"#,
        printed: &[r#"select({"//conditions:default": True})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.label()})"#,
        build: r#"r(name="a")"#,
        printed: &[r#"None"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.label()})"#,
        build: r#"r(name="a", x=":f1.txt")"#,
        printed: &[r#"select({"//conditions:default": Label("//:f1.txt")})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.label()})"#,
        build: r#"r(name="a", x="f1.txt")"#,
        printed: &[r#"select({"//conditions:default": Label("//:f1.txt")})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x, type(x))
r = macro(implementation=_i, attrs={'x':attr.label()})"#,
        build: r#"r(name="a", x="//:f1.txt")"#,
        printed: &[r#"select({"//conditions:default": Label("//:f1.txt")}) select"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.label()})"#,
        build: r#"r(name="a", x="x y z")"#,
        printed: &[r#"select({"//conditions:default": Label("//:x y z")})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.label()})"#,
        build: r#"r(name="a", x=1)"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"expected value of type 'string' for attribute 'x' of 'r', but got 1 (int)"#),
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.label_list()})"#,
        build: r#"r(name="a", x=["f1.txt"])"#,
        printed: &[r#"select({"//conditions:default": [Label("//:f1.txt")]})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.label_list()})"#,
        build: r#"r(name="a")"#,
        printed: &[r#"select({"//conditions:default": []})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.label_list()})"#,
        build: r#"r(name="a", x=["f1.txt", "f1.txt"])"#,
        printed: &[r#"select({"//conditions:default": [Label("//:f1.txt"), Label("//:f1.txt")]})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string_list()})"#,
        build: r#"r(name="a", x=["p","q"])"#,
        printed: &[r#"select({"//conditions:default": ["p", "q"]})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string_list()})"#,
        build: r#"r(name="a")"#,
        printed: &[r#"select({"//conditions:default": []})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string_list()})"#,
        build: r#"r(name="a", x="p")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'list(string)' for attribute 'x' of 'r', but got "p" (string)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string_dict()})"#,
        build: r#"r(name="a", x={"k":"v"})"#,
        printed: &[r#"select({"//conditions:default": {"k": "v"}})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string_dict()})"#,
        build: r#"r(name="a")"#,
        printed: &[r#"select({"//conditions:default": {}})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string_list_dict()})"#,
        build: r#"r(name="a", x={"k":["v"]})"#,
        printed: &[r#"select({"//conditions:default": {"k": ["v"]}})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x={"f1.txt":"v"})"#,
        printed: &[r#"select({"//conditions:default": {Label("//:f1.txt"): "v"}})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.int_list()})"#,
        build: r#"r(name="a", x=[1])"#,
        printed: &[r#"select({"//conditions:default": [1]})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.output()})"#,
        build: r#"r(name="a", x="o")"#,
        printed: &[r#"//:o"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.output_list()})"#,
        build: r#"r(name="a", x=["o"])"#,
        printed: &[r#"[Label("//:o")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string(values=['a','b'])})"#,
        build: r#"r(name="a", x="c")"#,
        printed: &[r#"select({"//conditions:default": "c"})"#],
        events: &[
            r#"//:a: invalid value in 'x' attribute: has to be one of 'a' or 'b' instead of 'c'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string(values=['a','b'])})"#,
        build: r#"r(name="a", x="a")"#,
        printed: &[r#"select({"//conditions:default": "a"})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.label_list(allow_empty=False)})"#,
        build: r#"r(name="a", x=[])"#,
        printed: &[r#"select({"//conditions:default": []})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.label(allow_single_file=True)})"#,
        build: r#"r(name="a", x="f1.txt")"#,
        printed: &[r#"select({"//conditions:default": Label("//:f1.txt")})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.label(default='//:f1.txt')})"#,
        build: r#"r(name="a")"#,
        printed: &[r#"select({"//conditions:default": Label("//:f1.txt")})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.label_list(default=['//:f1.txt'])})"#,
        build: r#"r(name="a")"#,
        printed: &[r#"select({"//conditions:default": [Label("//:f1.txt")]})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string_list(default=['p'])})"#,
        build: r#"r(name="a")"#,
        printed: &[r#"select({"//conditions:default": ["p"]})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.label(default='//other:y')})"#,
        build: r#"r(name="a")"#,
        printed: &[r#"select({"//conditions:default": Label("//other:y")})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.label(default=Label('//other:y'))})"#,
        build: r#"r(name="a")"#,
        printed: &[r#"select({"//conditions:default": Label("//other:y")})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string(configurable=False)})"#,
        build: r#"r(name="a", x="v")"#,
        printed: &[r#"v"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string()})"#,
        build: r#"r(name="a", x=select({"//conditions:default":"v"}))"#,
        printed: &[r#"select({"//conditions:default": "v"})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string_list()})"#,
        build: r#"r(name="a", x=select({"//conditions:default":["v"]}))"#,
        printed: &[r#"select({"//conditions:default": ["v"]})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, attrs={'x':attr.string_list()})"#,
        build: r#"r(name="a", x=select({"a":["v"]}) + ["p"])"#,
        printed: &[
            r#"select({Label("//:a"): ["v"]}) + select({Label("//conditions:default"): ["p"]})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(type(x), x)
r = macro(implementation=_i, attrs={'x':attr.string_list(configurable=False)})"#,
        build: r#"r(name="a", x=select({"a":["v"]}))"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"attribute "x" is not configurable"#),
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(type(x), x)
r = macro(implementation=_i, attrs={'x':attr.string()})"#,
        build: r#"r(name="a", x=select({"a":"v"}))"#,
        printed: &[r#"select select({Label("//:a"): "v"})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(type(x), x)
r = macro(implementation=_i, attrs={'x':attr.int()})"#,
        build: r#"r(name="a", x=select({"a":1}))"#,
        printed: &[r#"select select({Label("//:a"): 1})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(type(x), x)
r = macro(implementation=_i, attrs={'x':attr.label()})"#,
        build: r#"r(name="a", x=select({"a":":f1.txt"}))"#,
        printed: &[r#"select select({Label("//:a"): Label("//:f1.txt")})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(type(x), x)
r = macro(implementation=_i, attrs={'x':attr.string()})"#,
        build: r#"r(name="a", x=select({"a":1}))"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'string' for each branch in select expression of attribute 'x' of 'r' (including '//:a'), but got 1 (int)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
  print(native.existing_rules().keys())
r = macro(implementation=_i)"#,
        build: r#"filegroup(name="pre")
r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"existing_rules() can only be used while evaluating a BUILD file, a legacy macro, or a rule finalizer"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
  print(native.existing_rule(name+'_g') != None)
  print(native.existing_rule('pre'))
r = macro(implementation=_i)"#,
        build: r#"filegroup(name="pre")
r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"existing_rule() can only be used while evaluating a BUILD file, a legacy macro, or a rule finalizer"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(native.existing_rules().keys())
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
r(name="b")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"existing_rules() can only be used while evaluating a BUILD file, a legacy macro, or a rule finalizer"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
print(existing_rules().keys())"#,
        printed: &[r#"["a_g"]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.package(default_visibility=['//visibility:public'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"package() can only be used while evaluating a BUILD file"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.glob(['*.txt'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"glob() can only be used while evaluating a BUILD file or a legacy macro"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(native.glob(['*.txt']))
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"glob() can only be used while evaluating a BUILD file or a legacy macro"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(native.package_name(), native.repository_name())
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[r#" @"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(native.package_relative_label(':x'))
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[r#"//:x"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.exports_files(['f1.txt'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.package_group(name=name+'_pg', packages=[])
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.alias(name=name+'_al', actual=':f1.txt')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.existing_rules()
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"existing_rules() can only be used while evaluating a BUILD file, a legacy macro, or a rule finalizer"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g', visibility=kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//visibility:public"])
print(existing_rule("a_g")["visibility"])"#,
        printed: &[r#"("//visibility:public",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//visibility:public"])
print(existing_rule("a_g")["visibility"])"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g', visibility=['//foo:__pkg__'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
print(existing_rule("a_g")["visibility"])"#,
        printed: &[r#"("//foo:__pkg__",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g', visibility=['//foo:__pkg__'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//bar:__pkg__"])
print(existing_rule("a_g")["visibility"])"#,
        printed: &[r#"("//foo:__pkg__",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g', visibility=kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
print(existing_rule("a_g")["visibility"])"#,
        printed: &[r#"(":__pkg__",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g', visibility=kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//foo:__subpackages__"])
print(existing_rule("a_g")["visibility"])"#,
        printed: &[r#"(":__pkg__", "//foo:__subpackages__")"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g', visibility=None)
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
print(existing_rule("a_g")["visibility"])"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g', visibility=[Label("//foo:__pkg__")])
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
print(existing_rule("a_g")["visibility"])"#,
        printed: &[r#"("//foo:__pkg__",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g', visibility=['//visibility:private'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
print(existing_rule("a_g")["visibility"])"#,
        printed: &[r#"("//visibility:private",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g', visibility=['//visibility:public'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
print(existing_rule("a_g")["visibility"])"#,
        printed: &[r#"("//visibility:public",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g', visibility=['//visibility:public'])
r = macro(implementation=_i)"#,
        build: r#"package(default_visibility=["//foo:__pkg__"])
r(name="a")
print(existing_rule("a_g")["visibility"])"#,
        printed: &[r#"("//visibility:public",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"package(default_visibility=["//foo:__pkg__"])
r(name="a")
print(existing_rule("a_g")["visibility"])"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"package(default_visibility=["//foo:__pkg__"])
r(name="a", visibility=["//bar:__pkg__"])
print(existing_rule("a_g")["visibility"])"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
print(existing_rule("a_g")["visibility"])"#,
        printed: &[r#"()"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g', visibility=['//foo:__pkg__'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//visibility:public"])
print(existing_rule("a_g")["visibility"])"#,
        printed: &[r#"("//foo:__pkg__",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _inner(name, **kw):
  native.filegroup(name=name+'_g')
inner = macro(implementation=_inner)
def _i(name, **kw):
  inner(name=name+'_in')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _inner(name, **kw):
  native.filegroup(name=name+'_g')
inner = macro(implementation=_inner)
def _i(name, **kw):
  inner(name=name+'_in')
  print(native.existing_rules().keys())
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
print(existing_rules().keys())"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"existing_rules() can only be used while evaluating a BUILD file, a legacy macro, or a rule finalizer"#,
        ),
    },
    BuildRow {
        bzl: r#"def _inner(name, **kw):
  print(native.existing_rules().keys())
inner = macro(implementation=_inner)
def _i(name, **kw):
  native.filegroup(name=name+'_x')
  inner(name=name+'_in')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"existing_rules() can only be used while evaluating a BUILD file, a legacy macro, or a rule finalizer"#,
        ),
    },
    BuildRow {
        bzl: r#"def _inner(name, **kw):
  native.filegroup(name=name+'_g')
inner = macro(implementation=_inner)
def _i(name, **kw):
  inner(name=name+'_in')
  inner(name=name+'_in')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"macro 'a_in' conflicts with an existing macro (and was not created by it)"#),
    },
    BuildRow {
        bzl: r#"def _inner(name, **kw):
  print(kw)
inner = macro(implementation=_inner)
def _i(name, **kw):
  inner(name=name+'_in', visibility=kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[r#"{"visibility": [Label("//:__pkg__")]}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _inner(name, **kw):
  print(kw)
inner = macro(implementation=_inner)
def _i(name, **kw):
  inner(name=name+'_in')
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//foo:__pkg__"])"#,
        printed: &[r#"{"visibility": [Label("//:__pkg__")]}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  r(name=name+'_x')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"macro 'a_x' is a direct recursive call of 'a'. Macro instantiation traceback (most recent call last):"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
m = macro(implementation=_i)
def r(name):
  m(name=name)"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
m = macro(implementation=_i)
def r(name):
  m(name=name)
  print(native.existing_rules().keys())"#,
        build: r#"r("a")"#,
        printed: &[r#"["a_g"]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name='f1.txt')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name='sub/x')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:sub/x' is invalid because 'sub' is a subpackage; perhaps you meant to put the colon here: '//sub:x'?"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g', srcs=['sub/x.txt'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g', srcs=['nope'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g', srcs=['f1.txt', 'f1.txt'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:2: Label '//:f1.txt' is duplicated in the 'srcs' attribute of rule 'a_g'"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g', nope=1)
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[r#"BUILD.bazel:2:2: //:a_g: no such attribute 'nope' in 'filegroup' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.nope(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"no native function or rule 'nope'"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=1)
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"filegroup 'name' attribute must be a string"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup()
r = macro(implementation=_i)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"filegroup rule has no 'name' attribute"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
r(name="b")
print(existing_rules().keys())"#,
        printed: &[r#"[]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"filegroup(name="a")
r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"macro 'a' conflicts with an existing target."#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i)"#,
        build: r#"filegroup(name="a")
r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"macro 'a' conflicts with an existing target."#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
filegroup(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"target 'a' conflicts with an existing macro (and was not created by it)"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
filegroup(name="a_g")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name)
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
filegroup(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"filegroup rule 'a' conflicts with existing filegroup rule, defined at BUILD.bazel:2:2"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name)
r = macro(implementation=_i)"#,
        build: r#"filegroup(name="a")
r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"macro 'a' conflicts with an existing target."#),
    },
    BuildRow {
        bzl: r#"x_rule = rule(implementation=lambda ctx: [], outputs={'o':'%{name}.o'})
def _i(name, **kw):
  x_rule(name=name)
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
print(existing_rules().keys())"#,
        printed: &[r#"["a"]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=[])"#,
        printed: &[r#"[Label("//:__pkg__")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//visibility:private"])"#,
        printed: &[r#"[Label("//:__pkg__")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//visibility:public","//foo:__pkg__"])"#,
        printed: &[r#"[Label("//visibility:public")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//foo:__pkg__","//foo:__pkg__"])"#,
        printed: &[r#"[Label("//:__pkg__"), Label("//foo:__pkg__"), Label("//foo:__pkg__")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//foo:__pkg__",":__pkg__"])"#,
        printed: &[r#"[Label("//:__pkg__"), Label("//foo:__pkg__")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=[":x",":a"])"#,
        printed: &[r#"[Label("//:__pkg__"), Label("//:a"), Label("//:x")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["@other//foo:__pkg__"])"#,
        printed: &[
            r#"[Label("//:__pkg__"), Label("@@[unknown repo 'other' requested from @@]//foo:__pkg__")]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//foo:__subpackages__"])"#,
        printed: &[r#"[Label("//:__pkg__"), Label("//foo:__subpackages__")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//foo:x"])"#,
        printed: &[r#"[Label("//:__pkg__"), Label("//foo:x")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["__pkg__"])"#,
        printed: &[r#"[Label("//:__pkg__")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=[":__pkg__"])"#,
        printed: &[r#"[Label("//:__pkg__")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//:__pkg__"])"#,
        printed: &[r#"[Label("//:__pkg__")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//visibility:private","//foo:__pkg__"])"#,
        printed: &[r#"[Label("//:__pkg__"), Label("//foo:__pkg__")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=["//z:__pkg__","//a:__pkg__","//m:__pkg__"])"#,
        printed: &[
            r#"[Label("//:__pkg__"), Label("//a:__pkg__"), Label("//m:__pkg__"), Label("//z:__pkg__")]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=("//foo:__pkg__",))"#,
        printed: &[r#"[Label("//:__pkg__"), Label("//foo:__pkg__")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(kw['visibility'])
r = macro(implementation=_i)"#,
        build: r#"r(name="a", visibility=None)"#,
        printed: &[r#"[Label("//:__pkg__")]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('finalizer', name, sorted(native.existing_rules().keys()))
r = macro(implementation=_i, finalizer=True)"#,
        build: r#"filegroup(name="p1")
r(name="a")
filegroup(name="p2")
print("build done")"#,
        printed: &[r#"build done"#, r#"finalizer a ["p1", "p2"]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name+'_g')
  print(sorted(native.existing_rules().keys()))
r = macro(implementation=_i, finalizer=True)"#,
        build: r#"r(name="a")"#,
        printed: &[r#"[]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('impl', name)
r = macro(implementation=_i, finalizer=True)"#,
        build: r#"r(name="a")
print("build")"#,
        printed: &[r#"build"#, r#"impl a"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('impl', name)
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
print("build")"#,
        printed: &[r#"impl a"#, r#"build"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _n(name, **kw):
  print('inner finalizer')
finn = macro(implementation=_n, finalizer=True)
def _i(name, **kw):
  finn(name=name+'_in')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
print("b")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"Cannot instantiate a rule finalizer within a non-finalizer symbolic macro. Rule finalizers may only be instantiated while evaluating a BUILD file, a legacy macro called from a BUILD file, or another rule finalizer."#,
        ),
    },
    BuildRow {
        bzl: r#"def _n(name, **kw):
  print('inner finalizer', sorted(native.existing_rules().keys()))
finn = macro(implementation=_n, finalizer=True)
def _i(name, **kw):
  native.filegroup(name=name+'_g')
  finn(name=name+'_in')
  native.filegroup(name=name+'_h')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
print("b")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"Cannot instantiate a rule finalizer within a non-finalizer symbolic macro. Rule finalizers may only be instantiated while evaluating a BUILD file, a legacy macro called from a BUILD file, or another rule finalizer."#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i)
r(name='x')"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"a symbolic macro can only be instantiated while evaluating a BUILD file or a legacy or symbolic macro"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i)
def f():
  r(name='x')
f()"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"a symbolic macro can only be instantiated while evaluating a BUILD file or a legacy or symbolic macro"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i)"#,
        build: r#"r(name="a", **{"visibility":["//foo:__pkg__"]})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:13: **kwargs arguments are not allowed in BUILD files. Pass the arguments in explicitly."#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i)"#,
        build: r#"args={"name":"a"}
r(**args)"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:3:3: **kwargs arguments are not allowed in BUILD files. Pass the arguments in explicitly."#,
            r#"BUILD.bazel:3:3: **kwargs arguments must be a literal dict in BUILD files."#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
x=[macro(implementation=_i)]"#,
        build: r#"print(1)"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"file ':u.bzl' does not contain symbol 'r'"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
def mk():
  return macro(implementation=_i)
r = mk()"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  fail('x')
_r = macro(implementation=_i)
r=_r"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"x"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  fail('x')
_r = macro(implementation=_i)
def r(name): _r(name=name)"#,
        build: r#"r("a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"x"#),
    },
    BuildRow {
        bzl: r#"def _f(name, **kw):
  print('fin', sorted(native.existing_rules().keys()))
fin = macro(implementation=_f, finalizer=True)
def _i(name, **kw):
  native.filegroup(name=name+'_g')
r = macro(implementation=_i)"#,
        build: r#"load(":u.bzl","fin")
fin(name="f")
r(name="a")
r(name="b")"#,
        printed: &[r#"fin ["a_g", "b_g"]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _f(name, **kw):
  print('fin', name)
fin = macro(implementation=_f, finalizer=True)
r=1"#,
        build: r#"load(":u.bzl","fin")
fin(name="f1")
fin(name="f2")
print("b")"#,
        printed: &[r#"b"#, r#"fin f1"#, r#"fin f2"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _f(name, **kw):
  native.filegroup(name=name+'_x')
fin = macro(implementation=_f, finalizer=True)
r=1"#,
        build: r#"load(":u.bzl","fin")
fin(name="f1")
print(existing_rules().keys())"#,
        printed: &[r#"[]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _f(name, **kw):
  print(native.existing_rule('nope'))
fin = macro(implementation=_f, finalizer=True)
r=1"#,
        build: r#"load(":u.bzl","fin")
fin(name="f1")"#,
        printed: &[r#"None"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _f(name, **kw):
  print(native.glob(['*.txt']))
fin = macro(implementation=_f, finalizer=True)
r=1"#,
        build: r#"load(":u.bzl","fin")
fin(name="f1")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"glob() can only be used while evaluating a BUILD file or a legacy macro"#),
    },
    BuildRow {
        bzl: r#"def _f(name, **kw):
  native.package(default_visibility=['//visibility:public'])
fin = macro(implementation=_f, finalizer=True)
r=1"#,
        build: r#"load(":u.bzl","fin")
fin(name="f1")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"package() can only be used while evaluating a BUILD file"#),
    },
    BuildRow {
        bzl: r#"def _f(name, **kw):
  fail('boom')
fin = macro(implementation=_f, finalizer=True)
r=1"#,
        build: r#"load(":u.bzl","fin")
fin(name="f1")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"boom"#),
    },
    BuildRow {
        bzl: r#"def _f(name, **kw):
  print(sorted(native.existing_rules().keys()))
  native.filegroup(name=name+'_x')
  print(sorted(native.existing_rules().keys()))
fin = macro(implementation=_f, finalizer=True)
r=1"#,
        build: r#"load(":u.bzl","fin")
filegroup(name="p")
fin(name="f1")"#,
        printed: &[r#"["p"]"#, r#"["p"]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _n(name, **kw):
  print('inner')
inner = macro(implementation=_n)
def _f(name, **kw):
  inner(name=name+'_in')
fin = macro(implementation=_f, finalizer=True)
r=1"#,
        build: r#"load(":u.bzl","fin")
fin(name="f1")
print("b")"#,
        printed: &[r#"b"#, r#"inner"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _f(name, **kw):
  fin(name=name+'_x')
fin = macro(implementation=_f, finalizer=True)
r=1"#,
        build: r#"load(":u.bzl","fin")
fin(name="f1")"#,
        printed: &[],
        events: &[
            r#"macro 'f1_x' is a direct recursive call of 'f1'. Macro instantiation traceback (most recent call last):"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _f(name, **kw):
  print('fin')
fin = macro(implementation=_f, finalizer=True)
def leg(name):
  fin(name=name)
r=1"#,
        build: r#"load(":u.bzl","leg")
leg("a")
print("b")"#,
        printed: &[r#"b"#, r#"fin"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _f(name, **kw):
  print('fin', name)
fin = macro(implementation=_f, finalizer=True)
def _i(name, **kw):
  fin(name=name+'_f')
outer = macro(implementation=_i, finalizer=True)
r=1"#,
        build: r#"load(":u.bzl","outer")
outer(name="a")
print("b")"#,
        printed: &[r#"b"#, r#"fin a_f"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _f(name, **kw):
  print('fin1')
fin = macro(implementation=_f, finalizer=True)
def _g(name, **kw):
  print('impl2')
r = macro(implementation=_g)"#,
        build: r#"load(":u.bzl","fin")
fin(name="f1")
r(name="a")
print("b")"#,
        printed: &[r#"impl2"#, r#"b"#, r#"fin1"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _inner(name, **kw):
  native.filegroup(name=name+'_g')
inner = macro(implementation=_inner)
def _i(name, **kw):
  inner(name=name+'_in')
r = macro(implementation=_i)"#,
        build: r#"load(":u.bzl","inner")
r(name="a")
inner(name="b")
print(existing_rules().keys())"#,
        printed: &[r#"["a_in_g", "b_g"]"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('impl', name)
r=[macro(implementation=_i)]
m=r[0]"#,
        build: r#"load(":u.bzl","m")
m(name="a")"#,
        printed: &[r#"impl a"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('impl', name)
r={'a': macro(implementation=_i)}
m=r['a']"#,
        build: r#"load(":u.bzl","m")
m(name="a")"#,
        printed: &[r#"impl a"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('impl', name)
def mk():
  return macro(implementation=_i)
m = mk()
r=1"#,
        build: r#"load(":u.bzl","m")
m(name="a")"#,
        printed: &[r#"impl a"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('impl', name)
r = macro(implementation=_i)
m = r"#,
        build: r#"load(":u.bzl","m")
m(name="a")
r(name="b")"#,
        printed: &[r#"impl a"#, r#"impl b"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('impl', name)
m = macro(implementation=_i)
r = m"#,
        build: r#"load(":u.bzl","m")
r(name="a")
m(name="b")"#,
        printed: &[r#"impl a"#, r#"impl b"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('impl', name)
_m = macro(implementation=_i)
m = _m
r=1"#,
        build: r#"load(":u.bzl","m")
m(name="a")"#,
        printed: &[r#"impl a"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('impl', name)
r = macro(implementation=_i)
print(r)
m=r"#,
        build: r#"load(":u.bzl","m")
m(name="a")
print(m)"#,
        printed: &[r#"<macro r>"#, r#"impl a"#, r#"<macro r>"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  fail('x')
m = macro(implementation=_i)
r=1"#,
        build: r#"load(":u.bzl","m")
m(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"x"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  native.filegroup(name=name, nope=1)
m = macro(implementation=_i)
r=1"#,
        build: r#"load(":u.bzl","m")
m(name="a")"#,
        printed: &[],
        events: &[r#"BUILD.bazel:3:2: //:a: no such attribute 'nope' in 'filegroup' rule"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"E = rule(implementation=lambda ctx: [], executable=True, attrs={'x':attr.string()})
def _i(name, **kw):
  print(sorted(kw.keys()))
r = macro(implementation=_i, inherit_attrs=E)"#,
        build: r#"r(name="a")"#,
        printed: &[
            r#"["args", "aspect_hints", "compatible_with", "deprecation", "exec_compatible_with", "exec_group_compatible_with", "exec_properties", "expect_failure", "features", "output_licenses", "package_metadata", "restricted_to", "tags", "target_compatible_with", "testonly", "toolchains", "transitive_configs", "visibility", "x"]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(sorted(kw.keys()))
r = macro(implementation=_i, inherit_attrs=native.filegroup)"#,
        build: r#"r(name="a")"#,
        printed: &[
            r#"["aspect_hints", "compatible_with", "data", "deprecation", "distribs", "features", "licenses", "output_group", "output_licenses", "package_metadata", "restricted_to", "srcs", "tags", "target_compatible_with", "testonly", "transitive_configs", "visibility"]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(sorted(kw.keys()))
r = macro(implementation=_i, inherit_attrs=native.alias)"#,
        build: r#"r(name="a", actual=":x")"#,
        printed: &[
            r#"["actual", "aspect_hints", "compatible_with", "deprecation", "features", "package_metadata", "restricted_to", "tags", "target_compatible_with", "testonly", "transitive_configs", "visibility"]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(sorted(kw.keys()))
r = macro(implementation=_i, inherit_attrs=native.cc_library)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"in call to macro(), parameter 'inherit_attrs' got value of type 'function', want 'rule, macro, string, or NoneType'"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(sorted(kw.keys()))
r = macro(implementation=_i, inherit_attrs=native.genrule)"#,
        build: r#"r(name="a")"#,
        printed: &[
            r#"["aspect_hints", "cmd", "cmd_bash", "cmd_bat", "cmd_ps", "compatible_with", "deprecation", "distribs", "exec_compatible_with", "exec_group_compatible_with", "exec_properties", "executable", "features", "heuristic_label_expansion", "licenses", "local", "message", "output_licenses", "output_to_bindir", "outs", "package_metadata", "restricted_to", "srcs", "stamp", "tags", "target_compatible_with", "testonly", "toolchains", "tools", "transitive_configs", "visibility"]"#,
        ],
        events: &[r#"//:a: missing value for mandatory attribute 'outs' in 'r' macro"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _j(name, y, **kw):
  print(name, y, sorted(kw.keys()))
base = macro(implementation=_j, attrs={'y': attr.string(default='dy'), 'w': attr.int()})
def _i(name, **kw):
  print(sorted(kw.keys()), kw)
r = macro(implementation=_i, inherit_attrs=base)"#,
        build: r#"r(name="a", w=1)"#,
        printed: &[
            r#"["visibility", "w", "y"] {"visibility": [Label("//:__pkg__")], "y": None, "w": select({"//conditions:default": 1})}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _j(name, y, **kw):
  pass
base = macro(implementation=_j, attrs={'y': attr.string(default='dy'), 'w': attr.int()})
def _i(name, **kw):
  print(sorted(kw.keys()), kw)
r = macro(implementation=_i, inherit_attrs=base, attrs={'y': None})"#,
        build: r#"r(name="a", w=1)"#,
        printed: &[
            r#"["visibility", "w"] {"visibility": [Label("//:__pkg__")], "w": select({"//conditions:default": 1})}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _j(name, **kw):
  pass
base = macro(implementation=_j, inherit_attrs='common')
def _i(name, **kw):
  print(sorted(kw.keys()))
r = macro(implementation=_i, inherit_attrs=base)"#,
        build: r#"r(name="a")"#,
        printed: &[
            r#"["aspect_hints", "compatible_with", "deprecation", "exec_compatible_with", "exec_group_compatible_with", "exec_properties", "expect_failure", "features", "package_metadata", "restricted_to", "tags", "target_compatible_with", "testonly", "toolchains", "transitive_configs", "visibility"]"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _j(name, **kw):
  pass
base = macro(implementation=_j, attrs={'q': attr.string()})
def _i(name, **kw):
  print(sorted(kw.keys()), kw)
r = macro(implementation=_i, inherit_attrs=base, attrs={'q': attr.int()})"#,
        build: r#"r(name="a", q=1)"#,
        printed: &[
            r#"["q", "visibility"] {"visibility": [Label("//:__pkg__")], "q": select({"//conditions:default": 1})}"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _j(name, **kw):
  pass
base = macro(implementation=_j, attrs={'q': attr.string(mandatory=True)})
def _i(name, **kw):
  print(sorted(kw.keys()), kw)
r = macro(implementation=_i, inherit_attrs=base)"#,
        build: r#"r(name="a")"#,
        printed: &[
            r#"["q", "visibility"] {"visibility": [Label("//:__pkg__")], "q": select({"//conditions:default": ""})}"#,
        ],
        events: &[r#"//:a: missing value for mandatory attribute 'q' in 'r' macro"#],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _j(name, **kw):
  pass
base = macro(implementation=_j, attrs={'q': attr.string(default='d')})
def _i(name, **kw):
  print(sorted(kw.keys()), kw)
r = macro(implementation=_i, inherit_attrs=base)"#,
        build: r#"r(name="a")"#,
        printed: &[r#"["q", "visibility"] {"visibility": [Label("//:__pkg__")], "q": None}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _j(name, **kw):
  pass
base = macro(implementation=_j, attrs={'q': attr.string(default='d')})
r = macro(implementation=_j, inherit_attrs=base)
def _i(name, **kw):
  print(kw)
r2 = macro(implementation=_i, inherit_attrs=r)"#,
        build: r#"load(":u.bzl","r2")
r2(name="a")"#,
        printed: &[r#"{"visibility": [Label("//:__pkg__")], "q": None}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"R = rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), 'z':attr.string_list(mandatory=True), '_p':attr.string(default='d')})
def _i(name, **kw):
  print(kw['x'], kw['y'])
r = macro(implementation=_i, inherit_attrs=R, attrs={'z': None})"#,
        build: r#"r(name="a")"#,
        printed: &[r#"None None"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"R = rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), 'z':attr.string_list(mandatory=True), '_p':attr.string(default='d')})
def _i(name, x, **kw):
  print(x)
r = macro(implementation=_i, inherit_attrs=R)"#,
        build: r#"r(name="a", z=["q"])"#,
        printed: &[r#"None"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"R = rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), 'z':attr.string_list(mandatory=True), '_p':attr.string(default='d')})
def _i(name, **kw):
  native.filegroup(name=name, **{k:v for k,v in kw.items() if k in ['tags','visibility','testonly']})
r = macro(implementation=_i, inherit_attrs=R)"#,
        build: r#"r(name="a", z=["q"], tags=["t"])
print(existing_rule("a")["tags"])"#,
        printed: &[r#"("t",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"R = rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), 'z':attr.string_list(mandatory=True), '_p':attr.string(default='d')})
def _i(name, visibility, tags, **kw):
  native.filegroup(name=name, visibility=visibility, tags=tags)
r = macro(implementation=_i, inherit_attrs=R)"#,
        build: r#"r(name="a", z=["q"], tags=["t"])
print(existing_rule("a")["tags"], existing_rule("a")["visibility"])"#,
        printed: &[r#"("t",) (":__pkg__",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"R = rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), 'z':attr.string_list(mandatory=True), '_p':attr.string(default='d')})
def _i(name, **kw):
  R(name=name, **kw)
r = macro(implementation=_i, inherit_attrs=R)"#,
        build: r#"r(name="a", z=["q"], x="v")
print(existing_rule("a")["x"], existing_rule("a")["z"])"#,
        printed: &[r#"v ("q",)"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"R = rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), 'z':attr.string_list(mandatory=True), '_p':attr.string(default='d')})
def _i(name, **kw):
  R(name=name, **kw)
r = macro(implementation=_i, inherit_attrs=R)"#,
        build: r#"r(name="a", z=["q"], x=select({"//conditions:default":"v"}))
print(existing_rule("a")["x"])"#,
        printed: &[r#"v"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"R = rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), 'z':attr.string_list(mandatory=True), '_p':attr.string(default='d')})
def _i(name, **kw):
  R(name=name, **kw)
r = macro(implementation=_i, inherit_attrs=R)"#,
        build: r#"r(name="a", z=["q"], x=select({"//:k":"v"}))
print(existing_rule("a")["x"])"#,
        printed: &[r#"select({Label("//:k"): "v"})"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"R = rule(implementation=lambda ctx: [], attrs={'x':attr.string(), 'y':attr.label(), 'z':attr.string_list(mandatory=True), '_p':attr.string(default='d')})
def _i(name, **kw):
  R(name=name, **kw)
r = macro(implementation=_i, inherit_attrs=R)"#,
        build: r#"r(name="a", z=["q"], y=":f1.txt")
print(existing_rule("a")["y"])"#,
        printed: &[r#":f1.txt"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility=["x y"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility=[1])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'string' for element 0 of attribute 'visibility' of 'r', but got 1 (int)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility=[None])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'string' for element 0 of attribute 'visibility' of 'r', but got None (NoneType)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility="//visibility:public")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'list(label)' for attribute 'visibility' of 'r', but got "//visibility:public" (string)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility=["//visibility:public", 1])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'string' for element 1 of attribute 'visibility' of 'r', but got 1 (int)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility=["//x:"])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"invalid label '//x:' in element 0 of attribute 'visibility' of 'r': invalid target name '': empty target name"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility=[":"])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"invalid label ':' in element 0 of attribute 'visibility' of 'r': invalid target name '': empty target name"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility=[""])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"invalid label '' in element 0 of attribute 'visibility' of 'r': invalid target name '': empty target name"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility=["@@foo//x:y"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility=["//visibility:public:x"])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"invalid label '//visibility:public:x' in element 0 of attribute 'visibility' of 'r': invalid target name 'public:x': target names may not contain ':'"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility=["//pkg:__pkg__"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility=["//pkg:__subpackages__"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility=["//pkg"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility=["//:x","//:x"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility={"a":1})"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility={})"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility=())"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility=1.5)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'list(label)' for attribute 'visibility' of 'r', but got 1.5 (float)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={})"#,
        build: r#"r(name="a", visibility=True)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'list(label)' for attribute 'visibility' of 'r', but got True (bool)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.label()})"#,
        build: r#"r(name="a", x="x y")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.label()})"#,
        build: r#"r(name="a", x="//x:")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"invalid label '//x:' in attribute 'x' of 'r': invalid target name '': empty target name"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.label()})"#,
        build: r#"r(name="a", x="@@foo//x:y")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.label()})"#,
        build: r#"r(name="a", x="//x:y:z")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"invalid label '//x:y:z' in attribute 'x' of 'r': invalid target name 'y:z': target names may not contain ':'"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.label()})"#,
        build: r#"r(name="a", x=[])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'string' for attribute 'x' of 'r', but got [] (list)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.label()})"#,
        build: r#"r(name="a", x=None)"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.label()})"#,
        build: r#"r(name="a", x="//visibility:public")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.label()})"#,
        build: r#"r(name="a", x=":")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"invalid label ':' in attribute 'x' of 'r': invalid target name '': empty target name"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.label()})"#,
        build: r#"r(name="a", x="")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"invalid label '' in attribute 'x' of 'r': invalid target name '': empty target name"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.label_list()})"#,
        build: r#"r(name="a", x=["x y"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.label_list()})"#,
        build: r#"r(name="a", x=["//x:"])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"invalid label '//x:' in element 0 of attribute 'x' of 'r': invalid target name '': empty target name"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.label_list()})"#,
        build: r#"r(name="a", x=[1])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'string' for element 0 of attribute 'x' of 'r', but got 1 (int)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.label_list()})"#,
        build: r#"r(name="a", x="x")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'list(label)' for attribute 'x' of 'r', but got "x" (string)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.label_list()})"#,
        build: r#"r(name="a", x=[":a",":a"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.int()})"#,
        build: r#"r(name="a", x=2147483648)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"for attribute 'x' of 'r', got 2147483648, want value in signed 32-bit range"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.int()})"#,
        build: r#"r(name="a", x=True)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'int' for attribute 'x' of 'r', but got True (bool)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.int()})"#,
        build: r#"r(name="a", x=1.5)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'int' for attribute 'x' of 'r', but got 1.5 (float)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.int()})"#,
        build: r#"r(name="a", x=None)"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.bool()})"#,
        build: r#"r(name="a", x=2)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got 2 (int)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.bool()})"#,
        build: r#"r(name="a", x="x")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected one of [False, True, 0, 1] for attribute 'x' of 'r', but got "x" (string)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.bool()})"#,
        build: r#"r(name="a", x=0)"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.string_dict()})"#,
        build: r#"r(name="a", x={1:"a"})"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"expected value of type 'string' for dict key element, but got 1 (int)"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.string_dict()})"#,
        build: r#"r(name="a", x={"a":1})"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"expected value of type 'string' for dict value element, but got 1 (int)"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.string_dict()})"#,
        build: r#"r(name="a", x=[])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'dict(string, string)' for attribute 'x' of 'r', but got [] (list)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x={"x y":"a"})"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x={1:"a"})"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"expected value of type 'string' for dict key element, but got 1 (int)"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.label_keyed_string_dict()})"#,
        build: r#"r(name="a", x={":a":1})"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"expected value of type 'string' for dict value element, but got 1 (int)"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.string_list_dict()})"#,
        build: r#"r(name="a", x={"a":"b"})"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'list(string)' for dict value element, but got "b" (string)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.string_list_dict()})"#,
        build: r#"r(name="a", x={"a":[1]})"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'string' for element 0 of dict value element, but got 1 (int)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.output()})"#,
        build: r#"r(name="a", x="x y")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.output()})"#,
        build: r#"r(name="a", x=1)"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"expected value of type 'string' for attribute 'x' of 'r', but got 1 (int)"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.output()})"#,
        build: r#"r(name="a", x="sub/x.txt")"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.output()})"#,
        build: r#"r(name="a", x="//x:y")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"label '//x:y' is not in the current package"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.output_list()})"#,
        build: r#"r(name="a", x=["a","a"])"#,
        printed: &[],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, attrs={'x': attr.output_list()})"#,
        build: r#"r(name="a", x=[1])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'string' for element 0 of attribute 'x' of 'r', but got 1 (int)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  r(name=name+'_x')
r = macro(implementation=_i)"#,
        build: r#"r(name="a")
print("after")"#,
        printed: &[r#"after"#],
        events: &[
            r#"macro 'a_x' is a direct recursive call of 'a'. Macro instantiation traceback (most recent call last):"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _j(name, **kw):
  q(name=name+'_q')
j = macro(implementation=_j)
def _k(name, **kw):
  j(name=name+'_j')
q = macro(implementation=_k)
r=1"#,
        build: r#"load(":u.bzl","q")
q(name="a")
print("after")"#,
        printed: &[r#"after"#],
        events: &[
            r#"macro 'a_j_q' is an indirect recursive call of 'a'. Macro instantiation traceback (most recent call last):"#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  fail('deep')
def _o(name, **kw):
  inner(name=name+'_in')
inner = macro(implementation=_i)
r = macro(implementation=_o)"#,
        build: r#"r(name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"deep"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('impl', name)
cfg = {'a': macro(implementation=_i)}
r=1"#,
        build: r#"load(":u.bzl","cfg")
cfg["a"](name="a")"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"Cannot instantiate a macro that has not been exported (assign it to a global variable in the .bzl where it's defined)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('impl', name)
cfg = [macro(implementation=_i)]
r=1"#,
        build: r#"load(":u.bzl","cfg")
cfg[0](name="a", foo=1)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"Cannot instantiate a macro that has not been exported (assign it to a global variable in the .bzl where it's defined)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('impl', name)
r = macro(implementation=_i)
def leg(name):
  r(name=name, nope=1)"#,
        build: r#"load(":u.bzl","leg")
leg("a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"no such attribute 'nope' in 'r' macro"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('impl', name)
r = macro(implementation=_i)
def leg(name):
  r(name, name='b')"#,
        build: r#"load(":u.bzl","leg")
leg("a")"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"unexpected positional arguments"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('impl', name)
r = macro(implementation=_i)"#,
        build: r#"print(r.foo)"#,
        printed: &[],
        events: &[],
        fatal: Some(r#"'macro' value has no field or method 'foo'"#),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('impl', name)
r = macro(implementation=_i)"#,
        build: r#"x=r
x(name="a")"#,
        printed: &[r#"impl a"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i)"#,
        build: r#"print(type(r), bool(r), dir(r))"#,
        printed: &[r#"macro True []"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i)"#,
        build: r#"print({r: 1})"#,
        printed: &[r#"{<macro r>: 1}"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  pass
r = macro(implementation=_i, doc='d')"#,
        build: r#"r(name="a")
print(1)"#,
        printed: &[r#"1"#],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, x, **kw):
  print(name, x)
r = macro(implementation=_i, attrs={'x': attr.string()})"#,
        build: r#"r(name="a", x="v")
r(name="b")"#,
        printed: &[
            r#"a select({"//conditions:default": "v"})"#,
            r#"b select({"//conditions:default": ""})"#,
        ],
        events: &[],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print('impl', name)
r = macro(implementation=_i)"#,
        build: r#"r(name="a", **{})"#,
        printed: &[],
        events: &[
            r#"BUILD.bazel:2:13: **kwargs arguments are not allowed in BUILD files. Pass the arguments in explicitly."#,
        ],
        fatal: None,
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(type(name))
r = macro(implementation=_i)"#,
        build: r#"r(name=["a"])"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'string' for attribute 'name' of 'r', but got ["a"] (list)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(type(name))
r = macro(implementation=_i)"#,
        build: r#"r(name=True)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'string' for attribute 'name' of 'r', but got True (bool)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(type(name))
r = macro(implementation=_i)"#,
        build: r#"r(name=1.5)"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'string' for attribute 'name' of 'r', but got 1.5 (float)"#,
        ),
    },
    BuildRow {
        bzl: r#"def _i(name, **kw):
  print(type(name))
r = macro(implementation=_i)"#,
        build: r#"r(name={})"#,
        printed: &[],
        events: &[],
        fatal: Some(
            r#"expected value of type 'string' for attribute 'name' of 'r', but got {} (dict)"#,
        ),
    },
];
