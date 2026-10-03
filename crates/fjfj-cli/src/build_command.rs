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
    /// `--platforms`: the target platform, a label as written.
    pub platform: Option<String>,
    /// `--extra_toolchains`: patterns of toolchains to consider before the
    /// registered ones.
    pub extra_toolchains: Vec<String>,
    /// `--extra_execution_platforms`: patterns of platforms to try before the
    /// registered ones.
    pub extra_execution_platforms: Vec<String>,
    /// `--host_platform`: the platform fjfj runs on, a label as written.
    pub host_platform: Option<String>,
    /// `--aspects`: aspects to apply to the targets, `<bzl label>%<name>`.
    pub aspects: Vec<String>,
    /// `--output_groups`.
    pub output_groups: Vec<String>,
    pub keep_going: bool,
    /// `--nobuild`: stop after analysis, running no actions.
    pub build: bool,
    pub symlink_prefix: String,
    /// `--jobs`; the number of CPUs if unset.
    pub jobs: Option<usize>,
    /// `--spawn_strategy`: how commands are run.
    pub strategy: fjfj_exec::run::Strategy,
    /// `--show_result`: say where the results are for this many targets or fewer.
    pub show_result: usize,
    /// `test`: run the tests among the targets, and how much of their logs to show.
    pub test: Option<fjfj_bazel_compat::test_flags::TestOutput>,
}

/// The configuration the build flags ask for.
pub(crate) fn configuration_from(
    flags: &fjfj_bazel_compat::build_flags::BuildFlags,
) -> Result<Configuration, String> {
    let mut configuration = Configuration::default();
    if let Some(mode) = &flags.compilation_mode {
        configuration.compilation_mode = fjfj_graph::CompilationMode::parse(mode).ok_or_else(|| {
            format!(
                "While parsing option --compilation_mode={mode}: Invalid value '{mode}'; must be one of fastbuild, dbg, opt"
            )
        })?;
    }
    if let Some(cpu) = &flags.cpu {
        configuration.cpu = cpu.clone();
    }
    configuration.defines = flags.defines.iter().cloned().collect();
    for (name, value) in &flags.options {
        let entry = configuration.options.entry(name.clone()).or_default();
        if !entry.is_empty() {
            entry.push(' ');
        }
        entry.push_str(value);
    }
    for (flag, value) in &flags.starlark_flags {
        let label = fjfj_graph::Label::parse(
            flag,
            fjfj_graph::LabelContext {
                repo: "",
                package: "",
            },
        )
        .map_err(|e| format!("While parsing option --{flag}={value}: {e}"))?;
        configuration.settings.insert(
            fjfj_graph::expand::label_text(&label),
            fjfj_graph::SettingValue::Str(value.clone()),
        );
    }
    Ok(configuration)
}

/// `--test_env` and `--test_arg` into the configuration. `--test_env=NAME`
/// takes the value `NAME` has here.
pub(crate) fn apply_test_flags(
    configuration: &mut Configuration,
    flags: &fjfj_bazel_compat::test_flags::TestFlags,
) {
    for entry in &flags.env {
        match entry.split_once('=') {
            Some((name, value)) => {
                configuration
                    .test_env
                    .insert(name.to_owned(), value.to_owned());
            }
            None => {
                if let Ok(value) = std::env::var(entry) {
                    configuration.test_env.insert(entry.clone(), value);
                }
            }
        }
    }
    configuration.test_args = flags.args.clone();
}

/// `--jobs`: a number, `auto`, or `HOST_CPUS` with `*factor` or `-n`.
pub(crate) fn jobs_from(text: Option<&str>) -> Result<Option<usize>, String> {
    let Some(text) = text else { return Ok(None) };
    let cpus = std::thread::available_parallelism().map_or(1, |n| n.get());
    let bad = || {
        format!(
            "While parsing option --jobs={text}: '{text}' is not an integer or \"auto\" or HOST_CPUS[*factor|-n]"
        )
    };
    let jobs = if text == "auto" {
        cpus
    } else if let Some(rest) = text.strip_prefix("HOST_CPUS") {
        match rest {
            "" => cpus,
            r if r.starts_with('*') => {
                let factor: f64 = r[1..].parse().map_err(|_| bad())?;
                ((cpus as f64 * factor).floor() as usize).max(1)
            }
            r if r.starts_with('-') => {
                let n: usize = r[1..].parse().map_err(|_| bad())?;
                cpus.saturating_sub(n).max(1)
            }
            _ => return Err(bad()),
        }
    } else {
        text.parse().map_err(|_| bad())?
    };
    Ok(Some(jobs))
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
    /// What was analysed: its executable, its rule class, its runfiles.
    pub target: Arc<ConfiguredTarget>,
    /// The files it built, as `bazel-bin/...` paths.
    pub files: Vec<String>,
    pub built: bool,
}

