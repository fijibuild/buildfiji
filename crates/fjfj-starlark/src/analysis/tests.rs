//! Running rules against what `bazel aquery` and `bazel build` showed for the
//! same rules (Bazel 9.2.0).

use super::*;
use crate::test_support::module_in;
use fjfj_graph::rule::AttrValue;
use fjfj_graph::{ActionKind, Artifact, Configuration, Label};
use std::collections::BTreeMap;
use std::sync::Arc;

fn label(package: &str, name: &str) -> Label {
    Label {
        repo: String::new(),
        package: package.into(),
        name: name.into(),
    }
}

fn request(
    src: &str,
    rule: &str,
    attrs: Vec<(String, AttrValue)>,
    deps: Vec<DepInfo>,
) -> RuleRequest {
    request_in(module_in("", "", src).unwrap(), rule, attrs, deps)
}

fn request_in(
    module: starlark::environment::FrozenModule,
    rule: &str,
    attrs: Vec<(String, AttrValue)>,
    deps: Vec<DepInfo>,
) -> RuleRequest {
    RuleRequest {
        module,
        rule_name: rule.into(),
        label: label("", "t"),
        location: "BUILD.bazel:2:2".into(),
        build_file: "BUILD.bazel".into(),
        configuration: Configuration {
            cpu: "k8".into(),
            ..Configuration::default()
        },
        main_repo_name: "_main".into(),
        attrs,
        deps: deps.into_iter().map(|d| (d.label.clone(), d)).collect(),
        outputs: Vec::new(),
        mappings: Arc::new(crate::test_support::probe_mappings()),
        toolchains: Vec::new(),
        build_setting_value: None,
        native: None,
    }
}

fn paths(files: &[Artifact]) -> Vec<String> {
    files.iter().map(Artifact::exec_path).collect()
}

const BIN: &str = "bazel-out/k8-fastbuild/bin";

/// The rule `bazel aquery` showed the actions of.
const ACTIONS: &str = r##"
def _impl(ctx):
    w = ctx.actions.declare_file(ctx.label.name + ".w")
    ctx.actions.write(w, "hello")
    o2 = ctx.actions.declare_file("o2")
    ctx.actions.run_shell(outputs = [o2], inputs = [w], command = "cat %s > %s" % (w.path, o2.path))
    o3 = ctx.actions.declare_file("o3")
    ctx.actions.run_shell(outputs = [o3], inputs = [w], arguments = ["a", "b"], command = "echo $1 $2 > " + o3.path, mnemonic = "Three", progress_message = "making %s" % o3.short_path, env = {"K": "V"}, use_default_shell_env = True)
    o4 = ctx.actions.declare_file("o4")
    ctx.actions.run(outputs = [o4], inputs = [], executable = "/bin/cp", arguments = [w.path, o4.path])
    s = ctx.actions.declare_file("s")
    ctx.actions.symlink(output = s, target_file = w)
    e = ctx.actions.declare_file("e.sh")
    ctx.actions.write(e, "#!/bin/sh\n", is_executable = True)
    return [DefaultInfo(files = depset([o2, o3, o4, s, e]))]

r = rule(implementation = _impl)
"##;

