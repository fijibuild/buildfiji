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

// ---- what goes in MODULE.bazel.lock for an extension (buildfiji-mum.8.6) ----------

use crate::attrs::AttrValue;
use crate::lockfile::Json;
use crate::module::{ExtensionUsage, Module, Tag};
use base64::Engine as _;
use sha2::Digest as _;

/// An attribute value as Bazel writes it (`AttributeValuesAdapter`): `None` is
/// left out of a dict and kept in a list (as `null`); an int is a 32-bit one.
pub fn attr_json(value: &AttrValue) -> Json {
    match value {
        AttrValue::None => Json::Null,
        AttrValue::Bool(b) => Json::Bool(*b),
        AttrValue::Int(i) => Json::Number((*i).into()),
        AttrValue::String(s) => Json::String(s.clone()),
        AttrValue::List(items) => Json::Array(items.iter().map(attr_json).collect()),
        AttrValue::Dict(items) => Json::Object(
            items
                .iter()
                .filter(|(_, v)| !matches!(v, AttrValue::None))
                .map(|(k, v)| {
                    let key = match k {
                        AttrValue::String(s) => s.clone(),
                        AttrValue::Int(i) => i.to_string(),
                        AttrValue::Bool(b) => if *b { "True" } else { "False" }.to_owned(),
                        _ => String::new(),
                    };
                    (key, attr_json(v))
                })
                .collect(),
        ),
    }
}

/// The label of an extension's `.bzl` as the lockfile's digest sees it: one
/// written `//pkg:x.bzl` or `:x.bzl` is `@<the module's repo name>//pkg:x.bzl`,
/// any other as written.
pub fn normalized_bzl(written: &str, repo_name: &str) -> String {
    if let Some(rest) = written.strip_prefix("//") {
        format!("@{repo_name}//{rest}")
    } else if written.starts_with(':') {
        format!("@{repo_name}//{written}")
    } else {
        written.to_owned()
    }
}

fn tag_json(tag: &Tag) -> Json {
    Json::Object(vec![
        ("tagName".to_owned(), Json::String(tag.tag_class.clone())),
        (
            "attributeValues".to_owned(),
            Json::Object(
                tag.attrs
                    .iter()
                    .filter(|(_, v)| !matches!(v, AttrValue::None))
                    .map(|(k, v)| (k.clone(), attr_json(v)))
                    .collect(),
            ),
        ),
        ("devDependency".to_owned(), Json::Bool(tag.dev_dependency)),
        // `Location.BUILTIN`: where a tag was does not matter to the digest.
        (
            "location".to_owned(),
            Json::Object(vec![
                ("file".to_owned(), Json::String("<builtin>".to_owned())),
                ("line".to_owned(), Json::Number(0.into())),
                ("column".to_owned(), Json::Number(0.into())),
            ]),
        ),
    ])
}

fn usage_json(module: &Module, usages: &[&ExtensionUsage]) -> Json {
    let mut tags: Vec<&Tag> = usages.iter().flat_map(|u| u.tags.iter()).collect();
    tags.sort_by_key(|t| t.seq);
    Json::Object(vec![
        (
            "extensionBzlFile".to_owned(),
            Json::String(normalized_bzl(&usages[0].bzl_file, &module.repo_name)),
        ),
        (
            "extensionName".to_owned(),
            Json::String(usages[0].extension_name.clone()),
        ),
        ("proxies".to_owned(), Json::Array(Vec::new())),
        (
            "tags".to_owned(),
            Json::Array(tags.into_iter().map(tag_json).collect()),
        ),
        ("repoOverrides".to_owned(), Json::Object(Vec::new())),
    ])
}

impl Resolution {
    /// The extension as the lockfile names it: its `.bzl`'s label, `%`, and its
    /// name. `None` for what `use_repo_rule` makes, which has none.
    pub fn extension_lock_id(&self, extension: &ExtensionInstance) -> Option<String> {
        if extension.id.starts_with("@@") {
            return None;
        }
        Some(if extension.bzl_repo.is_empty() {
            extension.id.clone()
        } else {
            format!("@@{}{}", extension.bzl_repo, extension.id)
        })
    }

    /// The lockfile names of every extension the graph uses.
    pub fn extension_lock_ids(&self) -> Vec<String> {
        self.extensions()
            .iter()
            .filter_map(|e| self.extension_lock_id(e))
            .collect()
    }

    /// `usagesDigest`: a hash of what the modules that use an extension said
    /// to it, in the JSON Bazel's `SingleExtensionUsagesValue` gives, with
    /// where each tag was and what each `use_repo` imported left out: the
    /// SHA-256 of that text as UTF-16, in base64.
    pub fn extension_usages_digest(&self, extension: &ExtensionInstance) -> String {
        let extensions = self.extensions();
        let at = extensions.iter().position(|e| e == extension);
        let mut by_module: Vec<(&ModuleKey, &Module, Vec<&ExtensionUsage>)> = Vec::new();
        for (key, index) in &extension.usages {
            let module = &self
                .selection
                .resolved
                .iter()
                .find(|(k, _)| k == key)
                .expect("a selected module")
                .1;
            let usage = &module.extension_usages[*index];
            match by_module.iter_mut().find(|(k, _, _)| *k == key) {
                Some((_, _, list)) => list.push(usage),
                None => by_module.push((key, module, vec![usage])),
            }
        }
        let usages = Json::Object(
            by_module
                .iter()
                .map(|(key, module, list)| (key.to_string(), usage_json(module, list)))
                .collect(),
        );
        let modules = Json::Array(
            by_module
                .iter()
                .map(|(key, module, _)| {
                    Json::Object(vec![
                        ("name".to_owned(), Json::String(module.name.clone())),
                        (
                            "version".to_owned(),
                            Json::String(module.version.as_str().to_owned()),
                        ),
                        ("key".to_owned(), Json::String(key.to_string())),
                    ])
                })
                .collect(),
        );
        // What the root module replaced or added, by the canonical name of the
        // repo that takes its place.
        let targets = self.override_targets(&extensions);
        let mut overrides: Vec<(String, Json)> = Vec::new();
        if let Some(root) = self.selection.resolved.iter().find(|(k, _)| k.is_root()) {
            for (user, index) in &extension.usages {
                if !user.is_root() {
                    continue;
                }
                for change in &root.1.extension_usages[*index].repo_overrides {
                    let target = at.and_then(|at| {
                        targets
                            .get(&(at, change.overridden_repo_name.clone()))
                            .cloned()
                    });
                    let target = target.or_else(|| {
                        self.module_mapping(&root.0, &root.1, &extensions, None)
                            .into_iter()
                            .find(|(n, _)| *n == change.overriding_repo_name)
                            .map(|(_, c)| c)
                    });
                    if let Some(target) = target {
                        overrides.push((change.overridden_repo_name.clone(), Json::String(target)));
                    }
                }
            }
        }
        let value = Json::Object(vec![
            ("extensionUsages".to_owned(), usages),
            (
                "extensionUniqueName".to_owned(),
                Json::String(format!("{}+{}", extension.bzl_repo, extension.unique_name)),
            ),
            ("abridgedModules".to_owned(), modules),
            ("repoMappings".to_owned(), Json::Object(Vec::new())),
            ("repoOverrides".to_owned(), Json::Object(overrides)),
        ]);
        let text = serde_json::to_string(&value).expect("JSON is text");
        let utf16: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
        base64::engine::general_purpose::STANDARD.encode(sha2::Sha256::digest(&utf16))
    }
}
