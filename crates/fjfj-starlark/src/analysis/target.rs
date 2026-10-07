//! `Target`: a dependency as a rule's code sees it.

use super::file::alloc_file;
use crate::args::fatal;
use crate::depset::{Order, new_depset};
use crate::label::StarlarkLabel;
use crate::provider::same_provider;
use crate::structs::{new_instance, new_struct, provider_of};
use allocative::Allocative;
use fjfj_graph::{Artifact, Label};
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::starlark_module;
use starlark::starlark_simple_value;
use starlark::values::dict::DictRef;
use starlark::values::{
    Heap, NoSerialize, OwnedFrozenValue, ProvidesStaticType, StarlarkValue, Value, ValueLike,
};
use starlark_derive::starlark_value;
use std::fmt;
use std::sync::Arc;

/// A provider instance a target gave, kept after its evaluation ended.
#[derive(Clone, Debug)]
pub struct StoredProvider {
    pub value: OwnedFrozenValue,
}

/// Instances are the same when they are the same value; a recomputed target
/// is never equal to the one it replaces.
impl PartialEq for StoredProvider {
    fn eq(&self, other: &StoredProvider) -> bool {
        self.value.value().ptr_eq(other.value.value())
    }
}

impl Eq for StoredProvider {}

/// What a rule's code can see of a target it depends on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepInfo {
    pub label: Label,
    /// The rule class; `None` for a file.
    pub rule_class: Option<String>,
    /// A generated file's target, as opposed to a source file's.
    pub generated: bool,
    /// `DefaultInfo.files`.
    pub files: Vec<Artifact>,
    /// `DefaultInfo.executable`.
    pub executable: Option<Artifact>,
    /// `DefaultInfo.default_runfiles`.
    pub runfiles: fjfj_graph::Runfiles,
    /// The other providers it gave.
    pub providers: Vec<StoredProvider>,
}

#[derive(ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct TargetValue {
    #[allocative(skip)]
    pub(crate) info: Arc<DepInfo>,
    /// For `cquery --output=starlark`: the options of the target's
    /// configuration, which `build_options(target)` returns.
    #[allocative(skip)]
    pub(crate) build_options: Option<Arc<Vec<(String, fjfj_graph::SettingValue)>>>,
}

starlark_simple_value!(TargetValue);

impl fmt::Debug for TargetValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TargetValue({})", self.info.label)
    }
}

impl fmt::Display for TargetValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = crate::label::display_label(&self.info.label);
        match (&self.info.rule_class, self.info.generated) {
            (Some(_), _) => write!(f, "<target {label}>"),
            (None, true) => write!(f, "<output file target {label}>"),
            (None, false) => write!(f, "<input file target {label}>"),
        }
    }
}

fn target<'v>(this: Value<'v>) -> &'v TargetValue {
    this.downcast_ref::<TargetValue>().expect("a Target")
}

/// The builtin provider `name`, from the builtins module whose values every
/// evaluation of a rule keeps alive.
pub(crate) fn builtin<'v>(name: &str) -> Option<Value<'v>> {
    let (value, _) = crate::label::builtins().get_any_visibility(name).ok()?;
    value.value().unpack_frozen().map(|f| f.to_value())
}

/// The builtin provider at a dotted path: `platform_common.ToolchainInfo`.
pub(crate) fn builtin_by_path<'v>(path: &str) -> Option<Value<'v>> {
    let mut names = path.split('.');
    let mut value = builtin(names.next()?)?;
    for name in names {
        value = crate::structs::fields_of(value)?
            .into_iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| v)?;
    }
    Some(value)
}

