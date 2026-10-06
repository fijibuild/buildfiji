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
use crate::sandbox::Sandbox;
use crate::slots::Slots;
use fjfj_graph::{Action, ActionKind, Artifact};
use futures::future::{BoxFuture, FutureExt, Shared, join_all};
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tracing::Instrument;

#[derive(Debug, Clone)]
pub struct Options {
    /// Actions run at once (`--jobs`).
    pub jobs: usize,
    /// Run what can be run after a failure (`--keep_going`).
    pub keep_going: bool,
    /// How a command is run (`--spawn_strategy`).
    pub strategy: Strategy,
}

/// How a command is run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// In the execroot itself, where it sees everything there.
    Local,
    /// In a directory of its own that holds links to its declared inputs and
    /// nothing else (Bazel's `processwrapper-sandbox`); what it makes that
    /// is not a declared output is gone afterwards.
    Sandboxed,
    /// As `Sandboxed`, and also in new user, mount, pid and ipc namespaces
    /// with the file system read-only but for the sandbox directory and
    /// `/tmp` (Bazel's `linux-sandbox`), and without a network if the action
    /// asks (`block-network`). Where the host does not allow namespaces it
    /// is `Sandboxed`.
    LinuxSandbox,
}

impl Strategy {
    /// What Bazel's summary calls a process run this way.
    pub fn name(self) -> &'static str {
        match self {
            Strategy::Local => "local",
            Strategy::Sandboxed => "processwrapper-sandbox",
            Strategy::LinuxSandbox => "linux-sandbox",
        }
    }

    /// The strategy a `--spawn_strategy` list asks for: the first name that
    /// is one this runs.
    pub fn parse(list: &str) -> Result<Strategy, String> {
        for name in list.split(',') {
            match name.trim() {
                "local" | "standalone" => return Ok(Strategy::Local),
                "sandboxed" | "linux-sandbox" => return Ok(Strategy::LinuxSandbox),
                "processwrapper-sandbox" => return Ok(Strategy::Sandboxed),
                _ => {}
            }
        }
        Err(format!(
            "while parsing option --spawn_strategy={list}: none of the strategies is available"
        ))
    }
}

