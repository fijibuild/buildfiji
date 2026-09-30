#!/usr/bin/env python3
"""xrepro.py CASES.json OUT.json [N] -- run module extensions in real Bazel.
A case is {"files": {path: text}, "fetch": ["@a"]  (repos to fetch; default: the first use_repo name), "flags": [...]}.
Records DEBUG prints, ERROR lines, the exit code, the root repo mapping, and the tree of every repo that got made."""
import json, os, re, stat, subprocess, sys, shutil
here = os.environ.get("PROBE_SCRATCH") or "/tmp/fjfj-repo-probes"
os.makedirs(here, exist_ok=True)
BAZEL = os.environ.get("BAZEL", "bazel")
def tree_of(root):
    tree = {}
    for d, dirs, files in os.walk(root):
        for n in dirs + files:
            fp = f"{d}/{n}"; rel = os.path.relpath(fp, root)
            st = os.lstat(fp)
            if stat.S_ISLNK(st.st_mode): tree[rel] = "-> " + os.readlink(fp)
            elif stat.S_ISDIR(st.st_mode): tree[rel] = "<dir>"
            else:
                try: body = open(fp, errors="replace").read()
                except Exception: body = "<unreadable>"
                tree[rel] = ("x " if st.st_mode & 0o111 else "") + body
    return tree
def run_cases(cases, worker):
    D = f"{here}/w_ext_w{worker}"; OB = f"{here}/ob_xw{worker}"
    os.makedirs(D, exist_ok=True)
    env = dict(os.environ, USE_BAZEL_VERSION="9.2.0")
    out = []
    for i, c in enumerate(cases):
        for f in os.listdir(D):
            p = f"{D}/{f}"
            shutil.rmtree(p) if os.path.isdir(p) and not os.path.islink(p) else os.remove(p)
        files = {".bazelversion": "9.2.0\n", "BUILD.bazel": ""}
        files.update(c["files"])
        for path, text in files.items():
            os.makedirs(os.path.dirname(f"{D}/{path}") or D, exist_ok=True)
            open(f"{D}/{path}", "w").write(text)
        targets = c.get("fetch") or ["@" + (re.search(r'use_repo\(\s*\w+,\s*"([^"]+)"', files["MODULE.bazel"]) or [None, "a"])[1]]
        cmd = [BAZEL, f"--output_base={OB}", "fetch", *[f"--repo={t}" for t in targets], *c.get("flags", [])]
        subprocess.run([BAZEL, f"--output_base={OB}", "clean", "--expunge"], cwd=D, env=env, capture_output=True)
        p = subprocess.run(cmd, cwd=D, env=env, capture_output=True, text=True, errors="replace")
        text = p.stdout + p.stderr
        clean = lambda s: re.sub(r"\S*/w_ext_w\d+/", "", re.sub(r"\S*/ob_xw\d+/external", "<ext>", s))
        prints = [clean(re.sub(r"^\S*?([\w.]+\.bzl):(\d+):(\d+): ", r"\1:\2:\3: ", l[7:])) for l in text.split("\n") if l.startswith("DEBUG: ")]
        errs = [clean(l[7:]) for l in text.split("\n") if l.startswith("ERROR: ")]
        log = [clean(l) for l in text.split("\n") if l.strip() and not l.startswith(("DEBUG: ", "INFO: ", "Computing main repo", "Starting local", "Loading:", "\r")) and "Computing main repo mapping" not in l]
        repos = {}
        ext = f"{OB}/external"
        if os.path.isdir(ext):
            for n in sorted(os.listdir(ext)):
                if n.startswith("@") or n in ("bazel_tools", "_main") or n.startswith("bazel_tools"): continue
                if os.path.isdir(f"{ext}/{n}") and not os.path.islink(f"{ext}/{n}"):
                    repos[n] = tree_of(f"{ext}/{n}")
        mapping = None
        if p.returncode == 0:
            m = subprocess.run([BAZEL, f"--output_base={OB}", "mod", "dump_repo_mapping", ""], cwd=D, env=env, capture_output=True, text=True)
            mapping = (m.stdout or "").strip().split("\n")[-1] if m.returncode == 0 else None
        rec = {"case": c, "rc": p.returncode, "prints": prints, "errs": errs, "log": log, "repos": repos, "mapping": mapping}
        out.append(rec)
        print(p.returncode, prints[:2], errs[:2], list(repos)[:4], flush=True)
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
