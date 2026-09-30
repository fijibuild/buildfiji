"""lock_matrix.py CASES1.json OUT1.json [CASES2.json OUT2.json ...]: write
../src/lock_matrix.rs from `multi_runcase.py` runs of lock_cases*.py (cases with
"lock": true, whose output has `lock_ext`: the `moduleExtensions` Bazel wrote)."""
import json, sys
ENV = {"getenv": {"FJFJ_PROBE_A": "1"}, "getenv_value_with_space": {"FJFJ_SP": "a b  c"}}
SKIP = {"override_repo", "watch_tree", "file_in_other_repo"}  # gaps: see the lock section of the design doc
def raw(s):
    n = 1
    while ('"' + "#" * n) in s: n += 1
    return 'r%s"%s"%s' % ("#" * n, s, "#" * n)
rows = []
args = sys.argv[1:]
for cf, of in zip(args[0::2], args[1::2]):
    cases = {c["name"]: c for c in json.load(open(cf))}
    for o in json.load(open(of)):
        c = cases[o["name"]]
        if o["name"] in SKIP or o["exit"] != 0: continue
        expected = json.dumps(o["lock_ext"] or {}, separators=(",", ":"), ensure_ascii=False)
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
    },""" % (raw(o["name"]), registry, ", ".join("(%s, %s)" % (raw(f), raw(t)) for f, t in c["root"].items()),
             ", ".join(raw(f) for f in c["fetch"]), env, raw(expected)))
open("../src/lock_matrix.rs", "w").write("""//! Generated from probes of Bazel 9.2.0 (`bazel build` with `--lockfile_mode=update`):
//! the `moduleExtensions` it wrote; see `lock_tests.rs`.

use crate::lock_tests::LockRow;

pub(crate) const LOCK_ROWS: &[LockRow] = &[
""" + "\n".join(rows) + "\n];\n")
print(len(rows))