impl Default for Options {
    fn default() -> Options {
        Options {
            jobs: std::thread::available_parallelism().map_or(1, |n| n.get()),
            keep_going: false,
            strategy: Strategy::LinuxSandbox,
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
    /// How the command ended, if it ran and exited.
    pub exit_code: Option<i32>,
    /// It was killed for taking longer than its `timeout`.
    pub timed_out: bool,
    /// What the command printed.
    pub output: String,
}

/// What happened to a build's actions.
#[derive(Debug, Default)]
pub struct Outcome {
    /// Actions the requested outputs need: what makes them, and what that reads.
    pub closure: usize,
    /// The longest chain of actions that ran, by the time each took: what a
    /// build could not have gone faster than, however many jobs it had.
    pub critical_path: std::time::Duration,
    /// Actions that ran a command.
    pub spawned: usize,
    /// Actions of any kind that ran.
    pub ran: usize,
    /// Actions that did not need to run: their inputs were as the last run
    /// had them and their outputs were still there.
    pub cached: usize,
    pub failures: Vec<Failure>,
    /// How long each action that ran took, by its first output; an action that
    /// did not run has the time it took when it did.
    pub durations: Vec<(String, std::time::Duration)>,
    /// The first outputs of the actions that did not run.
    pub cached_outputs: Vec<String>,
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
    slots: Arc<Slots>,
    /// By action: the time the longest chain starting at it is expected to take.
    ranks: Vec<u64>,
    keep_going: bool,
    strategy: Strategy,
    sandboxes: AtomicUsize,
    stopped: AtomicBool,
    spawned: AtomicUsize,
    ran: AtomicUsize,
    cached: AtomicUsize,
    cache: ActionCache,
    failures: Mutex<Vec<Failure>>,
    durations: Mutex<Vec<(String, std::time::Duration)>>,
    cached_outputs: Mutex<Vec<String>>,
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
    // Beside the output bases, so a fresh one still has the durations.
    let history = layout
        .output_base
        .parent()
        .map(|root| root.join("cache").join("mnemonic-durations.json"));
    let cache =
        ActionCache::load_with_history(layout.output_base.join("fjfj-action-cache.json"), history);
    let ranks = ranks(&actions, &by_output, &cache);
    let scheduler = Arc::new(Scheduler {
        layout: layout.clone(),
        actions: actions.into_iter().map(Arc::new).collect(),
        by_output,
        memo: Mutex::new(HashMap::new()),
        slots: Slots::new(options.jobs.max(1)),
        ranks,
        keep_going: options.keep_going,
        strategy: options.strategy,
        sandboxes: AtomicUsize::new(0),
        stopped: AtomicBool::new(false),
        spawned: AtomicUsize::new(0),
        ran: AtomicUsize::new(0),
        cached: AtomicUsize::new(0),
        cache,
        failures: Mutex::new(Vec::new()),
        durations: Mutex::new(Vec::new()),
        cached_outputs: Mutex::new(Vec::new()),
        progress,
    });
    // A sandbox a killed build left is no use to this one.
    let _ = remove(&layout.output_base.join("sandbox"));
    let mut wanted: Vec<usize> = requested
        .iter()
        .filter_map(|a| scheduler.by_output.get(a).copied())
        .collect();
    wanted.sort_unstable();
    wanted.dedup();
    let closure = closure_size(&scheduler.actions, &scheduler.by_output, &wanted);
    let roots = wanted.clone();
    join_all(wanted.into_iter().map(|id| scheduler.run(id))).await;
    let _ = scheduler.cache.save();
    let _ = remove(&layout.output_base.join("sandbox"));
    let durations = std::mem::take(&mut *scheduler.durations.lock().unwrap());
    let cached_outputs = std::mem::take(&mut *scheduler.cached_outputs.lock().unwrap());
    let critical_path = critical_path(
        &scheduler.actions,
        &scheduler.by_output,
        &roots,
        &durations,
        &cached_outputs,
    );
    Outcome {
        closure,
        critical_path,
        spawned: scheduler.spawned.load(Ordering::Relaxed),
        ran: scheduler.ran.load(Ordering::Relaxed),
        cached: scheduler.cached.load(Ordering::Relaxed),
        failures: std::mem::take(&mut *scheduler.failures.lock().unwrap()),
        durations,
        cached_outputs,
    }
}

/// Whether the summary counts `action`. Bazel writes a parameter file as part
/// of the command that reads it, so it is not an action of its own there.
fn counted(action: &Action) -> bool {
    action.mnemonic != "ParameterFileWrite"
}

/// How many actions `roots` and everything they read come to.
fn closure_size(
    actions: &[Arc<Action>],
    by_output: &HashMap<Artifact, usize>,
    roots: &[usize],
) -> usize {
    let mut seen = vec![false; actions.len()];
    let mut stack: Vec<usize> = roots.to_vec();
    let mut count = 0;
    while let Some(id) = stack.pop() {
        if std::mem::replace(&mut seen[id], true) {
            continue;
        }
        count += usize::from(counted(&actions[id]));
        stack.extend(
            actions[id]
                .inputs
                .iter()
                .filter_map(|i| by_output.get(i).copied()),
        );
    }
    count
}

/// The longest chain, from the actions that make what was requested back
/// through what they read, of the time each action that ran took. An action
/// that did not run took no time.
fn critical_path(
    actions: &[Arc<Action>],
    by_output: &HashMap<Artifact, usize>,
    roots: &[usize],
    durations: &[(String, std::time::Duration)],
    cached_outputs: &[String],
) -> std::time::Duration {
    let cached: std::collections::HashSet<&str> =
        cached_outputs.iter().map(String::as_str).collect();
    let took: HashMap<&str, std::time::Duration> = durations
        .iter()
        .filter(|(path, _)| !cached.contains(path.as_str()))
        .map(|(path, d)| (path.as_str(), *d))
        .collect();
    let own = |id: usize| {
        actions[id]
            .outputs
            .first()
            .and_then(|first| took.get(first.exec_path().as_str()).copied())
            .unwrap_or_default()
    };
    let makers = |id: usize| {
        actions[id]
            .inputs
            .iter()
            .filter_map(|i| by_output.get(i).copied())
            .collect::<Vec<_>>()
    };
    // Post-order over the closure, so a chain is known once what it reads is.
    let mut chain: Vec<Option<std::time::Duration>> = vec![None; actions.len()];
    let mut stack: Vec<(usize, bool)> = roots.iter().map(|&r| (r, false)).collect();
    while let Some((id, visited)) = stack.pop() {
        if chain[id].is_some() {
            continue;
        }
        if visited {
            let behind = makers(id)
                .into_iter()
                .filter_map(|m| chain[m])
                .max()
                .unwrap_or_default();
            chain[id] = Some(own(id) + behind);
        } else {
            stack.push((id, true));
            stack.extend(
                makers(id)
                    .into_iter()
                    .filter(|&m| chain[m].is_none())
                    .map(|m| (m, false)),
            );
        }
    }
    roots
        .iter()
        .filter_map(|&r| chain[r])
        .max()
        .unwrap_or_default()
}

/// What an action not run before is expected to take, in microseconds: by
/// mnemonic, from what a cold build of this repository showed; actions that
/// run no command are near enough free.
const GUESS_US: u64 = 1_000_000;

fn guess_us(action: &Action) -> u64 {
    if !matches!(action.kind, ActionKind::Spawn { .. }) {
        return 10_000;
    }
    match action.mnemonic.as_str() {
        "Rustc" => 4 * GUESS_US,
        "CppCompile" | "CargoBuildScriptRun" | "CppLink" => 3 * GUESS_US,
        "Clippy" | "Rustfmt" => 2 * GUESS_US,
        _ => GUESS_US,
    }
}

/// For each action, how long the longest chain of actions that starts at it
/// is expected to take: its own time, as the last build had it or a guess,
/// plus the greatest of the chains of the actions that read its outputs.
/// An action that many others wait for has a greater rank than one nothing
/// waits for, and gets a slot first.
fn ranks(
    actions: &[Action],
    by_output: &HashMap<Artifact, usize>,
    cache: &ActionCache,
) -> Vec<u64> {
    let own: Vec<u64> = actions
        .iter()
        .map(|a| {
            cache
                .duration(a)
                .or_else(|| cache.mnemonic_mean(&a.mnemonic))
                .map_or_else(|| guess_us(a), |d| d.as_micros().max(1) as u64)
        })
        .collect();
    let mut readers: Vec<Vec<usize>> = vec![Vec::new(); actions.len()];
    let mut waiting_on = vec![0usize; actions.len()];
    for (id, action) in actions.iter().enumerate() {
        let mut makers: Vec<usize> = action
            .inputs
            .iter()
            .filter_map(|input| by_output.get(input).copied())
            .filter(|&maker| maker != id)
            .collect();
        makers.sort_unstable();
        makers.dedup();
        for maker in makers {
            readers[maker].push(id);
            waiting_on[id] += 1;
        }
    }
    // Topological order, makers first; the ranks then fill in from the end.
    let mut order: Vec<usize> = (0..actions.len()).filter(|&i| waiting_on[i] == 0).collect();
    let mut next = 0;
    while next < order.len() {
        let id = order[next];
        next += 1;
        for &reader in &readers[id] {
            waiting_on[reader] -= 1;
            if waiting_on[reader] == 0 {
                order.push(reader);
            }
        }
    }
    let mut ranks = own.clone();
    for &id in order.iter().rev() {
        let behind = readers[id].iter().map(|&r| ranks[r]).max().unwrap_or(0);
        ranks[id] = own[id].saturating_add(behind);
    }
    ranks
}

impl Scheduler {
    /// The action `id` having run, once.
    fn run(self: &Arc<Self>, id: usize) -> Done {
        let mut memo = self.memo.lock().unwrap();
        if let Some(done) = memo.get(&id) {
            return done.clone();
        }
        // A task of its own, so that one action's work does not wait on another's.
        let me = self.clone();
        let action = &self.actions[id];
        // A root: the action that asked for this one is not its parent.
        let span = tracing::info_span!(
            parent: None,
            "action",
            mnemonic = %action.mnemonic,
            target = %label_text(&action.owner),
            output = action.outputs.first().map(|o| o.exec_path()).unwrap_or_default(),
            cached = tracing::field::Empty,
            blocker = tracing::field::Empty,
        );
        let task = tokio::spawn(async move { me.run_now(id).await }.instrument(span));
        let done: Done = async move {
            task.await
                .unwrap_or_else(|e| Err(Arc::new(stray(format!("the action's task ended: {e}")))))
        }
        .boxed()
        .shared();
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
        // The dependency that finished last is what this action waited for.
        let finished = join_all(wait.into_iter().map(|other| {
            self.run(other)
                .map(move |result| (other, std::time::Instant::now(), result))
        }))
        .instrument(tracing::info_span!("step", step = "deps"))
        .await;
        if let Some((last, ..)) = finished.iter().max_by_key(|(_, at, _)| *at) {
            let blocker = self.actions[*last].outputs.first().map(|o| o.exec_path());
            tracing::Span::current().record("blocker", blocker.unwrap_or_default());
        }
        for (_, _, result) in finished {
            result?;
        }
        self.skip_if_stopped(&action)?;
        let (key, current) = {
            let (me, action) = (self.clone(), action.clone());
            blocking("key", move || {
                let execroot = me.layout.execroot();
                let key = me.cache.key(&execroot, &action);
                let current = key
                    .as_ref()
                    .is_some_and(|key| me.cache.is_current(&execroot, &action, key));
                (key, current)
            })
            .await
        };
        tracing::Span::current().record("cached", current);
        if current {
            if counted(&action) {
                self.cached.fetch_add(1, Ordering::Relaxed);
            }
            if let Some(first) = action.outputs.first() {
                let took = self.cache.duration(&action).unwrap_or_default();
                self.durations
                    .lock()
                    .unwrap()
                    .push((first.exec_path(), took));
                self.cached_outputs.lock().unwrap().push(first.exec_path());
            }
            return Ok(());
        }
        // Only a command holds a slot: an action that makes its outputs itself
        // runs at once, as Bazel's internal actions do.
        let _slot = if matches!(action.kind, ActionKind::Spawn { .. }) {
            let slot = self
                .slots
                .acquire(self.ranks[id])
                .instrument(tracing::info_span!("step", step = "slot"))
                .await;
            self.skip_if_stopped(&action)?;
            Some(slot)
        } else {
            None
        };
        self.progress.started(&action);
        let started = std::time::Instant::now();
        let outcome = self.execute_one(&action).await;
        if let Some(first) = action.outputs.first() {
            self.durations
                .lock()
                .unwrap()
                .push((first.exec_path(), started.elapsed()));
        }
        match outcome {
            Ok(()) => {
                if let Some(key) = key {
                    let (me, action, took) = (self.clone(), action.clone(), started.elapsed());
                    blocking("record", move || {
                        me.cache.record(&me.layout.execroot(), &action, key, took);
                    })
                    .await;
                }
                if counted(&action) {
                    self.ran.fetch_add(1, Ordering::Relaxed);
                }
                Ok(())
            }
            Err(failure) => {
                // A test that fails is a result, not a reason to stop.
                if action.mnemonic != "TestRunner" {
                    self.stopped.store(true, Ordering::Release);
                }
                self.failures.lock().unwrap().push((*failure).clone());
                Err(Arc::new(*failure))
            }
        }
    }

    fn skip_if_stopped(&self, action: &Action) -> Result<(), Arc<Failure>> {
        if self.stopped.load(Ordering::Acquire) && !self.keep_going {
            return Err(Arc::new(Failure {
                owner: String::new(),
                location: String::new(),
                repo: String::new(),
                progress: String::new(),
                mnemonic: action.mnemonic.clone(),
                message: "skipped after a failure".to_owned(),
                details: Vec::new(),
                exit_code: None,
                timed_out: false,
                output: String::new(),
            }));
        }
        Ok(())
    }

    /// `action` made, its blocking work on threads of their own and its command
    /// run without holding a thread.
    async fn execute_one(self: &Arc<Self>, action: &Arc<Action>) -> Result<(), Box<Failure>> {
        let (me, a) = (self.clone(), action.clone());
        let step = blocking("prepare", move || me.prepare(&a)).await?;
        let (me, a) = (self.clone(), action.clone());
        match step {
            Step::Done => blocking("make", move || me.outputs_made(&a)).await,
            Step::Run(pending) => {
                let Pending {
                    mut command,
                    sandbox,
                    limit,
                    started,
                } = *pending;
                let program = command
                    .as_std()
                    .get_program()
                    .to_string_lossy()
                    .into_owned();
                let fail = |message: String| Box::new(failure(action, message));
                // `spawn` forks and waits for the exec: not on a runtime thread.
                let child = blocking("spawn", move || {
                    command.spawn().map(|child| (child, command))
                })
                .await
                .map_err(|e| fail(format!("cannot run {program}: {e}")))?;
                let (child, _command) = child;
                let waited = child
                    .wait_with_output()
                    .instrument(tracing::info_span!("step", step = "command"));
                let output = match limit {
                    Some(limit) => match tokio::time::timeout(limit, waited).await {
                        Ok(done) => done,
                        Err(_) => {
                            return Err(Box::new(Failure {
                                timed_out: true,
                                ..*fail(format!(
                                    "(Timeout): killed after {} seconds",
                                    limit.as_secs()
                                ))
                            }));
                        }
                    },
                    None => waited.await,
                }
                .map_err(|e| fail("cannot run the command".to_owned() + &format!(": {e}")))?;
                blocking("collect", move || {
                    me.after_run(&a, sandbox, output, started)
                })
                .await
            }
        }
    }

    /// The work before a command runs, or all of an action that makes its
    /// outputs itself.
    /// The manifest of a runfiles tree: a line to each link and empty file, by
    /// path, the repo mapping among them, with absolute targets.
    fn manifest_text(
        &self,
        mapping_at: &Path,
        entries: &[(String, Artifact)],
        empty_files: &[String],
    ) -> String {
        // Sorted by path, an empty file with nothing after its path.
        let mut listed: Vec<(&str, String)> = entries
            .iter()
            .map(|(path, artifact)| {
                // A link that is not to be followed is listed as it reads.
                let unresolved =
                    self.by_output
                        .get(artifact)
                        .and_then(|&i| match &self.actions[i].kind {
                            ActionKind::UnresolvedSymlink { target } => Some(target.clone()),
                            _ => None,
                        });
                (
                    path.as_str(),
                    unresolved
                        .unwrap_or_else(|| self.layout.resolve(artifact).display().to_string()),
                )
            })
            .chain(
                empty_files
                    .iter()
                    .map(|path| (path.as_str(), String::new())),
            )
            .chain(std::iter::once((
                "_repo_mapping",
                mapping_at.display().to_string(),
            )))
            .collect();
        listed.sort();
        let mut lines = String::new();
        for (path, target) in &listed {
            // A path with a space, newline or backslash is escaped, and its
            // line starts with a space.
            if path.contains([' ', '\n', '\\']) {
                let escaped = path
                    .replace('\\', "\\b")
                    .replace(' ', "\\s")
                    .replace('\n', "\\n");
                lines.push_str(&format!(" {escaped} {target}\n"));
            } else {
                lines.push_str(&format!("{path} {target}\n"));
            }
        }
        lines
    }

    fn prepare(&self, action: &Action) -> Result<Step, Box<Failure>> {
        let fail = |message: String| Box::new(failure(action, message));
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
        // What a runfiles tree makes is not its own: it is what the `SymlinkTree` filled.
        let outputs: &[Artifact] = match action.kind {
            ActionKind::RunfilesTree => &[],
            _ => &action.outputs,
        };
        for out in outputs {
            let at = execroot.join(out.exec_path());
            remove(&at).map_err(|e| fail(format!("cannot remove {}: {e}", at.display())))?;
            if let Some(dir) = at.parent() {
                std::fs::create_dir_all(dir)
                    .map_err(|e| fail(format!("cannot create {}: {e}", dir.display())))?;
            }
            // A tree artifact exists, empty, before the action fills it.
            if out.tree {
                std::fs::create_dir_all(&at)
                    .map_err(|e| fail(format!("cannot create {}: {e}", at.display())))?;
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
            ActionKind::WorkspaceStatus { stable, volatile } => {
                for (out, contents) in action.outputs.iter().zip([stable, volatile]) {
                    let at = execroot.join(out.exec_path());
                    std::fs::write(&at, contents)
                        .map_err(|e| fail(format!("cannot write {}: {e}", at.display())))?;
                }
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
            ActionKind::SymlinkTree {
                dir,
                repo_mapping,
                entries,
                empty_files,
            } => {
                let mapping_at = execroot.join(repo_mapping);
                let lines = self.manifest_text(&mapping_at, entries, empty_files);
                // Not what an earlier run left: it may have linked more.
                let tree = execroot.join(dir);
                remove(&tree)
                    .map_err(|e| fail(format!("cannot remove {}: {e}", tree.display())))?;
                std::fs::create_dir_all(&tree)
                    .map_err(|e| fail(format!("cannot create {}: {e}", tree.display())))?;
                for (path, artifact) in entries {
                    let link = tree.join(path);
                    if let Some(parent) = link.parent() {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            fail(format!("cannot create {}: {e}", parent.display()))
                        })?;
                    }
                    std::os::unix::fs::symlink(self.layout.resolve(artifact), &link)
                        .map_err(|e| fail(format!("cannot link {}: {e}", link.display())))?;
                }
                for path in empty_files {
                    let file = tree.join(path);
                    if let Some(parent) = file.parent() {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            fail(format!("cannot create {}: {e}", parent.display()))
                        })?;
                    }
                    std::fs::write(&file, b"")
                        .map_err(|e| fail(format!("cannot write {}: {e}", file.display())))?;
                }
                std::fs::write(tree.join("MANIFEST"), lines).map_err(|e| {
                    fail(format!(
                        "cannot write the manifest in {}: {e}",
                        tree.display()
                    ))
                })?;
                std::os::unix::fs::symlink(&mapping_at, tree.join("_repo_mapping"))
                    .map_err(|e| fail(format!("cannot link in {}: {e}", tree.display())))?;
            }
            ActionKind::SourceManifest {
                repo_mapping,
                entries,
                empty_files,
            } => {
                let text = self.manifest_text(&execroot.join(repo_mapping), entries, empty_files);
                let at = execroot.join(action.outputs[0].exec_path());
                std::fs::write(&at, text)
                    .map_err(|e| fail(format!("cannot write {}: {e}", at.display())))?;
            }
            // The directory is the `SymlinkTree`'s: nothing is made.
            ActionKind::RunfilesTree => {}
            ActionKind::Symlink { target } => {
                let at = execroot.join(action.outputs[0].exec_path());
                std::os::unix::fs::symlink(execroot.join(target), &at)
                    .map_err(|e| fail(format!("cannot link {}: {e}", at.display())))?;
            }
            ActionKind::UnresolvedSymlink { target } => {
                let at = execroot.join(action.outputs[0].exec_path());
                std::os::unix::fs::symlink(target, &at)
                    .map_err(|e| fail(format!("cannot link {}: {e}", at.display())))?;
            }
            ActionKind::Spawn {
                argv,
                env,
                execution_requirements,
            } => {
                self.spawned.fetch_add(1, Ordering::Relaxed);
                let Some((program, args)) = argv.split_first() else {
                    return Err(fail("the command is empty".to_owned()));
                };
                let started = std::time::Instant::now();
                // A command that asks to runs in the execroot; a test asks
                // with its tags.
                let sandbox = if self.strategy != Strategy::Local
                    && !["local", "no-sandbox", "exclusive"]
                        .iter()
                        .any(|k| execution_requirements.contains_key(*k))
                {
                    let id = self.sandboxes.fetch_add(1, Ordering::Relaxed);
                    let root = self
                        .layout
                        .output_base
                        .join("sandbox/processwrapper-sandbox")
                        .join(id.to_string());
                    let sandbox = Sandbox {
                        exec: root.join("execroot").join(crate::execroot::MAIN_REPO_DIR),
                        root,
                    };
                    sandbox
                        .prepare(&self.layout, action)
                        .map_err(|e| fail(format!("cannot make the sandbox: {e}")))?;
                    Some(sandbox)
                } else {
                    None
                };
                let run_in = sandbox.as_ref().map_or(&execroot, |s| &s.exec);
                let isolation = (self.strategy == Strategy::LinuxSandbox
                    && sandbox.is_some()
                    && fjfj_sandbox::namespaces_available())
                .then(|| fjfj_sandbox::Isolation {
                    writable: sandbox.iter().map(|s| s.root.clone()).collect(),
                    block_network: execution_requirements.contains_key("block-network"),
                });
                let mut command = match &isolation {
                    Some(iso) => tokio::process::Command::from(
                        fjfj_sandbox::isolated(program, args, run_in, iso)
                            .map_err(|e| fail(format!("cannot isolate the command: {e}")))?,
                    ),
                    None => {
                        let mut command = tokio::process::Command::new(program);
                        command.args(args).current_dir(run_in);
                        command
                    }
                };
                command
                    .env_clear()
                    .envs(env)
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .kill_on_drop(true);
                let limit = execution_requirements
                    .get("timeout")
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(std::time::Duration::from_secs);
                return Ok(Step::Run(Box::new(Pending {
                    command,
                    sandbox,
                    limit,
                    started,
                })));
            }
        }
        Ok(Step::Done)
    }

