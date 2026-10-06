//! `fjfj fetch`: makes external repositories (buildfiji-mum.12.2), which is
//! `fjfj-bzlmod`'s resolution, `fjfj-repo`'s extensions and repository rules, and
//! the lockfile, joined to Bazel's flags.
//!
//! Bazel 9.2.0's `fetch --repo=@name` makes that repository (and what it needs),
//! `--all` every repository of the module graph. A repository that cannot be
//! made is exit code 8. `--repository_cache` (empty: none) and `--distdir`,
//! `--override_repository=name=path`, `--credential_helper=[scope=]program` and
//! `--credential_helper_timeout` are read as Bazel reads them, the message of
//! a bad one being `While parsing option --flag=value: why`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use fjfj_bazel_compat::bzlmod_flags::BzlmodFlags;
use fjfj_bzlmod::Resolution;
use fjfj_bzlmod::lockfile::LockSession;
use fjfj_graph::pattern::{PatternContext, TargetPattern};
use fjfj_repo::{
    CredentialHelper, CredentialHelpers, HttpDownloader, Options, Repos, module_extensions_json,
};
use sha2::Digest as _;

use crate::{CliError, bzlmod_registries};

/// Flag names this command reads, for `clap_flags::validate`.
pub(crate) const IMPLEMENTED: &[&str] = &[
    "repo",
    "all",
    "repository_cache",
    "distdir",
    "override_repository",
    "credential_helper",
    "credential_helper_timeout",
    "experimental_isolated_extension_usages",
];

/// The flags `build` shares with `fetch`: where repositories come from.
pub(crate) const BUILD_IMPLEMENTED: &[&str] = &[
    "repository_cache",
    "distdir",
    "override_repository",
    "credential_helper",
    "credential_helper_timeout",
    "experimental_isolated_extension_usages",
];

/// What `fetch` was asked for.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct FetchFlags {
    pub repos: Vec<String>,
    pub all: bool,
    /// Where to work; Bazel's startup option, taken here until fjfj has them.
    pub output_base: Option<PathBuf>,
    /// `None`: the default place; `Some(None)`: `--repository_cache=`, none.
    pub repository_cache: Option<Option<PathBuf>>,
    pub distdirs: Vec<PathBuf>,
    pub repo_overrides: Vec<(String, PathBuf)>,
    pub credential_helpers: Vec<CredentialHelper>,
    pub credential_helper_timeout: Option<Duration>,
    pub isolated_extension_usages: bool,
}

fn bad(flag: &str, value: &str, why: impl std::fmt::Display) -> CliError {
    CliError::CommandLine(anyhow::anyhow!(
        "While parsing option --{flag}={value}: {why}"
    ))
}

/// Takes the fetch flags out of `args`; what is left goes on to be validated.
pub(crate) fn extract(args: &[String]) -> Result<(FetchFlags, Vec<String>), CliError> {
    let mut flags = FetchFlags::default();
    let mut rest = Vec::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let Some(body) = arg.strip_prefix("--") else {
            rest.push(arg.clone());
            continue;
        };
        let (name, inline) = match body.split_once('=') {
            Some((n, v)) => (n, Some(v.to_owned())),
            None => (body, None),
        };
        let value = |iter: &mut std::slice::Iter<'_, String>| {
            inline
                .clone()
                .or_else(|| iter.next().cloned())
                .unwrap_or_default()
        };
        match name {
            "repo" => flags.repos.push(value(&mut iter)),
            "all" => flags.all = true,
            "noall" => flags.all = false,
            "output_base" => flags.output_base = Some(PathBuf::from(value(&mut iter))),
            "repository_cache" => {
                let v = value(&mut iter);
                flags.repository_cache = Some((!v.is_empty()).then(|| PathBuf::from(v)));
            }
            "distdir" => flags.distdirs.push(PathBuf::from(value(&mut iter))),
            "override_repository" => {
                let v = value(&mut iter);
                let Some((name, path)) = v.split_once('=') else {
                    return Err(bad(
                        name,
                        &v,
                        "Repository overrides must be of the form 'repository-name=path'",
                    ));
                };
                flags
                    .repo_overrides
                    .push((name.to_owned(), PathBuf::from(path)));
            }
            "credential_helper" => {
                let v = value(&mut iter);
                flags
                    .credential_helpers
                    .push(CredentialHelper::parse(&v).map_err(|why| bad(name, &v, why))?);
            }
            "credential_helper_timeout" => {
                let v = value(&mut iter);
                flags.credential_helper_timeout = Some(parse_duration(&v).ok_or_else(|| {
                    bad(name, &v, "expected a duration such as 10s, 500ms or 2m")
                })?);
            }
            "experimental_isolated_extension_usages" => flags.isolated_extension_usages = true,
            "noexperimental_isolated_extension_usages" => flags.isolated_extension_usages = false,
            _ => rest.push(arg.clone()),
        }
    }
    Ok((flags, rest))
}

