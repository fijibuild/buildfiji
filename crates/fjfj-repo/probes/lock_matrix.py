"""lock_matrix.py CASES1.json OUT1.json [CASES2.json OUT2.json ...]: write
../src/lock_matrix.rs from `multi_runcase.py` runs of lock_cases*.py (cases with
"lock": true, whose output has `lock_ext`: the `moduleExtensions` Bazel wrote)."""
import json, re, sys
ENV = {"getenv": {"FJFJ_PROBE_A": "1"}, "getenv_value_with_space": {"FJFJ_SP": "a b  c"}}
SKIP = {"override_repo", "watch_tree", "file_in_other_repo"}  # gaps: see the lock section of the design doc
def raw(s):
    n = 1
    while ('"' + "#" * n) in s: n += 1
    return 'r%s"%s"%s' % ("#" * n, s, "#" * n)
rows = []
seen = set()
args = sys.argv[1:]
for cf, of in zip(args[0::2], args[1::2]):
    cases = {c["name"]: c for c in json.load(open(cf))}
    for o in json.load(open(of)):
        c = cases[o["name"]]
        if o["name"] in SKIP or o["name"] in seen: continue
        seen.add(o["name"])
        expected = json.dumps(o["lock_ext"] or {}, separators=(",", ":"), ensure_ascii=False)
        facts = json.dumps((o.get("lock_facts") or {}).get("facts") or {}, separators=(",", ":"), ensure_ascii=False)
        clean = lambda s: re.sub(r"/tmp/lfe\w+/ws/", "", s)
        warnings = [clean(w) for w in re.findall(r"^WARNING: (.*?bazel mod tidy'\.)", o.get("log", ""), re.S | re.M)]
        error = None
        if o["exit"] != 0:
            m = re.search(r"^Error in \w+: (.*)$", o.get("log", ""), re.M)
            error = m.group(1) if m else next((e for e in o["errors"] if not e.startswith(("error evaluating", "Build did"))), None)
            if error is None: continue
            error = clean(error)
        registry = ", ".join(
            "(%s, %s, &[%s])" % (raw(m), raw(v), ", ".join("(%s, %s)" % (raw(f), raw(t)) for f, t in files.items()))
            for m, vs in c["registry"].items() for v, files in vs.items())
        env = ", ".join("(%s, %s)" % (raw(k), raw(v)) for k, v in ENV.get(o["name"], {}).items())
        rows.append("""    LockRow {
        name: %s,
        registry: &[%s],
        root: &[%s],
        fetch: &[%s],
        env: &[%s],
        expected: %s,
        facts: %s,
        warnings: &[%s],
        error: %s,
    },""" % (raw(o["name"]), registry, ", ".join("(%s, %s)" % (raw(f), raw(t)) for f, t in c["root"].items()),
             ", ".join(raw(f) for f in c["fetch"]), env, raw(expected), raw(facts),
             ", ".join(raw(w) for w in warnings), ("Some(%s)" % raw(error)) if error else "None"))
open("../src/lock_matrix.rs", "w").write("""//! Generated from probes of Bazel 9.2.0 (`bazel build` with `--lockfile_mode=update`):
//! the `moduleExtensions` it wrote; see `lock_tests.rs`.

use crate::lock_tests::LockRow;

pub(crate) const LOCK_ROWS: &[LockRow] = &[
""" + "\n".join(rows) + "\n];\n")
print(len(rows))
