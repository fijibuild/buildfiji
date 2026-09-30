import json, os, re, sys
def raw(s):
    for n in range(1, 8):
        h = "#" * n
        if ('"' + h) not in s and not s.endswith("\\"): return 'r%s"%s"%s' % (h, s, h)
    return json.dumps(s)
def norm(s):
    s = re.sub(r"/tmp/claude-1000/[^ \"']*?/p/ob_w\d+/external/\+r\+x\d+_\d+", "<repo>", s)
    s = re.sub(r"/tmp/claude-1000/[^ \"']*?/p/ob_w\d+/external", "<ext>", s)
    s = re.sub(r"/tmp/claude-1000/[^ \"']*?/p/w_repo_w\d+", "<ws>", s)
    s = re.sub(r"x\d+_\d+", "NAME", s)
    return s
SKIP = ["é", 'fail("boom", attr', "f1.txt.copy", "return_value", "\\x00", 'print(ctx.read("o"), ctx.read("o2"))', 'ctx.execute(["x.sh"])', "print(ctx.file)", "mandatory=True", 'ctx.which("sh")', "ctx.os.name", "readdir()", "workspace_root.basename", "timeout=0", "timeout=-1", '["env"]', "HOME", "readdir"]
rows, seen = [], set()
for f in sys.argv[1:]:
    for r in json.load(open(f)):
        c = r["case"]
        if c.get("call") or r["rc"] not in (0, 8): continue
        body = c["bzl"]
        if any(s in body for s in SKIP): continue
        if body in seen: continue
        seen.add(body)
        ok = r["rc"] == 0
        err = None
        if not ok:
            errs = r["errs"]
            if not errs: continue
            err = norm(errs[-1] if len(errs) > 1 else errs[0])
            err = re.sub(r"^r\.bzl:\d+:\d+: ", "", err)
        prints = [norm(p) for p in r["prints"]]
        tree = {norm(k): norm(v) for k, v in r["tree"].items() if k != "REPO.bazel"}
        files = c.get("files", {})
        rows.append("    RepoRow {\n        bzl: %s,\n        files: &[%s],\n        error: %s,\n        printed: &[%s],\n        tree: &[%s],\n    }," % (
            raw(body), ", ".join("(%s, %s)" % (raw(k), raw(v)) for k, v in files.items()),
            ("Some(%s)" % raw(err)) if err is not None else "None",
            ", ".join(raw(p) for p in prints),
            ", ".join("(%s, %s)" % (raw(k), raw(v)) for k, v in sorted(tree.items()))))
out = "//! Generated from Bazel 9.2.0 probes: what repository rules do to their repository.\n\nuse crate::repo_ctx_tests::RepoRow;\n\n"
out += "pub(crate) const REPO_CASES: &[RepoRow] = &[\n%s\n];\n" % "\n".join(rows)
open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "../../src/repo_ctx_matrix.rs"), "w").write(out)
print(len(rows))