#[test]
fn actions_are_the_ones_bazel_registers() {
    let out = run_rule(&request(ACTIONS, "r", Vec::new(), Vec::new())).unwrap();
    assert_eq!(
        paths(&out.files),
        [
            format!("{BIN}/o2"),
            format!("{BIN}/o3"),
            format!("{BIN}/o4"),
            format!("{BIN}/s"),
            format!("{BIN}/e.sh")
        ]
    );
    let described: Vec<(String, String, Vec<String>, Vec<String>)> = out
        .actions
        .iter()
        .map(|a| {
            (
                a.mnemonic.clone(),
                a.progress_message.clone().unwrap(),
                paths(&a.inputs),
                paths(&a.outputs),
            )
        })
        .collect();
    let w = format!("{BIN}/t.w");
    assert_eq!(
        described,
        [
            (
                "FileWrite".into(),
                "Writing file t.w".into(),
                vec![],
                vec![w.clone()]
            ),
            (
                "Action".into(),
                "Action o2".into(),
                vec![w.clone()],
                vec![format!("{BIN}/o2")]
            ),
            (
                "Three".into(),
                "making o3".into(),
                vec![w.clone()],
                vec![format!("{BIN}/o3")]
            ),
            (
                "Action".into(),
                "Action o4".into(),
                vec![],
                vec![format!("{BIN}/o4")]
            ),
            (
                "Symlink".into(),
                "Creating symlink s".into(),
                vec![w.clone()],
                vec![format!("{BIN}/s")]
            ),
            (
                "FileWrite".into(),
                "Writing script e.sh".into(),
                vec![],
                vec![format!("{BIN}/e.sh")]
            ),
        ]
    );
    let ActionKind::Spawn { argv, env, .. } = &out.actions[1].kind else {
        panic!()
    };
    assert_eq!(
        argv,
        &["/bin/bash", "-c", &format!("cat {BIN}/t.w > {BIN}/o2")]
    );
    assert!(env.is_empty());
    let ActionKind::Spawn { argv, env, .. } = &out.actions[2].kind else {
        panic!()
    };
    assert_eq!(
        argv,
        &[
            "/bin/bash",
            "-c",
            &format!("echo $1 $2 > {BIN}/o3"),
            "",
            "a",
            "b"
        ]
    );
    assert_eq!(
        env,
        &BTreeMap::from([
            ("K".to_owned(), "V".to_owned()),
            ("PATH".to_owned(), "/bin:/usr/bin:/usr/local/bin".to_owned())
        ])
    );
    let ActionKind::Spawn { argv, .. } = &out.actions[3].kind else {
        panic!()
    };
    assert_eq!(
        argv,
        &["/bin/cp", &format!("{BIN}/t.w"), &format!("{BIN}/o4")]
    );
    assert_eq!(out.actions[0].owner_kind, "r");
}

#[test]
fn a_file_has_the_members_bazel_gives_it() {
    let src = r#"
def _show(f):
    return "path=%s short=%s base=%s dir=%s ext=%s src=%s dirp=%s root=%s repr=%s type=%s" % (f.path, f.short_path, f.basename, f.dirname, f.extension, f.is_source, f.is_directory, f.root.path, repr(f), type(f))

def _impl(ctx):
    o = ctx.actions.declare_file("sub/x.tar.gz")
    d = ctx.actions.declare_file("noext")
    ctx.actions.write(o, "a")
    ctx.actions.write(d, "b")
    print(_show(o))
    print(_show(d))
    for f in ctx.files.srcs:
        print(_show(f))
    print("label=%s name=%s pkg=%s ws=%s bin=%s gen=%s" % (ctx.label, ctx.label.name, ctx.label.package, ctx.workspace_name, ctx.bin_dir.path, ctx.genfiles_dir.path))
    print("attrs", ctx.attr.n, ctx.attr.s, ctx.attr.l, ctx.attr.flag, type(ctx.attr.srcs), ctx.attr.srcs)
    print("var", ctx.var)
    print(ctx.build_file_path)
    return [DefaultInfo(files = depset([o, d]))]

r = rule(implementation = _impl, attrs = {"srcs": attr.label_list(allow_files = True), "n": attr.int(default = 3), "s": attr.string(default = "q"), "l": attr.string_list(default = ["a"]), "flag": attr.bool()})
"#;
    let dep = |package: &str| DepInfo {
        label: label(package, "s.txt"),
        rule_class: None,
        generated: false,
        files: vec![Artifact::source("", package, "s.txt")],
        executable: None,
        runfiles: fjfj_graph::Runfiles::default(),
        providers: Vec::new(),
    };
    let req = request(
        src,
        "r",
        vec![(
            "srcs".into(),
            AttrValue::LabelList(vec![label("", "s.txt"), label("a", "s.txt")]),
        )],
        vec![dep(""), dep("a")],
    );
    let out = run_rule(&req).unwrap();
    // Verbatim from `bazel build` of the same rule (the `owner` and `root`
    // object reprs aside).
    assert_eq!(
        out.printed,
        [
            "path=bazel-out/k8-fastbuild/bin/sub/x.tar.gz short=sub/x.tar.gz base=x.tar.gz dir=bazel-out/k8-fastbuild/bin/sub ext=gz src=False dirp=False root=bazel-out/k8-fastbuild/bin repr=<generated file sub/x.tar.gz> type=File",
            "path=bazel-out/k8-fastbuild/bin/noext short=noext base=noext dir=bazel-out/k8-fastbuild/bin ext= src=False dirp=False root=bazel-out/k8-fastbuild/bin repr=<generated file noext> type=File",
            "path=s.txt short=s.txt base=s.txt dir=. ext=txt src=True dirp=False root= repr=<source file s.txt> type=File",
            "path=a/s.txt short=a/s.txt base=s.txt dir=a ext=txt src=True dirp=False root= repr=<source file a/s.txt> type=File",
            "label=@@//:t name=t pkg= ws=_main bin=bazel-out/k8-fastbuild/bin gen=bazel-out/k8-fastbuild/bin",
            "attrs 3 q [\"a\"] False list [<input file target //:s.txt>, <input file target //a:s.txt>]",
            r#"var {"TARGET_CPU": "x86_64", "COMPILATION_MODE": "fastbuild", "BINDIR": "bazel-out/k8-fastbuild/bin", "GENDIR": "bazel-out/k8-fastbuild/bin"}"#,
            "BUILD.bazel",
        ]
    );
}

