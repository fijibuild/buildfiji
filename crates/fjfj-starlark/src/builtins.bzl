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

AnalysisFailureInfo = provider(
    doc = "Why a target failed analysis.",
    fields = ["causes"],
)

AnalysisTestResultInfo = provider(
    doc = "The result of an analysis test.",
    fields = ["success", "message"],
)

InstrumentedFilesInfo = provider(
    doc = "The files that coverage instruments.",
    fields = ["instrumented_files", "metadata_files"],
)

# The namespaces of the analysis phase, as far as loading a ruleset needs them:
# the names Bazel 9.2.0 has (`dir()` of each), the providers as providers, and
# every function a call that says which bead owns it. They are Starlark, not
# Rust, by the decision of buildfiji-136.14; buildfiji-136.15 and .16 make the
# functions do what Bazel's do. `type()` of a namespace is `struct` here and
# `platform_common` (for example) in Bazel.

def _unavailable(name, bead):
    def unavailable(*args, **kwargs):
        fail("%s is not implemented yet (%s)" % (name, bead))

    return unavailable

# `internal_DO_NOT_USE()` is the private API of the rulesets Bazel allowlists
# (rules_cc, rules_java, ...), which read it while loading. It has no members
# yet; the few that rules_cc calls while loading return placeholders, the rest
# fail where they are used, not where they are loaded.
def _cc_internals():
    return struct(
        check_private_api = lambda *args, **kwargs: None,
        freeze = lambda value: value,
        create_header_info = lambda *args, **kwargs: struct(),
        get_artifact_name_for_category = _unavailable("cc_common.internal_DO_NOT_USE().get_artifact_name_for_category", "buildfiji-136.15"),
        combine_cc_toolchain_variables = _unavailable("cc_common.internal_DO_NOT_USE().combine_cc_toolchain_variables", "buildfiji-136.15"),
        actions = _unavailable("cc_common.internal_DO_NOT_USE().actions", "buildfiji-136.15"),
        cc_toolchain_variables = _unavailable("cc_common.internal_DO_NOT_USE().cc_toolchain_variables", "buildfiji-136.15"),
        dynamic_library_symlink = _unavailable("cc_common.internal_DO_NOT_USE().dynamic_library_symlink", "buildfiji-136.15"),
        create_cc_compile_action = _unavailable("cc_common.internal_DO_NOT_USE().create_cc_compile_action", "buildfiji-136.15"),
        is_tree_artifact = _unavailable("cc_common.internal_DO_NOT_USE().is_tree_artifact", "buildfiji-136.15"),
        intern_string_sequence_variable_value = _unavailable("cc_common.internal_DO_NOT_USE().intern_string_sequence_variable_value", "buildfiji-136.15"),
        wrap_link_actions = _unavailable("cc_common.internal_DO_NOT_USE().wrap_link_actions", "buildfiji-136.15"),
        dynamic_library_soname = _unavailable("cc_common.internal_DO_NOT_USE().dynamic_library_soname", "buildfiji-136.15"),
        declare_other_output_file = _unavailable("cc_common.internal_DO_NOT_USE().declare_other_output_file", "buildfiji-136.15"),
        create_header_info_with_deps = _unavailable("cc_common.internal_DO_NOT_USE().create_header_info_with_deps", "buildfiji-136.15"),
        compute_output_name_prefix_dir = _unavailable("cc_common.internal_DO_NOT_USE().compute_output_name_prefix_dir", "buildfiji-136.15"),
        solib_symlink_action = _unavailable("cc_common.internal_DO_NOT_USE().solib_symlink_action", "buildfiji-136.15"),
        rule_class = _unavailable("cc_common.internal_DO_NOT_USE().rule_class", "buildfiji-136.15"),
        per_file_copts = _unavailable("cc_common.internal_DO_NOT_USE().per_file_copts", "buildfiji-136.15"),
        intern_seq = _unavailable("cc_common.internal_DO_NOT_USE().intern_seq", "buildfiji-136.15"),
        get_link_args = _unavailable("cc_common.internal_DO_NOT_USE().get_link_args", "buildfiji-136.15"),
        get_artifact_name_extension_for_category = _unavailable("cc_common.internal_DO_NOT_USE().get_artifact_name_extension_for_category", "buildfiji-136.15"),
        expand_and_tokenize = _unavailable("cc_common.internal_DO_NOT_USE().expand_and_tokenize", "buildfiji-136.15"),
        exec_os = _unavailable("cc_common.internal_DO_NOT_USE().exec_os", "buildfiji-136.15"),
        declare_compile_output_file = _unavailable("cc_common.internal_DO_NOT_USE().declare_compile_output_file", "buildfiji-136.15"),
        create_lto_backend_action_template = _unavailable("cc_common.internal_DO_NOT_USE().create_lto_backend_action_template", "buildfiji-136.15"),
        create_lto_backend_action = _unavailable("cc_common.internal_DO_NOT_USE().create_lto_backend_action", "buildfiji-136.15"),
        create_cc_compile_action_template = _unavailable("cc_common.internal_DO_NOT_USE().create_cc_compile_action_template", "buildfiji-136.15"),
        collect_per_file_lto_backend_opts = _unavailable("cc_common.internal_DO_NOT_USE().collect_per_file_lto_backend_opts", "buildfiji-136.15"),
        check_toplevel = _unavailable("cc_common.internal_DO_NOT_USE().check_toplevel", "buildfiji-136.15"),
        cc_toolchain_features = _unavailable("cc_common.internal_DO_NOT_USE().cc_toolchain_features", "buildfiji-136.15"),
        absolute_symlink = _unavailable("cc_common.internal_DO_NOT_USE().absolute_symlink", "buildfiji-136.15"),
    )

