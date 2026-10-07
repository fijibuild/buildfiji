# The providers Bazel 9.2.0 defines for every .bzl file. They are here as
# providers with their fields, not yet with what Bazel does with them
# (buildfiji-136.4 is the analysis phase's use of them, buildfiji-136.15 the
# mechanism that puts fjfj's own .bzl files in every file's globals).

load(
    "@_builtins//:cc_actions.bzl",
    _compute_output_name_prefix_dir = "compute_output_name_prefix_dir",
    _create_cc_compile_action = "create_cc_compile_action",
    _declare_compile_output_file = "declare_compile_output_file",
    _declare_other_output_file = "declare_other_output_file",
    _dynamic_library_soname = "dynamic_library_soname",
    _dynamic_library_symlink = "dynamic_library_symlink",
    _dynamic_library_symlink2 = "dynamic_library_symlink2",
    _get_link_args = "get_link_args",
    _solib_symlink_action = "solib_symlink_action",
    _absolute_symlink = "absolute_symlink",
    _get_artifact_name_extension_for_category = "get_artifact_name_extension_for_category",
    _get_artifact_name_for_category = "get_artifact_name_for_category",
    _is_tree_artifact = "is_tree_artifact",
    _per_file_copts = "per_file_copts",
)
load(
    "@_builtins//:cc_features.bzl",
    _action_is_enabled = "action_is_enabled",
    _cc_toolchain_features = "cc_toolchain_features",
    _cc_variables = "cc_variables",
    _combine_cc_variables = "combine_cc_variables",
    _get_environment_variables = "get_environment_variables",
    _get_execution_requirements = "get_execution_requirements",
    _get_memory_inefficient_command_line = "get_memory_inefficient_command_line",
    _get_tool_for_action = "get_tool_for_action",
)

# `DefaultInfo(...)` as Bazel's constructor takes it (probed on 9.2.0): the
# errors name `DefaultInfo`, as the native function does.
def _default_info_init(*args, **kwargs):
    if args:
        fjfj_native_error("DefaultInfo", "DefaultInfo() got unexpected positional argument")
    known = {
        "files": "depset",
        "runfiles": "runfiles",
        "data_runfiles": "runfiles",
        "default_runfiles": "runfiles",
        "executable": "File",
    }
    unknown = [k for k in kwargs if k not in known]
    if unknown:
        fjfj_native_error("DefaultInfo", "DefaultInfo() got unexpected keyword argument%s %s" % (
            "s" if len(unknown) > 1 else "",
            ", ".join(["'%s'" % k for k in unknown]),
        ) if len(unknown) > 1 else "DefaultInfo() got unexpected keyword argument '%s'" % unknown[0])
    given = {}
    for name, want in known.items():
        value = kwargs.get(name)
        if value == None:
            continue
        if type(value) != want:
            fjfj_native_error("DefaultInfo", "in call to DefaultInfo(), parameter '%s' got value of type '%s', want '%s or NoneType'" % (name, type(value), want))
        given[name] = value
    if "runfiles" in given and ("data_runfiles" in given or "default_runfiles" in given):
        fjfj_native_error("DefaultInfo", "Cannot specify the provider 'runfiles' together with 'data_runfiles' or 'default_runfiles'")
    # What an instance shows is Bazel's four members, None when not given;
    # the executable is kept under a name no code can write.
    return {
        "files": given.get("files"),
        "files_to_run": None,
        "data_runfiles": given.get("data_runfiles"),
        "default_runfiles": given.get("runfiles", given.get("default_runfiles")),
        "$executable": given.get("executable"),
    }

DefaultInfo, _raw_default_info = provider(
    doc = "The default providers of a target.",
    fields = ["files", "files_to_run", "data_runfiles", "default_runfiles", "$executable"],
    init = _default_info_init,
)

