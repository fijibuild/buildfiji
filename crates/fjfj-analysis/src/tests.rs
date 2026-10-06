//! Analysis against what `bazel aquery` showed for the same BUILD files
//! (Bazel 9.2.0, `bazel-out/k8-fastbuild`).

use crate::{ConfiguredTarget, ConfiguredTargetKey, Env, engine, expand_label_text};
use fjfj_bzlmod::eval::{EvalOptions, eval_module_file};
use fjfj_graph::{ActionKind, Configuration, Label};
use fjfj_repo::{Options, Repos};
use fjfj_starlark::WithoutSites;
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
    analyse_on(
        repos,
        label,
        configuration,
        registered_toolchains,
        Vec::new(),
    )
    .await
}

async fn analyse_on(
    repos: &Arc<Repos>,
    label: &str,
    configuration: Configuration,
    registered_toolchains: Vec<(String, String)>,
    registered_execution_platforms: Vec<(String, String)>,
) -> Result<Arc<ConfiguredTarget>, String> {
    analyse_traced(
        repos,
        label,
        configuration,
        registered_toolchains,
        registered_execution_platforms,
        None,
    )
    .await
}

async fn analyse_traced(
    repos: &Arc<Repos>,
    label: &str,
    configuration: Configuration,
    registered_toolchains: Vec<(String, String)>,
    registered_execution_platforms: Vec<(String, String)>,
    toolchain_resolution_debug: Option<crate::ResolutionDebug>,
) -> Result<Arc<ConfiguredTarget>, String> {
    let engine = engine(Env {
        source: repos.clone(),
        rules: repos.clone(),
        main_repo_name: "_main".into(),
        registered_toolchains,
        extra_toolchains: Vec::new(),
        registered_execution_platforms,
        extra_execution_platforms: Vec::new(),
        host_constraints: None,
        record_execution_platforms: false,
        toolchain_resolution_debug,
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
    let ActionKind::SymlinkTree { entries, .. } = &tree.kind else {
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
    let mapping = bin
        .actions
        .iter()
        .find(|a| a.mnemonic == "RepoMappingManifest")
        .unwrap();
    let ActionKind::WriteFile { contents, .. } = &mapping.kind else {
        panic!()
    };
    assert_eq!(contents, b",m,_main\n");
    // Bazel's four: the mapping and the manifest it writes, the tree it links
    // from the manifest and the directory that stands for the tree.
    let mnemonics: Vec<&str> = bin
        .actions
        .iter()
        .filter(|a| a.mnemonic != "FileWrite")
        .map(|a| a.mnemonic.as_str())
        .collect();
    assert_eq!(
        mnemonics,
        [
            "RepoMappingManifest",
            "SourceSymlinkManifest",
            "SymlinkTree",
            "RunfilesTree"
        ]
    );
}

/// Python's `legacy_create_init`, probed with `bazel build` of a py_binary: an
/// empty `__init__.py` in the runfiles root and above each Python file, not in
/// the main repository's own directory or above files that are not Python.
#[tokio::test(flavor = "multi_thread")]
async fn python_inits_go_above_python_files_in_the_runfiles_tree() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _impl(ctx):
    exe = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.write(exe, "exit 0\n", is_executable = True)
    return [DefaultInfo(executable = exe, runfiles = ctx.runfiles(files = ctx.files.data, _python_inits = True))]

my_bin = rule(implementation = _impl, executable = True, attrs = {"data": attr.label_list(allow_files = True)})
"#,
        ),
        (
            "pkg/BUILD.bazel",
            "load('//:defs.bzl', 'my_bin')\nexports_files(['a/b/m.py', 'c/d.txt', 'e/__init__.py', 'e/f.py'])\nmy_bin(name = 'bin', data = ['a/b/m.py', 'c/d.txt', 'e/__init__.py', 'e/f.py'])\n",
        ),
        ("BUILD.bazel", ""),
        ("pkg/a/b/m.py", ""),
        ("pkg/c/d.txt", ""),
        ("pkg/e/__init__.py", ""),
        ("pkg/e/f.py", ""),
    ]);
    let bin = analyse(&repos, "//pkg:bin").await.unwrap();
    let tree = bin
        .actions
        .iter()
        .find(|a| a.mnemonic == "SymlinkTree")
        .unwrap();
    let ActionKind::SymlinkTree { empty_files, .. } = &tree.kind else {
        panic!()
    };
    assert_eq!(
        empty_files,
        &[
            "__init__.py",
            "_main/pkg/__init__.py",
            "_main/pkg/a/__init__.py",
            "_main/pkg/a/b/__init__.py",
        ]
    );
}

/// A filegroup's default runfiles are its `data`, not its `srcs`, as in Bazel; rules_rust reads a linker's tools out of them.
#[tokio::test(flavor = "multi_thread")]
async fn a_filegroups_data_is_in_its_default_runfiles() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _impl(ctx):
    got = sorted([f.short_path for f in ctx.attr.dep[DefaultInfo].default_runfiles.files.to_list()])
    if got != ["pkg/data.txt"]:
        fail("runfiles were " + str(got))
    if [f.short_path for f in ctx.attr.dep[DefaultInfo].files.to_list()] != ["pkg/src.txt"]:
        fail("files were not the srcs")
    return []

check = rule(implementation = _impl, attrs = {"dep": attr.label()})
"#,
        ),
        (
            "pkg/BUILD.bazel",
            "load('//:defs.bzl', 'check')\nfilegroup(name = 'fg', srcs = ['src.txt'], data = ['data.txt'])\ncheck(name = 'c', dep = ':fg')\n",
        ),
        ("BUILD.bazel", ""),
        ("pkg/src.txt", ""),
        ("pkg/data.txt", ""),
    ]);
    analyse(&repos, "//pkg:c").await.unwrap();
}

/// What rules_shell's `runfiles` library is made of: root symlinks that a
/// `data` edge and an alias both carry to the binary that wants them.
#[tokio::test(flavor = "multi_thread")]
async fn root_symlinks_go_through_data_and_aliases_into_runfiles() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _links(ctx):
    f = ctx.files.srcs[0]
    rf = ctx.runfiles(root_symlinks = {"top/" + f.basename: f}, skip_conflict_checking = True)
    if len(rf.root_symlinks.to_list()) != 1:
        fail("rule's own runfiles lack the symlink")
    return [DefaultInfo(runfiles = rf)]

links = rule(implementation = _links, attrs = {"srcs": attr.label_list(allow_files = True)})

def _lib(ctx):
    return [DefaultInfo(runfiles = ctx.runfiles(collect_default = True))]

lib = rule(implementation = _lib, attrs = {"data": attr.label_list()})

def _bin(ctx):
    exe = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.write(exe, "", is_executable = True)
    rf = ctx.runfiles(collect_default = True)
    got = [k for k in rf.root_symlinks.to_list()]
    if len(got) != 1:
        fail("root symlinks were " + str(got))
    return [DefaultInfo(executable = exe, runfiles = rf)]

