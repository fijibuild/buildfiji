//! Running rules against what `bazel aquery` and `bazel build` showed for the
//! same rules (Bazel 9.2.0).

use super::*;
use crate::WithoutSites;
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
        splits: BTreeMap::new(),
        outputs: Vec::new(),
        mappings: Arc::new(crate::test_support::probe_mappings()),
        toolchains: Vec::new(),
        exec_groups: Vec::new(),
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
        build_runfile_links: true,
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
        out.printed.without_sites(),
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
        build_runfile_links: true,
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
    assert_eq!(second.printed.without_sites(), ["[42, 1]"]);
}

/// `ctx.expand_make_variables`, with what Bazel 9.2.0 printed for the same
/// calls (additional substitutions first, then the variables of the
/// `toolchains` attribute, then the configuration's).
#[test]
fn a_rule_that_fails_keeps_what_it_printed_ahead_of_its_error() {
    let src = r#"
def _fails(ctx):
    print("before fail")
    fail("boom")
f = rule(implementation = _fails)
def _bad(ctx):
    print("before attr")
    ctx.expand_make_variables("cmd", "$(NOPE)", {})
    return []
b = rule(implementation = _bad)
"#;
    let module = module_in("", "", src).unwrap();
    for (rule, line, error) in [("f", "before fail", "boom"), ("b", "before attr", "NOPE")] {
        let err = run_rule(&request_in(module.clone(), rule, Vec::new(), Vec::new())).unwrap_err();
        let (printed, rest) = split_printed(&err);
        assert_eq!(printed.len(), 1, "{err}");
        assert!(printed[0].ends_with(line), "{printed:?}");
        assert!(rest.contains(error), "{rest}");
    }
}

