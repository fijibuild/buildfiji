//! Analysis against what `bazel aquery` showed for the same BUILD files
//! (Bazel 9.2.0, `bazel-out/k8-fastbuild`).

use crate::{ConfiguredTarget, ConfiguredTargetKey, Env, engine};
use fjfj_bzlmod::eval::{EvalOptions, eval_module_file};
use fjfj_graph::{ActionKind, Configuration, Label};
use fjfj_repo::{Options, Repos};
use std::collections::BTreeMap;
use std::sync::Arc;

const BIN: &str = "bazel-out/k8-fastbuild/bin";

fn workspace(files: &[(&str, &str)]) -> (tempfile::TempDir, Arc<Repos>) {
    let dir = tempfile::tempdir().unwrap();
    let ws = dir.path().join("ws");
    for (file, text) in files {
        let at = ws.join(file);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, text).unwrap();
    }
    let module = eval_module_file("MODULE.bazel", "module(name = 'm')\n", &EvalOptions::root())
        .unwrap()
        .module;
    let repos = Repos::new(
        Options {
            workspace_root: ws,
            output_base: dir.path().join("ob"),
            environ: BTreeMap::new(),
            downloader: None,
            repository_cache: None,
            distdirs: Vec::new(),
            registries: Vec::new(),
            facts: Vec::new(),
            repo_overrides: Vec::new(),
        },
        module,
    )
    .unwrap();
    (dir, Arc::new(repos))
}

fn config() -> Configuration {
    Configuration {
        cpu: "k8".into(),
        ..Configuration::default()
    }
}

async fn analyse(repos: &Arc<Repos>, label: &str) -> Result<Arc<ConfiguredTarget>, String> {
    analyse_in(repos, label, config()).await
}

async fn analyse_in(
    repos: &Arc<Repos>,
    label: &str,
    configuration: Configuration,
) -> Result<Arc<ConfiguredTarget>, String> {
    analyse_registering(repos, label, configuration, Vec::new()).await
}

async fn analyse_registering(
    repos: &Arc<Repos>,
    label: &str,
    configuration: Configuration,
    registered_toolchains: Vec<(String, String)>,
) -> Result<Arc<ConfiguredTarget>, String> {
    let engine = engine(Env {
        source: repos.clone(),
        rules: repos.clone(),
        main_repo_name: "_main".into(),
        registered_toolchains,
    });
    let (package, name) = label.trim_start_matches("//").split_once(':').unwrap();
    engine
        .get(ConfiguredTargetKey {
            label: Label {
                repo: String::new(),
                package: package.into(),
                name: name.into(),
            },
            configuration,
        })
        .await
        .map_err(|e| e.to_string())
}