bin = rule(implementation = _bin, executable = True, attrs = {"data": attr.label_list()})
"#,
        ),
        (
            "pkg/BUILD.bazel",
            "load('//:defs.bzl', 'bin', 'lib', 'links')\nlinks(name = 'l', srcs = ['x.sh'])\nlib(name = 'impl', data = [':l'])\nalias(name = 'a', actual = ':impl')\nbin(name = 'b', data = [':a'])\n",
        ),
        ("BUILD.bazel", ""),
        ("pkg/x.sh", ""),
    ]);
    analyse(&repos, "//pkg:b").await.unwrap();
}

/// Probed with `bazel build`: an executable rule's default runfiles hold the
/// executable and what `runfiles` named, not its other files; a rule with no
/// executable has none unless it names them.
#[tokio::test(flavor = "multi_thread")]
async fn an_executable_is_among_its_own_default_runfiles() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _exe(ctx):
    exe = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.write(exe, "", is_executable = True)
    x = ctx.actions.declare_file(ctx.label.name + ".x")
    ctx.actions.write(x, "")
    return [DefaultInfo(files = depset([x]), executable = exe)]

exe = rule(implementation = _exe, executable = True)

def _plain(ctx):
    x = ctx.actions.declare_file(ctx.label.name + ".x")
    ctx.actions.write(x, "")
    return [DefaultInfo(files = depset([x]))]

plain = rule(implementation = _plain)

def _check(ctx):
    got = lambda t: [f.basename for f in t[DefaultInfo].default_runfiles.files.to_list()]
    if got(ctx.attr.exe) != ["e"] or got(ctx.attr.plain) != []:
        fail("runfiles were " + str(got(ctx.attr.exe)) + " and " + str(got(ctx.attr.plain)))
    return []

check = rule(implementation = _check, attrs = {"exe": attr.label(), "plain": attr.label()})
"#,
        ),
        (
            "BUILD.bazel",
            "load(':defs.bzl', 'check', 'exe', 'plain')\nexe(name = 'e')\nplain(name = 'p')\ncheck(name = 'c', exe = ':e', plain = ':p')\n",
        ),
    ]);
    analyse(&repos, "//:c").await.unwrap();
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
    assert_eq!(t.printed.without_sites(), ["toolchain: linux None"]);
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

/// Probed with `bazel build` of the same files: a group has toolchains of
/// its own, `ctx.toolchains` stays the rule's, and an action may only name a
/// group the rule declared.
#[tokio::test(flavor = "multi_thread")]
async fn an_exec_group_resolves_its_own_toolchains_and_an_action_names_only_declared_groups() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _tc_impl(ctx):
    return [platform_common.ToolchainInfo(name = ctx.attr.n)]
my_toolchain = rule(implementation = _tc_impl, attrs = {"n": attr.string()})

def _impl(ctx):
    group = ctx.exec_groups["eg"]
    print("groups:", "eg" in ctx.exec_groups, "" in ctx.exec_groups, group.toolchains["//:tt2"].name, group.toolchains["//:opt"], ctx.toolchains)
    print("group:", group.toolchains)
    if ctx.attr.bad == "get":
        ctx.exec_groups["nope"]
    if ctx.attr.bad == "type":
        group.toolchains["//:other"]
    if ctx.attr.bad == "action":
        ctx.actions.run_shell(outputs = [ctx.actions.declare_file("o")], command = "true", exec_group = "nope")
    return []
r = rule(
    implementation = _impl,
    attrs = {"bad": attr.string()},
    toolchains = ["//:tt"],
    exec_groups = {"eg": exec_group(toolchains = ["//:tt2", config_common.toolchain_type("//:opt", mandatory = False)])},
)
"#,
        ),
        (
            "BUILD.bazel",
            r#"
load(":defs.bzl", "my_toolchain", "r")
toolchain_type(name = "tt")
toolchain_type(name = "tt2")
toolchain_type(name = "opt")
toolchain_type(name = "other")
my_toolchain(name = "one", n = "one")
my_toolchain(name = "two", n = "two")
toolchain(name = "tc1", toolchain_type = ":tt", toolchain = ":one")
toolchain(name = "tc2", toolchain_type = ":tt2", toolchain = ":two")
r(name = "ok")
r(name = "get", bad = "get")
r(name = "type", bad = "type")
r(name = "action", bad = "action")
"#,
        ),
    ]);
    let registered = vec![
        (String::new(), "//:tc1".to_owned()),
        (String::new(), "//:tc2".to_owned()),
    ];
    let ok = analyse_registering(&repos, "//:ok", config(), registered.clone())
        .await
        .unwrap();
    assert_eq!(
        ok.printed.without_sites(),
        [
            "groups: True False two None <toolchain_context.resolved_labels: //:tt>",
            "group: <toolchain_context.resolved_labels: //:tt2, //:opt>"
        ]
    );
    for (target, wanted) in [
        (
            "//:get",
            "In r rule //:get, unrecognized exec group 'nope' requested. Available exec groups: [eg]",
        ),
        (
            "//:type",
            "In r rule //:type, toolchain type //:other was requested but only types [//:tt2, //:opt] are configured",
        ),
        (
            "//:action",
            "Action declared for non-existent exec group 'nope'.",
        ),
    ] {
        let error = analyse_registering(&repos, target, config(), registered.clone())
            .await
            .unwrap_err();
        assert!(error.contains(wanted), "{error}");
    }
}

/// Probed with `bazel aquery` of the same files: an action in an exec group
/// runs on the platform the group's constraints pick, whether the rule or the
/// target (`exec_group_compatible_with`) added them.
#[tokio::test(flavor = "multi_thread")]
async fn an_exec_group_runs_its_actions_on_a_platform_of_its_own() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _impl(ctx):
    a = ctx.actions.declare_file(ctx.label.name + ".a")
    b = ctx.actions.declare_file(ctx.label.name + ".b")
    ctx.actions.run_shell(outputs = [a], command = "true", mnemonic = "Own")
    ctx.actions.run_shell(outputs = [b], command = "true", mnemonic = "Grouped", exec_group = "eg")
    return [DefaultInfo(files = depset([a, b]))]
by_rule = rule(implementation = _impl, exec_groups = {"eg": exec_group(exec_compatible_with = ["//:c2"])})
by_target = rule(implementation = _impl, exec_groups = {"eg": exec_group()})
"#,
        ),
        (
            "BUILD.bazel",
            r#"
load(":defs.bzl", "by_rule", "by_target")
constraint_setting(name = "cs")
constraint_value(name = "c1", constraint_setting = ":cs")
constraint_value(name = "c2", constraint_setting = ":cs")
platform(name = "ex1", constraint_values = [":c1"])
platform(name = "ex2", constraint_values = [":c2"])
by_rule(name = "r")
by_target(name = "t", exec_group_compatible_with = {"eg": [":c2"]})
"#,
        ),
    ]);
    let platforms = vec![
        (String::new(), "//:ex1".to_owned()),
        (String::new(), "//:ex2".to_owned()),
    ];
    for target in ["//:r", "//:t"] {
        let done = analyse_on(&repos, target, config(), Vec::new(), platforms.clone())
            .await
            .unwrap();
        let groups: Vec<(&str, Option<&str>)> = done
            .actions
            .iter()
            .map(|a| (a.mnemonic.as_str(), a.exec_group.as_deref()))
            .collect();
        assert_eq!(groups, [("Own", None), ("Grouped", Some("eg"))]);
        let platform: Vec<(String, String)> = done
            .exec_group_platforms
            .iter()
            .map(|(name, p)| {
                (
                    name.clone(),
                    p.as_ref().map(expand_label_text).unwrap_or_default(),
                )
            })
            .collect();
        assert_eq!(
            platform,
            [("eg".to_owned(), "//:ex2".to_owned())],
            "{target}"
        );
    }
}

