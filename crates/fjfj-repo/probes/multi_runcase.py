"""runcase.py CASES.json OUT.json: run each case in real Bazel 9.2.0.

A case: {"name", "registry": {module: {version: {file: text}}}, "root": {file: text},
"targets": ["@x//:f"]}. Output per case: the prints (text after `file:line:col: `),
the errors, and the BUILD.bazel of every repository under external/ that the
case's targets fetched (name -> text)."""
import json, os, re, shutil, subprocess, sys, tempfile

def w(p, t):
    os.makedirs(os.path.dirname(p), exist_ok=True)
    open(p, "w").write(t)

def run(case):
    tmp = tempfile.mkdtemp(prefix="lfe")
    reg, ws, ob = tmp + "/reg", tmp + "/ws", tmp + "/ob"
    w(reg + "/bazel_registry.json", '{"mirrors": [], "module_base_path": "."}')
    for mod, versions in case["registry"].items():
        w(f"{reg}/modules/{mod}/metadata.json", json.dumps({"versions": sorted(versions), "yanked_versions": {}}))
        for v, files in versions.items():
            d = f"{reg}/modules/{mod}/{v}"
            w(d + "/source.json", json.dumps({"type": "local_path", "path": f"modules/{mod}/{v}"}))
            if "BUILD.bazel" not in files:
                w(d + "/BUILD.bazel", "")
            for f, t in files.items():
                w(f"{d}/{f}", t)
    for f, t in case["root"].items():
        w(f"{ws}/{f}", t)
    if "BUILD.bazel" not in case["root"]:
        w(ws + "/BUILD.bazel", "")
    extra = [f"--override_repository={o.replace('@WS@', ws)}" for o in case.get("overrides", [])]
    flags = [f"--output_base={ob}", "--registry=file://" + reg, "--registry=https://bcr.bazel.build",
             "--lockfile_mode=off", *extra]
    targets = [f"@@{c}//:f" for c in case["fetch"]]
    p = subprocess.run(["bazel", *flags[:1], "build", *targets, *flags[1:]], cwd=ws,
                       capture_output=True, text=True)
    prints = re.findall(r"^DEBUG: \S+?:\d+:\d+: (.*)$", p.stderr, re.M)
    errors = [l[7:].replace(ws, "<ws>") for l in p.stderr.splitlines() if l.startswith("ERROR: ")]
    builds = {}
    ext = ob + "/external"
    if os.path.isdir(ext):
        for name in sorted(os.listdir(ext)):
            f = os.path.realpath(f"{ext}/{name}") + "/BUILD.bazel"
            if name in case["fetch"] and os.path.isfile(f):
                builds[name] = open(f).read()
    subprocess.run(["bazel", f"--output_base={ob}", "shutdown"], cwd=ws, capture_output=True)
    shutil.rmtree(tmp, ignore_errors=True)
    return {"name": case["name"], "prints": prints, "errors": errors, "builds": builds,
            "exit": p.returncode, "reg": reg}

if __name__ == "__main__":
    cases = json.load(open(sys.argv[1]))
    out = [run(c) for c in cases]
    json.dump(out, open(sys.argv[2], "w"), indent=1)
