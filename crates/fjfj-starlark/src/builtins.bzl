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
# The OS of the execution platform, which is the target platform until the two
# are modelled apart (buildfiji-3wf).
def _exec_os(ctx):
    return ctx.fragments.platform._os

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
        exec_os = _exec_os,
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

# Providers inside a namespace are named by the private names they are bound
# to here, as `rule(provides = [...])` requires of a provider.
_ConstraintSettingInfo = provider(doc = "A constraint_setting.")
_ConstraintValueInfo = provider(doc = "A constraint_value.")
_PlatformInfo = provider(doc = "A platform.")
_TemplateVariableInfo = provider(doc = "Make variables.", fields = ["variables"])
_ToolchainInfo = provider(doc = "The data of a toolchain.")
_FeatureFlagInfo = provider(doc = "A feature flag's value.", fields = ["value"])
_ExecutionInfo = provider(doc = "How a test runs.", fields = ["requirements", "exec_group"])
_TestEnvironment = provider(doc = "The environment of a test.", fields = ["environment", "inherited_environment"])
_Objc = provider(doc = "Objective-C information.")
_XcodeProperties = provider(doc = "Xcode properties.")
def _xcode_version_config_init(
        ios_sdk_version,
        ios_minimum_os_version,
        visionos_sdk_version,
        visionos_minimum_os_version,
        watchos_sdk_version,
        watchos_minimum_os_version,
        tvos_sdk_version,
        tvos_minimum_os_version,
        macos_sdk_version,
        macos_minimum_os_version,
        xcode_version = None,
        availability = "unknown",
        xcode_version_flag = None,
        include_xcode_execution_info = False):
    sdk = {
        "ios": ios_sdk_version,
        "visionos": visionos_sdk_version,
        "watchos": watchos_sdk_version,
        "tvos": tvos_sdk_version,
        "macos": macos_sdk_version,
        "catalyst": macos_sdk_version,
    }
    minimum = {
        "ios": ios_minimum_os_version,
        "visionos": visionos_minimum_os_version,
        "watchos": watchos_minimum_os_version,
        "tvos": tvos_minimum_os_version,
        "macos": macos_minimum_os_version,
        "catalyst": macos_minimum_os_version,
    }
    return {
        "xcode_version": lambda: xcode_version,
        "availability": lambda: availability,
        "minimum_os_for_platform_type": lambda platform_type: minimum[platform_type],
        "sdk_version_for_platform": lambda platform: sdk[platform.platform_type],
        "execution_info": lambda: {"requires-darwin": ""} if include_xcode_execution_info else {},
    }

_XcodeVersionConfig, _ = provider(doc = "An xcode_config.", init = _xcode_version_config_init)

platform_common = struct(
    ConstraintSettingInfo = _ConstraintSettingInfo,
    ConstraintValueInfo = _ConstraintValueInfo,
    PlatformInfo = _PlatformInfo,
    TemplateVariableInfo = _TemplateVariableInfo,
    ToolchainInfo = _ToolchainInfo,
)

