"""Vendors the Bazel Central Registry files that `bazel_tools`' dependency
graph reads, into `bcr/`, laid out as the registry is.

    vendor_bcr.py [--cache <repository cache sha256 dir>]

The list of files is the `registryFileHashes` of `lockfiles/9.2.0.lock`, a
lockfile Bazel 9.2.0 wrote, and each file is checked against the hash Bazel
recorded, so what lands in `bcr/` is what Bazel read. Files come from
https://bcr.bazel.build, or, with `--cache`, from Bazel's repository cache
(`~/.cache/bazel/_bazel_$USER/cache/repos/v1/content_addressable/sha256`),
where Bazel keeps them by hash.

The Bazel Central Registry is Apache-2.0 licensed.
"""

import hashlib
import json
import os
import sys
import urllib.request

PREFIX = "https://bcr.bazel.build/"


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    cache = sys.argv[sys.argv.index("--cache") + 1] if "--cache" in sys.argv else None
    with open(os.path.join(here, "lockfiles", "9.2.0.lock")) as f:
        hashes = json.load(f)["registryFileHashes"]
    for url, digest in sorted(hashes.items()):
        assert url.startswith(PREFIX), url
        if cache:
            with open(os.path.join(cache, digest, "file"), "rb") as f:
                content = f.read()
        else:
            with urllib.request.urlopen(url) as r:
                content = r.read()
        assert hashlib.sha256(content).hexdigest() == digest, url
        path = os.path.join(here, "bcr", url[len(PREFIX):])
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "wb") as f:
            f.write(content)


if __name__ == "__main__":
    main()