#[test]
fn providers_are_kept_and_a_dependent_reads_them() {
    let src = r#"
MyInfo = provider(fields = ["x"])

def _impl(ctx):
    seen = []
    for d in ctx.attr.deps:
        seen.append(d[MyInfo].x if MyInfo in d else "none")
        seen.append(len(d[DefaultInfo].files.to_list()))
    print(seen)
    return [MyInfo(x = 42 + len(seen)), DefaultInfo(files = depset([]))]

r = rule(implementation = _impl, attrs = {"deps": attr.label_list()})
"#;
    // First a target with no deps gives `MyInfo`.
    // Both targets run the one rule, so they share the module.
    let module = module_in("", "", src).unwrap();
    let first = run_rule(&request_in(module.clone(), "r", Vec::new(), Vec::new())).unwrap();
    assert_eq!(first.providers.len(), 1);
    // A dependent sees it through `Target`.
    let dep = DepInfo {
        label: label("", "first"),
        rule_class: Some("r".into()),
        generated: false,
        files: vec![Artifact::source("", "", "f")],
        executable: None,
        runfiles: fjfj_graph::Runfiles::default(),
        providers: first.providers,
    };
    let second = run_rule(&request_in(
        module,
        "r",
        vec![(
            "deps".into(),
            AttrValue::LabelList(vec![label("", "first")]),
        )],
        vec![dep],
    ))
    .unwrap();
    assert_eq!(second.printed, ["[42, 1]"]);
}

#[test]
fn errors_in_the_implementation_come_back_as_text() {
    let src = r#"
def _impl(ctx):
    ctx.actions.run_shell(outputs = [], command = "true")
r = rule(implementation = _impl)
def _impl2(ctx):
    fail("nope")
r2 = rule(implementation = _impl2)
"#;
    let e = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap_err();
    assert!(e.contains("requires at least one output"), "{e}");
    let e = run_rule(&request(src, "r2", Vec::new(), Vec::new())).unwrap_err();
    assert!(e.contains("nope"), "{e}");
}

/// The rule `bazel aquery` showed the command line of; the parameter file is
/// what `bazel build` left in `bazel-bin`.
const ARGS: &str = r#"
def _dbl(s):
    return s + s

def _impl(ctx):
    o = ctx.actions.declare_file("o")
    a = ctx.actions.args()
    a.add("-x")
    a.add("--name", "v")
    a.add("--fmt", "v", format = "<%s>")
    a.add_all("--many", ["a", "b"], before_each = "-e", format_each = "[%s]", terminate_with = "END")
    a.add_all(["c", "d"], map_each = _dbl)
    a.add_joined("--j", ["p", "q"], join_with = ",", format_joined = "{%s}")
    a.add_all("--empty", [], omit_if_empty = True)
    a.add_all(depset(["z", "z"]), uniquify = True)
    a.add(3)
    a.add(ctx.file.src)
    inline = ctx.actions.args()
    inline.add_all("--inline", ["x", "y"])
    if ctx.attr.params:
        a.use_param_file("@%s", use_always = True)
    ctx.actions.run_shell(outputs = [o], inputs = [ctx.file.src], command = "echo $@ > " + o.path, arguments = [a, inline], mnemonic = "Args")
    return [DefaultInfo(files = depset([o]))]

