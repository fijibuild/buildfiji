//! The graph a query runs over: targets and the edges between them, as the
//! loading phase gives them. The crate does no I/O; a [`Graph`] supplies it.

use fjfj_graph::Label;
use fjfj_graph::rule::{AttrType, AttrValue};
use std::sync::Arc;

/// One call of a stack, as `query --output=build` and `--proto:instantiation_stack`
/// show it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// `/abs/path/BUILD:2:6`.
    pub location: String,
    /// `BUILD:2:6`, from the root of the repository.
    pub relative: String,
    /// The function the call was made in.
    pub function: String,
}

/// What kind of target a node is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeKind {
    Rule {
        class: String,
        /// A test rule: `*_test`, or built with `test = True`.
        test: bool,
        /// A rule that makes an executable.
        executable: bool,
    },
    SourceFile,
    GeneratedFile {
        rule: Label,
    },
    PackageGroup,
}

impl NodeKind {
    /// What `kind()` matches and `--output=label_kind` prints before the
    /// label.
    pub fn description(&self) -> String {
        match self {
            NodeKind::Rule { class, .. } => format!("{class} rule"),
            NodeKind::SourceFile => "source file".to_owned(),
            NodeKind::GeneratedFile { .. } => "generated file".to_owned(),
            NodeKind::PackageGroup => "package group".to_owned(),
        }
    }
}

/// An attribute of a rule as `attr()` and `labels()` see it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeAttr {
    pub name: String,
    /// What `attr()` matches against: a string as is, a number as digits, a
    /// boolean as `0` or `1`, a list as `[a, b]`.
    pub text: String,
    /// The labels the attribute names.
    pub labels: Vec<Label>,
    /// Set by the BUILD file, as `--output=build` shows.
    pub explicit: bool,
    pub ty: AttrType,
    /// The value, `select()`s included.
    pub value: AttrValue,
    /// The attribute has no value: a label or an output the rule did not
    /// give and the class has no default for. `value` is then empty.
    pub unset: bool,
}

/// A dependency of a target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub to: Label,
    /// From an attribute whose name starts with `_`, or a default the user
    /// did not write.
    pub implicit: bool,
    /// From an attribute built for the execution platform.
    pub tool: bool,
    /// For `--output=graph`: the `select()` condition that brings it in.
    pub condition: Option<String>,
    /// Comes from the `visibility` attribute, which `--output=xml` does not
    /// list among a rule's inputs.
    pub visibility: bool,
    /// The attribute it comes from, empty if none.
    pub attr: String,
    /// From an attribute whose `cfg` is a transition.
    pub transition: bool,
}

impl Node {
    /// The location as an output shows it.
    pub fn shown_location(&self, graph: &dyn Graph) -> &str {
        if graph.relative_locations() {
            &self.relative_location
        } else {
            &self.location
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub label: Label,
    pub kind: NodeKind,
    /// `/path/BUILD:5:6`.
    pub location: String,
    /// The same from the root of the repository: `BUILD:5:6`.
    pub relative_location: String,
    /// Every attribute, in the class's order.
    pub attrs: Vec<NodeAttr>,
    /// In the order Bazel visits them.
    pub edges: Vec<Edge>,
    /// For a rule: the files it declares it makes.
    pub outputs: Vec<Label>,
    /// Who may see the target, as `visibility` is written.
    pub visibility: Vec<String>,
    /// For a BUILD file: the `.bzl` files it loads.
    pub loads: Vec<Label>,
    /// This is the BUILD file of a package.
    pub build_file: bool,
    /// For a rule: the `config_setting`s its `select()`s read, once each in the
    /// order they were written (Bazel's `$config_dependencies`).
    pub config_deps: Vec<Label>,
    /// For a rule: the calls that led to it, outermost first, each as the
    /// absolute `file:line:col` it was made at and the function it was made in.
    pub stack: Vec<Frame>,
    /// For a rule of a class written in Starlark: the calls that defined the
    /// class, in the same form.
    pub definition_stack: Vec<Frame>,
    /// For a `package_group`: the other groups it includes and its package
    /// specifications, as written.
    pub group: Option<(Vec<Label>, Vec<String>)>,
}

/// Where a query's targets come from.
pub trait Graph: Sync {
    /// The targets a pattern (`//a:b`, `//a/...`, `//a:all`) selects, or
    /// Bazel's error.
    fn pattern(&self, text: &str) -> Result<Vec<Label>, String>;

    /// A target loaded.
    fn node(&self, label: &Label) -> Result<Arc<Node>, String>;

    /// The node as the BUILD file declares it: for `cquery`, with the edges
    /// of every branch of a `select()`, which is what a rule's `rule_input`
    /// lists, where `node` has only those of the branch taken.
    fn declared_node(&self, label: &Label) -> Result<Arc<Node>, String> {
        self.node(label)
    }

    /// Every target of the package of `label`, files included.
    fn siblings(&self, label: &Label) -> Result<Vec<Label>, String>;

    /// The BUILD file of the package of `label` and the `.bzl` files it
    /// loads, directly or not, with the BUILD files of their packages.
    fn build_files(&self, label: &Label) -> Result<Vec<Label>, String>;

    /// The `.bzl` files the BUILD file of the package of `label` loads,
    /// directly or not.
    fn load_files(&self, label: &Label) -> Result<Vec<Label>, String>;

    /// Whether `to` may be seen from `from`.
    fn visible(&self, from: &Label, to: &Label) -> Result<bool, String>;

    /// How an output format names a node: the label, unless the graph's nodes
    /// are more than labels (`cquery` adds the configuration).
    fn output_name(&self, label: &Label) -> String {
        self.display(label)
    }

    /// Whether `--output=graph` lists the edges of a node by the name of what
    /// they lead to, as `cquery` does, rather than in the order they were
    /// visited.
    fn sorts_edges(&self) -> bool {
        false
    }

    /// For `cquery`'s `config()`: whether `name` (`target`, `null`, or the
    /// start of a checksum) identifies one configuration of the graph.
    fn is_configuration(&self, _name: &str) -> bool {
        false
    }

    /// For `config()`: whether the node `label` is in the configuration `name`
    /// identifies.
    fn in_configuration(&self, _label: &Label, _name: &str) -> bool {
        false
    }

    /// Whether `--relative_locations` is on: locations in the outputs that
    /// have them are from the root of the repository.
    fn relative_locations(&self) -> bool {
        false
    }

    /// A label as the user reads it (`@repo//p:t`).
    fn display(&self, label: &Label) -> String {
        if label.repo.is_empty() {
            format!("//{}:{}", label.package, label.name)
        } else {
            format!("@{}//{}:{}", label.repo, label.package, label.name)
        }
    }
}
