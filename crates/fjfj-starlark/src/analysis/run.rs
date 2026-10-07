//! Running a rule's `implementation`.

use super::ctx::{CtxState, CtxValue};
use super::file::artifact_of;
use super::target::{DepInfo, StoredProvider, alloc_target, builtin};
use crate::depset::{depset_to_list, is_depset};
use crate::label::{BzlEval, RepoMappings, builtins};
use crate::provider::same_provider;
use crate::rule::{implementation_of, schema_of};
use crate::structs::{fields_of, provider_of};
use fjfj_graph::rule::{AttrType, AttrValue};
use fjfj_graph::schema::RuleSchema;
use fjfj_graph::{Action, Artifact, Configuration, Label};
use starlark::environment::{FrozenModule, Module};
use starlark::eval::Evaluator;
use starlark::values::dict::DictRef;
use starlark::values::{Heap, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

/// Toolchain types, each with the target that implements the toolchain resolved for it.
/// The branches of a split, each by its key with the targets in it.
pub type SplitBranches = Vec<(String, Vec<(Label, DepInfo)>)>;

pub type ResolvedToolchains = Vec<(Label, Option<DepInfo>)>;

/// What the engine hands a rule to analyse it.
pub struct RuleRequest {
    /// The `.bzl` whose `rule()` this is, evaluated.
    pub module: FrozenModule,
    /// The name the rule is bound to in it.
    pub rule_name: String,
    pub label: Label,
    /// `a/BUILD.bazel:3:8`.
    pub location: String,
    /// `a/BUILD.bazel`, for `ctx.build_file_path`.
    pub build_file: String,
    pub configuration: Configuration,
    /// `_main`.
    pub main_repo_name: String,
    /// What the BUILD file set, `select()`s already decided.
    pub attrs: Vec<(String, AttrValue)>,
    /// Every target an attribute names, set or defaulted.
    pub deps: BTreeMap<Label, DepInfo>,
    /// For an attribute with a split transition, each branch of the split by
    /// its key (in the order of the configurations) with the targets the
    /// attribute names in it.
    pub splits: BTreeMap<String, SplitBranches>,
    /// Outputs the BUILD file did not name and the rule declares, by the key
    /// `ctx.outputs` has them under, with the file's name in the package.
    pub outputs: Vec<(String, String)>,
    /// What `Label()` in the rule's code means in each repository.
    pub mappings: Arc<RepoMappings>,
    /// The toolchain types the rule asked for, each with the target that
    /// implements the toolchain resolved for it, if one was.
    pub toolchains: Vec<(Label, Option<DepInfo>)>,
    /// The same for each exec group the rule declared, by name.
    pub exec_groups: Vec<(String, ResolvedToolchains)>,
    /// For a build setting: its value in this configuration.
    pub build_setting_value: Option<fjfj_graph::SettingValue>,
    /// The schema of a native rule whose analysis is the function
    /// `_native_implementations[rule_name]` of the builtins, in which case
    /// `module` is the builtins and there is no `rule()` to look at.
    pub native: Option<Arc<RuleSchema>>,
}

/// What an `implementation` gave.
#[derive(Debug, Default)]
pub struct RuleResult {
    /// `DefaultInfo.files`.
    pub files: Vec<Artifact>,
    pub executable: Option<Artifact>,
    /// `DefaultInfo.default_runfiles` (or `runfiles`).
    pub runfiles: fjfj_graph::Runfiles,
    pub actions: Vec<Action>,
    /// Every provider but `DefaultInfo`.
    pub providers: Vec<StoredProvider>,
    /// The outputs the rule declared, by key, as files.
    pub outputs: BTreeMap<String, Artifact>,
    /// What `print()` wrote.
    pub printed: Vec<String>,
}

/// The schema of the rule `rule_name` of `module`.
pub fn rule_schema(module: &FrozenModule, rule_name: &str) -> Option<Arc<RuleSchema>> {
    let (rule, _) = module.get_any_visibility(rule_name).ok()?;
    schema_of(rule.value())
}

/// Each attribute's value, set or defaulted. An attribute with no value at all
/// (a label with no default) is left out.
pub fn resolved_attrs(
    schema: &RuleSchema,
    set: &[(String, AttrValue)],
) -> Vec<(String, AttrValue)> {
    let mut out = Vec::new();
    for attr in &schema.attrs {
        let value = set
            .iter()
            .find(|(n, _)| *n == attr.name)
            .map(|(_, v)| v.clone())
            .or_else(|| attr.def.default_value());
        if let Some(value) = value {
            out.push((attr.name.clone(), value));
        }
    }
    out
}

/// Attributes every rule has that name labels which are not targets the rule
/// reads: who may see it, where it may be built, what it is licensed under.
const NOT_DEPENDENCIES: [&str; 10] = [
    "visibility",
    "compatible_with",
    "restricted_to",
    "target_compatible_with",
    "exec_compatible_with",
    "exec_group_compatible_with",
    "package_metadata",
    "applicable_licenses",
    "aspect_hints",
    "transitive_configs",
];

/// Every label the rule's attributes name, set or defaulted: the targets its
/// code can see.
///
/// Each comes with the attribute that names it and that attribute's `cfg`.
/// A label named by attributes of different kinds is listed for each.
pub fn labels_of_attrs(schema: &RuleSchema, set: &[(String, AttrValue)]) -> Vec<DepEdge> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for (name, value) in resolved_attrs(schema, set) {
        if NOT_DEPENDENCIES.contains(&name.as_str()) {
            continue;
        }
        let Some(attr) = schema.attrs.iter().find(|a| a.name == name) else {
            continue;
        };
        let is_dep = matches!(
            attr.def.ty,
            AttrType::Label
                | AttrType::LabelList
                | AttrType::LabelKeyedStringDict
                | AttrType::StringKeyedLabelDict
                | AttrType::LabelListDict
        );
        if !is_dep {
            continue;
        }
        let mut found = Vec::new();
        value.labels(&mut found);
        for label in found {
            if seen.insert((label.clone(), attr.def.cfg as u8, name.clone())) {
                out.push(DepEdge {
                    label: label.clone(),
                    attr: name.clone(),
                    cfg: attr.def.cfg,
                });
            }
        }
    }
    out
}