    /// After a command ran: its outputs out of the sandbox, a test's log and
    /// XML, and its failure if it had one.
    fn after_run(
        &self,
        action: &Action,
        sandbox: Option<Sandbox>,
        output: std::process::Output,
        started: std::time::Instant,
    ) -> Result<(), Box<Failure>> {
        let fail = |message: String| Box::new(failure(action, message));
        let execroot = self.layout.execroot();
        let ActionKind::Spawn { argv, env, .. } = &action.kind else {
            return Ok(());
        };
        let program = argv.first().map_or("", String::as_str);
        if let Some(sandbox) = &sandbox {
            sandbox
                .collect(&self.layout, action)
                .map_err(|e| fail(format!("cannot take the outputs out of the sandbox: {e}")))?;
        }
        let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&output.stderr));
        // A test's output is its log, whatever became of it.
        if action.mnemonic == "TestRunner" {
            let log = execroot.join(action.outputs[0].exec_path());
            std::fs::write(&log, &text)
                .map_err(|e| fail(format!("cannot write {}: {e}", log.display())))?;
            text.clear();
            // Bazel writes the XML of a test that did not.
            let xml = execroot.join(action.outputs[1].exec_path());
            let missing = !xml.is_file() || std::fs::metadata(&xml).is_ok_and(|m| m.len() == 0);
            if missing
                && let Some(tool) = action
                    .inputs
                    .iter()
                    .find(|i| i.path.ends_with("tools/test/generate-xml.sh"))
            {
                let _ = std::process::Command::new("/bin/bash")
                    .arg(execroot.join(tool.exec_path()))
                    .arg(&log)
                    .arg(&xml)
                    .arg(started.elapsed().as_secs().to_string())
                    .arg(output.status.code().unwrap_or(1).to_string())
                    .current_dir(&execroot)
                    .env_clear()
                    .envs(env)
                    .output();
            }
        }
        if !output.status.success() {
            let how = match output.status.code() {
                Some(code) => format!("(Exit {code})"),
                None => "(Killed by a signal)".to_owned(),
            };
            return Err(Box::new(Failure {
                exit_code: output.status.code(),
                message: format!(
                    "{how}: {} failed: error executing {} command (from {} rule target {}) {}",
                    program.rsplit('/').next().unwrap_or(program),
                    action.mnemonic,
                    action.owner_kind,
                    label_text(&action.owner),
                    argv.iter().map(|a| quote(a)).collect::<Vec<_>>().join(" "),
                ),
                output: text,
                ..*fail(String::new())
            }));
        }
        if !text.is_empty() {
            self.progress.output(action, &text);
        }
        drop(sandbox);
        self.outputs_made(action)
    }

    /// Every output exists, and is protected as Bazel protects it.
    fn outputs_made(&self, action: &Action) -> Result<(), Box<Failure>> {
        let fail = |message: String| Box::new(failure(action, message));
        let execroot = self.layout.execroot();
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
                return Err(Box::new(Failure {
                    details: vec![detail],
                    ..*fail("not all outputs were created or valid".to_owned())
                }));
            }
            // Bazel writes a parameter file 0775, unlike its other outputs.
            let protected = if action.mnemonic == "ParameterFileWrite" {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o775))
            } else {
                make_read_only(&at, out.tree)
            };
            protected.map_err(|e| fail(format!("cannot protect {}: {e}", at.display())))?;
        }
        Ok(())
    }
}

