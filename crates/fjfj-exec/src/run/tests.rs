use super::*;
use crate::execroot::Layout;
use fjfj_graph::{Action, ActionKind, Artifact, Label};
use std::collections::BTreeMap;

const BIN: &str = "bazel-out/k8-fastbuild/bin";

fn layout() -> (tempfile::TempDir, Layout) {
    let dir = tempfile::tempdir().unwrap();
    let layout = Layout {
        workspace: dir.path().join("ws"),
        output_base: dir.path().join("ob"),
    };
    std::fs::create_dir_all(&layout.workspace).unwrap();
    layout.prepare().unwrap();
    (dir, layout)
}

fn out(name: &str) -> Artifact {
    Artifact::derived(BIN, "", "", name)
}

fn shell(script: &str, inputs: Vec<Artifact>, outputs: Vec<Artifact>) -> Action {
    Action {
        owner: Label {
            repo: String::new(),
            package: String::new(),
            name: "t".into(),
        },
        owner_kind: "genrule".into(),
        location: "BUILD.bazel:1:8".into(),
        configuration: "k8-fastbuild".into(),
        mnemonic: "Genrule".into(),
        progress_message: None,
        kind: ActionKind::Spawn {
            argv: vec!["/bin/bash".into(), "-c".into(), script.into()],
            env: BTreeMap::from([("PATH".into(), "/bin:/usr/bin".into())]),
            execution_requirements: BTreeMap::new(),
        },
        inputs,
        outputs,
    }
}

async fn run(
    layout: &Layout,
    actions: Vec<Action>,
    want: &[Artifact],
    keep_going: bool,
) -> Outcome {
    let options = Options {
        jobs: 4,
        keep_going,
    };
    execute(layout, actions, want, &options, Arc::new(Quiet)).await
}

#[tokio::test]
async fn actions_run_after_the_actions_that_make_their_inputs() {
    let (_dir, layout) = layout();
    std::fs::write(layout.workspace.join("in.txt"), "hello\n").unwrap();
    layout.prepare().unwrap();
    let a = out("a.txt");
    let b = out("b.txt");
    let src = Artifact::source("", "", "in.txt");
    let actions = vec![
        // Listed in the wrong order on purpose.
        shell(
            &format!(
                "cat {} > {}; echo more >> {}",
                a.exec_path(),
                b.exec_path(),
                b.exec_path()
            ),
            vec![a.clone()],
            vec![b.clone()],
        ),
        shell(
            &format!("cat in.txt > {}", a.exec_path()),
            vec![src],
            vec![a.clone()],
        ),
    ];
    let outcome = run(&layout, actions, std::slice::from_ref(&b), false).await;
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    assert_eq!((outcome.spawned, outcome.ran), (2, 2));
    let made = layout.execroot().join(b.exec_path());
    assert_eq!(std::fs::read_to_string(&made).unwrap(), "hello\nmore\n");
    // Outputs are read-only, as Bazel leaves them.
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        std::fs::metadata(&made).unwrap().permissions().mode() & 0o777,
        0o555
    );
}

#[tokio::test]
async fn an_action_that_fails_says_how_and_fails_its_dependents() {
    let (_dir, layout) = layout();
    let a = out("a.txt");
    let b = out("b.txt");
    let actions = vec![
        shell("echo oops >&2; exit 3", vec![], vec![a.clone()]),
        shell("true", vec![a.clone()], vec![b.clone()]),
    ];
    let outcome = run(&layout, actions, &[b], false).await;
    assert_eq!(outcome.failures.len(), 1);
    let failure = &outcome.failures[0];
    assert_eq!(failure.owner, "//:t");
    assert_eq!(
        failure.message,
        "(Exit 3): bash failed: error executing Genrule command (from genrule rule target //:t) /bin/bash -c 'echo oops >&2; exit 3'"
    );
    assert_eq!(failure.output, "oops\n");
    assert_eq!(outcome.ran, 0);
}

#[tokio::test]
async fn an_output_the_command_did_not_make_is_an_error() {
    let (_dir, layout) = layout();
    let a = out("a.txt");
    let outcome = run(
        &layout,
        vec![shell("true", vec![], vec![a.clone()])],
        &[a],
        false,
    )
    .await;
    assert_eq!(
        outcome.failures[0].message,
        "not all outputs were created or valid"
    );
    assert!(
        outcome.failures[0].details[0].starts_with(
            "declared output 'a.txt' was not created by genrule. This is probably because"
        ),
        "{:?}",
        outcome.failures[0].details
    );
}

