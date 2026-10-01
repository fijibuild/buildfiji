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
    let engine = engine(Env {
        source: repos.clone(),
        rules: repos.clone(),
        main_repo_name: "_main".into(),
    });
    let (package, name) = label.trim_start_matches("//").split_once(':').unwrap();
    engine
        .get(ConfiguredTargetKey {
            label: Label {
                repo: String::new(),
                package: package.into(),
                name: name.into(),
            },
            configuration: config(),
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
