//! The graph a query runs over: targets and the edges between them, as the
//! loading phase gives them. The crate does no I/O; a [`Graph`] supplies it.

use fjfj_graph::Label;
use fjfj_graph::rule::{AttrType, AttrValue};
use std::sync::Arc;

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub label: Label,
    pub kind: NodeKind,
    /// `/path/BUILD:5:6`.
    pub location: String,
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

    /// A label as the user reads it (`@repo//p:t`).
    fn display(&self, label: &Label) -> String {
        if label.repo.is_empty() {
            format!("//{}:{}", label.package, label.name)
        } else {
            format!("@{}//{}:{}", label.repo, label.package, label.name)
        }
    }
}
