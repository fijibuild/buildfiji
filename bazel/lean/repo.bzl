"""Interim, non-hermetic Lean toolchain discovery (buildfiji-4b0.3).

`lean_test` needs to actually run `lean`, but under
`--incompatible_strict_action_env` (.bazelrc) every action's `PATH` — and,
inside the sandbox, `HOME` — are fixed to sandboxed placeholders regardless
of the invoking shell's real environment (confirmed by hand). `lean` as
installed by elan is a shim that re-execs the pinned toolchain's real
binary by reading `~/.elan`, so it doesn't work under either restriction.
The real binary it resolves to has no such dependency (confirmed by hand:
running it directly with `$HOME` pointed at a nonexistent directory still
works) — a Bazel action just needs its literal path.

This repository rule finds that path once, at repository-fetch time (which
runs with the real environment, not a sandboxed action's), by asking `elan`
itself — `elan which lean` with `ELAN_TOOLCHAIN` set from
`spec/lean-toolchain`, so it resolves the pinned version even if some other
toolchain is the machine's elan default — and symlinks it in as
`@lean_local//:lean`, an ordinary Bazel label lean_test can depend on
directly; Bazel passes actions their declared inputs by path regardless of
`PATH`.

This is not buildfiji-4b0.1's toolchain repository rule: that one downloads
and pins Lean per platform from versions.bzl, verified by sha256, with no
dependency on what happens to already be installed. This one trusts
whatever `elan` resolves (hence `local = True`: it must re-run if the
environment's answer could have changed), so a machine without the pinned
version installed (`elan toolchain install $(cat spec/lean-toolchain)`)
fails at fetch time. buildfiji-4b0.1 replaces this; lean_test does not
change when it does.
"""

def _local_lean_repo_impl(repository_ctx):
    elan = repository_ctx.which("elan")
    if elan == None:
        fail("elan not found on PATH: install it (https://leanprover-community.github.io/get_started.html) before building lean_test targets under //spec/...")

    toolchain = repository_ctx.read(Label("//spec:lean-toolchain")).strip()
    result = repository_ctx.execute(
        [elan, "which", "lean"],
        environment = {"ELAN_TOOLCHAIN": toolchain},
    )
    if result.return_code != 0:
        fail("`elan which lean` failed for toolchain '%s' (%s): %s\nInstall it with `elan toolchain install %s`." %
             (toolchain, elan, result.stderr, toolchain))

    lean = result.stdout.strip()
    repository_ctx.symlink(lean, "lean")
    repository_ctx.file("BUILD.bazel", 'exports_files(["lean"])\n')

local_lean_repo = repository_rule(
    implementation = _local_lean_repo_impl,
    local = True,
    doc = "Symlinks the elan-resolved `lean` binary for spec/lean-toolchain's pinned version in as @lean_local//:lean. Interim until buildfiji-4b0.1's hermetic toolchain.",
)
