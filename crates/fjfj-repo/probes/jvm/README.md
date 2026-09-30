# Reading Bazel's bytecode (no JDK needed)

Bazel's behaviour is the spec, and where probes cannot show how a hash or a file
is made its classes can. `A-server.jar` (in `bazel info install_base`) holds them
and the embedded JRE runs them, though there is no `javap` or `javac`.

- `cp.py JAR CLASS...`: the strings of each class's constant pool.
- `dis.py JAR CLASS [METHOD...]`: a small disassembler (the opcodes Bazel's code
  uses): calls, field accesses, string constants, branches.
- `mk_probe_class.py` writes `com/google/devtools/build/lib/bazel/bzlmod/Probe.class`
  by hand (class file version 49, so no stack maps), a `main` that parses its
  argument as a `SingleExtensionUsagesValue`, prints what `trimForEvaluation` gives
  as JSON and `hashForEvaluation` as base64. `run_probe.sh` runs it.

It was used for buildfiji-mum.8.6: see `../README.md` and the design doc.
