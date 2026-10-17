# Check how fjfj will read a set of Bazel flags

Before switching a CI job or script from `bazel` to `fjfj`, find out whether
fjfj accepts every flag it passes.

## See the canonical form of a flag list

```sh
fjfj canonicalize-flags --for-command build -- -k --nostamp --keep_going
```

prints

```
--keep_going --nostamp --keep_going
```

Short forms are expanded to long names, and everything else is kept as
written, in order. `--for-command` defaults to `build`. Put `--` before the
flags so fjfj does not try to read them as its own options.

A flag that is not a Bazel flag at all is reported with Bazel's own
`unrecognized option` message and exit code `2`.

## Test the flags against a command

`canonicalize-flags` only checks that the flags are real Bazel flags. To
find out whether fjfj *implements* them, run the command:

```sh
fjfj build --remote_executor=grpc://localhost:8980 //...
```

- `ERROR: flag '--remote_executor' is recognized by Bazel for 'build' but not
  implemented by fjfj yet` (exit `2`): Bazel accepts it, fjfj does not do
  anything with it yet, so fjfj refuses to run. Check
  [Flags](../reference/flags.md) for what is implemented, and `bd search`
  for an existing bead before filing one.
- A clap error such as `unexpected argument '--nosuchflag' found` (exit
  `2`): not a Bazel flag for this command. Likely a typo, or a flag that
  belongs to a different command.
- No flag error: every flag was accepted.

To test a whole `.bazelrc`, run the command in the workspace: fjfj applies the
system, workspace and home rc files (with `import`s and `--config`s expanded)
to `build`, `test`, `run`, `query`, `cquery`, `aquery`, `info` and `clean`, and
checks those flags the same way. The error names the flag, not the rc line it
came from, so `grep` the rc files for it.
