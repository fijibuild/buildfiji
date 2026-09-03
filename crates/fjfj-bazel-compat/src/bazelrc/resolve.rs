//! Discovery order, `import`/`try-import` expansion, and `--config`
//! expansion — the pure-Bazel-semantics half of `.bazelrc` handling that
//! sits on top of `parse`'s grammar. See `docs/design/cli-compat.md`.

use std::path::{Path, PathBuf};

use super::ast::Directive;
use super::parse::{ParseError, parse_rc_file};

/// Where to look for `.bazelrc` files and in what order, mirroring Bazel's
/// `--system_rc`/`--workspace_rc`/`--home_rc`/`--bazelrc` flags.
#[derive(Debug, Clone)]
pub struct DiscoveryOptions {
    pub workspace_root: Option<PathBuf>,
    pub home: Option<PathBuf>,
    pub system_rc_path: PathBuf,
    pub no_system_rc: bool,
    pub no_workspace_rc: bool,
    pub no_home_rc: bool,
    /// `--bazelrc=PATH`, in the order given; read after the default
    /// locations. An empty path means "stop reading rc files here",
    /// matching Bazel's `--bazelrc=` / `--bazelrc=/dev/null` behavior.
    pub explicit_bazelrc: Vec<PathBuf>,
}

impl Default for DiscoveryOptions {
    fn default() -> Self {
        DiscoveryOptions {
            workspace_root: None,
            home: None,
            system_rc_path: PathBuf::from("/etc/bazel.bazelrc"),
            no_system_rc: false,
            no_workspace_rc: false,
            no_home_rc: false,
            explicit_bazelrc: Vec::new(),
        }
    }
}

/// A `CommandFlags` line with `import`/`try-import` already expanded inline
/// and its originating file recorded, in final discovery+file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLine {
    pub source: PathBuf,
    pub line_no: usize,
    pub command: String,
    pub config: Option<String>,
    pub flags: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    #[error("reading {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("parsing {path}: {}", errors.iter().map(|e| e.message.as_str()).collect::<Vec<_>>().join("; "))]
    Parse {
        path: PathBuf,
        errors: Vec<ParseError>,
    },
    #[error("import cycle: {}", chain.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(" -> "))]
    ImportCycle { chain: Vec<PathBuf> },
    #[error("--config cycle: {} -> {repeated}", chain.join(" -> "))]
    ConfigCycle {
        chain: Vec<String>,
        repeated: String,
    },
}

/// Discover `.bazelrc` files per `opts`, parse each, and expand every
/// `import`/`try-import` in place, producing one flat, ordered list of
/// `CommandFlags` lines (across every file) ready for `resolve_command`.
pub fn discover_and_parse(opts: &DiscoveryOptions) -> Result<Vec<ResolvedLine>, ResolveError> {
    let mut roots = Vec::new();
    if !opts.no_system_rc {
        roots.push(opts.system_rc_path.clone());
    }
    if !opts.no_workspace_rc
        && let Some(ws) = &opts.workspace_root
    {
        roots.push(ws.join(".bazelrc"));
    }
    if !opts.no_home_rc
        && let Some(home) = &opts.home
    {
        roots.push(home.join(".bazelrc"));
    }
    for explicit in &opts.explicit_bazelrc {
        if explicit.as_os_str().is_empty() {
            break;
        }
        roots.push(explicit.clone());
    }

    let mut out = Vec::new();
    for root in roots {
        if !root.exists() {
            continue; // default locations are optional; explicit ones that don't exist error below via load_file
        }
        let mut chain = Vec::new();
        load_file(
            &root,
            false,
            opts.workspace_root.as_deref(),
            &mut chain,
            &mut out,
        )?;
    }
    Ok(out)
}

fn load_file(
    path: &Path,
    is_try: bool,
    workspace_root: Option<&Path>,
    chain: &mut Vec<PathBuf>,
    out: &mut Vec<ResolvedLine>,
) -> Result<(), ResolveError> {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if chain.contains(&canonical) {
        let mut full = chain.clone();
        full.push(canonical);
        return Err(ResolveError::ImportCycle { chain: full });
    }

    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if is_try && e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => {
            return Err(ResolveError::Io {
                path: path.to_path_buf(),
                source: e,
            });
        }
    };

    let (rc, errors) = parse_rc_file(&text);
    if !errors.is_empty() {
        return Err(ResolveError::Parse {
            path: path.to_path_buf(),
            errors,
        });
    }

    chain.push(canonical);
    for line in rc.lines {
        match line.directive {
            Directive::Import { path: import_path } => {
                let resolved = resolve_import_path(path, &import_path, workspace_root);
                load_file(&resolved, false, workspace_root, chain, out)?;
            }
            Directive::TryImport { path: import_path } => {
                let resolved = resolve_import_path(path, &import_path, workspace_root);
                load_file(&resolved, true, workspace_root, chain, out)?;
            }
            Directive::CommandFlags {
                command,
                config,
                flags,
            } => out.push(ResolvedLine {
                source: path.to_path_buf(),
                line_no: line.line_no,
                command,
                config,
                flags,
            }),
        }
    }
    chain.pop();
    Ok(())
}

