#!/usr/bin/env python3
"""repro.py CASES.json OUT.json [N] -- run repository rules in real Bazel.
A case is {"bzl": text of r.bzl (defines r), "call": kwargs text for r(name=..., <call>), "files": {path: text}, "flags": [..]}.
Records DEBUG prints, ERROR lines, the return code and the tree the repo ended up as (path -> content | "-> target" for symlinks, with mode bits)."""
import json, os, re, stat, subprocess, sys, shutil
here = os.environ.get("PROBE_SCRATCH") or "/tmp/fjfj-repo-probes"
os.makedirs(here, exist_ok=True)
def run_cases(cases, worker):
    D = f"{here}/w_repo_w{worker}"; OB = f"{here}/ob_w{worker}"
    os.makedirs(D, exist_ok=True)
    env = dict(os.environ, USE_BAZEL_VERSION="9.2.0")
    out = []
    for i, c in enumerate(cases):
        for f in os.listdir(D):
            p = f"{D}/{f}"
            shutil.rmtree(p) if os.path.isdir(p) and not os.path.islink(p) else os.remove(p)
        name = f"x{worker}_{i}"
        open(f"{D}/.bazelversion", "w").write("9.2.0\n")
        open(f"{D}/BUILD.bazel", "w").write("")
        open(f"{D}/r.bzl", "w").write(c["bzl"])
        open(f"{D}/MODULE.bazel", "w").write('module(name="probe")\nr = use_repo_rule("//:r.bzl", "r")\nr(name="%s"%s)\n' % (name, (", " + c["call"]) if c.get("call") else ""))
        for path, text in c.get("files", {}).items():
            os.makedirs(os.path.dirname(f"{D}/{path}") or D, exist_ok=True)
            open(f"{D}/{path}", "w").write(text)
        p = subprocess.run([os.environ.get("BAZEL", "bazel"), f"--output_base={OB}", "fetch", f"--repo=@{name}", *c.get("flags", [])], cwd=D, env=env, capture_output=True, text=True, errors="replace")
        text = p.stdout + p.stderr
        prints = [re.sub(r"^\S*/r\.bzl:\d+:\d+: ", "", l[7:]) for l in text.split("\n") if l.startswith("DEBUG: ")]
        errs = [re.sub(r"\S*/w_repo_w\d+/", "", l[7:]) for l in text.split("\n") if l.startswith("ERROR: ")]
        tree = {}
        root = f"{OB}/external/+r+{name}"
        if os.path.isdir(root):
            for d, dirs, files in os.walk(root):
                for n in dirs + files:
                    fp = f"{d}/{n}"; rel = os.path.relpath(fp, root)
                    st = os.lstat(fp)
                    if stat.S_ISLNK(st.st_mode): tree[rel] = "-> " + os.readlink(fp).replace(D, "<ws>")
                    elif stat.S_ISDIR(st.st_mode): tree[rel] = "<dir>"
                    else:
                        try: body = open(fp, errors="replace").read()
                        except Exception as e: body = "<unreadable>"
                        tree[rel] = ("x " if st.st_mode & 0o111 else "") + body
        rec = {"case": c, "rc": p.returncode, "prints": prints, "errs": errs, "tree": tree}
        out.append(rec)
        print(p.returncode, prints[:3], errs[:2], list(tree)[:4], flush=True)
    subprocess.run([os.environ.get("BAZEL", "bazel"), f"--output_base={OB}", "shutdown"], cwd=D, env=env, capture_output=True)
    return out
if __name__ == "__main__":
    cases = json.load(open(sys.argv[1])); n = int(sys.argv[3]) if len(sys.argv) > 3 else 1
    if n == 1:
        res = run_cases(cases, 0)
    else:
        import concurrent.futures as cf
        chunks = [cases[i::n] for i in range(n)]
        with cf.ThreadPoolExecutor(n) as ex:
            parts = list(ex.map(lambda a: run_cases(a[1], a[0]), enumerate(chunks)))
        res = [None] * len(cases)
        for k, part in enumerate(parts):
            for j, r in enumerate(part): res[j * n + k] = r
    json.dump(res, open(sys.argv[2], "w"), indent=1)