r = rule(implementation = _impl, attrs = {"src": attr.label(allow_single_file = True), "params": attr.bool()})
"#;

#[test]
fn args_are_expanded_as_bazel_expands_them() {
    let src_file = DepInfo {
        label: label("", "s.txt"),
        rule_class: None,
        generated: false,
        files: vec![Artifact::source("", "", "s.txt")],
        executable: None,
        runfiles: fjfj_graph::Runfiles::default(),
        providers: Vec::new(),
    };
    let module = module_in("", "", ARGS).unwrap();
    let attrs = |params: bool| {
        vec![
            ("src".to_owned(), AttrValue::Label(label("", "s.txt"))),
            ("params".to_owned(), AttrValue::Bool(params)),
        ]
    };
    let expanded = [
        "-x", "--name", "v", "--fmt", "<v>", "--many", "-e", "[a]", "-e", "[b]", "END", "cc", "dd",
        "--j", "{p,q}", "z", "3", "s.txt",
    ];
    let out = run_rule(&request_in(
        module.clone(),
        "r",
        attrs(false),
        vec![src_file.clone()],
    ))
    .unwrap();
    let ActionKind::Spawn { argv, .. } = &out.actions[0].kind else {
        panic!()
    };
    let mut want: Vec<String> = ["/bin/bash", "-c", &format!("echo $@ > {BIN}/o"), ""]
        .iter()
        .map(|s| s.to_string())
        .collect();
    want.extend(expanded.iter().map(|s| s.to_string()));
    want.extend(["--inline", "x", "y"].iter().map(|s| s.to_string()));
    assert_eq!(argv, &want);

    // With a parameter file, the arguments go to `o-0.params`, quoted for a shell.
    let out = run_rule(&request_in(module, "r", attrs(true), vec![src_file])).unwrap();
    let [params, spawn] = &out.actions[..] else {
        panic!("{:?}", out.actions)
    };
    assert_eq!(paths(&params.outputs), [format!("{BIN}/o-0.params")]);
    let ActionKind::WriteFile { contents, .. } = &params.kind else {
        panic!()
    };
    let mut quoted = String::new();
    for arg in expanded {
        quoted.push_str(&match arg {
            "<v>" | "[a]" | "[b]" | "{p,q}" => format!("'{arg}'\n"),
            plain => format!("{plain}\n"),
        });
    }
    assert_eq!(String::from_utf8(contents.clone()).unwrap(), quoted);
    let ActionKind::Spawn { argv, .. } = &spawn.kind else {
        panic!()
    };
    assert_eq!(
        &argv[3..],
        &["", &format!("@{BIN}/o-0.params"), "--inline", "x", "y"]
    );
    assert!(paths(&spawn.inputs).contains(&format!("{BIN}/o-0.params")));
}

/// `ctx.fragments` are the structs of the builtins, made from the
/// configuration (buildfiji-136.18); a late-bound default is the label of its
/// option's default (buildfiji-bo8).
#[test]
fn fragments_and_late_bound_defaults() {
    let src = r#"
def _impl(ctx):
    cpp = ctx.fragments.cpp
    apple = ctx.fragments.apple
    config = apple_common.XcodeVersionConfig(
        ios_sdk_version = "1", ios_minimum_os_version = "2",
        visionos_sdk_version = "3", visionos_minimum_os_version = "4",
        watchos_sdk_version = "5", watchos_minimum_os_version = "6",
        tvos_sdk_version = "7", tvos_minimum_os_version = "8",
        macos_sdk_version = "10.11", macos_minimum_os_version = "10.12",
    )
    print(cpp.compilation_mode(), cpp.minimum_os_version(), apple.single_arch_platform.platform_type,
          config.minimum_os_for_platform_type(apple.single_arch_platform.platform_type))
    return [DefaultInfo()]
r = rule(
    implementation = _impl,
    fragments = ["apple", "cpp"],
    attrs = {"_xcode": attr.label(default = configuration_field("apple", "xcode_config_label"))},
)
"#;
    let req = request(src, "r", Vec::new(), Vec::new());
    assert_eq!(
        run_rule(&req).unwrap().printed,
        ["fastbuild None macos 10.12"]
    );
    let defaults = computed_defaults(
        &req.module,
        "r",
        &[],
        &crate::test_support::probe_mappings(),
        "",
    )
    .unwrap();
    assert_eq!(
        defaults,
        [(
            "_xcode".to_owned(),
            AttrValue::Label(Label {
                repo: "bazel_tools".into(),
                package: "tools/objc".into(),
                name: "host_xcodes".into()
            })
        )]
    );
}

