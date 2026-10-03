//! The graph `fjfj query` runs over (buildfiji-9s8.1): packages loaded on
//! demand through the same sources `build` uses, their rules' attributes read
//! with the defaults of the rule class.

use fjfj_graph::Label;
use fjfj_graph::package::{Package, Target, TargetKind};
use fjfj_graph::pattern::{PatternContext, TargetPattern};
use fjfj_graph::rule::{AttrType, AttrValue, Cfg, default_condition, native_rule};
use fjfj_graph::schema::RuleSchema;
use fjfj_graph::visibility::is_visible;
use fjfj_loading::{PackageSource, resolve_with};
use fjfj_query::{Edge, Frame, Graph, Node, NodeAttr, NodeKind};
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
    /// `--relative_locations`.
    relative: bool,
    /// `--consistent_labels`: labels as `@@repo//pkg:name`.
    consistent: bool,
}

impl QueryGraph {
    pub(crate) fn new(repos: Arc<Repos>) -> QueryGraph {
        // Every repo sees `@bazel_tools`.
        let mut apparent: BTreeMap<String, String> =
            BTreeMap::from([("bazel_tools".to_owned(), "bazel_tools".to_owned())]);
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
            relative: false,
            consistent: false,
        }
    }

    /// Write every label as `@@repo//pkg:name` (`--consistent_labels`).
    pub(crate) fn with_consistent_labels(mut self, on: bool) -> QueryGraph {
        self.consistent = on;
        self
    }

    /// Show locations from the root of the repository (`--relative_locations`).
    pub(crate) fn with_relative_locations(mut self, on: bool) -> QueryGraph {
        self.relative = on;
        self
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
        // Defaults that are functions of the other attributes.
        let computed: Vec<(String, AttrValue)> = match defined_in {
            Some(bzl) if schema.attrs.iter().any(|a| a.def.computed_default) => {
                let module = self.repos.module(bzl)?;
                let mut known: Vec<(String, AttrValue)> = Vec::new();
                for attr in &schema.attrs {
                    if let Some(v) = set
                        .iter()
                        .find(|(n, _)| *n == attr.name)
                        .map(|(_, v)| v.clone())
                        .or_else(|| attr.def.default_value())
                    {
                        known.push((attr.name.clone(), v));
                    }
                }
                known.push(("name".to_owned(), AttrValue::String(label.name.clone())));
                fjfj_starlark::computed_defaults(
                    &module,
                    class,
                    &known,
                    &self.repos.mappings(),
                    &bzl.repo,
                )?
            }
            _ => Vec::new(),
        };
        let mut attrs = Vec::new();
        let mut edges = Vec::new();
        let rule_attr = |name: &str| set.iter().find(|(n, _)| n == name).map(|(_, v)| v);
        for attr in &schema.attrs {
            // `name` is the one attribute every rule sets.
            let explicit = attr.name == "name" || rule_attr(&attr.name).is_some();
            let value = match attr.name.as_str() {
                "name" => Some(AttrValue::String(label.name.clone())),
                "visibility" => None,
                _ => rule_attr(&attr.name).cloned().or_else(|| {
                    computed
                        .iter()
                        .find(|(n, _)| *n == attr.name)
                        .map(|(_, v)| v.clone())
                        .or_else(|| attr.def.default_value())
                        .or_else(|| attr.def.ty.zero())
                }),
            };
            if attr.name == "visibility" {
                attrs.push(NodeAttr {
                    name: "visibility".to_owned(),
                    text: self.visibility_text(package, target),
                    labels: Vec::new(),
                    explicit: target.visibility.is_some(),
                    ty: AttrType::LabelList,
                    value: AttrValue::StringList(self.visibility_parts(package, target)),
                    unset: false,
                });
                continue;
            }
            let value = match (attr.name.as_str(), value) {
                // A test with no timeout has the one its size has.
                ("timeout", Some(AttrValue::String(none))) if none.is_empty() && !explicit => {
                    let size = match rule_attr("size") {
                        Some(AttrValue::String(size)) => size.as_str(),
                        _ => "medium",
                    };
                    Some(AttrValue::String(
                        match size {
                            "small" => "short",
                            "large" => "long",
                            "enormous" => "eternal",
                            _ => "moderate",
                        }
                        .to_owned(),
                    ))
                }
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
            let Some(value) = value else {
                attrs.push(NodeAttr {
                    name: shown_attr_name(&attr.name),
                    text: String::new(),
                    labels: Vec::new(),
                    explicit: false,
                    ty: attr.def.ty,
                    value: AttrValue::StringList(Vec::new()),
                    unset: true,
                });
                continue;
            };
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
                let transition = attr.def.cfg == Cfg::Transition;
                for (to, condition) in found {
                    labels.push(to.clone());
                    edges.push(Edge {
                        to,
                        implicit,
                        tool,
                        condition: condition.map(|c| self.display_text(&c)),
                        visibility: false,
                        attr: attr.name.clone(),
                        transition,
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
                                visibility: false,
                                attr: String::new(),
                                transition: false,
                            });
                        }
                    }
                }
            }
            attrs.push(NodeAttr {
                name: shown_attr_name(&attr.name),
                text: self.attr_text(&value),
                labels,
                explicit,
                ty: attr.def.ty,
                value: value.clone(),
                unset: false,
            });
        }
        // The attributes Bazel gives every executable and every test.
        if schema.executable || schema.test {
            attrs.push(NodeAttr {
                name: "$is_executable".to_owned(),
                text: "1".to_owned(),
                labels: Vec::new(),
                explicit: false,
                ty: AttrType::Bool,
                value: AttrValue::Bool(true),
                unset: false,
            });
        }
        if schema.test {
            let test = |name: &str| Label {
                repo: "bazel_tools".to_owned(),
                package: "tools/test".to_owned(),
                name: name.to_owned(),
            };
            let labelled: [(&str, AttrType, Vec<Label>); 8] = [
                (
                    "$collect_coverage_script",
                    AttrType::Label,
                    vec![test("collect_coverage")],
                ),
                ("$test_runtime", AttrType::LabelList, vec![test("runtime")]),
                (
                    "$test_setup_script",
                    AttrType::Label,
                    vec![test("test_setup")],
                ),
                ("$test_wrapper", AttrType::Label, vec![test("test_wrapper")]),
                (
                    "$xml_generator_script",
                    AttrType::Label,
                    vec![test("test_xml_generator")],
                ),
                ("$xml_writer", AttrType::Label, vec![test("xml_writer")]),
                (
                    ":coverage_report_generator",
                    AttrType::Label,
                    vec![test("coverage_report_generator")],
                ),
                (
                    ":coverage_support",
                    AttrType::Label,
                    vec![test("coverage_support")],
                ),
            ];
            for (name, ty, labels) in labelled {
                let value = if ty == AttrType::Label {
                    AttrValue::Label(labels[0].clone())
                } else {
                    AttrValue::LabelList(labels.clone())
                };
                attrs.push(NodeAttr {
                    name: name.to_owned(),
                    text: self.attr_text(&value),
                    labels: labels.clone(),
                    explicit: false,
                    ty,
                    value,
                    unset: false,
                });
                for to in labels {
                    edges.push(Edge {
                        to,
                        implicit: true,
                        tool: false,
                        condition: None,
                        visibility: false,
                        attr: name.to_owned(),
                        transition: false,
                    });
                }
            }
            // What `--run_under` sets: nothing here.
            for name in [":run_under_exec_config", ":run_under_target_config"] {
                attrs.push(NodeAttr {
                    name: name.to_owned(),
                    text: String::new(),
                    labels: Vec::new(),
                    explicit: false,
                    ty: AttrType::Label,
                    value: AttrValue::StringList(Vec::new()),
                    unset: true,
                });
            }
        }
        // A rule that transitions says so to an allowlist, by an attribute
        // Bazel adds.
        if schema.incoming_transition || schema.attrs.iter().any(|a| a.def.cfg == Cfg::Transition) {
            let allowlist = Label {
                repo: "bazel_tools".to_owned(),
                package: "tools/allowlists/function_transition_allowlist".to_owned(),
                name: "function_transition_allowlist".to_owned(),
            };
            attrs.push(NodeAttr {
                name: "$allowlist_function_transition".to_owned(),
                text: self.display_text(&allowlist),
                labels: vec![allowlist.clone()],
                explicit: false,
                ty: AttrType::Label,
                value: AttrValue::Label(allowlist.clone()),
                unset: false,
            });
            edges.push(Edge {
                to: allowlist,
                implicit: true,
                tool: false,
                condition: None,
                visibility: false,
                attr: "$allowlist_function_transition".to_owned(),
                transition: false,
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
                    visibility: true,
                    attr: String::new(),
                    transition: false,
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
                // A dependency, but not among the inputs of the rule.
                visibility: true,
                attr: String::new(),
                transition: false,
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
                visibility: false,
                attr: "$genrule_setup".to_owned(),
                transition: false,
            });
        }
        let location = target.location.clone();
        let rule_location = self.rule_location(&label.repo, &location)?;
        // A rule the BUILD file called has just its location for a stack.
        let stack = if target.stack.is_empty() {
            vec![Frame {
                location: rule_location.clone(),
                relative: location.clone(),
                function: "<toplevel>".to_owned(),
            }]
        } else {
            self.frames(&label.repo, &target.stack)?
        };
        let definition_stack = self.frames(&label.repo, &schema.definition_stack)?;
        let mut config_deps: Vec<Label> = Vec::new();
        for attr in &attrs {
            let AttrValue::Select(list) = &attr.value else {
                continue;
            };
            for selector in &list.elements {
                for (condition, _) in &selector.branches {
                    if *condition != default_condition() && !config_deps.contains(condition) {
                        config_deps.push(condition.clone());
                    }
                }
            }
        }
        Ok(Node {
            label: label.clone(),
            kind: NodeKind::Rule {
                class: class.to_owned(),
                test: schema.test,
                executable: schema.executable,
                native: defined_in.is_none(),
            },
            location: rule_location,
            relative_location: location.clone(),
            attrs,
            edges,
            outputs: package
                .targets()
                .iter()
                .filter(|t| matches!(&t.kind, TargetKind::GeneratedFile { rule } if *rule == target.name))
                .map(|t| package.label(t))
                .collect::<BTreeSet<Label>>()
                .into_iter()
                .collect(),
            visibility: self.visibility_parts(package, target),
            loads: Vec::new(),
            build_file: false,
            config_deps,
            stack,
            definition_stack,
            implementation_hash: defined_in.map(|bzl| {
                use sha2::{Digest, Sha256};
                hex::encode(Sha256::digest(
                    format!("{}%{class}", fjfj_graph::expand::label_text(bzl)).as_bytes(),
                ))
            }),
            group: None,
        })
    }

    /// The BUILD file of the package of `label`.
    fn build_file_of(&self, label: &Label) -> Result<Label, String> {
        self.maybe_build_file_of(label)
            .ok_or_else(|| format!("no BUILD file for package '{}'", label.package))
    }

    fn maybe_build_file_of(&self, label: &Label) -> Option<Label> {
        let lookup = self.repos.lookup(&label.repo).ok()?;
        let build = lookup.build_file(&label.package).ok()?;
        Some(Label {
            name: build.file_name()?.to_str()?.to_owned(),
            ..label.clone()
        })
    }

    /// A location a stack frame names, absolute: a BUILD file as the package
    /// of `repo` has it, a `.bzl` as `@@repo//pkg:file.bzl:line:col`.
    fn frame_location(&self, repo: &str, location: &str) -> Result<String, String> {
        let Some(label) = location.strip_prefix("@@") else {
            return self.rule_location(repo, location);
        };
        let mut parts = location.rsplitn(3, ':');
        let (Some(col), Some(line), Some(file)) = (parts.next(), parts.next(), parts.next()) else {
            return Ok(location.to_owned());
        };
        let _ = label;
        let file = file.trim_start_matches("@@");
        let (bzl_repo, rest) = file.split_once("//").unwrap_or(("", file));
        let (package, name) = rest.split_once(':').unwrap_or(("", rest));
        let path = self.repos.lookup(bzl_repo)?.package_dir(package).join(name);
        Ok(format!("{}:{line}:{col}", path.display()))
    }

    fn frames(
        &self,
        repo: &str,
        frames: &[fjfj_graph::package::StackFrame],
    ) -> Result<Vec<Frame>, String> {
        frames
            .iter()
            .map(|f| {
                // From the root of the repository: a `.bzl` is named by its
                // label, a BUILD file already is.
                let relative = match f.location.strip_prefix("@@") {
                    Some(label) => {
                        let mut parts = label.rsplitn(3, ':');
                        match (parts.next(), parts.next(), parts.next()) {
                            (Some(col), Some(line), Some(file)) => {
                                let (_, rest) = file.split_once("//").unwrap_or(("", file));
                                let (package, name) = rest.split_once(':').unwrap_or(("", rest));
                                if package.is_empty() {
                                    format!("{name}:{line}:{col}")
                                } else {
                                    format!("{package}/{name}:{line}:{col}")
                                }
                            }
                            _ => f.location.clone(),
                        }
                    }
                    None => f.location.clone(),
                };
                Ok(Frame {
                    location: self.frame_location(repo, &f.location)?,
                    relative,
                    function: f.function.clone(),
                })
            })
            .collect()
    }

    /// The aspects an action of `aspect` came from: the aspect, named by the
    /// file that defines it, and then those it requires, each so.
    pub(crate) fn aspect_chain(
        &self,
        aspect: &fjfj_starlark::AspectRef,
    ) -> Vec<fjfj_starlark::AspectRef> {
        let mut chain = Vec::new();
        let mut next = vec![aspect.clone()];
        while let Some(current) = next.pop() {
            let spec = self
                .repos
                .module(&current.bzl)
                .ok()
                .and_then(|module| fjfj_starlark::aspect_spec(&module, &current.name));
            let defined = spec
                .as_ref()
                .and_then(|s| s.schema.defined_in.clone())
                .unwrap_or_else(|| current.bzl.clone());
            let named = fjfj_starlark::AspectRef {
                bzl: defined,
                name: current.name.clone(),
            };
            if !chain.contains(&named) {
                chain.push(named);
            }
            if let Some(spec) = spec {
                // Depth first, in the order the aspect lists them.
                next.extend(spec.requires.into_iter().rev());
            }
        }
        chain
    }

    /// Where the transition on `attr` of the rule `rule_class` of `bzl` was
    /// written, as `/abs/path/file.bzl:line:col`.
    pub(crate) fn transition_location(
        &self,
        bzl: &Label,
        rule_class: &str,
        attr: &str,
    ) -> Option<String> {
        let module = self.repos.module(bzl).ok()?;
        let spec =
            fjfj_starlark::transition_spec(&module, rule_class, fjfj_starlark::Edge::Attr(attr))?;
        self.frame_location(&bzl.repo, &spec.defined_at).ok()
    }

    fn rule_location(&self, repo: &str, location: &str) -> Result<String, String> {
        match location.split_once(':') {
            Some((file, rest)) => Ok(format!("{}:{rest}", self.path(repo, file)?)),
            None => Ok(location.to_owned()),
        }
    }

    fn visibility_text(&self, package: &Package, target: &Target) -> String {
        format!("[{}]", self.visibility_parts(package, target).join(", "))
    }

    /// The entries of the target's `visibility` as they are written.
    fn visibility_parts(&self, package: &Package, target: &Target) -> Vec<String> {
        self.visibility_parts_of(
            target
                .visibility
                .as_ref()
                .unwrap_or(&package.default_visibility),
        )
    }

    fn visibility_parts_of(&self, visibility: &fjfj_graph::visibility::Visibility) -> Vec<String> {
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
        parts
    }

    /// A `packages` entry of a `package_group` as it is written.
    fn spec_text(&self, spec: &fjfj_graph::visibility::PackageSpec) -> String {
        use fjfj_graph::visibility::PackageScope;
        let body = match &spec.scope {
            PackageScope::Public => "public".to_owned(),
            PackageScope::Repo(r) => format!("{}//...", self.repo_prefix(r)),
            PackageScope::Package { repo, package } => {
                format!("{}//{package}", self.repo_prefix(repo))
            }
            PackageScope::Subpackages { repo, package } => {
                format!("{}//{package}/...", self.repo_prefix(repo))
            }
        };
        if spec.negated {
            format!("-{body}")
        } else {
            body
        }
    }

    fn repo_prefix(&self, repo: &str) -> String {
        if self.consistent {
            return format!("@@{repo}");
        }
        if repo.is_empty() {
            String::new()
        } else {
            match self.apparent.get(repo) {
                Some(a) => format!("@{a}"),
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
    pub(crate) fn attr_text(&self, value: &AttrValue) -> String {
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
            relative_location: format!(
                "{}:1:1",
                if label.package.is_empty() {
                    label.name.clone()
                } else {
                    format!("{}/{}", label.package, label.name)
                }
            ),
            attrs: Vec::new(),
            edges: Vec::new(),
            outputs: Vec::new(),
            visibility: vec!["//visibility:private".to_owned()],
            loads: Vec::new(),
            build_file: false,
            config_deps: Vec::new(),
            stack: Vec::new(),
            definition_stack: Vec::new(),
            implementation_hash: None,
            group: None,
        })
    }
}