/// Probed with `bazel build --toolchain_resolution_debug` of the same files.
#[tokio::test(flavor = "multi_thread")]
async fn toolchain_resolution_debug_says_how_each_type_resolved() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _tc_impl(ctx):
    return [platform_common.ToolchainInfo()]
my_toolchain = rule(implementation = _tc_impl)
def _impl(ctx):
    return []
r = rule(implementation = _impl, toolchains = ["//:tt"])
"#,
        ),
        (
            "BUILD.bazel",
            r#"
load(":defs.bzl", "my_toolchain", "r")
constraint_setting(name = "cs")
constraint_value(name = "c1", constraint_setting = ":cs")
constraint_value(name = "c2", constraint_setting = ":cs")
constraint_setting(name = "os")
constraint_value(name = "osx", constraint_setting = ":os")
config_setting(name = "never", values = {"compilation_mode": "dbg"})
platform(name = "ex1", constraint_values = [":c1"])
platform(name = "ex2", constraint_values = [":c2"])
toolchain_type(name = "tt")
my_toolchain(name = "a")
my_toolchain(name = "b")
my_toolchain(name = "c")
toolchain(name = "t1", toolchain_type = ":tt", toolchain = ":a", target_compatible_with = [":osx"])
toolchain(name = "t2", toolchain_type = ":tt", toolchain = ":b", exec_compatible_with = [":c2"])
toolchain(name = "t3", toolchain_type = ":tt", toolchain = ":c", target_settings = [":never"])
r(name = "x")
"#,
        ),
    ]);
    let said = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let sink = said.clone();
    let debug = crate::ResolutionDebug {
        matches: Arc::new(|label| label == "//:tt"),
        emit: Arc::new(move |message| sink.lock().unwrap().push(message.to_owned())),
    };
    let toolchains: Vec<(String, String)> = ["t1", "t2", "t3"]
        .iter()
        .map(|t| (String::new(), format!("//:{t}")))
        .collect();
    let platforms = vec![
        (String::new(), "//:ex1".to_owned()),
        (String::new(), "//:ex2".to_owned()),
    ];
    let x = analyse_traced(&repos, "//:x", config(), toolchains, platforms, Some(debug))
        .await
        .unwrap();
    // A rule with no toolchain says where it runs when the analysis is done;
    // alone, with nothing to place it, that is the first platform.
    let lines = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let sink = lines.clone();
    let debug_b = crate::ResolutionDebug {
        matches: Arc::new(|label| label == "//:b"),
        emit: Arc::new(move |message| sink.lock().unwrap().push(message.to_owned())),
    };
    let b = analyse_traced(
        &repos,
        "//:b",
        config(),
        Vec::new(),
        vec![(String::new(), "//:ex1".to_owned())],
        Some(debug_b.clone()),
    )
    .await
    .unwrap();
    assert!(b.debug_no_toolchains);
    crate::say_no_toolchains(&debug_b, &b, "@@platforms//host:host");
    assert_eq!(
        *lines.lock().unwrap(),
        [
            "INFO: ToolchainResolution: Target platform @@platforms//host:host: Selected execution platform //:ex1, "
        ]
    );
    // The implementation runs where the toolchain was resolved to run.
    assert_eq!(
        x.toolchain_platforms,
        [(
            Label {
                repo: String::new(),
                package: String::new(),
                name: "b".into()
            },
            Label {
                repo: String::new(),
                package: String::new(),
                name: "ex2".into()
            }
        )],
        "{:?}",
        x.toolchain_platforms
    );
    let said = said.lock().unwrap().join("\n");
    assert_eq!(
        said,
        "INFO: ToolchainResolution: Performing resolution of //:tt for target platform @@platforms//host:host
      ToolchainResolution:   Rejected toolchain //:t3; mismatching target_settings: never
      ToolchainResolution:   Rejected toolchain //:t1 (resolves to //:a) ; mismatching values: osx
      ToolchainResolution:   Toolchain //:t2 (resolves to //:b) is compatible with target platform, searching for execution platforms:
      ToolchainResolution:     Incompatible execution platform //:ex1; mismatching values: c2
      ToolchainResolution:     Compatible execution platform //:ex2
      ToolchainResolution: Recap of selected //:tt toolchains for target platform @@platforms//host:host:
      ToolchainResolution:   Selected //:b to run on execution platform //:ex2
INFO: ToolchainResolution: Target platform @@platforms//host:host: Selected execution platform //:ex2, type //:tt -> toolchain //:b"
    );
}

/// Probed with `bazel build` of the same files: a type with no toolchain for
/// the target platform is the one named; types that each have one, but not
/// on the same execution platform, are told apart.
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_resolution_says_which_it_is() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _tc_impl(ctx):
    return [platform_common.ToolchainInfo()]
my_toolchain = rule(implementation = _tc_impl)
def _impl(ctx):
    return []
both = rule(implementation = _impl, toolchains = ["//:ty", config_common.toolchain_type("//:opt", mandatory = False), "//:tt"])
lacking = rule(implementation = _impl, toolchains = ["//:tz", "//:ty", "//:tt", "//:nosuch"])
"#,
        ),
        (
            "BUILD.bazel",
            r#"
load(":defs.bzl", "my_toolchain", "both", "lacking")
constraint_setting(name = "cs")
constraint_value(name = "c1", constraint_setting = ":cs")
constraint_value(name = "c2", constraint_setting = ":cs")
platform(name = "ex1", constraint_values = [":c1"])
platform(name = "ex2", constraint_values = [":c2"])
toolchain_type(name = "tt")
toolchain_type(name = "ty")
toolchain_type(name = "tz")
toolchain_type(name = "opt")
toolchain_type(name = "nosuch")
my_toolchain(name = "impl")
toolchain(name = "tc_tt", toolchain_type = ":tt", toolchain = ":impl", exec_compatible_with = [":c2"])
toolchain(name = "tc_ty", toolchain_type = ":ty", toolchain = ":impl", exec_compatible_with = [":c1"])
both(name = "b")
lacking(name = "l")
"#,
        ),
    ]);
    let toolchains = vec![
        (String::new(), "//:tc_tt".to_owned()),
        (String::new(), "//:tc_ty".to_owned()),
    ];
    let platforms = vec![
        (String::new(), "//:ex1".to_owned()),
        (String::new(), "//:ex2".to_owned()),
    ];
    let both = analyse_on(
        &repos,
        "//:b",
        config(),
        toolchains.clone(),
        platforms.clone(),
    )
    .await
    .unwrap_err();
    assert!(
        both.contains(
            "Unable to find an execution platform for toolchains [//:ty, //:opt, //:tt] and target platform @@platforms//host:host from available execution platforms [//:ex1, //:ex2]"
        ),
        "{both}"
    );
    let lacking = analyse_on(&repos, "//:l", config(), toolchains, platforms)
        .await
        .unwrap_err();
    assert!(
        lacking.contains(
            "No matching toolchains found for types:\n  //:tz\n  //:nosuch\nTo debug, rerun with --toolchain_resolution_debug='//:tz|//:nosuch'"
        ),
        "{lacking}"
    );
}

