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
    /// Bazel 9.3.1's wording, and its exit code is a command-line error's.
    #[error("Config value '{0}' is not defined in any .rc file")]
    UndefinedConfig(String),
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
/// the plain (`config: None`) lines of `common`, then of each command
/// `command` inherits from, then of `command` itself, as Bazel applies them
/// (every `common` line of every file before any `build` line, whatever
/// their line numbers; buildfiji-ez35), with each `--config=NAME` flag
/// expanded in place to the flags of every matching `command:NAME` line, in
/// that same command order and recursively, so a config's own flags may
/// reference further configs. The `--config` token itself is spent in the
/// expansion. A config that (transitively) references itself is an error
/// rather than an infinite expansion, and so is one no line defines for
/// `command` (buildfiji-iggq).
pub fn resolve_command(lines: &[ResolvedLine], command: &str) -> Result<Vec<String>, ResolveError> {
    let mut out = Vec::new();
    let mut active_chain = Vec::new();
    for applicable in command_chain(command) {
        for line in lines {
            if line.config.is_none() && line.command == applicable {
                append_expanding(lines, command, &line.flags, &mut out, &mut active_chain)?;
            }
        }
    }
    // `--enable_platform_specific_config` is `--config=<host os>` after the
    // rest, and is spent here (the config's flags, not a `--config` flag).
    let mut enabled = false;
    out.retain(|flag| match flag.as_str() {
        "--enable_platform_specific_config" | "--enable_platform_specific_config=true" => {
            enabled = true;
            false
        }
        "--noenable_platform_specific_config" | "--enable_platform_specific_config=false" => {
            enabled = false;
            false
        }
        _ => true,
    });
    if enabled {
        let os = std::env::consts::OS;
        expand_config(lines, command, os, false, &mut out, &mut active_chain)?;
    }
    Ok(out)
}

/// `args` (flags and patterns of a command line) with each `--config=NAME` or
/// `--config NAME` before the `--` marker expanded in place, as the rc files'
/// own are. The flags of the rc files come first, from [`resolve_command`].
pub fn expand_command_line(
    lines: &[ResolvedLine],
    command: &str,
    args: &[String],
) -> Result<Vec<String>, ResolveError> {
    let mut out = Vec::with_capacity(args.len());
    let end = args.iter().position(|a| a == "--").unwrap_or(args.len());
    append_expanding(lines, command, &args[..end], &mut out, &mut Vec::new())?;
    out.extend(args[end..].iter().cloned());
    Ok(out)
}

/// What `--announce_rc` says about how the options of `command` were read
/// (probed on Bazel 9.3.1, buildfiji-e5go), one string per `INFO:` message:
///
/// - `Reading 'startup' options from <file>: <flags>` for each file with
///   startup lines;
/// - `Options provided by the client:` and the client's own options;
/// - for each section of the command chain (see [`resolve_command`]) and each
///   stretch of rc lines from one file (an `import` ends the importer's stretch,
///   which goes on after it), `Reading rc options for '<command>' from <file>:`
///   and `  [Inherited ]'<section>' options: <flags>`;
/// - for each `--config` met, in the order its lines are read,
///   `Found applicable config definition <section>:<name> in file <file>: <flags>`,
///   a config named by one of those lines following it at once.
pub fn announce(
    lines: &[ResolvedLine],
    command: &str,
    args: &[String],
    client_options: &str,
) -> Vec<String> {
    let mut out = Vec::new();
    let mut files: Vec<&Path> = Vec::new();
    for line in lines.iter().filter(|l| l.command == "startup") {
        if !files.contains(&line.source.as_path()) {
            files.push(&line.source);
        }
    }
    for file in files {
        let flags: Vec<&str> = lines
            .iter()
            .filter(|l| l.command == "startup" && l.source == file)
            .flat_map(|l| l.flags.iter().map(String::as_str))
            .collect();
        out.push(format!(
            "Reading 'startup' options from {}: {}",
            file.display(),
            flags.join(" ")
        ));
    }
    out.push(format!(
        "Options provided by the client:\n  Inherited 'common' options: {client_options}"
    ));
    let stretches = stretches(lines);
    let chain = command_chain(command);
    let mut seeds: Vec<String> = Vec::new();
    for section in &chain {
        for stretch in distinct(&stretches) {
            let in_stretch: Vec<&ResolvedLine> = lines
                .iter()
                .zip(&stretches)
                .filter(|(l, s)| **s == stretch && l.config.is_none() && l.command == *section)
                .map(|(l, _)| l)
                .collect();
            let Some(first) = in_stretch.first() else {
                continue;
            };
            let flags: Vec<&str> = in_stretch
                .iter()
                .flat_map(|l| l.flags.iter().map(String::as_str))
                .collect();
            let label = if *section == command {
                format!("'{section}'")
            } else {
                format!("Inherited '{section}'")
            };
            out.push(format!(
                "Reading rc options for '{command}' from {}:\n  {label} options: {}",
                first.source.display(),
                flags.join(" ")
            ));
            seeds.extend(config_names(flags.iter().map(|f| f.to_string())));
        }
    }
    let end = args.iter().position(|a| a == "--").unwrap_or(args.len());
    seeds.extend(config_names(args[..end].iter().cloned()));
    for name in seeds {
        announce_config(lines, &stretches, &chain, &name, &mut out);
    }
    out
}

