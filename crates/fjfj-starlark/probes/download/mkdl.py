import json, os, re, sys
def raw(s):
    for n in range(1, 8):
        h = "#" * n
        if ('"' + h) not in s and not s.endswith("\\"): return 'r%s"%s"%s' % (h, s, h)
    return json.dumps(s)
def tempn(s): return re.sub(r"temp\d{6,}", "tempN", s).replace("<ob>/external", "<ext>")
def plain(v):
    body = v[2:] if v.startswith("x ") else v
    ok = all(c in "\n\t\r" or 32 <= ord(c) < 127 for c in body)
    return v if ok else ("x <binary>" if v.startswith("x ") else "<binary>")
SKIP = ["\\r", "file:///etc/hostname", "Error in sorted", "/tmp/fjfj_out.txt", "a.7z", "block=False)\nprint(1)"]
def serve_lit(name, spec):
    if "text" in spec: body = "Serve::Text(%s)" % raw(spec["text"])
    elif "hex" in spec: body = "Serve::Hex(%s)" % raw(spec["hex"])
    elif "status" in spec: body = "Serve::Status(%d)" % spec["status"]
    else:
        items = []
        for path, v in spec["files"].items():
            if isinstance(v, str): items.append('(%s, "f", %s)' % (raw(path), raw(v)))
            elif v.get("dir"): items.append('(%s, "d", "")' % raw(path))
            elif "link" in v: items.append('(%s, "l", %s)' % (raw(path), raw(v["link"])))
            elif "hardlink" in v: items.append('(%s, "h", %s)' % (raw(path), raw(v["hardlink"])))
            else: items.append('(%s, "%s", %s)' % (raw(path), "x" if v.get("exec") else "f", raw(v["text"])))
        body = "Serve::Archive(%s, &[%s])" % (raw(spec["archive"]), ", ".join(items))
    return "(%s, %s)" % (raw(name), body)
def message(log):
    for l in log:
        m = re.match(r"^Error in [\w_]+: (.*)$", l)
        if m: return m.group(1)
    return None
rows, seen = [], set()
for f in sys.argv[1:]:
    for r in json.load(open(f)):
        c = r["case"]
        key = json.dumps([c["bzl"], c.get("serve"), c.get("twice")], sort_keys=True)
        if key in seen: continue
        seen.add(key)
        if any(s in c["bzl"] for s in SKIP) or c.get("flags") or c.get("call"): continue
        if r["rc"] not in (0, 8): continue
        if c.get("twice"): continue
        archive_served = any("archive" in s for s in c.get("serve", {}).values())
        if archive_served and any("integrity = " in p or "sha256 = " in p for p in r["prints"]): continue
        err = None
        if r["rc"] == 8:
            err = message(r["log"])
            if err is None: continue
            err = tempn(err)
            if archive_served and "Checksum was" in err: continue
        body = c["bzl"]
        tree = {tempn(k): plain(tempn(v)) for k, v in r["tree"].items() if k not in ("REPO.bazel",)}
        # the tree of a failed rule is gone
        if r["rc"] == 8: tree = {}
        reqs = []
        for q in r["requests"]:
            if not reqs or reqs[-1] != q: reqs.append(q)
        rows.append("    DlRow {\n        bzl: %s,\n        serve: &[%s],\n        twice: %s,\n        error: %s,\n        printed: &[%s],\n        tree: &[%s],\n        requests: &[%s],\n    }," % (
            raw(body), ", ".join(serve_lit(n, s) for n, s in c.get("serve", {}).items()),
            "true" if c.get("twice") else "false",
            ("Some(%s)" % raw(err)) if err is not None else "None",
            ", ".join(raw(tempn(p)) for p in r["prints"]),
            ", ".join("(%s, %s)" % (raw(k), raw(v)) for k, v in sorted(tree.items())),
            ", ".join(raw(q) for q in reqs)))
out = "//! Generated from Bazel 9.2.0 probes: what repository rules that download, extract and patch do.\n\nuse crate::repo_download_tests::{DlRow, Serve};\n\n"
out += "pub(crate) const DL_ROWS: &[DlRow] = &[\n%s\n];\n" % "\n".join(rows)
open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "../../src/repo_download_matrix.rs"), "w").write(out)
print(len(rows))
