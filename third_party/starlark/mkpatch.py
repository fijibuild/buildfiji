#!/usr/bin/env python3
"""mkpatch.py NAME FILE OLD NEW [FILE OLD NEW ...]: write third_party/starlark/NAME.patch,
which makes each edit to the crate's source (FILE relative to the crate root, OLD replaced
by NEW, which must occur once) with the `diff --git` headers Bazel's patch needs.
CRATE=starlark_syntax (say) patches another crate of the same version."""
import difflib, glob, os, sys

crate = os.environ.get("CRATE", "starlark")
root = glob.glob(os.path.expanduser(f"~/.cargo/registry/src/*/{crate}-0.14.2"))[0]
name, edits = sys.argv[1], sys.argv[2:]
out = []
files = {}
for i in range(0, len(edits), 3):
    f, old, new = edits[i:i + 3]
    text = files.get(f) or open(os.path.join(root, f)).read()
    assert text.count(old) == 1, f"{f}: {old!r} occurs {text.count(old)} times"
    files[f] = text.replace(old, new, 1)
for f, new in files.items():
    old = open(os.path.join(root, f)).read()
    diff = list(difflib.unified_diff(old.splitlines(True), new.splitlines(True), f"a/{f}", f"b/{f}"))
    out.append(f"diff --git a/{f} b/{f}\n" + "".join(diff))
here = os.path.dirname(os.path.abspath(__file__))
open(os.path.join(here, name + ".patch"), "w").write("".join(out))
print("wrote", name)