/// The names `--config=NAME` and `--config NAME` give among `flags`.
fn config_names(flags: impl Iterator<Item = String>) -> Vec<String> {
    let mut names = Vec::new();
    let mut flags = flags;
    while let Some(flag) = flags.next() {
        if let Some(name) = flag.strip_prefix("--config=") {
            names.push(name.to_owned());
        } else if flag == "--config"
            && let Some(name) = flags.next()
        {
            names.push(name);
        }
    }
    names
}

fn announce_config(
    lines: &[ResolvedLine],
    stretches: &[usize],
    chain: &[&str],
    name: &str,
    out: &mut Vec<String>,
) {
    for section in chain {
        for stretch in distinct(stretches) {
            let in_stretch: Vec<&ResolvedLine> = lines
                .iter()
                .zip(stretches)
                .filter(|(l, s)| {
                    **s == stretch && l.config.as_deref() == Some(name) && l.command == *section
                })
                .map(|(l, _)| l)
                .collect();
            let Some(first) = in_stretch.first() else {
                continue;
            };
            let flags: Vec<String> = in_stretch.iter().flat_map(|l| l.flags.clone()).collect();
            out.push(format!(
                "Found applicable config definition {section}:{name} in file {}: {}",
                first.source.display(),
                flags.join(" ")
            ));
            for nested in config_names(flags.into_iter()) {
                announce_config(lines, stretches, chain, &nested, out);
            }
        }
    }
}

/// The number of the stretch of one file each line is in: it changes whenever
/// the file the next line comes from does.
fn stretches(lines: &[ResolvedLine]) -> Vec<usize> {
    let mut out = Vec::with_capacity(lines.len());
    let mut current = 0;
    for (i, line) in lines.iter().enumerate() {
        if i > 0 && lines[i - 1].source != line.source {
            current += 1;
        }
        out.push(current);
    }
    out
}

/// The stretch numbers in order, once each.
fn distinct(stretches: &[usize]) -> Vec<usize> {
    let mut seen = Vec::new();
    for s in stretches {
        if !seen.contains(s) {
            seen.push(*s);
        }
    }
    seen
}

/// The commands whose flags `command` also takes, as Bazel has them:
/// `test` and `run` read `build`'s, `coverage` `test`'s and so `build`'s.
fn inherits(command: &str) -> &'static [&'static str] {
    match command {
        "test" | "run" | "cquery" | "aquery" | "mobile-install" | "print_action" | "info" => {
            &["build"]
        }
        "coverage" => &["build", "test"],
        _ => &[],
    }
}

/// The sections of an rc file that apply to `command`, in the order Bazel
/// applies them: `common`, the commands it inherits, then `command`.
fn command_chain(command: &str) -> Vec<&str> {
    let mut chain = vec!["common"];
    chain.extend(inherits(command));
    if command != "common" {
        chain.push(command);
    }
    chain
}