/// Bazel picks the execution platform among the registered ones: the first
/// that meets the target's `exec_compatible_with` and has a toolchain of each
/// mandatory type whose `exec_compatible_with` it meets.
#[tokio::test(flavor = "multi_thread")]
async fn toolchains_are_matched_to_the_execution_platform_the_target_picks() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _tc_impl(ctx):
    return [platform_common.ToolchainInfo(name = ctx.attr.n)]
my_toolchain = rule(implementation = _tc_impl, attrs = {"n": attr.string()})

def _impl(ctx):
    print("toolchain:", ctx.toolchains["//:tt"].name)
    return []
r = rule(implementation = _impl, toolchains = ["//:tt"])
"#,
        ),
        (
            "BUILD.bazel",
            r#"
load(":defs.bzl", "my_toolchain", "r")
toolchain_type(name = "tt")
constraint_setting(name = "os")
constraint_value(name = "linux", constraint_setting = ":os")
constraint_value(name = "windows", constraint_setting = ":os")
platform(name = "windows_box", constraint_values = [":windows"])
platform(name = "linux_box", constraint_values = [":linux"])
my_toolchain(name = "impl_linux", n = "linux")
my_toolchain(name = "impl_windows", n = "windows")
toolchain(name = "tc_linux", toolchain_type = ":tt", toolchain = ":impl_linux", exec_compatible_with = [":linux"])
toolchain(name = "tc_windows", toolchain_type = ":tt", toolchain = ":impl_windows", exec_compatible_with = [":windows"])
r(name = "any")
r(name = "wants_linux", exec_compatible_with = [":linux"])
r(name = "wants_nothing_there", exec_compatible_with = [":os"])
"#,
        ),
    ]);
    let registered = vec![
        (String::new(), "//:tc_windows".to_owned()),
        (String::new(), "//:tc_linux".to_owned()),
    ];
    let platforms = vec![
        (String::new(), "//:windows_box".to_owned()),
        (String::new(), "//:linux_box".to_owned()),
    ];
    let on = |label: &'static str, platforms: Vec<(String, String)>| {
        analyse_on(&repos, label, config(), registered.clone(), platforms)
    };
    // The first platform that has a toolchain is the one.
    assert_eq!(
        on("//:any", platforms.clone())
            .await
            .unwrap()
            .printed
            .without_sites(),
        ["toolchain: windows"]
    );
    // The target's own constraints rule platforms out.
    assert_eq!(
        on("//:wants_linux", platforms.clone())
            .await
            .unwrap()
            .printed
            .without_sites(),
        ["toolchain: linux"]
    );
    // No platform has them, or there is none: no toolchain matches.
    let error = on("//:wants_nothing_there", platforms).await.unwrap_err();
    assert!(error.contains("No matching toolchains found"), "{error}");
    let error = on("//:any", Vec::new()).await.unwrap_err();
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
            "load(':defs.bzl', 'tool', 'use')\ntool(name = 'tool')\nuse(name = 'u')\ngenrule(name = 'g', outs = ['g.txt'], tools = [':tool'], cmd = '$(location :tool) > $@')\nalias(name = 'tool_alias', actual = ':tool')\ngenrule(name = 'ga', outs = ['ga.txt'], tools = [':tool_alias'], cmd = '$(location :tool_alias) > $@')\n",
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
    // An executable tool brings its runfiles tree, as `bazel aquery` showed.
    let inputs: Vec<String> = g.actions[0].inputs.iter().map(|a| a.exec_path()).collect();
    assert!(
        inputs.contains(&"bazel-out/k8-opt-exec/bin/tool.sh.runfiles".to_owned()),
        "{inputs:?}"
    );
    // And so does one named through an alias.
    let ga = analyse(&repos, "//:ga").await.unwrap();
    let inputs: Vec<String> = ga.actions[0].inputs.iter().map(|a| a.exec_path()).collect();
    assert!(
        inputs.contains(&"bazel-out/k8-opt-exec/bin/tool.sh.runfiles".to_owned()),
        "{inputs:?}"
    );
}

/// Probed with `bazel build` on the same files: a tool is built for the
/// platform the target that wants it runs on, which is its target platform and
/// names its output directory.
#[tokio::test(flavor = "multi_thread")]
async fn a_tool_is_built_for_the_execution_platform_of_the_target_that_wants_it() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _tool(ctx):
    out = ctx.actions.declare_file(ctx.label.name + ".txt")
    ctx.actions.write(out, "x")
    print("c1:", ctx.target_platform_has_constraint(ctx.attr._c1[platform_common.ConstraintValueInfo]), "c2:", ctx.target_platform_has_constraint(ctx.attr._c2[platform_common.ConstraintValueInfo]))
    return [DefaultInfo(files = depset([out]))]
tool = rule(implementation = _tool, attrs = {"_c1": attr.label(default = "//:c1"), "_c2": attr.label(default = "//:c2")})
def _use(ctx):
    return [DefaultInfo(files = depset(ctx.files.tool))]
use = rule(implementation = _use, attrs = {"tool": attr.label(cfg = "exec")})
def _tc(ctx):
    return [platform_common.ToolchainInfo()]