/// What `prepare` leaves to do.
enum Step {
    /// The action made its outputs.
    Done,
    /// A command to run.
    Run(Box<Pending>),
}

struct Pending {
    command: tokio::process::Command,
    sandbox: Option<Sandbox>,
    limit: Option<std::time::Duration>,
    started: std::time::Instant,
}

/// `work` on a thread for blocking work, so the runtime's threads stay free.
/// The `step` span covers the wait for a thread of the blocking pool too.
async fn blocking<T: Send + 'static>(
    step: &'static str,
    work: impl FnOnce() -> T + Send + 'static,
) -> T {
    let span = tracing::info_span!(
        "step",
        step,
        queued_us = tracing::field::Empty,
        cpu_us = tracing::field::Empty
    );
    let requested = std::time::Instant::now();
    let run = move || {
        // What the step cost in wall time splits into the wait for a thread,
        // the thread's time on a CPU, and the rest (blocked, or not scheduled).
        span.record("queued_us", requested.elapsed().as_micros() as u64);
        let cpu = thread_cpu_us();
        let done = span.in_scope(work);
        span.record("cpu_us", thread_cpu_us().saturating_sub(cpu));
        done
    };
    match tokio::task::spawn_blocking(run).await {
        Ok(done) => done,
        Err(e) => std::panic::resume_unwind(e.into_panic()),
    }
}

