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
        input_set: None,
        outputs,
        exec_group: None,
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
        ..Options::default()
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
            input_set: None,
            outputs: vec![w.clone()],
            exec_group: None,
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
            input_set: None,
            outputs: vec![l.clone()],
            exec_group: None,
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

/// Bazel leaves a file output, and every file and directory inside a tree
/// artifact, 0555; a plain directory is left as it is.
#[test]
fn outputs_are_read_only_and_executable_throughout_a_tree_artifact() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let tree = dir.path().join("tree");
    std::fs::create_dir_all(tree.join("sub")).unwrap();
    std::fs::write(tree.join("sub/f.txt"), "x").unwrap();
    let mode = |p: &std::path::Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
    make_read_only(&tree, true).unwrap();
    assert_eq!(mode(&tree), 0o555);
    assert_eq!(mode(&tree.join("sub")), 0o555);
    assert_eq!(mode(&tree.join("sub/f.txt")), 0o555);
    let plain = dir.path().join("plain");
    std::fs::create_dir(&plain).unwrap();
    let before = mode(&plain);
    make_read_only(&plain, false).unwrap();
    assert_eq!(mode(&plain), before);
    // Let the temporary directory clean up.
    for p in [tree.join("sub"), tree.clone()] {
        std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

#[tokio::test]
async fn a_sandboxed_command_sees_its_declared_inputs_and_leaves_only_its_declared_outputs() {
    let (_dir, layout) = layout();
    std::fs::write(layout.workspace.join("in.txt"), "hello\n").unwrap();
    std::fs::write(layout.workspace.join("other.txt"), "other\n").unwrap();
    layout.prepare().unwrap();
    let a = out("a.txt");
    let src = Artifact::source("", "", "in.txt");
    // Reads the input, looks for one it was not given, and writes a file nobody declared.
    let action = shell(
        &format!(
            "cat in.txt > {a}; test ! -e other.txt && echo hidden >> {a}; echo x > {bin}/stray.txt",
            a = a.exec_path(),
            bin = BIN
        ),
        vec![src],
        vec![a.clone()],
    );
    let outcome = run(
        &layout,
        vec![action.clone()],
        std::slice::from_ref(&a),
        false,
    )
    .await;
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    let root = layout.execroot();
    assert_eq!(
        std::fs::read_to_string(root.join(a.exec_path())).unwrap(),
        "hello\nhidden\n"
    );
    assert!(!root.join(BIN).join("stray.txt").exists());
    assert!(!layout.output_base.join("sandbox").exists());
}

#[tokio::test]
async fn a_test_runs_in_the_sandbox_unless_its_tags_say_otherwise() {
    let (_dir, layout) = layout();
    std::fs::write(layout.workspace.join("other.txt"), "other\n").unwrap();
    layout.prepare().unwrap();
    for (tag, sees_other) in [(None, false), (Some("local"), true)] {
        let log = out(&format!("t{}.log", tag.unwrap_or("-")));
        let xml = out(&format!("t{}.xml", tag.unwrap_or("-")));
        let mut action = shell(
            &format!(
                "(test -e other.txt && echo sees || echo blind) > {x}; echo x > {bin}/stray.txt",
                x = xml.exec_path(),
                bin = BIN
            ),
            vec![],
            vec![log.clone(), xml.clone()],
        );
        action.mnemonic = "TestRunner".to_owned();
        if let (
            Some(tag),
            ActionKind::Spawn {
                execution_requirements,
                ..
            },
        ) = (tag, &mut action.kind)
        {
            execution_requirements.insert(tag.to_owned(), String::new());
        }
        let outcome = run(&layout, vec![action], std::slice::from_ref(&xml), false).await;
        assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
        let seen = std::fs::read_to_string(layout.execroot().join(xml.exec_path())).unwrap();
        assert_eq!(seen.trim() == "sees", sees_other, "{tag:?}");
        assert_eq!(
            layout.execroot().join(BIN).join("stray.txt").exists(),
            sees_other
        );
    }
}

#[tokio::test]
async fn a_local_command_sees_the_whole_execroot_and_leaves_what_it_makes() {
    let (_dir, layout) = layout();
    std::fs::write(layout.workspace.join("other.txt"), "other\n").unwrap();
    layout.prepare().unwrap();
    let a = out("a.txt");
    let action = shell(
        &format!(
            "cat other.txt > {a}; echo x > {bin}/stray.txt",
            a = a.exec_path(),
            bin = BIN
        ),
        vec![],
        vec![a.clone()],
    );
    let options = Options {
        jobs: 1,
        strategy: Strategy::Local,
        ..Options::default()
    };
    let outcome = execute(
        &layout,
        vec![action],
        std::slice::from_ref(&a),
        &options,
        Arc::new(Quiet),
    )
    .await;
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    assert!(layout.execroot().join(BIN).join("stray.txt").exists());
}

#[tokio::test]
async fn a_namespace_sandboxed_command_cannot_write_to_the_real_execroot() {
    if !fjfj_sandbox::namespaces_available() {
        return;
    }
    // `/tmp` stays writable, so the build tree is somewhere else.
    let here = std::env::current_dir().unwrap();
    if here.starts_with("/tmp") || here.starts_with("/dev/shm") {
        return;
    }
    let dir = tempfile::tempdir_in(here).unwrap();
    let layout = Layout {
        workspace: dir.path().join("ws"),
        output_base: dir.path().join("ob"),
    };
    std::fs::create_dir_all(&layout.workspace).unwrap();
    layout.prepare().unwrap();
    let a = out("a.txt");
    let real = layout.execroot().join(BIN).join("stray.txt");
    let action = shell(
        &format!(
            "(echo x > {real} && echo wrote || echo refused) > {a}",
            real = real.display(),
            a = a.exec_path()
        ),
        vec![],
        vec![a.clone()],
    );
    let outcome = run(&layout, vec![action], std::slice::from_ref(&a), false).await;
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    let seen = std::fs::read_to_string(layout.execroot().join(a.exec_path())).unwrap();
    assert_eq!(seen.trim(), "refused");
    assert!(!real.exists());
}

#[test]
fn spawn_strategy_names_pick_the_first_strategy_that_runs() {
    assert_eq!(Strategy::parse("local").unwrap(), Strategy::Local);
    assert_eq!(
        Strategy::parse("remote,sandboxed,local").unwrap(),
        Strategy::LinuxSandbox
    );
    assert_eq!(
        Strategy::parse("worker,standalone").unwrap(),
        Strategy::Local
    );
    assert_eq!(
        Strategy::parse("processwrapper-sandbox").unwrap(),
        Strategy::Sandboxed
    );
    assert!(Strategy::parse("remote").is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn independent_actions_run_at_once_up_to_jobs() {
    let (_dir, layout) = layout();
    let outs: Vec<Artifact> = (0..4).map(|i| out(&format!("s{i}.txt"))).collect();
    let actions = outs
        .iter()
        .map(|o| {
            shell(
                &format!("sleep 1; echo > {}", o.exec_path()),
                vec![],
                vec![o.clone()],
            )
        })
        .collect();
    let started = std::time::Instant::now();
    let outcome = run(&layout, actions, &outs, false).await;
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    assert_eq!(outcome.ran, 4);
    assert!(
        started.elapsed() < std::time::Duration::from_millis(2500),
        "four one-second actions took {:?}",
        started.elapsed()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn an_action_that_runs_no_command_does_not_wait_for_a_slot() {
    let (_dir, layout) = layout();
    let (made, seen) = (out("made.txt"), out("seen.txt"));
    let mut write = shell("", vec![], vec![made.clone()]);
    write.kind = ActionKind::WriteFile {
        contents: b"hi\n".to_vec(),
        executable: false,
    };
    // The only slot is held by a command that ends when the write has happened.
    let wait = shell(
        &format!(
            "for i in $(seq 100); do [ -e {made} ] && exec cp {made} {seen}; sleep 0.1; done; exit 1",
            made = made.exec_path(),
            seen = seen.exec_path()
        ),
        vec![],
        vec![seen.clone()],
    );
    let options = Options {
        jobs: 1,
        strategy: Strategy::Local,
        ..Options::default()
    };
    let outcome = execute(
        &layout,
        vec![wait, write],
        &[made, seen],
        &options,
        Arc::new(Quiet),
    )
    .await;
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
}

/// Spans reach the file from every thread, so the subscriber is the process's:
/// a span that closes on a blocking thread releases its parent through that
/// thread's default.
#[tokio::test]
async fn each_action_that_runs_has_a_span_with_a_span_for_each_step() {
    use tracing_subscriber::layer::SubscriberExt;
    let (dir, layout) = layout();
    let path = dir.path().join("trace.json");
    let trace = fjfj_telemetry::trace_file::TraceFile::create(&path).unwrap();
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(fjfj_telemetry::trace_file::TraceLayer(trace.clone())),
    )
    .unwrap();
    let (a, b) = (out("span-a.txt"), out("span-b.txt"));
    let actions = vec![
        shell(
            &format!("echo > {}", a.exec_path()),
            vec![],
            vec![a.clone()],
        ),
        shell(
            &format!("cat {} > {}", a.exec_path(), b.exec_path()),
            vec![a.clone()],
            vec![b.clone()],
        ),
    ];
    let outcome = run(&layout, actions, std::slice::from_ref(&b), false).await;
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);

    // The action spans close as their tasks end, a moment after the outcome.
    let mut events: Vec<serde_json::Value> = Vec::new();
    for _ in 0..100 {
        trace.flush();
        events = std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .filter_map(|l| serde_json::from_str(l.trim_end_matches(',')).ok())
            .filter(|e: &serde_json::Value| e["cat"].is_string())
            .collect();
        if events.iter().filter(|e| e["cat"] == "action").count() == 2 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    let action = |out: &str| {
        events
            .iter()
            .find(|e| e["cat"] == "action" && e["args"]["output"] == out)
            .unwrap_or_else(|| panic!("no action span for {out}: {events:?}"))
    };
    let steps = |row: &serde_json::Value| -> Vec<String> {
        let mut names: Vec<String> = events
            .iter()
            .filter(|e| e["cat"] == "step" && e["tid"] == row["tid"])
            .map(|e| e["name"].as_str().unwrap().to_owned())
            .collect();
        names.sort();
        names
    };
    let (first, second) = (action(&a.exec_path()), action(&b.exec_path()));
    let all = [
        "collect", "command", "deps", "key", "prepare", "record", "slot", "spawn",
    ];
    assert_eq!(steps(first), all);
    assert_eq!(steps(second), all);
    assert_eq!(first["args"]["cached"], false);
    assert_eq!(second["args"]["blocker"], a.exec_path());
}

#[test]
fn an_action_that_others_wait_for_ranks_above_one_nothing_waits_for() {
    let (a, b, c, lone) = (out("a"), out("b"), out("c"), out("lone"));
    let actions = vec![
        shell("true", vec![], vec![a.clone()]),
        shell("true", vec![a.clone()], vec![b.clone()]),
        shell("true", vec![b.clone()], vec![c]),
        shell("true", vec![], vec![lone]),
    ];
    let mut by_output = HashMap::new();
    for (i, action) in actions.iter().enumerate() {
        for output in &action.outputs {
            by_output.insert(output.clone(), i);
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let cache = ActionCache::load(dir.path().join("cache.json"));
    let ranks = ranks(&actions, &by_output, &cache);
    assert_eq!(ranks[0], 3 * GUESS_US);
    assert_eq!(ranks[1], 2 * GUESS_US);
    assert_eq!(ranks[2], GUESS_US);
    assert_eq!(ranks[3], GUESS_US);
}

#[test]
fn a_compile_outranks_a_symlink_at_the_same_depth_before_any_build_has_timed_them() {
    let (compiled, linked) = (out("compiled"), out("linked"));
    let mut compile = shell("true", vec![], vec![compiled]);
    compile.mnemonic = "Rustc".into();
    let mut link = shell("true", vec![], vec![linked]);
    link.mnemonic = "Symlink".into();
    link.kind = ActionKind::Symlink {
        target: "elsewhere".into(),
    };
    let actions = vec![compile, link];
    let by_output = HashMap::new();
    let dir = tempfile::tempdir().unwrap();
    let cache = ActionCache::load(dir.path().join("cache.json"));
    let ranks = ranks(&actions, &by_output, &cache);
    assert!(ranks[0] > ranks[1], "{ranks:?}");
}

#[test]
fn an_action_never_timed_is_ranked_by_what_its_mnemonic_took_not_the_hand_set_table() {
    let dir = tempfile::tempdir().unwrap();
    let cache = ActionCache::load(dir.path().join("cache.json"));
    // Earlier builds: Rustc took 1ms and a Genrule 10s, the reverse of the table.
    let mut fast = shell("true", vec![], vec![out("fast")]);
    fast.mnemonic = "Rustc".into();
    let slow = shell("true", vec![], vec![out("slow")]);
    cache.record(
        dir.path(),
        &fast,
        "k".into(),
        std::time::Duration::from_millis(1),
    );
    cache.record(
        dir.path(),
        &slow,
        "k".into(),
        std::time::Duration::from_secs(10),
    );
    let mut new_rustc = shell("true", vec![], vec![out("new_rustc")]);
    new_rustc.mnemonic = "Rustc".into();
    let new_genrule = shell("true", vec![], vec![out("new_genrule")]);
    let ranks = ranks(&[new_rustc, new_genrule], &HashMap::new(), &cache);
    assert!(ranks[1] > ranks[0], "{ranks:?}");
}

#[test]
fn what_a_mnemonic_took_is_kept_for_a_cache_in_a_fresh_output_base() {
    let dir = tempfile::tempdir().unwrap();
    let history = dir.path().join("cache").join("mnemonic-durations.json");
    let first = ActionCache::load_with_history(dir.path().join("a.json"), Some(history.clone()));
    let action = shell("true", vec![], vec![out("x")]);
    first.record(
        dir.path(),
        &action,
        "k".into(),
        std::time::Duration::from_secs(7),
    );
    first.save().unwrap();
    let fresh = ActionCache::load_with_history(dir.path().join("b.json"), Some(history));
    assert_eq!(
        fresh.mnemonic_mean("Genrule"),
        Some(std::time::Duration::from_secs(7))
    );
}

#[tokio::test]
async fn the_closure_counts_what_the_requested_outputs_need_and_no_more() {
    let (_dir, layout) = layout();
    let (a, b, c) = (out("a"), out("b"), out("c"));
    let actions = vec![
        shell(
            &format!("echo a > {}", a.exec_path()),
            vec![],
            vec![a.clone()],
        ),
        shell(
            &format!("echo b > {}", b.exec_path()),
            vec![a.clone()],
            vec![b.clone()],
        ),
        // Analysed, but nothing requested needs it.
        shell(&format!("echo c > {}", c.exec_path()), vec![], vec![c]),
    ];
    let outcome = run(&layout, actions, &[b], false).await;
    assert_eq!(outcome.closure, 2);
}

#[tokio::test]
async fn the_critical_path_is_the_longest_chain_not_the_time_of_all_of_them() {
    let (_dir, layout) = layout();
    let (a, b, c) = (out("a"), out("b"), out("c"));
    let actions = vec![
        shell(
            &format!("sleep 0.3; echo a > {}", a.exec_path()),
            vec![],
            vec![a.clone()],
        ),
        shell(
            &format!("sleep 0.3; echo b > {}", b.exec_path()),
            vec![a.clone()],
            vec![b.clone()],
        ),
        // Alongside the chain, not part of it.
        shell(
            &format!("sleep 0.3; echo c > {}", c.exec_path()),
            vec![],
            vec![c.clone()],
        ),
    ];
    let outcome = run(&layout, actions, &[b, c], false).await;
    let sum: std::time::Duration = outcome.durations.iter().map(|(_, d)| *d).sum();
    assert!(outcome.critical_path >= std::time::Duration::from_millis(600));
    assert!(outcome.critical_path < sum);
}

#[tokio::test]
async fn a_parameter_file_write_is_not_counted_as_an_action() {
    let (_dir, layout) = layout();
    let (params, b) = (out("p"), out("b"));
    let mut write = shell(
        &format!("echo p > {}", params.exec_path()),
        vec![],
        vec![params.clone()],
    );
    write.mnemonic = "ParameterFileWrite".into();
    let actions = vec![
        write,
        shell(
            &format!("echo b > {}", b.exec_path()),
            vec![params],
            vec![b.clone()],
        ),
    ];
    let outcome = run(&layout, actions, &[b], false).await;
    assert_eq!((outcome.closure, outcome.ran), (1, 1));
}

fn internal(
    mnemonic: &str,
    kind: ActionKind,
    inputs: Vec<Artifact>,
    outputs: Vec<Artifact>,
) -> Action {
    Action {
        mnemonic: mnemonic.into(),
        kind,
        ..shell("", inputs, outputs)
    }
}

/// The four actions of a runfiles tree for one file `data`.
fn runfiles(data: &Artifact) -> (Vec<Action>, Artifact) {
    let (dir, manifest, mapping) = (
        out("bin.runfiles"),
        out("bin.runfiles_manifest"),
        out("bin.repo_mapping"),
    );
    let tree_manifest = out("bin.runfiles/MANIFEST");
    let actions = vec![
        internal(
            "RepoMappingManifest",
            ActionKind::WriteFile {
                contents: b",m,_main\n".to_vec(),
                executable: false,
            },
            vec![],
            vec![mapping.clone()],
        ),
        internal(
            "SourceSymlinkManifest",
            ActionKind::SourceManifest {
                repo_mapping: mapping.exec_path(),
                entries: vec![("_main/data".into(), data.clone())],
                empty_files: vec!["_main/z.py".into()],
            },
            vec![],
            vec![manifest.clone()],
        ),
        internal(
            "SymlinkTree",
            ActionKind::SymlinkTree {
                dir: dir.exec_path(),
                repo_mapping: mapping.exec_path(),
                entries: vec![("_main/data".into(), data.clone())],
                empty_files: vec!["_main/z.py".into()],
            },
            vec![manifest],
            vec![tree_manifest.clone()],
        ),
        internal(
            "RunfilesTree",
            ActionKind::RunfilesTree,
            vec![data.clone(), tree_manifest, mapping],
            vec![dir.clone()],
        ),
    ];
    (actions, dir)
}

#[tokio::test]
async fn a_runfiles_tree_is_four_actions_and_its_directory_is_left_as_the_tree_made_it() {
    let (_dir, layout) = layout();
    let data = out("data");
    let mut actions = vec![shell(
        &format!("echo d > {}", data.exec_path()),
        vec![],
        vec![data.clone()],
    )];
    let (tree, dir) = runfiles(&data);
    actions.extend(tree);
    let outcome = run(&layout, actions.clone(), std::slice::from_ref(&dir), false).await;
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    assert_eq!((outcome.closure, outcome.ran), (5, 5));
    let at = layout.execroot().join(dir.exec_path());
    let listing = std::fs::read_to_string(at.join("MANIFEST")).unwrap();
    assert!(listing.starts_with("_main/data "), "{listing}");
    assert!(listing.contains("_repo_mapping "), "{listing}");
    // An empty file is a path and a space; a path to a file is absolute; and
    // the manifest of the tree is the manifest the tree was made from.
    assert!(listing.contains("\n_main/z.py \n"), "{listing}");
    let root = layout.execroot();
    assert!(
        listing.contains(&format!(
            "_main/data {}\n",
            root.join(data.exec_path()).display()
        )),
        "{listing}"
    );
    assert_eq!(
        std::fs::read_to_string(root.join(out("bin.runfiles_manifest").exec_path())).unwrap(),
        listing
    );
    assert_eq!(
        std::fs::read_to_string(at.join("_main/data")).unwrap(),
        "d\n"
    );
    // Run again with nothing changed: the directory is as it was.
    let again = run(&layout, actions, &[dir], false).await;
    assert_eq!((again.ran, again.cached), (0, 5));
    assert!(at.join("_main/data").exists());
}

#[tokio::test]
async fn the_workspace_status_action_writes_both_files() {
    let (_dir, layout) = layout();
    let stable = Artifact {
        root: fjfj_graph::Root::derived("bazel-out"),
        path: "stable-status.txt".into(),
        tree: false,
    };
    let volatile = Artifact {
        path: "volatile-status.txt".into(),
        ..stable.clone()
    };
    let action = internal(
        "BazelWorkspaceStatusAction",
        ActionKind::WorkspaceStatus {
            stable: "BUILD_USER me\n".into(),
            volatile: "BUILD_TIMESTAMP 1\n".into(),
        },
        vec![],
        vec![stable.clone(), volatile.clone()],
    );
    let outcome = run(&layout, vec![action], std::slice::from_ref(&stable), false).await;
    assert_eq!((outcome.closure, outcome.ran, outcome.spawned), (1, 1, 0));
    let root = layout.execroot();
    assert_eq!(
        std::fs::read_to_string(root.join(stable.exec_path())).unwrap(),
        "BUILD_USER me\n"
    );
    assert_eq!(
        std::fs::read_to_string(root.join(volatile.exec_path())).unwrap(),
        "BUILD_TIMESTAMP 1\n"
    );
}

#[tokio::test]
async fn a_manifest_lists_a_link_that_is_not_followed_as_it_reads() {
    let (_dir, layout) = layout();
    let link = out("venv/python3");
    let manifest = out("bin.runfiles_manifest");
    let mapping = out("bin.repo_mapping");
    let actions = vec![
        internal(
            "UnresolvedSymlink",
            ActionKind::UnresolvedSymlink {
                target: "../elsewhere/python3".into(),
            },
            vec![],
            vec![link.clone()],
        ),
        internal(
            "SourceSymlinkManifest",
            ActionKind::SourceManifest {
                repo_mapping: mapping.exec_path(),
                entries: vec![("_main/venv/python3".into(), link)],
                empty_files: vec![],
            },
            vec![],
            vec![manifest.clone()],
        ),
        internal(
            "RepoMappingManifest",
            ActionKind::WriteFile {
                contents: Vec::new(),
                executable: false,
            },
            vec![],
            vec![mapping],
        ),
    ];
    let outcome = run(&layout, actions, std::slice::from_ref(&manifest), false).await;
    assert!(outcome.failures.is_empty(), "{:?}", outcome.failures);
    let text = std::fs::read_to_string(layout.execroot().join(manifest.exec_path())).unwrap();
    assert!(
        text.starts_with("_main/venv/python3 ../elsewhere/python3\n"),
        "{text}"
    );
}
