#!/usr/bin/env bash
# Regenerate the aquery/cquery jsonproto fixtures in this directory
# (buildfiji-23d.19). Each upstream example is its own bzlmod workspace, so
# it's built with its own pinned Bazel version, not this repo's.
#
# Usage: fixtures/generate.sh <scratch-dir>
set -euo pipefail
scratch="${1:?usage: generate.sh <scratch-dir>}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mkdir -p "$scratch"

clone_and_query() {
  local repo="$1" example_dir="$2" out_name="$3"
  local dir="$scratch/$(basename "$repo" .git)"
  [ -d "$dir" ] || git clone --depth 1 "$repo" "$dir"
  (
    cd "$dir/$example_dir"
    bazel aquery 'deps(//...)' --output=jsonproto --include_artifacts \
      > "$here/$out_name.aquery.json"
    bazel cquery 'deps(//...)' --output=jsonproto \
      > "$here/$out_name.cquery.json"
  )
}

clone_and_query https://github.com/bazelbuild/rules_go.git examples/hello rules_go_hello
clone_and_query https://github.com/bazelbuild/rules_python.git examples/pip_parse rules_python_pip_parse