fn append_expanding(
    lines: &[ResolvedLine],
    command: &str,
    flags: &[String],
    out: &mut Vec<String>,
    active_chain: &mut Vec<String>,
) -> Result<(), ResolveError> {
    let mut iter = flags.iter();
    while let Some(flag) = iter.next() {
        let name = match flag.strip_prefix("--config=") {
            Some(name) => Some(name.to_owned()),
            // `--config NAME`: the next word is the name.
            None if flag == "--config" => iter.next().cloned(),
            None => None,
        };
        match name {
            Some(name) => expand_config(lines, command, &name, true, out, active_chain)?,
            None => out.push(flag.clone()),
        }
    }
    Ok(())
}

/// The flags of every `command:NAME` line, `required` to exist for `command`
/// (a config an option asks for by name must, the host OS's need not).
fn expand_config(
    lines: &[ResolvedLine],
    command: &str,
    config_name: &str,
    required: bool,
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
    let mut defined = false;
    for applicable in command_chain(command) {
        for line in lines {
            if line.config.as_deref() == Some(config_name) && line.command == applicable {
                defined = true;
                append_expanding(lines, command, &line.flags, out, active_chain)?;
            }
        }
    }
    active_chain.pop();
    if required && !defined {
        return Err(ResolveError::UndefinedConfig(config_name.to_owned()));
    }
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
    fn a_common_line_after_a_build_line_still_comes_first() {
        let lines = vec![
            line("build", None, &["-c", "dbg"]),
            line("common", None, &["-c", "opt"]),
        ];
        assert_eq!(
            resolve_command(&lines, "build").unwrap(),
            vec!["-c", "opt", "-c", "dbg"]
        );
    }

    #[test]
    fn an_inheriting_command_reads_common_then_its_parents_then_itself() {
        let lines = vec![
            line("test", None, &["--t"]),
            line("build", None, &["--b1"]),
            line("coverage", None, &["--cov"]),
            line("common", None, &["--c"]),
            line("build", None, &["--b2"]),
        ];
        assert_eq!(
            resolve_command(&lines, "test").unwrap(),
            vec!["--c", "--b1", "--b2", "--t"]
        );
        assert_eq!(
            resolve_command(&lines, "coverage").unwrap(),
            vec!["--c", "--b1", "--b2", "--t", "--cov"]
        );
    }

    #[test]
    fn a_config_is_read_command_by_command_with_nested_configs_met_in_line() {
        // Probed on Bazel 9.3.1: build:ci, then build:ci2 (named by it), then test:ci.
        let lines = vec![
            line("test", None, &["--config=ci"]),
            line("test", Some("ci"), &["--t"]),
            line("build", Some("ci"), &["--b", "--config=ci2"]),
            line("build", Some("ci2"), &["--b2"]),
        ];
        assert_eq!(
            resolve_command(&lines, "test").unwrap(),
            vec!["--b", "--b2", "--t"]
        );
    }

    #[test]
    fn the_command_lines_configs_expand_in_both_spellings_before_the_marker() {
        let lines = vec![
            line("build", Some("d"), &["-c", "dbg"]),
            line("common", Some("e"), &["--jobs=2"]),
        ];
        let args: Vec<String> = ["//:g", "--config=d", "--config", "e", "--", "--config=d"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(
            expand_command_line(&lines, "build", &args).unwrap(),
            vec!["//:g", "-c", "dbg", "--jobs=2", "--", "--config=d"]
        );
        let undefined = ["--config=nope".to_owned()];
        assert_eq!(
            expand_command_line(&lines, "build", &undefined)
                .unwrap_err()
                .to_string(),
            "Config value 'nope' is not defined in any .rc file"
        );
    }

    fn at(source: &str, command: &str, config: Option<&str>, flags: &[&str]) -> ResolvedLine {
        ResolvedLine {
            source: PathBuf::from(source),
            ..line(command, config, flags)
        }
    }

    /// What Bazel 9.3.1 printed for these files: .bazelrc has common, an
    /// import of extra.rc, then more common and a build section; build:ci
    /// names ci2.
    #[test]
    fn announce_says_what_bazel_says() {
        let lines = vec![
            at("/w/.bazelrc", "startup", None, &["--max_idle_secs=100"]),
            at("/w/.bazelrc", "common", None, &["--a_before"]),
            at("/w/extra.rc", "common", None, &["--noshow_timestamps"]),
            at("/w/extra.rc", "build", None, &["--jobs=4"]),
            at("/w/extra.rc", "startup", None, &["--max_idle_secs=200"]),
            at("/w/.bazelrc", "common", None, &["--a_after"]),
            at("/w/.bazelrc", "build", None, &["--jobs=3"]),
            at("/w/.bazelrc", "test", None, &["--test_output=errors"]),
            at(
                "/w/.bazelrc",
                "build",
                Some("ci"),
                &["-c", "dbg", "--config=ci2"],
            ),
            at("/w/.bazelrc", "test", Some("ci"), &["--test_timeout=5"]),
            at("/w/.bazelrc", "build", Some("ci2"), &["--jobs=2"]),
        ];
        let args = vec!["--config=ci".to_owned(), "//:g".to_owned()];
        assert_eq!(
            announce(&lines, "test", &args, "--isatty=0 --terminal_columns=80"),
            vec![
                "Reading 'startup' options from /w/.bazelrc: --max_idle_secs=100",
                "Reading 'startup' options from /w/extra.rc: --max_idle_secs=200",
                "Options provided by the client:\n  Inherited 'common' options: --isatty=0 --terminal_columns=80",
                "Reading rc options for 'test' from /w/.bazelrc:\n  Inherited 'common' options: --a_before",
                "Reading rc options for 'test' from /w/extra.rc:\n  Inherited 'common' options: --noshow_timestamps",
                "Reading rc options for 'test' from /w/.bazelrc:\n  Inherited 'common' options: --a_after",
                "Reading rc options for 'test' from /w/extra.rc:\n  Inherited 'build' options: --jobs=4",
                "Reading rc options for 'test' from /w/.bazelrc:\n  Inherited 'build' options: --jobs=3",
                "Reading rc options for 'test' from /w/.bazelrc:\n  'test' options: --test_output=errors",
                "Found applicable config definition build:ci in file /w/.bazelrc: -c dbg --config=ci2",
                "Found applicable config definition build:ci2 in file /w/.bazelrc: --jobs=2",
                "Found applicable config definition test:ci in file /w/.bazelrc: --test_timeout=5",
            ]
        );
    }

    #[test]
    fn info_reads_the_build_section() {
        let lines = vec![line("build", None, &["--b"]), line("info", None, &["--i"])];
        assert_eq!(resolve_command(&lines, "info").unwrap(), vec!["--b", "--i"]);
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
            vec!["--define=asan=true", "--copt=-fsanitize=address"]
        );
    }

    #[test]
    fn platform_specific_config_is_the_host_os_config() {
        let os = std::env::consts::OS;
        let lines = [
            line("build", None, &["--enable_platform_specific_config"]),
            line("build", Some(os), &["--jobs=3"]),
            line("build", Some("not_this_os"), &["--jobs=9"]),
        ];
        assert_eq!(
            resolve_command(&lines, "build").unwrap(),
            vec!["--jobs=3".to_owned()]
        );
        let lines = [
            line("build", None, &["--enable_platform_specific_config"]),
            line("build", None, &["--noenable_platform_specific_config"]),
            line("build", Some(os), &["--jobs=3"]),
        ];
        assert!(resolve_command(&lines, "build").unwrap().is_empty());
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
            vec!["--copt=-fsanitize=address", "--jobs=4"]
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
        // The `test:asan` line doesn't define `asan` for `build`.
        assert!(matches!(
            resolve_command(&lines, "build"),
            Err(ResolveError::UndefinedConfig(name)) if name == "asan"
        ));
    }
}
