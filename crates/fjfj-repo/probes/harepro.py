#!/usr/bin/env python3
"""dlrepro.py CASES.json OUT.json [N] -- repository rules that download, in real Bazel, against a local HTTP server.
A case is {"bzl": r.bzl text (defines r), "call": kwargs text, "files": {ws path: text}, "serve": {name: spec}, "flags": [...], "cache": bool}.
Placeholders in bzl/call: @URL@ (http://127.0.0.1:PORT), @SHA:name@ (hex sha256 of a served file), @INT:name@ (sha256-<base64>).
A served spec is {"text": s} | {"hex": h} | {"archive": kind, "files": {path: str | {"text":..,"exec":bool,"link":target,"dir":true}}} | {"status": 404}."""
import base64, hashlib, http.server, io, json, os, re, shutil, stat, subprocess, sys, tarfile, threading, time, zipfile
here = os.environ.get("PROBE_SCRATCH") or "/tmp/fjfj-repo-probes"
os.makedirs(here, exist_ok=True)
BAZEL = os.environ.get("BAZEL", "bazel")

def tar_bytes(files, mode):
    bio = io.BytesIO()
    with tarfile.open(fileobj=bio, mode="w") as t:
        for path, spec in files.items():
            spec = {"text": spec} if isinstance(spec, str) else spec
            ti = tarfile.TarInfo(path)
            ti.mtime = 1700000000
            if spec.get("dir"):
                ti.type = tarfile.DIRTYPE; ti.mode = 0o755; t.addfile(ti)
            elif "link" in spec:
                ti.type = tarfile.SYMTYPE; ti.linkname = spec["link"]; ti.mode = 0o777; t.addfile(ti)
            elif "hardlink" in spec:
                ti.type = tarfile.LNKTYPE; ti.linkname = spec["hardlink"]; t.addfile(ti)
            else:
                data = spec["text"].encode()
                ti.size = len(data); ti.mode = 0o755 if spec.get("exec") else 0o644
                t.addfile(ti, io.BytesIO(data))
    raw = bio.getvalue()
    if mode == "tar": return raw
    if mode == "tar.gz": return subprocess.run(["gzip", "-n", "-c"], input=raw, capture_output=True).stdout
    if mode == "tar.bz2": return subprocess.run(["bzip2", "-c"], input=raw, capture_output=True).stdout
    if mode == "tar.xz": return subprocess.run(["xz", "-c"], input=raw, capture_output=True).stdout
    if mode == "tar.zst": return subprocess.run(["zstd", "-q", "-c"], input=raw, capture_output=True).stdout
    raise ValueError(mode)

def zip_bytes(files):
    bio = io.BytesIO()
    with zipfile.ZipFile(bio, "w", zipfile.ZIP_DEFLATED) as z:
        for path, spec in files.items():
            spec = {"text": spec} if isinstance(spec, str) else spec
            zi = zipfile.ZipInfo(path, (2023, 1, 1, 0, 0, 0))
            if spec.get("dir"):
                zi = zipfile.ZipInfo(path.rstrip("/") + "/", (2023, 1, 1, 0, 0, 0)); zi.external_attr = (0o40755 << 16)
                z.writestr(zi, b"")
            elif "link" in spec:
                zi.external_attr = (0o120777 << 16); z.writestr(zi, spec["link"])
            else:
                zi.external_attr = ((0o100755 if spec.get("exec") else 0o100644) << 16)
                z.writestr(zi, spec["text"])
    return bio.getvalue()

def ar_bytes(members):
    out = b"!<arch>\n"
    for name, data in members:
        out += ("%-16s%-12d%-6d%-6d%-8s%-10d`\n" % (name + "/", 0, 0, 0, "100644", len(data))).encode() + data
        if len(data) % 2: out += b"\n"
    return out

def build(spec):
    if "text" in spec: return spec["text"].encode(), 200
    if "hex" in spec: return bytes.fromhex(spec["hex"]), 200
    if "status" in spec: return b"not here", spec["status"]
    kind = spec["archive"]
    if kind in ("zip", "whl", "jar", "war", "aar", "nupkg"): return zip_bytes(spec["files"]), 200
    if kind.startswith("tar"): return tar_bytes(spec["files"], kind), 200
    if kind.startswith("deb"):
        comp = kind.split(":")[1] if ":" in kind else "tar.xz"
        data = tar_bytes(spec["files"], comp)
        return ar_bytes([("debian-binary", b"2.0\n"), ("control.tar.gz", tar_bytes({"control": "x"}, "tar.gz")), ("data." + comp, data)]), 200
    raise ValueError(kind)