#[tokio::test]
async fn a_missing_source_input_is_reported() {
    let (_dir, layout) = layout();
    let a = out("a.txt");
    let actions = vec![shell(
        "true",
        vec![Artifact::source("", "", "gone.txt")],
        vec![a.clone()],
    )];
    let outcome = run(&layout, actions, &[a], false).await;
    assert_eq!(
        outcome.failures[0].message,
        "missing input file 'gone.txt', owner: '//:t'"
    );
}

#[tokio::test]
async fn keep_going_runs_what_does_not_depend_on_the_failure() {
    let (_dir, layout) = layout();
    let (bad, good) = (out("bad"), out("good"));
    let actions = vec![
        shell("exit 1", vec![], vec![bad.clone()]),
        shell(
            &format!("echo ok > {}", good.exec_path()),
            vec![],
            vec![good.clone()],
        ),
    ];
    let outcome = run(&layout, actions, &[bad, good.clone()], true).await;
    assert_eq!(outcome.failures.len(), 1);
    assert_eq!(outcome.ran, 1);
    assert!(layout.execroot().join(good.exec_path()).exists());
}

#[tokio::test]
async fn write_file_and_symlink_actions_make_their_outputs() {
    let (_dir, layout) = layout();
    let (w, l) = (out("w.txt"), out("l.txt"));
    let owner = Label {
        repo: String::new(),
        package: String::new(),
        name: "t".into(),
    };
    let actions = vec![
        Action {
            owner: owner.clone(),
            owner_kind: "x".into(),
            location: String::new(),
            configuration: "k8-fastbuild".into(),
            mnemonic: "FileWrite".into(),
            progress_message: None,
            kind: ActionKind::WriteFile {
                contents: b"data".to_vec(),
                executable: false,
            },
            inputs: vec![],
            outputs: vec![w.clone()],
        },
        Action {
            owner,
            owner_kind: "x".into(),
            location: String::new(),
            configuration: "k8-fastbuild".into(),
            mnemonic: "Symlink".into(),
            progress_message: None,
            kind: ActionKind::Symlink {
                target: w.exec_path(),
            },
            inputs: vec![w.clone()],
            outputs: vec![l.clone()],
        },
    ];
    let outcome = run(&layout, actions, std::slice::from_ref(&l), false).await;
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    assert_eq!(
        std::fs::read_to_string(layout.execroot().join(l.exec_path())).unwrap(),
        "data"
    );
    assert_eq!(outcome.spawned, 0);
}

#[tokio::test]
async fn an_action_whose_inputs_and_outputs_are_unchanged_does_not_run_again() {
    let (_dir, layout) = layout();
    std::fs::write(layout.workspace.join("in.txt"), "one\n").unwrap();
    layout.prepare().unwrap();
    let out = out("a.txt");
    let src = Artifact::source("", "", "in.txt");
    let actions = || {
        vec![shell(
            &format!(
                "cat in.txt > {}; echo ran >> {}.log",
                out.exec_path(),
                out.exec_path()
            ),
            vec![src.clone()],
            vec![out.clone()],
        )]
    };
    let made = layout.execroot().join(out.exec_path());

    let first = run(&layout, actions(), std::slice::from_ref(&out), false).await;
    assert_eq!((first.ran, first.cached, first.spawned), (1, 0, 1));
    assert_eq!(std::fs::read_to_string(&made).unwrap(), "one\n");

    // Nothing changed: nothing runs, in this process or a later one.
    let again = run(&layout, actions(), std::slice::from_ref(&out), false).await;
    assert_eq!((again.ran, again.cached, again.spawned), (0, 1, 0));

    // The input changed.
    std::fs::write(layout.workspace.join("in.txt"), "two\n").unwrap();
    let third = run(&layout, actions(), std::slice::from_ref(&out), false).await;
    assert_eq!((third.ran, third.cached), (1, 0));
    assert_eq!(std::fs::read_to_string(&made).unwrap(), "two\n");

    // The output was removed, or tampered with.
    std::fs::remove_file(&made).unwrap();
    let fourth = run(&layout, actions(), std::slice::from_ref(&out), false).await;
    assert_eq!((fourth.ran, fourth.cached), (1, 0));
    std::fs::set_permissions(&made, std::os::unix::fs::PermissionsExt::from_mode(0o644)).unwrap();
    std::fs::write(&made, "changed\n").unwrap();
    let fifth = run(&layout, actions(), std::slice::from_ref(&out), false).await;
    assert_eq!((fifth.ran, fifth.cached), (1, 0));
    assert_eq!(std::fs::read_to_string(&made).unwrap(), "two\n");

    // The command changed.
    let changed = vec![shell(
        &format!("cat in.txt in.txt > {}", out.exec_path()),
        vec![Artifact::source("", "", "in.txt")],
        vec![out.clone()],
    )];
    let sixth = run(&layout, changed, std::slice::from_ref(&out), false).await;
    assert_eq!((sixth.ran, sixth.cached), (1, 0));
}