/// The name an attribute has in output: the private attribute `_x` of a
/// Starlark rule is the implicit attribute `$x`.
fn shown_attr_name(name: &str) -> String {
    match name.strip_prefix('_') {
        Some(rest) => format!("${rest}"),
        None => name.to_owned(),
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
    fn relative_locations(&self) -> bool {
        self.relative
    }

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
                TargetKind::SourceFile => {
                    let mut node = self.file_node(label, NodeKind::SourceFile)?;
                    node.visibility = self.visibility_parts(&package, target);
                    node
                }
                TargetKind::PackageGroup(group) => {
                    let mut node = self.file_node(label, NodeKind::PackageGroup)?;
                    node.location = self.rule_location(&label.repo, &target.location)?;
                    node.group = Some((
                        group.includes.clone(),
                        group.specs.iter().map(|s| self.spec_text(s)).collect(),
                    ));
                    node.edges = group
                        .includes
                        .iter()
                        .map(|to| Edge {
                            to: to.clone(),
                            implicit: false,
                            tool: false,
                            condition: None,
                            visibility: false,
                            attr: String::new(),
                            transition: false,
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
                        visibility: false,
                        attr: String::new(),
                        transition: false,
                    }];
                    node
                }
            },
            None => {
                let is_build = lookup
                    .build_file(&label.package)
                    .ok()
                    .and_then(|b| b.file_name().map(|n| n.to_string_lossy().into_owned()))
                    .is_some_and(|n| n == label.name);
                let exists = lookup
                    .package_dir(&label.package)
                    .join(&label.name)
                    .is_file();
                // A file the load graph reaches is a target here even where a
                // pattern naming it would be refused.
                if !exists {
                    fjfj_loading::declared_target(&package, &lookup, label)?;
                }
                let mut node = self.file_node(label, NodeKind::SourceFile)?;
                if is_build {
                    node.build_file = true;
                    node.loads = package.loads.clone();
                    node.visibility = self.visibility_parts_of(&package.default_visibility);
                } else if label.name.ends_with(".bzl") {
                    node.loads = self.repos.mappings().loads_of(label);
                }
                node
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
        out.extend(package.input_files().iter().map(|name| Label {
            name: name.clone(),
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
        let mut out = BTreeSet::new();
        for file in std::iter::once(self.build_file_of(label)?).chain(self.load_files(label)?) {
            if let Some(build) = self.maybe_build_file_of(&file) {
                out.insert(build);
            }
            out.insert(file);
        }
        Ok(out.into_iter().collect())
    }

    fn load_files(&self, label: &Label) -> Result<Vec<Label>, String> {
        let package = self.package(label)?;
        let mappings = self.repos.mappings();
        let mut seen: BTreeSet<Label> = BTreeSet::new();
        let mut todo: Vec<Label> = package.loads.clone();
        while let Some(file) = todo.pop() {
            if seen.insert(file.clone()) {
                todo.extend(mappings.loads_of(&file));
            }
        }
        Ok(seen.into_iter().collect())
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
        workspace_of(&[
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
        ])
    }

    /// A workspace of these files.
    fn workspace_of(files: &[(&str, &str)]) -> (tempfile::TempDir, Arc<Repos>) {
        let dir = tempfile::tempdir().unwrap();
        let ws = dir.path().join("ws");
        for (file, text) in files {
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
    fn xml_is_what_bazel_prints() {
        let (dir, repos) = workspace();
        let graph = QueryGraph::new(repos);
        let xml = query(
            &graph,
            "//a:a.txt + //a:gen.out + //a:sel + //a:ts + //c/d:pg + //b:gen",
            Format::Xml,
            Order::Auto,
            Options::default(),
        )
        .replace(&dir.path().join("ws").display().to_string(), "<ws>");
        assert_eq!(
            xml,
            r#"<?xml version="1.1" encoding="UTF-8" standalone="no"?>
<query version="2">
    <source-file location="<ws>/a/a.txt:1:1" name="//a:a.txt">
        <visibility-label name="//visibility:public"/>
    </source-file>
    <generated-file generating-rule="//a:gen" location="<ws>/a/BUILD:7:8" name="//a:gen.out"/>
    <rule class="mylib" location="<ws>/a/BUILD:10:6" name="//a:sel">
        <string name="name" value="sel"/>
        <list name="deps">
            <label value="//b:lib"/>
            <label value="//c:tool"/>
        </list>
        <rule-input name="//a:cfg"/>
        <rule-input name="//b:lib"/>
        <rule-input name="//c:tool"/>
    </rule>
    <rule class="test_suite" location="<ws>/a/BUILD:11:11" name="//a:ts">
        <string name="name" value="ts"/>
        <list name="tests">
            <label value="//a:gen"/>
        </list>
        <rule-input name="//a:gen"/>
    </rule>
    <rule class="genrule" location="<ws>/b/BUILD:4:8" name="//b:gen">
        <string name="name" value="gen"/>
        <list name="outs">
            <output value="//b:gen.txt"/>
        </list>
        <string name="cmd" value="echo &gt; $@"/>
        <rule-input name="@bazel_tools//tools/genrule:genrule-setup.sh"/>
        <rule-output name="//b:gen.txt"/>
    </rule>
    <package-group location="<ws>/c/d/BUILD:3:14" name="//c/d:pg">
        <list name="includes"/>
        <list name="packages">
            <string value="//a/..."/>
        </list>
    </package-group>
</query>
"#
        );
    }

    /// The targets of `text` as `--output=streamed_jsonproto` prints them, each
    /// parsed.
    fn jsonproto(
        graph: &QueryGraph,
        text: &str,
        proto: &fjfj_query::target_proto::ProtoOptions,
    ) -> Vec<serde_json::Value> {
        let ev = Evaluator::new(graph, Options::default());
        let set = ev.eval(&fjfj_query::parse(text).unwrap()).unwrap();
        let bytes = fjfj_query::output::render_bytes(
            &ev,
            &set,
            Format::StreamedJsonProto,
            Order::Auto,
            '\n',
            proto,
            &Default::default(),
        )
        .unwrap();
        String::from_utf8(bytes)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    /// Each expectation is what `bazel query --output=streamed_jsonproto`
    /// printed for the same kinds of target.
    #[test]
    fn proto_describes_each_kind_of_target_as_bazel_does() {
        let (dir, repos) = workspace();
        let graph = QueryGraph::new(repos);
        let ws = dir.path().join("ws").display().to_string();
        let all = jsonproto(
            &graph,
            "//a:a.txt + //b:gen.txt + //b:gen + //c/d:pg + //a:lib",
            &Default::default(),
        );
        let by_name = |name: &str| {
            all.iter()
                .find(|t| {
                    ["rule", "sourceFile", "generatedFile", "packageGroup"]
                        .iter()
                        .any(|k| t[k]["name"] == name)
                })
                .unwrap_or_else(|| panic!("no {name}"))
        };
        assert_eq!(
            by_name("//a:a.txt"),
            &serde_json::json!({"type": "SOURCE_FILE", "sourceFile": {
                "name": "//a:a.txt", "location": format!("{ws}/a/a.txt:1:1"),
                "visibilityLabel": ["//visibility:public"]}})
        );
        assert_eq!(
            by_name("//b:gen.txt"),
            &serde_json::json!({"type": "GENERATED_FILE", "generatedFile": {
                "name": "//b:gen.txt", "generatingRule": "//b:gen",
                "location": format!("{ws}/b/BUILD:4:8")}})
        );
        assert_eq!(
            by_name("//c/d:pg"),
            &serde_json::json!({"type": "PACKAGE_GROUP", "packageGroup": {
                "name": "//c/d:pg", "containedPackage": ["//a/..."]}})
        );
        // A genrule has the attributes of its class and Bazel's hidden ones, by
        // name, and says what it reads and makes.
        let genrule = &by_name("//b:gen")["rule"];
        let names: Vec<&str> = genrule["attribute"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["name"].as_str().unwrap())
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
        assert_eq!(
            &names[..4],
            [
                "$config_dependencies",
                "$genrule_setup",
                "$is_executable",
                ":action_listener"
            ]
        );
        assert_eq!(genrule["ruleOutput"], serde_json::json!(["//b:gen.txt"]));
        let attr = |name: &str| {
            genrule["attribute"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["name"] == name)
                .unwrap()
                .clone()
        };
        assert_eq!(
            attr("outs"),
            serde_json::json!({"name": "outs", "type": "OUTPUT_LIST",
                "stringListValue": ["//b:gen.txt"], "explicitlySpecified": true, "nodep": false})
        );
        assert_eq!(
            attr("testonly"),
            serde_json::json!({"name": "testonly", "type": "BOOLEAN", "intValue": 0,
                "stringValue": "false", "explicitlySpecified": false, "booleanValue": false})
        );
        // `name` is explicit, a value the rule left alone is not.
        assert_eq!(attr("name")["explicitlySpecified"], true);
    }

    #[test]
    fn proto_flags_shape_the_rules() {
        let (_dir, repos) = workspace();
        let graph = QueryGraph::new(repos);
        let rule = |proto: &fjfj_query::target_proto::ProtoOptions| {
            jsonproto(&graph, "//a:sel", proto).remove(0)["rule"].clone()
        };
        // A select() shows every value it could take, or as it is.
        let flat = rule(&Default::default());
        let deps = flat["attribute"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["name"] == "deps")
            .unwrap();
        assert_eq!(deps["type"], "LABEL_LIST");
        assert_eq!(
            deps["stringListValue"],
            serde_json::json!(["//b:lib", "//c:tool"])
        );
        let mut raw = fjfj_query::target_proto::ProtoOptions::default();
        assert!(raw.flag("noproto:flatten_selects", None).unwrap());
        let rule_raw = rule(&raw);
        let deps = rule_raw["attribute"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["name"] == "deps")
            .unwrap();
        assert_eq!(deps["type"], "SELECTOR_LIST");
        assert_eq!(deps["selectorList"]["type"], "LABEL_LIST");
        // Only the attributes asked for, with no inputs or location.
        let mut narrow = fjfj_query::target_proto::ProtoOptions::default();
        assert!(
            narrow
                .flag("proto:output_rule_attrs", Some("name,opt"))
                .unwrap()
        );
        assert!(
            narrow
                .flag("noproto:rule_inputs_and_outputs", None)
                .unwrap()
        );
        assert!(narrow.flag("noproto:locations", None).unwrap());
        assert!(
            narrow
                .flag("proto:include_attribute_source_aspects", None)
                .unwrap()
        );
        let rule_narrow = rule(&narrow);
        let names: Vec<&str> = rule_narrow["attribute"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["name"].as_str().unwrap())
            .collect();
        // Bazel adds the digest of a Starlark rule class whatever is asked for.
        assert_eq!(names, ["name", "opt", "$rule_implementation_hash"]);
        assert!(rule_narrow.get("ruleInput").is_none());
        assert!(rule_narrow.get("location").is_none());
    }

    /// A rule the BUILD file called, one a two-level legacy macro made and
    /// one a native rule in a macro, on a root package and a subpackage:
    /// `bazel query --output=build` printed this for the same files.
    #[test]
    fn build_shows_the_rule_as_written_with_where_it_was_made_and_defined() {
        let (dir, repos) = workspace_of(&[
            ("MODULE.bazel", ""),
            (
                "macros.bzl",
                "def _impl(ctx):\n    return []\nr = rule(implementation = _impl, attrs = {\"deps\": attr.label_list(), \"s\": attr.string()})\n\ndef inner(name, **kwargs):\n    r(name = name + \"_in\", **kwargs)\n\ndef outer(name, deps = []):\n    inner(name = name, deps = deps, s = \"x\")\n    native.genrule(name = name + \"_gen\", outs = [name + \".out\"], cmd = \"echo > $@\")\n",
            ),
            (
                "BUILD",
                "load(\":macros.bzl\", \"outer\", \"r\")\nouter(name = \"m\")\nr(name = \"direct\")\n",
            ),
            (
                "sub/BUILD",
                "load(\"//:macros.bzl\", \"outer\")\nouter(name = \"n\")\n",
            ),
        ]);
        let graph = QueryGraph::new(repos);
        let ws = dir.path().join("ws").display().to_string();
        let text = query(
            &graph,
            "//:m_in + //:m_gen + //:direct + //sub:n_in",
            Format::Build,
            Order::Auto,
            Options::default(),
        )
        .replace(&ws, "<ws>");
        assert_eq!(
            text,
            r#"# <ws>/BUILD:3:2
r(
  name = "direct",
)
# Rule direct instantiated at (most recent call last):
#   <ws>/BUILD:3:2 in <toplevel>
# Rule r defined at (most recent call last):
#   <ws>/macros.bzl:3:9 in <toplevel>

# <ws>/BUILD:2:6
genrule(
  name = "m_gen",
  generator_name = "m",
  generator_function = "outer",
  generator_location = "<ws>/BUILD:2:6",
  outs = ["//:m.out"],
  cmd = "echo > $@",
)
# Rule m_gen instantiated at (most recent call last):
#   <ws>/BUILD:2:6        in <toplevel>
#   <ws>/macros.bzl:10:19 in outer

# <ws>/BUILD:2:6
r(
  name = "m_in",
  generator_name = "m",
  generator_function = "outer",
  generator_location = "<ws>/BUILD:2:6",
  deps = [],
  s = "x",
)
# Rule m_in instantiated at (most recent call last):
#   <ws>/BUILD:2:6       in <toplevel>
#   <ws>/macros.bzl:9:10 in outer
#   <ws>/macros.bzl:6:6  in inner
# Rule r defined at (most recent call last):
#   <ws>/macros.bzl:3:9 in <toplevel>

# <ws>/sub/BUILD:2:6
r(
  name = "n_in",
  generator_name = "n",
  generator_function = "outer",
  generator_location = "sub/BUILD:2:6",
  deps = [],
  s = "x",
)
# Rule n_in instantiated at (most recent call last):
#   <ws>/sub/BUILD:2:6   in <toplevel>
#   <ws>/macros.bzl:9:10 in outer
#   <ws>/macros.bzl:6:6  in inner
# Rule r defined at (most recent call last):
#   <ws>/macros.bzl:3:9 in <toplevel>

"#
        );
    }

    /// A rule of every kind of attribute, with `select()`s and a genrule:
    /// `bazel query --output=build` printed this for the same files.
    #[test]
    fn build_writes_every_kind_of_value_as_bazel_does() {
        let (dir, repos) = workspace_of(&[
            ("MODULE.bazel", ""),
            ("f.txt", "hi\n"),
            (
                "defs.bzl",
                "def _impl(ctx):\n    for f in ctx.outputs.ol + ([ctx.outputs.o] if ctx.outputs.o else []):\n        ctx.actions.write(f, '')\n    return []\nallk = rule(implementation = _impl, attrs = {\n    \"i\": attr.int(default = 7),\n    \"b\": attr.bool(default = True),\n    \"s\": attr.string(default = \"str\"),\n    \"l\": attr.label(allow_files = True),\n    \"ls\": attr.label_list(allow_files = True),\n    \"ss\": attr.string_list(),\n    \"il\": attr.int_list(),\n    \"sd\": attr.string_dict(),\n    \"sld\": attr.string_list_dict(),\n    \"lsd\": attr.label_keyed_string_dict(allow_files = True),\n    \"o\": attr.output(),\n    \"ol\": attr.output_list(),\n})\n",
            ),
            (
                "BUILD",
                "load(\":defs.bzl\", \"allk\")\nconfig_setting(name = \"opt\", values = {\"compilation_mode\": \"opt\"})\nallk(name = \"x\", i = 1, b = False, s = \"a\\\"b\\n>\", l = \"f.txt\", ls = [\"f.txt\", \":fg\"], ss = [\"p\", \"q\"], il = [1, 2], sd = {\"k\": \"v\"}, sld = {\"k\": [\"a\", \"b\"]}, lsd = {\"f.txt\": \"v\"}, o = \"out.txt\", ol = [\"o1\", \"o2\"], tags = [\"t1\"], testonly = True, visibility = [\"//visibility:public\"])\nallk(name = \"y\", ss = select({\":opt\": [\"o\"], \"//conditions:default\": [\"d\"]}), ls = select({\":opt\": [\"f.txt\"], \"//conditions:default\": []}) + [\":fg\"], deprecation = \"old\")\ngenrule(name = \"g\", srcs = [\"f.txt\"], outs = [\"g.out\"], cmd = \"cp $< $@\", stamp = 1, executable = False, tools = [\":y\"], visibility = [\"//visibility:private\"])\nfilegroup(name = \"fg\", srcs = [\"f.txt\"])\npackage_group(name = \"pg\", packages = [\"//a/...\"], includes = [])\nexports_files([\"f.txt\"])\n",
            ),
        ]);
        let graph = QueryGraph::new(repos);
        let ws = dir.path().join("ws").display().to_string();
        let text = query(
            &graph,
            "//:x + //:y + //:g + //:fg + f.txt + //:pg + //:g.out + //:opt",
            Format::Build,
            Order::Auto,
            Options::default(),
        )
        .replace(&ws, "<ws>");
        assert_eq!(
            text,
            r#"# <ws>/BUILD:6:10
filegroup(
  name = "fg",
  srcs = ["//:f.txt"],
)
# Rule fg instantiated at (most recent call last):
#   <ws>/BUILD:6:10 in <toplevel>

# <ws>/BUILD:5:8
genrule(
  name = "g",
  visibility = ["//visibility:private"],
  srcs = ["//:f.txt"],
  tools = ["//:y"],
  outs = ["//:g.out"],
  cmd = "cp $< $@",
  executable = False,
  stamp = 1,
)
# Rule g instantiated at (most recent call last):
#   <ws>/BUILD:5:8 in <toplevel>

# <ws>/BUILD:2:15
config_setting(
  name = "opt",
  values = {"compilation_mode": "opt"},
)
# Rule opt instantiated at (most recent call last):
#   <ws>/BUILD:2:15 in <toplevel>

# <ws>/BUILD:3:5
allk(
  name = "x",
  visibility = ["//visibility:public"],
  tags = ["t1"],
  testonly = True,
  i = 1,
  b = False,
  s = "a\"b\n>",
  l = "//:f.txt",
  ls = ["//:f.txt", "//:fg"],
  ss = ["p", "q"],
  il = [1, 2],
  sd = {"k": "v"},
  sld = {"k": ["a", "b"]},
  lsd = {"//:f.txt": "v"},
  o = "//:out.txt",
  ol = ["//:o1", "//:o2"],
)
# Rule x instantiated at (most recent call last):
#   <ws>/BUILD:3:5 in <toplevel>
# Rule allk defined at (most recent call last):
#   <ws>/defs.bzl:5:12 in <toplevel>

# <ws>/BUILD:4:5
allk(
  name = "y",
  deprecation = "old",
  ls = select({"//:opt": ["//:f.txt"], "//conditions:default": []}) + ["//:fg"],
  ss = select({"//:opt": ["o"], "//conditions:default": ["d"]}),
)
# Rule y instantiated at (most recent call last):
#   <ws>/BUILD:4:5 in <toplevel>
# Rule allk defined at (most recent call last):
#   <ws>/defs.bzl:5:12 in <toplevel>

"#
        );
    }

    #[test]
    fn graph_merges_equivalent_nodes_unless_told_not_to() {
        let (_dir, repos) = workspace_of(&[
            ("MODULE.bazel", ""),
            (
                "BUILD",
                "filegroup(name = \"x\", srcs = [\"p.txt\", \"q.txt\", \":y\"])\nfilegroup(name = \"y\", srcs = [\"q.txt\"])\n",
            ),
        ]);
        let graph = QueryGraph::new(repos);
        let ev = Evaluator::new(&graph, Options::default());
        let set = ev.eval(&fjfj_query::parse("deps(//:x)").unwrap()).unwrap();
        let draw = |options: &fjfj_query::output::GraphOptions| {
            fjfj_query::output::render_with(&ev, &set, Format::Graph, Order::Auto, '\n', options)
                .unwrap()
        };
        // //:p.txt is read by //:x alone and //:q.txt by //:x and //:y: not
        // equivalent. Nothing merges.
        let plain = draw(&Default::default());
        assert!(!plain.contains("\\n"), "{plain}");
        // With a second leaf read by //:x alone they are.
        let (_dir, repos) = workspace_of(&[
            ("MODULE.bazel", ""),
            (
                "BUILD",
                "filegroup(name = \"x\", srcs = [\"p.txt\", \"r.txt\"])\n",
            ),
        ]);
        let graph = QueryGraph::new(repos);
        let ev = Evaluator::new(&graph, Options::default());
        let set = ev.eval(&fjfj_query::parse("deps(//:x)").unwrap()).unwrap();
        let draw = |options: &fjfj_query::output::GraphOptions| {
            fjfj_query::output::render_with(&ev, &set, Format::Graph, Order::Auto, '\n', options)
                .unwrap()
        };
        let merged = draw(&Default::default());
        assert!(
            merged.contains("\"//:r.txt\\n//:p.txt\"")
                || merged.contains("\"//:p.txt\\n//:r.txt\""),
            "{merged}"
        );
        assert_eq!(merged.matches("//:x\" -> ").count(), 1);
        let mut apart = fjfj_query::output::GraphOptions::default();
        assert!(apart.flag("nograph:factored", None).unwrap());
        let separate = draw(&apart);
        assert!(!separate.contains("\\n"), "{separate}");
        assert_eq!(separate.matches("//:x\" -> ").count(), 2);
        // A node limit that cannot hold both says how many it left out.
        let mut short = fjfj_query::output::GraphOptions::default();
        assert!(short.flag("graph:node_limit", Some("9")).unwrap());
        assert!(draw(&short).contains("\"//:p.txt\\n...and 1 more items\""));
    }

    #[test]
    fn relative_locations_start_at_the_root_of_the_repository() {
        let (dir, repos) = workspace();
        let ws = dir.path().join("ws").display().to_string();
        let locations = |relative: bool| {
            let graph = QueryGraph::new(repos.clone()).with_relative_locations(relative);
            query(
                &graph,
                "//a:lib + //a:a.txt",
                Format::Location,
                Order::Auto,
                Options::default(),
            )
        };
        assert_eq!(
            locations(true),
            "a/a.txt:1:1: source file //a:a.txt\na/BUILD:6:6: mylib rule //a:lib\n"
        );
        assert!(locations(false).starts_with(&format!("{ws}/a/a.txt:1:1: ")));
    }

    #[test]
    fn consistent_labels_name_every_repository_canonically() {
        let (_dir, repos) = workspace();
        let labels = |consistent: bool| {
            let graph = QueryGraph::new(repos.clone()).with_consistent_labels(consistent);
            query(
                &graph,
                "//a:lib + @bazel_tools//tools/genrule:genrule-setup.sh",
                Format::Label,
                Order::Auto,
                Options::default(),
            )
        };
        assert_eq!(
            labels(false),
            "//a:lib\n@bazel_tools//tools/genrule:genrule-setup.sh\n"
        );
        assert_eq!(
            labels(true),
            "@@//a:lib\n@@bazel_tools//tools/genrule:genrule-setup.sh\n"
        );
    }

    /// What `bazel query --output=streamed_jsonproto` named the attributes of
    /// a Starlark test and a Starlark executable with a private attribute.
    #[test]
    fn executables_and_tests_have_the_implicit_attributes_bazel_gives_them() {
        let (_dir, repos) = workspace_of(&[
            ("MODULE.bazel", ""),
            (
                "defs.bzl",
                "def _impl(ctx):\n    return []\nmy_test = rule(implementation = _impl, test = True, attrs = {\"_tool\": attr.label(default = \"//:t\")})\nmy_bin = rule(implementation = _impl, executable = True)\n",
            ),
            (
                "BUILD",
                "load(\":defs.bzl\", \"my_test\", \"my_bin\")\nfilegroup(name = \"t\")\nmy_test(name = \"tt\", size = \"large\")\nmy_bin(name = \"bb\")\n",
            ),
        ]);
        let graph = QueryGraph::new(repos);
        let rules = jsonproto(&graph, "//:tt + //:bb", &Default::default());
        let rule =
            |name: &str| rules.iter().find(|t| t["rule"]["name"] == name).unwrap()["rule"].clone();
        let names = |rule: &serde_json::Value| -> Vec<String> {
            rule["attribute"]
                .as_array()
                .unwrap()
                .iter()
                .map(|a| a["name"].as_str().unwrap().to_owned())
                .filter(|n| n.starts_with(['$', ':']))
                .collect()
        };
        assert_eq!(
            names(&rule("//:bb")),
            [
                "$config_dependencies",
                "$is_executable",
                ":action_listener",
                "$rule_implementation_hash"
            ]
        );
        let test = rule("//:tt");
        assert_eq!(
            names(&test),
            [
                "$collect_coverage_script",
                "$config_dependencies",
                "$is_executable",
                "$test_runtime",
                "$test_setup_script",
                "$test_wrapper",
                "$tool",
                "$xml_generator_script",
                "$xml_writer",
                ":action_listener",
                ":coverage_report_generator",
                ":coverage_support",
                ":run_under_exec_config",
                ":run_under_target_config",
                "$rule_implementation_hash"
            ]
        );
        // The test infrastructure is what it reads; a private attribute's
        // target is too.
        assert_eq!(
            test["ruleInput"],
            serde_json::json!([
                "//:t",
                "@bazel_tools//tools/test:collect_coverage",
                "@bazel_tools//tools/test:coverage_report_generator",
                "@bazel_tools//tools/test:coverage_support",
                "@bazel_tools//tools/test:runtime",
                "@bazel_tools//tools/test:test_setup",
                "@bazel_tools//tools/test:test_wrapper",
                "@bazel_tools//tools/test:test_xml_generator",
                "@bazel_tools//tools/test:xml_writer"
            ])
        );
        // A large test has the long timeout.
        let timeout = test["attribute"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["name"] == "timeout")
            .unwrap();
        assert_eq!(timeout["stringValue"], "long");
    }

    #[test]
    fn package_lists_each_package_once() {
        let (_dir, repos) = workspace();
        let graph = QueryGraph::new(repos);
        let packages = |q: &str| query(&graph, q, Format::Package, Order::Auto, Options::default());
        assert_eq!(
            packages("//a:lib + //a:gen + //c/d:leaf + //b:lib"),
            "a\nb\nc/d\n"
        );
        assert_eq!(packages("//a:a.txt"), "a\n");
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