/// How a test ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TestStatus {
    Passed,
    Failed(i32),
    Timeout,
    /// It did not run: something it needs failed to build.
    NoStatus,
}

/// A test that was run.
#[derive(Debug, Clone)]
pub(crate) struct TestResult {
    pub label: Label,
    pub status: TestStatus,
    pub took: Duration,
    /// The action did not run again: the last result still holds.
    pub cached: bool,
    /// Where `test.log` is on disk.
    pub log: std::path::PathBuf,
    pub size: String,
}

#[derive(Debug)]
pub(crate) struct Report {
    pub layout: Layout,
    pub tests: Vec<TestResult>,
    /// What successful commands printed, with what each was doing.
    pub outputs: Vec<(String, String)>,
    /// What rules `print()`ed while they were analysed.
    pub printed: Vec<String>,
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
    fn new(layout: Layout) -> Report {
        Report {
            layout,
            tests: Vec::new(),
            outputs: Vec::new(),
            printed: Vec::new(),
            results: Vec::new(),
            analysis_errors: Vec::new(),
            failures: Vec::new(),
            configured: 0,
            packages: 0,
            total_actions: 0,
            spawned: 0,
            elapsed: Duration::ZERO,
            execution: Duration::ZERO,
        }
    }

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

/// `--aspects` as aspects: `<bzl label>%<name>`, the label as the main
/// repository names repositories.
fn parse_aspects(repos: &Repos, specs: &[String]) -> Result<Vec<fjfj_starlark::AspectRef>, String> {
    use fjfj_starlark::RuleSource;
    let mappings = repos.mappings();
    specs
        .iter()
        .map(|spec| {
            let (bzl, name) = spec.rsplit_once('%').ok_or_else(|| {
                format!("Invalid aspect '{spec}': want <bzl label>%<aspect name>")
            })?;
            let bzl = Label::parse_mapped(
                bzl,
                fjfj_graph::LabelContext {
                    repo: "",
                    package: "",
                },
                &mut |apparent| mappings.resolve_apparent("", apparent),
            )
            .map_err(|e| format!("Invalid aspect '{spec}': {e}"))?;
            Ok(fjfj_starlark::AspectRef {
                bzl,
                name: name.to_owned(),
            })
        })
        .collect()
}

/// The output groups to build: `--output_groups` applied to the defaults.
fn output_groups(flags: &[String]) -> BTreeSet<String> {
    let mut groups: BTreeSet<String> = ["default", "_validation", "_hidden_top_level_INTERNAL_"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    // A name with no sign replaces the defaults, once.
    let mut replaced = false;
    for flag in flags {
        if let Some(name) = flag.strip_prefix('+') {
            groups.insert(name.to_owned());
        } else if let Some(name) = flag.strip_prefix('-') {
            groups.remove(name);
        } else {
            if !replaced {
                groups.clear();
                replaced = true;
            }
            groups.insert(flag.clone());
        }
    }
    groups
}

/// The files of `target` in the output groups `groups`: its `DefaultInfo`
/// files for `default`, the others from its `OutputGroupInfo`.
fn group_files(target: &ConfiguredTarget, groups: &BTreeSet<String>) -> Vec<Artifact> {
    let mut out = Vec::new();
    for group in groups {
        if group == "default" {
            out.extend(target.files.to_vec());
        } else {
            out.extend(fjfj_starlark::output_group_files(&target.providers, group));
        }
    }
    out
}

/// Analyse `targets` and every target they read, in `configuration`, and
/// `aspects` applied to each of them.
async fn analyse(
    engine: &Engine,
    targets: &[Label],
    aspects: &[fjfj_starlark::AspectRef],
    configuration: &Configuration,
    report: &mut Report,
) -> (
    Vec<(Label, Arc<ConfiguredTarget>)>,
    Vec<Arc<ConfiguredTarget>>,
    Vec<Arc<ConfiguredTarget>>,
) {
    let key = |label: &Label| ConfiguredTargetKey {
        label: label.clone(),
        configuration: configuration.clone(),
    };
    // Each root is its own pipeline: its aspects need only its own analysis,
    // so they start as soon as it finishes, not when the slowest root does.
    let pipelines = futures::future::join_all(targets.iter().map(|label| async move {
        let root = match engine.get(key(label)).await {
            Ok(done) => done,
            Err(e) => return (label, Err(e.to_string()), Vec::new()),
        };
        let applied = futures::future::join_all(aspects.iter().map(|aspect| {
            engine.get(fjfj_analysis::AspectKey {
                target: ConfiguredTargetKey {
                    label: root.label.clone(),
                    configuration: root.configuration.clone(),
                },
                aspect: aspect.clone(),
            })
        }))
        .await;
        (label, Ok(root), applied)
    }))
    .await;
    let mut roots = Vec::new();
    let mut aspect_roots: Vec<Arc<ConfiguredTarget>> = Vec::new();
    for (label, root, applied) in pipelines {
        match root {
            Ok(done) => roots.push((label.clone(), done)),
            Err(e) => report.analysis_errors.push((label.clone(), e)),
        }
        for result in applied {
            match result {
                Ok(done) => aspect_roots.push(done),
                Err(e) => report.analysis_errors.push((label.clone(), e.to_string())),
            }
        }
    }
    // Everything they read, once each.
    let mut all: Vec<Arc<ConfiguredTarget>> = Vec::new();
    // An engine hands out the same value for a key every time, so one is seen once.
    let mut seen: HashSet<*const ConfiguredTarget> = HashSet::new();
    let mut queue: Vec<Arc<ConfiguredTarget>> = roots
        .iter()
        .map(|(_, t)| t.clone())
        .chain(aspect_roots.iter().cloned())
        .collect();
    while let Some(next) = queue.pop() {
        if !seen.insert(Arc::as_ptr(&next)) {
            continue;
        }
        for dep in &next.deps {
            if let Ok(done) = engine.get(dep.clone()).await {
                queue.push(done);
            }
        }
        for aspect in &next.aspect_deps {
            if let Ok(done) = engine.get(aspect.clone()).await {
                queue.push(done);
            }
        }
        all.push(next);
    }
    (roots, aspect_roots, all)
}

/// The constraint values of the platform fjfj runs on: the machine's, as
/// `@platforms` names them.
fn host_constraints(repos: &Repos) -> Option<std::collections::BTreeSet<fjfj_graph::Label>> {
    let platforms = repos.module_repo("platforms")?;
    Some(
        fjfj_graph::config::host_constraints()
            .into_iter()
            .map(|(setting, value)| fjfj_graph::Label {
                repo: platforms.clone(),
                package: setting.to_owned(),
                name: value.to_owned(),
            })
            .collect(),
    )
}

/// Build `targets`. Blocking; run where a Tokio runtime is current.
pub(crate) fn run(repos: &Arc<Repos>, targets: &[Label], request: &Request) -> Report {
    let started = Instant::now();
    let mut report = Report::new(request.layout.clone());
    let env = |host_constraints| Env {
        source: repos.clone(),
        rules: repos.clone(),
        main_repo_name: MAIN_REPO_DIR.to_owned(),
        registered_toolchains: repos.registered_toolchains(),
        extra_toolchains: request.options.extra_toolchains.clone(),
        registered_execution_platforms: repos.registered_execution_platforms(),
        extra_execution_platforms: request.options.extra_execution_platforms.clone(),
        host_constraints,
    };
    let handle = tokio::runtime::Handle::current();
    // `--host_platform` says what the host is, in place of the machine's own.
    let mut host = host_constraints(repos);
    if let Some(text) = &request.options.host_platform {
        let named = fjfj_graph::Label::parse(
            text,
            fjfj_graph::LabelContext {
                repo: "",
                package: "",
            },
        )
        .map_err(|e| e.to_string())
        .and_then(|label| {
            handle.block_on(fjfj_analysis::platform_constraints(
                &engine(env(None)),
                &label,
            ))
        });
        match named {
            Ok(constraints) => host = Some(constraints),
            Err(message) => {
                report.analysis_errors.push((
                    fjfj_graph::Label {
                        repo: String::new(),
                        package: String::new(),
                        name: text.clone(),
                    },
                    message,
                ));
                report.elapsed = started.elapsed();
                return report;
            }
        }
    }
    let analysis = engine(env(host));
    let mut configuration = request.options.configuration.clone();
    if let Some(text) = &request.options.platform {
        let platform = fjfj_graph::Label::parse(
            text,
            fjfj_graph::LabelContext {
                repo: "",
                package: "",
            },
        )
        .map_err(|e| e.to_string())
        .and_then(|label| {
            handle
                .block_on(fjfj_analysis::platform_constraints(&analysis, &label))
                .map(|constraints| (label, constraints))
        });
        match platform {
            Ok((label, constraints)) => {
                configuration.constraints = constraints;
                configuration.settings.insert(
                    format!("{}platforms", fjfj_graph::config::COMMAND_LINE_OPTION),
                    fjfj_graph::SettingValue::List(vec![fjfj_graph::expand::label_text(&label)]),
                );
            }
            Err(message) => {
                report.analysis_errors.push((
                    fjfj_graph::Label {
                        repo: String::new(),
                        package: String::new(),
                        name: text.clone(),
                    },
                    message,
                ));
                report.elapsed = started.elapsed();
                return report;
            }
        }
    }
    let aspects = match parse_aspects(repos, &request.options.aspects) {
        Ok(aspects) => aspects,
        Err(message) => {
            report.analysis_errors.push((
                fjfj_graph::Label {
                    repo: String::new(),
                    package: String::new(),
                    name: String::new(),
                },
                message,
            ));
            report.elapsed = started.elapsed();
            return report;
        }
    };
    let (roots, aspect_roots, all) = handle.block_on(analyse(
        &analysis,
        targets,
        &aspects,
        &configuration,
        &mut report,
    ));
    report.printed = all.iter().flat_map(|t| t.printed.clone()).collect();
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

    let mut actions: Vec<Action> = all.iter().flat_map(|t| t.actions.clone()).collect();
    let mut wanted: Vec<Artifact> = roots
        .iter()
        .flat_map(|(_, t)| t.files.to_vec().into_iter().chain(t.extra_outputs.clone()))
        .collect();
    // The output groups besides the default one, of the targets and of the
    // aspects applied to them (`_validation` among them).
    let groups = output_groups(&request.options.output_groups);
    for target in roots.iter().map(|(_, t)| t).chain(&aspect_roots) {
        wanted.extend(group_files(target, &groups));
    }
    // The tests among the roots run too, with their logs among the results.
    let mut tests_to_run: Vec<(Label, fjfj_graph::TestInfo)> = Vec::new();
    if request.options.test.is_some() {
        for (label, target) in &roots {
            if let Some(test) = &target.test {
                actions.push(test.action.clone());
                wanted.extend(test.action.outputs.iter().cloned());
                tests_to_run.push((label.clone(), test.clone()));
            }
        }
    }
    if !request.options.build {
        actions.clear();
        wanted.clear();
        tests_to_run.clear();
    }
    report.total_actions = actions.len();
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
            jobs: request
                .options
                .jobs
                .unwrap_or_else(|| fjfj_exec::run::Options::default().jobs),
            strategy: request.options.strategy,
        },
        collector.clone(),
    ));
    report.execution = execution_started.elapsed();
    report.outputs = std::mem::take(&mut *collector.outputs.lock().unwrap());
    report.spawned = outcome.spawned;
    // A test that fails is a result; everything else that failed is an error.
    let (test_failures, failures): (Vec<_>, Vec<_>) = outcome
        .failures
        .into_iter()
        .partition(|f| f.mnemonic == "TestRunner");
    report.failures = failures;
    for (label, test) in &tests_to_run {
        let log = test.log.exec_path();
        let took = outcome
            .durations
            .iter()
            .find(|(path, _)| *path == log)
            .map(|(_, d)| *d)
            .unwrap_or_default();
        let cached = outcome.cached_outputs.contains(&log);
        let failure = test_failures.iter().find(|f| f.owner == label_text(label));
        let status = match failure {
            Some(f) if f.timed_out => TestStatus::Timeout,
            Some(f) => match f.exit_code {
                Some(code) => TestStatus::Failed(code),
                // It could not run; the build error says why.
                None => TestStatus::NoStatus,
            },
            None if request.layout.execroot().join(&log).exists() => TestStatus::Passed,
            None => TestStatus::NoStatus,
        };
        report.tests.push(TestResult {
            label: label.clone(),
            status,
            took,
            cached,
            log: request.layout.execroot().join(&log),
            size: test.size.clone(),
        });
    }
    report.tests.sort_by_key(|t| label_text(&t.label));
    let prefix = &request.options.symlink_prefix;
    let _ = request
        .layout
        .convenience_links(prefix, &request.options.configuration.mnemonic());
    // Which targets a root needs matters only once something has failed; the walk clones and
    // hashes a key per target, so a build that failed nothing does not make it.
    let by_key: std::collections::HashMap<ConfiguredTargetKey, &Arc<ConfiguredTarget>> =
        if report.failures.is_empty() {
            std::collections::HashMap::new()
        } else {
            all.iter()
                .map(|t| {
                    (
                        ConfiguredTargetKey {
                            label: t.label.clone(),
                            configuration: t.configuration.clone(),
                        },
                        t,
                    )
                })
                .collect()
        };
    for (label, target) in &roots {
        // It was built unless one of the targets it needs failed.
        let mut needed: HashSet<String> = HashSet::new();
        if !report.failures.is_empty() {
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
        }
        // With `--nobuild` nothing was built, so nothing is listed as up to date.
        let built =
            request.options.build && report.failures.iter().all(|f| !needed.contains(&f.owner));
        // Only what was built is listed, not the sources among a target's files.
        let files = target
            .files
            .to_vec()
            .iter()
            .filter(|a| !a.is_source())
            .map(|a| shown(prefix, &request.options.configuration, a))
            .collect();
        report.results.push(TargetResult {
            label: label.clone(),
            target: target.clone(),
            files,
            built,
        });
    }
    report.elapsed = started.elapsed();
    report
}

