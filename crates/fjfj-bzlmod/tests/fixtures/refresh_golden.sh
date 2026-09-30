#!/usr/bin/env bash
# Regenerates the golden module graphs in workspaces/*/expected_graph.txt
# and expected_graph_builtin.txt,
# repo mappings in workspaces/*/expected_repo_mapping.txt and lockfile
# registry file hashes in workspaces/*/expected_lock_hashes.txt
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
dump_mappings="$(rlocation _main/crates/fjfj-bzlmod/tests/fixtures/dump_mappings)"
lock_hashes="$(rlocation _main/crates/fjfj-bzlmod/tests/fixtures/lock_hashes)"

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
  # The same with the built-in `bazel_tools` subtree shown, which is where its
  # own dependencies raise the versions of modules the workspace shares them with.
  (
    cd "${workspace}"
    bazel mod graph --output=json --include_builtin \
      --registry="${registry}" \
      --registry=https://bcr.bazel.build \
      --lockfile_mode=off \
      2>/dev/null
  ) | "${graph_to_golden}" > "${workspace}/expected_graph_builtin.txt"
  # What each repo of the graph calls the others (`bazel mod
  # dump_repo_mapping`), for the repo mappings fjfj builds from the graph.
  "${dump_mappings}" "${workspace}" \
    --registry="${registry}" \
    --registry=https://bcr.bazel.build \
    --lockfile_mode=off \
    > "${workspace}/expected_repo_mapping.txt"
  # What Bazel records in MODULE.bazel.lock for the registry's files. A
  # workspace that uses module extensions cannot be locked from the served
  # registry, whose sources are not real, so it has no such golden.
  if [[ ! -f "${workspace}/extension_repos.txt" ]]; then
    "${lock_hashes}" "${workspace}" \
      --registry=https://bcr.bazel.build \
      > "${workspace}/expected_lock_hashes.txt"
  fi
done

# Bazel refuses the `yanked` workspace, so it has no graph, but with
# --allow_yanked_versions=all it resolves, and its lockfile records the yanked
# version it selected.
"${lock_hashes}" "${fixtures}/workspaces/yanked" \
  --registry=https://bcr.bazel.build \
  --allow_yanked_versions=all \
  > "${fixtures}/workspaces/yanked/expected_lock_hashes.txt"
