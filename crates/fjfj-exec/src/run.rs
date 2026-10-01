//! Running actions (buildfiji-fyz.1, 136.3): the scheduler, and the `local`
//! strategy.
//!
//! Each action runs once, after the actions that make its inputs, with at
//! most `jobs` of them running at a time. An action runs in the execroot with
//! the environment it names and nothing else. Its outputs are removed first,
//! must exist afterwards, and are made read-only as Bazel makes them.
//!
//! A failed action fails the actions that read its outputs. Without
//! `keep_going` it also stops the build from starting any other.

use crate::cache::ActionCache;
use crate::execroot::Layout;
use fjfj_graph::{Action, ActionKind, Artifact};
use futures::future::{BoxFuture, FutureExt, Shared, join_all};
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;

#[derive(Debug, Clone)]
pub struct Options {
    /// Actions run at once (`--jobs`).
    pub jobs: usize,
    /// Run what can be run after a failure (`--keep_going`).
    pub keep_going: bool,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            jobs: std::thread::available_parallelism().map_or(1, |n| n.get()),
            keep_going: false,
        }
    }
}

/// An action that did not give its outputs, in Bazel's words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    /// `//:g`, the target that registered the action.
    pub owner: String,
    /// Where it is declared, `a/BUILD.bazel:3:8`, and its repository.
    pub location: String,
    pub repo: String,
    /// What the console said the action was doing: `Executing genrule //:g`.
    pub progress: String,
    pub mnemonic: String,
    /// What went wrong: `(Exit 1): bash failed: ...`.
    pub message: String,
    /// More errors Bazel reports first, each its own `ERROR:` line.
    pub details: Vec<String>,
    /// What the command printed.
    pub output: String,
}

/// What happened to a build's actions.
#[derive(Debug, Default)]
pub struct Outcome {
    /// Actions that ran a command.
    pub spawned: usize,
    /// Actions of any kind that ran.
    pub ran: usize,
    /// Actions that did not need to run: their inputs were as the last run
    /// had them and their outputs were still there.
    pub cached: usize,
    pub failures: Vec<Failure>,
}

/// What the console is told as actions run.
pub trait Progress: Send + Sync {
    fn started(&self, action: &Action);
    /// A command printed this and succeeded.
    fn output(&self, action: &Action, text: &str);
}

/// Progress that nobody reads.
pub struct Quiet;

impl Progress for Quiet {
    fn started(&self, _: &Action) {}
    fn output(&self, _: &Action, _: &str) {}
}

type Done = Shared<BoxFuture<'static, Result<(), Arc<Failure>>>>;

struct Scheduler {
    layout: Layout,
    actions: Vec<Arc<Action>>,
    by_output: HashMap<Artifact, usize>,
    memo: Mutex<HashMap<usize, Done>>,
    slots: Semaphore,
    keep_going: bool,
    stopped: AtomicBool,
    spawned: AtomicUsize,
    ran: AtomicUsize,
    cached: AtomicUsize,
    cache: ActionCache,
    failures: Mutex<Vec<Failure>>,
    progress: Arc<dyn Progress>,
}

/// Make `requested`: run the actions that make them, and what those read.
pub async fn execute(
    layout: &Layout,
    actions: Vec<Action>,
    requested: &[Artifact],
    options: &Options,
    progress: Arc<dyn Progress>,
) -> Outcome {
    let mut by_output = HashMap::new();
    for (i, action) in actions.iter().enumerate() {
        for out in &action.outputs {
            by_output.insert(out.clone(), i);
        }
    }
    let scheduler = Arc::new(Scheduler {
        layout: layout.clone(),
        actions: actions.into_iter().map(Arc::new).collect(),
        by_output,
        memo: Mutex::new(HashMap::new()),
        slots: Semaphore::new(options.jobs.max(1)),
        keep_going: options.keep_going,
        stopped: AtomicBool::new(false),
        spawned: AtomicUsize::new(0),
        ran: AtomicUsize::new(0),
        cached: AtomicUsize::new(0),
        cache: ActionCache::load(layout.output_base.join("fjfj-action-cache.json")),
        failures: Mutex::new(Vec::new()),
        progress,
    });
    let mut wanted: Vec<usize> = requested
        .iter()
        .filter_map(|a| scheduler.by_output.get(a).copied())
        .collect();
    wanted.sort_unstable();
    wanted.dedup();
    join_all(wanted.into_iter().map(|id| scheduler.run(id))).await;
    let _ = scheduler.cache.save();
    Outcome {
        spawned: scheduler.spawned.load(Ordering::Relaxed),
        ran: scheduler.ran.load(Ordering::Relaxed),
        cached: scheduler.cached.load(Ordering::Relaxed),
        failures: std::mem::take(&mut *scheduler.failures.lock().unwrap()),
    }
}