/// `ctx.configuration`, an unset file attribute as `None` in `ctx.file`, and
/// the stamping templates, as rules_cc and bazel_tools read them.
#[test]
fn configuration_unset_files_and_status_templates() {
    let src = r#"
def _impl(ctx):
    c = ctx.configuration
    print(c.short_id, c.is_tool_configuration(), c.host_path_separator, c.coverage_enabled)
    print(ctx.file._zipper, ctx.files._zipper, ctx.executable._tool)
    template = ctx.actions.declare_file("t.template")
    ctx.actions.write(template, "{A}")
    out = ctx.actions.transform_info_file(
        transform_func = lambda status: {"{A}": status["BUILD_USER"]},
        template = template,
        output_file_name = "info.h",
    )
    return [DefaultInfo(files = depset([out]))]
r = rule(
    implementation = _impl,
    attrs = {
        "_zipper": attr.label(allow_single_file = True),
        "_tool": attr.label(executable = True, cfg = "exec"),
    },
)
"#;
    let out = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap();
    assert_eq!(out.printed, ["k8-fastbuild False : False", "None [] None"]);
    let [_, expand] = &out.actions[..] else {
        panic!("{:?}", out.actions)
    };
    let ActionKind::Template { substitutions, .. } = &expand.kind else {
        panic!()
    };
    assert_eq!(substitutions, &[("{A}".to_owned(), "username".to_owned())]);
}

/// The members of the cpp fragment that rules_cc reads, with the values Bazel
/// 9.2.0 gives them (the ones it does not hide, probed; the rest are their
/// flags' defaults) and the options of the configuration.
#[test]
fn the_cpp_fragment_has_the_members_rules_read() {
    let src = r#"
def _impl(ctx):
    cpp = ctx.fragments.cpp
    print(cpp.copts, cpp.cxxopts, cpp.conlyopts, cpp.linkopts, cpp.dynamic_mode(), cpp.apple_generate_dsym)
    print(cpp.should_strip_binaries(), cpp.start_end_lib(), cpp.force_pic(), cpp.compilation_mode(), cpp.use_llvm_coverage_map_format())
    return []
r = rule(implementation = _impl, fragments = ["cpp"])
"#;
    let mut req = request(src, "r", Vec::new(), Vec::new());
    req.configuration
        .options
        .insert("copt".into(), "-O2 -g".into());
    assert_eq!(
        run_rule(&req).unwrap().printed,
        [
            "[\"-O2\", \"-g\"] [] [] [] DEFAULT False",
            "True True False fastbuild False"
        ]
    );
}