class Server:
    def __init__(self):
        self.files = {}; self.requests = []
        outer = self
        class H(http.server.BaseHTTPRequestHandler):
            def do_GET(self):
                outer.requests.append(self.path + ((" auth=" + self.headers["Authorization"]) if self.headers.get("Authorization") else "") + ((" cookie=" + self.headers["Cookie"]) if self.headers.get("Cookie") else ""))
                name = self.path.lstrip("/").split("?")[0]
                if name in outer.files:
                    data, status = outer.files[name]
                    self.send_response(status); self.send_header("Content-Length", str(len(data))); self.end_headers(); self.wfile.write(data)
                else:
                    self.send_response(404); self.send_header("Content-Length", "0"); self.end_headers()
            def log_message(self, *a): pass
        self.httpd = http.server.ThreadingHTTPServer(("127.0.0.1", 0), H)
        self.port = self.httpd.server_address[1]
        threading.Thread(target=self.httpd.serve_forever, daemon=True).start()
    def stop(self): self.httpd.shutdown()

def make_git(root, name, spec):
    d = f"{root}/git_{name}"
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d, exist_ok=True)
    env = dict(os.environ, GIT_AUTHOR_NAME="a", GIT_AUTHOR_EMAIL="a@x", GIT_COMMITTER_NAME="a", GIT_COMMITTER_EMAIL="a@x", GIT_AUTHOR_DATE="2023-01-01T00:00:00Z", GIT_COMMITTER_DATE="2023-01-01T00:00:00Z")
    run = lambda *a: subprocess.run(["git", *a], cwd=d, env=env, capture_output=True, text=True, check=True).stdout.strip()
    run("init", "-q", "-b", "main")
    commits = []
    for i, files in enumerate(spec["commits"]):
        for path, text in files.items():
            os.makedirs(os.path.dirname(f"{d}/{path}") or d, exist_ok=True)
            open(f"{d}/{path}", "w").write(text)
        run("add", "-A"); run("commit", "-q", "-m", f"c{i}")
        commits.append(run("rev-parse", "HEAD"))
        if spec.get("tags", {}).get(str(i)): run("tag", spec["tags"][str(i)])
    return d, commits

