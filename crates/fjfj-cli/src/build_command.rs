//! `fjfj build` past the loading phase (buildfiji-gwl.3, 136.1, fyz.1): the
//! targets are analysed, their actions run, and what was made is reported.

use fjfj_analysis::{ConfiguredTarget, ConfiguredTargetKey, Env, engine};
use fjfj_engine::Engine;
use fjfj_exec::execroot::{Layout, MAIN_REPO_DIR};
use fjfj_exec::run::{Failure, Progress, execute};
use fjfj_graph::{Action, Artifact, Configuration, Label};
use fjfj_repo::Repos;
use std::collections::{BTreeMap, BTreeSet, HashSet};
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
    /// `--toolchain_resolution_debug`: the regular expressions as written.
    pub toolchain_resolution_debug: Option<String>,
    /// The Starlark flags, `--//pkg:name=value`: the label as written, and the
    /// value.
    pub starlark_flags: Vec<(String, String)>,
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
    /// Decide the execution platform of every rule, for `aquery`.
    pub record_execution_platforms: bool,
    /// `test`: run the tests among the targets, and how much of their logs to show.
    pub test: Option<fjfj_bazel_compat::test_flags::TestOutput>,
    /// The contents of `stable-status.txt` and `volatile-status.txt`, for the
    /// action that writes them; none for a command that builds nothing.
    pub workspace_status: Option<(String, String)>,
    /// What to do with a target the platform cannot build; `None` leaves them
    /// in, as `cquery` and `aquery` show them.
    pub incompatible: Option<IncompatibleRoots>,
    /// `run`: every target asked for must be an executable, which Bazel
    /// finds out when it has analysed them, before it builds anything.
    pub run: bool,
}

