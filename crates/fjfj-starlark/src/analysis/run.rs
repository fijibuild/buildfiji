//! Running a rule's `implementation`.

use super::ctx::{CtxState, CtxValue};
use super::file::artifact_of;
use super::target::{DepInfo, StoredProvider, builtin};
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
use starlark::values::{Heap, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

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
    /// Outputs the BUILD file did not name and the rule declares, by the key
    /// `ctx.outputs` has them under, with the file's name in the package.
    pub outputs: Vec<(String, String)>,
    /// What `Label()` in the rule's code means in each repository.
    pub mappings: Arc<RepoMappings>,
    /// The toolchain types the rule asked for, each with the target that
    /// implements the toolchain resolved for it, if one was.
    pub toolchains: Vec<(Label, Option<DepInfo>)>,
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
/// The flag says the attribute's `cfg` builds the target for the execution
/// platform. A label named by both kinds of attribute is listed for each.
pub fn labels_of_attrs(schema: &RuleSchema, set: &[(String, AttrValue)]) -> Vec<(Label, bool)> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for (name, value) in resolved_attrs(schema, set) {
        if NOT_DEPENDENCIES.contains(&name.as_str()) {
            continue;
        }
        let is_dep = schema
            .attrs
            .iter()
            .find(|a| a.name == name)
            .is_some_and(|a| {
                matches!(
                    a.def.ty,
                    AttrType::Label
                        | AttrType::LabelList
                        | AttrType::LabelKeyedStringDict
                        | AttrType::StringKeyedLabelDict
                        | AttrType::LabelListDict
                )
            });
        if !is_dep {
            continue;
        }
        let exec = schema
            .attrs
            .iter()
            .find(|a| a.name == name)
            .is_some_and(|a| {
                matches!(
                    a.def.cfg,
                    fjfj_graph::rule::Cfg::Exec | fjfj_graph::rule::Cfg::Host
                )
            });
        let mut found = Vec::new();
        value.labels(&mut found);
        for label in found {
            if seen.insert((label.clone(), exec)) {
                out.push((label.clone(), exec));
            }
        }
    }
    out
}

/// Prints what a rule's code `print()`s.
struct Printed(Mutex<Vec<String>>);

impl starlark::PrintHandler for Printed {
    fn println(&self, text: &str) -> starlark::Result<()> {
        self.0.lock().unwrap().push(text.to_owned());
        Ok(())
    }
}

/// Run the rule's `implementation`.
pub fn run_rule(req: &RuleRequest) -> Result<RuleResult, String> {
    let (rule, _) = req
        .module
        .get_any_visibility(&req.rule_name)
        .map_err(|_| format!("no rule named {} in its .bzl", req.rule_name))?;
    let schema =
        schema_of(rule.value()).ok_or_else(|| format!("{} is not a rule", req.rule_name))?;
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
        schema: schema.clone(),
        attrs,
        deps: req
            .deps
            .iter()
            .map(|(l, d)| (l.clone(), Arc::new(d.clone())))
            .collect(),
        outputs: outputs.clone(),
        toolchains: req
            .toolchains
            .iter()
            .map(|(l, d)| (l.clone(), d.clone().map(Arc::new)))
            .collect(),
        actions: Mutex::new(Vec::new()),
        declared: Mutex::new(outputs.iter().map(|(_, a)| a.exec_path()).collect()),
    });

    let printed = Printed(Mutex::new(Vec::new()));
    let frozen = Module::with_temp_heap(|module| -> Result<_, String> {
        module.frozen_heap().add_reference(rule.owner());
        module.frozen_heap().add_reference(builtins_owner());
        for info in req.deps.values() {
            for provider in &info.providers {
                module.frozen_heap().add_reference(provider.value.owner());
            }
        }
        let rule_value = rule
            .value()
            .unpack_frozen()
            .expect("a global is frozen")
            .to_value();
        let implementation = implementation_of(rule_value)
            .ok_or_else(|| format!("{} has no implementation", req.rule_name))?;
        let running = BzlEval::running(&req.mappings);
        let heap = module.heap();
        let ctx = heap.alloc(CtxValue {
            state: state.clone(),
        });
        let returned = {
            let mut eval = Evaluator::new(&module);
            eval.extra = Some(&running);
            eval.set_print_handler(&printed);
            eval.eval_function(implementation, &[ctx], &[])
                .map_err(|e| format!("{e}"))?
        };
        let mut default_files: Option<Vec<Artifact>> = None;
        let mut executable = None;
        let mut runfiles = fjfj_graph::Runfiles::default();
        let mut others: Vec<Value<'_>> = Vec::new();
        let default_info = builtin("DefaultInfo");
        for instance in instances(returned, heap)? {
            let Some(provider) = provider_of(instance) else {
                return Err(format!(
                    "Rule '{}' returned a {}; it must return providers",
                    req.rule_name,
                    instance.get_type()
                ));
            };
            if same_provider(default_info, provider) {
                let (files, exe, rf) = read_default_info(instance, &req.rule_name)?;
                default_files = Some(files);
                executable = exe;
                runfiles = rf;
            } else {
                others.push(instance);
            }
        }
        module.set("providers", heap.alloc(others));
        let frozen = module.freeze().map_err(|e| format!("{e:?}"))?;
        Ok((frozen, default_files, executable, runfiles))
    })?;
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
    Ok(RuleResult {
        files,
        executable,
        runfiles,
        actions: state.actions.lock().unwrap().clone(),
        providers,
        outputs: outputs.into_iter().collect(),
        printed: std::mem::take(&mut *printed.0.lock().unwrap()),
    })
}

pub(crate) fn builtins_owner() -> &'static starlark::values::FrozenHeapRef {
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
    let executable = field("executable")
        .filter(|v| !v.is_none())
        .and_then(artifact_of);
    // `default_runfiles`, or the older `runfiles` that means the same.
    let mut runfiles = fjfj_graph::Runfiles::default();
    for name in ["runfiles", "default_runfiles"] {
        if let Some(found) = field(name)
            .filter(|v| !v.is_none())
            .and_then(super::runfiles::runfiles_of)
        {
            runfiles = runfiles.merge(&found);
        }
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