fn parse_duration(text: &str) -> Option<Duration> {
    let split = text.find(|c: char| !c.is_ascii_digit())?;
    let (number, unit) = text.split_at(split);
    let n: u64 = number.parse().ok()?;
    match unit {
        "ms" => Some(Duration::from_millis(n)),
        "s" => Some(Duration::from_secs(n)),
        "m" => Some(Duration::from_secs(n * 60)),
        "h" => Some(Duration::from_secs(n * 3600)),
        _ => None,
    }
}

/// The repository cache when `--repository_cache` is not given.
pub(crate) fn default_repository_cache() -> PathBuf {
    output_user_root().join("cache").join("repos").join("v1")
}

/// The startup options that say where fjfj works, set once by `run`.
static STARTUP: std::sync::OnceLock<(Option<PathBuf>, Option<PathBuf>)> =
    std::sync::OnceLock::new();

/// Records `--output_base` and `--output_user_root`, which come before the
/// command.
pub(crate) fn set_startup(output_base: Option<PathBuf>, output_user_root: Option<PathBuf>) {
    let _ = STARTUP.set((output_base, output_user_root));
}

/// `--output_user_root`, else `$XDG_CACHE_HOME` or `~/.cache`, then
/// `fjfj/_fjfj_<user>`.
fn output_user_root() -> PathBuf {
    if let Some(root) = STARTUP.get().and_then(|(_, root)| root.clone()) {
        return root;
    }
    let cache = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .unwrap_or_else(std::env::temp_dir);
    let user = std::env::var("USER").unwrap_or_else(|_| "user".to_owned());
    cache.join("fjfj").join(format!("_fjfj_{user}"))
}

/// The output base of a workspace: a directory of the output user root named by
/// the workspace's path.
pub(crate) fn default_output_base(workspace_root: &Path) -> PathBuf {
    if let Some(base) = STARTUP.get().and_then(|(base, _)| base.clone()) {
        return base;
    }
    let digest = sha2::Sha256::digest(workspace_root.display().to_string().as_bytes());
    output_user_root().join(&hex::encode(digest)[..32])
}

/// Prints what extensions and rules `print` as Bazel does, to stderr, with the
/// file of its call site as a path.
struct Printer {
    workspace_root: PathBuf,
    external: PathBuf,
}

impl Printer {
    fn of(repos: &Repos) -> Printer {
        Printer {
            workspace_root: repos.workspace_root().to_owned(),
            external: repos.output_base().join("external"),
        }
    }
}

impl starlark::PrintHandler for Printer {
    fn println(&self, text: &str) -> starlark::Result<()> {
        eprintln!("{}", debug_line(text, &self.workspace_root, &self.external));
        Ok(())
    }
}

/// Where the file `name` is: a canonical `@@<repo>//<package>:<file>` is in
/// the workspace or under `external`, and anything else is as it was.
pub(crate) fn file_path(name: &str, workspace_root: &Path, external: &Path) -> String {
    let located = name.strip_prefix("@@").and_then(|rest| {
        let (repo, rest) = rest.split_once("//")?;
        let (package, file) = rest.split_once(':')?;
        let root = if repo.is_empty() {
            workspace_root.to_owned()
        } else {
            external.join(repo)
        };
        Some(root.join(package).join(file).display().to_string())
    });
    located.unwrap_or_else(|| name.to_owned())
}

/// `DEBUG: <path>:<line>:<column>: <text>`, from what `print_line` made:
/// `@@<repo>//<package>:<file>:<line>:<column>: <text>` names the file by
/// its label, and a path is where that file is.
pub(crate) fn debug_line(text: &str, workspace_root: &Path, external: &Path) -> String {
    let located = text.strip_prefix("@@").and_then(|rest| {
        let (repo, rest) = rest.split_once("//")?;
        let (package, rest) = rest.split_once(':')?;
        let (file, rest) = rest.split_once(':')?;
        let root = if repo.is_empty() {
            workspace_root.to_owned()
        } else {
            external.join(repo)
        };
        Some(format!(
            "{}:{rest}",
            root.join(package).join(file).display()
        ))
    });
    format!("DEBUG: {}", located.as_deref().unwrap_or(text))
}

