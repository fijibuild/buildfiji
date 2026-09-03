#!/usr/bin/env bash
# Fetch the Starlark corpora for the parser spike (buildfiji-mum.1).
#
# Each repo is cloned blobless + sparse so only BUILD/*.bzl blobs are
# downloaded (envoy and tensorflow are multi-GB otherwise).
#
# Usage: fixtures/fetch.sh <corpus-dir>
set -euo pipefail
corpus="${1:?usage: fetch.sh <corpus-dir>}"
mkdir -p "$corpus"

fetch() {
  local repo="$1" name="$2"
  local dir="$corpus/$name"
  if [ -d "$dir" ]; then echo "have $name"; return; fi
  git clone --filter=blob:none --no-checkout --depth 1 "$repo" "$dir"
  git -C "$dir" sparse-checkout set --no-cone \
    '/**/BUILD' '/**/BUILD.bazel' '/**/*.bzl' '/**/*.star' '/**/*.bazel'
  git -C "$dir" checkout
}

fetch https://github.com/envoyproxy/envoy.git envoy
fetch https://github.com/tensorflow/tensorflow.git tensorflow
fetch https://github.com/bazelbuild/bazel.git bazel