/// A dependency a rule's attribute names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepEdge {
    pub label: Label,
    pub attr: String,
    pub cfg: fjfj_graph::rule::Cfg,
}

/// Prints what a rule's code `print()`s.
struct Printed(Mutex<Vec<String>>);

impl starlark::PrintHandler for Printed {
    fn println(&self, text: &str) -> starlark::Result<()> {
        self.0.lock().unwrap().push(text.to_owned());
        Ok(())
    }

    fn println_at(
        &self,
        location: Option<&starlark::codemap::FileSpan>,
        text: &str,
    ) -> starlark::Result<()> {
        self.println(&crate::print_line(location, text))
    }
}

/// Starts the error of a rule whose attributes were wrong, which already says
/// which rule it is and so is not wrapped as an `in <rule> rule` failure.
pub const ATTRIBUTE_ERRORS: &str = "\u{1}";

/// The events in an analysis error: one for each line of one that starts with
/// [`ATTRIBUTE_ERRORS`], otherwise the whole message.
pub fn error_events(message: &str) -> Vec<String> {
    if let Some(t) = super::transition::split_transition_error(message) {
        let at = if t.location.is_empty() {
            String::new()
        } else {
            format!("{}: ", t.location)
        };
        return vec![
            format!("{at}{}", t.text),
            format!(
                "{}Errors encountered while applying Starlark transition",
                t.edge
            ),
        ];
    }
    match message.strip_prefix(ATTRIBUTE_ERRORS) {
        Some(events) => events.lines().map(str::to_owned).collect(),
        None => vec![message.to_owned()],
    }
}

/// Starts each line a rule printed before its analysis failed, at the head of
/// its error message; [`PRINTED_END`] ends them and the message follows.
pub const PRINTED_LINE: char = '\u{2}';
/// Ends what a failed rule printed at the head of its error.
pub const PRINTED_END: char = '\u{3}';

/// Puts what a rule printed ahead of its error, for [`split_printed`].
pub fn with_printed(printed: &[String], message: String) -> String {
    if printed.is_empty() {
        return message;
    }
    let mut out = String::new();
    for line in printed {
        out.push(PRINTED_LINE);
        out.push_str(line);
    }
    out.push(PRINTED_END);
    out + &message
}