/// An instance of `DefaultInfo` with these files.
pub(crate) fn default_info<'v>(
    heap: Heap<'v>,
    files: &[Artifact],
    executable: Option<&Artifact>,
    // Whether the executable has a runfiles tree, which a plain file has not.
    has_runfiles_tree: bool,
    runfiles: &fjfj_graph::Runfiles,
    owner: &Label,
) -> Value<'v> {
    let provider = builtin("DefaultInfo").expect("the builtins define DefaultInfo");
    let items: Vec<Value<'v>> = files
        .iter()
        .map(|a| alloc_file(heap, a.clone(), owner.clone()))
        .collect();
    let depset = new_depset(heap, &items, Order::Default, &[]).expect("files of one type");
    let exe = executable
        .map(|a| alloc_file(heap, a.clone(), owner.clone()))
        .unwrap_or_else(Value::new_none);
    // What `files_to_run` says of an executable: it and its runfiles tree.
    let files_to_run = {
        let sibling = |suffix: &str| {
            executable
                .filter(|_| has_runfiles_tree)
                .map_or_else(Value::new_none, |e| {
                    alloc_file(
                        heap,
                        Artifact {
                            root: e.root.clone(),
                            path: format!("{}{suffix}", e.path),
                            tree: false,
                        },
                        owner.clone(),
                    )
                })
        };
        new_struct(
            heap,
            vec![
                ("executable".to_owned(), exe),
                ("repo_mapping_manifest".to_owned(), sibling(".repo_mapping")),
                (
                    "runfiles_manifest".to_owned(),
                    sibling(".runfiles_manifest"),
                ),
            ],
        )
    };
    let runfiles = super::runfiles::alloc_runfiles(heap, runfiles.clone(), owner.clone());
    new_instance(
        heap,
        provider,
        vec![
            ("files".to_owned(), depset),
            ("runfiles".to_owned(), runfiles),
            ("executable".to_owned(), exe),
            ("data_runfiles".to_owned(), runfiles),
            ("default_runfiles".to_owned(), runfiles),
            ("files_to_run".to_owned(), files_to_run),
        ],
    )
}

#[starlark_value(type = "Target")]
impl<'v> StarlarkValue<'v> for TargetValue {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("Target", target_members);
        Some(RES.methods())
    }

    fn write_hash(
        &self,
        hasher: &mut starlark::collections::StarlarkHasher,
    ) -> starlark::Result<()> {
        use std::hash::Hash;
        self.info.label.hash(hasher);
        Ok(())
    }

    fn equals(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(other
            .downcast_ref::<TargetValue>()
            .is_some_and(|o| o.info.label == self.info.label))
    }

    fn at(&self, index: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        match self.find(index, heap) {
            Some(found) => Ok(found),
            None => Err(fatal(format!(
                "{} doesn't contain declared provider '{}'",
                self,
                crate::provider::instance_of(Some(index))
            ))),
        }
    }

    fn is_in(&self, other: Value<'v>) -> starlark::Result<bool> {
        // `Provider in target`: the default info is always there.
        let default = builtin("DefaultInfo");
        if same_provider(default, Some(other)) {
            return Ok(true);
        }
        Ok(self.info.providers.iter().any(|p| {
            provider_of(p.value.value().to_value()).is_some_and(|q| same_provider(q, Some(other)))
        }))
    }
}

/// The Make variables a target gives, which is what its `TemplateVariableInfo`
/// holds in `variables`.
pub(super) fn template_variables<'v>(info: &Arc<DepInfo>, heap: Heap<'v>) -> Vec<(String, String)> {
    let target = TargetValue {
        info: info.clone(),
        build_options: None,
    };
    let Some(instance) =
        builtin_by_path("platform_common.TemplateVariableInfo").and_then(|p| target.find(p, heap))
    else {
        return Vec::new();
    };
    let variables = crate::structs::fields_of(instance)
        .and_then(|fields| fields.into_iter().find(|(name, _)| *name == "variables"))
        .and_then(|(_, value)| DictRef::from_value(value));
    variables
        .map(|dict| {
            dict.iter()
                .filter_map(|(k, v)| Some((k.unpack_str()?.to_owned(), v.unpack_str()?.to_owned())))
                .collect()
        })
        .unwrap_or_default()
}

/// The Make variables the target that gave `providers` has, for a rule that
/// reads them without a Starlark heap of its own (the native `genrule`).
pub fn template_variables_of(providers: Vec<StoredProvider>) -> Vec<(String, String)> {
    let info = Arc::new(DepInfo {
        label: Label {
            repo: String::new(),
            package: String::new(),
            name: String::new(),
        },
        rule_class: None,
        generated: false,
        files: Vec::new(),
        executable: None,
        runfiles: fjfj_graph::Runfiles::default(),
        providers,
    });
    starlark::environment::Module::with_temp_heap(|module| template_variables(&info, module.heap()))
}

