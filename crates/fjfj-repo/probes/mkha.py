import json, re, sys
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
def serve_lit(name, spec):
    if "text" in spec: body = "Serve::Text(%s)" % raw(spec["text"])
    elif "status" in spec: body = "Serve::Status(%d)" % spec["status"]
    else:
        items = []
        for path, v in spec["files"].items():
            if isinstance(v, str): items.append('(%s, "f", %s)' % (raw(path), raw(v)))
            elif v.get("dir"): items.append('(%s, "d", "")' % raw(path))
            elif "link" in v: items.append('(%s, "l", %s)' % (raw(path), raw(v["link"])))
            else: items.append('(%s, "%s", %s)' % (raw(path), "x" if v.get("exec") else "f", raw(v["text"])))
        body = "Serve::Archive(%s, &[%s])" % (raw(spec["archive"]), ", ".join(items))
    return "(%s, %s)" % (raw(name), body)
SHA40 = re.compile(r"[0-9a-f]{40}")
def commits(s): return SHA40.sub("<commit>", s)
def git_lit(git):
    out = []
    for name, spec in git.items():
        commits_lit = ", ".join("&[%s]" % ", ".join("(%s, %s)" % (raw(p), raw(t)) for p, t in files.items()) for files in spec["commits"])
        tags = ", ".join("(%s, %s)" % (k, raw(v)) for k, v in spec.get("tags", {}).items())
        out.append("(%s, &[%s], &[%s])" % (raw(name), commits_lit, tags))
    return ", ".join(out)
def message(log):
    for l in log:
        m = re.match(r"^Error in [\w_]+: (.*)$", l)
        if m: return m.group(1)
    return None
SKIP = ["verbose=True", "bogus=1", 'urls="@URL@'] 
rows, seen = [], set()
for f in sys.argv[1:]:
    for r in json.load(open(f)):
        c = r["case"]
        key = json.dumps([c["module"], c.get("serve"), c.get("files"), c.get("flags")], sort_keys=True)
        if key in seen: continue
        seen.add(key)
        if any(s in c["module"] for s in SKIP): continue
        if any(not f.startswith("--credential_helper=") for f in c.get("flags", [])): continue
        if r["rc"] not in (0, 2, 8): continue
        if any("lockFileVersion" in str(v) for v in r["tree"].values()): continue
        archive_served = any("archive" in s for s in c.get("serve", {}).values())
        err = None
        if r["rc"] == 2:
            first = next((l for l in r["log"] if l.startswith("ERROR: ")), None)
            if first is None or "on PATH" in first: continue
            err = first[7:]
        if r["rc"] == 8:
            err = message(r["log"])
            if err is None: continue
            err = commits(tempn(err))
            if archive_served and "Checksum was" in err: continue
        tree = {tempn(k): plain(commits(tempn(v))) for k, v in r["tree"].items() if not k.endswith("REPO.bazel")} if r["rc"] == 0 else {}
        if r["rc"] == 0 and not tree: continue
        helper_log = [l for l in r.get("helper_log", "").strip().split("\n") if l]
        reqs = []
        for q in r["requests"]:
            if not reqs or reqs[-1] != q: reqs.append(q)
        rows.append("    HaRow {\n        module: %s,\n        flags: &[%s],\n        helper_log: &[%s],\n        git: &[%s],\n        files: &[%s],\n        serve: &[%s],\n        fetch: &[%s],\n        error: %s,\n        printed: &[%s],\n        tree: &[%s],\n        requests: &[%s],\n    }," % (
            raw(c["module"]), ", ".join(raw(f) for f in c.get("flags", [])), ", ".join(raw(l) for l in helper_log), git_lit(c.get("git", {})), ", ".join("(%s, %s)" % (raw(k), raw(v["text"] if isinstance(v, dict) else v)) for k, v in c.get("files", {}).items()),
            ", ".join(serve_lit(n, s) for n, s in c.get("serve", {}).items()),
            ", ".join(raw(x) for x in c.get("fetch", ["@x"])),
            ("Some(%s)" % raw(err)) if err is not None else "None",
            ", ".join(raw(commits(tempn(p))) for p in (re.sub(r"^\S*\.bzl:\d+:\d+: ", "", q) for q in r["prints"]) if not p.startswith("Repo ")),
            ", ".join("(%s, %s)" % (raw(k), raw(v)) for k, v in sorted(tree.items())),
            ", ".join(raw(q) for q in reqs)))
out = "//! Generated from Bazel 9.2.0 probes: what Bazel's own `http_archive` and `http_file` do.\n\nuse crate::http_archive_tests::{HaRow, Serve};\n\n"
out += "pub(crate) const HA_ROWS: &[HaRow] = &[\n%s\n];\n" % "\n".join(rows)
open("/home/nathan/buildfiji/crates/fjfj-repo/src/http_archive_matrix.rs", "w").write(out)
print(len(rows))