/// What a failed rule printed, and its error without that.
pub fn split_printed(message: &str) -> (Vec<String>, &str) {
    let Some(rest) = message.strip_prefix(PRINTED_LINE) else {
        return (Vec::new(), message);
    };
    match rest.split_once(PRINTED_END) {
        Some((lines, message)) => (
            lines.split(PRINTED_LINE).map(str::to_owned).collect(),
            message,
        ),
        None => (Vec::new(), message),
    }
}

/// Run the rule's `implementation`.
pub fn run_rule(req: &RuleRequest) -> Result<RuleResult, String> {
    // A native rule is its schema and a function in a table of the builtins.
    let (rule, schema) = match &req.native {
        Some(schema) => {
            let (table, _) = req
                .module
                .get_any_visibility("_native_implementations")
                .map_err(|_| "the builtins have no _native_implementations".to_owned())?;
            (table, schema.clone())
        }
        None => {
            let (rule, _) = req
                .module
                .get_any_visibility(&req.rule_name)
                .map_err(|_| format!("no rule named {} in its .bzl", req.rule_name))?;
            let schema = schema_of(rule.value())
                .ok_or_else(|| format!("{} is not a rule", req.rule_name))?;
            (rule, schema)
        }
    };
    let attrs = resolved_attrs(&schema, &req.attrs);
    // The predeclared outputs are files of the package, and declared.
    let bin_dir = req.configuration.bin_dir();
    let outputs: Vec<(String, Artifact)> = req
        .outputs
        .iter()
        .map(|(key, name)| {
            (
                key.clone(),
                Artifact::derived(&bin_dir, &req.label.repo, &req.label.package, name),
            )
        })
        .collect();
    let state = Arc::new(CtxState {
        label: req.label.clone(),
        rule_kind: req.rule_name.clone(),
        location: req.location.clone(),
        build_file: req.build_file.clone(),
        configuration: req.configuration.clone(),
        main_repo_name: req.main_repo_name.clone(),
        mappings: req.mappings.clone(),
        schema: schema.clone(),
        build_setting_value: req.build_setting_value.clone(),
        attrs,
        deps: req
            .deps
            .iter()
            .map(|(l, d)| (l.clone(), Arc::new(d.clone())))
            .collect(),
        splits: req
            .splits
            .iter()
            .map(|(attr, branches)| {
                let branches = branches
                    .iter()
                    .map(|(key, deps)| {
                        let deps = deps
                            .iter()
                            .map(|(l, d)| (l.clone(), Arc::new(d.clone())))
                            .collect();
                        (key.clone(), deps)
                    })
                    .collect();
                (attr.clone(), branches)
            })
            .collect(),
        outputs: outputs.clone(),
        toolchains: req
            .toolchains
            .iter()
            .map(|(l, d)| (l.clone(), d.clone().map(Arc::new)))
            .collect(),
        exec_groups: req
            .exec_groups
            .iter()
            .map(|(name, types)| {
                (
                    name.clone(),
                    types
                        .iter()
                        .map(|(l, d)| (l.clone(), d.clone().map(Arc::new)))
                        .collect(),
                )
            })
            .collect(),
        rule: None,
        aspect_ids: Vec::new(),
        actions: Mutex::new(Vec::new()),
        nested: Mutex::default(),
        declared: Mutex::new(outputs.iter().map(|(_, a)| a.exec_path()).collect()),
        errors: Mutex::default(),
    });
    execute(
        state,
        &rule,
        &|rule_value| {
            if req.native.is_some() {
                DictRef::from_value(rule_value).and_then(|table| table.get_str(&req.rule_name))
            } else {
                implementation_of(rule_value)
            }
        },
        req.deps.values(),
        &req.rule_name,
        None,
    )
}

