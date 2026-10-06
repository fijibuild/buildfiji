//! `fjfj run` (buildfiji-gwl.8): build one target and run it.
//!
//! As `bazel run` on Linux does it (Bazel 9.2.0): the program is the
//! executable of the target, run with the real path in `argv[0]`, in the
//! `_main` directory of its runfiles tree, with the environment fjfj was run
//! in and `BUILD_WORKSPACE_DIRECTORY`, `BUILD_WORKING_DIRECTORY`,
//! `BUILD_EXECROOT` and `BUILD_ID` added. Its exit code is `run`'s.

use crate::{CliError, build_main};
use fjfj_bazel_compat::TargetArgs;
use fjfj_bazel_compat::run_flags::{RunUnder, shell_escape};
use std::os::unix::process::ExitStatusExt;
use std::process::Command;

pub(crate) async fn run(args: TargetArgs) -> Result<(), CliError> {
    let built = build_main(args, "run", true).await?;
    let layout = &built.layout;
    let under_target = matches!(built.run_flags.run_under, Some(RunUnder::Target { .. }));
    let results = &built.report.results[..];
    // The target, then the `--run_under` target if it is another one.
    let wanted = if under_target && results.len() > 1 {
        2
    } else {
        1
    };
    if results.len() != wanted {
        return Err(CliError::Build(anyhow::anyhow!(
            "Only one target can be run, not {}",
            results.len()
        )));
    }
    let result = &results[0];
    let executable = &executable_of(result, layout)?;
    let program = layout.resolve(executable);
    // A derived file is shown by its link, a source file by its real path.
    let shown_of = |artifact: &fjfj_graph::Artifact| {
        let shown = crate::build_command::shown_path(
            &built.options.symlink_prefix,
            &built.options.configuration,
            artifact,
        );
        if shown == artifact.exec_path() {
            layout.resolve(artifact).display().to_string()
        } else {
            shown
        }
    };
    let shown = shown_of(executable);
    // The runfiles tree is beside the executable.
    let runfiles = std::path::PathBuf::from(format!("{}.runfiles", program.display()));
    let cwd = if runfiles.is_dir() {
        runfiles.join(crate::build_command::MAIN_REPO_NAME)
    } else {
        layout.workspace.clone()
    };
    // What runs, as words, and as the console shows it.
    let program_text = program.display().to_string();
    let (mut argv, console): (Vec<String>, Vec<String>) = match &built.run_flags.run_under {
        None => (vec![program_text], vec![shown]),
        Some(RunUnder::Command(command)) => {
            let line = format!(
                "{command} {}",
                std::iter::once(&program_text)
                    .chain(&built.program_args)
                    .map(|w| shell_escape(w))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            (
                vec!["/bin/bash".to_owned(), "-c".to_owned(), line],
                vec![command.clone(), shown],
            )
        }
        Some(RunUnder::Target { options, .. }) => {
            // Built after the target unless it is the target.
            let under = results.get(1).unwrap_or(result);
            let under_exe = &executable_of(under, layout)?;
            let mut words = vec![layout.resolve(under_exe).display().to_string()];
            words.extend(options.iter().cloned());
            words.push(program_text);
            let mut shown_words = vec![shown_of(under_exe)];
            shown_words.extend(options.iter().map(|o| shell_escape(o)));
            shown_words.push(shown);
            (words, shown_words)
        }
    };
    let working_directory = std::env::current_dir()
        .map_err(|e| CliError::Internal(anyhow::anyhow!("couldn't get current directory: {e}")))?;
    let build_id = build_id();
    let env = [
        ("BUILD_EXECROOT", layout.execroot().display().to_string()),
        ("BUILD_ID", build_id),
        (
            "BUILD_WORKING_DIRECTORY",
            working_directory.display().to_string(),
        ),
        (
            "BUILD_WORKSPACE_DIRECTORY",
            layout.workspace.display().to_string(),
        ),
    ];
    if let Some(path) = &built.run_flags.script_path {
        // Under anything the script runs a shell with the whole line, the
        // program's arguments in it; its own arguments follow.
        let line = |words: &[String]| {
            words
                .iter()
                .chain(&built.program_args)
                .map(|w| shell_escape(w))
                .collect::<Vec<_>>()
                .join(" ")
        };
        let command = match &built.run_flags.run_under {
            None => line(&argv),
            Some(RunUnder::Command(_)) => argv
                .iter()
                .map(|w| shell_escape(w))
                .collect::<Vec<_>>()
                .join(" "),
            Some(RunUnder::Target { .. }) => {
                format!("/bin/bash -c {}", shell_escape(&line(&argv)))
            }
        };
        let mut script = format!(
            "#!/bin/bash\ncd {} && \\\n  exec env \\\n",
            shell_escape(&cwd.display().to_string())
        );
        for name in UNSET {
            script.push_str(&format!("    -u {name} \\\n"));
        }
        for (name, value) in &env {
            script.push_str(&format!("    {name}={} \\\n", shell_escape(value)));
        }
        script.push_str(&format!("  {command} \"$@\""));
        return write_script(path, &script);
    }
    if !matches!(built.run_flags.run_under, Some(RunUnder::Command(_))) {
        argv.extend(built.program_args.iter().cloned());
    }
    if built.program_args.is_empty() {
        eprintln!("INFO: Running command line: {}", console.join(" "));
    } else {
        eprintln!(
            "INFO: Running command line: {} <args omitted>",
            console.join(" ")
        );
    }
    let mut command = Command::new(&argv[0]);
    command.args(&argv[1..]).current_dir(&cwd).env("PWD", &cwd);
    for name in UNSET {
        command.env_remove(name);
    }
    for (name, value) in &env {
        command.env(name, value);
    }
    let status = command
        .status()
        .map_err(|e| CliError::Build(anyhow::anyhow!("Cannot execute {}: {e}", argv[0])))?;
    match (status.code(), status.signal()) {
        (Some(0), _) => Ok(()),
        (Some(code), _) => Err(CliError::Program(u8::try_from(code).unwrap_or(1))),
        (None, Some(signal)) => Err(CliError::Program(u8::try_from(128 + signal).unwrap_or(255))),
        _ => Err(CliError::Program(1)),
    }
}

/// What a target runs: its executable, or the file it is if that is a source
/// file with the executable bit. Anything else was stopped before the build,
/// except a source file that cannot be run.
fn executable_of(
    result: &crate::build_command::TargetResult,
    layout: &fjfj_exec::execroot::Layout,
) -> Result<fjfj_graph::Artifact, CliError> {
    use std::os::unix::fs::PermissionsExt as _;
    if let Some(executable) = &result.target.executable {
        return Ok(executable.clone());
    }
    let files = result.target.files.to_vec();
    let [file] = &files[..] else {
        return Err(CliError::Internal(anyhow::anyhow!(
            "a target that is not an executable reached run"
        )));
    };
    let path = layout.resolve(file);
    let runnable =
        std::fs::metadata(&path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0);
    if runnable {
        Ok(file.clone())
    } else {
        Err(CliError::Build(anyhow::anyhow!(
            "Non-existent or non-executable {}",
            path.display()
        )))
    }
}

/// What the program does not inherit: the runfiles variables of whatever ran
/// `fjfj run`.
const UNSET: [&str; 5] = [
    "JAVA_RUNFILES",
    "RUNFILES_DIR",
    "RUNFILES_MANIFEST_FILE",
    "RUNFILES_MANIFEST_ONLY",
    "TEST_SRCDIR",
];

/// `--script_path`: the script is made executable, and `run` ends there
/// without running anything.
fn write_script(path: &std::path::Path, script: &str) -> Result<(), CliError> {
    use std::io::Write as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    let written = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o777)
        .open(path)
        .and_then(|mut f| f.write_all(script.as_bytes()));
    written.map_err(|e| {
        CliError::RunFailure(anyhow::anyhow!(
            "Error writing run script: {} ({})",
            std::env::current_dir()
                .unwrap_or_default()
                .join(path)
                .display(),
            os_message(&e)
        ))
    })
}