/// The CPU time this thread has used, in microseconds.
fn thread_cpu_us() -> u64 {
    let mut at = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `at` is a valid timespec for the call to fill.
    unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut at) };
    at.tv_sec as u64 * 1_000_000 + at.tv_nsec as u64 / 1000
}

/// A failure of `action`.
fn failure(action: &Action, message: String) -> Failure {
    Failure {
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
        exit_code: None,
        timed_out: false,
        output: String::new(),
    }
}

/// A failure that belongs to no action.
fn stray(message: String) -> Failure {
    Failure {
        owner: String::new(),
        location: String::new(),
        repo: String::new(),
        progress: String::new(),
        mnemonic: String::new(),
        message,
        details: Vec::new(),
        exit_code: None,
        timed_out: false,
        output: String::new(),
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
            .all(|c| c.is_ascii_alphanumeric() || "-_./:,+@%".contains(c));
    if plain {
        arg.to_owned()
    } else {
        format!("'{}'", arg.replace('\'', "'\\''"))
    }
}

pub(crate) fn remove(at: &Path) -> std::io::Result<()> {
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

/// Bazel leaves a file output, and everything in a tree artifact, read-only and
/// executable (0555). Other directories, a runfiles tree among them, are not
/// touched.
fn make_read_only(at: &Path, tree: bool) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let meta = std::fs::symlink_metadata(at)?;
    if meta.file_type().is_symlink() || (meta.is_dir() && !tree) {
        return Ok(());
    }
    if meta.is_dir() {
        for entry in std::fs::read_dir(at)? {
            make_read_only(&entry?.path(), true)?;
        }
    }
    std::fs::set_permissions(at, std::fs::Permissions::from_mode(0o555))
}

#[cfg(test)]
mod tests;
