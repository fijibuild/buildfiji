//! What `MODULE.bazel.lock` records for module extensions (buildfiji-mum.8.6)
//! against Bazel 9.2.0: the table in `lock_matrix` is the `moduleExtensions` it
//! wrote after building repositories of a local registry of local-path modules,
//! and the test here makes the same from `Repos::locked_extensions`.

use crate::lock_matrix::LOCK_ROWS;
use crate::{Options, Repos, module_extensions_json};
use fjfj_bzlmod::discovery::RegistrySource;
use fjfj_bzlmod::registry::Registry;
use fjfj_bzlmod::resolve::{ResolveOptions, resolve};
use std::collections::BTreeMap;

type Files = &'static [(&'static str, &'static str)];

/// A registry of modules (name, version, files), the root module's files, the
/// repositories built (canonical names), the environment, and the
/// `moduleExtensions` Bazel wrote, as compact JSON.
pub(crate) struct LockRow {
    pub(crate) name: &'static str,
    pub(crate) registry: &'static [(&'static str, &'static str, Files)],
    pub(crate) root: Files,
    pub(crate) fetch: &'static [&'static str],
    pub(crate) env: &'static [(&'static str, &'static str)],
    pub(crate) expected: &'static str,
    /// The `facts` section, as compact JSON.
    pub(crate) facts: &'static str,
    /// The warnings, as worded after `WARNING: `.
    pub(crate) warnings: &'static [&'static str],
    /// What stopped it, a part of the message.
    pub(crate) error: Option<&'static str>,
}

fn write(path: &std::path::Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

#[test]
fn what_extensions_leave_in_the_lockfile_is_what_bazel_writes() {
    let mut wrong = Vec::new();
    for row in LOCK_ROWS {
        let dir = tempfile::tempdir().unwrap();
        let reg = dir.path().join("reg");
        let ws = dir.path().join("ws");
        write(
            &reg.join("bazel_registry.json"),
            r#"{"mirrors": [], "module_base_path": "."}"#,
        );
        let mut versions: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for (module, version, files) in row.registry {
            versions.entry(module).or_default().push(version);
            let at = reg.join("modules").join(module).join(version);
            write(
                &at.join("source.json"),
                &format!(r#"{{"type": "local_path", "path": "modules/{module}/{version}"}}"#),
            );
            if !files.iter().any(|(f, _)| *f == "BUILD.bazel") {
                write(&at.join("BUILD.bazel"), "");
            }
            for (file, text) in *files {
                write(&at.join(file), text);
            }
        }
        for (module, list) in &versions {
            let list: Vec<String> = list.iter().map(|v| format!("\"{v}\"")).collect();
            write(
                &reg.join("modules").join(module).join("metadata.json"),
                &format!(
                    r#"{{"versions": [{}], "yanked_versions": {{}}}}"#,
                    list.join(",")
                ),
            );
        }
        write(&ws.join("BUILD.bazel"), "");
        for (file, text) in row.root {
            write(&ws.join(file), text);
        }
        let root_text = row
            .root
            .iter()
            .find(|(f, _)| *f == "MODULE.bazel")
            .unwrap()
            .1;
        let source = RegistrySource::new(vec![Registry::local(&reg)])
            .with_builtin_module("bazel_tools", "module(name = 'bazel_tools')");
        let resolution = resolve(root_text, &source, &ResolveOptions::default())
            .unwrap_or_else(|e| panic!("{}: {e}", row.name));
        let options = Options {
            workspace_root: ws.clone(),
            output_base: dir.path().join("ob"),
            environ: row
                .env
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
            downloader: None,
            repository_cache: None,
            registries: vec![Registry::local(&reg)],
            facts: Vec::new(),
            repo_overrides: Vec::new(),
        };
        let repos = Repos::from_resolution(options, resolution).unwrap();
        let mut error = None;
        for target in row.fetch {
            if let Err(e) = repos.fetch(target, None) {
                error = Some(e.message);
                break;
            }
        }
        let error_ok = match (&error, row.error) {
            (None, None) => true,
            (Some(got), Some(want)) => got.contains(want),
            _ => false,
        };
        let got =
            serde_json::to_string(&module_extensions_json(&repos.locked_extensions())).unwrap();
        let got_facts =
            serde_json::to_string(&fjfj_bzlmod::lockfile::Json::Object(repos.locked_facts()))
                .unwrap();
        let warnings = repos.warnings();
        // An extension that failed wrote nothing.
        let ext_ok = row.error.is_some() || got == row.expected;
        let facts_ok = row.error.is_some() || got_facts == row.facts;
        if !error_ok || warnings != row.warnings || !ext_ok || !facts_ok {
            wrong.push(format!(
                "{}\n  want error {:?}, warnings {:?}, facts {}, extensions {}\n  got  error {:?}, warnings {:?}, facts {}, extensions {}",
                row.name, row.error, row.warnings, row.facts, row.expected, error, warnings, got_facts, got
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} differ:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
