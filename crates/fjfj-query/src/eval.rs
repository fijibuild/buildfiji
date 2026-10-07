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
    /// The transitive closure of `--universe_scope`: what a universe query
    /// can see. `None` for a query that has no universe.
    universe: Option<Set>,
    /// The packages of the universe: a pattern outside them names nothing.
    universe_packages: BTreeSet<(String, String)>,
    /// The `--universe_scope` patterns that did not load, as `ERROR:` lines,
    /// when they do not just fail the query.
    scope_errors: Vec<String>,
    /// The directories the scope patterns that end in `...` cover: only a
    /// `...` pattern beneath one of them can be read in a universe.
    scope_trees: Vec<String>,
    /// What the query warns of, as `WARNING: ...` lines.
    warnings: Mutex<Vec<String>>,
}

impl<'g> Evaluator<'g> {
    pub fn new(graph: &'g dyn Graph, options: Options) -> Evaluator<'g> {
        Evaluator {
            graph,
            options,
            nodes: Mutex::new(HashMap::new()),
            listing: Vec::new(),
            skipped: Mutex::new(Vec::new()),
            universe: None,
            universe_packages: BTreeSet::new(),
            scope_errors: Vec::new(),
            scope_trees: Vec::new(),
            warnings: Mutex::new(Vec::new()),
        }
    }

    /// Run over the closure of `patterns` rather than over everything: a
    /// target outside it has no dependencies and no reverse dependencies. A
    /// pattern that does not load is the error, or under `--keep_going` is
    /// skipped.
    pub fn with_universe(mut self, patterns: &[String]) -> Result<Evaluator<'g>, String> {
        let mut roots = Set::new();
        for pattern in patterns {
            match self.graph.pattern(pattern) {
                Ok(labels) => roots.extend(labels),
                Err(e) => {
                    let line = format!("ERROR: Skipping '{pattern}': {e}");
                    match self.options.keep_going {
                        true => self.skipped.lock().unwrap().push(line),
                        false => self.scope_errors.push(line),
                    }
                }
            }
        }
        self.scope_trees = patterns.iter().filter_map(|p| Self::tree_of(p)).collect();
        let universe = self.deps(&roots, None)?;
        self.universe_packages = universe
            .iter()
            .map(|l| (l.repo.clone(), l.package.clone()))
            .collect();
        self.universe = Some(universe);
        Ok(self)
    }

    /// The `--universe_scope` patterns that failed to load, which fail the
    /// query once it has run.
    pub fn scope_errors(&self) -> &[String] {
        &self.scope_errors
    }

    /// The directory a main repository pattern that ends in `...` is below,
    /// empty for the root.
    fn tree_of(pattern: &str) -> Option<String> {
        let dir = pattern.strip_prefix("//")?.strip_suffix("...")?;
        Some(dir.trim_end_matches('/').to_owned())
    }

    /// The targets `word` names, in a universe only those of its packages.
    fn pattern(&self, word: &str) -> Result<Set, String> {
        if self.universe.is_some()
            && let Some(dir) = Self::tree_of(word)
            && !self
                .scope_trees
                .iter()
                .any(|t| t.is_empty() || dir == *t || dir.starts_with(&format!("{t}/")))
        {
            return Err(format!("no targets found beneath '{dir}'"));
        }
        let labels = self.graph.pattern(word)?;
        if self.universe.is_none() || labels.is_empty() {
            return Ok(labels.into_iter().collect());
        }
        let inside: Set = labels
            .iter()
            .filter(|l| {
                self.universe_packages
                    .contains(&(l.repo.clone(), l.package.clone()))
            })
            .cloned()
            .collect();
        if !inside.is_empty() {
            return Ok(inside);
        }
        match word.strip_suffix("/...") {
            Some(dir) => Err(format!(
                "no targets found beneath '{}'",
                dir.trim_start_matches('/').trim_start_matches('@')
            )),
            None => Err(format!(
                "no such package '{}': BUILD file not found on package path",
                labels[0].package
            )),
        }
    }

    /// The lines of warning the query printed.
    pub fn warnings(&self) -> Vec<String> {
        self.warnings.lock().unwrap().clone()
    }

    /// Split `targets` into those in the universe and those not; all are in
    /// the first without a universe. A target the graph lacks is warned of.
    fn within_universe(&self, targets: &Set, warn: bool) -> (Set, Set) {
        let Some(universe) = &self.universe else {
            return (targets.clone(), Set::new());
        };
        let (inside, outside): (Set, Set) =
            targets.iter().cloned().partition(|l| universe.contains(l));
        if warn && !outside.is_empty() {
            let names: Vec<String> = outside.iter().map(|l| self.graph.display(l)).collect();
            self.warnings.lock().unwrap().push(format!(
                "WARNING: Targets were missing from graph: [{}]",
                names.join(", ")
            ));
        }
        (inside, outside)
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
        let out = self.eval_in(expr, &mut Vec::new());
        // A universe query that is one pattern fails on the query, not on a
        // subquery of it.
        match (expr, out) {
            (Expr::Word(word), Err(e)) if self.universe.is_some() => {
                let subquery = format!(
                    "Evaluation of subquery \"{word}\" failed (did you want to use --keep_going?): "
                );
                Err(match e.strip_prefix(&subquery) {
                    Some(rest) => format!("Evaluation of query \"{word}\" failed: {rest}"),
                    None => e,
                })
            }
            (_, out) => out,
        }
    }