fn paths(set: &fjfj_graph::NestedSet<fjfj_graph::Artifact>) -> Vec<String> {
    set.to_vec().iter().map(|a| a.exec_path()).collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_genrule_is_the_action_bazel_registers() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "BUILD.bazel",
            "genrule(\n    name = \"g\",\n    srcs = [\"in.txt\"],\n    outs = [\"out.txt\"],\n    cmd = \"cat $(SRCS) > $@ && echo $(location in.txt) >> $@\",\n)\nfilegroup(name = \"fg\", srcs = [\"in.txt\", \":g\"])\n",
        ),
        ("in.txt", "hello\n"),
    ]);
    let g = analyse(&repos, "//:g").await.unwrap();
    assert_eq!(paths(&g.files), [format!("{BIN}/out.txt")]);
    let [action] = &g.actions[..] else {
        panic!("{:?}", g.actions)
    };
    assert_eq!(action.mnemonic, "Genrule");
    assert_eq!(action.configuration, "k8-fastbuild");
    assert_eq!(
        action.progress_message.as_deref(),
        Some("Executing genrule //:g")
    );
    let ActionKind::Spawn { argv, env, .. } = &action.kind else {
        panic!("{:?}", action.kind)
    };
    // Verbatim from `bazel aquery`.
    assert_eq!(
        argv,
        &[
            "/bin/bash",
            "-c",
            "source external/bazel_tools/tools/genrule/genrule-setup.sh; cat in.txt > bazel-out/k8-fastbuild/bin/out.txt && echo ./in.txt >> bazel-out/k8-fastbuild/bin/out.txt"
        ]
    );
    assert_eq!(
        env.get("PATH").map(String::as_str),
        Some("/bin:/usr/bin:/usr/local/bin")
    );
    let inputs: Vec<String> = action.inputs.iter().map(|a| a.exec_path()).collect();
    assert_eq!(
        inputs,
        [
            "external/bazel_tools/tools/genrule/genrule-setup.sh",
            "in.txt"
        ]
    );
    let outputs: Vec<String> = action.outputs.iter().map(|a| a.exec_path()).collect();
    assert_eq!(outputs, [format!("{BIN}/out.txt")]);

    // A filegroup is its sources' files, and a generated file is the rule's.
    let fg = analyse(&repos, "//:fg").await.unwrap();
    assert_eq!(
        paths(&fg.files),
        ["in.txt".to_owned(), format!("{BIN}/out.txt")]
    );
    let file = analyse(&repos, "//:out.txt").await.unwrap();
    assert_eq!(paths(&file.files), [format!("{BIN}/out.txt")]);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_command_that_cannot_be_expanded_fails_analysis_in_bazels_words() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "BUILD.bazel",
            "genrule(name = \"t\", srcs = [\"in.txt\"], outs = [\"o\"], cmd = \"echo $(foo)\")\ngenrule(name = \"u\", outs = [\"o2\"])\n",
        ),
        ("in.txt", ""),
    ]);
    assert_eq!(
        analyse(&repos, "//:t").await.unwrap_err(),
        "in cmd attribute of genrule rule //:t: $(foo) not defined"
    );
    assert!(
        analyse(&repos, "//:u")
            .await
            .unwrap_err()
            .contains("missing value for `cmd` attribute")
    );
    // A rule written in Starlark is not analysed yet, and says so.
    assert!(
        analyse(&repos, "//:nope")
            .await
            .unwrap_err()
            .contains("no such target")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_rule_written_in_starlark_is_analysed_with_the_targets_it_names() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
Info = provider(fields = ["n"])

def _impl(ctx):
    out = ctx.actions.declare_file(ctx.label.name + ".txt")
    total = ctx.attr.k
    srcs = []
    for d in ctx.attr.deps:
        total += d[Info].n
        srcs += d[DefaultInfo].files.to_list()
    ctx.actions.run_shell(
        outputs = [out],
        inputs = srcs + ctx.files.data,
        command = "cat %s > %s; echo %d >> %s" % (" ".join([f.path for f in srcs + ctx.files.data]), out.path, total, out.path),
    )
    return [DefaultInfo(files = depset([out])), Info(n = total)]

r = rule(implementation = _impl, attrs = {"k": attr.int(default = 1), "deps": attr.label_list(), "data": attr.label_list(allow_files = True)})
"#,
        ),
        (
            "BUILD.bazel",
            "load(':defs.bzl', 'r')\nr(name = 'a', k = 2, data = ['in.txt'])\nr(name = 'b', deps = [':a'], k = 10)\n",
        ),
        ("in.txt", "x\n"),
    ]);
    let b = analyse(&repos, "//:b").await.unwrap();
    assert_eq!(b.rule_class.as_deref(), Some("r"));
    assert_eq!(paths(&b.files), [format!("{BIN}/b.txt")]);
    let [action] = &b.actions[..] else {
        panic!("{:?}", b.actions)
    };
    assert_eq!(action.progress_message.as_deref(), Some("Action b.txt"));
    let inputs: Vec<String> = action.inputs.iter().map(|a| a.exec_path()).collect();
    assert_eq!(inputs, [format!("{BIN}/a.txt")]);
    let ActionKind::Spawn { argv, .. } = &action.kind else {
        panic!()
    };
    // `a` gave `Info(n = 2)`, so `b` is 10 + 2.
    assert_eq!(
        argv[2],
        format!("cat {BIN}/a.txt > {BIN}/b.txt; echo 12 >> {BIN}/b.txt")
    );
    assert_eq!(b.deps.len(), 1);
}

/// Probed with `bazel cquery` on the same BUILD file.
#[tokio::test(flavor = "multi_thread")]
async fn select_chooses_the_branch_bazel_chooses() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "BUILD.bazel",
            r#"
