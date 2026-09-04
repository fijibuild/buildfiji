#!/usr/bin/env bash
# buildfiji-4b0.4: assert spec/Fjfj.lean's imports and spec/BUILD.bazel's
# lean_library targets agree with the .lean files actually on disk, so a
# module added to one and forgotten in the other fails fast instead of
# `lake build` silently skipping it or `bazel build //spec/...` silently
# missing it.
#
# Sandbox-safe: no lake, no network, no bazel-in-bazel — everything it
# needs comes in as $1 (the root module) and the rest of argv (the module
# files), both wired as `data` + `args` in spec/BUILD.bazel.
set -euo pipefail

root="$1"
shift
modules=("$@")

# Module names spec/Fjfj.lean actually imports.
imported=$(grep -oE '^import Fjfj\.[A-Za-z0-9_]+' "$root" | awk '{print $2}' | sort -u)

# Module names spec/BUILD.bazel declared a lean_library target for, derived
# from the .lean files under Fjfj/ it was given as data.
declared=$(printf '%s\n' "${modules[@]}" | sed -E 's#.*/Fjfj/([A-Za-z0-9_]+)\.lean$#Fjfj.\1#' | sort -u)

if [[ "$imported" != "$declared" ]]; then
  echo "spec/Fjfj.lean imports and spec/BUILD.bazel lean_library targets disagree:" >&2
  diff <(echo "$imported") <(echo "$declared") >&2 || true
  exit 1
fi