/// Resolve an `import`/`try-import` path: `%workspace%/...` is relative to
/// the workspace root, an absolute path is used as-is, anything else is
/// relative to the importing file's directory.
fn resolve_import_path(importer: &Path, raw: &str, workspace_root: Option<&Path>) -> PathBuf {
    if let Some(rest) = raw.strip_prefix("%workspace%/")
        && let Some(ws) = workspace_root
    {
        return ws.join(rest);
    }
    let candidate = PathBuf::from(raw);
    if candidate.is_absolute() {
        return candidate;
    }
    importer
        .parent()
        .map(|dir| dir.join(&candidate))
        .unwrap_or(candidate)
}

/// Compute the final, ordered flag list for `command` (e.g. `"build"`):
/// every plain (`config: None`) `command`/`common` line across `lines`, in
/// order, with each `--config=NAME` flag expanded in place to the flags of
/// every matching `command:NAME`/`common:NAME` line — recursively, so a
/// config's own flags may reference further configs. A config that
/// (transitively) references itself is an error rather than an infinite
/// expansion.
pub fn resolve_command(lines: &[ResolvedLine], command: &str) -> Result<Vec<String>, ResolveError> {
    let mut out = Vec::new();
    let mut active_chain = Vec::new();
    for line in lines {
        if line.config.is_none() && applies_to(line, command) {
            append_expanding(lines, command, &line.flags, &mut out, &mut active_chain)?;
        }
    }
    Ok(out)
}

fn applies_to(line: &ResolvedLine, command: &str) -> bool {
    line.command == command || line.command == "common"
}

fn append_expanding(
    lines: &[ResolvedLine],
    command: &str,
    flags: &[String],
    out: &mut Vec<String>,
    active_chain: &mut Vec<String>,
) -> Result<(), ResolveError> {
    for flag in flags {
        out.push(flag.clone());
        if let Some(config_name) = flag.strip_prefix("--config=") {
            expand_config(lines, command, config_name, out, active_chain)?;
        }
    }
    Ok(())
}

fn expand_config(
    lines: &[ResolvedLine],
    command: &str,
    config_name: &str,
    out: &mut Vec<String>,
    active_chain: &mut Vec<String>,
) -> Result<(), ResolveError> {
    if active_chain.iter().any(|c| c == config_name) {
        return Err(ResolveError::ConfigCycle {
            chain: active_chain.clone(),
            repeated: config_name.to_string(),
        });
    }
    active_chain.push(config_name.to_string());
    for line in lines {
        if line.config.as_deref() == Some(config_name) && applies_to(line, command) {
            append_expanding(lines, command, &line.flags, out, active_chain)?;
        }
    }
    active_chain.pop();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(command: &str, config: Option<&str>, flags: &[&str]) -> ResolvedLine {
        ResolvedLine {
            source: PathBuf::from(".bazelrc"),
            line_no: 1,
            command: command.into(),
            config: config.map(String::from),
            flags: flags.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn common_and_command_lines_apply_in_order() {
        let lines = vec![
            line("common", None, &["--color=yes"]),
            line("build", None, &["--copt=-O2"]),
            line("test", None, &["--test_output=errors"]),
        ];
        assert_eq!(
            resolve_command(&lines, "build").unwrap(),
            vec!["--color=yes", "--copt=-O2"]
        );
    }

    #[test]
    fn config_expands_in_place() {
        let lines = vec![
            line("build", None, &["--config=asan"]),
            line("build", Some("asan"), &["--copt=-fsanitize=address"]),
            line("common", Some("asan"), &["--define=asan=true"]),
        ];
        assert_eq!(
            resolve_command(&lines, "build").unwrap(),
            vec![
                "--config=asan",
                "--copt=-fsanitize=address",
                "--define=asan=true",
            ]
        );
    }

    #[test]
    fn config_can_reference_another_config() {
        let lines = vec![
            line("build", None, &["--config=ci"]),
            line("build", Some("ci"), &["--config=asan", "--jobs=4"]),
            line("build", Some("asan"), &["--copt=-fsanitize=address"]),
        ];
        assert_eq!(
            resolve_command(&lines, "build").unwrap(),
            vec![
                "--config=ci",
                "--config=asan",
                "--copt=-fsanitize=address",
                "--jobs=4",
            ]
        );
    }

    #[test]
    fn self_referential_config_is_an_error() {
        let lines = vec![
            line("build", None, &["--config=loop"]),
            line("build", Some("loop"), &["--config=loop"]),
        ];
        assert!(matches!(
            resolve_command(&lines, "build"),
            Err(ResolveError::ConfigCycle { .. })
        ));
    }

    #[test]
    fn config_only_applies_to_its_command() {
        let lines = vec![
            line("build", None, &["--config=asan"]),
            line("test", Some("asan"), &["--test_arg=--asan"]),
        ];
        // The `test:asan` line doesn't match `build`'s `--config=asan`.
        assert_eq!(
            resolve_command(&lines, "build").unwrap(),
            vec!["--config=asan"]
        );
    }
}