#[test]
fn expand_make_variables_reads_substitutions_toolchain_variables_and_the_configuration() {
    let src = r#"
def _tc(ctx):
    return [platform_common.TemplateVariableInfo({"FOO": "from_toolchain", "TC": "tc", "TARGET_CPU": "mine", "DEFINE_ME": "tcdef", "SAME": "first"})]
tc = rule(implementation = _tc)
def _tc2(ctx):
    return [platform_common.TemplateVariableInfo({"SAME": "second", "ONLY2": "o2"})]
tc2 = rule(implementation = _tc2)

def _impl(ctx):
    for s in ctx.attr.cmds:
        print("OUT[" + s + "] =", ctx.expand_make_variables("cmd", s, {"FOO": "bar", "SPACE": "a b"}))
    print("var", ctx.var["TC"], ctx.var["FOO"], ctx.var["TARGET_CPU"], ctx.var["DEFINE_ME"], ctx.var["SAME"])
    return []
r = rule(implementation = _impl, attrs = {"cmds": attr.string_list()})
"#;
    let module = module_in("", "", src).unwrap();
    let provided = run_rule(&request_in(module.clone(), "tc", Vec::new(), Vec::new())).unwrap();
    assert_eq!(provided.providers.len(), 1);
    let provided2 = run_rule(&request_in(module.clone(), "tc2", Vec::new(), Vec::new())).unwrap();
    let dep_of = |name: &str, class: &str, providers| DepInfo {
        label: label("", name),
        rule_class: Some(class.into()),
        generated: false,
        files: Vec::new(),
        executable: None,
        runfiles: fjfj_graph::Runfiles::default(),
        providers,
        build_runfile_links: true,
    };
    let deps = vec![
        dep_of("tc", "tc", provided.providers),
        dep_of("tc2", "tc2", provided2.providers),
    ];
    let run = |cmds: &[&str]| {
        run_rule(&request_in(
            module.clone(),
            "r",
            vec![
                (
                    "cmds".into(),
                    AttrValue::StringList(cmds.iter().map(|c| c.to_string()).collect()),
                ),
                (
                    "toolchains".into(),
                    AttrValue::LabelList(vec![label("", "tc"), label("", "tc2")]),
                ),
            ],
            deps.clone(),
        ))
    };
    let ok = run(&[
        "plain",
        "$(FOO) $(FOO)",
        "$$FOO",
        "$(TC)",
        "$(SPACE)",
        "$(COMPILATION_MODE)",
        "$(TARGET_CPU)",
        "$(DEFINE_ME)",
        "$(SAME)",
        "$(ONLY2)",
        "$(BINDIR)x",
    ])
    .unwrap();
    assert_eq!(
        ok.printed.without_sites(),
        [
            "OUT[plain] = plain",
            "OUT[$(FOO) $(FOO)] = bar bar",
            "OUT[$$FOO] = $FOO",
            "OUT[$(TC)] = tc",
            "OUT[$(SPACE)] = a b",
            "OUT[$(COMPILATION_MODE)] = fastbuild",
            "OUT[$(TARGET_CPU)] = mine",
            "OUT[$(DEFINE_ME)] = tcdef",
            "OUT[$(SAME)] = first",
            "OUT[$(ONLY2)] = o2",
            "OUT[$(BINDIR)x] = bazel-out/k8-fastbuild/binx",
            "var tc from_toolchain mine tcdef second",
        ]
    );
    // An error is the attribute's: the call returns its argument, the rule
    // goes on, and fails at the end, each message once.
    let failed = run(&["$FOO", "$(NOPE)", "$(NOPE)", "$(location :t)"]).unwrap_err();
    assert_eq!(
        split_printed(&failed).1,
        format!(
            "{}in cmd attribute of r rule //:t: '$FOO' syntax is not supported; use '$(FOO)' instead for \"Make\" variables, or escape the '$' as '$$' if you intended this for the shell\nin cmd attribute of r rule //:t: $(NOPE) not defined\nin cmd attribute of r rule //:t: $(location) not defined",
            crate::ATTRIBUTE_ERRORS
        )
    );
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
    assert!(e.contains("param 'outputs' may not be empty"), "{e}");
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
        build_runfile_links: true,
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

/// Probed on Bazel 9.2.0: `ctx.resolve_command` expands `$(location)` only if
/// asked to, make variables only if given some, and runs the result with bash;
/// `ctx.expand_location` and the expansion in `resolve_command` report a
/// problem as an error of the rule, which goes on, and `ctx.resolve_tools` is
/// refused.
#[test]
fn resolve_command_and_the_expansion_functions_report_as_bazel_does() {
    let src = r#"
def _impl(ctx):
    _, argv, manifests = ctx.resolve_command(command = "echo $(X) $$", expand_locations = True, make_variables = {"X": "y"})
    print(argv, manifests)
    _, plain, _ = ctx.resolve_command(command = "echo $(X) $$", make_variables = {"X": "y"})
    print(plain)
    print(ctx.expand_location("$(location nope.txt) kept"))
    _, same, _ = ctx.resolve_command(command = "$(BAD)", attribute = "cmd", make_variables = {"X": "y"})
    print(same)
    return [DefaultInfo()]
r = rule(implementation = _impl)
def _refuse(ctx):
    ctx.resolve_tools(tools = [])
    return [DefaultInfo()]
refuse = rule(implementation = _refuse)
"#;
    let err = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap_err();
    let (printed, err) = super::run::split_printed(&err);
    let printed: Vec<&str> = printed
        .iter()
        .map(|line| line.split_once("bzl:").map_or("", |(_, rest)| rest))
        .map(|rest| rest.split_once(": ").map_or("", |(_, text)| text))
        .collect();
    assert_eq!(
        printed,
        [
            r#"["/bin/bash", "-c", "echo y $"] []"#,
            r#"["/bin/bash", "-c", "echo y $"]"#,
            "$(location nope.txt) kept",
            r#"["/bin/bash", "-c", "$(BAD)"]"#,
        ]
    );
    let events: Vec<&str> = err
        .strip_prefix(super::run::ATTRIBUTE_ERRORS)
        .expect("errors of the rule")
        .lines()
        .collect();
    assert_eq!(
        events,
        [
            "in r rule //:t: label '//:nope.txt' in $(location) expression is not a declared prerequisite of this rule",
            "in cmd attribute of r rule //:t: $(BAD) not defined",
        ]
    );
    let err = run_rule(&request(src, "refuse", Vec::new(), Vec::new())).unwrap_err();
    assert!(
        err.contains("Pass an executable or tools argument to ctx.actions.run or ctx.actions.run_shell instead of calling ctx.resolve_tools.\nUse --noincompatible_disallow_ctx_resolve_tools"),
        "{err}"
    );
}

/// Probed on Bazel 9.2.0: what each wrong call of `Args` says.
#[test]
fn the_wrong_calls_of_args_say_what_bazel_says() {
    let cases: [(&str, &str); 14] = [
        (
            "a.add_all('abc')",
            "Error in add_all: expected value of type 'sequence or depset' for values, got 'string'",
        ),
        (
            "a.add_all('--flag', 5)",
            "Error in add_all: in call to add_all(), parameter 'values' got value of type 'int', want 'sequence or depset'",
        ),
        (
            "a.add(['x'])",
            "Error in add: Args.add() doesn't accept vectorized arguments. Please use Args.add_all() or Args.add_joined() instead.",
        ),
        (
            "a.add(1, 2)",
            "Error in add: expected value of type 'string' for arg name, got 'int'",
        ),
        (
            "a.add('v', format = 'x')",
            "Error in add: Invalid value for parameter \"format\": Expected string with a single \"%s\"",
        ),
        (
            "a.add_all(['a'], format_each = '%s%s')",
            "Error in add_all: Invalid value for parameter \"format_each\": Expected string with a single \"%s\"",
        ),
        (
            "a.add_joined(['a'], join_with = ',', format_joined = 'x')",
            "Error in add_joined: Invalid value for parameter \"format_joined\": Expected string with a single \"%s\"",
        ),
        (
            "a.add_joined(['a'], join_with = 3)",
            "Error in add_joined: in call to add_joined(), parameter 'join_with' got value of type 'int', want 'string'",
        ),
        (
            "a.add_all(['a'], map_each = 'str')",
            "Error in add_all: in call to add_all(), parameter 'map_each' got value of type 'string', want 'callable or NoneType'",
        ),
        (
            "a.add_all(['a'], expand_directories = 'x')",
            "Error in add_all: in call to add_all(), parameter 'expand_directories' got value of type 'string', want 'bool'",
        ),
        (
            "a.use_param_file('nopercent')",
            "Error in use_param_file: Invalid value for parameter \"param_file_arg\": Expected string with a single \"%s\", got \"nopercent\"",
        ),
        (
            "a.use_param_file(3)",
            "Error in use_param_file: in call to use_param_file(), parameter 'param_file_arg' got value of type 'int', want 'string'",
        ),
        (
            "a.set_param_file_format('bogus')",
            "Error in set_param_file_format: Invalid value for parameter \"format\": Expected one of \"shell\", \"multiline\", \"flag_per_line\"",
        ),
        (
            "ctx.actions.declare_file('f').tree_relative_path",
            "Error: tree_relative_path not allowed for files that are not tree artifact files.",
        ),
    ];
    for (call, want) in cases {
        let src = format!(
            "def _impl(ctx):\n    a = ctx.actions.args()\n    {call}\n    return []\nr = rule(implementation = _impl)\n"
        );
        let err = run_rule(&request(&src, "r", Vec::new(), Vec::new())).unwrap_err();
        assert!(
            err.lines().any(|line| line == want),
            "{call}: wanted `{want}` in\n{err}"
        );
    }
}

/// Probed on Bazel 9.2.0: a value that is not a string, int, File or Label
/// goes on the command line as its Java string (`str()`, but `true` and
/// `false`); `map_each` may return only strings, None or a list of strings.
#[test]
fn args_stringify_other_values_and_map_each_returns_strings_only() {
    let src = r#"
def _m(x):
    return {"s": struct(), "n": 5, "l": ["a", 1], "f": 1.5, "ok": ["p", "q"], "none": None}[x]

def _impl(ctx):
    a = ctx.actions.args()
    if ctx.attr.k == "values":
        a.add_all([None, True, 1.5, {"a": 1}, (1, 2), [3, 4], struct(), depset(["q"])])
        a.add(struct(z = [1]))
        a.add("--flag", False)
    else:
        a.add_all([ctx.attr.k], map_each = _m)
    ctx.actions.run_shell(outputs = [ctx.actions.declare_file("o")], command = "true", arguments = [a])
    return []
r = rule(implementation = _impl, attrs = {"k": attr.string()})
"#;
    let module = module_in("", "", src).unwrap();
    let run = |k: &str| {
        run_rule(&request_in(
            module.clone(),
            "r",
            vec![("k".to_owned(), AttrValue::String(k.to_owned()))],
            Vec::new(),
        ))
    };
    let out = run("values").unwrap();
    let ActionKind::Spawn { argv, .. } = &out.actions[0].kind else {
        panic!()
    };
    assert_eq!(
        &argv[4..],
        [
            "None",
            "true",
            "1.5",
            r#"{"a": 1}"#,
            "(1, 2)",
            "[3, 4]",
            "struct()",
            r#"depset(["q"])"#,
            "struct(z = [1])",
            "--flag",
            "false"
        ]
    );
    let out = run("ok").unwrap();
    let ActionKind::Spawn { argv, .. } = &out.actions[0].kind else {
        panic!()
    };
    assert_eq!(&argv[4..], ["p", "q"]);
    let out = run("none").unwrap();
    let ActionKind::Spawn { argv, .. } = &out.actions[0].kind else {
        panic!()
    };
    assert_eq!(argv, &["/bin/bash", "-c", "true"]);
    for (k, found) in [
        ("s", "struct"),
        ("n", "int"),
        ("f", "float"),
        ("l", "list containing int"),
    ] {
        let err = run(k).unwrap_err();
        let want = format!(
            "Error in add_all: Expected map_each to return string, None, or list of strings, found {found}"
        );
        assert!(err.lines().any(|line| line == want), "{k}: {err}");
    }
}

/// Probed on Bazel 9.2.0: `ctx.info_file` and `ctx.version_file` are the files
/// the build writes its workspace status to, under `bazel-out` itself and made
/// by no target.
#[test]
fn the_status_files_are_files_of_the_build_that_no_target_made() {
    let src = r#"
def _impl(ctx):
    for f in [ctx.info_file, ctx.version_file]:
        print(f.path, f.short_path, f.root.path, f.basename, f.dirname, f.extension, f.is_source, f.is_directory, f.owner)
    out = ctx.actions.declare_file("o")
    ctx.actions.run_shell(outputs = [out], inputs = [ctx.info_file, ctx.version_file], command = "true")
    return [DefaultInfo(files = depset([out]))]
r = rule(implementation = _impl)
"#;
    let out = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap();
    assert_eq!(
        out.printed.without_sites(),
        [
            "bazel-out/stable-status.txt stable-status.txt bazel-out stable-status.txt bazel-out txt False False None",
            "bazel-out/volatile-status.txt volatile-status.txt bazel-out volatile-status.txt bazel-out txt False False None"
        ]
    );
    assert_eq!(
        paths(&out.actions[0].inputs),
        [
            "bazel-out/stable-status.txt",
            "bazel-out/volatile-status.txt"
        ]
    );
}

/// Probed on Bazel 9.2.0: a tree artifact among the values of `add_all` and
/// `add_joined` is a word for each file in it when the action runs, and the
/// tree itself in what `aquery` shows; `add` refuses one.
#[test]
fn a_tree_among_the_values_of_args_is_expanded_when_the_action_runs() {
    use fjfj_graph::command_line::{LazyArg, LazyCall, LazyItem};
    let src = r#"
def _impl(ctx):
    d = ctx.actions.declare_directory("d")
    ctx.actions.run_shell(outputs = [d], command = "mkdir -p " + d.path)
    kept = ctx.actions.args()
    kept.add_all("--keep", [d], expand_directories = False)
    a = ctx.actions.args()
    a.add_all("--name", [d, "x"], before_each = "-I", format_each = "<%s>", terminate_with = "END")
    a.add_joined("--j", [d], join_with = ",", format_joined = "{%s}")
    a.add_all("--plain", ["p"])
    ctx.actions.run_shell(outputs = [ctx.actions.declare_file("o")], inputs = [d], command = "true", arguments = [kept, a])
    p = ctx.actions.args()
    p.add_all([d])
    p.use_param_file("@%s", use_always = True)
    ctx.actions.run_shell(outputs = [ctx.actions.declare_file("o2")], inputs = [d], command = "true", arguments = [p])
    return []
r = rule(implementation = _impl)
"#;
    let out = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap();
    let spawns: Vec<&fjfj_graph::Action> = out
        .actions
        .iter()
        .filter(|a| matches!(a.kind, ActionKind::Spawn { .. }))
        .collect();
    // The first makes the tree.
    let ActionKind::Spawn { argv, lazy, .. } = &spawns[1].kind else {
        panic!()
    };
    let dir = format!("{BIN}/d");
    // The tree is one word where Bazel's `aquery` has it.
    let want: Vec<String> = [
        "/bin/bash",
        "-c",
        "true",
        "",
        "--keep",
        &dir,
        "--name",
        "-I",
        &format!("<{dir}>"),
        "-I",
        "<x>",
        "END",
        "--j",
        &format!("{{{dir}}}"),
        "--plain",
        "p",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    assert_eq!(argv, &want);
    let tree = |format: Option<&str>| LazyItem::Tree {
        dir: dir.clone(),
        format_each: format.map(str::to_owned),
    };
    assert_eq!(
        lazy,
        &[
            LazyArg {
                at: 6,
                len: 6,
                call: LazyCall::AddAll {
                    name: Some("--name".into()),
                    items: vec![tree(Some("<%s>")), LazyItem::Text("<x>".into())],
                    before_each: Some("-I".into()),
                    omit_if_empty: true,
                    uniquify: false,
                    terminate_with: Some("END".into()),
                }
            },
            LazyArg {
                at: 12,
                len: 2,
                call: LazyCall::AddJoined {
                    name: Some("--j".into()),
                    items: vec![tree(None)],
                    join_with: ",".into(),
                    format_joined: Some("{%s}".into()),
                    omit_if_empty: true,
                    uniquify: false,
                }
            }
        ]
    );
    // What it comes to once the tree holds `b` and `x/a`.
    let files = |dir: &str| vec![format!("{dir}/b"), format!("{dir}/x/a")];
    assert_eq!(
        fjfj_graph::command_line::expand_words(argv, lazy, &files)[4..],
        [
            "--keep".to_owned(),
            dir.clone(),
            "--name".into(),
            "-I".into(),
            format!("<{dir}/b>"),
            "-I".into(),
            format!("<{dir}/x/a>"),
            "-I".into(),
            "<x>".into(),
            "END".into(),
            "--j".into(),
            format!("{{{dir}/b,{dir}/x/a}}"),
            "--plain".into(),
            "p".into()
        ]
    );
    // The parameter file reads the tree too, and is made when it exists.
    let params = out
        .actions
        .iter()
        .find(|a| matches!(a.kind, ActionKind::ParamFile { .. }))
        .expect("a parameter file that waits for the tree");
    assert_eq!(paths(&params.inputs), [dir]);
}

/// A `map_each` over a tree is called when the action runs in Bazel, which
/// analysis cannot do (buildfiji-fcw9): it is refused unless it is to see the
/// tree itself.
#[test]
fn a_map_each_over_a_tree_that_expands_is_refused_until_it_can_be_made_to_work() {
    let src = |call: &str| {
        format!(
            "def _f(x):\n    return x.basename\n\ndef _impl(ctx):\n    d = ctx.actions.declare_directory('d')\n    ctx.actions.run_shell(outputs = [d], command = 'true')\n    a = ctx.actions.args()\n    {call}\n    ctx.actions.run_shell(outputs = [ctx.actions.declare_file('o')], inputs = [d], command = 'true', arguments = [a])\n    return []\nr = rule(implementation = _impl)\n"
        )
    };
    let err = run_rule(&request(
        &src("a.add_all([d], map_each = _f)"),
        "r",
        Vec::new(),
        Vec::new(),
    ))
    .unwrap_err();
    assert!(err.contains("buildfiji-fcw9"), "{err}");
    let out = run_rule(&request(
        &src("a.add_all([d], map_each = _f, expand_directories = False)"),
        "r",
        Vec::new(),
        Vec::new(),
    ))
    .unwrap();
    let ActionKind::Spawn { argv, lazy, .. } = &out.actions.last().unwrap().kind else {
        panic!()
    };
    assert_eq!(argv.last().map(String::as_str), Some("d"));
    assert!(lazy.is_empty());
}

/// `add` does not take a tree, which may be many values.
#[test]
fn add_refuses_a_tree_artifact() {
    let src = "def _impl(ctx):\n    d = ctx.actions.declare_directory('d')\n    ctx.actions.args().add('--x', d)\n    return []\nr = rule(implementation = _impl)\n";
    let err = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap_err();
    assert!(
        err.lines().any(|l| l
            == "Error in add: Cannot add directories to Args#add since they may expand to multiple values. Either use Args#add_all (if you want expansion) or args.add(directory.path) (if you do not)."),
        "{err}"
    );
}

/// Probed on Bazel 9.2.0: the functions of `ctx` that read a string the way a
/// label or a shell does, and `print` of a label.
#[test]
fn ctx_reads_labels_command_lines_and_placeholders_as_bazel_does() {
    let src = r#"
def _impl(ctx):
    print(ctx.package_relative_label("foo"), ctx.package_relative_label(":bar"), ctx.package_relative_label("//x:y"), ctx.package_relative_label("@@r//a:b"), ctx.label)
    print(str(ctx.package_relative_label("foo")), repr(ctx.package_relative_label("foo")))
    print(ctx.tokenize("a 'b c' \"d e\" f\\ g"))
    print(ctx.check_placeholders("a %{x} b", ["x"]), ctx.check_placeholders("a %{y} b", ["x"]), ctx.check_placeholders("a %{x", ["x"]))
    if ctx.attr.fail == "label":
        ctx.package_relative_label("")
    elif ctx.attr.fail == "label_type":
        ctx.package_relative_label(1)
    elif ctx.attr.fail == "quote":
        ctx.tokenize("'abc")
    elif ctx.attr.fail == "backslash":
        ctx.tokenize("a\\")
    return []
r = rule(implementation = _impl, attrs = {"fail": attr.string()})
"#;
    let module = module_in("", "", src).unwrap();
    let run = |fail: &str| {
        run_rule(&request_in(
            module.clone(),
            "r",
            vec![("fail".to_owned(), AttrValue::String(fail.to_owned()))],
            Vec::new(),
        ))
    };
    assert_eq!(
        run("").unwrap().printed.without_sites(),
        [
            "//:foo //:bar //x:y @@r//a:b //:t",
            r#"@@//:foo Label("//:foo")"#,
            r#"["a", "b c", "d e", "f g"]"#,
            "True False True"
        ]
    );
    for (fail, want) in [
        (
            "label",
            "Error in package_relative_label: invalid label in ctx.package_relative_label: invalid target name '': empty target name",
        ),
        (
            "label_type",
            "Error in package_relative_label: in call to package_relative_label(), parameter 'input' got value of type 'int', want 'string or Label'",
        ),
        (
            "quote",
            "Error in tokenize: unterminated quotation while tokenizing ''abc'",
        ),
        (
            "backslash",
            "Error in tokenize: backslash at end of string while tokenizing 'a\\'",
        ),
    ] {
        let err = run(fail).unwrap_err();
        assert!(err.lines().any(|l| l == want), "{fail}: {err}");
    }
}

/// Probed on Bazel 9.2.0 (the members printed by both tools): what `ctx`,
/// `ctx.attr`, `ctx.configuration` and a `Target` have.
#[test]
fn ctx_and_target_members_are_bazels() {
    let src = r#"
def _impl(ctx):
    print(sorted(dir(ctx.attr)))
    print(ctx.attr.name, ctx.attr.testonly, ctx.attr.deprecation, ctx.attr.package_metadata)
    print(sorted(dir(ctx.files)))
    print(sorted(dir(ctx.fragments)))
    c = ctx.configuration
    print(c.bin_dir.path, c.genfiles_dir.path, c.disabled_features(), len(c.short_id))
    print(ctx.expand_location("$(location //pkg:a.txt)"))
    print(ctx.file.src.owner, ctx.files.srcs[0].owner)
    d = ctx.attr.dep
    print(OutputGroupInfo in d, d[OutputGroupInfo], d.output_groups, d.actions)
    print(InstrumentedFilesInfo in d, d.files_to_run.executable)
    print(str(ctx), str(ctx.actions))
    if ctx.attr.fail == "rule":
        ctx.rule
    elif ctx.attr.fail == "aspect_ids":
        ctx.aspect_ids
    elif ctx.attr.fail == "constraint":
        ctx.target_platform_has_constraint(Label("//:x"))
    elif ctx.attr.fail == "stamp":
        c.stamp_binaries()
    return []
r = rule(
    implementation = _impl,
    fragments = ["cpp"],
    attrs = {
        "srcs": attr.label_list(allow_files = True),
        "src": attr.label(allow_single_file = True),
        "dep": attr.label(),
        "fail": attr.string(),
    },
)
"#;
    let source = |name: &str| DepInfo {
        label: label("pkg", name),
        rule_class: None,
        generated: false,
        files: vec![Artifact::source("", "pkg", name)],
        executable: None,
        runfiles: fjfj_graph::Runfiles::default(),
        providers: Vec::new(),
        build_runfile_links: true,
    };
    let group = DepInfo {
        label: label("pkg", "g"),
        rule_class: Some("filegroup".to_owned()),
        generated: false,
        files: vec![Artifact::source("", "pkg", "a.txt")],
        executable: None,
        runfiles: fjfj_graph::Runfiles::default(),
        providers: Vec::new(),
        build_runfile_links: true,
    };
    let module = module_in("rules_cc+", "", src).unwrap();
    let run = |fail: &str, main: bool| {
        let module = if main {
            module_in("", "", src).unwrap()
        } else {
            module.clone()
        };
        run_rule(&request_in(
            module,
            "r",
            vec![
                (
                    "srcs".to_owned(),
                    AttrValue::LabelList(vec![label("pkg", "a.txt")]),
                ),
                ("src".to_owned(), AttrValue::Label(label("pkg", "a.txt"))),
                ("dep".to_owned(), AttrValue::Label(label("pkg", "g"))),
                ("fail".to_owned(), AttrValue::String(fail.to_owned())),
            ],
            vec![source("a.txt"), group.clone()],
        ))
    };
    let printed = run("", false).unwrap().printed.without_sites();
    let names = |text: &str| -> Vec<String> {
        text.trim_matches(['[', ']'])
            .split(", ")
            .map(|name| name.trim_matches('"').to_owned())
            .collect()
    };
    assert_eq!(
        names(&printed[0]),
        [
            "_action_listener",
            "_config_dependencies",
            "compatible_with",
            "dep",
            "deprecation",
            "exec_compatible_with",
            "exec_properties",
            "expect_failure",
            "fail",
            "features",
            "generator_function",
            "generator_location",
            "generator_name",
            "name",
            "package_metadata",
            "restricted_to",
            "src",
            "srcs",
            "tags",
            "target_compatible_with",
            "testonly",
            "toolchains",
            "transitive_configs",
            "visibility"
        ]
    );
    assert_eq!(printed[1], "t False None []");
    assert_eq!(
        names(&printed[2]),
        [
            "_action_listener",
            "_config_dependencies",
            "compatible_with",
            "dep",
            "exec_compatible_with",
            "package_metadata",
            "restricted_to",
            "src",
            "srcs",
            "target_compatible_with",
            "toolchains"
        ]
    );
    assert_eq!(
        names(&printed[3]),
        [
            "android",
            "apple",
            "bazel_android",
            "coverage",
            "cpp",
            "j2objc",
            "java",
            "objc",
            "platform",
            "proto"
        ]
    );
    assert!(printed[4].starts_with(&format!("{BIN} {BIN} [] ")));
    // The rule reads `a.txt` through `srcs`.
    assert_eq!(printed[5], "pkg/a.txt");
    assert_eq!(printed[6], "//pkg:a.txt //pkg:a.txt");
    assert_eq!(
        printed[7],
        "True struct(_hidden_top_level_INTERNAL_ = depset([])) struct(_hidden_top_level_INTERNAL_ = depset([])) []"
    );
    assert_eq!(printed[8], "True <source file pkg/a.txt>");
    assert_eq!(
        printed[9],
        "<rule context for //:t> actions for<rule context for //:t>"
    );
    for (fail, want) in [
        (
            "rule",
            "Error: 'rule' is only available in aspect implementations",
        ),
        (
            "aspect_ids",
            "Error: 'aspect_ids' is only available in aspect implementations",
        ),
        (
            "constraint",
            "Error in target_platform_has_constraint: in call to target_platform_has_constraint(), parameter 'constraintValue' got value of type 'Label', want 'ConstraintValueInfo'",
        ),
        (
            "stamp",
            "Error in stamp_binaries: file '//:t.bzl' cannot use private API",
        ),
    ] {
        let err = run(fail, true).unwrap_err();
        assert!(err.lines().any(|l| l == want), "{fail}: {err}");
    }
}

/// Probed on Bazel 9.2.0: a `map_each` that is not a top-level def is
/// refused, at the name of a def and at the `lambda` of a lambda, unless
/// `allow_closure`; a builtin function is accepted.
#[test]
fn map_each_must_be_a_top_level_def_unless_closures_are_allowed() {
    let src = |body: &str| {
        format!(
            "def _top(x):\n    return x\n\ndef _impl(ctx):\n    a = ctx.actions.args()\n    def nested(x):\n        return x\n    {body}\n    return []\nr = rule(implementation = _impl)\n"
        )
    };
    let run = |body: &str| run_rule(&request(&src(body), "r", Vec::new(), Vec::new()));
    for ok in [
        "a.add_all(['a'], map_each = _top)",
        "a.add_all(['a'], map_each = str)",
        "a.add_all(['a'], map_each = nested, allow_closure = True)",
        "a.add_joined(['a'], join_with = ',', map_each = lambda x: x, allow_closure = True)",
    ] {
        run(ok).unwrap_or_else(|e| panic!("{ok}: {e}"));
    }
    for (call, function, at) in [
        ("a.add_all(['a'], map_each = nested)", "add_all", "6:9"),
        (
            "a.add_all(['a'], map_each = lambda x: x)",
            "add_all",
            "8:33",
        ),
        (
            "a.add_joined(['a'], join_with = ',', map_each = lambda x: x)",
            "add_joined",
            "8:53",
        ),
    ] {
        let err = run(call).unwrap_err();
        let want = format!(
            "Error in {function}: to avoid unintended retention of analysis data structures, the map_each function (declared at "
        );
        let line = err
            .lines()
            .find(|line| line.starts_with(&want))
            .unwrap_or_else(|| panic!("{call}: no refusal in\n{err}"));
        assert!(
            line.ends_with(&format!(
                ":{at}) must be declared by a top-level def statement"
            )),
            "{call}: {line}"
        );
    }
}

/// An `Args` that asked for a param file gets one when the command line is
/// longer than 31744, each word counting its separator and the executable
/// counting too. Probed on Bazel 9.2.0 at both sides of the limit.
#[test]
fn an_args_that_may_use_a_param_file_does_when_the_command_line_is_too_long() {
    let src = r#"
def _impl(ctx):
    o = ctx.actions.declare_file("o")
    a = ctx.actions.args()
    a.add_all(["x" * int(ctx.attr.w)] * int(ctx.attr.n))
    a.use_param_file("@%s")
    ctx.actions.run(outputs = [o], executable = "/bin/true", arguments = [a])
    return [DefaultInfo(files = depset([o]))]
r = rule(implementation = _impl, attrs = {"w": attr.string(), "n": attr.string()})
"#;
    let module = module_in("", "", src).unwrap();
    let uses_param_file = |w: &str, n: &str| {
        let attrs = vec![
            ("w".to_owned(), AttrValue::String(w.to_owned())),
            ("n".to_owned(), AttrValue::String(n.to_owned())),
        ];
        let out = run_rule(&request_in(module.clone(), "r", attrs, Vec::new())).unwrap();
        out.actions.len() == 2
    };
    // "/bin/true " is 10 and each word costs its length and one.
    assert!(!uses_param_file("2", "10575"));
    assert!(uses_param_file("10", "2885"));
    assert!(!uses_param_file("100", "314"));
    assert!(uses_param_file("100", "315"));
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
    let req = request_in(
        module_in("rules_cc+", "", src).unwrap(),
        "r",
        Vec::new(),
        Vec::new(),
    );
    assert_eq!(
        run_rule(&req).unwrap().printed.without_sites(),
        ["fastbuild None macos 10.12"]
    );
    let defaults = computed_defaults(
        &req.module,
        "r",
        &[],
        &crate::test_support::probe_mappings(),
        "",
        &Default::default(),
    )
    .unwrap();
    assert_eq!(
        defaults,
        [(
            "_xcode".to_owned(),
            AttrValue::Label(Label {
                repo: "bazel_tools".into(),
                package: "tools/cpp".into(),
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
    print(len(c.short_id), c.is_tool_configuration(), c.host_path_separator, c.coverage_enabled)
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
    assert_eq!(
        out.printed.without_sites(),
        ["7 False : False", "None [] None"]
    );
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
    let mut req = request_in(
        module_in("rules_cc+", "", src).unwrap(),
        "r",
        Vec::new(),
        Vec::new(),
    );
    req.configuration
        .options
        .insert("copt".into(), "-O2 -g".into());
    assert_eq!(
        run_rule(&req).unwrap().printed.without_sites(),
        [
            "[\"-O2\", \"-g\"] [] [] [] DEFAULT False",
            "True True False fastbuild False"
        ]
    );
}

/// Probed on Bazel 9.2.0 over a matrix of repos and packages (the same for
/// every member): the members of the cpp fragment behind its private-API
/// allowlist fail for a file of the main repo or of a repo that is not on the
/// list (rules_python, rules_go, ...), which rules_cc, rules_apple,
/// rules_android, protobuf and rules_shell are, all of them; rules_java is
/// under `java`, rules_rust under `rust/private`; three members name the
/// feature.
#[test]
fn private_members_of_the_cpp_fragment_fail_for_files_that_are_not_allowlisted() {
    let src = r#"
def _impl(ctx):
    cpp = ctx.fragments.cpp
    print(cpp.copts, cpp.dynamic_mode())
    print(getattr(cpp, ctx.attr.member)())
    return []
r = rule(implementation = _impl, fragments = ["cpp"], attrs = {"member": attr.string()})
"#;
    let run = |repo: &str, package: &str, member: &str| {
        let module = module_in(repo, package, src).unwrap();
        run_rule(&request_in(
            module,
            "r",
            vec![("member".to_owned(), AttrValue::String(member.to_owned()))],
            Vec::new(),
        ))
    };
    for (repo, package) in [
        ("rules_cc+", ""),
        ("rules_cc+", "cc/private"),
        ("rules_apple+", ""),
        ("rules_android+", "src/common"),
        ("protobuf+", ""),
        ("rules_shell+", ""),
        ("rules_java+", "java"),
        ("rules_java+", "java/common"),
        ("rules_rust+", "rust/private"),
        ("rules_rust+", "rust/private/rules"),
    ] {
        run(repo, package, "fission_active_for_current_compilation_mode")
            .unwrap_or_else(|e| panic!("{repo}//{package}: {e}"));
    }
    for (repo, package, member, file, feature) in [
        ("", "", "compilation_mode", "//:t.bzl", ""),
        ("rules_java+", "", "grte_top", "@@rules_java+//:t.bzl", ""),
        (
            "rules_java+",
            "javatests",
            "grte_top",
            "@@rules_java+//javatests:t.bzl",
            "",
        ),
        ("rules_rust+", "", "grte_top", "@@rules_rust+//:t.bzl", ""),
        (
            "rules_rust+",
            "rust",
            "grte_top",
            "@@rules_rust+//rust:t.bzl",
            "",
        ),
        (
            "rules_python+",
            "python/private",
            "save_temps",
            "@@rules_python+//python/private:t.bzl",
            "",
        ),
        (
            "",
            "",
            "force_pic",
            "//:t.bzl",
            " (feature 'force_pic' in CppConfiguration)",
        ),
        (
            "",
            "",
            "fdo_instrument",
            "//:t.bzl",
            " (feature 'fdo_instrument' in CppConfiguration)",
        ),
        (
            "",
            "",
            "generate_llvm_lcov",
            "//:t.bzl",
            " (feature 'generate_llvm_lcov' in CppConfiguration)",
        ),
    ] {
        let err = run(repo, package, member).unwrap_err();
        let want = format!("Error in {member}: file '{file}' cannot use private API{feature}");
        assert!(err.lines().any(|line| line == want), "{member}: {err}");
    }
}

/// `ctx.fragments.apple` has what Bazel 9.2.0's has: `apple_cpus` as a struct
/// of the flags' default cpus and `apple_platform_type`.
#[test]
fn the_apple_fragment_has_the_cpus_and_the_platform_type() {
    let src = r#"
def _impl(ctx):
    apple = ctx.fragments.apple
    print(apple.apple_platform_type, apple.apple_cpus)
    return []
r = rule(implementation = _impl, fragments = ["apple"])
"#;
    assert_eq!(
        run_rule(&request(src, "r", Vec::new(), Vec::new()))
            .unwrap()
            .printed
            .without_sites(),
        [
            r#"macos struct(apple_split_cpu = "", catalyst_cpus = ("x86_64",), ios_multi_cpus = ("x86_64",), macos_cpus = ("x86_64",), tvos_cpus = ("x86_64",), visionos_cpus = ("sim_arm64",), watchos_cpus = ("x86_64",))"#
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
        out.printed.without_sites(),
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
        build_runfile_links: true,
    };
    let out = run_rule(&request(src, "r", Vec::new(), vec![helper])).unwrap();
    assert_eq!(out.printed.without_sites(), ["t 7 h None", "ok"]);
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
        build_runfile_links: true,
    };
    let attrs = vec![("src".to_owned(), AttrValue::Label(label("", "a.cc")))];
    let out = run_rule(&request(src, "r", attrs, vec![src_file])).unwrap();
    assert_eq!(
        out.printed.without_sites(),
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

/// Probed on Bazel 9.2.0: every file a rule declares has an action that makes
/// it, whether the rule returns it or not, and the error lists those that do
/// not by path in the output directory, in order, after the place of the
/// implementation.
#[test]
fn a_declared_file_that_no_action_makes_fails_the_rule() {
    let src = r#"
def _impl(ctx):
    a = ctx.actions.declare_file("a")
    ctx.actions.declare_file("sub/b")
    ctx.actions.declare_directory("z")
    ctx.actions.write(ctx.actions.declare_file("made"), "x")
    ctx.actions.run_shell(outputs = [ctx.actions.declare_file("o")], inputs = [a], command = "true")
    return []
r = rule(implementation = _impl)
"#;
    let err = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap_err();
    assert_eq!(
        err,
        "\n@@//:t.bzl:2:5: The following files have no generating action:\na\nsub/b\nz"
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
    ctx.actions.write(plain, "")
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
        build_runfile_links: true,
    };
    let attrs = vec![("tool".to_owned(), AttrValue::Label(label("", "tool")))];
    let out = run_rule(&request(src, "r", attrs, vec![tool])).unwrap();
    assert_eq!(
        out.printed.without_sites(),
        [format!("True False {BIN}/out"), "tool MANIFEST".to_owned()]
    );
    let [_write, run] = &out.actions[..] else {
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
            "bazel-out/k8-opt-exec/bin/tool.runfiles".to_owned(),
        ]
    );
}

/// A file that is a `ctx.executable` of the rule runs with its target's
/// runfiles, as a `files_to_run` does; any other file is just a file.
#[test]
fn an_executable_attribute_file_comes_with_its_runfiles_tree_to_an_action() {
    let src = r#"
def _impl(ctx):
    out = ctx.actions.declare_file("out")
    ctx.actions.run(executable = ctx.executable.tool, outputs = [out], arguments = ["x"])
    return []
r = rule(implementation = _impl, attrs = {"tool": attr.label(executable = True, cfg = "exec")})
"#;
    let exe = Artifact::derived("bazel-out/k8-opt-exec/bin", "", "", "tool");
    let tool = DepInfo {
        label: label("", "tool"),
        rule_class: Some("sh_binary".to_owned()),
        generated: false,
        files: vec![exe.clone()],
        executable: Some(exe),
        runfiles: Default::default(),
        providers: Vec::new(),
        build_runfile_links: true,
    };
    let attrs = vec![("tool".to_owned(), AttrValue::Label(label("", "tool")))];
    let out = run_rule(&request(src, "r", attrs, vec![tool])).unwrap();
    let [run] = &out.actions[..] else {
        panic!("{:?}", out.actions)
    };
    let mut inputs: Vec<String> = run.inputs.iter().map(|i| i.exec_path()).collect();
    inputs.sort();
    assert_eq!(
        inputs,
        [
            "bazel-out/k8-opt-exec/bin/tool".to_owned(),
            "bazel-out/k8-opt-exec/bin/tool.runfiles".to_owned(),
        ]
    );
}

/// The proto fragment's late-bound defaults are the labels `bazel query`
/// showed as rule inputs; a field Bazel leaves unset stays unset.
#[test]
fn the_proto_fragment_late_bound_defaults_are_the_bazel_tools_targets() {
    let src = r#"
def _impl(ctx):
    return []
r = rule(implementation = _impl, attrs = {
    "_protoc": attr.label(default = configuration_field("proto", "proto_compiler")),
    "_cc": attr.label(default = configuration_field("proto", "proto_toolchain_for_cc")),
    "_java": attr.label(default = configuration_field("proto", "proto_toolchain_for_java")),
    "_lite": attr.label(default = configuration_field("proto", "proto_toolchain_for_java_lite")),
    "_malloc": attr.label(default = configuration_field("cpp", "custom_malloc")),
})
"#;
    let req = request(src, "r", Vec::new(), Vec::new());
    let defaults = computed_defaults(
        &req.module,
        "r",
        &[],
        &crate::test_support::probe_mappings(),
        "",
        &Default::default(),
    )
    .unwrap();
    let shown: Vec<(String, String)> = defaults
        .into_iter()
        .map(|(n, v)| match v {
            AttrValue::Label(l) => (n, format!("@{}//{}:{}", l.repo, l.package, l.name)),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        shown,
        [
            ("_protoc", "@bazel_tools//tools/proto:protoc"),
            ("_cc", "@bazel_tools//tools/proto:cc_toolchain"),
            ("_java", "@bazel_tools//tools/proto:java_toolchain"),
            ("_lite", "@bazel_tools//tools/proto:javalite_toolchain"),
        ]
        .map(|(a, b)| (a.to_owned(), b.to_owned()))
    );
}

/// `--custom_malloc` and the like replace the default of the field they set,
/// as `bazel cquery deps(...)` showed; a field with no flag keeps its own.
#[test]
fn late_bound_defaults_follow_their_options() {
    let src = r#"
def _impl(ctx):
    return []
r = rule(implementation = _impl, attrs = {
    "_protoc": attr.label(default = configuration_field("proto", "proto_compiler")),
    "_malloc": attr.label(default = configuration_field("cpp", "custom_malloc")),
    "_lite": attr.label(default = configuration_field("proto", "proto_toolchain_for_java_lite")),
    "_launcher": attr.label(default = configuration_field("java", "launcher")),
    "_libc": attr.label(default = configuration_field("cpp", "libc_top")),
    "_fdo": attr.label(default = configuration_field("cpp", "fdo_optimize")),
})
"#;
    let req = request(src, "r", Vec::new(), Vec::new());
    let options = std::collections::BTreeMap::from([
        ("custom_malloc".to_owned(), "//:m".to_owned()),
        ("proto_compiler".to_owned(), "//tools:mine".to_owned()),
        ("java_launcher".to_owned(), "//j:l".to_owned()),
        ("grte_top".to_owned(), "//g:x".to_owned()),
        ("fdo_optimize".to_owned(), "//:m".to_owned()),
    ]);
    let defaults = computed_defaults(
        &req.module,
        "r",
        &[],
        &crate::test_support::probe_mappings(),
        "",
        &options,
    )
    .unwrap();
    let shown: Vec<(String, String)> = defaults
        .into_iter()
        .map(|(n, v)| match v {
            AttrValue::Label(l) => (n, format!("@{}//{}:{}", l.repo, l.package, l.name)),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        shown,
        [
            ("_protoc", "@//tools:mine"),
            ("_malloc", "@//:m"),
            ("_lite", "@bazel_tools//tools/proto:javalite_toolchain"),
            ("_launcher", "@//j:l"),
            ("_libc", "@//g:everything"),
            ("_fdo", "@//:m"),
        ]
        .map(|(a, b)| (a.to_owned(), b.to_owned()))
    );
}

#[test]
fn a_parameter_file_quotes_what_bazels_shell_escaper_quotes() {
    use fjfj_graph::command_line::{ParamFormat, param_file_contents};
    // Bazel's safe set is [A-Za-z0-9] and "@%-_+:,./"; `=` is outside it.
    let items = ["a=b", "--flag=v", "plain-x_1.2/y:z,@%+", "", "it's"].map(String::from);
    assert_eq!(
        param_file_contents(&items, ParamFormat::Shell),
        "'a=b'\n'--flag=v'\nplain-x_1.2/y:z,@%+\n''\n'it'\\''s'\n"
    );
}

const FLAG_LINES: &str = r#"
def _impl(ctx):
    o = ctx.actions.declare_file("o")
    a = ctx.actions.args()
    a.add("--k=v w")
    a.add("positional")
    a.add("--x", "y")
    a.add_all("--t", ["a", "b c"])
    a.add_all(["p", "q"])
    a.add_all("-single", ["a", "b"])
    a.add_all("--t0", [], omit_if_empty = False)
    a.add_joined("--j", ["a", "b"], join_with = ",")
    a.add_all("--eq=", ["a", "b"])
    a.use_param_file("@%s", use_always = True)
    a.set_param_file_format("flag_per_line")
    ctx.actions.run_shell(outputs = [o], command = "true", arguments = [a])
    return []

r = rule(implementation = _impl)
"#;

#[test]
fn flag_per_line_writes_a_line_for_each_flag_call() {
    let module = module_in("", "", FLAG_LINES).unwrap();
    let out = run_rule(&request_in(module, "r", Vec::new(), Vec::new())).unwrap();
    let ActionKind::WriteFile { contents, .. } = &out.actions[0].kind else {
        panic!("{:?}", out.actions)
    };
    assert_eq!(
        String::from_utf8_lossy(contents),
        "--k=v w\n--x=y\n--t=a b c\n--t0\n--j=a,b\n--eq==a b\n"
    );
}

/// Probed on Bazel 9.2.0.
#[test]
fn declare_file_do_nothing_and_private_api_as_bazel_has_them() {
    let cases: [(&str, &str); 5] = [
        (
            "ctx.actions.declare_file('/abs')",
            "Error in declare_file: the output artifact '/abs' is not under package directory '' for target '//:t'",
        ),
        (
            "ctx.actions.do_nothing()",
            "Error in do_nothing: do_nothing() missing 1 required named argument: mnemonic",
        ),
        (
            "ctx.actions.do_nothing(mnemonic = 1)",
            "Error in do_nothing: in call to do_nothing(), parameter 'mnemonic' got value of type 'int', want 'string'",
        ),
        (
            "ctx.actions.args().add_joined()",
            "Error in add_joined: add_joined() missing 1 required positional argument: arg_name_or_values",
        ),
        (
            "ctx.actions.declare_shareable_artifact('x')",
            "cannot use private API",
        ),
    ];
    for (call, want) in cases {
        let src = format!(
            "def _impl(ctx):\n    {call}\n    return []\nr = rule(implementation = _impl)\n"
        );
        let err = run_rule(&request(&src, "r", Vec::new(), Vec::new())).unwrap_err();
        assert!(err.contains(want), "{call}: wanted `{want}` in\n{err}");
    }
    // `a/../b` is `b`, and `do_nothing` and `created_actions` are fine.
    let src = r#"
def _impl(ctx):
    f = ctx.actions.declare_file("a/../b")
    ctx.actions.write(f, "x")
    ctx.actions.do_nothing(mnemonic = "M", inputs = [f])
    if ctx.created_actions() != None:
        fail("created_actions")
    return [DefaultInfo(files = depset([f]))]
r = rule(implementation = _impl)
"#;
    let out = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap();
    assert_eq!(out.actions[0].outputs[0].path, "b");
    assert_eq!(out.actions.len(), 2);
}

/// Probed on Bazel 9.2.0: how `runfiles`, `ctx.outputs` and `ctx.exec_groups`
/// print and what type they have.
#[test]
fn runfiles_outputs_and_exec_groups_print_as_bazel_prints_them() {
    let src = r#"
def _impl(ctx):
    f = ctx.actions.declare_file("f")
    ctx.actions.write(f, "x")
    ctx.actions.write(ctx.outputs.o, "x")
    rf = ctx.runfiles(files = [f], symlinks = {"a/b": f}, root_symlinks = {"c": f})
    fail("|".join([
        str(ctx.runfiles()),
        str(rf),
        type(rf.symlinks.to_list()[0]),
        rf.symlinks.to_list()[0].path,
        str(ctx.outputs),
        type(ctx.outputs),
        str(dir(ctx.outputs)),
        str(ctx.exec_groups),
    ]))
r = rule(implementation = _impl, outputs = {"o": "%{name}.txt"})
"#;
    let mut req = request(src, "r", Vec::new(), Vec::new());
    req.outputs = vec![("o".to_owned(), "t.txt".to_owned())];
    let err = run_rule(&req).unwrap_err();
    let want = [
        r#"Runfiles(empty_files = depset([]), files = depset([], order = "postorder"), root_symlinks = depset([]), symlinks = depset([]))"#,
        r#"Runfiles(empty_files = depset([]), files = depset([<generated file f>], order = "postorder"), root_symlinks = depset([SymlinkEntry(path = "c", target_file = <generated file f>)]), symlinks = depset([SymlinkEntry(path = "a/b", target_file = <generated file f>)]))"#,
        "SymlinkEntry",
        "a/b",
        "ctx.outputs(o = <generated file t.txt>)",
        "Outputs",
        r#"["o"]"#,
        "<ctx.exec_groups: >",
    ]
    .join("|");
    assert!(err.contains(&want), "wanted `{want}` in\n{err}");
}

/// Probed on Bazel 9.2.0: a `DefaultInfo` has four members, None when not
/// given, and a dependency's shows its runfiles and files to run as Java
/// objects; `files_to_run` is a `FilesToRunProvider`.
#[test]
fn default_info_prints_and_lists_its_four_members() {
    let src = r#"
def _impl(ctx):
    exe = ctx.actions.declare_file("e")
    ctx.actions.write(exe, "x", is_executable = True)
    d = DefaultInfo(executable = exe, runfiles = ctx.runfiles(files = [exe]))
    fail("|".join([str(DefaultInfo(files = depset([exe]))), str(dir(d)), str(d.files_to_run), hasattr(d, "executable") and "has" or "none"]))
r = rule(implementation = _impl)
"#;
    let err = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap_err();
    let want = [
        "struct(data_runfiles = None, default_runfiles = None, files = depset([<generated file e>]), files_to_run = None)",
        r#"["data_runfiles", "default_runfiles", "files", "files_to_run"]"#,
        "None",
        "none",
    ]
    .join("|");
    assert!(err.contains(&want), "wanted `{want}` in\n{err}");
}

/// Probed on Bazel 9.2.0: `single_arch_platform` is an `apple_platform`.
#[test]
fn single_arch_platform_is_an_apple_platform() {
    let src = r#"
def _impl(ctx):
    s = ctx.fragments.apple.single_arch_platform
    fail("|".join([str(s), type(s), str(dir(s)), s.name_in_plist, str(s.is_device), str(s == s), str(s == apple_common.platform.macos), str({s: 1}[s])]))
r = rule(implementation = _impl, fragments = ["apple"])
"#;
    let err = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap_err();
    let want = r#"macos|apple_platform|["is_device", "name", "name_in_plist", "platform_type"]|MacOSX|True|True|False|1"#;
    assert!(err.contains(want), "wanted `{want}` in\n{err}");
}

/// Probed on Bazel 9.2.0: a type annotation in a `.bzl` is parsed and ignored,
/// so a name in it need not exist and nothing in it is evaluated.
#[test]
fn type_annotations_in_a_bzl_are_ignored() {
    let src = r#"
x: undefined_name = 1
def f(a: undefined_name, *args: depset, b: struct = 2, **kw: typing.Any) -> "int" | nothing:
    y: tuple[int, str] = (a, b)
    return y[0]
def _impl(ctx):
    fail("got %s %s" % (f(7), x))
r = rule(implementation = _impl)
"#;
    let err = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap_err();
    assert!(err.contains("got 7 1"), "{err}");
}

/// Probed on Bazel 9.2.0: an instance of a provider Bazel implements natively
/// has the provider's name as its type; one of a provider a `.bzl` made is a
/// `struct`, whatever the provider is called.
#[test]
fn the_type_of_a_native_provider_instance_is_its_name() {
    let src = r#"
Mine = provider()
OutputGroupInfo2 = provider()
def _impl(ctx):
    fail("|".join([
        type(DefaultInfo()),
        type(OutputGroupInfo()),
        type(RunEnvironmentInfo()),
        type(Mine()),
        type(OutputGroupInfo2()),
        type(struct()),
        type(Mine),
        type(1),
    ]))
r = rule(implementation = _impl)
"#;
    let err = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap_err();
    let want = "DefaultInfo|OutputGroupInfo|RunEnvironmentInfo|struct|struct|struct|Provider|int";
    assert!(err.contains(want), "wanted `{want}` in\n{err}");
}

/// Probed on Bazel 9.2.0: `testing.ExecutionInfo` and `testing.TestEnvironment`
/// (a `RunEnvironmentInfo`) take their arguments positionally, with defaults.
#[test]
fn execution_info_and_test_environment_take_positional_arguments() {
    let src = r#"
def _impl(ctx):
    e = testing.ExecutionInfo({"a": "b"})
    t = testing.TestEnvironment({"A": "b"}, ["C"])
    fail("|".join([
        str(e),
        str(t),
        str(RunEnvironmentInfo()),
        type(t),
    ]))
r = rule(implementation = _impl)
"#;
    let err = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap_err();
    let want = r#"struct(exec_group = "test", requirements = {"a": "b"})|struct(environment = {"A": "b"}, inherited_environment = ["C"])|struct(environment = {}, inherited_environment = [])|RunEnvironmentInfo"#;
    assert!(err.contains(want), "wanted `{want}` in\n{err}");
    for (call, text) in [
        (
            "testing.ExecutionInfo(1)",
            "in call to ExecutionInfo(), parameter 'requirements' got value of type 'int', want 'dict'",
        ),
        (
            "testing.ExecutionInfo({}, 'x', 3)",
            "ExecutionInfo() accepts no more than 2 positional arguments but got 3",
        ),
        (
            "RunEnvironmentInfo(nope = 1)",
            "RunEnvironmentInfo() got unexpected keyword argument 'nope'",
        ),
        (
            "RunEnvironmentInfo({'a': 1})",
            "got dict<string, int> for 'environment', want dict<string, string>",
        ),
    ] {
        let src = format!(
            "def _impl(ctx):\n    {call}\n    return []\nr = rule(implementation = _impl)\n"
        );
        let err = run_rule(&request(&src, "r", Vec::new(), Vec::new())).unwrap_err();
        assert!(err.contains(text), "{call}: wanted `{text}` in\n{err}");
    }
}

/// Probed on Bazel 9.2.0: a provider Bazel implements natively prints as a
/// function of its name, one a `.bzl` makes (and a few of Bazel's own) as
/// `<provider>`, and `apple_common.XcodeProperties` is None.
#[test]
fn native_providers_print_as_functions() {
    let src = r#"
Mine = provider()
def _impl(ctx):
    fail("|".join([
        str(DefaultInfo),
        str(testing.ExecutionInfo),
        str(platform_common.ToolchainInfo),
        str(Mine),
        str(apple_common.Objc),
        str(apple_common.XcodeVersionConfig),
        str(apple_common.XcodeProperties),
    ]))
r = rule(implementation = _impl)
"#;
    let err = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap_err();
    let want = "<function DefaultInfo>|<function ExecutionInfo>|<function ToolchainInfo>|<provider>|<provider>|<provider>|None";
    assert!(err.contains(want), "wanted `{want}` in\n{err}");
}

/// Probed on Bazel 9.2.0: methods of the native module values print as
/// built-in methods of them and have that type.
#[test]
fn native_module_methods_print_as_built_in_methods() {
    let src = r#"
def _impl(ctx):
    fail("|".join([
        str(testing.TestEnvironment),
        type(testing.TestEnvironment),
        str(coverage_common.instrumented_files_info),
        str(config_common.toolchain_type),
        str(testing.TestEnvironment({"A": "b"}).environment),
    ]))
r = rule(implementation = _impl)
"#;
    let err = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap_err();
    let want = r#"<built-in method TestEnvironment of testing value>|builtin_function_or_method|<built-in method instrumented_files_info of coverage_common value>|<built-in method toolchain_type of config_common value>|{"A": "b"}"#;
    assert!(err.contains(want), "wanted `{want}` in\n{err}");
}

/// Probed on Bazel 9.2.0: the functions of `apple_common` that Bazel defines
/// in its builtins print with their file, `new_objc_provider` is a provider,
/// and the feature flag provider is its own type.
#[test]
fn apple_common_functions_print_with_their_builtins_file() {
    let src = r#"
def _impl(ctx):
    fail("|".join([
        str(apple_common.target_apple_env),
        type(apple_common.target_apple_env),
        str(apple_common.dotted_version),
        str(apple_common.new_objc_provider),
        type(apple_common.new_objc_provider),
        type(config_common.FeatureFlagInfo),
        apple_common.dotted_version("1.2"),
    ]))
r = rule(implementation = _impl)
"#;
    let err = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap_err();
    let want = "<function target_apple_env from @@_builtins//:common/objc/apple_env.bzl>|function|<function lambda from @@_builtins//:common/objc/apple_common.bzl>|<provider>|Provider|FeatureFlagInfo|1.2";
    assert!(err.contains(want), "wanted `{want}` in\n{err}");
}

/// Probed on Bazel 9.2.0: strings have no `codepoints`.
#[test]
fn strings_have_no_codepoints() {
    let src = r#"
def _impl(ctx):
    fail("%s|%s" % ("codepoints" in dir(""), "elems" in dir("")))
r = rule(implementation = _impl)
"#;
    let err = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap_err();
    assert!(err.contains("False|True"), "{err}");
    let src = "def _impl(ctx):\n    \"x\".codepoints()\nr = rule(implementation = _impl)\n";
    let err = run_rule(&request(src, "r", Vec::new(), Vec::new())).unwrap_err();
    assert!(
        err.contains("'string' value has no field or method 'codepoints'"),
        "{err}"
    );
}
