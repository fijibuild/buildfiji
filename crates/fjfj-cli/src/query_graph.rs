//! The graph `fjfj query` runs over (buildfiji-9s8.1): packages loaded on
//! demand through the same sources `build` uses, their rules' attributes read
//! with the defaults of the rule class.

use fjfj_graph::Label;
use fjfj_graph::package::{Package, Target, TargetKind};
use fjfj_graph::pattern::{PatternContext, TargetPattern};
use fjfj_graph::rule::{AttrType, AttrValue, Cfg, default_condition, native_rule};
use fjfj_graph::schema::RuleSchema;
use fjfj_graph::visibility::is_visible;
use fjfj_loading::{PackageSource, input_files, resolve_with};
use fjfj_query::{Edge, Graph, Node, NodeAttr, NodeKind};
use fjfj_repo::Repos;
use fjfj_starlark::{RuleSource, rule_schema};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

pub(crate) struct QueryGraph {
    repos: Arc<Repos>,
    nodes: Mutex<BTreeMap<Label, Arc<Node>>>,
    /// The `package_group`s a visibility check has read. Kept for the life of
    /// the process, as the check borrows them for as long as it runs.
    groups: Mutex<BTreeMap<Label, &'static fjfj_graph::visibility::PackageGroup>>,
    /// What the main repository calls each repo: canonical to apparent.
    apparent: BTreeMap<String, String>,
}

impl QueryGraph {
    pub(crate) fn new(repos: Arc<Repos>) -> QueryGraph {
        let mut apparent: BTreeMap<String, String> = BTreeMap::new();
        for (name, canonical) in repos.mappings().entries("") {
            if !canonical.is_empty() {
                apparent.entry(canonical).or_insert(name);
            }
        }
        QueryGraph {
            repos,
            nodes: Mutex::new(BTreeMap::new()),
            groups: Mutex::new(BTreeMap::new()),
            apparent,
        }
    }

    fn package(&self, label: &Label) -> Result<Arc<Package>, String> {
        self.repos.package(&label.repo, &label.package)
    }

    fn schema(&self, class: &str, defined_in: Option<&Label>) -> Result<Arc<RuleSchema>, String> {
        match defined_in {
            None => native_rule(class)
                .map(|c| Arc::new(RuleSchema::native(c)))
                .ok_or_else(|| format!("unknown native rule '{class}'")),
            Some(bzl) => {
                let module = self.repos.module(bzl)?;
                rule_schema(&module, class).ok_or_else(|| {
                    format!("rule '{class}' is not defined by {}", self.display(bzl))
                })
            }
        }
    }

    /// An absolute path in `repo`.
    fn path(&self, repo: &str, relative: &str) -> Result<String, String> {
        let lookup = self.repos.lookup(repo)?;
        Ok(lookup
            .package_dir("")
            .join(relative)
            .to_string_lossy()
            .into_owned())
    }

