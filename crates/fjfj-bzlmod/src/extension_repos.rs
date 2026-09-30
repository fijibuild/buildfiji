//! The names of the repos module extensions make, and what they add to repo
//! mappings (buildfiji-mum.8.5), read off `bazel mod dump_repo_mapping`.
//!
//! - An extension repo is `<canonical repo of the .bzl>+<unique extension
//!   name>+<repo>`: `ext++gen+r1` for `gen` in `@ext//:ext.bzl`, `+gen+a` for
//!   one in the main repo, `ext+2.0+gen+r1` when `ext` has two versions.
//! - The unique extension name is the extension's own name, with `2`, `3`
//!   added to the later ones of the same name in the same repo (`gen`,
//!   `gen2`): two `.bzl` files of one repo may both define `gen`. "Later"
//!   is by first use, modules in breadth-first order and each module's
//!   usages in the order written.
//! - A `use_repo_rule` is an extension of its own: its `.bzl` repo is the
//!   module that calls it, it is named for the rule (`+http_archive+x`,
//!   `p++rr+made`), and every call is an import.
//! - Only the root module's `override_repo` and `inject_repo` count, and they
//!   change what the repo means to every module. A dependency's are dropped.
//! - An extension repo's mapping is its `.bzl`'s repo's mapping, with the
//!   extension's other repos after it, then what was injected.
//!
//! An isolated usage (`isolate = True`, behind
//! `--experimental_isolated_extension_usages`) is named
//! `_<ext>+<module>+<version>+<the variable it was assigned to>`, which the
//! module file evaluation does not keep: these are left out (buildfiji-mum.8.7).

use std::collections::BTreeMap;

use crate::eval::INNATE_EXTENSION_FILE;
use crate::module::ModuleKey;
use crate::resolve::Resolution;

/// A module extension of the graph, with the modules that use it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionInstance {
    /// The canonical repo of the `.bzl` that defines the extension; for a
    /// `use_repo_rule`, the repo of the module that calls it.
    pub bzl_repo: String,
    /// The `.bzl` within that repo as `//pkg:file.bzl`, and the extension
    /// (`%name`), or the rule for a `use_repo_rule`.
    pub id: String,
    /// The extension's name as it appears in its repos' names.
    pub unique_name: String,
    /// Each use: the module, and the index in its `extension_usages`.
    pub usages: Vec<(ModuleKey, usize)>,
}

impl ExtensionInstance {
    /// The canonical name of the repo `repo` of this extension.
    pub fn repo_name(&self, repo: &str) -> String {
        format!("{}+{}+{repo}", self.bzl_repo, self.unique_name)
    }
}