/// What the main repository is called in a runfiles tree.
pub(crate) const MAIN_REPO_NAME: &str = MAIN_REPO_DIR;

pub(crate) fn label_name(label: &Label) -> String {
    label_text(label)
}

/// A file as the console names it: `bazel-bin/pkg/f` for a derived one.
pub(crate) fn shown_path(
    prefix: &str,
    configuration: &Configuration,
    artifact: &Artifact,
) -> String {
    shown(prefix, configuration, artifact)
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
    show_result: usize,
    keep_going: bool,
    layout: &Layout,
    verbose_failures: bool,
    test_output: Option<fjfj_bazel_compat::test_flags::TestOutput>,
) -> bool {
    for text in &report.printed {
        eprintln!("DEBUG: {text}");
    }
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
    if let Some(mode) = test_output {
        print_test_output(report, mode);
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
        let tests = report.tests.len();
        let found = match (requested - tests.min(requested), tests) {
            (_, 0) => plural(requested, "target", "targets"),
            (0, t) => plural(t, "test target", "test targets"),
            (n, t) => format!(
                "{} and {}",
                plural(n, "target", "targets"),
                plural(t, "test target", "test targets")
            ),
        };
        eprintln!("INFO: Found {found}...");
        // `--show_result=1`: say where the result is when there is one target.
        if report.results.len() <= show_result {
            for result in &built {
                if result.files.is_empty() {
                    eprintln!(
                        "Target {} up-to-date (nothing to build)",
                        label_text(&result.label)
                    );
                    continue;
                }
                eprintln!("Target {} up-to-date:", label_text(&result.label));
                for file in &result.files {
                    eprintln!("  {file}");
                }
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
    let tests_failed = report
        .tests
        .iter()
        .filter(|t| t.status != TestStatus::Passed)
        .count();
    if ok && tests_failed > 0 {
        eprintln!(
            "INFO: Build completed, {} {}, {}",
            tests_failed,
            if tests_failed == 1 {
                "test FAILED"
            } else {
                "tests FAILED"
            },
            plural(report.total_actions, "total action", "total actions")
        );
    } else if ok {
        eprintln!(
            "INFO: Build completed successfully, {}",
            plural(report.total_actions, "total action", "total actions")
        );
    } else {
        eprintln!("ERROR: Build did NOT complete successfully");
    }
    if ok && test_output.is_some() {
        print_test_summary(report);
    }
    ok
}

/// What a failed test, and with `--test_output` a finished one, tells.
fn print_test_output(report: &Report, mode: fjfj_bazel_compat::test_flags::TestOutput) {
    use fjfj_bazel_compat::test_flags::TestOutput;
    for test in &report.tests {
        let failed = match &test.status {
            TestStatus::Passed => false,
            TestStatus::Failed(code) => {
                eprintln!(
                    "FAIL: {} (Exit {code}) (see {})",
                    label_text(&test.label),
                    test.log.display()
                );
                true
            }
            TestStatus::Timeout => {
                eprintln!(
                    "TIMEOUT: {} (see {})",
                    label_text(&test.label),
                    test.log.display()
                );
                true
            }
            TestStatus::NoStatus => continue,
        };
        let show = match mode {
            TestOutput::Summary => false,
            TestOutput::Errors => failed,
            TestOutput::All | TestOutput::Streamed => true,
        };
        if !show {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&test.log) else {
            continue;
        };
        // After the header test-setup.sh writes: the PAGER line, the test's
        // name, and a line of dashes.
        let body = match text.split_once(
            "-----------------------------------------------------------------------------\n",
        ) {
            Some((_, rest)) => rest,
            None => &text,
        };
        let label = label_text(&test.label);
        eprintln!("INFO: From Testing {label}:");
        eprintln!("==================== Test output for {label}:");
        eprint!("{body}");
        if !body.ends_with('\n') {
            eprintln!();
        }
        eprintln!(
            "================================================================================"
        );
    }
}

/// The table of results and the count under it.
fn print_test_summary(report: &Report) {
    let mut too_big = false;
    for test in &report.tests {
        let (status, cached) = match &test.status {
            TestStatus::Passed => ("PASSED", test.cached),
            TestStatus::Failed(_) => ("FAILED", false),
            TestStatus::Timeout => ("TIMEOUT", false),
            TestStatus::NoStatus => ("NO STATUS", false),
        };
        let label = label_text(&test.label);
        let prefix = if cached { "(cached) " } else { "" };
        let width = 73 - prefix.len();
        let took = if matches!(test.status, TestStatus::NoStatus) {
            String::new()
        } else {
            format!(" in {:.1}s", test.took.as_secs_f64())
        };
        eprintln!("{label:<width$}{prefix}{status}{took}");
        if !matches!(test.status, TestStatus::Passed | TestStatus::NoStatus) {
            eprintln!("  {}", test.log.display());
        }
        if test.status == TestStatus::Passed && is_too_big(test) {
            too_big = true;
        }
    }
    let ran = report.tests.iter().filter(|t| !t.cached).count();
    let total = report.tests.len();
    let passed = report
        .tests
        .iter()
        .filter(|t| t.status == TestStatus::Passed)
        .count();
    let failed = report
        .tests
        .iter()
        .filter(|t| !matches!(t.status, TestStatus::Passed | TestStatus::NoStatus))
        .count();
    let mut parts = Vec::new();
    if passed > 0 {
        parts.push(if passed == 1 {
            "1 test passes".to_owned()
        } else {
            format!("{passed} tests pass")
        });
    }
    if failed > 0 {
        parts.push(if failed == 1 {
            "1 fails locally".to_owned()
        } else {
            format!("{failed} fail locally")
        });
    }
    eprintln!();
    eprintln!(
        "Executed {ran} out of {}: {}.",
        plural(total, "test", "tests"),
        parts.join(" and ")
    );
    if too_big {
        eprintln!(
            "There were tests whose specified size is too big. Use the --test_verbose_timeout_warnings command line option to see which ones these are."
        );
    }
}

/// Whether a test that passed asked for more time than it needed: it would
/// have done within half the timeout of a smaller size.
fn is_too_big(test: &TestResult) -> bool {
    let smaller = match test.size.as_str() {
        "medium" => 60,
        "large" => 300,
        "enormous" => 900,
        _ => return false,
    };
    test.took.as_secs_f64() * 2.0 <= smaller as f64
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