def run_cases(cases, worker):
    D = f"{here}/w_ha_w{worker}"; OB = f"{here}/ob_hw{worker}"; RC = f"{here}/rc_hw{worker}"
    os.makedirs(D, exist_ok=True)
    env = dict(os.environ, USE_BAZEL_VERSION="9.2.0")
    out = []
    srv = Server()
    for i, c in enumerate(cases):
        for f in os.listdir(D):
            p = f"{D}/{f}"
            shutil.rmtree(p) if os.path.isdir(p) and not os.path.islink(p) else os.remove(p)
        shutil.rmtree(RC, ignore_errors=True)
        ext0 = f"{OB}/external"
        if os.path.isdir(ext0):
            for n0 in os.listdir(ext0):
                if n0.startswith("+"):
                    p0 = f"{ext0}/{n0}"
                    shutil.rmtree(p0) if os.path.isdir(p0) and not os.path.islink(p0) else os.remove(p0)
        srv.files = {name: build(spec) for name, spec in c.get("serve", {}).items()}
        gits = {n: make_git(f"{here}/gitw{worker}", n, s) for n, s in c.get("git", {}).items()}
        srv.requests = []
        url = f"http://127.0.0.1:{srv.port}"
        def sub(s):
            s = s.replace("@URL@", url).replace("@WS@", D)
            s = re.sub(r"@SHA:([\w./-]+)@", lambda m: hashlib.sha256(srv.files[m.group(1)][0]).hexdigest(), s)
            s = re.sub(r"@GIT:(\w+)@", lambda m: gits[m.group(1)][0], s)
            s = re.sub(r"@COMMIT(\d*):(\w+)@", lambda m: gits[m.group(2)][1][int(m.group(1) or len(gits[m.group(2)][1])) - 1], s)
            s = re.sub(r"@INT:([\w./-]+)@", lambda m: "sha256-" + base64.b64encode(hashlib.sha256(srv.files[m.group(1)][0]).digest()).decode(), s)
            return s
        if c.get("fresh"):
            subprocess.run([BAZEL, f"--output_base={OB}", "shutdown"], cwd=D, env=env, capture_output=True)
        name = f"x{worker}_{i}"
        open(f"{D}/.bazelversion", "w").write("9.2.0\n")
        open(f"{D}/BUILD.bazel", "w").write("")
        open(f"{D}/MODULE.bazel", "w").write(sub(c["module"]))
        for path, text in c.get("files", {}).items():
            os.makedirs(os.path.dirname(f"{D}/{path}") or D, exist_ok=True)
            if isinstance(text, dict) and "serve" in text:
                open(f"{D}/{path}", "wb").write(srv.files[text["serve"]][0])
                continue
            body = text["text"] if isinstance(text, dict) else text
            open(f"{D}/{path}", "w").write(sub(body))
            if isinstance(text, dict) and text.get("exec"): os.chmod(f"{D}/{path}", 0o755)
        cmd = [BAZEL, f"--output_base={OB}", "fetch", *[f"--repo={x}" for x in c.get("fetch", ["@x"])], f"--repository_cache={RC}", *[sub(f) for f in c.get("flags", [])]]
        p = subprocess.run(cmd, cwd=D, env=env, capture_output=True, text=True, errors="replace")
        if c.get("twice"):
            srv.requests.append("--second--")
            shutil.rmtree(f"{OB}/external/+r+{name}", ignore_errors=True)
            subprocess.run([BAZEL, f"--output_base={OB}", "clean"], cwd=D, env=env, capture_output=True)
            p2 = subprocess.run(cmd, cwd=D, env=env, capture_output=True, text=True, errors="replace")
        if c.get("then"):
            srv.requests.append("--then--")
            open(f"{D}/MODULE.bazel", "w").write(sub(c["then"]["module"]))
            cmd2 = [BAZEL, f"--output_base={OB}", "fetch", *[f"--repo={x}" for x in c["then"].get("fetch", ["@y"])], f"--repository_cache={RC}", *[sub(f) for f in c["then"].get("flags", c.get("flags", []))]]
            p2 = subprocess.run(cmd2, cwd=D, env=env, capture_output=True, text=True, errors="replace")
            p = type("R", (), {"stdout": p.stdout + p2.stdout, "stderr": p.stderr + p2.stderr, "returncode": p2.returncode})()
        text = p.stdout + p.stderr
        scrub = lambda s: s.replace(url, "@URL@").replace(f"{OB}", "<ob>").replace(RC, "<cache>").replace(D, "<ws>").replace(name, "NAME")
        prints = [scrub(re.sub(r"^\S*\.bzl:\d+:\d+: ", "", l[7:])) for l in text.split("\n") if l.startswith("DEBUG: ")]
        log = [scrub(l) for l in text.split("\n") if l.strip() and not l.startswith(("DEBUG: ", "INFO: ", "Computing main repo", "Starting local", "Loading:", "\r", "Fetching")) and "Computing main repo mapping" not in l and "Starting local" not in l]
        tree = {}
        ext = f"{OB}/external"
        if os.path.isdir(ext):
            for n in sorted(os.listdir(ext)):
                if not n.startswith(("+http_archive+", "+http_file+", "+local_repository+", "+new_local_repository+", "+git_repository+", "+new_git_repository+")) or not os.path.isdir(f"{ext}/{n}"): continue
                root = os.path.realpath(f"{ext}/{n}")
                for d, dirs, files in os.walk(root):
                    for m in dirs + files:
                        fp = f"{d}/{m}"; rel = n + "/" + os.path.relpath(fp, root)
                        st = os.lstat(fp)
                        if stat.S_ISLNK(st.st_mode): tree[rel] = "-> " + scrub(os.readlink(fp))
                        elif stat.S_ISDIR(st.st_mode): tree[rel] = "<dir>"
                        else:
                            try: body = open(fp, errors="replace").read()
                            except Exception: body = "<unreadable>"
                            tree[rel] = ("x " if st.st_mode & 0o111 else "") + body
        cache = sorted(os.path.relpath(os.path.join(d, f), RC) for d, _, fs in os.walk(RC) for f in fs) if os.path.isdir(RC) else []
        helper_log = open(f"{D}/helper.log").read() if os.path.exists(f"{D}/helper.log") else ""
        entry = {}
        for n0, (b0, _) in srv.files.items():
            hexd = hashlib.sha256(b0).hexdigest()
            d0 = f"{RC}/content_addressable/sha256/{hexd}"
            if os.path.isdir(d0): entry[n0] = sorted(os.listdir(d0))
        rec = {"cache_entry": entry, "helper_log": scrub(helper_log), "case": c, "rc": p.returncode, "prints": prints, "log": log, "tree": tree, "requests": [scrub(r) for r in srv.requests], "cache": [scrub(x) for x in cache]}
        out.append(rec)
        print(p.returncode, prints[:2], log[:1], list(tree)[:4], rec["requests"], flush=True)
    srv.stop()
    subprocess.run([BAZEL, f"--output_base={OB}", "shutdown"], cwd=D, env=env, capture_output=True)
    return out
if __name__ == "__main__":
    cases = json.load(open(sys.argv[1])); n = int(sys.argv[3]) if len(sys.argv) > 3 else 1
    if n == 1: res = run_cases(cases, 0)
    else:
        import concurrent.futures as cf
        chunks = [cases[i::n] for i in range(n)]
        with cf.ThreadPoolExecutor(n) as ex:
            parts = list(ex.map(lambda a: run_cases(a[1], a[0]), enumerate(chunks)))
        res = [None] * len(cases)
        for k, part in enumerate(parts):
            for j, r in enumerate(part): res[j * n + k] = r
    json.dump(res, open(sys.argv[2], "w"), indent=1)
