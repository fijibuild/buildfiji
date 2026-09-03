"""Flattens `bazel mod graph --output=json` into a canonical edge list.

One line per dependency edge:

    <parent key> <apparent repo name> <child key>

sorted and deduplicated, so the golden is stable against the order Bazel
happens to print a tree in, and readable in a diff. The node set is
recoverable from the edges plus the root, which is always `<root>`.
"""

import json
import sys


def walk(node, edges):
    parent = node["key"]
    for dep in node.get("dependencies", []):
        edges.add((parent, dep.get("apparentName", dep["name"]), dep["key"]))
        # `unexpanded` marks a node Bazel already printed in full
        # elsewhere in the tree, so its children are not repeated here.
        if not dep.get("unexpanded"):
            walk(dep, edges)


def main():
    root = json.load(sys.stdin)
    edges = set()
    walk(root, edges)
    for parent, apparent, child in sorted(edges):
        print(f"{parent} {apparent} {child}")


if __name__ == "__main__":
    main()