/// How a build treats the targets it was asked for that its platform cannot
/// build: those a wildcard selected are skipped, and one named outright is an
/// error unless it is skipped too.
#[derive(Debug, Clone, Default)]
pub(crate) struct IncompatibleRoots {
    /// The targets a pattern names, not selects.
    pub explicit: BTreeSet<Label>,
    /// `--skip_incompatible_explicit_targets`.
    pub skip_explicit: bool,
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
    for entry in &flags.action_env {
        match entry.split_once('=') {
            Some((name, value)) => {
                configuration
                    .action_env
                    .insert(name.to_owned(), value.to_owned());
            }
            None => {
                if let Ok(value) = std::env::var(entry) {
                    configuration.action_env.insert(entry.clone(), value);
                }
            }
        }
    }
    for (name, value) in &flags.options {
        let entry = configuration.options.entry(name.clone()).or_default();
        if !entry.is_empty() {
            entry.push(' ');
        }
        entry.push_str(value);
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
    /// The platform cannot build it.
    Skipped,
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

/// The target an analysis error is about, which need not be the one asked for:
/// where its BUILD file declared it, and the configuration it was analysed in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Site {
    pub failing: Label,
    /// `pkg/BUILD:line:column`, in the repository of `failing`.
    pub location: String,
    /// The first digits of the configuration's checksum.
    pub config: String,
}

/// The target a message of analysis says it is about: the rule label in
/// `in <what> rule <label>: <why>`, which is how Bazel starts the error of a
/// rule, whichever target of the build that rule is the target of.
fn failing_target(message: &str) -> Option<Label> {
    let message = message
        .strip_prefix(fjfj_starlark::ATTRIBUTE_ERRORS)
        .unwrap_or(message);
    let after = message.strip_prefix("in ")?.split_once(" rule ")?.1;
    let text = after.split_once(": ")?.0;
    Label::parse(
        text,
        fjfj_graph::LabelContext {
            repo: "",
            package: "",
        },
    )
    .ok()
}

/// Where each analysis error of `report` happened.
fn place_errors(report: &mut Report, repos: &Repos, configuration: &Configuration) {
    let config = configuration.checksum()[..7].to_owned();
    for (label, message) in &report.analysis_errors {
        let failing = failing_target(message).unwrap_or_else(|| label.clone());
        let location = fjfj_loading::PackageSource::package(repos, &failing.repo, &failing.package)
            .ok()
            .and_then(|package| package.target(&failing.name).map(|t| t.location.clone()))
            .filter(|location| !location.is_empty());
        if let Some(location) = location {
            report.analysis_sites.insert(
                label.clone(),
                Site {
                    failing,
                    location,
                    config: config.clone(),
                },
            );
        }
    }
}

/// `--toolchain_resolution_debug`'s value as a test of a label: the
/// comma-separated regular expressions it finds, less those a `-` marks.
pub(crate) fn resolution_filter(text: &str) -> Result<fjfj_analysis::LabelFilter, String> {
    let (mut include, mut exclude) = (Vec::new(), Vec::new());
    for item in text.split(',') {
        match item.strip_prefix('-') {
            Some(rest) => exclude.push(regex::Regex::new(rest).map_err(|e| e.to_string())?),
            None => include.push(regex::Regex::new(item).map_err(|e| e.to_string())?),
        }
    }
    Ok(Arc::new(move |label| {
        (include.is_empty() || include.iter().any(|r| r.is_match(label)))
            && !exclude.iter().any(|r| r.is_match(label))
    }))
}

/// A Starlark flag that does not name a build setting, as Bazel says it.
#[derive(Debug, Clone)]
pub(crate) struct FlagError {
    /// Lines said before the error.
    pub before: Vec<String>,
    pub message: String,
}

/// The labels the Starlark flags name, read as the main repository sees them,
/// each with its value. Bazel loads the target and refuses one it cannot find.
pub(crate) fn resolve_starlark_flags(
    repos: &Repos,
    flags: &[(String, String)],
) -> Result<Vec<(String, Label, String)>, FlagError> {
    let mut found = Vec::new();
    for (name, value) in flags {
        let fail = |before: Vec<String>, why: &str| FlagError {
            before,
            message: format!("{name} :: Error loading option {name}: {why}"),
        };
        let unknown = std::cell::RefCell::new(None);
        let parsed = fjfj_graph::pattern::TargetPattern::parse(
            name,
            fjfj_graph::pattern::PatternContext {
                repo: "",
                offset: "",
            },
            &mut |apparent| match apparent {
                "" => String::new(),
                _ => repos.main_repo_canonical(apparent).unwrap_or_else(|| {
                    unknown.borrow_mut().get_or_insert(apparent.to_owned());
                    apparent.to_owned()
                }),
            },
        )
        .map_err(|e| fail(Vec::new(), &e.to_string()))?;
        if let Some(apparent) = unknown.into_inner() {
            return Err(fail(
                Vec::new(),
                &format!("No repository visible as '@{apparent}' from main repository"),
            ));
        }
        if let Some(written) = parsed
            .repo_written
            .as_deref()
            .and_then(|r| r.strip_prefix("@@"))
            && !written.is_empty()
            && written != "bazel_tools"
            && !repos.all_repos().iter().any(|r| r == written)
        {
            return Err(fail(
                Vec::new(),
                &format!("Repository '@@{written}' is not defined"),
            ));
        }
        let fjfj_graph::pattern::Pattern::Target(label) = &parsed.pattern else {
            return Err(FlagError {
                before: Vec::new(),
                message: format!("{name} :: Unrecognized option: {name}"),
            });
        };
        let resolved = fjfj_loading::resolve(std::slice::from_ref(&parsed), repos);
        if let Some(failure) = resolved.failures.first() {
            return Err(fail(
                vec![
                    "WARNING: Target pattern parsing failed.".to_owned(),
                    format!("ERROR: Skipping '{}': {}", failure.pattern, failure.message),
                ],
                &failure.message,
            ));
        }
        found.push((name.clone(), label.clone(), value.clone()));
    }
    Ok(found)
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
    /// Targets named outright that the platform cannot build, and why.
    pub incompatible_errors: Vec<(Label, String)>,
    /// The checksum of the configuration the targets were built in.
    pub configuration_checksum: String,
    /// What Bazel warns of for each target asked for that is a source file.
    pub source_file_warnings: Vec<String>,
    /// Those targets, which `--show_result` has nothing to say of.
    pub source_files: BTreeSet<Label>,
    /// `run`: the first target asked for that is not an executable.
    pub not_executable: Option<Label>,
    /// A Starlark flag that names nothing it can set.
    pub flag_error: Option<FlagError>,
    /// What the Starlark flags set, by the label that is read.
    pub starlark_settings: Vec<(String, fjfj_graph::SettingValue)>,
    /// The targets asked for whose analysis failed.
    pub failed_roots: Vec<Label>,
    /// Targets of the request left unbuilt as the platform cannot build them:
    /// they were analysed, and are not among the results.
    pub skipped: Vec<Label>,
    /// Where each of those stopped, for the targets that could be placed.
    pub analysis_sites: BTreeMap<Label, Site>,
    pub failures: Vec<Failure>,
    /// Every configured target analysis made, the roots and what they read.
    pub analysed: Vec<Arc<ConfiguredTarget>>,
    pub configured: usize,
    pub packages: usize,
    pub total_actions: usize,
    pub spawned: usize,
    /// Actions that ran in fjfj itself (a symlink, a written file).
    pub internal: usize,
    /// Actions whose outputs were as the last run left them.
    pub cache_hits: usize,
    /// What `spawned` ran under, as Bazel names it.
    pub strategy: &'static str,
    pub elapsed: Duration,
    pub execution: Duration,
    /// The longest chain of actions that ran, by the time each took.
    pub critical_path: Duration,
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
            incompatible_errors: Vec::new(),
            configuration_checksum: String::new(),
            source_file_warnings: Vec::new(),
            source_files: BTreeSet::new(),
            not_executable: None,
            flag_error: None,
            starlark_settings: Vec::new(),
            failed_roots: Vec::new(),
            skipped: Vec::new(),
            analysis_sites: BTreeMap::new(),
            failures: Vec::new(),
            analysed: Vec::new(),
            configured: 0,
            packages: 0,
            total_actions: 0,
            spawned: 0,
            internal: 0,
            cache_hits: 0,
            strategy: "linux-sandbox",
            elapsed: Duration::ZERO,
            execution: Duration::ZERO,
            critical_path: Duration::ZERO,
        }
    }

    pub fn succeeded(&self) -> bool {
        self.analysis_errors.is_empty()
            && self.incompatible_errors.is_empty()
            && self.failures.is_empty()
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
            Err(e) => {
                report.failed_roots.push(label.clone());
                report.analysis_errors.push((label.clone(), e));
            }
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

/// A requested target the platform cannot build.
struct SkippedRoot {
    label: Label,
    is_test: bool,
    /// What a target named outright says, which is an error.
    error: Option<String>,
}

/// `roots` without the targets the platform cannot build, and those.
fn split_incompatible(
    roots: Vec<(Label, Arc<ConfiguredTarget>)>,
    how: Option<&IncompatibleRoots>,
) -> (Vec<(Label, Arc<ConfiguredTarget>)>, Vec<SkippedRoot>) {
    let Some(how) = how else {
        return (roots, Vec::new());
    };
    let mut kept = Vec::new();
    let mut skipped = Vec::new();
    for (label, target) in roots {
        let Some(why) = &target.incompatible else {
            kept.push((label, target));
            continue;
        };
        let error = (how.explicit.contains(&label) && !how.skip_explicit)
            .then(|| incompatible_message(&label, why));
        skipped.push(SkippedRoot {
            is_test: target.test.is_some() || why.is_test,
            label,
            error,
        });
    }
    (kept, skipped)
}

/// What Bazel says of a target it was asked for and cannot build: the chain
/// of targets that led to the one the platform does not suit, and what
/// that one lacked.
fn incompatible_message(label: &Label, why: &fjfj_analysis::Incompatible) -> String {
    let mut chain = String::new();
    let platform = why
        .chain
        .first()
        .and_then(|key| {
            key.configuration
                .settings
                .get("//command_line_option:platforms")
        })
        .and_then(|value| match value {
            fjfj_graph::SettingValue::List(items) => items.first().cloned(),
            _ => None,
        })
        .unwrap_or_else(|| "@@platforms//host:host".to_owned());
    for (n, key) in why.chain.iter().enumerate() {
        chain.push_str(&format!(
            "\n    {} ({})",
            label_text(&key.label),
            &key.configuration.checksum()[..6]
        ));
        if n + 1 == why.chain.len() {
            let wanted: Vec<String> = why.unsatisfied.iter().map(label_text).collect();
            let wanted = match wanted.as_slice() {
                [one] => format!("constraint {one}"),
                many => format!("constraints [{}]", many.join(", ")),
            };
            chain.push_str(&format!(
                "   <-- target platform ({platform}) didn't satisfy {wanted}"
            ));
        }
    }
    format!(
        "Target {} is incompatible and cannot be built, but was explicitly requested.\nDependency chain:{chain}",
        label_text(label)
    )
}

/// Build `targets`. Blocking; run where a Tokio runtime is current.
pub(crate) fn run(repos: &Arc<Repos>, targets: &[Label], request: &Request) -> Report {
    let started = Instant::now();
    let mut report = Report::new(request.layout.clone());
    let resolution_debug = match request.options.toolchain_resolution_debug.as_deref() {
        None => None,
        Some(text) => match resolution_filter(text) {
            Ok(filter) => Some(fjfj_analysis::ResolutionDebug {
                matches: filter,
                emit: Arc::new(|message| eprintln!("{message}")),
            }),
            Err(why) => {
                report.flag_error = Some(FlagError {
                    before: Vec::new(),
                    message: format!(
                        "While parsing option --toolchain_resolution_debug={text}: Failed to build valid regular expression: {why}"
                    ),
                });
                report.elapsed = started.elapsed();
                return report;
            }
        },
    };
    let env = |host_constraints| Env {
        source: repos.clone(),
        rules: repos.clone(),
        main_repo_name: MAIN_REPO_DIR.to_owned(),
        registered_toolchains: repos.registered_toolchains(),
        extra_toolchains: request.options.extra_toolchains.clone(),
        registered_execution_platforms: repos.registered_execution_platforms(),
        extra_execution_platforms: request.options.extra_execution_platforms.clone(),
        host_constraints,
        record_execution_platforms: request.options.record_execution_platforms,
        toolchain_resolution_debug: resolution_debug.clone(),
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
    match resolve_starlark_flags(repos, &request.options.starlark_flags) {
        Ok(flags) => {
            for (name, label, value) in flags {
                let is_setting = handle
                    .block_on(analysis.get(fjfj_analysis::BuildSettingKey(label.clone())))
                    .is_ok_and(|found| *found);
                if !is_setting {
                    report.flag_error = Some(FlagError {
                        before: Vec::new(),
                        message: format!("{name} :: Unrecognized option: {name}"),
                    });
                    report.elapsed = started.elapsed();
                    return report;
                }
                let key = fjfj_graph::expand::label_text(&label);
                let value = fjfj_graph::SettingValue::Str(value);
                configuration.settings.insert(key.clone(), value.clone());
                report.starlark_settings.push((key, value));
            }
        }
        Err(error) => {
            report.flag_error = Some(error);
            report.elapsed = started.elapsed();
            return report;
        }
    }
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
    let (mut roots, skipped_roots) =
        split_incompatible(roots, request.options.incompatible.as_ref());
    let mut aspect_roots = aspect_roots;
    report.analysed = all.clone();
    // A source file is built by nothing; a source file that is executable
    // can still be run.
    let mut source_files: BTreeSet<Label> = BTreeSet::new();
    for (label, _) in &roots {
        let Ok(package) =
            fjfj_loading::PackageSource::package(&**repos, &label.repo, &label.package)
        else {
            continue;
        };
        if let Some(location) = package.source_file_location(&label.name) {
            source_files.insert(label.clone());
            report.source_file_warnings.push(format!(
                "WARNING: {}: {} is a source file, nothing will be built for it. If you want to build a target that consumes this file, try --compile_one_dependency",
                absolute(&request.layout, &label.repo, location),
                label_text(label)
            ));
        }
    }
    report.source_files = source_files.clone();
    if request.options.run
        && report.analysis_errors.is_empty()
        && let Some((label, _)) = roots
            .iter()
            .find(|(label, t)| t.executable.is_none() && !source_files.contains(label))
    {
        report.not_executable = Some(label.clone());
        report.elapsed = started.elapsed();
        return report;
    }
    let mut failed_prints: Vec<String> = Vec::new();
    for (_, message) in &mut report.analysis_errors {
        let (printed, rest) = fjfj_starlark::split_printed(message);
        let rest = rest.to_owned();
        for line in printed {
            if !failed_prints.contains(&line) {
                failed_prints.push(line);
            }
        }
        *message = rest;
    }
    report.configuration_checksum = configuration.checksum();
    place_errors(&mut report, repos, &configuration);
    report.printed = failed_prints
        .into_iter()
        .chain(all.iter().flat_map(|t| t.printed.clone()))
        .collect();
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
    let mut skipped_tests: Vec<Label> = Vec::new();
    for skipped in skipped_roots {
        report.skipped.push(skipped.label.clone());
        if skipped.is_test && request.options.test.is_some() && skipped.error.is_none() {
            skipped_tests.push(skipped.label.clone());
        }
        if let Some(message) = skipped.error {
            report.incompatible_errors.push((skipped.label, message));
        }
    }
    // The build of what was asked stops at the first of them, which leaves
    // only what every build does.
    let mut stopped_tests: Vec<(Label, String)> = Vec::new();
    if !report.incompatible_errors.is_empty() && !request.options.keep_going {
        // The tests among them are in the summary, with no status.
        if request.options.test.is_some() {
            stopped_tests.extend(roots.iter().filter_map(|(label, target)| {
                target
                    .test
                    .as_ref()
                    .map(|test| (label.clone(), test.size.clone()))
            }));
        }
        roots.clear();
        aspect_roots.clear();
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
    // Every build writes the workspace status, whatever it stamps.
    if let Some((stable, volatile)) = &request.options.workspace_status {
        let output = |name: &str| Artifact {
            root: fjfj_graph::Root::derived("bazel-out"),
            path: name.to_owned(),
            tree: false,
        };
        let outputs = vec![output("stable-status.txt"), output("volatile-status.txt")];
        wanted.push(outputs[0].clone());
        actions.push(Action {
            owner: Label {
                repo: String::new(),
                package: String::new(),
                name: String::new(),
            },
            owner_kind: String::new(),
            location: String::new(),
            configuration: String::new(),
            mnemonic: "BazelWorkspaceStatusAction".to_owned(),
            progress_message: None,
            kind: fjfj_graph::ActionKind::WorkspaceStatus {
                stable: stable.clone(),
                volatile: volatile.clone(),
            },
            inputs: Vec::new(),
            input_set: None,
            outputs,
            exec_group: None,
        });
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
    report.critical_path = outcome.critical_path;
    report.spawned = outcome.spawned;
    report.internal = outcome.ran.saturating_sub(outcome.spawned);
    report.cache_hits = outcome.cached;
    report.strategy = request.options.strategy.name();
    // Bazel counts the actions the requested targets need, not every one analysed.
    if request.options.build {
        report.total_actions = outcome.closure;
    }
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
    for (label, size) in stopped_tests {
        report.tests.push(TestResult {
            label,
            status: TestStatus::NoStatus,
            took: Duration::ZERO,
            cached: false,
            log: std::path::PathBuf::new(),
            size,
        });
    }
    for label in skipped_tests {
        report.tests.push(TestResult {
            label,
            status: TestStatus::Skipped,
            took: Duration::ZERO,
            cached: false,
            log: std::path::PathBuf::new(),
            size: String::new(),
        });
    }
    report.tests.sort_by_key(|t| label_text(&t.label));
    let prefix = &request.options.symlink_prefix;
    // Bazel makes no links when it builds nothing (`--nobuild`, `cquery`).
    if request.options.build
        && let Ok(failed) = request
            .layout
            .convenience_links(prefix, &request.options.configuration.mnemonic())
        && !failed.is_empty()
    {
        eprintln!(
            "WARNING: failed to create one or more convenience symlinks for prefix '{prefix}':\n{}",
            failed.join("\n")
        );
    }
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

/// The end of a command that stopped before building anything.
pub(crate) fn print_nothing_built(elapsed: Duration) {
    eprintln!("INFO: Elapsed time: {:.3}s", elapsed.as_secs_f64());
    eprintln!("INFO: 0 processes.");
    eprintln!("ERROR: Build did NOT complete successfully");
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

/// What Bazel says of the targets that did not analyse: for each failing
/// target, once, the error and `Analysis of target ... failed` at the place
/// its BUILD file declared it, then, for each target asked for and not built,
/// that the build was aborted.
fn analysis_error_lines(report: &Report, layout: &Layout, keep_going: bool) -> Vec<String> {
    let files =
        |name: &str| crate::fetch_command::file_path(name, &layout.workspace, &layout.external());
    let mut lines = Vec::new();
    let mut reported = HashSet::new();
    for (label, message) in &report.analysis_errors {
        let message = fjfj_starlark::absolute_files(message, &files);
        match report.analysis_sites.get(label) {
            Some(site) => {
                let at = absolute(layout, &site.failing.repo, &site.location);
                if reported.insert(site.failing.clone()) {
                    for event in fjfj_starlark::error_events(&message) {
                        lines.push(format!("ERROR: {at}: {event}"));
                    }
                    lines.push(format!(
                        "ERROR: {at}: Analysis of target '{}' (config: {}) failed",
                        label_text(&site.failing),
                        site.config
                    ));
                }
                if !keep_going {
                    lines.push(format!(
                        "ERROR: Analysis of target '{}' failed; build aborted{}",
                        label_text(label),
                        if site.failing == *label {
                            ""
                        } else {
                            ": Analysis failed"
                        }
                    ));
                }
            }
            None => {
                for event in fjfj_starlark::error_events(&message) {
                    lines.push(format!("ERROR: {event}"));
                }
                lines.push(format!(
                    "ERROR: Analysis of target '{}' failed{}",
                    label_text(label),
                    if keep_going { "" } else { "; build aborted" }
                ));
            }
        }
    }
    lines
}

/// Say what happened as `bazel build` does, on standard error. `Ok` if the
/// build succeeded.
pub(crate) fn print(
    report: &Report,
    requested: usize,
    show_result: usize,
    keep_going: bool,
    verbose_failures: bool,
    test_output: Option<fjfj_bazel_compat::test_flags::TestOutput>,
    pattern_errors: bool,
) -> bool {
    let layout = &report.layout;
    for text in &report.printed {
        eprintln!(
            "{}",
            crate::fetch_command::debug_line(text, &layout.workspace, &layout.external())
        );
    }
    for line in analysis_error_lines(report, layout, keep_going) {
        eprintln!("{line}");
    }
    let incompatible = !report.incompatible_errors.is_empty();
    // What the platform cannot build and what failed to analyse were
    // analysed as well.
    let analysed = report.results.len() + report.skipped.len() + report.failed_roots.len();
    if ((report.analysis_errors.is_empty() && !incompatible) || keep_going)
        && !(pattern_errors && analysed == 0)
    {
        let what = if analysed == 1 {
            let only = report
                .results
                .first()
                .map(|r| &r.label)
                .or(report.skipped.first())
                .or(report.failed_roots.first());
            only.map(|label| format!("target {}", label_text(label)))
                .unwrap_or_default()
        } else {
            plural(analysed, "target", "targets")
        };
        eprintln!(
            "INFO: Analyzed {what} ({}, {}).",
            plural(report.packages, "package loaded", "packages loaded"),
            plural(report.configured, "target configured", "targets configured"),
        );
        for warning in &report.source_file_warnings {
            eprintln!("{warning}");
        }
    }
    if keep_going {
        for (label, message) in &report.incompatible_errors {
            eprintln!(
                "WARNING: errors encountered while analyzing target '{}', it will not be built.\n{message}",
                label_text(label)
            );
        }
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
        match (&failure.command_block, verbose_failures) {
            (block, true) if !block.is_empty() => {
                // The command moves from the message to its own block.
                let message = failure
                    .message
                    .strip_suffix(&failure.command_text)
                    .unwrap_or(&failure.message);
                eprintln!(
                    "ERROR: {at}: {} failed: {message}{block}# Configuration: {}",
                    failure.progress, report.configuration_checksum
                );
                let platform = report
                    .analysed
                    .iter()
                    .find(|t| label_text(&t.label) == failure.owner)
                    .and_then(|t| t.execution_platform.as_ref())
                    .map_or_else(|| "@@platforms//host:host".to_owned(), label_text);
                eprintln!("# Execution platform: {platform}\n");
            }
            _ => eprintln!(
                "ERROR: {at}: {} failed: {}",
                failure.progress, failure.message
            ),
        }
        if report.strategy == "linux-sandbox"
            && failure.exit_code.is_some_and(|code| code != 0)
            && !failure.timed_out
        {
            eprintln!(
                "Use --sandbox_debug to see verbose messages from the sandbox and retain the sandbox build root for debugging"
            );
        }
        if !failure.output.is_empty() {
            eprint!("{}", failure.output);
        }
        if !failed_owners.contains(&failure.owner.as_str()) {
            failed_owners.push(&failure.owner);
        }
    }
    // Under `--keep_going` there is a line for the target only if it is the
    // only one asked for, and the hint comes after the count of targets.
    if !keep_going || requested == 1 {
        for owner in &failed_owners {
            eprintln!("Target {owner} failed to build");
        }
    }
    if !keep_going && !failed_owners.is_empty() && !verbose_failures {
        eprintln!("Use --verbose_failures to see the command lines of failed build steps.");
    }
    // A target named outright that cannot be built stops the build, and with
    // one target asked for it is the one that failed.
    if !keep_going && let Some((label, message)) = report.incompatible_errors.first() {
        if requested == 1 {
            eprintln!("Target {} failed to build", label_text(label));
        }
        eprintln!("Use --verbose_failures to see the command lines of failed build steps.");
        eprintln!(
            "ERROR: Analysis of target '{}' failed; build aborted: {message}",
            label_text(label)
        );
    }
    let ok = report.succeeded() && !pattern_errors;
    if ok || keep_going {
        let built: Vec<&TargetResult> = report.results.iter().filter(|r| r.built).collect();
        if !report.succeeded() && !built.is_empty() {
            eprintln!(
                "INFO: Build succeeded for only {} of {} top-level targets",
                built.len(),
                requested
            );
        }
        let tests = report.tests.len();
        let found = match (requested - tests.min(requested), tests) {
            (_, 0) if test_output.is_some() && requested == 0 => "0 test targets".to_owned(),
            (_, 0) => plural(requested, "target", "targets"),
            (0, t) => plural(t, "test target", "test targets"),
            (n, t) => format!(
                "{} and {}",
                plural(n, "target", "targets"),
                plural(t, "test target", "test targets")
            ),
        };
        eprintln!("INFO: Found {found}...");
        if keep_going && (incompatible || (!failed_owners.is_empty() && !verbose_failures)) {
            eprintln!("Use --verbose_failures to see the command lines of failed build steps.");
        }
        if keep_going && incompatible {
            eprintln!("ERROR: command succeeded, but not all targets were analyzed");
        }

        // `--show_result=1`: say where the result is when there is one target.
        if analysed - report.source_files.len().min(analysed) <= show_result {
            for result in built
                .iter()
                .filter(|r| !report.source_files.contains(&r.label))
            {
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
    if pattern_errors {
        eprintln!("ERROR: command succeeded, but there were errors parsing the target pattern");
    }
    eprintln!(
        "INFO: Elapsed time: {:.3}s, Critical Path: {:.2}s",
        report.elapsed.as_secs_f64(),
        report.critical_path.as_secs_f64()
    );
    // Bazel's summary: `N processes: H action cache hit, I internal, S linux-sandbox.`
    let processes = report.spawned + report.internal;
    if processes == 0 && report.cache_hits == 0 {
        eprintln!("INFO: 0 processes.");
    } else {
        let mut parts = Vec::new();
        if report.cache_hits > 0 {
            parts.push(format!("{} action cache hit", report.cache_hits));
        }
        if report.internal > 0 {
            parts.push(format!("{} internal", report.internal));
        }
        if report.spawned > 0 {
            parts.push(format!("{} {}", report.spawned, report.strategy));
        }
        eprintln!(
            "INFO: {}: {}.",
            plural(processes, "process", "processes"),
            parts.join(", ")
        );
    }
    let tests_failed = report
        .tests
        .iter()
        .filter(|t| !matches!(t.status, TestStatus::Passed | TestStatus::Skipped))
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
        if test_output.is_some() && report.tests.is_empty() && report.succeeded() {
            eprintln!("ERROR: No test targets were found, yet testing was requested");
        }
    }
    // What was built is tested, and the summary says so whatever else failed.
    if test_output.is_some() && !report.tests.is_empty() {
        print_test_summary(report);
        let none_failed = report
            .tests
            .iter()
            .all(|t| matches!(t.status, TestStatus::Passed | TestStatus::Skipped));
        if !ok && keep_going && none_failed {
            eprintln!("All tests passed but there were other errors during the build.");
            eprintln!();
        }
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
            TestStatus::NoStatus | TestStatus::Skipped => continue,
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
    for line in test_summary_lines(report) {
        eprintln!("{line}");
    }
}

fn test_summary_lines(report: &Report) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut too_big = false;
    // What was skipped comes after what ran.
    let skipped_last = report
        .tests
        .iter()
        .filter(|t| t.status != TestStatus::Skipped)
        .chain(
            report
                .tests
                .iter()
                .filter(|t| t.status == TestStatus::Skipped),
        );
    for test in skipped_last {
        let (status, cached) = match &test.status {
            TestStatus::Passed => ("PASSED", test.cached),
            TestStatus::Failed(_) => ("FAILED", false),
            TestStatus::Timeout => ("TIMEOUT", false),
            TestStatus::NoStatus => ("NO STATUS", false),
            TestStatus::Skipped => ("SKIPPED", false),
        };
        let label = label_text(&test.label);
        let prefix = if cached { "(cached) " } else { "" };
        // The status ends where PASSED does.
        let width = 73 - prefix.len() - status.len().saturating_sub(6);
        let took = if matches!(test.status, TestStatus::NoStatus | TestStatus::Skipped) {
            String::new()
        } else {
            format!(" in {:.1}s", test.took.as_secs_f64())
        };
        lines.push(format!("{label:<width$}{prefix}{status}{took}"));
        if !matches!(
            test.status,
            TestStatus::Passed | TestStatus::NoStatus | TestStatus::Skipped
        ) {
            lines.push(format!("  {}", test.log.display()));
        }
        if test.status == TestStatus::Passed && is_too_big(test) {
            too_big = true;
        }
    }
    // A test that never got a status was not run, as one skipped was not.
    let not_run = |t: &&TestResult| matches!(t.status, TestStatus::Skipped | TestStatus::NoStatus);
    let skipped = report.tests.iter().filter(not_run).count();
    let ran = report
        .tests
        .iter()
        .filter(|t| !t.cached && !not_run(t))
        .count();
    let total = report.tests.len();
    let passed = report
        .tests
        .iter()
        .filter(|t| t.status == TestStatus::Passed)
        .count();
    let failed = report
        .tests
        .iter()
        .filter(|t| {
            !matches!(
                t.status,
                TestStatus::Passed | TestStatus::NoStatus | TestStatus::Skipped
            )
        })
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
    if skipped > 0 {
        parts.push(if skipped == 1 {
            "1 was skipped".to_owned()
        } else {
            format!("{skipped} were skipped")
        });
    }
    lines.push(String::new());
    lines.push(format!(
        "Executed {ran} out of {}: {}.",
        plural(total, "test", "tests"),
        parts.join(" and ")
    ));
    if too_big {
        lines.push("There were tests whose specified size is too big. Use the --test_verbose_timeout_warnings command line option to see which ones these are.".to_owned());
    }
    lines
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

#[cfg(test)]
mod tests {
    use super::*;

    fn label(package: &str, name: &str) -> Label {
        Label {
            repo: String::new(),
            package: package.to_owned(),
            name: name.to_owned(),
        }
    }

    fn report(errors: &[(&str, &str)], sites: &[(&str, &str)]) -> Report {
        let mut report = Report::new(Layout {
            workspace: "ws".into(),
            output_base: "ob".into(),
        });
        for (name, message) in errors {
            report
                .analysis_errors
                .push((label("k", name), (*message).to_owned()));
        }
        for (name, failing) in sites {
            report.analysis_sites.insert(
                label("k", name),
                Site {
                    failing: label("k", failing),
                    location: "k/BUILD:1:8".to_owned(),
                    config: "a7a71fd".to_owned(),
                },
            );
        }
        report
    }

    #[test]
    fn the_target_of_a_rules_error_is_the_label_after_rule() {
        assert_eq!(
            failing_target("in cmd attribute of genrule rule //k:bad: $(nope) not defined"),
            Some(label("k", "bad"))
        );
        assert_eq!(
            failing_target("in r rule //i:t: \nTraceback (most recent call last):"),
            Some(label("i", "t"))
        );
        assert_eq!(failing_target("no such package 'p'"), None);
    }

    #[test]
    fn a_failing_target_is_reported_at_its_place_and_once() {
        let message = "in cmd attribute of genrule rule //k:bad: $(nope) not defined";
        let report = report(
            &[("bad", message), ("top", message)],
            &[("bad", "bad"), ("top", "bad")],
        );
        let lines = analysis_error_lines(&report, &report.layout, false);
        assert_eq!(
            lines,
            [
                "ERROR: ws/k/BUILD:1:8: in cmd attribute of genrule rule //k:bad: $(nope) not defined",
                "ERROR: ws/k/BUILD:1:8: Analysis of target '//k:bad' (config: a7a71fd) failed",
                "ERROR: Analysis of target '//k:bad' failed; build aborted",
                "ERROR: Analysis of target '//k:top' failed; build aborted: Analysis failed",
            ]
        );
    }

    #[test]
    fn the_errors_of_a_rules_attributes_are_one_event_each() {
        let message = format!(
            "{}in cmd attribute of r rule //k:bad: $(nope) not defined\nin cmd attribute of r rule //k:bad: unterminated $",
            fjfj_starlark::ATTRIBUTE_ERRORS
        );
        assert_eq!(failing_target(&message), Some(label("k", "bad")));
        let report = report(&[("top", &message)], &[("top", "bad")]);
        assert_eq!(
            analysis_error_lines(&report, &report.layout, true),
            [
                "ERROR: ws/k/BUILD:1:8: in cmd attribute of r rule //k:bad: $(nope) not defined",
                "ERROR: ws/k/BUILD:1:8: in cmd attribute of r rule //k:bad: unterminated $",
                "ERROR: ws/k/BUILD:1:8: Analysis of target '//k:bad' (config: a7a71fd) failed",
            ]
        );
    }

    #[test]
    fn an_error_with_no_place_is_shown_as_it_is() {
        let report = report(&[("x", "no platform")], &[]);
        assert_eq!(
            analysis_error_lines(&report, &report.layout, true),
            [
                "ERROR: no platform",
                "ERROR: Analysis of target '//k:x' failed"
            ]
        );
    }

    #[test]
    fn a_traceback_in_an_error_names_files_by_path() {
        let report = report(
            &[(
                "t",
                "in r rule //k:t: \nTraceback (most recent call last):\n\tFile \"@@//k:l.bzl\", line 2, column 9, in _impl",
            )],
            &[("t", "t")],
        );
        let lines = analysis_error_lines(&report, &report.layout, false);
        assert!(
            lines[0].contains("File \"ws/k/l.bzl\", line 2"),
            "{lines:?}"
        );
    }
}

#[cfg(test)]
mod incompatible_tests {
    use super::*;

    fn key(name: &str) -> ConfiguredTargetKey {
        ConfiguredTargetKey {
            label: Label {
                repo: String::new(),
                package: "p".into(),
                name: name.into(),
            },
            configuration: Configuration::default(),
        }
    }

    /// What `bazel build //p:chain2` said, but for the digits of the
    /// configuration, which are fjfj's own.
    #[test]
    fn the_chain_ends_at_the_target_that_lacked_the_constraints() {
        let why = fjfj_analysis::Incompatible {
            chain: vec![key("chain2"), key("c")],
            unsatisfied: vec![key("cv").label, key("cv2").label],
            is_test: false,
        };
        let digits = &key("c").configuration.checksum()[..6].to_owned();
        assert_eq!(
            incompatible_message(&key("chain2").label, &why),
            format!(
                "Target //p:chain2 is incompatible and cannot be built, but was explicitly requested.\nDependency chain:\n    //p:chain2 ({digits})\n    //p:c ({digits})   <-- target platform (@@platforms//host:host) didn't satisfy constraints [//p:cv, //p:cv2]"
            )
        );
        let one = fjfj_analysis::Incompatible {
            chain: vec![key("c")],
            unsatisfied: vec![key("cv").label],
            is_test: false,
        };
        assert!(
            incompatible_message(&key("c").label, &one)
                .ends_with("didn't satisfy constraint //p:cv")
        );
    }

    #[test]
    fn a_target_named_outright_is_an_error_and_one_a_wildcard_selected_is_skipped() {
        let why = Arc::new(fjfj_analysis::Incompatible {
            chain: vec![key("c")],
            unsatisfied: vec![key("cv").label],
            is_test: false,
        });
        let mut incompatible = ConfiguredTarget::new(&key("c"));
        incompatible.incompatible = Some(why);
        let incompatible = Arc::new(incompatible);
        let fine = Arc::new(ConfiguredTarget::new(&key("ok")));
        let roots = || {
            vec![
                (key("c").label, incompatible.clone()),
                (key("ok").label, fine.clone()),
            ]
        };
        let how = |named: &[&str], skip_explicit| IncompatibleRoots {
            explicit: named.iter().map(|n| key(n).label).collect(),
            skip_explicit,
        };
        let (kept, skipped) = split_incompatible(roots(), Some(&how(&["c"], false)));
        assert_eq!(kept.len(), 1);
        assert!(
            skipped[0]
                .error
                .as_ref()
                .unwrap()
                .contains("explicitly requested")
        );
        let (_, skipped) = split_incompatible(roots(), Some(&how(&["c"], true)));
        assert!(skipped[0].error.is_none());
        let (_, skipped) = split_incompatible(roots(), Some(&how(&[], false)));
        assert!(skipped[0].error.is_none());
        let (kept, skipped) = split_incompatible(roots(), None);
        assert_eq!((kept.len(), skipped.len()), (2, 0));
    }

    /// `bazel test //p:t //p:t_ok` stopped by an incompatible //p:t: the test
    /// that did not run has no status and counts among the skipped.
    #[test]
    fn a_test_that_never_ran_has_no_status_and_was_skipped() {
        let mut report = Report::new(Layout {
            workspace: "ws".into(),
            output_base: "ob".into(),
        });
        let result = |name: &str, status| TestResult {
            label: key(name).label,
            status,
            took: Duration::ZERO,
            cached: false,
            log: std::path::PathBuf::new(),
            size: String::new(),
        };
        report.tests.push(result("t1", TestStatus::NoStatus));
        report.tests.push(result("t2", TestStatus::Passed));
        let lines = test_summary_lines(&report);
        assert_eq!(
            lines[0],
            "//p:t1                                                                NO STATUS"
        );
        assert_eq!(
            lines.last().unwrap(),
            "Executed 1 out of 2 tests: 1 test passes and 1 was skipped."
        );
    }
}