    fn rule_node(
        &self,
        label: &Label,
        package: &Package,
        target: &Target,
        class: &str,
        defined_in: Option<&Label>,
        set: &[(String, AttrValue)],
    ) -> Result<Node, String> {
        let schema = self.schema(class, defined_in)?;
        let mut attrs = Vec::new();
        let mut edges = Vec::new();
        let rule_attr = |name: &str| set.iter().find(|(n, _)| n == name).map(|(_, v)| v);
        for attr in &schema.attrs {
            let explicit = rule_attr(&attr.name).is_some();
            let value = match attr.name.as_str() {
                "name" => Some(AttrValue::String(label.name.clone())),
                "visibility" => None,
                _ => rule_attr(&attr.name)
                    .cloned()
                    .or_else(|| attr.def.default_value().or_else(|| attr.def.ty.zero())),
            };
            if attr.name == "visibility" {
                attrs.push(NodeAttr {
                    name: "visibility".to_owned(),
                    text: self.visibility_text(package, target),
                    labels: Vec::new(),
                    explicit: target.visibility.is_some(),
                });
                continue;
            }
            let value = match (attr.name.as_str(), value) {
                // The package's default stands where the rule gave none.
                ("package_metadata", Some(AttrValue::LabelList(none)))
                    if none.is_empty() && !explicit =>
                {
                    Some(AttrValue::LabelList(
                        package.defaults.package_metadata.clone(),
                    ))
                }
                (_, value) => value,
            };
            let Some(value) = value else { continue };
            let mut labels = Vec::new();
            let implicit = attr.name.starts_with('_') || attr.name.starts_with('$');
            let is_label = matches!(
                attr.def.ty,
                AttrType::Label
                    | AttrType::LabelList
                    | AttrType::LabelKeyedStringDict
                    | AttrType::StringKeyedLabelDict
                    | AttrType::LabelListDict
            );
            if is_label {
                let mut found: BTreeMap<Label, Option<Label>> = BTreeMap::new();
                collect_labels(&value, None, &mut found);
                found.remove(&default_condition());
                let tool = matches!(attr.def.cfg, Cfg::Exec | Cfg::Host);
                for (to, condition) in found {
                    labels.push(to.clone());
                    edges.push(Edge {
                        to,
                        implicit,
                        tool,
                        condition: condition.map(|c| self.display_text(&c)),
                    });
                }
            }
            // The conditions of a `select()` are dependencies whatever the
            // attribute holds.
            if !is_label && let AttrValue::Select(list) = &value {
                for selector in &list.elements {
                    for (condition, _) in &selector.branches {
                        if *condition != default_condition() {
                            edges.push(Edge {
                                to: condition.clone(),
                                implicit,
                                tool: false,
                                condition: None,
                            });
                        }
                    }
                }
            }
            attrs.push(NodeAttr {
                name: attr.name.clone(),
                text: self.attr_text(&value),
                labels,
                explicit,
            });
        }
        // The `package_group`s that say who may see the rule.
        let visibility = target
            .visibility
            .as_ref()
            .unwrap_or(&package.default_visibility);
        for entry in &visibility.entries {
            if let fjfj_graph::visibility::VisibilityEntry::Group(group) = entry {
                edges.push(Edge {
                    to: group.clone(),
                    implicit: false,
                    tool: false,
                    condition: None,
                });
            }
        }
        // And the toolchain types it asks for.
        for (toolchain_type, _) in &schema.toolchains {
            edges.push(Edge {
                to: toolchain_type.clone(),
                implicit: true,
                tool: false,
                condition: None,
            });
        }
        // The one implicit dependency of a native rule that its attributes do
        // not list.
        if defined_in.is_none() && class == "genrule" {
            edges.push(Edge {
                to: Label {
                    repo: "bazel_tools".to_owned(),
                    package: "tools/genrule".to_owned(),
                    name: "genrule-setup.sh".to_owned(),
                },
                implicit: true,
                tool: false,
                condition: None,
            });
        }
        let location = target.location.clone();
        Ok(Node {
            label: label.clone(),
            kind: NodeKind::Rule {
                class: class.to_owned(),
                test: schema.test,
                executable: schema.executable,
            },
            location: self.rule_location(&label.repo, &location)?,
            attrs,
            edges,
        })
    }

    fn rule_location(&self, repo: &str, location: &str) -> Result<String, String> {
        match location.split_once(':') {
            Some((file, rest)) => Ok(format!("{}:{rest}", self.path(repo, file)?)),
            None => Ok(location.to_owned()),
        }
    }

    fn visibility_text(&self, package: &Package, target: &Target) -> String {
        let visibility = target
            .visibility
            .as_ref()
            .unwrap_or(&package.default_visibility);
        let mut parts = Vec::new();
        for entry in &visibility.entries {
            match entry {
                fjfj_graph::visibility::VisibilityEntry::Scope(scope) => {
                    parts.push(match scope {
                        fjfj_graph::visibility::PackageScope::Public => {
                            "//visibility:public".to_owned()
                        }
                        fjfj_graph::visibility::PackageScope::Repo(r) => {
                            format!("{}//...", self.repo_prefix(r))
                        }
                        fjfj_graph::visibility::PackageScope::Package { repo, package } => {
                            format!("{}//{package}:__pkg__", self.repo_prefix(repo))
                        }
                        fjfj_graph::visibility::PackageScope::Subpackages { repo, package } => {
                            format!("{}//{package}:__subpackages__", self.repo_prefix(repo))
                        }
                    });
                }
                fjfj_graph::visibility::VisibilityEntry::Group(label) => {
                    parts.push(self.display(label));
                }
            }
        }
        if parts.is_empty() {
            parts.push("//visibility:private".to_owned());
        }
        format!("[{}]", parts.join(", "))
    }