config_common = struct(
    FeatureFlagInfo = _FeatureFlagInfo,
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
    ExecutionInfo = _ExecutionInfo,
    TestEnvironment = _TestEnvironment,
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
    Objc = _Objc,
    XcodeProperties = _XcodeProperties,
    XcodeVersionConfig = _XcodeVersionConfig,
    apple_host_system_env = _unavailable("apple_common.apple_host_system_env", "buildfiji-136.15"),
    apple_toolchain = lambda: struct(developer_dir = _unavailable("apple_common.apple_toolchain().developer_dir", "buildfiji-136.15"), platform_developer_framework_dir = _unavailable("apple_common.apple_toolchain().platform_developer_framework_dir", "buildfiji-136.15"), sdk_dir = _unavailable("apple_common.apple_toolchain().sdk_dir", "buildfiji-136.15")),
    dotted_version = lambda version: str(version) if version else None,
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

# What rules_python's `py_internal_renamed.bzl` re-exports: a native object in
# Bazel that only the python rules may use. Loading needs the symbol; calling
# a member is buildfiji-136.16's.
py_internal = struct(
    cc_toolchain_build_info_files = _unavailable("py_internal.cc_toolchain_build_info_files", "buildfiji-136.16"),
    copy_without_caching = _unavailable("py_internal.copy_without_caching", "buildfiji-136.16"),
    create_repo_mapping_manifest = _unavailable("py_internal.create_repo_mapping_manifest", "buildfiji-136.16"),
    declare_shareable_artifact = _unavailable("py_internal.declare_shareable_artifact", "buildfiji-136.16"),
    get_label_repo_runfiles_path = _unavailable("py_internal.get_label_repo_runfiles_path", "buildfiji-136.16"),
    get_legacy_external_runfiles = _unavailable("py_internal.get_legacy_external_runfiles", "buildfiji-136.16"),
    is_bzlmod_enabled = lambda ctx: True,
    is_tool_configuration = _unavailable("py_internal.is_tool_configuration", "buildfiji-136.16"),
    link = _unavailable("py_internal.link", "buildfiji-136.16"),
    linkstamp_file = _unavailable("py_internal.linkstamp_file", "buildfiji-136.16"),
    regex_match = _unavailable("py_internal.regex_match", "buildfiji-136.16"),
    runfiles_enabled = _unavailable("py_internal.runfiles_enabled", "buildfiji-136.16"),
    share_native_deps = _unavailable("py_internal.share_native_deps", "buildfiji-136.16"),
    stamp_binaries = _unavailable("py_internal.stamp_binaries", "buildfiji-136.16"),
)

# The native rules whose analysis is Starlark (buildfiji-4qs): `ctx` is the rule's
# attributes as the native schema types them. The rules that stay Rust are the
# ones that read or write what a provider cannot say.
def _filegroup(ctx):
    return [DefaultInfo(files = depset(transitive = [src[DefaultInfo].files for src in ctx.attr.srcs]))]

_native_implementations = {
    "filegroup": _filegroup,
}

# `ctx.fragments` (buildfiji-136.18): the options of the configuration as the
# structs Bazel's configuration fragments are. `options` is what the Rust side
# knows of the configuration; a field a rule reads that is not here is an
# error of the rule's, so it is added when one reads it.
def _make_fragments(options):
    macos = apple_common.platform.macos
    apple = struct(
        xcode_version_flag = None,
        ios_sdk_version_flag = None,
        macos_sdk_version_flag = None,
        tvos_sdk_version_flag = None,
        watchos_sdk_version_flag = None,
        ios_minimum_os_flag = None,
        macos_minimum_os_flag = None,
        tvos_minimum_os_flag = None,
        watchos_minimum_os_flag = None,
        prefer_mutual_xcode = True,
        include_xcode_exec_requirements = False,
        single_arch_platform = macos,
        single_arch_cpu = "x86_64",
    )
    mode = options["compilation_mode"]

    # The options are joined by spaces; a value with one is split (buildfiji-136.16.1).
    def words(name):
        return options[name].split(" ") if options[name] else []

    cpp = struct(
        # Members Bazel hides behind an allowlist have the value of their flag's
        # default; the others were probed on Bazel 9.2.0.
        apple_generate_dsym = False,
        build_test_dwp = lambda: False,
        compilation_mode = lambda: mode,
        conlyopts = words("conlyopt"),
        copts = words("copt"),
        cs_fdo_instrument = lambda: None,
        cs_fdo_path = lambda: None,
        custom_malloc = None,
        cxxopts = words("cxxopt"),
        disable_nocopts = lambda: True,
        do_not_use_macos_set_install_name = False,
        dynamic_mode = lambda: "DEFAULT",
        experimental_cc_implementation_deps = lambda: False,
        experimental_cpp_modules = lambda: False,
        experimental_link_static_libraries_once = lambda: False,
        fdo_instrument = lambda: None,
        fdo_path = lambda: None,
        fission_active_for_current_compilation_mode = lambda: False,
        force_pic = lambda: False,
        generate_llvm_lcov = lambda: False,
        grte_top = lambda: None,
        include_scanning = lambda: False,
        incompatible_remove_legacy_whole_archive = lambda: True,
        incompatible_use_specific_tool_files = lambda: True,
        interface_shared_objects = lambda: True,
        legacy_whole_archive = lambda: True,
        linkopts = words("linkopt"),
        lto_backend_options = [],
        lto_index_options = lambda: [],
        minimum_os_version = lambda: None,
        objc_enable_binary_stripping = lambda: False,
        objc_generate_linkmap = False,
        objc_should_generate_dotd_files = lambda: True,
        objc_should_strip_binary = False,
        objccopts = [],
        process_headers_in_dependencies = lambda: False,
        propeller_optimize_absolute_cc_profile = lambda: None,
        propeller_optimize_absolute_ld_profile = lambda: None,
        proto_profile = lambda: False,
        save_feature_state = lambda: False,
        save_temps = lambda: False,
        share_native_deps = lambda: True,
        should_generate_dotd_files = lambda: True,
        # --strip=sometimes strips in fastbuild.
        should_strip_binaries = lambda: options["strip"] == "always" or (options["strip"] != "never" and mode == "fastbuild"),
        start_end_lib = lambda: True,
        strip_opts = lambda: [],
        use_llvm_coverage_map_format = lambda: False,
        _dont_enable_host_nonhost = True,
        _fdo_prefetch_hints_label = None,
    )
    return struct(
        apple = apple,
        cpp = cpp,
        platform = struct(_os = options["os"]),
        java = struct(),
        proto = struct(),
        py = struct(),
        coverage = struct(),
    )

# `ctx.configuration`.
def _make_configuration(options):
    return struct(
        coverage_enabled = False,
        default_shell_env = {},
        host_path_separator = ":",
        short_id = options["short_id"],
        test_env = {},
        is_tool_configuration = lambda: bool(options["exec"]),
        stamp_binaries = lambda: False,
        is_sibling_repository_layout = lambda: False,
        has_separate_genfiles_directory = lambda: False,
        runfiles_enabled = lambda: True,
    )