tc = rule(implementation = _tc)
with_toolchain = rule(implementation = _use, attrs = {"tool": attr.label(cfg = "exec")}, toolchains = ["//:tt"])
"#,
        ),
        (
            "BUILD.bazel",
            r#"
load(":defs.bzl", "tc", "tool", "use", "with_toolchain")
constraint_setting(name = "cs")
constraint_value(name = "c1", constraint_setting = ":cs")
constraint_value(name = "c2", constraint_setting = ":cs")
platform(name = "ex1", constraint_values = [":c1"])
platform(name = "ex2", constraint_values = [":c2"])
toolchain_type(name = "tt")
tc(name = "tci")
toolchain(name = "tcd", toolchain_type = ":tt", toolchain = ":tci", exec_compatible_with = [":c2"])
tool(name = "t")
use(name = "first", tool = ":t")
use(name = "wants_c2", tool = ":t", exec_compatible_with = [":c2"])
with_toolchain(name = "by_toolchain", tool = ":t")
"#,
        ),
    ]);
    let platforms = vec![
        (String::new(), "//:ex1".to_owned()),
        (String::new(), "//:ex2".to_owned()),
    ];
    let toolchains = vec![(String::new(), "//:tcd".to_owned())];
    let on = |label: &'static str| {
        analyse_on(
            &repos,
            label,
            config(),
            toolchains.clone(),
            platforms.clone(),
        )
    };
    // The first platform, unless the target's constraints or toolchains say otherwise.
    for (label, dir, printed) in [
        ("//:first", "ex1", "c1: True c2: False"),
        ("//:wants_c2", "ex2", "c1: False c2: True"),
        ("//:by_toolchain", "ex2", "c1: False c2: True"),
    ] {
        let target = on(label).await.unwrap();
        assert_eq!(
            paths(&target.files),
            [format!("bazel-out/{dir}-opt-exec/bin/t.txt")],
            "{label}"
        );
        let tool = analyse_on(
            &repos,
            "//:t",
            target.exec_configuration.clone().unwrap(),
            toolchains.clone(),
            platforms.clone(),
        )
        .await
        .unwrap();
        assert_eq!(tool.printed.without_sites(), [printed], "{label}");
    }
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

/// Probed with `bazel build` on the same files: a split gives `ctx.split_attr`
/// a branch for each key, in the order of the configurations, and `ctx.attr`
/// the targets of every branch; a split on `rule(cfg =)` is refused.
#[tokio::test(flavor = "multi_thread")]
async fn a_split_transition_gives_one_dependency_for_each_key() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _leaf(ctx):
    out = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.write(out, "x")
    return [DefaultInfo(files = depset([out]))]
leaf = rule(implementation = _leaf)

def _split(settings, attr):
    return {
        "c": {"//command_line_option:compilation_mode": "opt"},
        "a": {"//command_line_option:compilation_mode": "dbg"},
    }
split = transition(implementation = _split, inputs = [], outputs = ["//command_line_option:compilation_mode"])
def _list(settings, attr):
    return [{"//command_line_option:compilation_mode": "opt"}, {"//command_line_option:compilation_mode": "dbg"}]
lists = transition(implementation = _list, inputs = [], outputs = ["//command_line_option:compilation_mode"])

def _top(ctx):
    print("keys:", ctx.split_attr.dep.keys(), "attr:", len(ctx.attr.dep))
    print("by key:", {k: [f.path for f in v[DefaultInfo].files.to_list()] for k, v in ctx.split_attr.dep.items()})
    return [DefaultInfo(files = depset(ctx.files.dep))]
top = rule(implementation = _top, attrs = {"dep": attr.label(cfg = split)})
by_index = rule(implementation = _top, attrs = {"dep": attr.label(cfg = lists)})
rule_split = rule(implementation = _leaf, cfg = split)
"#,
        ),
        (
            "BUILD.bazel",
            "load(':defs.bzl', 'leaf', 'top', 'by_index', 'rule_split')\nleaf(name = 'leaf')\ntop(name = 't', dep = ':leaf')\nby_index(name = 'i', dep = ':leaf')\nrule_split(name = 'r')\n",
        ),
    ]);
    let t = analyse(&repos, "//:t").await.unwrap();
    assert_eq!(
        t.printed.without_sites(),
        [
            r#"keys: ["a", "c"] attr: 2"#,
            r#"by key: {"a": ["bazel-out/k8-dbg/bin/leaf"], "c": ["bazel-out/k8-opt/bin/leaf"]}"#
        ]
    );
    assert_eq!(
        paths(&t.files),
        ["bazel-out/k8-dbg/bin/leaf", "bazel-out/k8-opt/bin/leaf"]
    );
    let i = analyse(&repos, "//:i").await.unwrap();
    assert_eq!(i.printed.without_sites()[0], r#"keys: ["1", "0"] attr: 2"#);
    let err = analyse(&repos, "//:r").await.unwrap_err();
    assert!(
        err.to_string().contains(
            "Rule transition only allowed to return a single transitioned configuration."
        ),
        "{err}"
    );
}

/// A transition that fails says what it did wrong where its function is
/// written, which is the first of the two events Bazel prints, and for which
/// target and edge, which are the second.
#[tokio::test(flavor = "multi_thread")]
async fn a_failing_transition_says_where_its_function_is_written() {
    let (_dir, repos) = workspace(&[
        (
            "defs.bzl",
            r#"
def _leaf(ctx):
    return []
def _bad(settings, attr):
    return {"//command_line_option:copt": ["x"]}
bad = transition(implementation = _bad, inputs = [], outputs = ["//command_line_option:compilation_mode"])
leaf = rule(implementation = _leaf)
rule_bad = rule(implementation = _leaf, cfg = bad)
top = rule(implementation = _leaf, attrs = {"dep": attr.label(cfg = bad)})
"#,
        ),
        (
            "BUILD.bazel",
            "load(':defs.bzl', 'leaf', 'rule_bad', 'top')\nleaf(name = 'leaf')\nrule_bad(name = 'r')\ntop(name = 't', dep = ':leaf')\n",
        ),
    ]);
    for (target, edge) in [("r", ""), ("t", "on dependency edge //:t (")] {
        let err = analyse(&repos, &format!("//:{target}")).await.unwrap_err();
        let message = err.to_string();
        let parts = fjfj_starlark::split_transition_error(&message).expect(&message);
        assert!(parts.location.ends_with("defs.bzl:4:5"), "{message}");
        assert_eq!(
            parts.text,
            "invalid result from transition function: transition function returned undeclared \
             output '//command_line_option:copt'"
        );
        assert!(parts.edge.starts_with(edge), "{message}");
        assert_eq!(parts.target, format!("//:{target}"));
    }
}

/// `a.and_then(b)` runs `b` on what `a` made, reading what `a` set; a split in
/// each makes a branch for each pair, keyed `a,x`.
#[tokio::test(flavor = "multi_thread")]
async fn a_composed_transition_runs_each_part_on_what_the_one_before_made() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
MODE = "//command_line_option:compilation_mode"
COPT = "//command_line_option:copt"
def _leaf(ctx):
    out = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.write(out, "x")
    return [DefaultInfo(files = depset([out]))]
leaf = rule(implementation = _leaf)

def _first(settings, attr):
    return {COPT: ["-a"]}
def _second(settings, attr):
    # Reads what the first set, and the mode it did not touch.
    return {MODE: "opt" if settings[COPT] == ["-a"] else "dbg"}
first = transition(implementation = _first, inputs = [], outputs = [COPT])
second = transition(implementation = _second, inputs = [COPT], outputs = [MODE])
def _by_mode(settings, attr):
    return {"a": {MODE: "dbg"}, "b": {MODE: "opt"}}
def _by_copt(settings, attr):
    return {"x": {COPT: ["-x"]}, "y": {COPT: ["-y"]}}
by_mode = transition(implementation = _by_mode, inputs = [], outputs = [MODE])
by_copt = transition(implementation = _by_copt, inputs = [], outputs = [COPT])

def _top(ctx):
    print({k: [f.path for f in v[DefaultInfo].files.to_list()] for k, v in ctx.split_attr.dep.items()})
    return []