config_setting(name = "opt", values = {"compilation_mode": "opt"})
config_setting(name = "fast", values = {"compilation_mode": "fastbuild"})
config_setting(name = "fast_def", values = {"compilation_mode": "fastbuild"}, define_values = {"k": "v"})
filegroup(name = "a", srcs = select({":opt": ["o.txt"], ":fast": ["f.txt"]}))
filegroup(name = "b", srcs = select({":opt": ["o.txt"]}))
filegroup(name = "c", srcs = select({":opt": ["o.txt"]}, no_match_error = "no luck"))
filegroup(name = "d", srcs = select({":fast": ["f.txt"], ":fast_def": ["fd.txt"], "//conditions:default": []}))
filegroup(name = "e", srcs = select({":fast": ["f.txt"], ":opt": ["o.txt"]}) + ["x.txt"])
"#,
        ),
        ("o.txt", ""),
        ("f.txt", ""),
        ("fd.txt", ""),
        ("x.txt", ""),
    ]);
    assert_eq!(
        paths(&analyse(&repos, "//:a").await.unwrap().files),
        ["f.txt"]
    );
    assert_eq!(
        paths(&analyse(&repos, "//:e").await.unwrap().files),
        ["f.txt", "x.txt"]
    );
    let error = analyse(&repos, "//:b").await.unwrap_err();
    assert!(
        error.contains("configurable attribute \"srcs\" in //:b doesn't match this configuration. Would a default condition help?"),
        "{error}"
    );
    let error = analyse(&repos, "//:c").await.unwrap_err();
    assert!(
        error.contains("doesn't match this configuration: no luck"),
        "{error}"
    );
    // A setting that says more wins over one it contains.
    assert_eq!(
        paths(&analyse(&repos, "//:d").await.unwrap().files),
        ["f.txt"]
    );
    let mut with_define = config();
    with_define.defines.insert("k".into(), "v".into());
    assert_eq!(
        paths(&analyse_in(&repos, "//:d", with_define).await.unwrap().files),
        ["fd.txt"]
    );
    let mut opt = config();
    opt.compilation_mode = fjfj_graph::CompilationMode::Opt;
    assert_eq!(
        paths(&analyse_in(&repos, "//:b", opt).await.unwrap().files),
        ["o.txt"]
    );
}

/// Probed with `bazel build` and `aquery` of the same rule.
#[tokio::test(flavor = "multi_thread")]
async fn an_executable_rule_gets_the_runfiles_tree_bazel_makes() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _impl(ctx):
    exe = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.write(exe, "exit 0\n", is_executable = True)
    return [DefaultInfo(executable = exe, runfiles = ctx.runfiles(files = ctx.files.data))]

my_bin = rule(implementation = _impl, executable = True, attrs = {"data": attr.label_list(allow_files = True)})
"#,
        ),
        (
            "pkg/BUILD.bazel",
            "load('//:defs.bzl', 'my_bin')\nexports_files(['data.txt'])\nmy_bin(name = 'bin', data = ['data.txt'])\n",
        ),
        ("BUILD.bazel", ""),
        ("pkg/data.txt", "d"),
    ]);
    let bin = analyse(&repos, "//pkg:bin").await.unwrap();
    // The executable is built with the target; the tree is built, not reported.
    assert_eq!(paths(&bin.files), [format!("{BIN}/pkg/bin")]);
    let extra: Vec<String> = bin.extra_outputs.iter().map(|a| a.exec_path()).collect();
    assert_eq!(
        extra,
        [
            format!("{BIN}/pkg/bin.runfiles"),
            format!("{BIN}/pkg/bin.runfiles_manifest"),
            format!("{BIN}/pkg/bin.repo_mapping"),
        ]
    );
    let tree = bin
        .actions
        .iter()
        .find(|a| a.mnemonic == "SymlinkTree")
        .unwrap();
    let ActionKind::RunfilesTree {
        entries,
        repo_mapping_contents,
        ..
    } = &tree.kind
    else {
        panic!()
    };
    let entries: Vec<(String, String)> = entries
        .iter()
        .map(|(p, a)| (p.clone(), a.exec_path()))
        .collect();
    assert_eq!(
        entries,
        [
            ("_main/pkg/bin".to_owned(), format!("{BIN}/pkg/bin")),
            ("_main/pkg/data.txt".to_owned(), "pkg/data.txt".to_owned()),
        ]
    );
    assert_eq!(repo_mapping_contents, ",m,_main\n");
}

/// Probed with `bazel build` of the same BUILD file and `register_toolchains`.
#[tokio::test(flavor = "multi_thread")]
async fn a_rule_gets_the_first_registered_toolchain_its_platform_fits() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _tc_impl(ctx):
    return [platform_common.ToolchainInfo(name = ctx.attr.n)]
