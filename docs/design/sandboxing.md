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