# The arguments of a native constructor with these parameters, in order, as
# Bazel's argument binding takes them (probed on 9.2.0): `fn` names the call in
# the errors, `params` is `(name, default, type, want)` and `want` is how the
# type reads in the error.
def _bind_native(fn, params, args, kwargs):
    if len(args) > len(params):
        fjfj_native_error(fn, "%s() accepts no more than %d positional arguments but got %d" % (fn, len(params), len(args)))
    known = [p[0] for p in params]
    for name in kwargs:
        if name not in known:
            fjfj_native_error(fn, "%s() got unexpected keyword argument '%s'" % (fn, name))
    out = {}
    for i, (name, default, kind, want) in enumerate(params):
        if i < len(args):
            if name in kwargs:
                fjfj_native_error(fn, "%s() got multiple values for parameter '%s'" % (fn, name))
            value = args[i]
        else:
            value = kwargs.get(name, default)
        if type(value) != kind:
            fjfj_native_error(fn, "in call to %s(), parameter '%s' got value of type '%s', want '%s'" % (fn, name, type(value), want))
        if kind == "dict":
            for k, v in value.items():
                if type(k) != "string" or type(v) != "string":
                    fjfj_native_error(fn, "got dict<%s, %s> for '%s', want dict<string, string>" % (type(k), type(v), name))
        out[name] = value
    return out

def _run_environment_fields(fn, args, kwargs):
    fields = _bind_native(fn, [
        ("environment", {}, "dict", "dict"),
        ("inherited_environment", [], "list", "sequence"),
    ], args, kwargs)
    fields["inherited_environment"] = list(fields["inherited_environment"])
    return fields

RunEnvironmentInfo, _raw_run_environment_info = provider(
    doc = "The environment a binary or test runs in.",
    fields = ["environment", "inherited_environment"],
    init = lambda *args, **kwargs: _run_environment_fields("RunEnvironmentInfo", args, kwargs),
)

PackageSpecificationInfo = provider(
    doc = "What a package_group says.",
    fields = [],
)

# `OutputGroupInfo(group = depset_of_files, ...)`: each group is a depset of
# files, as Bazel's constructor checks.
def _output_group_info_init(*args, **kwargs):
    if args:
        fjfj_native_error("OutputGroupInfo", "OutputGroupInfo() got unexpected positional argument")
    for name, value in kwargs.items():
        if type(value) != "depset":
            fjfj_native_error("OutputGroupInfo", "for output group '%s', got %s, want a depset of File" % (name, type(value)))
        element = fjfj_depset_item_type(value)
        if element != None and element != "File":
            fjfj_native_error("OutputGroupInfo", "for 'output group '%s'', got a depset of '%s', expected a depset of 'File'" % (name, element))
    return kwargs