/// Run the function that `implementation` finds in `owner`'s value with the
/// `ctx` that `state` makes, and read the providers it returns.
pub(super) fn execute<'a>(
    state: Arc<CtxState>,
    owner: &starlark::values::OwnedFrozenValue,
    implementation: &dyn for<'v> Fn(Value<'v>) -> Option<Value<'v>>,
    deps: impl Iterator<Item = &'a DepInfo>,
    rule_name: &str,
    target: Option<&DepInfo>,
) -> Result<RuleResult, String> {
    let outputs = state.outputs.clone();
    let printed = Printed(Mutex::new(Vec::new()));
    let frozen = Module::with_temp_heap(|module| -> Result<_, String> {
        module.frozen_heap().add_reference(owner.owner());
        module.frozen_heap().add_reference(builtins_owner());
        for info in deps {
            for provider in &info.providers {
                module.frozen_heap().add_reference(provider.value.owner());
            }
        }
        let rule_value = owner
            .value()
            .unpack_frozen()
            .expect("a global is frozen")
            .to_value();
        let implementation = implementation(rule_value)
            .ok_or_else(|| format!("{rule_name} has no implementation"))?;
        let mut running = BzlEval::running(&state.mappings);
        running.rule_ctx = Some(state.clone());
        let heap = module.heap();
        let ctx = heap.alloc(CtxValue {
            state: state.clone(),
        });
        let returned = {
            let mut eval = Evaluator::new(&module);
            eval.extra = Some(&running);
            eval.set_print_handler(&printed);
            // An aspect's implementation is given the target first.
            let mut args = Vec::with_capacity(2);
            if let Some(info) = target {
                args.push(alloc_target(heap, Arc::new(info.clone())));
            }
            args.push(ctx);
            // Bazel puts the traceback on a line of its own.
            eval.eval_function(implementation, &args, &[])
                .map_err(|e| format!("\n{}", crate::traceback(&e)))?
        };
        // Errors the rule went on after fail it now.
        let errors = state.errors.lock().unwrap();
        if !errors.is_empty() {
            return Err(format!("{ATTRIBUTE_ERRORS}{}", errors.join("\n")));
        }
        drop(errors);
        // Every file the rule declared has an action that makes it, used or not.
        let made: BTreeSet<String> = state
            .actions
            .lock()
            .unwrap()
            .iter()
            .flat_map(|a| a.outputs.iter().map(|o| o.exec_path()))
            .collect();
        let bin = format!("{}/", state.configuration.bin_dir());
        let missing: Vec<String> = state
            .declared
            .lock()
            .unwrap()
            .iter()
            .filter(|path| !made.contains(*path))
            .map(|path| path.strip_prefix(&bin).unwrap_or(path).to_owned())
            .collect();
        if !missing.is_empty() {
            let at = starlark::eval::definition_span(implementation)
                .map(|span| {
                    let at = span.resolve();
                    format!(
                        "{}:{}:{}: ",
                        at.file,
                        at.span.begin.line + 1,
                        at.span.begin.column + 1
                    )
                })
                .unwrap_or_default();
            let mut missing = missing;
            missing.sort();
            return Err(format!(
                "\n{at}The following files have no generating action:\n{}",
                missing.join("\n")
            ));
        }
        let mut default_files: Option<Vec<Artifact>> = None;
        let mut executable = None;
        let mut runfiles = fjfj_graph::Runfiles::default();
        let mut others: Vec<Value<'_>> = Vec::new();
        let default_info = builtin("DefaultInfo");
        for instance in instances(returned, heap)? {
            let Some(provider) = provider_of(instance) else {
                return Err(format!(
                    "Rule '{rule_name}' returned a {}; it must return providers",
                    instance.get_type()
                ));
            };
            if same_provider(default_info, provider) {
                let (files, exe, rf) = read_default_info(instance, rule_name)?;
                default_files = Some(files);
                executable = exe;
                runfiles = rf;
            } else {
                others.push(instance);
            }
        }
        // An analysis test gives its result and no executable: Bazel makes the
        // script that says it.
        if executable.is_none()
            && let Some(result_info) = builtin("AnalysisTestResultInfo")
            && let Some(result) = others
                .iter()
                .copied()
                .find(|instance| same_provider(Some(result_info), provider_of(*instance).flatten()))
            && let Some(make) = builtin("_analysis_test_script")
        {
            let mut eval = Evaluator::new(&module);
            eval.extra = Some(&running);
            eval.set_print_handler(&printed);
            let script = eval
                .eval_function(make, &[ctx, result], &[])
                .map_err(|e| format!("\n{}", crate::traceback(&e)))?;
            executable = artifact_of(script);
        }
        module.set("providers", heap.alloc(others));
        let frozen = module.freeze().map_err(|e| format!("{e:?}"))?;
        Ok((frozen, default_files, executable, runfiles))
    })
    .map_err(|message| with_printed(&printed.0.lock().unwrap(), message))?;
    let (frozen, default_files, executable, runfiles) = frozen;
    let providers = match frozen.get_any_visibility("providers") {
        Ok((list, _)) => frozen_items(&list),
        Err(_) => Vec::new(),
    };
    let mut files =
        default_files.unwrap_or_else(|| outputs.iter().map(|(_, a)| a.clone()).collect());
    // The executable is built with the target whether or not it is listed.
    if let Some(exe) = &executable
        && !files.contains(exe)
    {
        files.push(exe.clone());
    }
    let actions = distinct_actions(state.actions.lock().unwrap().clone());
    Ok(RuleResult {
        files,
        executable,
        runfiles,
        actions,
        providers,
        outputs: outputs.into_iter().collect(),
        printed: std::mem::take(&mut *printed.0.lock().unwrap()),
    })
}