impl Resolution {
    /// The repo a label written in `module` names, and the rest of the
    /// label as `//pkg:name`. `None` if the repo is not one the module
    /// sees through its own name or `bazel_dep`s.
    fn label_repo(&self, key: &ModuleKey, text: &str) -> Option<(String, String)> {
        let module = self
            .selection
            .resolved
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, m)| m)?;
        let (repo, rest) = if let Some(rest) = text.strip_prefix("@@") {
            let (repo, rest) = rest.split_once("//")?;
            (repo.to_owned(), format!("//{rest}"))
        } else if let Some(rest) = text.strip_prefix('@') {
            let (apparent, rest) = rest.split_once("//")?;
            let canonical = self
                .base_rows(key, module)
                .into_iter()
                .find(|(name, _)| name == apparent)?
                .1;
            (canonical, format!("//{rest}"))
        } else if text.starts_with("//") {
            (self.canonical_name_of(key), text.to_owned())
        } else {
            (self.canonical_name_of(key), format!("//{text}"))
        };
        Some((repo, rest))
    }

    /// Every module extension the selected modules use, in the order they
    /// are first used.
    pub fn extensions(&self) -> Vec<ExtensionInstance> {
        let mut found: Vec<ExtensionInstance> = Vec::new();
        let mut taken: BTreeMap<(String, String), usize> = BTreeMap::new();
        for (key, module) in &self.selection.resolved {
            for (index, usage) in module.extension_usages.iter().enumerate() {
                if usage.isolate {
                    continue;
                }
                let (bzl_repo, id, base) = if usage.bzl_file == INNATE_EXTENSION_FILE {
                    let (rule_bzl, rule) = usage
                        .extension_name
                        .split_once(' ')
                        .unwrap_or((&usage.extension_name, ""));
                    let Some((repo, rest)) = self.label_repo(key, rule_bzl) else {
                        continue;
                    };
                    (
                        self.canonical_name_of(key),
                        format!("@@{repo}{rest}%{rule}"),
                        rule.to_owned(),
                    )
                } else {
                    let Some((repo, rest)) = self.label_repo(key, &usage.bzl_file) else {
                        continue;
                    };
                    (
                        repo,
                        format!("{rest}%{}", usage.extension_name),
                        usage.extension_name.clone(),
                    )
                };
                match found
                    .iter_mut()
                    .find(|e| e.bzl_repo == bzl_repo && e.id == id)
                {
                    Some(existing) => existing.usages.push((key.clone(), index)),
                    None => {
                        let count = taken.entry((bzl_repo.clone(), base.clone())).or_insert(0);
                        *count += 1;
                        let unique_name = if *count == 1 {
                            base
                        } else {
                            format!("{base}{count}")
                        };
                        found.push(ExtensionInstance {
                            bzl_repo,
                            id,
                            unique_name,
                            usages: vec![(key.clone(), index)],
                        });
                    }
                }
            }
        }
        found
    }

    /// What the root module's `override_repo` and `inject_repo` point each
    /// `(extension, repo name)` at: a canonical repo, as the root module
    /// sees the name it gave.
    pub(crate) fn override_targets(
        &self,
        extensions: &[ExtensionInstance],
    ) -> BTreeMap<(usize, String), String> {
        let Some((root_key, root)) = self.selection.resolved.iter().find(|(k, _)| k.is_root())
        else {
            return BTreeMap::new();
        };
        // The names the root sees, as its imports mean before any override.
        let seen = self.module_mapping(root_key, root, extensions, None);
        let mut targets = BTreeMap::new();
        for (at, extension) in extensions.iter().enumerate() {
            for (user, index) in &extension.usages {
                if !user.is_root() {
                    continue;
                }
                for change in &root.extension_usages[*index].repo_overrides {
                    if let Some((_, canonical)) = seen
                        .iter()
                        .find(|(name, _)| *name == change.overriding_repo_name)
                    {
                        targets
                            .insert((at, change.overridden_repo_name.clone()), canonical.clone());
                    }
                }
            }
        }
        targets
    }

    /// The mapping of a repo `extension` made, given the names of every repo
    /// it made (what running it gave): the mapping of the module that holds
    /// its `.bzl`, then the extension's own repos, then what the root module
    /// injected.
    pub fn extension_repo_mapping(
        &self,
        extension: &ExtensionInstance,
        generated: &[String],
    ) -> Vec<(String, String)> {
        let extensions = self.extensions();
        let at = extensions
            .iter()
            .position(|e| e == extension)
            .expect("an extension of this resolution");
        let overrides = self.override_targets(&extensions);
        let mut rows = self
            .repo_mappings()
            .into_iter()
            .find(|(repo, _)| *repo == extension.bzl_repo)
            .map(|(_, rows)| rows)
            .unwrap_or_default();
        let target = |name: &str| {
            overrides
                .get(&(at, name.to_owned()))
                .cloned()
                .unwrap_or_else(|| extension.repo_name(name))
        };
        for name in generated {
            if !rows.iter().any(|(n, _)| n == name) {
                rows.push((name.clone(), target(name)));
            }
        }
        if let Some((root_key, root)) = self.selection.resolved.iter().find(|(k, _)| k.is_root()) {
            let seen = self.module_mapping(root_key, root, &extensions, Some(&overrides));
            for (user, index) in &extension.usages {
                if !user.is_root() {
                    continue;
                }
                for change in &root.extension_usages[*index].repo_overrides {
                    if change.must_exist
                        || rows.iter().any(|(n, _)| *n == change.overridden_repo_name)
                    {
                        continue;
                    }
                    if let Some((_, canonical)) =
                        seen.iter().find(|(n, _)| *n == change.overriding_repo_name)
                    {
                        rows.push((change.overridden_repo_name.clone(), canonical.clone()));
                    }
                }
            }
        }
        rows
    }
}