    fn eval_in(&self, expr: &Expr, env: &mut Vec<(String, Set)>) -> Result<Set, String> {
        match expr {
            Expr::Word(word) => match self.pattern(word) {
                Ok(labels) => Ok(labels),
                Err(e) if self.options.keep_going => {
                    let line = match self.universe {
                        Some(_) => format!("ERROR: Evaluation of query \"{word}\" failed: {e}"),
                        None => format!("ERROR: Skipping '{word}': {e}"),
                    };
                    self.skipped.lock().unwrap().push(line);
                    Ok(Set::new())
                }
                Err(e) if self.universe.is_some() => Err(format!(
                    "Evaluation of subquery \"{word}\" failed (did you want to use --keep_going?): {e}"
                )),
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
                    out.extend(self.pattern(word).map_err(|e| match self.universe {
                        Some(_) => format!(
                            "Evaluation of subquery \"{word}\" failed (did you want to use --keep_going?): {e}"
                        ),
                        None => e,
                    })?);
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
                let text = crate::ast::Expr::Call(call.clone()).to_string();
                let (inside, outside) = self.within_universe(&roots, true);
                let mut out = self.deps_in(&inside, Self::depth_arg(call, 1), Some(&text))?;
                out.extend(outside);
                Ok(out)
            }
            Function::RDeps => {
                let universe = self.expr_arg(call, 0, env)?;
                let targets = self.expr_arg(call, 1, env)?;
                let (universe, _) = self.within_universe(&universe, false);
                self.rdeps(&universe, &targets, Self::depth_arg(call, 2))
            }
            Function::AllRDeps => {
                let targets = self.expr_arg(call, 0, env)?;
                let (inside, outside) = self.within_universe(&targets, false);
                let universe = self.universe.clone().unwrap_or_default();
                let mut out = self.reverse_closure(&universe, &inside, Self::depth_arg(call, 1))?;
                out.extend(outside);
                Ok(out)
            }
            Function::RBuildFiles => {
                let paths: Vec<&str> = (0..call.args.len())
                    .map(|i| Self::word_arg(call, i))
                    .collect();
                self.rbuildfiles(&paths)
            }
            Function::SomePath => {
                let from = self.expr_arg(call, 0, env)?;
                let to = self.expr_arg(call, 1, env)?;
                let (from, to) = self.both_within_universe(&from, &to);
                self.some_path(&from, &to)
            }
            Function::AllPaths => {
                let from = self.expr_arg(call, 0, env)?;
                let to = self.expr_arg(call, 1, env)?;
                let (from, to) = self.both_within_universe(&from, &to);
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
        self.deps_in(roots, depth, None)
    }

    /// [`Evaluator::deps`] for the call written `text`, which a dependency
    /// that cannot be loaded is reported against: its error with the place of
    /// the rule that names it, twice, and that the closure failed, which
    /// `--keep_going` skips the dependency after.
    fn deps_in(
        &self,
        roots: &Set,
        depth: Option<usize>,
        text: Option<&str>,
    ) -> Result<Set, String> {
        let mut seen: Set = roots.clone();
        let mut frontier: Vec<Label> = roots.iter().cloned().collect();
        let mut level = 0;
        let mut failed = false;
        // What named each label first.
        let mut named_by: BTreeMap<Label, Label> = BTreeMap::new();
        while !frontier.is_empty() && depth.is_none_or(|d| level < d) {
            self.preload(&frontier);
            let mut next = Vec::new();
            for label in &frontier {
                let node = match self.node(label) {
                    Ok(node) => node,
                    Err(e) => {
                        let (Some(text), Some(by)) = (text, named_by.get(label)) else {
                            return Err(e);
                        };
                        let place = self.node(by)?.location.clone();
                        // A target that is not there says who needs it.
                        let said = match e.starts_with("no such target ") {
                            true => format!("{e} and referenced by '{}'", self.graph.display(by)),
                            false => e.clone(),
                        };
                        if !self.options.keep_going {
                            return Err(format!(
                                "{place}: {said}\nERROR: Evaluation of query \"{text}\" failed: preloading transitive closure failed: {e}"
                            ));
                        }
                        let mut skipped = self.skipped.lock().unwrap();
                        skipped.push(format!("ERROR: {place}: {said}"));
                        skipped.push(format!("ERROR: {place}: {said}"));
                        failed = true;
                        seen.remove(label);
                        continue;
                    }
                };
                for edge in self.edges(&node) {
                    if seen.insert(edge.to.clone()) {
                        named_by.insert(edge.to.clone(), label.clone());
                        next.push(edge.to);
                    }
                }
            }
            frontier = next;
            level += 1;
        }
        if let (true, Some(text)) = (failed, text) {
            self.skipped.lock().unwrap().push(format!(
                "ERROR: Evaluation of query \"{text}\" failed: errors were encountered while computing transitive closure"
            ));
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
        self.reverse_closure(&closure, targets, depth)
    }

    /// Both ends of a path, those in the universe, once the rest is warned
    /// of.
    fn both_within_universe(&self, from: &Set, to: &Set) -> (Set, Set) {
        let both: Set = from.union(to).cloned().collect();
        self.within_universe(&both, true);
        let universe = self.universe.as_ref();
        let keep = |set: &Set| -> Set {
            set.iter()
                .filter(|l| universe.is_none_or(|u| u.contains(*l)))
                .cloned()
                .collect()
        };
        (keep(from), keep(to))
    }

    /// The BUILD files of the universe that read one of `paths`, as the BUILD
    /// file itself or through the `.bzl` files it loads.
    fn rbuildfiles(&self, paths: &[&str]) -> Result<Set, String> {
        let universe = self.universe.clone().unwrap_or_default();
        let mut packages: BTreeMap<(String, String), Label> = BTreeMap::new();
        for label in &universe {
            packages
                .entry((label.repo.clone(), label.package.clone()))
                .or_insert_with(|| label.clone());
        }
        let path_of = |l: &Label| match l.package.is_empty() {
            true => l.name.clone(),
            false => format!("{}/{}", l.package, l.name),
        };
        let mut out = Set::new();
        for ((repo, package), label) in &packages {
            let files = self.graph.build_files(label)?;
            if !files.iter().any(|f| paths.contains(&path_of(f).as_str())) {
                continue;
            }
            out.extend(
                files
                    .into_iter()
                    .filter(|f| f.repo == *repo && f.package == *package)
                    .filter(|f| f.name.starts_with("BUILD")),
            );
        }
        Ok(out)
    }

    /// The targets of `closure` that depend on `targets`, within `depth`
    /// edges if given.
    fn reverse_closure(
        &self,
        closure: &Set,
        targets: &Set,
        depth: Option<usize>,
    ) -> Result<Set, String> {
        let mut reverse: BTreeMap<Label, Vec<Label>> = BTreeMap::new();
        for label in closure {
            let node = self.node(label)?;
            for edge in self.edges(&node) {
                reverse.entry(edge.to).or_default().push(label.clone());
            }
        }
        let mut seen: Set = targets.intersection(closure).cloned().collect();
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
            // What `query tests()` makes of a suite; a member that is no test
            // and a cycle are left out, not refused.
            let suites = QuerySuites { eval: self };
            if let Ok(found) = fjfj_graph::suite::expand(&suites, label, false) {
                for l in found {
                    out.insert(l);
                }
            }
        } else if *test {
            out.insert(label.clone());
        }
        Ok(())
    }
}

/// The tests of a graph, for [`fjfj_graph::suite`].
struct QuerySuites<'a, 'g> {
    eval: &'a Evaluator<'g>,
}

impl QuerySuites<'_, '_> {
    fn strings(node: &Node, name: &str) -> Vec<String> {
        match node.attrs.iter().find(|a| a.name == name).map(|a| &a.value) {
            Some(fjfj_graph::rule::AttrValue::StringList(items)) => items.clone(),
            _ => Vec::new(),
        }
    }

    fn test_of(node: &Node) -> fjfj_graph::suite::SuiteTest {
        let size = match node
            .attrs
            .iter()
            .find(|a| a.name == "size")
            .map(|a| &a.value)
        {
            Some(fjfj_graph::rule::AttrValue::String(size)) if !size.is_empty() => size.clone(),
            _ => "medium".to_owned(),
        };
        fjfj_graph::suite::SuiteTest {
            tags: Self::strings(node, "tags"),
            size,
        }
    }
}

impl fjfj_graph::suite::Suites for QuerySuites<'_, '_> {
    fn member(&self, label: &Label) -> Result<fjfj_graph::suite::Member, String> {
        use fjfj_graph::suite::Member;
        let Ok(node) = self.eval.node(label) else {
            return Ok(Member::Other);
        };
        let NodeKind::Rule { class, test, .. } = &node.kind else {
            return Ok(Member::Other);
        };
        Ok(if class == "test_suite" {
            Member::Suite {
                tests: node
                    .attrs
                    .iter()
                    .find(|a| a.name == "tests")
                    .map(|a| a.labels.clone())
                    .unwrap_or_default(),
                tags: Self::strings(&node, "tags"),
            }
        } else if *test {
            Member::Test(Self::test_of(&node))
        } else {
            Member::Other
        })
    }

    fn package_tests(
        &self,
        label: &Label,
    ) -> Result<Vec<(Label, fjfj_graph::suite::SuiteTest)>, String> {
        let mut out = Vec::new();
        for sibling in self.eval.graph.siblings(label)? {
            let node = self.eval.node(&sibling)?;
            if matches!(node.kind, NodeKind::Rule { test: true, .. }) {
                out.push((sibling, Self::test_of(&node)));
            }
        }
        Ok(out)
    }
}