impl TargetValue {
    /// `providers(target)` of `cquery`: each provider the target gave, by
    /// name, `DefaultInfo` first.
    pub(super) fn provider_instances<'v>(&self, heap: Heap<'v>) -> Vec<(String, Value<'v>)> {
        let mut out = Vec::new();
        if let Some(default) = builtin("DefaultInfo").and_then(|p| self.find(p, heap)) {
            out.push(("DefaultInfo".to_owned(), default));
        }
        for stored in &self.info.providers {
            // SAFETY: as in `find`, the evaluation took a reference to the heap
            // that owns the provider.
            let instance = unsafe { stored.value.unchecked_frozen_value().to_value() };
            if let Some(provider) = provider_of(instance) {
                out.push((crate::provider::instance_of(provider), instance));
            }
        }
        out
    }

    pub(super) fn find<'v>(&self, provider: Value<'v>, heap: Heap<'v>) -> Option<Value<'v>> {
        if same_provider(builtin("DefaultInfo"), Some(provider)) {
            // A file, or an alias of one, is its own executable for `files_to_run`.
            let executable = self.info.executable.as_ref().or_else(|| {
                match (self.info.rule_class.as_deref(), &self.info.files[..]) {
                    (None | Some("alias"), [file]) => Some(file),
                    _ => None,
                }
            });
            return Some(default_info(
                heap,
                &self.info.files,
                executable,
                self.info.executable.is_some(),
                &self.info.runfiles,
                &self.info.label,
            ));
        }
        let mut matching = self
            .info
            .providers
            .iter()
            // SAFETY: the evaluation that made this `Target` took a reference to
            // the heap that owns each provider (`run_rule`), so the value lives
            // as long as `'v`.
            .map(|p| unsafe { p.value.unchecked_frozen_value().to_value() })
            .filter(|instance| {
                provider_of(*instance).is_some_and(|q| same_provider(q, Some(provider)))
            });
        let first = matching.next()?;
        // The output groups of the rule and of the aspects applied to it are
        // all the target's, in one `OutputGroupInfo`.
        if same_provider(builtin("OutputGroupInfo"), Some(provider)) {
            let rest: Vec<Value<'v>> = matching.collect();
            if !rest.is_empty() {
                let mut fields: Vec<(String, Value<'v>)> = Vec::new();
                for instance in std::iter::once(first).chain(rest) {
                    for (name, value) in crate::structs::fields_of(instance).unwrap_or_default() {
                        if !fields.iter().any(|(n, _)| n == name) {
                            fields.push((name.to_owned(), value));
                        }
                    }
                }
                return Some(new_instance(heap, provider, fields));
            }
        }
        Some(first)
    }
}

#[starlark_module]
fn target_members(builder: &mut MethodsBuilder) {
    #[starlark(attribute)]
    fn label<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        Ok(heap.alloc(StarlarkLabel::from(target(this).info.label.clone())))
    }

    #[starlark(attribute)]
    fn default_runfiles<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let info = &target(this).info;
        Ok(super::runfiles::alloc_runfiles(
            heap,
            info.runfiles.clone(),
            info.label.clone(),
        ))
    }

    #[starlark(attribute)]
    fn data_runfiles<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let info = &target(this).info;
        Ok(super::runfiles::alloc_runfiles(
            heap,
            info.runfiles.clone(),
            info.label.clone(),
        ))
    }

    /// `DefaultInfo.files_to_run`, which a `Target` also has.
    #[starlark(attribute)]
    fn files_to_run<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let default = target(this)
            .find(
                builtin("DefaultInfo").expect("DefaultInfo is a builtin"),
                heap,
            )
            .expect("every target has a DefaultInfo");
        default.get_attr_error("files_to_run", heap)
    }

    #[starlark(attribute)]
    fn files<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let info = &target(this).info;
        let items: Vec<Value<'v>> = info
            .files
            .iter()
            .map(|a| alloc_file(heap, a.clone(), info.label.clone()))
            .collect();
        new_depset(heap, &items, Order::Default, &[])
    }
}

/// The `Target` for `info`.
pub(crate) fn alloc_target<'v>(heap: Heap<'v>, info: Arc<DepInfo>) -> Value<'v> {
    heap.alloc(TargetValue {
        info,
        build_options: None,
    })
}

/// The `Target` for `info` as `cquery` shows it, with the options of its
/// configuration.
pub(super) fn alloc_configured_target<'v>(
    heap: Heap<'v>,
    info: Arc<DepInfo>,
    build_options: Vec<(String, fjfj_graph::SettingValue)>,
) -> Value<'v> {
    heap.alloc(TargetValue {
        info,
        build_options: Some(Arc::new(build_options)),
    })
}