impl Scheduler {
    /// The action `id` having run, once.
    fn run(self: &Arc<Self>, id: usize) -> Done {
        let mut memo = self.memo.lock().unwrap();
        if let Some(done) = memo.get(&id) {
            return done.clone();
        }
        let me = self.clone();
        let done: Done = async move { me.run_now(id).await }.boxed().shared();
        memo.insert(id, done.clone());
        done
    }

    async fn run_now(self: Arc<Self>, id: usize) -> Result<(), Arc<Failure>> {
        let action = self.actions[id].clone();
        let mut wait: Vec<usize> = action
            .inputs
            .iter()
            .filter_map(|input| self.by_output.get(input).copied())
            .filter(|&other| other != id)
            .collect();
        wait.sort_unstable();
        wait.dedup();
        for result in join_all(wait.into_iter().map(|other| self.run(other))).await {
            result?;
        }
        let _slot = self
            .slots
            .acquire()
            .await
            .expect("the semaphore stays open");
        if self.stopped.load(Ordering::Acquire) && !self.keep_going {
            return Err(Arc::new(Failure {
                owner: String::new(),
                location: String::new(),
                repo: String::new(),
                progress: String::new(),
                mnemonic: action.mnemonic.clone(),
                message: "skipped after a failure".to_owned(),
                details: Vec::new(),
                output: String::new(),
            }));
        }
        let execroot = self.layout.execroot();
        let key = self.cache.key(&execroot, &action);
        if let Some(key) = &key
            && self.cache.is_current(&execroot, &action, key)
        {
            self.cached.fetch_add(1, Ordering::Relaxed);
            return Ok(());
        }
        self.progress.started(&action);
        match self.execute_one(&action).await {
            Ok(()) => {
                if let Some(key) = key {
                    self.cache.record(&execroot, &action, key);
                }
                self.ran.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(failure) => {
                self.stopped.store(true, Ordering::Release);
                self.failures.lock().unwrap().push(failure.clone());
                Err(Arc::new(failure))
            }
        }
    }

    async fn execute_one(&self, action: &Action) -> Result<(), Failure> {
        let fail = |message: String| Failure {
            owner: label_text(&action.owner),
            location: action.location.clone(),
            repo: action.owner.repo.clone(),
            progress: action
                .progress_message
                .clone()
                .unwrap_or_else(|| format!("{} {}", action.mnemonic, label_text(&action.owner))),
            mnemonic: action.mnemonic.clone(),
            message,
            details: Vec::new(),
            output: String::new(),
        };
        let execroot = self.layout.execroot();
        for input in action.inputs.iter().filter(|a| a.is_source()) {
            let at = execroot.join(input.exec_path());
            if !at.exists() {
                return Err(fail(format!(
                    "missing input file '{}', owner: '{}'",
                    input.exec_path(),
                    label_text(&action.owner)
                )));
            }
        }
        for out in &action.outputs {
            let at = execroot.join(out.exec_path());
            remove(&at).map_err(|e| fail(format!("cannot remove {}: {e}", at.display())))?;
            if let Some(dir) = at.parent() {
                std::fs::create_dir_all(dir)
                    .map_err(|e| fail(format!("cannot create {}: {e}", dir.display())))?;
            }
        }
        match &action.kind {
            ActionKind::WriteFile {
                contents,
                executable,
            } => {
                let at = execroot.join(action.outputs[0].exec_path());
                std::fs::write(&at, contents)
                    .map_err(|e| fail(format!("cannot write {}: {e}", at.display())))?;
                let _ = executable;
            }
            ActionKind::Template {
                template,
                substitutions,
                executable,
            } => {
                let text = std::fs::read_to_string(execroot.join(template))
                    .map_err(|e| fail(format!("cannot read template {template}: {e}")))?;
                let mut text = text;
                for (key, value) in substitutions {
                    text = text.replace(key, value);
                }
                let at = execroot.join(action.outputs[0].exec_path());
                std::fs::write(&at, text)
                    .map_err(|e| fail(format!("cannot write {}: {e}", at.display())))?;
                let _ = executable;
            }
            ActionKind::Symlink { target } => {
                let at = execroot.join(action.outputs[0].exec_path());
                std::os::unix::fs::symlink(execroot.join(target), &at)
                    .map_err(|e| fail(format!("cannot link {}: {e}", at.display())))?;
            }
            ActionKind::Spawn { argv, env, .. } => {
                self.spawned.fetch_add(1, Ordering::Relaxed);
                let Some((program, args)) = argv.split_first() else {
                    return Err(fail("the command is empty".to_owned()));
                };
                let output = tokio::process::Command::new(program)
                    .args(args)
                    .current_dir(&execroot)
                    .env_clear()
                    .envs(env)
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .kill_on_drop(true)
                    .output()
                    .await
                    .map_err(|e| fail(format!("cannot run {program}: {e}")))?;
                let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
                text.push_str(&String::from_utf8_lossy(&output.stderr));
                if !output.status.success() {
                    let how = match output.status.code() {
                        Some(code) => format!("(Exit {code})"),
                        None => "(Killed by a signal)".to_owned(),
                    };
                    return Err(Failure {
                        message: format!(
                            "{how}: {} failed: error executing {} command (from {} rule target {}) {}",
                            program.rsplit('/').next().unwrap_or(program),
                            action.mnemonic,
                            action.owner_kind,
                            label_text(&action.owner),
                            argv.iter().map(|a| quote(a)).collect::<Vec<_>>().join(" "),
                        ),
                        output: text,
                        ..fail(String::new())
                    });
                }
                if !text.is_empty() {
                    self.progress.output(action, &text);
                }
            }
        }
        for out in &action.outputs {
            let at = execroot.join(out.exec_path());
            if std::fs::symlink_metadata(&at).is_err() {
                let detail = if action.mnemonic == "Genrule" {
                    format!(
                        "declared output '{}' was not created by genrule. This is probably because the genrule actually didn't create this output, or because the output was a directory and the genrule was run remotely (note that only the contents of declared file outputs are copied from genrules run remotely)",
                        out.path
                    )
                } else {
                    format!("output '{}' was not created", out.exec_path())
                };
                return Err(Failure {
                    details: vec![detail],
                    ..fail("not all outputs were created or valid".to_owned())
                });
            }
            make_read_only(&at)
                .map_err(|e| fail(format!("cannot protect {}: {e}", at.display())))?;
        }
        Ok(())
    }
}

fn label_text(label: &fjfj_graph::Label) -> String {
    if label.repo.is_empty() {
        format!("//{}:{}", label.package, label.name)
    } else {
        format!("@@{}//{}:{}", label.repo, label.package, label.name)
    }
}

/// An argument as a shell reads it back, quoted only when it needs it.
fn quote(arg: &str) -> String {
    let plain = !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./=:,+@%".contains(c));
    if plain {
        arg.to_owned()
    } else {
        format!("'{}'", arg.replace('\'', "'\\''"))
    }
}

fn remove(at: &Path) -> std::io::Result<()> {
    match std::fs::symlink_metadata(at) {
        Ok(meta) if meta.is_dir() => {
            // A directory output of an earlier run was made read-only.
            let _ = std::process::Command::new("chmod")
                .arg("-R")
                .arg("u+w")
                .arg(at)
                .status();
            std::fs::remove_dir_all(at)
        }
        Ok(_) => std::fs::remove_file(at),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

fn make_read_only(at: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let meta = std::fs::symlink_metadata(at)?;
    if meta.file_type().is_symlink() || meta.is_dir() {
        return Ok(());
    }
    std::fs::set_permissions(at, std::fs::Permissions::from_mode(0o555))
}

#[cfg(test)]
mod tests;