def _one(ctx):
    # A transition of a label attribute makes it a list, even if it is no split.
    print(type(ctx.attr.dep), [f.path for f in ctx.attr.dep[0][DefaultInfo].files.to_list()])
    print(ctx.split_attr.dep.keys())
    return []
one = rule(implementation = _one, attrs = {"dep": attr.label(cfg = first.and_then(second))})
two = rule(implementation = _top, attrs = {"dep": attr.label(cfg = by_mode.and_then(by_copt))})
"#,
        ),
        (
            "BUILD.bazel",
            "load(':defs.bzl', 'leaf', 'one', 'two')\nleaf(name = 'leaf')\none(name = 'o', dep = ':leaf')\ntwo(name = 'w', dep = ':leaf')\n",
        ),
    ]);
    let o = analyse(&repos, "//:o").await.unwrap();
    let printed = o.printed.without_sites();
    assert!(printed[0].starts_with("list [\""), "{printed:?}");
    assert!(printed[0].contains("k8-opt"), "{printed:?}");
    assert_eq!(printed[1], "[None]");
    let w = analyse(&repos, "//:w").await.unwrap();
    let printed = w.printed.without_sites();
    for key in ["a,x", "a,y", "b,x", "b,y"] {
        assert!(printed[0].contains(&format!("\"{key}\"")), "{printed:?}");
    }
}

/// A transition that returns an empty dict changes nothing; one that returns
/// some of its outputs and not all of them says which it left out.
#[tokio::test(flavor = "multi_thread")]
async fn a_transition_may_return_an_empty_dict_but_not_some_of_its_outputs() {
    let (_dir, repos) = workspace(&[
        (
            "defs.bzl",
            r#"
def _show(ctx):
    print("mode:", ctx.var["COMPILATION_MODE"])
    return []
def _none(settings, attr):
    return {}
def _some(settings, attr):
    return {"//command_line_option:copt": ["-O1"]}
def _list(settings, attr):
    return [{}]
outs = ["//command_line_option:copt", "//command_line_option:compilation_mode"]
none = transition(implementation = _none, inputs = [], outputs = outs)
some = transition(implementation = _some, inputs = [], outputs = outs)
listed = transition(implementation = _list, inputs = [], outputs = outs)
r_none = rule(implementation = _show, cfg = none)
r_some = rule(implementation = _show, cfg = some)
r_list = rule(implementation = _show, cfg = listed)
"#,
        ),
        (
            "BUILD.bazel",
            "load(':defs.bzl', 'r_none', 'r_some', 'r_list')\nr_none(name = 'n')\nr_some(name = 's')\nr_list(name = 'l')\n",
        ),
    ]);
    let n = analyse(&repos, "//:n").await.unwrap();
    assert_eq!(n.printed.without_sites(), ["mode: fastbuild"]);
    for target in ["s", "l"] {
        let message = analyse(&repos, &format!("//:{target}"))
            .await
            .unwrap_err()
            .to_string();
        let parts = fjfj_starlark::split_transition_error(&message).expect(&message);
        let missing = if target == "s" {
            "//command_line_option:compilation_mode"
        } else {
            "//command_line_option:copt,//command_line_option:compilation_mode"
        };
        assert_eq!(
            parts.text,
            format!(
                "invalid result from transition function: transition outputs [{missing}] were \
                 not defined by transition function"
            )
        );
    }
}

/// A `toolchain` may name a `toolchain_type` through an alias (rules_rust does),
/// and a `label_flag` is the target its value names.
#[tokio::test(flavor = "multi_thread")]
async fn toolchain_types_may_be_aliases_and_a_label_flag_is_its_value() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        ("a.txt", ""),
        ("b.txt", ""),
        (
            "defs.bzl",
            r#"
def _tc_impl(ctx):
    return [platform_common.ToolchainInfo(name = ctx.attr.n)]
my_toolchain = rule(implementation = _tc_impl, attrs = {"n": attr.string()})

def _impl(ctx):
    print("toolchain:", ctx.toolchains["//:tt"].name)
    return []
r = rule(implementation = _impl, toolchains = ["//:tt"])
"#,
        ),
        (
            "BUILD.bazel",
            r#"
load(":defs.bzl", "my_toolchain", "r")
toolchain_type(name = "tt")
alias(name = "tt_alias", actual = ":tt")
my_toolchain(name = "impl", n = "via alias")
toolchain(name = "tc", toolchain_type = ":tt_alias", toolchain = ":impl")
r(name = "t")
filegroup(name = "default_target", srcs = ["a.txt"])
filegroup(name = "other_target", srcs = ["b.txt"])
label_flag(name = "flag", build_setting_default = ":default_target")
filegroup(name = "user", srcs = [":flag"])
"#,
        ),
    ]);
    let registered = vec![(String::new(), "//:tc".to_owned())];
    let t = analyse_registering(&repos, "//:t", config(), registered)
        .await
        .unwrap();
    assert_eq!(t.printed.without_sites(), ["toolchain: via alias"]);
    let user = analyse(&repos, "//:user").await.unwrap();
    assert_eq!(paths(&user.files), ["a.txt"]);
    let mut set = config();
    set.settings.insert(
        "//:flag".to_owned(),
        fjfj_graph::SettingValue::Str("//:other_target".to_owned()),
    );
    let user = analyse_registering(&repos, "//:user", set, Vec::new())
        .await
        .unwrap();
    assert_eq!(paths(&user.files), ["b.txt"]);
}

/// A transition's `attr` has every attribute of the rule: a label as a `Label`,
/// and one with no value as `None` (rules_rust reads `attr.platform`).
#[tokio::test(flavor = "multi_thread")]
async fn a_transition_sees_label_attributes_and_unset_ones() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _go(settings, attr):
    if attr.platform:
        fail("platform should be None")
    if str(attr.helper) != "@@//:h" or [str(l) for l in attr.more] != ["@@//:h"]:
        fail("labels: %s %s" % (attr.helper, attr.more))
    return {"//command_line_option:compilation_mode": "dbg"}
go = transition(implementation = _go, inputs = [], outputs = ["//command_line_option:compilation_mode"])

def _impl(ctx):
    return [DefaultInfo()]
r = rule(
    implementation = _impl,
    cfg = go,
    attrs = {
        "platform": attr.label(default = None),
        "helper": attr.label(default = "//:h"),
        "more": attr.label_list(default = ["//:h"]),
    },
)
"#,
        ),
        (
            "BUILD.bazel",
            "load(':defs.bzl', 'r')\nfilegroup(name = 'h')\nr(name = 't')\n",
        ),
    ]);
    analyse(&repos, "//:t").await.unwrap();
}

