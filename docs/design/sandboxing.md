# Sandboxing and local execution strategies

## What Bazel provides
`--spawn_strategy`, `--strategy=Mnemonic=strategy`, `--strategy_regexp`,
strategies `local`, `sandboxed`, `linux-sandbox`, `darwin-sandbox`,
`processwrapper-sandbox`, `worker`, `docker`, `remote`; `--sandbox_*` flags
(`--sandbox_tmpfs_path`, `--sandbox_writable_path`, `--sandbox_add_mount_pair`,
`--experimental_sandbox_hermetic_tmp`, `--reuse_sandbox_directories`);
persistent workers (`--worker_max_instances`, `--worker_sandboxing`,
`--experimental_worker_multiplex`); `--incompatible_strict_action_env`;
`--action_env`, `--host_action_env`.

## Compatibility bar
- Same flag names and same default strategy selection per OS.
- Same execroot layout (`bazel-out/<cfg>/bin`, runfiles trees, `external/`)
  so tools that hard-code paths keep working.
- Persistent worker protocol compatibility (proto over stdio, JSON variant)
  so existing workers (rules_java, rules_kotlin, rules_scala, TypeScript)
  run unchanged.

## Design sketch
`fjfj-sandbox::Sandbox` trait: `prepare(execroot, inputs)`, `run(cmd)`,
`collect(outputs)`, `cleanup()`. Implementations:
- `Local`: scratch execroot, symlinked/hardlinked inputs, no isolation.
- `LinuxNamespaces`: `unshare(CLONE_NEWUSER|NEWNS|NEWPID|NEWNET)`, bind
  mounts, tmpfs, using the `nix` crate; equivalent of `linux-sandbox`.
- `DarwinSeatbelt`: `sandbox-exec` with a generated profile; equivalent of
  `darwin-sandbox`.
- `Oci`: run via an OCI runtime for `docker`-strategy parity.
- `Worker`: persistent worker pool implementing Bazel's worker protocol.

Input materialisation is the expensive part; share one implementation with
remote execution (Merkle tree -> execroot) and cache it across actions
(`--reuse_sandbox_directories`).

## Open questions
- Hermetic `/tmp` and network isolation defaults.
- Sandboxing on Windows (Bazel has none; likely `local` only initially).
- Whether `Oci` is worth maintaining vs. delegating to remote execution.

## Cancellation and crash safety

- Every action runs in its own process group (and cgroup / PID namespace on
  Linux). Cancel is an immediate SIGKILL of the group; no SIGTERM grace.
  Reap with `pidfd` (Linux) or `kqueue` `EVFILT_PROC` (macOS) before the
  sandbox directory is released.
- Persistent workers get one protocol cancel request, then a kill on a
  short deadline; killed workers are respawned.
- Outputs are written to a per-action scratch directory and published into
  `bazel-out` by rename; CAS blobs are temp + fsync + rename. The action
  cache entry is written only after outputs are verified. A kill at any
  point costs at most a rerun.
- The daemon is multithreaded tokio and never calls `fork()`. Spawning uses
  `posix_spawn` / `clone3`; Linux namespace setup lives in a small static
  helper binary, as Bazel's `linux-sandbox` does.
- Children must not outlive a dead daemon: `PR_SET_PDEATHSIG` on Linux,
  process groups recorded in the output_base lock file, and a startup sweep
  that kills leftovers.
