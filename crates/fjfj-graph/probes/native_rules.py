#!/usr/bin/env python3
"""Reads the schema of each native rule class from Bazel 9.2.0.

For every class the script instantiates `R(name = "x", ...)` in a scratch
workspace, giving each mandatory attribute a value of the type Bazel says it
wants, then records from `bazel query --output=streamed_jsonproto` the
attributes (name, type, default) and from `existing_rule("x").keys()` which of
them `native.existing_rule` shows. Writes native_rules.json next to itself.

    BAZEL=/path/to/bazel python3 native_rules.py
"""
import ast, json, os, re, subprocess, sys, tempfile

BAZEL = os.environ.get("BAZEL", "bazel")
HERE = os.path.dirname(os.path.abspath(__file__))

CLASSES = """action_listener alias cc_binary cc_import cc_libc_top_alias cc_library
cc_shared_library cc_static_library cc_test cc_toolchain cc_toolchain_alias
cc_toolchain_suite config_feature_flag config_setting constraint_setting
constraint_value environment extra_action fdo_prefetch_hints fdo_profile filegroup
genquery genrule java_binary java_import java_library java_package_configuration
java_plugin java_plugins_flag_alias java_runtime java_test java_toolchain
label_flag label_setting memprof_profile objc_import objc_library platform
propeller_optimize starlark_doc_extract test_suite toolchain toolchain_type
alias""".split()

# What to give an attribute of each type the error message names.
BY_TYPE = {
    "string": '"x"',
    "int": "1",
    "bool": "True",
    "list(string)": '["x"]',
    "sequence": '["x"]',
    "list(label)": '["//:x"]',
    "list(output)": '["x.out"]',
    "dict": '{"x": "x"}',
    "dict(string, string)": '{"x": "x"}',
    "dict(string, list(string))": '{"x": ["x"]}',
    "NoneType": "None",
}


def run(ws, build):
    with open(os.path.join(ws, "BUILD.bazel"), "w") as f:
        f.write(build)
    p = subprocess.run(
        [BAZEL, "query", "--noshow_progress", "--output=streamed_jsonproto", "//:x"],
        cwd=ws, capture_output=True, text=True)
    return p.returncode, p.stdout, p.stderr


def literal(args):
    return ", ".join(f"{k} = {v}" for k, v in args.items())


def instantiate(ws, cls):
    args = {}
    mandatory = set()
    for _ in range(12):
        build = f'{cls}(name = "x"{", " if args else ""}{literal(args)})\nprint("KEYS", sorted(existing_rule("x").keys()))\n'
        code, out, err = run(ws, build)
        if code == 0:
            keys = None
            for line in err.splitlines():
                m = re.search(r"KEYS (\[.*\])", line)
                if m:
                    keys = ast.literal_eval(m.group(1))
            return args, out, keys, None, mandatory
        changed = False
        for m in re.finditer(r"missing value for mandatory attribute '([^']+)'", err):
            mandatory.add(m.group(1))
            if m.group(1) not in args:
                args[m.group(1)] = '"x"'
                changed = True
        for m in re.finditer(
            r"expected value of type '([^']+)' for attribute '([^']+)'", err):
            ty, name = m.groups()
            if ty in BY_TYPE and args.get(name) != BY_TYPE[ty]:
                args[name] = BY_TYPE[ty]
                changed = True
        for m in re.finditer(r"attribute '([^']+)'.*?want '([^']+)'", err):
            name, ty = m.groups()
            if ty in BY_TYPE and args.get(name) != BY_TYPE[ty]:
                args[name] = BY_TYPE[ty]
                changed = True
        for attr, value in (("size", '"small"'), ("timeout", '"short"')):
            if f"{attr} '' is not a valid {attr}" in err and attr not in args:
                args[attr] = value
                changed = True
        if "has been removed from Bazel" in err:
            return args, None, None, "removed", mandatory
        if not changed:
            return args, None, None, err.strip()[-600:], mandatory
    return args, None, None, "gave up", mandatory


# A value for an attribute `existing_rule` hides until it is set, by type.
SETTABLE = {
    "STRING": '"x"',
    "BOOLEAN": "True",
    "STRING_LIST": "[]",
    "LABEL_LIST": "[]",
    "LICENSE": '["notice"]',
    "INTEGER": "1",
    "TRISTATE": "1",
    "STRING_DICT": "{}",
    "LABEL_LIST_DICT": "{}",
}


def definition_order(ws, cls, args, attributes):
    """The attributes in the order the class declares them: `existing_rule`
    lists what is set in that order, so set everything that can be set."""
    extra = {a["name"]: SETTABLE[a["type"]] for a in attributes
             if not a["shown"] and a["type"] in SETTABLE and a["name"] not in args}
    while True:
        every = dict(args, **extra)
        build = f'{cls}(name = "x", {literal(every)})\nprint("ORDER", list(existing_rule("x").keys()))\n'
        code, out, err = run(ws, build)
        if code == 0:
            for line in err.splitlines():
                m = re.search(r"ORDER (\[.*\])", line)
                if m:
                    return ast.literal_eval(m.group(1)), sorted(extra)
        bad = [n for n in extra if re.search(rf"attribute '{re.escape(n)}'|'{re.escape(n)}'", err)]
        if not bad:
            return None, sorted(extra)
        for n in bad:
            del extra[n]


def attribute(a, shown):
    name = a["name"]
    ty = a["type"]
    d = {"name": name, "type": ty, "shown": name in shown if shown is not None else None}
    for k in ("stringValue", "intValue", "booleanValue", "stringListValue", "mandatory"):
        if k in a:
            d[k] = a[k]
    return d


def main():
    out = {}
    with tempfile.TemporaryDirectory() as ws:
        with open(os.path.join(ws, "MODULE.bazel"), "w") as f:
            f.write('module(name = "probe")\n')
        for cls in dict.fromkeys(CLASSES):
            args, proto, keys, error, mandatory = instantiate(ws, cls)
            if proto is None:
                out[cls] = {"error": error, "given": args}
                print(f"{cls}: FAILED {error}", file=sys.stderr)
                continue
            rule = json.loads(proto.splitlines()[0])["rule"]
            if rule["ruleClass"] != cls:
                out[cls] = {"error": "macro", "rule_class": rule["ruleClass"]}
                print(f"{cls}: makes a {rule['ruleClass']}", file=sys.stderr)
                continue
            attrs = [attribute(a, keys) for a in rule["attribute"]
                     if not a["name"].startswith(("$", ":"))]
            order, forced = definition_order(ws, cls, args, attrs)
            out[cls] = {"order": order, "forced": forced, "given": args, "mandatory": sorted(mandatory), "attributes": attrs, "keys": keys}
            print(f"{cls}: {len(attrs)} attributes, mandatory {sorted(mandatory)}", file=sys.stderr)
    with open(os.path.join(HERE, "native_rules.json"), "w") as f:
        json.dump(out, f, indent=1, sort_keys=True)


main()