/// The C++ feature engine: which features a request enables (requires, implies,
/// provides), and the flags, environment and tool they give an action.
#[test]
fn the_cc_feature_engine_selects_features_and_expands_flags() {
    let src = r#"
def _flag_group(flags = [], flag_groups = [], iterate_over = None, expand_if_available = None, expand_if_not_available = None, expand_if_true = None, expand_if_false = None, expand_if_equal = None):
    return struct(flags = flags, flag_groups = flag_groups, iterate_over = iterate_over, expand_if_available = expand_if_available, expand_if_not_available = expand_if_not_available, expand_if_true = expand_if_true, expand_if_false = expand_if_false, expand_if_equal = expand_if_equal)

def _flag_set(actions, flag_groups, with_features = []):
    return struct(actions = actions, flag_groups = flag_groups, with_features = with_features)

def _feature(name, enabled = False, flag_sets = [], env_sets = [], requires = [], implies = [], provides = []):
    return struct(name = name, enabled = enabled, flag_sets = flag_sets, env_sets = env_sets, requires = requires, implies = implies, provides = provides)

def _impl(ctx):
    internals = cc_common.internal_DO_NOT_USE()
    compile = _flag_set(["c++-compile"], [
        _flag_group(flags = ["-c", "%{source_file}"]),
        _flag_group(iterate_over = "includes", flags = ["-I", "%{includes}"]),
        _flag_group(flag_groups = [_flag_group(flags = ["-O%{opt_level}"])], expand_if_available = "opt_level"),
        _flag_group(flags = ["-DPIC"], expand_if_true = "pic"),
        _flag_group(flags = ["-fno-pic"], expand_if_false = "pic"),
        _flag_group(iterate_over = "libs", flag_groups = [_flag_group(flags = ["-l%{libs.name}", "100%%"])]),
    ])
    features = [
        _feature("opt", flag_sets = [_flag_set(["c++-compile"], [_flag_group(flags = ["-O2"])])]),
        _feature("base", enabled = True, flag_sets = [compile], implies = ["dep"]),
        _feature("dep", flag_sets = [_flag_set(["c++-compile"], [_flag_group(flags = ["-dep"])], with_features = [struct(features = [], not_features = ["opt"])])]),
        _feature("needs_opt", requires = [struct(features = ["opt"])], flag_sets = [_flag_set(["c++-compile"], [_flag_group(flags = ["-needs-opt"])])]),
        _feature("env", env_sets = [struct(actions = ["c++-compile"], with_features = [], env_entries = [struct(key = "K", value = "v-%{source_file}", expand_if_available = None)])]),
    ]
    gcc = struct(path = "bin/gcc", tool = None, with_features = [], execution_requirements = ["requires-x"])
    config = struct(
        _features_DO_NOT_USE = features,
        _action_configs_DO_NOT_USE = [struct(action_name = "c++-compile", config_name = "c++-compile", enabled = True, tools = [gcc], flag_sets = [], implies = ["env"])],
        _artifact_name_patterns_DO_NOT_USE = [],
    )
    engine = internals.cc_toolchain_features(toolchain_config_info = config, tools_directory = "external/tc")
    print(engine.default_features_and_action_configs())
    variables = internals.cc_toolchain_variables(vars = {
        "source_file": "a.cc",
        "includes": ["x", "y"],
        "pic": "",
        "libs": [struct(name = "m"), struct(name = "z")],
    })
    for requested in [["c++-compile"], ["c++-compile", "opt", "needs_opt"], ["needs_opt"]]:
        fc = engine.configure_features(requested_features = requested + engine.default_features_and_action_configs())
        print(requested, [n for n in ["opt", "base", "dep", "needs_opt", "env"] if fc.is_enabled(n)])
        print(cc_common.get_memory_inefficient_command_line(feature_configuration = fc, action_name = "c++-compile", variables = variables))
    fc = engine.configure_features(requested_features = ["c++-compile", "base"])
    print(cc_common.get_environment_variables(feature_configuration = fc, action_name = "c++-compile", variables = variables))
    print(cc_common.get_tool_for_action(feature_configuration = fc, action_name = "c++-compile"))
    print(cc_common.get_execution_requirements(feature_configuration = fc, action_name = "c++-compile"))
    print(cc_common.action_is_enabled(feature_configuration = fc, action_name = "c++-compile"), cc_common.action_is_enabled(feature_configuration = fc, action_name = "link"))
    return []
r = rule(implementation = _impl)
"#;
    let out = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap();
    assert_eq!(
        out.printed,
        [
            r#"["base", "c++-compile"]"#,
            // base implies dep, and the c++-compile action config implies env.
            r#"["c++-compile"] ["base", "dep", "env"]"#,
            r#"["-c", "a.cc", "-I", "x", "-I", "y", "-fno-pic", "-lm", "100%", "-lz", "100%", "-dep"]"#,
            // opt turns dep's flag off (not_features) and allows needs_opt.
            r#"["c++-compile", "opt", "needs_opt"] ["opt", "base", "dep", "needs_opt", "env"]"#,
            r#"["-O2", "-c", "a.cc", "-I", "x", "-I", "y", "-fno-pic", "-lm", "100%", "-lz", "100%", "-needs-opt"]"#,
            // needs_opt without opt is not satisfied and goes.
            r#"["needs_opt"] ["base", "dep", "env"]"#,
            r#"["-c", "a.cc", "-I", "x", "-I", "y", "-fno-pic", "-lm", "100%", "-lz", "100%", "-dep"]"#,
            r#"{"K": "v-a.cc"}"#,
            "external/tc/bin/gcc",
            r#"["requires-x"]"#,
            "True False",
        ]
    );
}

