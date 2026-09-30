#!/bin/sh
# run_probe.sh '<SingleExtensionUsagesValue JSON>': print the JSON Bazel hashes for
# it (trimmed) and the usagesDigest, by running Bazel's own classes.
# Needs `python3 mk_probe_class.py` first (it writes ./com/.../Probe.class).
IB=$(bazel info install_base 2>/dev/null)
exec "$IB/embedded_tools/jdk/bin/java" \
  --add-opens=java.base/java.lang=ALL-UNNAMED --add-opens=java.base/java.util=ALL-UNNAMED \
  --add-opens=java.base/java.nio=ALL-UNNAMED --add-opens=java.base/sun.nio.ch=ALL-UNNAMED \
  -cp "$(dirname "$0"):$IB/A-server.jar" com.google.devtools.build.lib.bazel.bzlmod.Probe "$1"