my_toolchain = rule(implementation = _tc_impl, attrs = {"n": attr.string()})

def _impl(ctx):
    info = ctx.toolchains["//:tt"]
    print("toolchain:", info.name if info else None, ctx.toolchains["//:opt"])
    return []
r = rule(implementation = _impl, toolchains = ["//:tt", config_common.toolchain_type("//:opt", mandatory = False)])
"#,
        ),
        (
            "BUILD.bazel",
            r#"
load(":defs.bzl", "my_toolchain", "r")
toolchain_type(name = "tt")
toolchain_type(name = "opt")
constraint_setting(name = "os")
constraint_value(name = "linux", constraint_setting = ":os")
constraint_value(name = "windows", constraint_setting = ":os")
my_toolchain(name = "impl_linux", n = "linux")
my_toolchain(name = "impl_other", n = "other")
toolchain(name = "tc_other", toolchain_type = ":tt", toolchain = ":impl_other", target_compatible_with = [":windows"])
toolchain(name = "tc_linux", toolchain_type = ":tt", toolchain = ":impl_linux", target_compatible_with = [":linux"])
r(name = "t")
"#,
        ),
    ]);
    let mut on_linux = config();
    on_linux.constraints.insert(Label {
        repo: String::new(),
        package: String::new(),
        name: "linux".into(),
    });
    let registered = vec![
        (String::new(), "//:tc_other".to_owned()),
        (String::new(), "//:tc_linux".to_owned()),
    ];
    let t = analyse_registering(&repos, "//:t", on_linux.clone(), registered.clone())
        .await
        .unwrap();
    assert_eq!(t.printed, ["toolchain: linux None"]);
    // None registered, or none that fits: Bazel's words.
    let error = analyse_registering(&repos, "//:t", on_linux, Vec::new())
        .await
        .unwrap_err();
    assert!(
        error.contains("While resolving toolchains for target //:t")
            && error.contains("No matching toolchains found for types:\n  //:tt\nTo debug, rerun with --toolchain_resolution_debug='//:tt'\nFor more information on platforms or toolchains see https://bazel.build/concepts/platforms-intro."),
        "{error}"
    );
    let error = analyse_registering(&repos, "//:t", config(), registered)
        .await
        .unwrap_err();
    assert!(error.contains("No matching toolchains found"), "{error}");
}

/// Probed with `bazel build` on the same files: a tool is built in
/// `k8-opt-exec`, whatever `-c` says.
#[tokio::test(flavor = "multi_thread")]
async fn a_tool_attribute_is_built_in_the_exec_configuration() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _tool(ctx):
    out = ctx.actions.declare_file(ctx.label.name + ".sh")
    ctx.actions.write(out, "x", is_executable = True)
    return [DefaultInfo(executable = out)]
tool = rule(implementation = _tool, executable = True)

def _use(ctx):
    out = ctx.actions.declare_file(ctx.label.name + ".txt")
    ctx.actions.run_shell(outputs = [out], tools = [ctx.executable.t], command = ctx.executable.t.path + " > " + out.path)
    return [DefaultInfo(files = depset([out]))]
use = rule(implementation = _use, attrs = {"t": attr.label(cfg = "exec", executable = True, default = "//:tool")})
"#,
        ),
        (
            "BUILD.bazel",
            "load(':defs.bzl', 'tool', 'use')\ntool(name = 'tool')\nuse(name = 'u')\ngenrule(name = 'g', outs = ['g.txt'], tools = [':tool'], cmd = '$(location :tool) > $@')\n",
        ),
    ]);
    let u = analyse(&repos, "//:u").await.unwrap();
    let ActionKind::Spawn { argv, .. } = &u.actions[0].kind else {
        panic!()
    };
    assert!(
        argv[2].starts_with("bazel-out/k8-opt-exec/bin/tool.sh"),
        "{argv:?}"
    );
    let g = analyse(&repos, "//:g").await.unwrap();
    let ActionKind::Spawn { argv, .. } = &g.actions[0].kind else {
        panic!()
    };
    assert!(
        argv[2].contains("bazel-out/k8-opt-exec/bin/tool.sh"),
        "{argv:?}"
    );
}

/// Probed with `bazel build` on the same files: the output directories, and
/// that a transition setting a flag to its default changes nothing.
#[tokio::test(flavor = "multi_thread")]
async fn a_transition_changes_the_configuration_of_the_edge_it_is_on() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
Setting = provider(fields = ["value"])
def _flag(ctx):
    return [Setting(value = ctx.build_setting_value)]