/// A `constraint_value` is a condition of `select()`, as in Bazel, and
/// `cc_toolchain_suite` is a macro that makes a bare `filegroup`.
#[tokio::test(flavor = "multi_thread")]
async fn a_constraint_value_can_be_a_select_condition() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        ("a.txt", ""),
        ("b.txt", ""),
        (
            "BUILD.bazel",
            r#"
constraint_setting(name = "os")
constraint_value(name = "linux", constraint_setting = ":os")
filegroup(name = "f", srcs = select({":linux": ["a.txt"], "//conditions:default": ["b.txt"]}))
cc_toolchain_suite(name = "suite", toolchains = {"k8": ":x"}, visibility = ["//visibility:public"])
"#,
        ),
    ]);
    let mut on_linux = config();
    on_linux.constraints.insert(Label {
        repo: String::new(),
        package: String::new(),
        name: "linux".into(),
    });
    let linux = analyse_registering(&repos, "//:f", on_linux, Vec::new())
        .await
        .unwrap();
    assert_eq!(paths(&linux.files), ["a.txt"]);
    let other = analyse(&repos, "//:f").await.unwrap();
    assert_eq!(paths(&other.files), ["b.txt"]);
    assert_eq!(
        analyse(&repos, "//:suite").await.unwrap().rule_class,
        Some("filegroup".to_owned())
    );
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

/// An aspect on an attribute runs on the targets it names and, along its
/// `attr_aspects`, on theirs; the rule sees what it provides.
#[tokio::test(flavor = "multi_thread")]
async fn an_aspect_on_an_attribute_follows_the_attributes_it_names() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
AInfo = provider(fields = ["n"])

def _count(target, ctx):
    n = 1
    for d in ctx.rule.attr.deps:
        n += d[AInfo].n
    out = ctx.actions.declare_file(target.label.name + ".count")
    ctx.actions.write(out, "%d\n" % n)
    return [AInfo(n = n), DefaultInfo(files = depset([out]))]

count = aspect(implementation = _count, attr_aspects = ["deps"])

def _impl(ctx):
    out = ctx.actions.declare_file(ctx.label.name + ".txt")
    total = 0
    for d in ctx.attr.deps:
        total += d[AInfo].n
    ctx.actions.write(out, "%d\n" % total)
    return [DefaultInfo(files = depset([out]))]

def _plain(ctx):
    return [DefaultInfo()]

plain = rule(implementation = _plain, attrs = {"deps": attr.label_list()})
top = rule(implementation = _impl, attrs = {"deps": attr.label_list(aspects = [count])})
"#,
        ),
        (
            "BUILD.bazel",
            "load(':defs.bzl', 'plain', 'top')\nplain(name = 'a')\nplain(name = 'b', deps = [':a'])\ntop(name = 't', deps = [':b'])\n",
        ),
    ]);
    let t = analyse(&repos, "//:t").await.unwrap();
    let [action] = &t.actions[..] else {
        panic!("{:?}", t.actions)
    };
    // b counts itself and a.
    assert!(
        matches!(&action.kind, ActionKind::WriteFile { contents, .. } if contents == b"2\n"),
        "{:?}",
        action.kind
    );
    let [aspect] = &t.aspect_deps[..] else {
        panic!("{:?}", t.aspect_deps)
    };
    assert_eq!(aspect.target.label.name, "b");
}

/// An aspect's `requires` run on the same target first, and what they provide
/// is there to read on the target the aspect looks at.
#[tokio::test(flavor = "multi_thread")]
async fn an_aspect_sees_what_the_aspects_it_requires_provide() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
InnerInfo = provider(fields = ["name"])
OuterInfo = provider(fields = ["seen"])

def _inner(target, ctx):
    return [InnerInfo(name = target.label.name)]

inner = aspect(implementation = _inner)

def _outer(target, ctx):
    return [OuterInfo(seen = target[InnerInfo].name)]

outer = aspect(implementation = _outer, requires = [inner])

def _impl(ctx):
    out = ctx.actions.declare_file(ctx.label.name + ".txt")
    ctx.actions.write(out, ctx.attr.dep[OuterInfo].seen)
    return [DefaultInfo(files = depset([out]))]

def _plain(ctx):
    return [DefaultInfo()]

plain = rule(implementation = _plain)
top = rule(implementation = _impl, attrs = {"dep": attr.label(aspects = [outer])})
"#,
        ),
        (
            "BUILD.bazel",
            "load(':defs.bzl', 'plain', 'top')\nplain(name = 'a')\ntop(name = 't', dep = ':a')\n",
        ),
    ]);
    let t = analyse(&repos, "//:t").await.unwrap();
    let [action] = &t.actions[..] else {
        panic!("{:?}", t.actions)
    };
    assert!(
        matches!(&action.kind, ActionKind::WriteFile { contents, .. } if contents == b"a"),
        "{:?}",
        action.kind
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn ctx_outputs_has_an_output_list_as_a_list_and_an_unset_output_as_none() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _impl(ctx):
    outs = ctx.outputs.ol + ([ctx.outputs.o] if ctx.outputs.o else [])
    for f in outs:
        ctx.actions.write(f, "")
    return [DefaultInfo(files = depset(outs))]
rule_with_outputs = rule(
    implementation = _impl,
    attrs = {"o": attr.output(), "ol": attr.output_list()},
)
"#,
        ),
        (
            "BUILD.bazel",
            r#"
load(":defs.bzl", "rule_with_outputs")
rule_with_outputs(name = "set", o = "one.txt", ol = ["a.txt", "b.txt"])
rule_with_outputs(name = "unset")
"#,
        ),
    ]);
    let set = analyse(&repos, "//:set").await.unwrap();
    assert_eq!(
        paths(&set.files),
        [
            format!("{BIN}/a.txt"),
            format!("{BIN}/b.txt"),
            format!("{BIN}/one.txt")
        ]
    );
    let unset = analyse(&repos, "//:unset").await.unwrap();
    assert!(paths(&unset.files).is_empty());
}

/// Probed with `bazel build`: `collect_default` makes a rule's `data` runfiles
/// of whatever the targets make, and of `srcs` and `deps` only their runfiles.
#[tokio::test(flavor = "multi_thread")]
async fn collect_default_takes_the_files_of_data_and_the_runfiles_of_the_rest() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "defs.bzl",
            r#"
def _a(ctx):
    f = ctx.actions.declare_file(ctx.label.name + ".txt")
    ctx.actions.write(f, "x")
    return [DefaultInfo(files = depset([f]))]

a = rule(implementation = _a)

def _b(ctx):
    e = ctx.actions.declare_file(ctx.label.name)
    ctx.actions.write(e, "", is_executable = True)
    return [DefaultInfo(executable = e, runfiles = ctx.runfiles(collect_default = True))]

b = rule(implementation = _b, executable = True, attrs = {
    "deps": attr.label_list(allow_files = True),
    "srcs": attr.label_list(allow_files = True),
    "data": attr.label_list(allow_files = True),
})
"#,
        ),
        (
            "BUILD.bazel",
            "load(':defs.bzl', 'a', 'b')\na(name = 'made')\nfilegroup(name = 'group', srcs = ['s.txt'])\nb(name = 'bin', deps = [':made', ':group', 'd.txt'], srcs = ['s.txt'], data = [':made', ':group', 'f.txt'])\n",
        ),
        ("s.txt", ""),
        ("d.txt", ""),
        ("f.txt", ""),
    ]);
    let bin = analyse(&repos, "//:bin").await.unwrap();
    let mut got: Vec<String> = bin.runfiles.files.iter().map(|a| a.exec_path()).collect();
    got.sort();
    assert_eq!(
        got,
        [
            format!("{BIN}/bin"),
            format!("{BIN}/made.txt"),
            "f.txt".into(),
            "s.txt".into()
        ]
    );
}

