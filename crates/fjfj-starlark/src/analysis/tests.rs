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