/// Resolve the module graph, make the repositories asked for, and write the
/// lockfile with what the extensions that ran gave it. Blocking.
pub(crate) fn run(
    flags: &FetchFlags,
    bzlmod: &BzlmodFlags,
    workspace_root: &Path,
    module_bazel_text: &str,
) -> Result<(), CliError> {
    if flags.repos.is_empty() && !flags.all {
        return Err(CliError::CommandLine(anyhow::anyhow!(
            "fjfj fetch needs --repo=@name or --all"
        )));
    }
    run_inner(flags, bzlmod, workspace_root, module_bazel_text).map(|_| ())
}

/// What `build` does before it has a loading phase of its own: the module graph
/// is resolved, the target patterns are turned into targets (which makes the
/// repositories they and the BUILD files reach), and the lockfile is written
/// as `--lockfile_mode` says, with the extensions that ran on the way.
pub(crate) fn run_for_build(
    flags: &FetchFlags,
    bzlmod: &BzlmodFlags,
    workspace_root: &Path,
    module_bazel_text: &str,
    patterns: &[String],
    offset: &str,
    build: Option<&crate::build_command::Options>,
) -> Result<BuildLoad, CliError> {
    let (resolved, repos) = begin(flags, bzlmod, workspace_root, module_bazel_text)?;
    let unknown = std::cell::RefCell::new(None);
    let context = PatternContext { repo: "", offset };
    let parsed = patterns
        .iter()
        .map(|p| {
            TargetPattern::parse(p, context, &mut |apparent| match apparent {
                "" => String::new(),
                _ => repos.main_repo_canonical(apparent).unwrap_or_else(|| {
                    unknown.borrow_mut().get_or_insert(apparent.to_owned());
                    apparent.to_owned()
                }),
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::CommandLine(anyhow::Error::from(e)))?;
    if let Some(apparent) = unknown.into_inner() {
        return Err(CliError::CommandLine(anyhow::anyhow!(
            "No repository visible as '@{apparent}' from main repository"
        )));
    }
    let targets = fjfj_loading::resolve(&parsed, &repos);
    // The build happens before the lockfile is written, so that it holds the
    // extensions the analysis ran.
    let repos = Arc::new(repos);
    let report = match build {
        Some(options) if targets.failures.is_empty() || options.keep_going => {
            let mut options = options.clone();
            if let Some(platforms) = repos.module_repo("platforms") {
                for (setting, value) in fjfj_graph::config::host_constraints() {
                    options.configuration.constraints.insert(fjfj_graph::Label {
                        repo: platforms.clone(),
                        package: setting.to_owned(),
                        name: value.to_owned(),
                    });
                }
            }
            let request = crate::build_command::Request {
                layout: fjfj_exec::execroot::Layout {
                    workspace: workspace_root.to_path_buf(),
                    output_base: flags
                        .output_base
                        .clone()
                        .unwrap_or_else(|| default_output_base(workspace_root)),
                },
                options,
            };
            Some(crate::build_command::run(
                &repos,
                &targets.targets,
                &request,
            ))
        }
        _ => None,
    };
    let resolution = finish(resolved, &repos)?;
    Ok(BuildLoad {
        resolution,
        targets,
        report,
    })
}

/// What `build` has learnt once its patterns are resolved.
pub(crate) struct BuildLoad {
    pub resolution: Resolution,
    pub targets: fjfj_loading::Resolved,
    /// What building the targets did; `None` when a pattern failed and the
    /// build does not go on.
    pub report: Option<crate::build_command::Report>,
}

/// Resolves the module graph, makes the repositories `flags` ask for, and hands
/// back what the lock still needs written once more has run.
pub(crate) fn begin(
    flags: &FetchFlags,
    bzlmod: &BzlmodFlags,
    workspace_root: &Path,
    module_bazel_text: &str,
) -> Result<(Resolved, Repos), CliError> {
    let repository_cache = match &flags.repository_cache {
        Some(choice) => choice.clone(),
        None => Some(default_repository_cache()),
    };
    let resolved = crate::resolve_bzlmod_session(
        module_bazel_text,
        workspace_root,
        bzlmod,
        flags.isolated_extension_usages,
        repository_cache.clone(),
    )?;
    let output_base = flags
        .output_base
        .clone()
        .unwrap_or_else(|| default_output_base(workspace_root));
    let environ: BTreeMap<String, String> = std::env::vars().collect();
    let mut downloader = HttpDownloader::standard()
        .map_err(|e| CliError::Internal(anyhow::anyhow!("cannot make the HTTP client: {e}")))?;
    if !flags.credential_helpers.is_empty() {
        let path_env = environ.get("PATH").cloned().unwrap_or_default();
        let helpers = CredentialHelpers::new(
            &flags.credential_helpers,
            workspace_root.to_path_buf(),
            &path_env,
            flags
                .credential_helper_timeout
                .unwrap_or(Duration::from_secs(10)),
        )
        .map_err(|why| CliError::CommandLine(anyhow::anyhow!("{why}")))?;
        downloader = downloader.with_credential_helpers(helpers);
    }
    let options = Options {
        workspace_root: workspace_root.to_path_buf(),
        output_base,
        environ,
        downloader: Some(Arc::new(downloader)),
        repository_cache,
        distdirs: flags.distdirs.clone(),
        registries: bzlmod_registries(bzlmod, None)?,
        facts: resolved
            .session
            .as_ref()
            .map(|s| s.previous_facts())
            .unwrap_or_default(),
        repo_overrides: flags.repo_overrides.clone(),
    };
    let mut repos = Repos::from_resolution(options, resolved.resolution.clone())
        .map_err(|e| CliError::Build(anyhow::anyhow!(e.message)))?;
    // Extension results the lockfile already has stand while their inputs do.
    if let Some(session) = &resolved.session
        && session.mode() != fjfj_bzlmod::lockfile::LockfileMode::Refresh
    {
        repos.set_previous_extensions(
            session
                .previous_extensions()
                .into_iter()
                .map(|(id, factors, entry)| fjfj_repo::LockedExtension { id, factors, entry })
                .collect(),
        );
    }
    let outcome = fetch_repos(flags, &mut repos);
    if outcome.is_err() {
        print_warnings(&repos);
    }
    outcome?;
    Ok((resolved, repos))
}

fn print_warnings(repos: &Repos) {
    for event in repos.take_events() {
        eprintln!("ERROR: {event}");
    }
    repos.flush_prints(&Printer::of(repos));
    for warning in repos.warnings() {
        eprintln!("WARNING: {warning}");
    }
}

fn run_inner(
    flags: &FetchFlags,
    bzlmod: &BzlmodFlags,
    workspace_root: &Path,
    module_bazel_text: &str,
) -> Result<Resolution, CliError> {
    let (resolved, repos) = begin(flags, bzlmod, workspace_root, module_bazel_text)?;
    finish(resolved, &repos)
}

/// Writes the lockfile with what the extensions that have run gave it.
pub(crate) fn finish(resolved: Resolved, repos: &Repos) -> Result<Resolution, CliError> {
    print_warnings(repos);
    if let Some(session) = &resolved.session {
        session.set_module_extensions(
            module_extensions_json(&repos.locked_extensions()),
            resolved.resolution.extension_lock_ids(),
        );
        session.set_facts(repos.locked_facts());
    }
    crate::write_lockfile(&resolved)?;
    Ok(resolved.resolution)
}

/// What `fetch` makes: `--all`, or each `--repo`.
fn fetch_repos(flags: &FetchFlags, repos: &mut Repos) -> Result<(), CliError> {
    let failed = |message: String| CliError::Fetch(anyhow::anyhow!(message));
    let printer = Printer::of(repos);
    if flags.all {
        repos
            .run_extensions(Some(&printer))
            .map_err(|e| failed(e.message))?;
        for repo in repos.all_repos() {
            repos
                .fetch(&repo, Some(&printer))
                .map_err(|e| failed(e.message))?;
        }
        return Ok(());
    }
    for asked in &flags.repos {
        let canonical = match asked.strip_prefix("@@") {
            Some(canonical) => canonical.to_owned(),
            None => {
                let apparent = asked.strip_prefix('@').unwrap_or(asked);
                repos.main_repo_canonical(apparent).ok_or_else(|| {
                    CliError::CommandLine(anyhow::anyhow!(
                        "no repository visible as '@{apparent}' from the main repository"
                    ))
                })?
            }
        };
        repos
            .fetch(&canonical, Some(&printer))
            .map_err(|e| failed(e.message))?;
    }
    Ok(())
}

/// What `resolve_bzlmod_session` gives.
pub(crate) struct Resolved {
    pub resolution: Resolution,
    pub session: Option<Arc<LockSession>>,
    pub existing: Option<String>,
    pub lock_path: PathBuf,
}

#[cfg(test)]
mod print_tests {
    use super::debug_line;
    use std::path::Path;

    #[test]
    fn a_print_site_becomes_the_path_of_its_file() {
        let (ws, ext) = (Path::new("ws"), Path::new("ob/external"));
        assert_eq!(
            debug_line("@@//a/b:defs.bzl:3:5: hi", ws, ext),
            "DEBUG: ws/a/b/defs.bzl:3:5: hi"
        );
        assert_eq!(
            debug_line("@@rules_x+//:BUILD.bazel:1:1: x: y", ws, ext),
            "DEBUG: ob/external/rules_x+/BUILD.bazel:1:1: x: y"
        );
        assert_eq!(debug_line("no site", ws, ext), "DEBUG: no site");
    }
}
