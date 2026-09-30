"""Writes the repo mapping of every repo of a workspace's module graph.

    dump_mappings.py <workspace> [bazel flags...]

asks Bazel for `mod dump_repo_mapping` of the main repo, then of every repo
a mapping names, breadth first, and prints one line per repo, sorted by the
repo's canonical name:

    <canonical repo name> <TAB> <the JSON object Bazel printed>

`@bazel_tools` is left out: its own dependencies come from the Bazel
Central Registry, not from the workspace under test.
"""

import subprocess
import sys


def dump(repo, workspace, flags):
    out = subprocess.run(
        ["bazel", "mod", "dump_repo_mapping", repo, *flags],
        cwd=workspace,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    lines = [l for l in out.splitlines() if l.startswith("{")]
    assert len(lines) == 1, (repo, out)
    return lines[0]


def main():
    workspace, flags = sys.argv[1], sys.argv[2:]
    import json

    seen = {""}
    queue = [""]
    found = {}
    while queue:
        repo = queue.pop(0)
        text = dump(repo, workspace, flags)
        found[repo] = text
        for canonical in json.loads(text).values():
            if canonical not in seen and canonical != "bazel_tools":
                seen.add(canonical)
                queue.append(canonical)
    for repo in sorted(found):
        print(f"{repo}\t{found[repo]}")


if __name__ == "__main__":
    main()