/// What `bazel cquery 'deps(//p:c)'` showed: a target whose `target_compatible_with`
/// the platform does not meet is analysed no further than the constraint values,
/// and what reads it is incompatible too, with its dependencies still analysed.
#[tokio::test(flavor = "multi_thread")]
async fn a_target_the_platform_cannot_build_is_incompatible_and_so_is_what_reads_it() {
    let (_dir, repos) = workspace(&[(
        "BUILD.bazel",
        r#"
constraint_setting(name = "cs")
constraint_value(name = "cv", constraint_setting = ":cs")
constraint_setting(name = "os")
constraint_value(name = "linux", constraint_setting = ":os")
genrule(name = "c", outs = ["c.txt"], cmd = "true", target_compatible_with = [":linux", ":cv"])
genrule(name = "ok", outs = ["ok.txt"], cmd = "true", target_compatible_with = [":linux"])
genrule(name = "reads", srcs = [":c"], outs = ["r.txt"], cmd = "true")
genrule(name = "reads_ok", srcs = [":ok"], outs = ["ro.txt"], cmd = "true")
genrule(name = "unless", outs = ["u.txt"], cmd = "true", target_compatible_with = select({":linux": ["//:cv"], "//conditions:default": []}))
"#,
    )]);
    let mut on_linux = config();
    on_linux.constraints.insert(Label {
        repo: String::new(),
        package: String::new(),
        name: "linux".into(),
    });
    let label = |name: &str| Label {
        repo: String::new(),
        package: String::new(),
        name: name.into(),
    };
    let c = analyse_in(&repos, "//:c", on_linux.clone()).await.unwrap();
    let why = c.incompatible.as_ref().unwrap();
    assert_eq!(why.unsatisfied, [label("cv")]);
    assert_eq!(why.chain.len(), 1);
    // Only what decided it is a dependency, not the genrule's own.
    let deps: Vec<Label> = c.deps.iter().map(|k| k.label.clone()).collect();
    assert_eq!(deps, [label("linux"), label("cv")]);
    assert!(c.actions.is_empty());
    // A platform with all of them builds it.
    let mut both = on_linux.clone();
    both.constraints.insert(label("cv"));
    assert!(
        analyse_in(&repos, "//:c", both)
            .await
            .unwrap()
            .incompatible
            .is_none()
    );
    let ok = analyse_in(&repos, "//:ok", on_linux.clone()).await.unwrap();
    assert!(ok.incompatible.is_none());
    assert!(!ok.actions.is_empty());
    // What reads an incompatible target is, after being analysed in full.
    let reads = analyse_in(&repos, "//:reads", on_linux.clone())
        .await
        .unwrap();
    let why = reads.incompatible.as_ref().unwrap();
    let chain: Vec<Label> = why.chain.iter().map(|k| k.label.clone()).collect();
    assert_eq!(chain, [label("reads"), label("c")]);
    assert_eq!(why.unsatisfied, [label("cv")]);
    assert!(!reads.actions.is_empty());
    assert!(
        analyse_in(&repos, "//:reads_ok", on_linux.clone())
            .await
            .unwrap()
            .incompatible
            .is_none()
    );
    // The `select()` that decides what is asked is read to get there.
    let unless = analyse_in(&repos, "//:unless", on_linux).await.unwrap();
    assert!(unless.incompatible.is_some());
    // A configuration with no platform asks for nothing.
    assert!(
        analyse(&repos, "//:c")
            .await
            .unwrap()
            .incompatible
            .is_none()
    );
}

/// A platform that names no value for a `constraint_setting` has its default
/// (buildfiji-491h): `target_compatible_with` of it holds, and a `select()` on
/// it matches, unless the platform names another value of that setting.
#[tokio::test(flavor = "multi_thread")]
async fn a_platform_that_names_no_value_for_a_setting_has_its_default() {
    let (_dir, repos) = workspace(&[(
        "BUILD.bazel",
        r#"
constraint_setting(name = "os")
constraint_value(name = "linux", constraint_setting = ":os")
constraint_setting(name = "libc", default_constraint_value = "glibc")
constraint_value(name = "glibc", constraint_setting = ":libc")
constraint_value(name = "musl", constraint_setting = ":libc")
genrule(name = "g", outs = ["g.txt"], cmd = "true", target_compatible_with = [":glibc"])
genrule(name = "m", outs = ["m.txt"], cmd = "true", target_compatible_with = [":musl"])
"#,
    )]);
    let label = |name: &str| Label {
        repo: String::new(),
        package: String::new(),
        name: name.into(),
    };
    let mut on_linux = config();
    on_linux.constraints.insert(label("linux"));
    let compatible = |c: &Configuration, t: &'static str| {
        let (repos, c) = (repos.clone(), c.clone());
        async move {
            analyse_in(&repos, t, c)
                .await
                .unwrap()
                .incompatible
                .is_none()
        }
    };
    assert!(compatible(&on_linux, "//:g").await);
    assert!(!compatible(&on_linux, "//:m").await);
    let mut on_musl = on_linux.clone();
    on_musl.constraints.insert(label("musl"));
    assert!(!compatible(&on_musl, "//:g").await);
    assert!(compatible(&on_musl, "//:m").await);
}

#[tokio::test(flavor = "multi_thread")]
async fn recording_execution_platforms_does_not_cycle_through_an_extra_platform() {
    let (_dir, repos) = workspace(&[
        ("MODULE.bazel", ""),
        (
            "BUILD.bazel",
            "constraint_setting(name = \"s\")\nconstraint_value(name = \"v\", constraint_setting = \":s\")\nplatform(name = \"px\", constraint_values = [\":v\"])\ngenrule(name = \"g\", outs = [\"o\"], cmd = \"true\", exec_compatible_with = [\":v\"])\n",
        ),
    ]);
    let engine = engine(Env {
        source: repos.clone(),
        rules: repos.clone(),
        main_repo_name: "_main".into(),
        registered_toolchains: Vec::new(),
        extra_toolchains: Vec::new(),
        registered_execution_platforms: Vec::new(),
        extra_execution_platforms: vec!["//:px".to_owned()],
        host_constraints: None,
        record_execution_platforms: true,
        toolchain_resolution_debug: None,
    });
    let g = engine
        .get(ConfiguredTargetKey {
            label: Label {
                repo: String::new(),
                package: String::new(),
                name: "g".into(),
            },
            configuration: Configuration::default(),
        })
        .await
        .unwrap();
    assert_eq!(
        g.execution_platform.as_ref().map(|l| l.name.as_str()),
        Some("px")
    );
}
