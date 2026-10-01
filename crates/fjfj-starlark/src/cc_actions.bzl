# The C++ internals that make files and actions (buildfiji-136.16.4): what
# Bazel's CcModule does in Java for rules_cc's Starlark `cc_common.compile` and
# `cc_common.link`. The features and variables they expand are cc_features.bzl's.

load(
    "@_builtins//:cc_features.bzl",
    _get_environment_variables = "get_environment_variables",
    _get_execution_requirements = "get_execution_requirements",
    _get_memory_inefficient_command_line = "get_memory_inefficient_command_line",
    _get_tool_for_action = "get_tool_for_action",
)

# The prefix and the extension an artifact of each category has unless the
# toolchain says otherwise (rules_cc's own table of ArtifactCategory).
_DEFAULT_NAMES = {
    "STATIC_LIBRARY": ("lib", ".a"),
    "ALWAYSLINK_STATIC_LIBRARY": ("lib", ".lo"),
    "DYNAMIC_LIBRARY": ("lib", ".so"),
    "EXECUTABLE": ("", ""),
    "INTERFACE_LIBRARY": ("lib", ".ifso"),
    "PIC_FILE": ("", ".pic"),
    "INCLUDED_FILE_LIST": ("", ".d"),
    "SERIALIZED_DIAGNOSTICS_FILE": ("", ".dia"),
    "OBJECT_FILE": ("", ".o"),
    "PIC_OBJECT_FILE": ("", ".pic.o"),
    "CPP_MODULE": ("", ".pcm"),
    "CPP_MODULES_INFO": ("", ".CXXModules.json"),
    "CPP_MODULES_DDI": ("", ".ddi"),
    "CPP_MODULES_MODMAP": ("", ".modmap"),
    "CPP_MODULES_MODMAP_INPUT": ("", ".modmap.input"),
    "GENERATED_ASSEMBLY": ("", ".s"),
    "PROCESSED_HEADER": ("", ".processed"),
    "GENERATED_HEADER": ("", ".h"),
    "PREPROCESSED_C_SOURCE": ("", ".i"),
    "PREPROCESSED_CPP_SOURCE": ("", ".ii"),
    "COVERAGE_DATA_FILE": ("", ".gcno"),
    "CLIF_OUTPUT_PROTO": ("", ".opb"),
}

def _name_pattern(cc_toolchain, category):
    # A toolchain's artifact_name_patterns name categories in lower case.
    for pattern in cc_toolchain._toolchain_features._artifact_name_patterns:
        if pattern.category_name == category.lower():
            return (pattern.prefix, pattern.extension)
    if category not in _DEFAULT_NAMES:
        fail("Unknown artifact category: %s" % category)
    return _DEFAULT_NAMES[category]

def get_artifact_name_for_category(cc_toolchain, category, output_name):
    prefix, extension = _name_pattern(cc_toolchain, category)
    directory, _, base = output_name.rpartition("/")
    name = prefix + base + extension
    return directory + "/" + name if directory else name

def get_artifact_name_extension_for_category(cc_toolchain, category):
    return _name_pattern(cc_toolchain, category)[1]

def compute_output_name_prefix_dir(configuration, purpose):
    # An object file of a cc_library is _objs/<name>/<base>.o (probed on Bazel
    # 9.2.0): no directory of the purpose between.
    return ""

def declare_compile_output_file(ctx, label, output_name, configuration):
    return ctx.actions.declare_file("_objs/" + label.name + "/" + output_name)

def declare_other_output_file(ctx, output_name, object_file):
    return ctx.actions.declare_file(output_name, sibling = object_file)

def is_tree_artifact(file):
    return file.is_directory

def per_file_copts(cpp_configuration, source_file, label):
    # --per_file_copt is not supported.
    return []

_MNEMONICS = {
    "c++-module-compile": "CppModuleCompile",
    "c++20-module-compile": "CppModuleCompile",
    "linkstamp-compile": "CppLinkstampCompile",
    "c++-module-deps-scanning": "CppModuleDepScanning",
}

def _as_list(value):
    if value == None:
        return []
    if type(value) == "depset":
        return value.to_list()
    return list(value)

def _compile_action_name(source):
    """The action that compiles `source`, by its extension (CppFileTypes)."""
    extension = "." + source.extension if source != None and source.extension else ""
    if extension == ".c":
        return "c-compile"
    if extension in (".s", ".asm"):
        return "assemble"
    if extension == ".S":
        return "preprocess-assemble"
    if extension == ".m":
        return "objc-compile"
    if extension == ".mm":
        return "objc++-compile"
    return "c++-compile"

