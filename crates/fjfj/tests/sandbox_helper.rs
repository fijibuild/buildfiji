//! A command in namespaces starts as a fresh process of `fjfj` itself
//! (buildfiji-yi6i): this runs the helper as the executor does.

use std::process::Command;

// The test runs in the runfiles root, where `data` puts the binary.

fn helper(writable: &std::path::Path, script: &str) -> std::process::Output {
    Command::new("crates/fjfj/fjfj")
        .args(["--fjfj-sandbox-helper", "host", "-w"])
        .arg(writable)
        .args(["--", "/bin/sh", "-c", script])
        .output()
        .unwrap()
}

#[test]
fn the_helper_runs_the_command_as_pid_1_with_the_file_system_read_only() {
    if !Command::new("unshare")
        .args(["-Ur", "true"])
        .status()
        .is_ok_and(|s| s.success())
    {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let out = helper(
        dir.path(),
        &format!(
            "echo pid=$$; touch {}/made && echo wrote; touch /usr/not-here 2>/dev/null || echo read-only; exit 3",
            dir.path().display()
        ),
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("pid=1"), "{text} {:?}", out.stderr);
    assert!(
        text.contains("wrote") && text.contains("read-only"),
        "{text}"
    );
    assert!(dir.path().join("made").exists());
    assert_eq!(out.status.code(), Some(3));
}