OutputGroupInfo, _raw_output_group_info = provider(
    doc = "The output groups of a target.",
    init = _output_group_info_init,
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

# HeaderInfo, which a CcCompilationContextInfo keeps of its headers and modules.
def _create_header_info(
        header_module = None,
        pic_header_module = None,
        modular_public_headers = [],
        modular_private_headers = [],
        textual_headers = [],
        separate_module_headers = [],
        separate_module = None,
        separate_pic_module = None,
        deps = [],
        merged_deps = []):
    return struct(
        header_module = header_module,
        pic_header_module = pic_header_module,
        modular_public_headers = list(modular_public_headers),
        modular_private_headers = list(modular_private_headers),
        textual_headers = list(textual_headers),
        separate_module_headers = list(separate_module_headers),
        separate_module = separate_module,
        separate_pic_module = separate_pic_module,
        deps = list(deps),
        merged_deps = list(merged_deps),
    )

def _create_header_info_with_deps(header_info, deps, merged_deps):
    return _create_header_info(
        header_module = header_info.header_module,
        pic_header_module = header_info.pic_header_module,
        modular_public_headers = header_info.modular_public_headers,
        modular_private_headers = header_info.modular_private_headers,
        textual_headers = header_info.textual_headers,
        separate_module_headers = header_info.separate_module_headers,
        separate_module = header_info.separate_module,
        separate_pic_module = header_info.separate_pic_module,
        deps = deps,
        merged_deps = merged_deps,
    )

def _cc_internals():
    return struct(
        check_private_api = lambda *args, **kwargs: None,
        freeze = fjfj_freeze,
        create_header_info = _create_header_info,
        get_artifact_name_for_category = _get_artifact_name_for_category,
        actions2ctx_cheat = fjfj_actions_ctx,
        combine_cc_toolchain_variables = _combine_cc_variables,
        actions = _unavailable("cc_common.internal_DO_NOT_USE().actions", "buildfiji-136.15"),
        cc_toolchain_variables = lambda *, vars: _cc_variables(vars),
        dynamic_library_symlink = _dynamic_library_symlink,
        dynamic_library_symlink2 = _dynamic_library_symlink2,
        create_cc_compile_action = _create_cc_compile_action,
        is_tree_artifact = _is_tree_artifact,
        intern_string_sequence_variable_value = lambda values: values,
        wrap_link_actions = lambda actions, build_config = None, use_shareable_artifact_factory = False: actions,
        dynamic_library_soname = _dynamic_library_soname,
        declare_other_output_file = _declare_other_output_file,
        create_header_info_with_deps = _create_header_info_with_deps,
        compute_output_name_prefix_dir = _compute_output_name_prefix_dir,
        solib_symlink_action = _solib_symlink_action,
        rule_class = fjfj_rule_kind,
        per_file_copts = _per_file_copts,
        intern_seq = lambda values: values,
        get_link_args = _get_link_args,
        get_artifact_name_extension_for_category = _get_artifact_name_extension_for_category,
        expand_and_tokenize = _unavailable("cc_common.internal_DO_NOT_USE().expand_and_tokenize", "buildfiji-136.15"),
        exec_os = _exec_os,
        declare_compile_output_file = _declare_compile_output_file,
        create_lto_backend_action_template = _unavailable("cc_common.internal_DO_NOT_USE().create_lto_backend_action_template", "buildfiji-136.15"),
        create_lto_backend_action = _unavailable("cc_common.internal_DO_NOT_USE().create_lto_backend_action", "buildfiji-136.15"),
        create_cc_compile_action_template = _unavailable("cc_common.internal_DO_NOT_USE().create_cc_compile_action_template", "buildfiji-136.15"),
        collect_per_file_lto_backend_opts = _unavailable("cc_common.internal_DO_NOT_USE().collect_per_file_lto_backend_opts", "buildfiji-136.15"),
        check_toplevel = lambda *args, **kwargs: None,
        cc_toolchain_features = lambda *, toolchain_config_info, tools_directory: _cc_toolchain_features(toolchain_config_info, tools_directory),
        absolute_symlink = _absolute_symlink,
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
_TemplateVariableInfo, _raw_TemplateVariableInfo = provider(doc = "Make variables.", fields = ["variables"], init = lambda variables: {"variables": variables})
_ToolchainInfo = provider(doc = "The data of a toolchain.")
_FeatureFlagInfo = provider(doc = "A feature flag's value.", fields = ["value"])
_ExecutionInfo, _raw_execution_info = provider(
    doc = "How a test runs.",
    fields = ["requirements", "exec_group"],
    init = lambda *args, **kwargs: _bind_native("ExecutionInfo", [
        ("requirements", {}, "dict", "dict"),
        ("exec_group", "test", "string", "string"),
    ], args, kwargs),
)
_TestEnvironment = lambda *args, **kwargs: _raw_run_environment_info(**_run_environment_fields("TestEnvironment", args, kwargs))
_Objc = provider(doc = "Objective-C information.")
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

_XcodeVersionConfig, _raw_XcodeVersionConfig = provider(doc = "An xcode_config.", init = _xcode_version_config_init)

platform_common = struct(
    ConstraintSettingInfo = _ConstraintSettingInfo,
    ConstraintValueInfo = _ConstraintValueInfo,
    PlatformInfo = _PlatformInfo,
    TemplateVariableInfo = _TemplateVariableInfo,
    ToolchainInfo = _ToolchainInfo,
)

config_common = struct(
    FeatureFlagInfo = _FeatureFlagInfo,
    config_feature_flag_transition = fjfj_native_method("config_common", "config_feature_flag_transition", _unavailable("config_common.config_feature_flag_transition", "buildfiji-136.16")),
    toolchain_type = fjfj_native_method("config_common", "toolchain_type", _toolchain_type),
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
    instrumented_files_info = fjfj_native_method("coverage_common", "instrumented_files_info", _instrumented_files_info),
)

testing = struct(
    ExecutionInfo = _ExecutionInfo,
    TestEnvironment = fjfj_native_method("testing", "TestEnvironment", _TestEnvironment),
    analysis_test = fjfj_native_method("testing", "analysis_test", _unavailable("testing.analysis_test", "buildfiji-136.16")),
)

cc_common = struct(
    action_is_enabled = lambda *, feature_configuration, action_name: _action_is_enabled(feature_configuration, action_name),
    add_go_exec_groups_to_binary_rules = _unavailable("cc_common.add_go_exec_groups_to_binary_rules", "buildfiji-136.15"),
    check_experimental_cc_shared_library = _unavailable("cc_common.check_experimental_cc_shared_library", "buildfiji-136.15"),
    do_not_use_tools_cpp_compiler_present = None,
    empty_variables = lambda: _cc_variables({}),
    get_environment_variables = lambda *, feature_configuration, action_name, variables: _get_environment_variables(feature_configuration, action_name, variables),
    get_execution_requirements = lambda *, feature_configuration, action_name: _get_execution_requirements(feature_configuration, action_name),
    get_memory_inefficient_command_line = lambda *, feature_configuration, action_name, variables: _get_memory_inefficient_command_line(feature_configuration, action_name, variables),
    get_tool_for_action = lambda *, feature_configuration, action_name: _get_tool_for_action(feature_configuration, action_name),
    get_tool_requirement_for_action = lambda *, feature_configuration, action_name: _get_execution_requirements(feature_configuration, action_name),
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
    XcodeProperties = None,
    XcodeVersionConfig = _XcodeVersionConfig,
    apple_host_system_env = fjfj_builtins_function("apple_host_system_env", "common/objc/apple_env.bzl", _unavailable("apple_common.apple_host_system_env", "buildfiji-136.15")),
    apple_toolchain = fjfj_builtins_function("lambda", "common/objc/apple_common.bzl", lambda: struct(developer_dir = _unavailable("apple_common.apple_toolchain().developer_dir", "buildfiji-136.15"), platform_developer_framework_dir = _unavailable("apple_common.apple_toolchain().platform_developer_framework_dir", "buildfiji-136.15"), sdk_dir = _unavailable("apple_common.apple_toolchain().sdk_dir", "buildfiji-136.15"))),
    dotted_version = fjfj_builtins_function("lambda", "common/objc/apple_common.bzl", lambda version: str(version) if version else None),
    platform = _APPLE_PLATFORM,
    platform_type = _APPLE_PLATFORM_TYPE,
    new_objc_provider = _Objc,
    target_apple_env = fjfj_builtins_function("target_apple_env", "common/objc/apple_env.bzl", _unavailable("apple_common.target_apple_env", "buildfiji-136.15")),
)

android_common = struct(
    create_dex_merger_actions = _unavailable("android_common.create_dex_merger_actions", "buildfiji-136.15"),
    resource_source_directory = _unavailable("android_common.resource_source_directory", "buildfiji-136.15"),
)

# What Bazel 9.2.0's `proto_common_do_not_use` has, which rules_java reads.
proto_common_do_not_use = struct(INCOMPATIBLE_ENABLE_PROTO_TOOLCHAIN_RESOLUTION = True)

# A built-in function in Bazel; its transition is buildfiji-136.6's.
exec_transition = _unavailable("exec_transition", "buildfiji-136.6")

# Python's legacy_create_init: an empty `__init__.py` above every Python file
# in the runfiles tree. The tree is made later, from everything merged in by
# then, so this only marks the runfiles; see runfiles_tree.rs.
def _merge_runfiles_with_generated_inits(*, ctx, runfiles):
    return runfiles.merge(ctx.runfiles(_python_inits = True))

# An analysis test (`rule(analysis_test = True)`) returns an
# AnalysisTestResultInfo and no executable; Bazel makes the test's script: it
# says the message, each character escaped for sh, a line to each line of it,
# and exits 0 or 1. Called with the context once the implementation is done.
def _analysis_test_script(ctx, result):
    script = ctx.actions.declare_file(ctx.label.name + ".sh")
    lines = ["echo " + "".join(["\\" + c for c in line.elems()]) for line in result.message.split("\n")]
    status = "0" if result.success else "1"
    ctx.actions.write(script, "#!/bin/sh\n" + "\n".join(lines) + "\nexit " + status, is_executable = True)
    return script

# `ctx.actions.template_dict()`: substitutions for `expand_template` that are
# computed from lists. `_computed_substitutions` is what `expand_template`
# turns one into.
def _template_dict():
    entries = []

    def add(key, value):
        entries.append((key, [value], None, None, False, None))
        return None

    def add_joined(key, values, *, join_with, map_each, uniquify = False, format_joined = None, allow_closure = False):
        items = values.to_list() if type(values) == "depset" else list(values)
        entries.append((key, items, join_with, map_each, uniquify, format_joined))
        return None

    return struct(add = add, add_joined = add_joined, _entries = entries)

def _computed_substitutions(template_dict):
    out = {}
    for key, items, join_with, map_each, uniquify, format_joined in template_dict._entries:
        if join_with == None:
            out[key] = items[0]
            continue
        parts = []
        for item in items:
            mapped = map_each(item)
            if mapped == None:
                continue
            for part in (mapped if type(mapped) == "list" else [mapped]):
                if not uniquify or part not in parts:
                    parts.append(part)
        joined = join_with.join(parts)
        if format_joined != None:
            joined = format_joined % joined
        out[key] = joined
    return out

# What rules_python's `py_internal_renamed.bzl` re-exports: a native object in
# Bazel that only the python rules may use. Loading needs the symbol; calling
# a member is buildfiji-136.16's.
py_internal = struct(
    cc_toolchain_build_info_files = _unavailable("py_internal.cc_toolchain_build_info_files", "buildfiji-136.16"),
    # A file that does not change from build to build; here an ordinary one.
    declare_constant_metadata_file = lambda *, ctx, name, root: ctx.actions.declare_file(name),
    copy_without_caching = _unavailable("py_internal.copy_without_caching", "buildfiji-136.16"),
    create_repo_mapping_manifest = lambda *, ctx, runfiles, output: ctx.actions.write(output, fjfj_repo_mapping(ctx, runfiles)),
    declare_shareable_artifact = _unavailable("py_internal.declare_shareable_artifact", "buildfiji-136.16"),
    # RepositoryName.getRunfilesPath: nothing for the main repository, else "../<repo>".
    get_label_repo_runfiles_path = lambda label: ("../" + label.workspace_name + "/" if label.workspace_name else "") + label.package,
    get_legacy_external_runfiles = lambda ctx: False,
    # The few cc helpers rules_python reads. Stamping is not implemented
    # (buildfiji-ivq), so nothing is stamped.
    cc_helper = struct(
        is_valid_shared_library_artifact = lambda file: file.extension in ["so", "dylib", "dll", "pyd"] or file.basename.find(".so.") >= 0,
        is_stamping_enabled = lambda ctx: False,
        get_static_mode_params_for_dynamic_library_libraries = lambda libraries: libraries.to_list() if type(libraries) == "depset" else libraries,
    ),
    is_bzlmod_enabled = lambda ctx: True,
    # --legacy_external_runfiles is off, so the runfiles are as they are.
    make_runfiles_respect_legacy_external_runfiles = lambda ctx, runfiles: runfiles,
    merge_runfiles_with_generated_inits_empty_files_supplier = _merge_runfiles_with_generated_inits,
    is_singleton_depset = lambda files: len(files.to_list()) == 1,
    is_tool_configuration = lambda ctx: ctx.configuration.is_tool_configuration(),
    link = _unavailable("py_internal.link", "buildfiji-136.16"),
    linkstamp_file = _unavailable("py_internal.linkstamp_file", "buildfiji-136.16"),
    regex_match = _unavailable("py_internal.regex_match", "buildfiji-136.16"),
    runfiles_enabled = lambda ctx: True,
    share_native_deps = _unavailable("py_internal.share_native_deps", "buildfiji-136.16"),
    stamp_binaries = lambda ctx: False,
)

# The native rules whose analysis is Starlark (buildfiji-4qs): `ctx` is the rule's
# attributes as the native schema types them. The rules that stay Rust are the
# ones that read or write what a provider cannot say.
def _filegroup(ctx):
    files = depset(transitive = [src[DefaultInfo].files for src in ctx.attr.srcs])

    # As Bazel's filegroup has it, the runfiles are `data` alone, files and
    # what they bring along, not the `srcs`: rules_rust links the srcs into a
    # sysroot beside what it reads out of these.
    runfiles = ctx.runfiles().merge_all(
        [t[DefaultInfo].default_runfiles for t in ctx.attr.data] +
        [ctx.runfiles(transitive_files = t[DefaultInfo].files) for t in ctx.attr.data],
    )
    return [DefaultInfo(files = files, runfiles = runfiles)]

_native_implementations = {
    "filegroup": _filegroup,
}

# `ctx.fragments` (buildfiji-136.18): the options of the configuration as the
# structs Bazel's configuration fragments are. `options` is what the Rust side
# knows of the configuration; a field a rule reads that is not here is an
# error of the rule's, so it is added when one reads it.
def _make_fragments(options):
    macos = fjfj_apple_platform("macos")
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
        apple_platform_type = "macos",
        # The defaults of the flags, as probed on a Linux host.
        apple_cpus = struct(
            apple_split_cpu = "",
            catalyst_cpus = ("x86_64",),
            ios_multi_cpus = ("x86_64",),
            macos_cpus = ("x86_64",),
            tvos_cpus = ("x86_64",),
            visionos_cpus = ("sim_arm64",),
            watchos_cpus = ("x86_64",),
        ),
    )
    mode = options["compilation_mode"]

    # The options are joined by spaces; a value with one is split (buildfiji-136.16.1).
    def words(name):
        return options[name].split(" ") if options[name] else []

    # Bazel hides these members behind its private-API allowlist: they fail for
    # a file that is not on it (buildfiji-136.25). Each is a def of its own
    # because the error names the function that was called.
    def build_test_dwp():
        fjfj_check_private_api()
        return False

    def compilation_mode():
        fjfj_check_private_api()
        return mode

    def cs_fdo_instrument():
        fjfj_check_private_api()
        return None

    def cs_fdo_path():
        fjfj_check_private_api()
        return None

    def disable_nocopts():
        fjfj_check_private_api()
        return True

    def experimental_cc_implementation_deps():
        fjfj_check_private_api()
        return False

    def experimental_cpp_modules():
        fjfj_check_private_api()
        return False

    def experimental_link_static_libraries_once():
        fjfj_check_private_api()
        return False

    def fdo_instrument():
        fjfj_check_private_api("fdo_instrument")
        return None

    def fdo_path():
        fjfj_check_private_api()
        return None

    def fission_active_for_current_compilation_mode():
        fjfj_check_private_api()
        return bool(options["fission"])

    def force_pic():
        fjfj_check_private_api("force_pic")
        return bool(options["force_pic"])

    def generate_llvm_lcov():
        fjfj_check_private_api("generate_llvm_lcov")
        return False

    def grte_top():
        fjfj_check_private_api()
        return None

    def include_scanning():
        fjfj_check_private_api()
        return False

    def incompatible_remove_legacy_whole_archive():
        fjfj_check_private_api()
        return True

    def incompatible_use_specific_tool_files():
        fjfj_check_private_api()
        return True

    def interface_shared_objects():
        fjfj_check_private_api()
        return True

    def legacy_whole_archive():
        fjfj_check_private_api()
        return True

    def lto_index_options():
        fjfj_check_private_api()
        return []

    def objc_enable_binary_stripping():
        fjfj_check_private_api()
        return False

    def objc_should_generate_dotd_files():
        fjfj_check_private_api()
        return True

    def process_headers_in_dependencies():
        fjfj_check_private_api()
        return False

    def propeller_optimize_absolute_cc_profile():
        fjfj_check_private_api()
        return None

    def propeller_optimize_absolute_ld_profile():
        fjfj_check_private_api()
        return None

    def proto_profile():
        fjfj_check_private_api()
        return False

    def save_feature_state():
        fjfj_check_private_api()
        return False

    def save_temps():
        fjfj_check_private_api()
        return bool(options["save_temps"])

    def share_native_deps():
        fjfj_check_private_api()
        return True

    def should_generate_dotd_files():
        fjfj_check_private_api()
        return True

    def should_strip_binaries():
        # --strip=sometimes strips in fastbuild.
        fjfj_check_private_api()
        return options["strip"] == "always" or (options["strip"] != "never" and mode == "fastbuild")

    def start_end_lib():
        fjfj_check_private_api()
        return True

    def strip_opts():
        fjfj_check_private_api()
        return []

    def use_llvm_coverage_map_format():
        fjfj_check_private_api()
        return False

    cpp = struct(
        # Members Bazel hides behind an allowlist have the value of their flag's
        # default; the others were probed on Bazel 9.2.0.
        apple_generate_dsym = False,
        build_test_dwp = build_test_dwp,
        compilation_mode = compilation_mode,
        conlyopts = words("conlyopt"),
        copts = words("copt"),
        cs_fdo_instrument = cs_fdo_instrument,
        cs_fdo_path = cs_fdo_path,
        custom_malloc = None,
        cxxopts = words("cxxopt"),
        disable_nocopts = disable_nocopts,
        do_not_use_macos_set_install_name = False,
        dynamic_mode = lambda: "DEFAULT",
        experimental_cc_implementation_deps = experimental_cc_implementation_deps,
        experimental_cpp_modules = experimental_cpp_modules,
        experimental_link_static_libraries_once = experimental_link_static_libraries_once,
        fdo_instrument = fdo_instrument,
        fdo_path = fdo_path,
        fission_active_for_current_compilation_mode = fission_active_for_current_compilation_mode,
        force_pic = force_pic,
        generate_llvm_lcov = generate_llvm_lcov,
        grte_top = grte_top,
        include_scanning = include_scanning,
        incompatible_remove_legacy_whole_archive = incompatible_remove_legacy_whole_archive,
        incompatible_use_specific_tool_files = incompatible_use_specific_tool_files,
        interface_shared_objects = interface_shared_objects,
        legacy_whole_archive = legacy_whole_archive,
        linkopts = words("linkopt"),
        lto_backend_options = [],
        lto_index_options = lto_index_options,
        minimum_os_version = lambda: None,
        objc_enable_binary_stripping = objc_enable_binary_stripping,
        objc_generate_linkmap = False,
        objc_should_generate_dotd_files = objc_should_generate_dotd_files,
        objc_should_strip_binary = False,
        objccopts = [],
        process_headers_in_dependencies = process_headers_in_dependencies,
        propeller_optimize_absolute_cc_profile = propeller_optimize_absolute_cc_profile,
        propeller_optimize_absolute_ld_profile = propeller_optimize_absolute_ld_profile,
        proto_profile = proto_profile,
        save_feature_state = save_feature_state,
        save_temps = save_temps,
        share_native_deps = share_native_deps,
        should_generate_dotd_files = should_generate_dotd_files,
        should_strip_binaries = should_strip_binaries,
        start_end_lib = start_end_lib,
        strip_opts = strip_opts,
        use_llvm_coverage_map_format = use_llvm_coverage_map_format,
        _dont_enable_host_nonhost = True,
        _fdo_prefetch_hints_label = None,
    )
    return struct(
        apple = apple,
        cpp = cpp,
        platform = struct(_os = options["os"]),
        java = struct(),
        proto = struct(experimental_protoc_opts = []),
        coverage = struct(),
        # Bazel 9.2.0 has these too and no `py`; nothing reads them here.
        android = struct(),
        bazel_android = struct(),
        j2objc = struct(),
        objc = struct(),
    )

# `ctx.configuration`.
def _make_configuration(options):
    # Bazel hides it behind its private-API allowlist (buildfiji-136.25).
    def stamp_binaries():
        fjfj_check_private_api()
        return bool(options["stamp"])

    return struct(
        coverage_enabled = bool(options["collect_code_coverage"]),
        default_shell_env = dict([line.split("=", 1) for line in options["default_shell_env"].split("\n")]),
        host_path_separator = ":",
        short_id = options["short_id"],
        bin_dir = fjfj_root(options["bin_dir"]),
        genfiles_dir = fjfj_root(options["bin_dir"]),
        disabled_features = lambda: [],
        test_env = dict([line.split("=", 1) for line in options["test_env"].split("\n") if line]),
        is_tool_configuration = lambda: bool(options["exec"]),
        stamp_binaries = stamp_binaries,
        is_sibling_repository_layout = lambda: False,
        has_separate_genfiles_directory = lambda: False,
        runfiles_enabled = lambda: True,
    )