def create_cc_compile_action(
        *,
        action_construction_context,
        cc_compilation_context,
        cc_toolchain,
        feature_configuration,
        output_file,
        compile_build_variables,
        action_name = None,
        configuration = None,
        source = None,
        copts_filter = None,
        additional_compilation_inputs = None,
        additional_compilation_inputs_set = None,
        additional_include_scanning_roots = None,
        dotd_file = None,
        diagnostics_file = None,
        gcno_file = None,
        dwo_file = None,
        lto_indexing_file = None,
        use_pic = False,
        needs_include_validation = False,
        toolchain_type = None,
        cache_key_inputs = None,
        build_info_header_files = None,
        should_scan_includes = True,
        shareable = False,
        additional_outputs = None,
        module_files = None,
        modmap_file = None,
        modmap_input_file = None,
        **_unused):
    ctx = action_construction_context
    if not action_name:
        action_name = _compile_action_name(source)
    tool = _get_tool_for_action(feature_configuration, action_name)
    arguments = _get_memory_inefficient_command_line(feature_configuration, action_name, compile_build_variables)
    env = _get_environment_variables(feature_configuration, action_name, compile_build_variables)
    requirements = {}
    for requirement in _get_execution_requirements(feature_configuration, action_name):
        requirements[requirement] = ""

    direct = [f for f in [source] if f]
    direct.extend(_as_list(additional_compilation_inputs))
    direct.extend(_as_list(additional_compilation_inputs_set))
    direct.extend(_as_list(cache_key_inputs))
    direct.extend(_as_list(module_files))
    direct.extend([f for f in [modmap_file, modmap_input_file] if f])
    transitive = [cc_toolchain._compiler_files, cc_compilation_context.headers]
    if build_info_header_files != None:
        transitive.append(build_info_header_files)
    outputs = [f for f in [output_file, dotd_file, diagnostics_file, gcno_file, dwo_file, lto_indexing_file] if f]
    outputs.extend(_as_list(additional_outputs))

    ctx.actions.run(
        executable = tool,
        arguments = arguments,
        inputs = depset(direct, transitive = transitive),
        outputs = outputs,
        env = env,
        execution_requirements = requirements,
        use_default_shell_env = True,
        mnemonic = _MNEMONICS.get(action_name, "CppCompile"),
        progress_message = "Compiling %s" % (source.short_path if source else output_file.short_path),
    )

def get_link_args(*, feature_configuration, action_name, build_variables, parameter_file_type):
    """The command line of a link as an Args, in a parameter file if the type says so."""
    args = fjfj_new_args()
    # The toolchain writes `@<param file>` where the arguments after it may go
    # to a file if the line is too long (`LINKER_PARAM_FILE_PLACEHOLDER`);
    # run inline, they are all just arguments.
    line = _get_memory_inefficient_command_line(feature_configuration, action_name, build_variables)
    args.add_all([a for a in line if a != "@LINKER_PARAM_FILE_PLACEHOLDER"])
    if parameter_file_type != None:
        # Whether it is written is the Args': the command line is split when it
        # would be too long, which fjfj does not measure yet.
        args.use_param_file("@%s", use_always = False)
        args.set_param_file_format("multiline" if parameter_file_type == "UNQUOTED" else "shell")
    return args

# ---- the solib directory -------------------------------------------------------------

def _escape(text):
    """Actions.escapedPath: what a path or label looks like as one directory name."""
    return text.replace("_", "_U").replace("/", "_S").replace("\\", "_B").replace(":", "_C").replace("@", "_A")

def dynamic_library_soname(actions, path, preserve_name):
    """The name a dynamic library is linked under (SolibSymlinkAction)."""
    base = path.rpartition("/")[2]
    if preserve_name:
        return base
    return "lib" + _escape(path[:-len(base)] + base.removeprefix("lib")) if path else base

def _solib_symlink(actions, library, solib_dir, relative):
    output = actions.declare_shareable_artifact(solib_dir + "/" + relative)
    actions.symlink(output = output, target_file = library)
    return output

def dynamic_library_symlink(actions, library, solib_dir, preserve_name, prefix_consumer):
    """A symlink to `library` under the solib directory, named after its owner."""
    label = fjfj_actions_ctx(actions).label
    owner = "_U_S" + _escape(label.package) + "_C" + _escape(label.name) + "___U" + _escape(label.workspace_name)
    return _solib_symlink(actions, library, solib_dir, owner + "/" + dynamic_library_soname(actions, library.short_path, preserve_name))

def dynamic_library_symlink2(actions, library, solib_dir, symlink_path):
    return _solib_symlink(actions, library, solib_dir, symlink_path)

def solib_symlink_action(*, ctx, artifact, solib_directory, runtime_solib_dir_base):
    return _solib_symlink(ctx.actions, artifact, solib_directory, runtime_solib_dir_base + "/" + artifact.basename)

def absolute_symlink(*, ctx, output, target_path, progress_message):
    ctx.actions.symlink(output = output, target_path = target_path, progress_message = progress_message)
