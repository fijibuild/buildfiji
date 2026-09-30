import json, os, re, sys
def raw(s):
    for n in range(1, 8):
        h = "#" * n
        if ('"' + h) not in s and not s.endswith("\\"): return 'r%s"%s"%s' % (h, s, h)
    return json.dumps(s)
def norm(s):
    s = re.sub(r"/tmp/fjfj-repo-probes/ob_xw\d+/modextwd", "<work>", s)
    s = re.sub(r"/tmp/fjfj-repo-probes/ob_xw\d+/external", "<ext>", s)
    s = re.sub(r"/tmp/fjfj-repo-probes/w_ext_w\d+", "<ws>", s)
    return s
SKIP = ['r(s="v")', 'r(name="x", s="one")', '"r", dev_dependency=True)', "\"r\", dev_dependency=True)", "//sub:x.bzl", "--ignore_dev_dependency", "Label('//:x')", "helper.bzl", "t.lab", "print(r.stdout, r.return_code)", "isolate=True", "tag_sort_key", "extension_metadata", "mctx.facts", 'mctx.path(Label', "mctx.os", "mctx.modules)", "mctx.modules[0])", "StarlarkBazelModule", "print(mctx.modules[0].tags)", 'is_dev_dependency(1)', "mctx.watch"]
def message(log):
    for l in log:
        m = re.match(r"^(?:\t*)Error(?: in [\w_]+)?: (.*)$", l)
        if m: return m.group(1)
    for l in log:
        if l.startswith("ERROR: ") and "Traceback" not in l and not l.startswith("ERROR: error evaluating module extension") and not l.startswith("ERROR: Error computing the main repository mapping"):
            return l[7:]
    return None
rows, seen = [], set()
for f in sys.argv[1:]:
    for r in json.load(open(f)):
        c = r["case"]
        key = json.dumps(c, sort_keys=True)
        if key in seen: continue
        seen.add(key)
        body = "\n".join(c["files"].values())
        if any(s in body for s in SKIP) or c.get('flags'): continue
        if r["rc"] not in (0, 8, 48): continue
        ok = r["rc"] == 0
        err = None
        if not ok:
            err = message(r.get("log", []))
            if err is None: continue
            err = norm(err)
        fetch = c.get("fetch") or ["@" + (re.search(r'use_repo\(\s*\w+,\s*(?:\w+\s*=\s*)?"([^"]+)"', c["files"]["MODULE.bazel"]) or [None, "a"])[1]]
        mapping = None
        if r["mapping"]:
            mapping = json.loads(r["mapping"])
        repos = {n: {norm(k): norm(v) for k, v in t.items() if k != "REPO.bazel"} for n, t in r["repos"].items()}
        files = {k: v for k, v in c["files"].items()}
        rows.append("    ExtRow {\n        files: &[%s],\n        fetch: &[%s],\n        error: %s,\n        printed: &[%s],\n        mapping: &[%s],\n        repos: &[%s],\n    }," % (
            ", ".join("(%s, %s)" % (raw(k), raw(v)) for k, v in files.items() if k not in (".bazelversion",)),
            ", ".join(raw(x) for x in fetch),
            ("Some(%s)" % raw(err)) if err is not None else "None",
            ", ".join(raw(norm(re.sub(r"^[\w./]+:\d+:\d+: ", "", p))) for p in r["prints"]),
            ", ".join("(%s, %s)" % (raw(k), raw(v)) for k, v in (mapping or {}).items()),
            ", ".join("(%s, &[%s])" % (raw(n), ", ".join("(%s, %s)" % (raw(k), raw(v)) for k, v in sorted(t.items()))) for n, t in sorted(repos.items()))))
out = "//! Generated from Bazel 9.2.0 probes: what `bazel fetch` made of a module extension.\n\nuse crate::replay_tests::ExtRow;\n\n"
out += "pub(crate) const EXT_ROWS: &[ExtRow] = &[\n%s\n];\n" % "\n".join(rows)
open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "../src/replay_matrix.rs"), "w").write(out)
print(len(rows))
