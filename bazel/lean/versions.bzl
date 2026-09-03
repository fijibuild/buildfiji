"""Pinned Lean 4 toolchain release artifacts, one entry per platform
buildfiji supports (macOS arm64, Linux x86_64, Linux arm64 —
buildfiji-4b0.5). This is data only, for buildfiji-4b0.1's toolchain
repository rule to consume; refresh it by hand against
`spec/lean-toolchain` and
https://github.com/leanprover/lean4/releases/tag/v<version> when the
pinned version bumps — there's no equivalent of `refresh_bazel_flags.rs`
for this yet.

Archive format: `.tar.zst`, not `.zip` — noticeably smaller downloads
and faster extraction (zstd decodes far cheaper than DEFLATE at these
sizes, ~550-600 MB per platform), which is this bead's "cache-friendly
extraction" half.

`sha256` pins are the `digest` field GitHub's own release API reports
for each asset (`GET /repos/leanprover/lean4/releases/tags/v<version>`),
not recomputed locally — GitHub computes these when the asset is
uploaded, so they're authoritative for what a `download_and_extract`
will actually fetch.

Each archive has exactly one top-level directory, named after the asset
itself (`lean-<version>-<platform>`), that must be stripped on
extraction. Lean ships no manifest documenting this, so it's confirmed
instead against elan's own unpacking code — `unpack_without_first_dir`
in leanprover/elan's `src/elan-dist/src/component/package.rs` — which
strips exactly one leading path component from every archive entry.
"""

# Keep in sync with spec/lean-toolchain (currently "leanprover/lean4:v4.33.1").
LEAN_VERSION = "4.33.1"

_ASSET_URL = "https://github.com/leanprover/lean4/releases/download/v{version}/lean-{version}-{asset}.tar.zst"

# Each entry: Bazel platform `constraint_values` (from the `platforms`
# module — already a `bazel_dep` in MODULE.bazel, no new dependency
# needed), the release asset's sha256, and the directory name to strip
# on extraction. Keyed by a short platform name for
# buildfiji-4b0.1's repository rule to select on.
LEAN_TOOLCHAINS = {
    "darwin_arm64": struct(
        constraint_values = [
            "@platforms//os:macos",
            "@platforms//cpu:arm64",
        ],
        url = _ASSET_URL.format(version = LEAN_VERSION, asset = "darwin_aarch64"),
        sha256 = "88c45aad985b5d2a8d925fe10bd1296bd35f66f408480ab182d3facccd065a9d",
        strip_prefix = "lean-{version}-darwin_aarch64".format(version = LEAN_VERSION),
    ),
    "linux_x86_64": struct(
        constraint_values = [
            "@platforms//os:linux",
            "@platforms//cpu:x86_64",
        ],
        url = _ASSET_URL.format(version = LEAN_VERSION, asset = "linux"),
        sha256 = "890afd185370f85666025b883914ab4f4b339136f8c96167b69cfb62aecaf235",
        strip_prefix = "lean-{version}-linux".format(version = LEAN_VERSION),
    ),
    "linux_arm64": struct(
        constraint_values = [
            "@platforms//os:linux",
            "@platforms//cpu:arm64",
        ],
        url = _ASSET_URL.format(version = LEAN_VERSION, asset = "linux_aarch64"),
        sha256 = "f7353a8b2a8741c84558523e450556f9a1c45e3cafcf54399ce68c6a24c55f07",
        strip_prefix = "lean-{version}-linux_aarch64".format(version = LEAN_VERSION),
    ),
}
