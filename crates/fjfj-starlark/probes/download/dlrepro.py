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
                outer.requests.append(self.path)
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

def run_cases(cases, worker):
    D = f"{here}/w_dl_w{worker}"; OB = f"{here}/ob_dw{worker}"; RC = f"{here}/rc_dw{worker}"
    os.makedirs(D, exist_ok=True)
    env = dict(os.environ, USE_BAZEL_VERSION="9.2.0")
    out = []
    srv = Server()
    for i, c in enumerate(cases):
        for f in os.listdir(D):
            p = f"{D}/{f}"
            shutil.rmtree(p) if os.path.isdir(p) and not os.path.islink(p) else os.remove(p)
        shutil.rmtree(RC, ignore_errors=True)
        srv.files = {name: build(spec) for name, spec in c.get("serve", {}).items()}
        srv.requests = []
        url = f"http://127.0.0.1:{srv.port}"
        def sub(s):
            s = s.replace("@URL@", url)
            s = re.sub(r"@SHA:([\w./-]+)@", lambda m: hashlib.sha256(srv.files[m.group(1)][0]).hexdigest(), s)
            s = re.sub(r"@INT:([\w./-]+)@", lambda m: "sha256-" + base64.b64encode(hashlib.sha256(srv.files[m.group(1)][0]).digest()).decode(), s)
            return s
        name = f"x{worker}_{i}"
        open(f"{D}/.bazelversion", "w").write("9.2.0\n")
        open(f"{D}/BUILD.bazel", "w").write("")
        open(f"{D}/r.bzl", "w").write(sub(c["bzl"]))
        call = sub(c.get("call", ""))
        open(f"{D}/MODULE.bazel", "w").write('module(name="probe")\nr = use_repo_rule("//:r.bzl", "r")\nr(name="%s"%s)\n' % (name, (", " + call) if call else ""))
        for path, text in c.get("files", {}).items():
            os.makedirs(os.path.dirname(f"{D}/{path}") or D, exist_ok=True)
            open(f"{D}/{path}", "w").write(sub(text))
        cmd = [BAZEL, f"--output_base={OB}", "fetch", f"--repo=@{name}", f"--repository_cache={RC}", *c.get("flags", [])]
        p = subprocess.run(cmd, cwd=D, env=env, capture_output=True, text=True, errors="replace")
        if c.get("twice"):
            srv.requests.append("--second--")
            shutil.rmtree(f"{OB}/external/+r+{name}", ignore_errors=True)
            subprocess.run([BAZEL, f"--output_base={OB}", "clean"], cwd=D, env=env, capture_output=True)
            p2 = subprocess.run(cmd, cwd=D, env=env, capture_output=True, text=True, errors="replace")
        text = p.stdout + p.stderr
        scrub = lambda s: s.replace(url, "@URL@").replace(f"{OB}/external/+r+{name}", "<repo>").replace(f"{OB}", "<ob>").replace(RC, "<cache>").replace(D, "<ws>").replace(name, "NAME")
        prints = [scrub(re.sub(r"^\S*/r\.bzl:\d+:\d+: ", "", l[7:])) for l in text.split("\n") if l.startswith("DEBUG: ")]
        log = [scrub(l) for l in text.split("\n") if l.strip() and not l.startswith(("DEBUG: ", "INFO: ", "Computing main repo", "Starting local", "Loading:", "\r", "Fetching")) and "Computing main repo mapping" not in l and "Starting local" not in l]
        tree = {}
        root = f"{OB}/external/+r+{name}"
        if os.path.isdir(root):
            for d, dirs, files in os.walk(root):
                for n in dirs + files:
                    fp = f"{d}/{n}"; rel = os.path.relpath(fp, root)
                    st = os.lstat(fp)
                    if stat.S_ISLNK(st.st_mode): tree[rel] = "-> " + scrub(os.readlink(fp))
                    elif stat.S_ISDIR(st.st_mode): tree[rel] = "<dir>"
                    else:
                        try: body = open(fp, errors="replace").read()
                        except Exception: body = "<unreadable>"
                        tree[rel] = ("x " if st.st_mode & 0o111 else "") + body
        cache = sorted(os.path.relpath(os.path.join(d, f), RC) for d, _, fs in os.walk(RC) for f in fs) if os.path.isdir(RC) else []
        rec = {"case": c, "rc": p.returncode, "prints": prints, "log": log, "tree": tree, "requests": [scrub(r) for r in srv.requests], "cache": [scrub(x) for x in cache]}
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
