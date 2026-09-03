#!/usr/bin/env bash
# Regenerates the golden module graphs in workspaces/*/expected_graph.txt
# from real Bazel, which is the specification these fixtures test against
# (docs/ARCHITECTURE.md: "Bazel 9.2.0 observable behaviour is the spec").
#
#   bazel run //crates/fjfj-bzlmod/tests/fixtures:refresh_golden
#
# It is a `bazel run` target, not a test: it shells out to `bazel` and
# needs the network, neither of which belongs inside `bazel test`. It
# writes into the source tree, so run it by hand when a fixture changes
# and commit the result.
#
# The Bazel Central Registry is passed as a second registry because every
# module implicitly depends on `bazel_tools`, whose own MODULE.bazel has
# `bazel_dep`s that only BCR can serve. `bazel mod graph` hides the
# `bazel_tools` subtree (it is shown only under --include_builtin), so
# none of it reaches the golden files.
set -euo pipefail

# --- begin runfiles.bash initialization ---
f=bazel_tools/tools/bash/runfiles/runfiles.bash
set +e
source "${RUNFILES_DIR:-/dev/null}/$f" 2>/dev/null ||
  source "$(grep -sm1 "^$f " "${RUNFILES_MANIFEST_FILE:-/dev/null}" | cut -f2- -d' ')" 2>/dev/null ||
  source "$0.runfiles/$f" 2>/dev/null ||
  source "$(grep -sm1 "^$f " "$0.runfiles_manifest" | cut -f2- -d' ')" 2>/dev/null ||
  source "$(grep -sm1 "^$f " "$0.exe.runfiles_manifest" | cut -f2- -d' ')" 2>/dev/null ||
  { echo>&2 "ERROR: cannot find $f"; exit 1; }
set -e
# --- end runfiles.bash initialization ---

if [[ -z "${BUILD_WORKSPACE_DIRECTORY:-}" ]]; then
  echo >&2 "ERROR: run this with 'bazel run', not directly."
  exit 1
fi

fixtures="${BUILD_WORKSPACE_DIRECTORY}/crates/fjfj-bzlmod/tests/fixtures"
registry="file://${fixtures}/registry"
graph_to_golden="$(rlocation _main/crates/fjfj-bzlmod/tests/fixtures/graph_to_golden)"

for workspace in "${fixtures}"/workspaces/*/; do
  name="$(basename "${workspace}")"
  # A fixture that Bazel rejects records the error it printed instead of a
  # graph; there is nothing for `mod graph` to output.
  if [[ -f "${workspace}/expect_error" ]]; then
    echo "skipping ${name} (expects an error)"
    continue
  fi
  echo "refreshing ${name}"
  (
    cd "${workspace}"
    bazel mod graph --output=json \
      --registry="${registry}" \
      --registry=https://bcr.bazel.build \
      --lockfile_mode=off \
      2>/dev/null
  ) | "${graph_to_golden}" > "${workspace}/expected_graph.txt"
done