/// A subrule runs in the rule that calls it, with the rule's `ctx` and the
/// values of the attributes it declares; `freeze` makes a list hashable.
#[test]
fn a_subrule_runs_in_its_callers_ctx_and_freeze_makes_lists_hashable() {
    let src = r#"
def _sub_impl(ctx, x, *, _helper, _absent):
    return "%s %s %s %s" % (ctx.label.name, x, _helper.label.name if _helper else None, _absent)
sub = subrule(implementation = _sub_impl, attrs = {
    "_helper": attr.label(default = "//:h"),
    "_absent": attr.label(default = configuration_field("cpp", "zipper")),
})
def _impl(ctx):
    print(sub(7))
    frozen = cc_common.internal_DO_NOT_USE().freeze([1, 2])
    print({struct(a = frozen): "ok"}[struct(a = [1, 2])] if False else {struct(a = frozen): "ok"}[struct(a = frozen)])
    return []
r = rule(implementation = _impl, subrules = [sub])
"#;
    let helper = DepInfo {
        label: label("", "h"),
        rule_class: None,
        generated: false,
        files: Vec::new(),
        executable: None,
        runfiles: Default::default(),
        providers: Vec::new(),
    };
    let out = run_rule(&request(src, "r", Vec::new(), vec![helper])).unwrap();
    assert_eq!(out.printed, ["t 7 h None", "ok"]);
}

/// Names and actions of the C++ internals: categories name their files by the
/// toolchain's patterns or the defaults, a compile action runs the tool with
/// the expanded flags, and the link arguments are an `Args`.
#[test]
fn the_cc_internals_name_files_and_make_compile_actions() {
    let src = r#"
def _flag_group(flags):
    return struct(flags = flags, flag_groups = [], iterate_over = None, expand_if_available = None, expand_if_not_available = None, expand_if_true = None, expand_if_false = None, expand_if_equal = None)

def _impl(ctx):
    internals = cc_common.internal_DO_NOT_USE()
    gcc = struct(path = "/usr/bin/gcc", tool = None, with_features = [], execution_requirements = [])
    features = [struct(name = "f", enabled = True, requires = [], implies = [], provides = [], env_sets = [], flag_sets = [
        struct(actions = ["c++-compile"], with_features = [], flag_groups = [_flag_group(["-c", "%{source_file}", "-o", "%{output_file}"])]),
    ])]
    config = struct(
        _features_DO_NOT_USE = features,
        _action_configs_DO_NOT_USE = [struct(action_name = "c++-compile", config_name = "c++-compile", enabled = True, tools = [gcc], flag_sets = [], implies = [])],
        _artifact_name_patterns_DO_NOT_USE = [struct(category_name = "static_library", prefix = "", extension = ".lib")],
    )
    engine = internals.cc_toolchain_features(toolchain_config_info = config, tools_directory = "tc")
    toolchain = struct(_toolchain_features = engine, _compiler_files = depset([ctx.file.src]))
    print(internals.get_artifact_name_for_category(cc_toolchain = toolchain, category = "OBJECT_FILE", output_name = "d/foo.pic"))
    print(internals.get_artifact_name_for_category(cc_toolchain = toolchain, category = "STATIC_LIBRARY", output_name = "x"))
    print(internals.get_artifact_name_for_category(cc_toolchain = toolchain, category = "DYNAMIC_LIBRARY", output_name = "x"))
    print(internals.get_artifact_name_extension_for_category(toolchain, "PIC_OBJECT_FILE"))
    fc = engine.configure_features(requested_features = engine.default_features_and_action_configs())
    obj = internals.declare_compile_output_file(ctx = ctx, label = ctx.label, output_name = "foo.o", configuration = ctx.configuration)
    print(obj.short_path, internals.actions2ctx_cheat(ctx.actions).label.name, internals.rule_class(ctx))
    internals.create_cc_compile_action(
        action_construction_context = ctx,
        cc_compilation_context = struct(headers = depset()),
        cc_toolchain = toolchain,
        feature_configuration = fc,
        source = ctx.file.src,
        output_file = obj,
        compile_build_variables = internals.cc_toolchain_variables(vars = {"source_file": ctx.file.src.path, "output_file": obj.path}),
    )
    args = internals.get_link_args(feature_configuration = fc, action_name = "c++-compile", build_variables = internals.cc_toolchain_variables(vars = {"source_file": "s", "output_file": "o"}), parameter_file_type = None)
    print(type(args))
    return []
r = rule(implementation = _impl, attrs = {"src": attr.label(allow_single_file = True)}, fragments = ["cpp"])
"#;
    let src_file = DepInfo {
        label: label("", "a.cc"),
        rule_class: None,
        generated: false,
        files: vec![Artifact::source("", "", "a.cc")],
        executable: None,
        runfiles: Default::default(),
        providers: Vec::new(),
    };
    let attrs = vec![("src".to_owned(), AttrValue::Label(label("", "a.cc")))];
    let out = run_rule(&request(src, "r", attrs, vec![src_file])).unwrap();
    assert_eq!(
        out.printed,
        [
            "d/foo.pic.o",
            "x.lib",
            "libx.so",
            ".pic.o",
            "_objs/t/foo.o t r",
            "Args"
        ]
    );
    let [compile] = &out.actions[..] else {
        panic!("{:?}", out.actions)
    };
    assert_eq!(compile.mnemonic, "CppCompile");
    assert_eq!(compile.progress_message.as_deref(), Some("Compiling a.cc"));
    let ActionKind::Spawn { argv, .. } = &compile.kind else {
        panic!()
    };
    assert_eq!(
        argv,
        &[
            "/usr/bin/gcc",
            "-c",
            "a.cc",
            "-o",
            &format!("{BIN}/_objs/t/foo.o")
        ]
    );
}

