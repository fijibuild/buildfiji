//! Evaluating a query expression.

use crate::ast::{Arg, Call, Expr, Function, Op};
use crate::graph::{Edge, Graph, Node, NodeKind};
use fjfj_graph::Label;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::sync::{Arc, Mutex};

/// The set a query denotes, in label order.
pub type Set = BTreeSet<Label>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// `--[no]implicit_deps`.
    pub implicit_deps: bool,
    /// `--[no]tool_deps`.
    pub tool_deps: bool,
    /// `--[no]nodep_deps`.
    pub nodep_deps: bool,
    /// `--keep_going`: a pattern that fails is skipped (see
    /// [`Evaluator::skipped`]) instead of failing the query.
    pub keep_going: bool,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            implicit_deps: true,
            tool_deps: true,
            nodep_deps: true,
            keep_going: false,
        }
    }
}

pub struct Evaluator<'g> {
    graph: &'g dyn Graph,
    options: Options,
    nodes: Mutex<HashMap<Label, Arc<Node>>>,
    /// The order the result of the query is listed in, when it is not label
    /// order: see [`Evaluator::eval_ordered`].
    listing: Vec<Label>,
    /// The patterns `--keep_going` skipped, as `ERROR: Skipping ...` lines.
    skipped: Mutex<Vec<String>>,
}

impl<'g> Evaluator<'g> {
    pub fn new(graph: &'g dyn Graph, options: Options) -> Evaluator<'g> {
        Evaluator {
            graph,
            options,
            nodes: Mutex::new(HashMap::new()),
            listing: Vec::new(),
            skipped: Mutex::new(Vec::new()),
        }
    }

    /// The error lines of the patterns skipped under `--keep_going`; empty
    /// if the result is complete.
    pub fn skipped(&self) -> Vec<String> {
        self.skipped.lock().unwrap().clone()
    }

    /// List results that are in `listing` in its order, the rest after them
    /// in label order.
    pub fn with_listing(mut self, listing: Vec<Label>) -> Evaluator<'g> {
        self.listing = listing;
        self
    }

    /// `set` in the order of the listing.
    pub fn listed(&self, set: &Set) -> Vec<Label> {
        let mut out: Vec<Label> = self
            .listing
            .iter()
            .filter(|l| set.contains(*l))
            .cloned()
            .collect();
        let placed: BTreeSet<&Label> = out.iter().collect();
        let rest: Vec<Label> = set
            .iter()
            .filter(|l| !placed.contains(l))
            .cloned()
            .collect();
        out.extend(rest);
        out
    }

    /// The result of `expr` in the order Bazel's cquery lists it: a union
    /// keeps its operands' order, an intersection and a difference the order
    /// of the left operand; anything a function computes is in label order.
    pub fn eval_ordered(&self, expr: &Expr) -> Result<Vec<Label>, String> {
        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        let mut push = |label: Label| {
            if seen.insert(label.clone()) {
                out.push(label);
            }
        };
        match expr {
            // A pattern's targets are in label order.
            Expr::Word(word) => self
                .graph
                .pattern(word)?
                .into_iter()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .for_each(&mut push),
            Expr::Set(words) => {
                for word in words {
                    self.graph
                        .pattern(word)?
                        .into_iter()
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .for_each(&mut push);
                }
            }
            Expr::Binary(Op::Union, l, r) => {
                self.eval_ordered(l)?.into_iter().for_each(&mut push);
                self.eval_ordered(r)?.into_iter().for_each(&mut push);
            }
            Expr::Binary(op, l, r) => {
                let right = self.eval(r)?;
                for label in self.eval_ordered(l)? {
                    if right.contains(&label) == (*op == Op::Intersect) {
                        push(label);
                    }
                }
            }
            _ => self.eval(expr)?.into_iter().for_each(&mut push),
        }
        Ok(out)
    }