    fn repo_prefix(&self, repo: &str) -> String {
        if repo.is_empty() {
            String::new()
        } else {
            match self.apparent.get(repo) {
                Some(a) => format!("@{a}"),
                // A repo with a plain name, like `bazel_tools`, reads as itself.
                None if !repo.contains(['+', '~']) => format!("@{repo}"),
                None => format!("@@{repo}"),
            }
        }
    }

    fn display_text(&self, label: &Label) -> String {
        format!(
            "{}//{}:{}",
            self.repo_prefix(&label.repo),
            label.package,
            label.name
        )
    }

    /// How an attribute value reads to `attr()`.
    fn attr_text(&self, value: &AttrValue) -> String {
        let list = |items: Vec<String>| format!("[{}]", items.join(", "));
        match value {
            AttrValue::Bool(b) => if *b { "1" } else { "0" }.to_owned(),
            AttrValue::Int(i) => i.to_string(),
            AttrValue::String(s) => s.clone(),
            AttrValue::Label(l) => self.display(l),
            AttrValue::StringList(items) => list(items.clone()),
            AttrValue::IntList(items) => list(items.iter().map(|i| i.to_string()).collect()),
            AttrValue::LabelList(items) => list(items.iter().map(|l| self.display(l)).collect()),
            AttrValue::StringDict(items) => {
                format!(
                    "{{{}}}",
                    items
                        .iter()
                        .map(|(k, v)| format!("{k}: {v}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            AttrValue::StringListDict(items) => format!(
                "{{{}}}",
                items
                    .iter()
                    .map(|(k, v)| format!("{k}: [{}]", v.join(", ")))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            AttrValue::LabelKeyedStringDict(items) => format!(
                "{{{}}}",
                items
                    .iter()
                    .map(|(k, v)| format!("{}: {v}", self.display(k)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            AttrValue::StringKeyedLabelDict(items) => format!(
                "{{{}}}",
                items
                    .iter()
                    .map(|(k, v)| format!("{k}: {}", self.display(v)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            AttrValue::LabelListDict(items) => format!(
                "{{{}}}",
                items
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{k}: [{}]",
                            v.iter()
                                .map(|l| self.display(l))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            AttrValue::Select(list) => {
                let mut parts = Vec::new();
                for selector in &list.elements {
                    for (_, v) in &selector.branches {
                        if let Some(v) = v {
                            parts.push(self.attr_text(v));
                        }
                    }
                }
                parts.join(" ")
            }
        }
    }

    fn file_node(&self, label: &Label, kind: NodeKind) -> Result<Node, String> {
        let lookup = self.repos.lookup(&label.repo)?;
        let path = lookup.package_dir(&label.package).join(&label.name);
        Ok(Node {
            label: label.clone(),
            kind,
            location: format!("{}:1:1", path.display()),
            attrs: Vec::new(),
            edges: Vec::new(),
        })
    }
}

/// Every label `value` names, with the `select()` condition it comes under.
fn collect_labels(
    value: &AttrValue,
    condition: Option<&Label>,
    out: &mut BTreeMap<Label, Option<Label>>,
) {
    let _ = condition;
    fn walk(
        value: &AttrValue,
        condition: Option<&Label>,
        out: &mut BTreeMap<Label, Option<Label>>,
    ) {
        if let AttrValue::Select(list) = value {
            for selector in &list.elements {
                for (cond, branch) in &selector.branches {
                    out.entry(cond.clone()).or_insert(None);
                    if let Some(branch) = branch {
                        walk(branch, Some(cond), out);
                    }
                }
            }
            return;
        }
        let mut found = Vec::new();
        value.labels(&mut found);
        for l in found {
            out.entry(l.clone()).or_insert_with(|| condition.cloned());
        }
    }
    walk(value, None, out);
}

impl Graph for QueryGraph {
    fn pattern(&self, text: &str) -> Result<Vec<Label>, String> {
        let context = PatternContext {
            repo: "",
            offset: "",
        };
        let parsed = TargetPattern::parse(text, context, &mut |apparent| match apparent {
            "" => String::new(),
            _ => self
                .repos
                .main_repo_canonical(apparent)
                .unwrap_or_else(|| apparent.to_owned()),
        })
        .map_err(|e| e.to_string())?;
        let resolved = resolve_with(&[parsed], &*self.repos, true);
        if let Some(failure) = resolved.failures.first() {
            return Err(failure.message.clone());
        }
        Ok(resolved.targets)
    }

    fn node(&self, label: &Label) -> Result<Arc<Node>, String> {
        if let Some(done) = self.nodes.lock().unwrap().get(label) {
            return Ok(done.clone());
        }
        let package = self.package(label)?;
        let lookup = self.repos.lookup(&label.repo)?;
        let node = match package.target(&label.name) {
            Some(target) => match &target.kind {
                TargetKind::Rule {
                    rule_class,
                    defined_in,
                    attrs,
                } => self.rule_node(
                    label,
                    &package,
                    target,
                    rule_class,
                    defined_in.as_ref(),
                    attrs,
                )?,
                TargetKind::SourceFile => self.file_node(label, NodeKind::SourceFile)?,
                TargetKind::PackageGroup(group) => {
                    let mut node = self.file_node(label, NodeKind::PackageGroup)?;
                    node.location = self.rule_location(&label.repo, &target.location)?;
                    node.edges = group
                        .includes
                        .iter()
                        .map(|to| Edge {
                            to: to.clone(),
                            implicit: false,
                            tool: false,
                            condition: None,
                        })
                        .collect();
                    node
                }
                TargetKind::GeneratedFile { rule } => {
                    let producer = Label {
                        name: rule.clone(),
                        ..label.clone()
                    };
                    let mut node = self.file_node(
                        label,
                        NodeKind::GeneratedFile {
                            rule: producer.clone(),
                        },
                    )?;
                    node.location = self.rule_location(&label.repo, &target.location)?;
                    node.edges = vec![Edge {
                        to: producer,
                        implicit: false,
                        tool: false,
                        condition: None,
                    }];
                    node
                }
            },
            None => {
                fjfj_loading::declared_target(&package, &lookup, label)?;
                self.file_node(label, NodeKind::SourceFile)?
            }
        };
        let node = Arc::new(node);
        self.nodes
            .lock()
            .unwrap()
            .insert(label.clone(), node.clone());
        Ok(node)
    }

    fn siblings(&self, label: &Label) -> Result<Vec<Label>, String> {
        let package = self.package(label)?;
        let lookup = self.repos.lookup(&label.repo)?;
        let mut out: BTreeSet<Label> = package.targets().iter().map(|t| package.label(t)).collect();
        out.extend(input_files(&package).into_iter().map(|name| Label {
            name,
            ..label.clone()
        }));
        if let Ok(build) = lookup.build_file(&label.package)
            && let Some(name) = build.file_name().and_then(|n| n.to_str())
        {
            out.insert(Label {
                name: name.to_owned(),
                ..label.clone()
            });
        }
        Ok(out.into_iter().collect())
    }

    fn build_files(&self, label: &Label) -> Result<Vec<Label>, String> {
        let lookup = self.repos.lookup(&label.repo)?;
        let build = lookup
            .build_file(&label.package)
            .map_err(|e| e.to_string())?;
        let name = build
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("BUILD")
            .to_owned();
        Ok(vec![Label {
            name,
            ..label.clone()
        }])
    }

    fn load_files(&self, _label: &Label) -> Result<Vec<Label>, String> {
        Ok(Vec::new())
    }

    fn visible(&self, from: &Label, to: &Label) -> Result<bool, String> {
        let package = self.package(to)?;
        let Some(target) = package.target(&to.name) else {
            // A file nothing exports is private to its package.
            return Ok((&from.repo, &from.package) == (&to.repo, &to.package));
        };
        let visibility = target
            .visibility
            .as_ref()
            .unwrap_or(&package.default_visibility);
        let groups = |label: &Label| -> Option<&'static fjfj_graph::visibility::PackageGroup> {
            if let Some(done) = self.groups.lock().unwrap().get(label) {
                return Some(*done);
            }
            let group_package = self.repos.package(&label.repo, &label.package).ok()?;
            let TargetKind::PackageGroup(group) = &group_package.target(&label.name)?.kind else {
                return None;
            };
            let kept: &'static fjfj_graph::visibility::PackageGroup =
                Box::leak(Box::new(group.clone()));
            self.groups.lock().unwrap().insert(label.clone(), kept);
            Some(kept)
        };
        is_visible(
            visibility,
            (&to.repo, &to.package),
            (&from.repo, &from.package),
            &groups,
        )
        .map_err(|e| e.to_string())
    }

    fn display(&self, label: &Label) -> String {
        self.display_text(label)
    }
}

/// Every expected output below is what `bazel query` 9.2.0 printed for the
/// same files.
#[cfg(test)]
mod tests {
    use super::*;
    use fjfj_bzlmod::eval::{EvalOptions, eval_module_file};
    use fjfj_query::output::{Format, Order, render};
    use fjfj_query::{Evaluator, Options};

    fn workspace() -> (tempfile::TempDir, Arc<Repos>) {
        let dir = tempfile::tempdir().unwrap();
        let ws = dir.path().join("ws");
        for (file, text) in [
            ("MODULE.bazel", ""),
            ("BUILD", ""),
            (
                "rules.bzl",
                r#"
def _impl(ctx):
    return []
mylib = rule(implementation=_impl, attrs={"deps": attr.label_list(), "srcs": attr.label_list(allow_files=True), "_tool": attr.label(default="//c:tool", cfg="exec"), "opt": attr.string()})
"#,
            ),
            (
                "a/BUILD",
                r#"
load("//:rules.bzl", "mylib")
package(default_visibility=["//visibility:public"])
exports_files(["a.txt"])
filegroup(name="srcs", srcs=["a.txt", "//b:gen"], tags=["x"])
mylib(name="lib", deps=["//b:lib", ":srcs"], srcs=["a.txt"], opt="hello")
genrule(name="gen", srcs=[":lib"], outs=["gen.out"], cmd="echo > $@", tools=["//c:tool"])
alias(name="al", actual=":lib")
config_setting(name="cfg", values={"compilation_mode":"opt"})
mylib(name="sel", deps=select({":cfg": ["//b:lib"], "//conditions:default": ["//c:tool"]}))
test_suite(name="ts", tests=[":gen"])
"#,
            ),
            ("a/a.txt", ""),
            (
                "b/BUILD",
                r#"
load("//:rules.bzl", "mylib")
mylib(name="lib", deps=["//c/d:leaf"], visibility=["//a:__pkg__"])
genrule(name="gen", outs=["gen.txt"], cmd="echo > $@")
"#,
            ),
            ("c/BUILD", "filegroup(name='tool', srcs=['tool.sh'])\n"),
            ("c/tool.sh", ""),
            (
                "c/d/BUILD",
                "load('//:rules.bzl', 'mylib')\nmylib(name='leaf')\npackage_group(name='pg', packages=['//a/...'])\n",
            ),
        ] {
            let at = ws.join(file);
            std::fs::create_dir_all(at.parent().unwrap()).unwrap();
            std::fs::write(at, text).unwrap();
        }
        let module = eval_module_file("MODULE.bazel", "module(name = 'm')\n", &EvalOptions::root())
            .unwrap()
            .module;
        let repos = Repos::new(
            fjfj_repo::Options {
                workspace_root: ws,
                output_base: dir.path().join("ob"),
                environ: BTreeMap::new(),
                downloader: None,
                repository_cache: None,
                distdirs: Vec::new(),
                registries: Vec::new(),
                facts: Vec::new(),
                repo_overrides: Vec::new(),
            },
            module,
        )
        .unwrap();
        (dir, Arc::new(repos))
    }

    fn query(
        graph: &QueryGraph,
        text: &str,
        format: Format,
        order: Order,
        options: Options,
    ) -> String {
        let expr = fjfj_query::parse(text).unwrap();
        let ev = Evaluator::new(graph, options);
        match ev.eval(&expr) {
            Ok(set) => render(&ev, &set, format, order, '\n').unwrap(),
            Err(e) => format!("ERROR: {e}\n"),
        }
    }

    fn labels(graph: &QueryGraph, text: &str) -> Vec<String> {
        query(graph, text, Format::Label, Order::Auto, Options::default())
            .lines()
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn deps_follow_attributes_implicit_ones_and_select_branches() {
        let (_dir, repos) = workspace();
        let graph = QueryGraph::new(repos);
        assert_eq!(
            labels(&graph, "deps(//a:lib)"),
            [
                "//a:a.txt",
                "//a:lib",
                "//a:srcs",
                "//b:gen",
                "//b:lib",
                "//c:tool",
                "//c:tool.sh",
                "//c/d:leaf",
                "@bazel_tools//tools/genrule:genrule-setup.sh",
            ]
        );
        let no_implicit = Options {
            implicit_deps: false,
            ..Options::default()
        };
        assert_eq!(
            query(
                &graph,
                "deps(//a:lib)",
                Format::Label,
                Order::Auto,
                no_implicit
            ),
            "//a:a.txt\n//a:lib\n//a:srcs\n//b:gen\n//b:lib\n//c/d:leaf\n"
        );
        assert_eq!(
            labels(&graph, "deps(//a:sel)"),
            [
                "//a:cfg",
                "//a:sel",
                "//b:lib",
                "//c:tool",
                "//c:tool.sh",
                "//c/d:leaf"
            ]
        );
        assert_eq!(
            labels(&graph, "deps(//a:lib, 1)"),
            ["//a:a.txt", "//a:lib", "//a:srcs", "//b:lib", "//c:tool"]
        );
    }

    #[test]
    fn wildcards_take_rules_or_everything() {
        let (_dir, repos) = workspace();
        let graph = QueryGraph::new(repos);
        assert_eq!(
            labels(&graph, "//a:all"),
            [
                "//a:al", "//a:cfg", "//a:gen", "//a:lib", "//a:sel", "//a:srcs", "//a:ts"
            ]
        );
        assert_eq!(
            query(
                &graph,
                "//a:*",
                Format::LabelKind,
                Order::Auto,
                Options::default()
            ),
            "source file //a:BUILD\nsource file //a:a.txt\nalias rule //a:al\nconfig_setting rule //a:cfg\ngenrule rule //a:gen\ngenerated file //a:gen.out\nmylib rule //a:lib\nmylib rule //a:sel\nfilegroup rule //a:srcs\ntest_suite rule //a:ts\n"
        );
        assert_eq!(
            labels(&graph, "//..."),
            [
                "//a:al",
                "//a:cfg",
                "//a:gen",
                "//a:lib",
                "//a:sel",
                "//a:srcs",
                "//a:ts",
                "//b:gen",
                "//b:lib",
                "//c:tool",
                "//c/d:leaf",
            ]
        );
    }

    #[test]
    fn functions_select_the_targets_bazel_selects() {
        let (_dir, repos) = workspace();
        let graph = QueryGraph::new(repos);
        assert_eq!(labels(&graph, "attr(opt, \"hel\", //a:*)"), ["//a:lib"]);
        assert_eq!(
            labels(&graph, "attr(deps, \"b:lib\", //a:*)"),
            ["//a:lib", "//a:sel"]
        );
        assert_eq!(
            labels(&graph, "attr(deps, \"^\\[//b:lib, //a:srcs\\]$\", //a:*)"),
            ["//a:lib"]
        );
        assert_eq!(
            labels(&graph, "attr(srcs, \"a.txt\", //a:*)"),
            ["//a:lib", "//a:srcs"]
        );
        assert_eq!(labels(&graph, "attr(tags, \"x\", //a:*)"), ["//a:srcs"]);
        assert_eq!(
            labels(&graph, "attr(testonly, 0, //a:*)"),
            [
                "//a:al", "//a:cfg", "//a:gen", "//a:lib", "//a:sel", "//a:srcs"
            ]
        );
        assert_eq!(
            labels(&graph, "attr(_tool, \"tool\", //a:*)"),
            Vec::<String>::new()
        );
        assert_eq!(
            labels(&graph, "labels(deps, //a:lib)"),
            ["//a:srcs", "//b:lib"]
        );
        assert_eq!(
            labels(&graph, "labels(srcs, //a:*)"),
            ["//a:a.txt", "//a:lib", "//b:gen"]
        );
        assert_eq!(
            labels(&graph, "filter(\"^//b\", deps(//a:lib))"),
            ["//b:gen", "//b:lib"]
        );
        assert_eq!(
            labels(&graph, "kind(\"^mylib rule$\", //a:*)"),
            ["//a:lib", "//a:sel"]
        );
        assert_eq!(
            labels(&graph, "kind(\"generated file\", //a:*)"),
            ["//a:gen.out"]
        );
        assert_eq!(
            labels(&graph, "same_pkg_direct_rdeps(//a:lib)"),
            ["//a:al", "//a:gen"]
        );
        assert_eq!(
            labels(&graph, "rdeps(//..., //c/d:leaf)"),
            [
                "//a:al",
                "//a:gen",
                "//a:lib",
                "//a:sel",
                "//a:ts",
                "//b:lib",
                "//c/d:leaf"
            ]
        );
        assert_eq!(
            labels(&graph, "rdeps(//..., //c/d:leaf, 1)"),
            ["//b:lib", "//c/d:leaf"]
        );
        assert_eq!(
            labels(&graph, "somepath(//a:gen, //c/d:leaf)"),
            ["//a:gen", "//a:lib", "//b:lib", "//c/d:leaf"]
        );
        assert_eq!(labels(&graph, "visible(//a:lib, //b:*)"), ["//b:lib"]);
        assert_eq!(labels(&graph, "tests(//a:ts)"), Vec::<String>::new());
    }

    #[test]
    fn full_order_lists_a_target_before_what_it_depends_on() {
        let (_dir, repos) = workspace();
        let graph = QueryGraph::new(repos);
        assert_eq!(
            query(
                &graph,
                "deps(//a:lib)",
                Format::Label,
                Order::Full,
                Options::default()
            ),
            "//a:lib\n//b:lib\n//c/d:leaf\n//c:tool\n//c:tool.sh\n//a:srcs\n//b:gen\n@bazel_tools//tools/genrule:genrule-setup.sh\n//a:a.txt\n"
        );
        assert_eq!(
            query(
                &graph,
                "deps(//a:gen) + deps(//a:al) + //a:cfg",
                Format::Label,
                Order::Full,
                Options::default()
            ),
            "//a:gen\n//a:cfg\n//a:al\n//a:lib\n//b:lib\n//c/d:leaf\n//c:tool\n//c:tool.sh\n//a:srcs\n//b:gen\n@bazel_tools//tools/genrule:genrule-setup.sh\n//a:a.txt\n"
        );
    }

    #[test]
    fn errors_are_bazels() {
        let (_dir, repos) = workspace();
        let graph = QueryGraph::new(repos);
        let one = |q: &str| query(&graph, q, Format::Label, Order::Auto, Options::default());
        assert!(one("//a:nosuch").starts_with(
            "ERROR: no such target '//a:nosuch': target 'nosuch' not declared in package 'a' defined by "
        ));
        assert_eq!(
            one("//nosuch/..."),
            "ERROR: no targets found beneath 'nosuch'\n"
        );
        assert_eq!(
            one("let x = //a:lib in $y"),
            "ERROR: Evaluation of subquery \"$y\" failed (did you want to use --keep_going?): undefined variable 'y'\n"
        );
    }
}