def _java_internals():
    return struct(
        google_legacy_api_enabled = lambda: False,
        expand_java_opts = _unavailable("java_common.internal_DO_NOT_USE().expand_java_opts", "buildfiji-136.15"),
        check_provider_instances = _unavailable("java_common.internal_DO_NOT_USE().check_provider_instances", "buildfiji-136.15"),
        check_java_toolchain_is_declared_on_rule = _unavailable("java_common.internal_DO_NOT_USE().check_java_toolchain_is_declared_on_rule", "buildfiji-136.15"),
        target_kind = _unavailable("java_common.internal_DO_NOT_USE().target_kind", "buildfiji-136.15"),
        incompatible_java_info_merge_runtime_module_flags = _unavailable("java_common.internal_DO_NOT_USE().incompatible_java_info_merge_runtime_module_flags", "buildfiji-136.15"),
        get_runtime_classpath_for_archive = _unavailable("java_common.internal_DO_NOT_USE().get_runtime_classpath_for_archive", "buildfiji-136.15"),
        create_header_compilation_action = _unavailable("java_common.internal_DO_NOT_USE().create_header_compilation_action", "buildfiji-136.15"),
        create_compilation_action = _unavailable("java_common.internal_DO_NOT_USE().create_compilation_action", "buildfiji-136.15"),
        collect_native_deps_dirs = _unavailable("java_common.internal_DO_NOT_USE().collect_native_deps_dirs", "buildfiji-136.15"),
    )

platform_common = struct(
    ConstraintSettingInfo = provider(doc = "A constraint_setting."),
    ConstraintValueInfo = provider(doc = "A constraint_value."),
    PlatformInfo = provider(doc = "A platform."),
    TemplateVariableInfo = provider(doc = "Make variables.", fields = ["variables"]),
    ToolchainInfo = provider(doc = "The data of a toolchain."),
)

config_common = struct(
    FeatureFlagInfo = provider(doc = "A feature flag's value.", fields = ["value"]),
    config_feature_flag_transition = _unavailable("config_common.config_feature_flag_transition", "buildfiji-136.16"),
    toolchain_type = _toolchain_type,
)

# Coverage is not collected yet (buildfiji-fyz.9); the provider is, with what the
# call says it would instrument, so a rule that asks for it loads.
def _instrumented_files_info(
        ctx,
        source_attributes = [],
        dependency_attributes = [],
        extensions = None,
        metadata_files = [],
        baseline_coverage_files = None,
        **kwargs):
    return InstrumentedFilesInfo(
        instrumented_files = depset(),
        metadata_files = depset(metadata_files),
    )

coverage_common = struct(
    instrumented_files_info = _instrumented_files_info,
)

testing = struct(
    ExecutionInfo = provider(doc = "How a test runs.", fields = ["requirements", "exec_group"]),
    TestEnvironment = provider(doc = "The environment of a test.", fields = ["environment", "inherited_environment"]),
    analysis_test = _unavailable("testing.analysis_test", "buildfiji-136.16"),
)

