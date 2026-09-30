import json, re, sys
cases = json.load(open("multi_cases.json")); outs = {o["name"]: o for o in json.load(open("multi_out.json"))}
def raw(s):
    n = 0
    while ('"' + "#" * n) in s: n += 1
    return 'r%s"%s"%s' % ("#" * (n + 1) if n else "#", s, "#" * (n + 1) if n else "#") if False else 'r%s"%s"%s' % ("#" * (n+1), s, "#" * (n+1))
def norm(s, reg):
    s = s.replace("file://" + reg, "<reg>")
    return re.sub(r"/tmp/lfe\w+/ws/", "", s)
rows = []
for c in cases:
    o = outs[c["name"]]
    reg = o["reg"]
    registry = ", ".join(
        "(%s, %s, &[%s])" % (raw(m), raw(v), ", ".join("(%s, %s)" % (raw(f), raw(t)) for f, t in files.items()))
        for m, vs in c["registry"].items() for v, files in vs.items())
    error = None
    for e in o["errors"]:
        if e != "Build did NOT complete successfully" and not e.startswith("Error computing"):
            error = norm(e, reg); break
    rows.append(f"""    MultiRow {{
        name: {raw(c["name"])},
        registry: &[{registry}],
        root: &[{", ".join("(%s, %s)" % (raw(f), raw(t)) for f, t in c["root"].items())}],
        fetch: &[{", ".join(raw(f) for f in c["fetch"])}],
        error: {"Some(" + raw(error) + ")" if error else "None"},
        printed: &[{", ".join(raw(p) for p in o["prints"])}],
        builds: &[{", ".join("(%s, %s)" % (raw(k), raw(v)) for k, v in sorted(o["builds"].items()))}],
    }},""")
open("../src/multi_matrix.rs", "w").write(
"""//! Generated from probes of Bazel 9.2.0 (`bazel build` against a local registry
//! of local-path modules); see `multi_tests.rs`.

use crate::multi_tests::MultiRow;

pub(crate) const MULTI_ROWS: &[MultiRow] = &[
""" + "\n".join(rows) + "\n];\n")
print(len(rows))