/// A tree artifact is a directory (`declare_directory`), and an executable's
/// `files_to_run` brings its runfiles tree to an action that uses it.
#[test]
fn directories_are_declared_and_files_to_run_is_a_tool() {
    let src = r#"
def _impl(ctx):
    out = ctx.actions.declare_directory("out")
    plain = ctx.actions.declare_file("plain")
    print(out.is_directory, plain.is_directory, out.path)
    tool = ctx.attr.tool[DefaultInfo].files_to_run
    print(tool.executable.basename, tool.runfiles_manifest.basename)
    ctx.actions.run(executable = tool, outputs = [out], tools = [depset([plain])], arguments = ["x"])
    return []
r = rule(implementation = _impl, attrs = {"tool": attr.label(executable = True, cfg = "exec")})
"#;
    let exe = Artifact::derived("bazel-out/k8-opt-exec/bin", "", "", "tool");
    let tool = DepInfo {
        label: label("", "tool"),
        rule_class: Some("sh_binary".to_owned()),
        generated: false,
        files: vec![exe.clone()],
        executable: Some(exe.clone()),
        runfiles: Default::default(),
        providers: Vec::new(),
    };
    let attrs = vec![("tool".to_owned(), AttrValue::Label(label("", "tool")))];
    let out = run_rule(&request(src, "r", attrs, vec![tool])).unwrap();
    assert_eq!(
        out.printed,
        [
            format!("True False {BIN}/out"),
            "tool tool.runfiles_manifest".to_owned()
        ]
    );
    let [run] = &out.actions[..] else {
        panic!("{:?}", out.actions)
    };
    assert!(run.outputs[0].tree);
    let mut inputs: Vec<String> = run.inputs.iter().map(|i| i.exec_path()).collect();
    inputs.sort();
    assert_eq!(
        inputs,
        [
            format!("{BIN}/plain"),
            "bazel-out/k8-opt-exec/bin/tool".to_owned(),
            "bazel-out/k8-opt-exec/bin/tool.repo_mapping".to_owned(),
            "bazel-out/k8-opt-exec/bin/tool.runfiles".to_owned(),
            "bazel-out/k8-opt-exec/bin/tool.runfiles_manifest".to_owned(),
        ]
    );
}
