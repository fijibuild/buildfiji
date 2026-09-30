"""Writes the registry file hashes Bazel records in MODULE.bazel.lock.

    lock_hashes.py <workspace> [bazel flags...]

Bazel only locks files read from a non-`file://` registry, so this serves the
fixture registry from a local HTTP server on a free port, runs
`bazel mod graph --lockfile_mode=update` on a copy of the workspace, and prints
what the lockfile recorded for that registry, one `<path> <TAB> <hash>` line per
file, with the registry's URL taken off the front (the port is not part of the
fixture). The hash is `not found` for a file the registry did not have.

The registry is served with every `source.json` rewritten to an archive
source: Bazel refuses a `local_path` module from a remote registry. The
rewrite is deterministic, and `conformance.rs` makes the same one, so the
`source.json` hashes compare.
"""

import functools
import http.server
import json
import os
import shutil
import subprocess
import sys
import tempfile
import threading

ARCHIVE_SOURCE = (
    '{"url":"https://example.invalid/%s-%s.tar.gz",'
    '"integrity":"sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",'
    '"strip_prefix":"%s"}\n'
)


def archive_registry(registry, dest):
    shutil.copytree(registry, dest)
    modules = os.path.join(dest, "modules")
    for name in os.listdir(modules):
        for version in os.listdir(os.path.join(modules, name)):
            source = os.path.join(modules, name, version, "source.json")
            if os.path.isfile(source):
                with open(source, "w") as f:
                    f.write(ARCHIVE_SOURCE % (name, version, name))


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *args):
        pass


def main():
    workspace, flags = sys.argv[1], sys.argv[2:]
    registry = os.path.join(os.path.dirname(os.path.abspath(workspace)), "..", "registry")
    registry = os.path.normpath(registry)
    with tempfile.TemporaryDirectory() as tmp:
        served = os.path.join(tmp, "registry")
        archive_registry(registry, served)
        handler = functools.partial(Quiet, directory=served)
        server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), handler)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        base = "http://127.0.0.1:%d/" % server.server_address[1]
        work = os.path.join(tmp, "workspace")
        shutil.copytree(workspace, work)
        subprocess.run(
            ["bazel", "mod", "graph", "--registry=" + base[:-1], *flags,
             "--lockfile_mode=update"],
            cwd=work, capture_output=True, text=True, check=True,
        )
        with open(os.path.join(work, "MODULE.bazel.lock")) as f:
            lock = json.load(f)
        subprocess.run(["bazel", "shutdown"], cwd=work, capture_output=True)
        server.shutdown()
    for url, digest in sorted(lock["registryFileHashes"].items()):
        if url.startswith(base):
            print("%s\t%s" % (url[len(base):], digest))
    yanked = lock["selectedYankedVersions"]
    for key in yanked:
        print("yanked\t%s\t%s" % (key, yanked[key]))


if __name__ == "__main__":
    main()
