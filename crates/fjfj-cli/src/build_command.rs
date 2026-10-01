//! `fjfj build` past the loading phase (buildfiji-gwl.3, 136.1, fyz.1): the
//! targets are analysed, their actions run, and what was made is reported.

use fjfj_analysis::{ConfiguredTarget, ConfiguredTargetKey, Env, engine};
use fjfj_engine::Engine;
use fjfj_exec::execroot::{Layout, MAIN_REPO_DIR};
use fjfj_exec::run::{Failure, Progress, execute};
use fjfj_graph::{Action, Artifact, Configuration, Label};
use fjfj_repo::Repos;
use std::collections::{BTreeSet, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// What `build` was asked, past the patterns.
#[derive(Debug, Clone)]
pub(crate) struct Options {
    pub configuration: Configuration,
    pub keep_going: bool,
    pub symlink_prefix: String,
}

/// Where, and what.
pub(crate) struct Request {
    pub layout: Layout,
    pub options: Options,
}

/// One requested target and what building it made.
#[derive(Debug)]
pub(crate) struct TargetResult {
    pub label: Label,
    /// The files it built, as `bazel-bin/...` paths.
    pub files: Vec<String>,
    pub built: bool,
}

#[derive(Debug, Default)]
pub(crate) struct Report {
    /// What successful commands printed, with what each was doing.
    pub outputs: Vec<(String, String)>,
    pub results: Vec<TargetResult>,
    /// Analysis errors, one message each, with the target they stopped.
    pub analysis_errors: Vec<(Label, String)>,
    pub failures: Vec<Failure>,
    pub configured: usize,
    pub packages: usize,
    pub total_actions: usize,
    pub spawned: usize,
    pub elapsed: Duration,
    pub execution: Duration,
}

impl Report {
    pub fn succeeded(&self) -> bool {
        self.analysis_errors.is_empty() && self.failures.is_empty()
    }
}

fn label_text(label: &Label) -> String {
    if label.repo.is_empty() {
        format!("//{}:{}", label.package, label.name)
    } else {
        format!("@@{}//{}:{}", label.repo, label.package, label.name)
    }
}

/// Analyse `targets` and every target they read, in `configuration`.
async fn analyse(
    engine: &Engine,
    targets: &[Label],
    configuration: &Configuration,
    report: &mut Report,
) -> (
    Vec<(Label, Arc<ConfiguredTarget>)>,
    Vec<Arc<ConfiguredTarget>>,
) {
    let key = |label: &Label| ConfiguredTargetKey {
        label: label.clone(),
        configuration: configuration.clone(),
    };
    let mut roots = Vec::new();
    let results = futures::future::join_all(targets.iter().map(|t| engine.get(key(t)))).await;
    for (label, result) in targets.iter().zip(results) {
        match result {
            Ok(done) => roots.push((label.clone(), done)),
            Err(e) => report.analysis_errors.push((label.clone(), e.to_string())),
        }
    }
    // Everything they read, once each.
    let mut all: Vec<Arc<ConfiguredTarget>> = Vec::new();
    let mut seen: HashSet<ConfiguredTargetKey> = HashSet::new();
    let mut queue: Vec<Arc<ConfiguredTarget>> = roots.iter().map(|(_, t)| t.clone()).collect();
    while let Some(next) = queue.pop() {
        let me = ConfiguredTargetKey {
            label: next.label.clone(),
            configuration: next.configuration.clone(),
        };
        if !seen.insert(me) {
            continue;
        }
        for dep in &next.deps {
            if let Ok(done) = engine.get(dep.clone()).await {
                queue.push(done);
            }
        }
        all.push(next);
    }
    (roots, all)
}

/// Build `targets`. Blocking; run where a Tokio runtime is current.
pub(crate) fn run(repos: &Arc<Repos>, targets: &[Label], request: &Request) -> Report {
    let started = Instant::now();
    let mut report = Report::default();
    let analysis = engine(Env {
        source: repos.clone(),
        main_repo_name: MAIN_REPO_DIR.to_owned(),
    });
    let handle = tokio::runtime::Handle::current();
    let (roots, all) = handle.block_on(analyse(
        &analysis,
        targets,
        &request.options.configuration,
        &mut report,
    ));
    report.configured = all.iter().filter(|t| t.rule_class.is_some()).count();
    report.packages = all
        .iter()
        .map(|t| (t.label.repo.clone(), t.label.package.clone()))
        .collect::<BTreeSet<_>>()
        .len();
    if !report.analysis_errors.is_empty() && !request.options.keep_going {
        report.elapsed = started.elapsed();
        return report;
    }

    let actions: Vec<Action> = all.iter().flat_map(|t| t.actions.clone()).collect();
    report.total_actions = actions.len();
    let wanted: Vec<Artifact> = roots.iter().flat_map(|(_, t)| t.files.to_vec()).collect();
    if let Err(e) = request.layout.prepare() {
        report.analysis_errors.push((
            Label {
                repo: String::new(),
                package: String::new(),
                name: String::new(),
            },
            format!("cannot prepare the execution root: {e}"),
        ));
        return report;
    }
    let collector = Arc::new(Collector::default());
    let execution_started = Instant::now();
    let outcome = handle.block_on(execute(
        &request.layout,
        actions,
        &wanted,
        &fjfj_exec::run::Options {
            keep_going: request.options.keep_going,
            ..fjfj_exec::run::Options::default()
        },
        collector.clone(),
    ));
    report.execution = execution_started.elapsed();
    report.outputs = std::mem::take(&mut *collector.outputs.lock().unwrap());
    report.spawned = outcome.spawned;
    report.failures = outcome.failures;
    let prefix = &request.options.symlink_prefix;
    let _ = request
        .layout
        .convenience_links(prefix, &request.options.configuration.mnemonic());
    let by_key: std::collections::HashMap<ConfiguredTargetKey, &Arc<ConfiguredTarget>> = all
        .iter()
        .map(|t| {
            (
                ConfiguredTargetKey {
                    label: t.label.clone(),
                    configuration: t.configuration.clone(),
                },
                t,
            )
        })
        .collect();
    for (label, target) in &roots {
        // It was built unless one of the targets it needs failed.
        let mut needed: HashSet<String> = HashSet::new();
        let mut pending = vec![ConfiguredTargetKey {
            label: label.clone(),
            configuration: target.configuration.clone(),
        }];
        let mut visited: HashSet<ConfiguredTargetKey> = HashSet::new();
        while let Some(next) = pending.pop() {
            if !visited.insert(next.clone()) {
                continue;
            }
            if let Some(t) = by_key.get(&next) {
                needed.insert(label_text(&t.label));
                pending.extend(t.deps.iter().cloned());
            }
        }
        let built = report.failures.iter().all(|f| !needed.contains(&f.owner));
        let files = target
            .files
            .to_vec()
            .iter()
            .map(|a| shown(prefix, &request.options.configuration, a))
            .collect();
        report.results.push(TargetResult {
            label: label.clone(),
            files,
            built,
        });
    }
    report.elapsed = started.elapsed();
    report
}

/// A file as the console names it: `bazel-bin/pkg/f` for a derived one.
fn shown(prefix: &str, configuration: &Configuration, artifact: &Artifact) -> String {
    let bin = configuration.bin_dir();
    match artifact.exec_path().strip_prefix(&format!("{bin}/")) {
        Some(rest) => format!("{prefix}bin/{rest}"),
        None => artifact.exec_path(),
    }
}

/// Gathers what commands print, to say it when the build is done.
#[derive(Default)]
struct Collector {
    outputs: Mutex<Vec<(String, String)>>,
}

impl Progress for Collector {
    fn started(&self, _: &Action) {}

    fn output(&self, action: &Action, text: &str) {
        let what = action
            .progress_message
            .clone()
            .unwrap_or_else(|| format!("{} {}", action.mnemonic, label_text(&action.owner)));
        self.outputs.lock().unwrap().push((what, text.to_owned()));
    }
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Say what happened as `bazel build` does, on standard error. `Ok` if the
/// build succeeded.
pub(crate) fn print(
    report: &Report,
    requested: usize,
    keep_going: bool,
    layout: &Layout,
    verbose_failures: bool,
) -> bool {
    for (label, message) in &report.analysis_errors {
        eprintln!("ERROR: {message}");
        eprintln!(
            "ERROR: Analysis of target '{}' failed{}",
            label_text(label),
            if keep_going { "" } else { "; build aborted" }
        );
    }
    if report.analysis_errors.is_empty() || keep_going {
        let analysed = report.results.len();
        let what = if analysed == 1 {
            format!("target {}", label_text(&report.results[0].label))
        } else {
            plural(analysed, "target", "targets")
        };
        eprintln!(
            "INFO: Analyzed {what} ({}, {}).",
            plural(report.packages, "package loaded", "packages loaded"),
            plural(report.configured, "target configured", "targets configured"),
        );
    }
    for (what, text) in &report.outputs {
        eprintln!("INFO: From {what}:\n{}", text.trim_end());
    }
    let mut failed_owners: Vec<&str> = Vec::new();
    for failure in &report.failures {
        if failure.owner.is_empty() {
            continue;
        }
        let at = absolute(layout, &failure.repo, &failure.location);
        for detail in &failure.details {
            eprintln!("ERROR: {at}: {detail}");
        }
        eprintln!(
            "ERROR: {at}: {} failed: {}",
            failure.progress, failure.message
        );
        if !failure.output.is_empty() {
            eprint!("{}", failure.output);
        }
        if !failed_owners.contains(&failure.owner.as_str()) {
            failed_owners.push(&failure.owner);
        }
    }
    for owner in &failed_owners {
        eprintln!("Target {owner} failed to build");
    }
    if !failed_owners.is_empty() && !verbose_failures {
        eprintln!("Use --verbose_failures to see the command lines of failed build steps.");
    }
    let ok = report.succeeded();
    if ok || keep_going {
        let built: Vec<&TargetResult> = report.results.iter().filter(|r| r.built).collect();
        if !ok {
            eprintln!(
                "INFO: Build succeeded for only {} of {} top-level targets",
                built.len(),
                requested
            );
        }
        eprintln!("INFO: Found {}...", plural(requested, "target", "targets"));
        // `--show_result=1`: say where the result is when there is one target.
        if let ([only], 1) = (&built[..], report.results.len()) {
            eprintln!("Target {} up-to-date:", label_text(&only.label));
            for file in &only.files {
                eprintln!("  {file}");
            }
        }
    }
    eprintln!(
        "INFO: Elapsed time: {:.3}s, Critical Path: {:.2}s",
        report.elapsed.as_secs_f64(),
        report.execution.as_secs_f64()
    );
    if report.spawned == 0 {
        eprintln!("INFO: 0 processes.");
    } else {
        eprintln!(
            "INFO: {}: {} local.",
            plural(report.spawned, "process", "processes"),
            report.spawned
        );
    }
    if ok {
        eprintln!(
            "INFO: Build completed successfully, {}",
            plural(report.total_actions, "total action", "total actions")
        );
    } else {
        eprintln!("ERROR: Build did NOT complete successfully");
    }
    ok
}

/// A location as `bazel` prints it: the BUILD file's absolute path, then the
/// line and column.
fn absolute(layout: &Layout, repo: &str, location: &str) -> String {
    let root = if repo.is_empty() {
        layout.workspace.clone()
    } else {
        layout.external().join(repo)
    };
    root.join(location).display().to_string()
}
