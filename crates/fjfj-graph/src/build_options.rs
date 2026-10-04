//! The options Bazel 9.2.0's configuration holds and what each is set to with
//! no flags, as `build_options(target)` of `cquery --output=starlark` shows
//! them (probed on a bare workspace). The options of fragments fjfj has no use
//! for, Android and Apple among them, are here so the keys match; their values
//! are what Bazel starts at. Options whose values Bazel shows only as an opaque
//! Java enum (`<unknown object ...>`) are the empty string.

use crate::SettingValue;

/// How a default is spelled in the table.
pub(crate) enum Default {
    Bool(bool),
    Int(i64),
    Str(&'static str),
    Label(&'static str),
    List(&'static [&'static str]),
    None,
}

impl Default {
    pub(crate) fn value(&self) -> SettingValue {
        match self {
            Default::Bool(b) => SettingValue::Bool(*b),
            Default::Int(i) => SettingValue::Int(*i),
            Default::Str(s) => SettingValue::Str((*s).to_owned()),
            Default::Label(l) => SettingValue::Label((*l).to_owned()),
            Default::List(items) => {
                SettingValue::List(items.iter().map(|s| (*s).to_owned()).collect())
            }
            Default::None => SettingValue::None,
        }
    }
}

/// Every option but the five fjfj computes (`cpu`, `compilation_mode`,
/// `define`, `action_env`, `is exec configuration`), by name.
pub(crate) const DEFAULTS: &[(&str, Default)] = &[
    ("Android configuration distinguisher", Default::Str("")),
    ("affected by starlark transition", Default::List(&[])),
    (
        "all feature flag values are present (internal)",
        Default::Bool(true),
    ),
    ("allow_analysis_failures", Default::Bool(false)),
    ("allow_unresolved_symlinks", Default::Bool(true)),
    ("allowed_cpu_values", Default::List(&[])),
    ("analysis_testing_deps_limit", Default::Int(2000)),
    ("android_compiler", Default::None),
    ("android_databinding_use_androidx", Default::Bool(true)),
    ("android_databinding_use_v3_4_args", Default::Bool(true)),
    ("android_dynamic_mode", Default::Str("")),
    ("android_fixed_resource_neverlinking", Default::Bool(true)),
    ("android_manifest_merger", Default::Str("")),
    ("android_manifest_merger_order", Default::Str("")),
    ("android_migration_tag_check", Default::Bool(false)),
    ("android_platforms", Default::List(&[])),
    ("android_resource_shrinking", Default::Bool(false)),
    ("apple configuration distinguisher", Default::Str("")),
    ("apple_generate_dsym", Default::Bool(false)),
    ("apple_platform_type", Default::Str("macos")),
    ("apple_platforms", Default::List(&[])),
    ("apple_split_cpu", Default::Str("")),
    (
        "break_build_on_parallel_dex2oat_failure",
        Default::Bool(false),
    ),
    ("build_python_zip", Default::Str("")),
    ("build_runfile_links", Default::Bool(true)),
    ("build_runfile_manifests", Default::Bool(true)),
    ("build_test_dwp", Default::Bool(false)),
    ("bytecode_optimization_pass_actions", Default::Int(1)),
    ("catalyst_cpus", Default::List(&[])),
    ("cc_dotd_files", Default::Bool(true)),
    ("cc_include_scanning", Default::Bool(false)),
    ("cc_output_directory_tag", Default::Str("")),
    (
        "cc_proto_library_header_suffixes",
        Default::List(&[".pb.h"]),
    ),
    (
        "cc_proto_library_source_suffixes",
        Default::List(&[".pb.cc"]),
    ),
    ("check_licenses", Default::Bool(false)),
    ("check_visibility", Default::Bool(true)),
    ("collect_code_coverage", Default::Bool(false)),
    ("compiler", Default::None),
    ("conlyopt", Default::List(&[])),
    ("copt", Default::List(&[])),
    (
        "coverage_output_generator",
        Default::Label("@@bazel_tools//tools/test:lcov_merger"),
    ),
    (
        "coverage_report_generator",
        Default::Label("@@bazel_tools//tools/test:coverage_report_generator"),
    ),
    (
        "crosstool_top",
        Default::Label("@@bazel_tools//tools/cpp:toolchain"),
    ),
    ("cs_fdo_absolute_path", Default::None),
    ("cs_fdo_instrument", Default::None),
    ("cs_fdo_profile", Default::None),
    ("custom_malloc", Default::None),
    ("cxxopt", Default::List(&[])),
    ("desugar_for_android", Default::Bool(true)),
    ("desugar_java8_libs", Default::Bool(false)),
    ("device_debug_entitlements", Default::Bool(true)),
    (
        "dexopts_supported_in_dexmerger",
        Default::List(&["--minimal-main-dex", "--set-max-idx-number"]),
    ),
    (
        "dexopts_supported_in_dexsharder",
        Default::List(&["--minimal-main-dex"]),
    ),
    (
        "dexopts_supported_in_incremental_dexing",
        Default::List(&["--no-optimize", "--no-locals"]),
    ),
    ("dynamic_mode", Default::Str("")),
    (
        "enable_propeller_optimize_absolute_paths",
        Default::Bool(true),
    ),
    ("enable_remaining_fdo_absolute_paths", Default::Bool(true)),
    ("enable_runfiles", Default::Str("")),
    ("enforce_constraints", Default::Bool(true)),
    ("enforce_proguard_file_extension", Default::Bool(false)),
    (
        "enforce_transitive_configs_for_config_feature_flag",
        Default::Bool(false),
    ),
    ("evaluating for analysis test", Default::Bool(false)),
    ("exec_aspects", Default::List(&[])),
    ("experimental_action_listener", Default::List(&[])),
    (
        "experimental_add_test_support_to_compile_time_deps",
        Default::Bool(true),
    ),
    (
        "experimental_allow_android_library_deps_without_srcs",
        Default::Bool(false),
    ),
    ("experimental_allow_map_directory", Default::Bool(true)),
    (
        "experimental_always_filter_duplicate_classes_from_android_test",
        Default::Bool(false),
    ),
    (
        "experimental_android_assume_minsdkversion",
        Default::Bool(false),
    ),
    (
        "experimental_android_compress_java_resources",
        Default::Bool(false),
    ),
    ("experimental_android_databinding_v2", Default::Bool(true)),
    (
        "experimental_android_library_exports_manifest_default",
        Default::Bool(false),
    ),
    (
        "experimental_android_resource_cycle_shrinking",
        Default::Bool(false),
    ),
    (
        "experimental_android_resource_name_obfuscation",
        Default::Bool(false),
    ),
    (
        "experimental_android_resource_path_shortening",
        Default::Bool(false),
    ),
    (
        "experimental_android_resource_shrinking",
        Default::Bool(false),
    ),
    (
        "experimental_android_rewrite_dexes_with_rex",
        Default::Bool(false),
    ),
    (
        "experimental_android_use_parallel_dex2oat",
        Default::Bool(false),
    ),
    ("experimental_cc_implementation_deps", Default::Bool(true)),
    ("experimental_check_desugar_deps", Default::Bool(true)),
    (
        "experimental_collect_code_coverage_for_generated_files",
        Default::Bool(false),
    ),
    (
        "experimental_cpp_compile_resource_estimation",
        Default::Bool(false),
    ),
    ("experimental_cpp_modules", Default::Bool(false)),
    (
        "experimental_debug_selects_always_succeed",
        Default::Bool(false),
    ),
    (
        "experimental_disable_instrumentation_manifest_merge",
        Default::Bool(false),
    ),
    ("experimental_enable_jspecify", Default::Bool(true)),
    (
        "experimental_enforce_transitive_visibility",
        Default::Bool(false),
    ),
    (
        "experimental_exclude_defines_from_exec_config",
        Default::Bool(false),
    ),
    (
        "experimental_exec_config",
        Default::Str("@_builtins//:common/builtin_exec_platforms.bzl%bazel_exec_transition"),
    ),
    (
        "experimental_exec_configuration_distinguisher",
        Default::Str(""),
    ),
    ("experimental_extended_sanity_checks", Default::Bool(false)),
    (
        "experimental_filter_library_jar_with_program_jar",
        Default::Bool(false),
    ),
    (
        "experimental_filter_r_jars_from_android_test",
        Default::Bool(false),
    ),
    ("experimental_fix_deps_tool", Default::Str("add_dep")),
    ("experimental_generate_llvm_lcov", Default::Bool(false)),
    (
        "experimental_get_android_java_resources_from_optimized_jar",
        Default::Bool(false),
    ),
    (
        "experimental_include_xcode_execution_requirements",
        Default::Bool(false),
    ),
    (
        "experimental_incremental_dexing_after_proguard",
        Default::Int(50),
    ),
    (
        "experimental_incremental_dexing_after_proguard_by_default",
        Default::Bool(true),
    ),
    ("experimental_inmemory_dotd_files", Default::Bool(true)),
    ("experimental_inmemory_jdeps_files", Default::Bool(true)),
    ("experimental_java_classpath", Default::Str("")),
    (
        "experimental_java_test_auto_create_deploy_jar",
        Default::Bool(false),
    ),
    (
        "experimental_link_static_libraries_once",
        Default::Bool(true),
    ),
    (
        "experimental_local_java_optimization_configuration",
        Default::None,
    ),
    (
        "experimental_local_java_optimizations",
        Default::Bool(false),
    ),
    (
        "experimental_objc_fastbuild_options",
        Default::List(&["-O0", "-DDEBUG=1"]),
    ),
    (
        "experimental_objc_provider_from_linked",
        Default::Bool(false),
    ),
    (
        "experimental_omit_resources_info_provider_from_android_binary",
        Default::Bool(false),
    ),
    ("experimental_omitfp", Default::Bool(false)),
    ("experimental_one_version_enforcement", Default::Str("")),
    (
        "experimental_one_version_enforcement_use_transitive_jars_for_binary_under_test",
        Default::Bool(false),
    ),
    (
        "experimental_output_directory_naming_scheme",
        Default::Str(""),
    ),
    ("experimental_output_paths", Default::Str("")),
    (
        "experimental_override_platform_cpu_name",
        Default::List(&[]),
    ),
    (
        "experimental_persistent_aar_extractor",
        Default::Bool(false),
    ),
    ("experimental_platform_in_output_dir", Default::Str("")),
    ("experimental_prefer_mutual_xcode", Default::Bool(true)),
    ("experimental_propagate_custom_flag", Default::List(&[])),
    (
        "experimental_proto_descriptor_sets_include_source_info",
        Default::Bool(false),
    ),
    (
        "experimental_py_binaries_include_label",
        Default::Bool(false),
    ),
    (
        "experimental_python_import_all_repositories",
        Default::Bool(true),
    ),
    (
        "experimental_remotable_source_manifests",
        Default::Bool(false),
    ),
    (
        "experimental_remove_r_classes_from_instrumentation_test_jar",
        Default::Bool(true),
    ),
    (
        "experimental_run_android_lint_on_java_rules",
        Default::Bool(false),
    ),
    ("experimental_save_feature_state", Default::Bool(false)),
    ("experimental_strict_fileset_output", Default::Bool(false)),
    ("experimental_strict_java_deps", Default::Str("")),
    (
        "experimental_throttle_action_cache_check",
        Default::Bool(true),
    ),
    (
        "experimental_turbine_annotation_processing",
        Default::Bool(false),
    ),
    (
        "experimental_unsupported_and_brittle_include_scanning",
        Default::Bool(false),
    ),
    (
        "experimental_use_cpp_compile_action_args_params_file",
        Default::Bool(false),
    ),
    (
        "experimental_use_dex_splitter_for_incremental_dexing",
        Default::Bool(true),
    ),
    ("experimental_use_llvm_covmap", Default::Bool(false)),
    (
        "experimental_use_platforms_in_output_dir_legacy_heuristic",
        Default::Bool(true),
    ),
    (
        "experimental_use_rtxt_from_merged_resources",
        Default::Bool(false),
    ),
    ("experimental_writable_outputs", Default::Bool(false)),
    ("explicit_java_test_deps", Default::Bool(false)),
    ("extra_execution_platforms", Default::List(&[])),
    ("extra_toolchains", Default::List(&[])),
    ("fat_apk_hwasan", Default::Bool(false)),
    ("fdo_instrument", Default::None),
    ("fdo_optimize", Default::None),
    ("fdo_prefetch_hints", Default::None),
    ("fdo_profile", Default::None),
    ("features", Default::List(&[])),
    ("fission", Default::List(&[])),
    ("force_pic", Default::Bool(false)),
    ("grte_top", Default::None),
    ("host_action_env", Default::List(&[])),
    ("host_compilation_mode", Default::Str("opt")),
    ("host_compiler", Default::None),
    ("host_conlyopt", Default::List(&[])),
    ("host_copt", Default::List(&[])),
    ("host_cpu", Default::Str("k8")),
    ("host_cxxopt", Default::List(&[])),
    ("host_features", Default::List(&[])),
    ("host_grte_top", Default::None),
    ("host_java_launcher", Default::None),
    ("host_javacopt", Default::List(&[])),
    ("host_jvmopt", Default::List(&[])),
    ("host_linkopt", Default::List(&[])),
    ("host_macos_minimum_os", Default::None),
    ("host_per_file_copt", Default::List(&[])),
    (
        "host_platform",
        Default::Label("@@bazel_tools//tools:host_platform"),
    ),
    ("include_config_fragments_provider", Default::Str("")),
    (
        "incompatible_always_include_files_in_data",
        Default::Bool(true),
    ),
    ("incompatible_auto_exec_groups", Default::Bool(false)),
    (
        "incompatible_avoid_hardcoded_objc_compilation_flags",
        Default::Bool(true),
    ),
    (
        "incompatible_bazel_test_exec_run_under",
        Default::Bool(true),
    ),
    ("incompatible_bep_cpu_from_platform", Default::Bool(false)),
    (
        "incompatible_builtin_objc_strip_action",
        Default::Bool(true),
    ),
    (
        "incompatible_check_testonly_for_output_files",
        Default::Bool(false),
    ),
    (
        "incompatible_compact_repo_mapping_manifest",
        Default::Bool(true),
    ),
    (
        "incompatible_default_to_explicit_init_py",
        Default::Bool(false),
    ),
    (
        "incompatible_disable_legacy_cc_provider",
        Default::Bool(true),
    ),
    (
        "incompatible_disable_native_android_rules",
        Default::Bool(false),
    ),
    (
        "incompatible_disable_native_apple_binary_rule",
        Default::Bool(false),
    ),
    ("incompatible_disable_nocopts", Default::Bool(true)),
    ("incompatible_disable_select_on", Default::List(&[])),
    (
        "incompatible_disallow_java_import_exports",
        Default::Bool(false),
    ),
    (
        "incompatible_disallow_sdk_frameworks_attributes",
        Default::Bool(false),
    ),
    (
        "incompatible_dont_enable_host_nonhost_crosstool_features",
        Default::Bool(true),
    ),
    (
        "incompatible_enable_apple_toolchain_resolution",
        Default::Bool(false),
    ),
    (
        "incompatible_enable_cc_toolchain_resolution",
        Default::Bool(true),
    ),
    (
        "incompatible_exclude_starlark_flags_from_exec_config",
        Default::Bool(false),
    ),
    (
        "incompatible_filegroup_runfiles_for_data",
        Default::Bool(true),
    ),
    (
        "incompatible_limit_platforms_in_output_dir_to",
        Default::List(&[]),
    ),
    (
        "incompatible_make_thinlto_command_lines_standalone",
        Default::Bool(true),
    ),
    ("incompatible_merge_genfiles_directory", Default::Bool(true)),
    (
        "incompatible_modify_execution_info_additive",
        Default::Bool(true),
    ),
    (
        "incompatible_multi_release_deploy_jars",
        Default::Bool(true),
    ),
    (
        "incompatible_objc_alwayslink_by_default",
        Default::Bool(false),
    ),
    (
        "incompatible_python_disallow_native_rules",
        Default::Bool(false),
    ),
    (
        "incompatible_remove_ctx_android_fragment",
        Default::Bool(false),
    ),
    (
        "incompatible_remove_ctx_bazel_py_fragment",
        Default::Bool(true),
    ),
    ("incompatible_remove_ctx_py_fragment", Default::Bool(true)),
    (
        "incompatible_remove_legacy_whole_archive",
        Default::Bool(true),
    ),
    (
        "incompatible_require_ctx_in_configure_features",
        Default::Bool(true),
    ),
    ("incompatible_strict_action_env", Default::Bool(true)),
    ("incompatible_strip_executable_safely", Default::Bool(false)),
    ("incompatible_target_cpu_from_platform", Default::Bool(true)),
    (
        "incompatible_use_cpp_compile_header_mnemonic",
        Default::Bool(false),
    ),
    ("incompatible_use_specific_tool_files", Default::Bool(true)),
    (
        "incompatible_use_toolchain_resolution_for_java_rules",
        Default::Bool(true),
    ),
    (
        "incompatible_validate_top_level_header_inclusions",
        Default::Bool(true),
    ),
    ("incremental_dexing", Default::Bool(true)),
    ("instrument_test_targets", Default::Bool(false)),
    ("interface_shared_objects", Default::Bool(true)),
    (
        "internal_persistent_android_dex_desugar",
        Default::Bool(false),
    ),
    ("internal_persistent_busybox_tools", Default::Bool(false)),
    (
        "internal_persistent_multiplex_android_dex_desugar",
        Default::Bool(false),
    ),
    (
        "internal_persistent_multiplex_busybox_tools",
        Default::Bool(false),
    ),
    ("ios_memleaks", Default::Bool(false)),
    ("ios_minimum_os", Default::None),
    ("ios_multi_cpus", Default::List(&[])),
    ("ios_sdk_version", Default::None),
    ("ios_signing_cert_name", Default::None),
    ("ios_simulator_device", Default::None),
    ("ios_simulator_version", Default::None),
    ("j2objc_dead_code_report", Default::None),
    ("j2objc_translation_flags", Default::List(&[])),
    ("java_debug", Default::None),
    ("java_deps", Default::Bool(true)),
    ("java_header_compilation", Default::Bool(true)),
    ("java_language_version", Default::Str("")),
    ("java_launcher", Default::None),
    ("java_runtime_version", Default::Str("local_jdk")),
    ("javacopt", Default::List(&[])),
    ("jvmopt", Default::List(&[])),
    ("legacy_main_dex_list_generator", Default::None),
    ("legacy_whole_archive", Default::Bool(true)),
    ("linkopt", Default::List(&[])),
    ("ltobackendopt", Default::List(&[])),
    ("ltoindexopt", Default::List(&[])),
    ("macos_cpus", Default::List(&[])),
    ("macos_minimum_os", Default::None),
    ("macos_sdk_version", Default::None),
    ("memprof_profile", Default::None),
    ("merge_android_manifest_permissions", Default::Bool(false)),
    ("min_param_file_size", Default::Int(32768)),
    ("minimum_os_version", Default::None),
    ("modify_execution_info", Default::List(&[])),
    (
        "non_incremental_per_target_dexopts",
        Default::List(&["--positions"]),
    ),
    ("objc_debug_with_GLIBCXX", Default::Bool(false)),
    ("objc_enable_binary_stripping", Default::Bool(false)),
    ("objc_generate_linkmap", Default::Bool(false)),
    ("objc_use_dotd_pruning", Default::Bool(true)),
    ("objccopt", Default::List(&[])),
    ("one_version_enforcement_on_java_tests", Default::Bool(true)),
    ("optimizing_dexer", Default::None),
    ("output_library_merged_assets", Default::Bool(true)),
    ("per_file_copt", Default::List(&[])),
    ("per_file_ltobackendopt", Default::List(&[])),
    ("persistent_android_dex_desugar", Default::None),
    ("persistent_android_resource_processor", Default::None),
    ("persistent_multiplex_android_dex_desugar", Default::None),
    (
        "persistent_multiplex_android_resource_processor",
        Default::None,
    ),
    ("persistent_multiplex_android_tools", Default::None),
    ("platform_suffix", Default::None),
    (
        "platforms",
        Default::List(&["@@bazel_tools//tools:host_platform"]),
    ),
    ("plugin", Default::List(&[])),
    ("process_headers_in_dependencies", Default::Bool(false)),
    ("proguard_top", Default::None),
    ("propeller_optimize", Default::None),
    ("propeller_optimize_absolute_cc_profile", Default::None),
    ("propeller_optimize_absolute_ld_profile", Default::None),
    (
        "proto_compiler",
        Default::Label("@@bazel_tools//tools/proto:protoc"),
    ),
    ("proto_profile", Default::Bool(true)),
    ("proto_profile_path", Default::None),
    (
        "proto_toolchain_for_cc",
        Default::Label("@@bazel_tools//tools/proto:cc_toolchain"),
    ),
    (
        "proto_toolchain_for_j2objc",
        Default::Label("@@bazel_tools//tools/j2objc:j2objc_proto_toolchain"),
    ),
    (
        "proto_toolchain_for_java",
        Default::Label("@@bazel_tools//tools/proto:java_toolchain"),
    ),
    (
        "proto_toolchain_for_javalite",
        Default::Label("@@bazel_tools//tools/proto:javalite_toolchain"),
    ),
    ("protocopt", Default::List(&[])),
    ("python_native_rules_allowlist", Default::None),
    ("python_path", Default::None),
    ("run_under", Default::None),
    ("save_temps", Default::Bool(false)),
    ("scl_config", Default::None),
    ("share_native_deps", Default::Bool(true)),
    ("shell_executable", Default::None),
    ("split_bytecode_optimization_pass", Default::Bool(false)),
    ("stamp", Default::Bool(false)),
    ("start_end_lib", Default::Bool(true)),
    ("strict_filesets", Default::Bool(false)),
    ("strict_proto_deps", Default::Str("")),
    ("strict_public_imports", Default::Str("")),
    ("strict_system_includes", Default::Bool(false)),
    ("strip", Default::Str("")),
    ("stripopt", Default::List(&[])),
    ("target_environment", Default::List(&[])),
    ("tool_java_language_version", Default::Str("")),
    ("tool_java_runtime_version", Default::Str("remotejdk_11")),
    ("tvos_cpus", Default::List(&[])),
    ("tvos_minimum_os", Default::None),
    ("tvos_sdk_version", Default::None),
    ("use_ijars", Default::Bool(true)),
    (
        "use_platforms_in_apple_crosstool_transition",
        Default::Bool(false),
    ),
    ("use_target_platform_for_tests", Default::Bool(false)),
    ("verbose_visibility_errors", Default::Bool(false)),
    ("visionos_cpus", Default::List(&[])),
    ("watchos_cpus", Default::List(&[])),
    ("watchos_minimum_os", Default::None),
    ("watchos_sdk_version", Default::None),
    ("xbinary_fdo", Default::None),
    ("xcode_version", Default::None),
    (
        "xcode_version_config",
        Default::Label("@@bazel_tools//tools/cpp:host_xcodes"),
    ),
];