/// An I/O error as Java words it: no `(os error N)` suffix.
fn os_message(e: &std::io::Error) -> String {
    let text = e.to_string();
    match text.find(" (os error") {
        Some(at) => text[..at].to_owned(),
        None => text,
    }
}

/// An id for this invocation: a random-looking UUID.
fn build_id() -> String {
    use sha2::Digest as _;
    let seed = format!(
        "{:?}{}{}",
        std::time::SystemTime::now(),
        std::process::id(),
        std::env::args().collect::<Vec<_>>().join(" ")
    );
    let hash = sha2::Sha256::digest(seed.as_bytes());
    let h = hex::encode(&hash[..16]);
    format!(
        "{}-{}-{}-{}-{}",
        &h[..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..32]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt as _;

    #[test]
    fn a_script_is_written_executable_and_an_unwritable_path_is_a_run_failure() {
        let dir = std::env::temp_dir().join(format!("fjfj-run-script-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("s.sh");
        write_script(&path, "#!/bin/bash\ntrue").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "#!/bin/bash\ntrue");
        assert_ne!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o100,
            0
        );
        let missing = dir.join("no/such/s.sh");
        let Err(CliError::RunFailure(e)) = write_script(&missing, "x") else {
            panic!("written");
        };
        assert_eq!(
            e.to_string(),
            format!(
                "Error writing run script: {} (No such file or directory)",
                missing.display()
            )
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