    pub fn graph(&self) -> &'g dyn Graph {
        self.graph
    }

    pub fn options(&self) -> Options {
        self.options
    }

    /// The node of `label`, loaded once.
    pub fn node(&self, label: &Label) -> Result<Arc<Node>, String> {
        if let Some(done) = self.nodes.lock().expect("nodes").get(label) {
            return Ok(done.clone());
        }
        let node = self.graph.node(label)?;
        self.nodes
            .lock()
            .expect("nodes")
            .insert(label.clone(), node.clone());
        Ok(node)
    }

    /// Load `labels` together. What fails here fails again, in order, when
    /// the evaluator asks for the node.
    fn preload<'a>(&self, labels: impl IntoIterator<Item = &'a Label>) {
        let wanted: Vec<&Label> = labels
            .into_iter()
            .filter(|l| !self.nodes.lock().expect("nodes").contains_key(*l))
            .collect();
        if wanted.len() < 2 {
            return;
        }
        let next = std::sync::atomic::AtomicUsize::new(0);
        let threads = std::thread::available_parallelism()
            .map_or(4, |n| n.get())
            .min(16);
        std::thread::scope(|scope| {
            for _ in 0..threads.min(wanted.len()) {
                scope.spawn(|| {
                    loop {
                        let at = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some(label) = wanted.get(at) else { break };
                        let _ = self.node(label);
                    }
                });
            }
        });
    }

    /// The edges of `node` the options keep, without repeats: a target that
    /// an aspect's attribute and the rule's both name is two.
    pub fn edges(&self, node: &Node) -> Vec<Edge> {
        let mut seen = BTreeSet::new();
        node.edges
            .iter()
            .filter(|e| {
                (self.options.implicit_deps || !e.implicit)
                    && (self.options.tool_deps || !e.tool)
                    && (self.options.nodep_deps || !e.nodep)
            })
            .filter(|e| seen.insert((e.to.clone(), e.aspect)))
            .cloned()
            .collect()
    }

    pub fn eval(&self, expr: &Expr) -> Result<Set, String> {
        self.eval_in(expr, &mut Vec::new())
    }

    fn eval_in(&self, expr: &Expr, env: &mut Vec<(String, Set)>) -> Result<Set, String> {
        match expr {
            Expr::Word(word) => match self.graph.pattern(word) {
                Ok(labels) => Ok(labels.into_iter().collect()),
                Err(e) if self.options.keep_going => {
                    self.skipped
                        .lock()
                        .unwrap()
                        .push(format!("ERROR: Skipping '{word}': {e}"));
                    Ok(Set::new())
                }
                Err(e) => Err(e),
            },
            Expr::Variable(name) => env
                .iter()
                .rev()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.clone())
                .ok_or_else(|| {
                    format!(
                        "Evaluation of subquery \"${name}\" failed (did you want to use --keep_going?): undefined variable '{name}'"
                    )
                }),
            Expr::Set(words) => {
                let mut out = Set::new();
                for word in words {
                    out.extend(self.graph.pattern(word)?);
                }
                Ok(out)
            }
            Expr::Binary(op, l, r) => {
                // Both sides are evaluated, and when both fail it is the right
                // one that Bazel reports.
                let left = self.eval_in(l, env);
                let right = self.eval_in(r, env)?;
                let left = left?;
                Ok(match op {
                    Op::Union => left.union(&right).cloned().collect(),
                    Op::Intersect => left.intersection(&right).cloned().collect(),
                    Op::Except => left.difference(&right).cloned().collect(),
                })
            }
            Expr::Let { name, value, body } => {
                let value = self.eval_in(value, env)?;
                env.push((name.clone(), value));
                let out = self.eval_in(body, env);
                env.pop();
                out
            }
            Expr::Call(call) => self.call(call, env),
        }
    }

    fn expr_arg(
        &self,
        call: &Call,
        at: usize,
        env: &mut Vec<(String, Set)>,
    ) -> Result<Set, String> {
        match &call.args[at] {
            Arg::Expr(e) => self.eval_in(e, env),
            _ => Err(format!(
                "{}: argument {at} is not an expression",
                call.function.name()
            )),
        }
    }

    fn word_arg(call: &Call, at: usize) -> &str {
        match &call.args[at] {
            Arg::Word(w) => w,
            _ => "",
        }
    }

    fn depth_arg(call: &Call, at: usize) -> Option<usize> {
        match call.args.get(at) {
            Some(Arg::Int(n)) => Some(*n as usize),
            _ => None,
        }
    }

    fn regex(&self, function: &str, pattern: &str) -> Result<Regex, String> {
        Regex::new(pattern).map_err(|e| {
            format!(
                "illegal '{function}' pattern regexp '{pattern}': {}",
                e.to_string().lines().last().unwrap_or("").trim()
            )
        })
    }

    fn call(&self, call: &Call, env: &mut Vec<(String, Set)>) -> Result<Set, String> {
        match call.function {
            Function::Deps => {
                let roots = self.expr_arg(call, 0, env)?;
                self.deps(&roots, Self::depth_arg(call, 1))
            }
            Function::RDeps => {
                let universe = self.expr_arg(call, 0, env)?;
                let targets = self.expr_arg(call, 1, env)?;
                self.rdeps(&universe, &targets, Self::depth_arg(call, 2))
            }
            Function::SomePath => {
                let from = self.expr_arg(call, 0, env)?;
                let to = self.expr_arg(call, 1, env)?;
                self.some_path(&from, &to)
            }
            Function::AllPaths => {
                let from = self.expr_arg(call, 0, env)?;
                let to = self.expr_arg(call, 1, env)?;
                let reach = self.deps(&from, None)?;
                let back = self.rdeps(&reach, &to, None)?;
                Ok(reach.intersection(&back).cloned().collect())
            }
            // The actions they keep are chosen by whoever prints them; as an
            // expression they stand for the targets they were given.
            Function::Inputs | Function::Mnemonic | Function::Outputs => {
                if call.args.len() != 2 {
                    return Err(format!(
                        "Evaluation of query \"{}('{}')\" failed: aquery filter functions (inputs, outputs, mnemonics) must have exactly 2 arguments,except when --skyframe_state is used.",
                        call.function.name(),
                        Self::word_arg(call, 0)
                    ));
                }
                self.expr_arg(call, 1, env)
            }
            Function::Config => {
                let name = Self::word_arg(call, 1);
                if name == "host" {
                    return Err("Evaluation failed: 'host' configuration no longer exists. Use a specific configuration hash instead".to_owned());
                }
                if !self.graph.is_configuration(name) {
                    return Err(format!(
                        "Evaluation failed: Unknown configuration ID '{name}'.\nconfig()'s second argument must identify a unique configuration.\n\nValid values:\n 'target' for the default configuration\n 'null' for source files (which have no configuration)\n an arbitrary configuration's full or short ID\n\nA short ID is any prefix of a full ID. cquery shows short IDs. 'bazel config' shows full IDs.\n\nFor more help, see https://bazel.build/docs/cquery."
                    ));
                }
                let input = self.expr_arg(call, 0, env)?;
                let out: Set = input
                    .into_iter()
                    .filter(|l| self.graph.in_configuration(l, name))
                    .collect();
                if out.is_empty() {
                    let Arg::Expr(expr) = &call.args[0] else {
                        unreachable!("config() takes an expression first")
                    };
                    let place = if name == "target" || name == "null" {
                        format!("the '{name}' configuration")
                    } else {
                        format!("the configuration with checksum '{name}'")
                    };
                    return Err(format!(
                        "Evaluation failed: No target (in) {} could be found in {place}",
                        expr.canonical()
                    ));
                }
                Ok(out)
            }
            Function::Kind => {
                let pattern = self.regex("kind", Self::word_arg(call, 0))?;
                let input = self.expr_arg(call, 1, env)?;
                let mut out = Set::new();
                for label in input {
                    if pattern.is_match(&self.node(&label)?.kind.description()) {
                        out.insert(label);
                    }
                }
                Ok(out)
            }
            Function::Filter => {
                let pattern = self.regex("filter", Self::word_arg(call, 0))?;
                let input = self.expr_arg(call, 1, env)?;
                Ok(input
                    .into_iter()
                    .filter(|l| pattern.is_match(&self.graph.display(l)))
                    .collect())
            }
            Function::Attr => {
                let name = Self::word_arg(call, 0);
                let pattern = self.regex("attr", Self::word_arg(call, 1))?;
                let input = self.expr_arg(call, 2, env)?;
                let mut out = Set::new();
                for label in input {
                    let node = self.node(&label)?;
                    if !matches!(node.kind, NodeKind::Rule { .. }) {
                        continue;
                    }
                    if node.attrs.iter().any(|a| {
                        a.name == name
                            && !a.unset
                            && !a.name.starts_with(['_', '$'])
                            && pattern.is_match(&a.text)
                    }) {
                        out.insert(label);
                    }
                }
                Ok(out)
            }
            Function::Labels => {
                let name = Self::word_arg(call, 0);
                let input = self.expr_arg(call, 1, env)?;
                let mut out = Set::new();
                for label in input {
                    let node = self.node(&label)?;
                    if let Some(attr) = node.attrs.iter().find(|a| a.name == name) {
                        out.extend(attr.labels.iter().cloned());
                    }
                }
                Ok(out)
            }
            Function::Tests => {
                let input = self.expr_arg(call, 0, env)?;
                let mut out = Set::new();
                let mut seen = BTreeSet::new();
                for label in input {
                    self.tests_of(&label, &mut seen, &mut out)?;
                }
                Ok(out)
            }
            Function::Some => {
                let input = self.expr_arg(call, 0, env)?;
                match input.into_iter().next() {
                    Some(first) => Ok(Set::from([first])),
                    None => Err("argument set is empty".to_owned()),
                }
            }
            Function::Siblings => {
                let input = self.expr_arg(call, 0, env)?;
                let mut out = Set::new();
                for label in input {
                    out.extend(self.graph.siblings(&label)?);
                }
                Ok(out)
            }
            Function::SamePkgDirectRDeps => {
                let input = self.expr_arg(call, 0, env)?;
                let mut out = Set::new();
                for label in &input {
                    for sibling in self.graph.siblings(label)? {
                        let node = self.node(&sibling)?;
                        if self.edges(&node).iter().any(|e| e.to == *label) {
                            out.insert(sibling);
                        }
                    }
                }
                Ok(out)
            }
            Function::BuildFiles => {
                let input = self.expr_arg(call, 0, env)?;
                let mut out = Set::new();
                for label in input {
                    out.extend(self.graph.build_files(&label)?);
                }
                Ok(out)
            }
            Function::LoadFiles => {
                let input = self.expr_arg(call, 0, env)?;
                let mut out = Set::new();
                for label in input {
                    out.extend(self.graph.load_files(&label)?);
                }
                Ok(out)
            }
            Function::Executables => {
                let input = self.expr_arg(call, 0, env)?;
                let mut out = Set::new();
                for label in input {
                    if matches!(
                        self.node(&label)?.kind,
                        NodeKind::Rule {
                            executable: true,
                            ..
                        }
                    ) {
                        out.insert(label);
                    }
                }
                Ok(out)
            }
            Function::Visible => {
                let preds = self.expr_arg(call, 0, env)?;
                let input = self.expr_arg(call, 1, env)?;
                let mut out = Set::new();
                'targets: for label in input {
                    for pred in &preds {
                        if !self.graph.visible(pred, &label)? {
                            continue 'targets;
                        }
                    }
                    out.insert(label);
                }
                Ok(out)
            }
        }
    }

    /// `roots` and what they depend on, within `depth` edges if given.
    pub fn deps(&self, roots: &Set, depth: Option<usize>) -> Result<Set, String> {
        let mut seen: Set = roots.clone();
        let mut frontier: Vec<Label> = roots.iter().cloned().collect();
        let mut level = 0;
        while !frontier.is_empty() && depth.is_none_or(|d| level < d) {
            self.preload(&frontier);
            let mut next = Vec::new();
            for label in &frontier {
                let node = self.node(label)?;
                for edge in self.edges(&node) {
                    if seen.insert(edge.to.clone()) {
                        next.push(edge.to);
                    }
                }
            }
            frontier = next;
            level += 1;
        }
        Ok(seen)
    }

    /// The targets of `universe`'s closure that depend on `targets`.
    pub fn rdeps(
        &self,
        universe: &Set,
        targets: &Set,
        depth: Option<usize>,
    ) -> Result<Set, String> {
        let closure = self.deps(universe, None)?;
        let mut reverse: BTreeMap<Label, Vec<Label>> = BTreeMap::new();
        for label in &closure {
            let node = self.node(label)?;
            for edge in self.edges(&node) {
                reverse.entry(edge.to).or_default().push(label.clone());
            }
        }
        let mut seen: Set = targets.intersection(&closure).cloned().collect();
        let mut frontier: Vec<Label> = seen.iter().cloned().collect();
        let mut level = 0;
        while !frontier.is_empty() && depth.is_none_or(|d| level < d) {
            let mut next = Vec::new();
            for label in &frontier {
                for parent in reverse.get(label).into_iter().flatten() {
                    if seen.insert(parent.clone()) {
                        next.push(parent.clone());
                    }
                }
            }
            frontier = next;
            level += 1;
        }
        Ok(seen)
    }

    /// A shortest path from a target of `from` to one of `to`.
    fn some_path(&self, from: &Set, to: &Set) -> Result<Set, String> {
        let mut parent: BTreeMap<Label, Option<Label>> = BTreeMap::new();
        let mut queue: VecDeque<Label> = VecDeque::new();
        for start in from {
            parent.insert(start.clone(), None);
            queue.push_back(start.clone());
        }
        while let Some(label) = queue.pop_front() {
            if to.contains(&label) {
                let mut path = Set::new();
                let mut at = Some(label);
                while let Some(l) = at {
                    at = parent.get(&l).cloned().flatten();
                    path.insert(l);
                }
                return Ok(path);
            }
            let node = self.node(&label)?;
            for edge in self.edges(&node) {
                if !parent.contains_key(&edge.to) {
                    parent.insert(edge.to.clone(), Some(label.clone()));
                    queue.push_back(edge.to);
                }
            }
        }
        Ok(Set::new())
    }

    /// The tests `label` stands for: itself if a test, the tests a
    /// `test_suite` lists or, with none listed, those of its package.
    fn tests_of(
        &self,
        label: &Label,
        seen: &mut BTreeSet<Label>,
        out: &mut Set,
    ) -> Result<(), String> {
        if !seen.insert(label.clone()) {
            return Ok(());
        }
        let node = self.node(label)?;
        let NodeKind::Rule { class, test, .. } = &node.kind else {
            return Ok(());
        };
        if class == "test_suite" {
            let listed = node.attrs.iter().find(|a| a.name == "tests");
            match listed {
                Some(tests) if !tests.labels.is_empty() => {
                    for l in &tests.labels {
                        self.tests_of(l, seen, out)?;
                    }
                }
                _ => {
                    for sibling in self.graph.siblings(label)? {
                        if sibling == *label {
                            continue;
                        }
                        if let NodeKind::Rule { test: true, .. } = self.node(&sibling)?.kind {
                            out.insert(sibling);
                        }
                    }
                }
            }
        } else if *test {
            out.insert(label.clone());
        }
        Ok(())
    }
}
