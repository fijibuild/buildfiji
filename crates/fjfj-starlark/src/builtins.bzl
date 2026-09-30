# The providers Bazel 9.2.0 defines for every .bzl file. They are here as
# providers with their fields, not yet with what Bazel does with them
# (buildfiji-136.4 is the analysis phase's use of them, buildfiji-136.15 the
# mechanism that puts fjfj's own .bzl files in every file's globals).

DefaultInfo = provider(
    doc = "The default providers of a target.",
    fields = ["files", "runfiles", "data_runfiles", "default_runfiles", "executable"],
)

RunEnvironmentInfo = provider(
    doc = "The environment a binary or test runs in.",
    fields = ["environment", "inherited_environment"],
)

PackageSpecificationInfo = provider(
    doc = "What a package_group says.",
    fields = [],
)

OutputGroupInfo = provider(
    doc = "The output groups of a target.",
)

InstrumentedFilesInfo = provider(
    doc = "The files that coverage instruments.",
    fields = ["instrumented_files", "metadata_files"],
)