flag = rule(implementation = _flag, build_setting = config.string(flag = True))

def _leaf(ctx):
    out = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.write(out, ctx.attr.flag[Setting].value)
    return [DefaultInfo(files = depset([out]))]
leaf = rule(implementation = _leaf, attrs = {"flag": attr.label(default = "//:flag")})

def _on(settings, attr):
    return {"//:flag": "on", "//command_line_option:compilation_mode": "opt"}
on = transition(implementation = _on, inputs = [], outputs = ["//:flag", "//command_line_option:compilation_mode"])
def _same(settings, attr):
    return {"//:flag": "off"}
same = transition(implementation = _same, inputs = ["//:flag"], outputs = ["//:flag"])

def _top(ctx):
    return [DefaultInfo(files = depset(transitive = [d[DefaultInfo].files for d in ctx.attr.deps]))]
top = rule(implementation = _top, attrs = {"deps": attr.label_list(cfg = on)})
keep = rule(implementation = _top, attrs = {"deps": attr.label_list(cfg = same)})
"#,
        ),
        (
            "BUILD.bazel",
            "load(':defs.bzl', 'flag', 'leaf', 'top', 'keep')\nflag(name = 'flag', build_setting_default = 'off')\nleaf(name = 'leaf')\ntop(name = 't', deps = [':leaf'])\nkeep(name = 'k', deps = [':leaf'])\n",
        ),
    ]);
    let t = analyse(&repos, "//:t").await.unwrap();
    assert_eq!(
        paths(&t.files),
        ["bazel-out/k8-opt-ST-c59cc04586de/bin/leaf"]
    );
    let k = analyse(&repos, "//:k").await.unwrap();
    assert_eq!(paths(&k.files), [format!("{BIN}/leaf")]);
}

/// Probed with `--platforms` on `bazel build`: a child's value of a setting
/// replaces its parent's.
#[tokio::test(flavor = "multi_thread")]
async fn a_platform_has_its_parents_constraints_overridden_by_its_own() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "BUILD.bazel",
            r#"
constraint_setting(name = "os")
constraint_value(name = "linux", constraint_setting = ":os")
constraint_value(name = "osx", constraint_setting = ":os")
constraint_setting(name = "cpu")
constraint_value(name = "arm", constraint_setting = ":cpu")
platform(name = "parent", constraint_values = [":linux", ":arm"])
platform(name = "child", parents = [":parent"], constraint_values = [":osx"])
"#,
        ),
    ]);
    let child = analyse(&repos, "//:child").await.unwrap();
    let have: Vec<String> = child
        .platform
        .as_ref()
        .unwrap()
        .values()
        .iter()
        .map(|l| l.name.clone())
        .collect();
    assert_eq!(have, ["arm", "osx"]);
}

/// A default that is a function of the other attributes (rules_cc's
/// `_def_parser`) is called with the ones its parameters name.
#[tokio::test(flavor = "multi_thread")]
async fn a_default_that_is_a_function_is_called_with_the_attributes_it_names() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _default(name, tags):
    if "skip" in tags:
        return None
    return Label("//:" + name + "_helper")

def _impl(ctx):
    out = ctx.actions.declare_file(ctx.label.name + ".txt")
    ctx.actions.write(out, str(ctx.attr._helper.label if ctx.attr._helper else None))
    return [DefaultInfo(files = depset([out]))]

r = rule(implementation = _impl, attrs = {"_helper": attr.label(default = _default)})
"#,
        ),
        (
            "BUILD.bazel",
            "load(':defs.bzl', 'r')\nfilegroup(name = 'a_helper')\nr(name = 'a')\nr(name = 'b', tags = ['skip'])\n",
        ),
    ]);
    let a = analyse(&repos, "//:a").await.unwrap();
    let ActionKind::WriteFile { contents, .. } = &a.actions[0].kind else {
        panic!("{:?}", a.actions)
    };
    assert_eq!(String::from_utf8_lossy(contents), "@@//:a_helper");
    assert!(a.deps.iter().any(|d| d.label.name == "a_helper"));
    let b = analyse(&repos, "//:b").await.unwrap();
    let ActionKind::WriteFile { contents, .. } = &b.actions[0].kind else {
        panic!("{:?}", b.actions)
    };
    assert_eq!(String::from_utf8_lossy(contents), "None");
}