cc_common = struct(
    action_is_enabled = _unavailable("cc_common.action_is_enabled", "buildfiji-136.15"),
    add_go_exec_groups_to_binary_rules = _unavailable("cc_common.add_go_exec_groups_to_binary_rules", "buildfiji-136.15"),
    check_experimental_cc_shared_library = _unavailable("cc_common.check_experimental_cc_shared_library", "buildfiji-136.15"),
    do_not_use_tools_cpp_compiler_present = None,
    empty_variables = _unavailable("cc_common.empty_variables", "buildfiji-136.15"),
    get_environment_variables = _unavailable("cc_common.get_environment_variables", "buildfiji-136.15"),
    get_execution_requirements = _unavailable("cc_common.get_execution_requirements", "buildfiji-136.15"),
    get_memory_inefficient_command_line = _unavailable("cc_common.get_memory_inefficient_command_line", "buildfiji-136.15"),
    get_tool_for_action = _unavailable("cc_common.get_tool_for_action", "buildfiji-136.15"),
    get_tool_requirement_for_action = _unavailable("cc_common.get_tool_requirement_for_action", "buildfiji-136.15"),
    implementation_deps_allowed_by_allowlist = _unavailable("cc_common.implementation_deps_allowed_by_allowlist", "buildfiji-136.15"),
    incompatible_disable_objc_library_transition = _unavailable("cc_common.incompatible_disable_objc_library_transition", "buildfiji-136.15"),
    internal_DO_NOT_USE = _cc_internals,
    legacy_cc_flags_make_variable_do_not_use = _unavailable("cc_common.legacy_cc_flags_make_variable_do_not_use", "buildfiji-136.15"),
)

java_common = struct(
    internal_DO_NOT_USE = _java_internals,
)

_APPLE_PLATFORM = struct(
    catalyst = struct(is_device = True, name = "catalyst", name_in_plist = "MacOSX", platform_type = "catalyst"),
    ios_device = struct(is_device = True, name = "ios_device", name_in_plist = "iPhoneOS", platform_type = "ios"),
    ios_simulator = struct(is_device = False, name = "ios_simulator", name_in_plist = "iPhoneSimulator", platform_type = "ios"),
    macos = struct(is_device = True, name = "macos", name_in_plist = "MacOSX", platform_type = "macos"),
    tvos_device = struct(is_device = True, name = "tvos_device", name_in_plist = "AppleTVOS", platform_type = "tvos"),
    tvos_simulator = struct(is_device = False, name = "tvos_simulator", name_in_plist = "AppleTVSimulator", platform_type = "tvos"),
    visionos_device = struct(is_device = True, name = "visionos_device", name_in_plist = "XROS", platform_type = "visionos"),
    visionos_simulator = struct(is_device = False, name = "visionos_simulator", name_in_plist = "XRSimulator", platform_type = "visionos"),
    watchos_device = struct(is_device = True, name = "watchos_device", name_in_plist = "WatchOS", platform_type = "watchos"),
    watchos_simulator = struct(is_device = False, name = "watchos_simulator", name_in_plist = "WatchSimulator", platform_type = "watchos"),
)

_APPLE_PLATFORM_TYPE = struct(
    catalyst = "catalyst",
    ios = "ios",
    macos = "macos",
    tvos = "tvos",
    visionos = "visionos",
    watchos = "watchos",
)

apple_common = struct(
    Objc = provider(doc = "Objective-C information."),
    XcodeProperties = provider(doc = "Xcode properties."),
    XcodeVersionConfig = provider(doc = "An xcode_config."),
    apple_host_system_env = _unavailable("apple_common.apple_host_system_env", "buildfiji-136.15"),
    apple_toolchain = lambda: struct(developer_dir = _unavailable("apple_common.apple_toolchain().developer_dir", "buildfiji-136.15"), platform_developer_framework_dir = _unavailable("apple_common.apple_toolchain().platform_developer_framework_dir", "buildfiji-136.15"), sdk_dir = _unavailable("apple_common.apple_toolchain().sdk_dir", "buildfiji-136.15")),
    dotted_version = lambda version: struct(_version = version, compare_to = _unavailable("DottedVersion.compare_to", "buildfiji-136.15")),
    new_objc_provider = _unavailable("apple_common.new_objc_provider", "buildfiji-136.15"),
    platform = _APPLE_PLATFORM,
    platform_type = _APPLE_PLATFORM_TYPE,
    target_apple_env = _unavailable("apple_common.target_apple_env", "buildfiji-136.15"),
)

android_common = struct(
    create_dex_merger_actions = _unavailable("android_common.create_dex_merger_actions", "buildfiji-136.15"),
    resource_source_directory = _unavailable("android_common.resource_source_directory", "buildfiji-136.15"),
)

# What Bazel 9.2.0's `proto_common_do_not_use` has, which rules_java reads.
proto_common_do_not_use = struct(INCOMPATIBLE_ENABLE_PROTO_TOOLCHAIN_RESOLUTION = True)

# A built-in function in Bazel; its transition is buildfiji-136.6's.
exec_transition = _unavailable("exec_transition", "buildfiji-136.6")