/// The actions without the repeats: Bazel lets a rule register an equal action
/// again. Two that differ and make the same file are kept, for the check of the
/// whole build to find (`fjfj_graph::conflicts`).
fn distinct_actions(actions: Vec<Action>) -> Vec<Action> {
    let mut out: Vec<Action> = Vec::with_capacity(actions.len());
    for action in actions {
        if !out.contains(&action) {
            out.push(action);
        }
    }
    out
}

pub(super) fn builtins_owner() -> &'static starlark::values::FrozenHeapRef {
    let (_, _) = builtins()
        .get_any_visibility("DefaultInfo")
        .expect("builtin");
    static OWNER: std::sync::OnceLock<starlark::values::FrozenHeapRef> = std::sync::OnceLock::new();
    OWNER.get_or_init(|| {
        builtins()
            .get_any_visibility("DefaultInfo")
            .expect("builtin")
            .0
            .owner()
            .clone()
    })
}

/// The provider instances an `implementation` returned: a list, a tuple, a
/// single instance, or `None`.
fn instances<'v>(returned: Value<'v>, heap: Heap<'v>) -> Result<Vec<Value<'v>>, String> {
    if returned.is_none() {
        return Ok(Vec::new());
    }
    if let Some(items) = crate::args::sequence(returned) {
        return Ok(items);
    }
    let _ = heap;
    Ok(vec![returned])
}

fn read_default_info(
    instance: Value<'_>,
    rule: &str,
) -> Result<(Vec<Artifact>, Option<Artifact>, fjfj_graph::Runfiles), String> {
    let fields = fields_of(instance).unwrap_or_default();
    let field = |name: &str| fields.iter().find(|(n, _)| *n == name).map(|(_, v)| *v);
    let mut files = Vec::new();
    if let Some(value) = field("files").filter(|v| !v.is_none()) {
        let items = if is_depset(value) {
            depset_to_list(value)
                .expect("a depset")
                .map_err(|e| format!("{e}"))?
        } else {
            crate::args::sequence(value).unwrap_or_default()
        };
        for item in items {
            files.push(artifact_of(item).ok_or_else(|| {
                format!(
                    "Rule '{rule}' returned a DefaultInfo whose files hold a {}",
                    item.get_type()
                )
            })?);
        }
    }
    let executable = field("$executable")
        .filter(|v| !v.is_none())
        .and_then(artifact_of);
    let mut runfiles = fjfj_graph::Runfiles::default();
    for name in ["default_runfiles"] {
        if let Some(found) = field(name)
            .filter(|v| !v.is_none())
            .and_then(super::runfiles::runfiles_of)
        {
            runfiles = runfiles.merge(&found);
        }
    }
    // The executable is among its own runfiles, as Bazel has it (rules_rust
    // reads a build script's out of a stand-in's `default_runfiles`).
    if let Some(exe) = &executable
        && !runfiles.files.contains(exe)
    {
        runfiles.files.push(exe.clone());
    }
    Ok((files, executable, runfiles))
}

/// Each item of a frozen list as a value of its own.
fn frozen_items(list: &starlark::values::OwnedFrozenValue) -> Vec<StoredProvider> {
    use starlark::values::list::ListRef;
    let Some(items) = ListRef::from_value(list.value()) else {
        return Vec::new();
    };
    items
        .iter()
        .filter_map(|item| item.unpack_frozen())
        .map(|item| StoredProvider {
            value: list.map(|_| item),
        })
        .collect()
}
