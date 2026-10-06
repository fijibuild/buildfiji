//! Module extensions of several modules against Bazel 9.2.0 (buildfiji-lfe):
//! the table in `multi_matrix` is what `bazel build` did with a registry of
//! local-path modules, some of them defining an extension that others use,
//! and what this test makes of the same registry.

use crate::multi_matrix::MULTI_ROWS;
use crate::{Options, Repos};
use fjfj_bzlmod::discovery::RegistrySource;
use fjfj_bzlmod::registry::Registry;
use fjfj_bzlmod::resolve::{ResolveOptions, resolve};
use std::cell::RefCell;
use std::collections::BTreeMap;

type Files = &'static [(&'static str, &'static str)];

/// A registry of modules (name, version, files), the root module's files, the
/// repositories `bazel build` was asked for (canonical names), and what it did:
/// the error it stopped with, what was printed, and the `BUILD.bazel` of each
/// repository that was made.
pub(crate) struct MultiRow {
    pub(crate) name: &'static str,
    pub(crate) registry: &'static [(&'static str, &'static str, Files)],
    pub(crate) root: Files,
    pub(crate) fetch: &'static [&'static str],
    /// `--override_repository` values, `name=path`; `@WS@` is the workspace.
    pub(crate) overrides: &'static [&'static str],
    pub(crate) error: Option<&'static str>,
    pub(crate) printed: &'static [&'static str],
    pub(crate) builds: &'static [(&'static str, &'static str)],
}

struct Capture(RefCell<Vec<String>>);

impl starlark::PrintHandler for Capture {
    fn println(&self, text: &str) -> starlark::Result<()> {
        self.0.borrow_mut().push(
            fjfj_starlark::without_site(text)
                .trim_end_matches('\n')
                .to_owned(),
        );
        Ok(())
    }
}

fn write(path: &std::path::Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

#[test]
fn modules_share_their_extensions_as_bazel_does() {
    for row in MULTI_ROWS {
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
        // `bazel_tools` here has no dependencies, which the Bazel Central
        // Registry would otherwise serve.
        let source = RegistrySource::new(vec![Registry::local(&reg)])
            .with_builtin_module("bazel_tools", "module(name = 'bazel_tools')");
        let resolution = resolve(root_text, &source, &ResolveOptions::default())
            .unwrap_or_else(|e| panic!("{}: {e}", row.name));
        let base = dir.path().join("ob");
        let options = Options {
            workspace_root: ws.clone(),
            output_base: base.clone(),
            environ: BTreeMap::new(),
            downloader: None,
            repository_cache: None,
            distdirs: Vec::new(),
            registries: vec![Registry::local(&reg)],
            facts: Vec::new(),
            repo_overrides: row
                .overrides
                .iter()
                .map(|o| {
                    let (name, path) = o.split_once('=').unwrap();
                    let path = path.replace("@WS@", &ws.display().to_string());
                    (name.to_owned(), std::path::PathBuf::from(path))
                })
                .collect(),
        };
        let capture = Capture(RefCell::new(Vec::new()));
        let clean = |message: &str| {
            message
                .replace(&format!("file://{}", reg.display()), "<reg>")
                .replace(&ws.display().to_string(), "<ws>")
        };
        let mut error = None;
        match Repos::from_resolution(options, resolution) {
            Err(e) => error = Some(clean(&e.message)),
            Ok(repos) => {
                for target in row.fetch {
                    if let Err(e) = repos.fetch(target, Some(&capture)) {
                        error = Some(clean(&e.message));
                        break;
                    }
                }
            }
        }
        assert_eq!(error.as_deref(), row.error, "{}: the error", row.name);
        assert_eq!(
            *capture.0.borrow(),
            row.printed,
            "{}: what was printed",
            row.name
        );
        for (name, build) in row.builds {
            let file = base.join("external").join(name).join("BUILD.bazel");
            let made = std::fs::read_to_string(&file)
                .unwrap_or_else(|e| panic!("{}: {}: {e}", row.name, file.display()));
            assert_eq!(made, *build, "{}: {name}'s BUILD.bazel", row.name);
        }
    }
}
