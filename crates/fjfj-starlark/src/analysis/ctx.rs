//! `ctx`, the first argument of a rule's `implementation`.

use super::actions::ActionsValue;
use super::file::alloc_file;
use super::target::{DepInfo, alloc_target};
use crate::args::{Wording, bind, fatal, param};
use crate::label::StarlarkLabel;
use crate::structs::new_struct;
use allocative::Allocative;
use fjfj_graph::expand::{Expander, Prerequisite};
use fjfj_graph::rule::{AttrType, AttrValue};
use fjfj_graph::schema::RuleSchema;
use fjfj_graph::{Action, Artifact, Configuration, Label, LabelContext};
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_module;
use starlark::starlark_simple_value;
use starlark::values::dict::AllocDict;
use starlark::values::list::AllocList;
use starlark::values::{Heap, NoSerialize, ProvidesStaticType, StarlarkValue, Value, ValueLike};
use starlark_derive::starlark_value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::{Arc, Mutex};

/// Everything one run of an `implementation` knows and builds.
pub(crate) struct CtxState {
    pub(crate) label: Label,
    pub(crate) rule_kind: String,
    /// `a/BUILD.bazel:3:8`: where the target is declared.
    pub(crate) location: String,
    pub(crate) build_file: String,
    pub(crate) configuration: Configuration,
    pub(crate) main_repo_name: String,
    pub(crate) schema: Arc<RuleSchema>,
    /// `ctx.build_setting_value`, for a build setting.
    pub(crate) build_setting_value: Option<fjfj_graph::SettingValue>,
    /// Every attribute, set or defaulted.
    pub(crate) attrs: Vec<(String, AttrValue)>,
    pub(crate) deps: BTreeMap<Label, Arc<DepInfo>>,
    /// Predeclared outputs by the name `ctx.outputs` gives them.
    pub(crate) outputs: Vec<(String, Artifact)>,
    /// `ctx.toolchains`: each type and the implementation resolved for it.
    pub(crate) toolchains: Vec<(Label, Option<Arc<DepInfo>>)>,
    pub(crate) actions: Mutex<Vec<Action>>,
    /// Exec paths declared so far, which another declaration may not repeat.
    pub(crate) declared: Mutex<BTreeSet<String>>,
}

impl CtxState {
    pub(crate) fn bin_dir(&self) -> String {
        self.configuration.bin_dir()
    }

    /// A new output file `name` of this target's package.
    pub(crate) fn derived(&self, name: &str) -> Artifact {
        Artifact::derived(&self.bin_dir(), &self.label.repo, &self.label.package, name)
    }

    /// The target of a label an attribute names.
    fn dep<'v>(&self, heap: Heap<'v>, label: &Label) -> Value<'v> {
        match self.deps.get(label) {
            Some(info) => alloc_target(heap, info.clone()),
            None => Value::new_none(),
        }
    }

    /// An attribute's value as the code sees it.
    fn attr_value<'v>(&self, heap: Heap<'v>, value: &AttrValue) -> Value<'v> {
        match value {
            AttrValue::Bool(b) => Value::new_bool(*b),
            AttrValue::Int(i) => heap.alloc(*i),
            AttrValue::String(s) => heap.alloc(s.as_str()),
            AttrValue::StringList(items) => heap.alloc(AllocList(items.iter().map(String::as_str))),
            AttrValue::IntList(items) => heap.alloc(AllocList(items.iter().copied())),
            AttrValue::Label(l) => self.dep(heap, l),
            AttrValue::LabelList(items) => {
                heap.alloc(AllocList(items.iter().map(|l| self.dep(heap, l))))
            }
            AttrValue::StringDict(items) => heap.alloc(AllocDict(
                items
                    .iter()
                    .map(|(k, v)| (heap.alloc(k.as_str()), heap.alloc(v.as_str()))),
            )),
            AttrValue::StringListDict(items) => {
                heap.alloc(AllocDict(items.iter().map(|(k, v)| {
                    (
                        heap.alloc(k.as_str()),
                        heap.alloc(AllocList(v.iter().map(String::as_str))),
                    )
                })))
            }
            AttrValue::LabelKeyedStringDict(items) => heap.alloc(AllocDict(
                items
                    .iter()
                    .map(|(k, v)| (self.dep(heap, k), heap.alloc(v.as_str()))),
            )),
            AttrValue::StringKeyedLabelDict(items) => heap.alloc(AllocDict(
                items
                    .iter()
                    .map(|(k, v)| (heap.alloc(k.as_str()), self.dep(heap, v))),
            )),
            AttrValue::LabelListDict(items) => heap.alloc(AllocDict(items.iter().map(|(k, v)| {
                (
                    heap.alloc(k.as_str()),
                    heap.alloc(AllocList(v.iter().map(|l| self.dep(heap, l)))),
                )
            }))),
            // Resolved before the rule runs.
            AttrValue::Select(_) => Value::new_none(),
        }
    }

    /// The files an attribute's targets give.
    fn files_of(&self, value: &AttrValue) -> Vec<Artifact> {
        let labels: Vec<&Label> = match value {
            AttrValue::Label(l) => vec![l],
            AttrValue::LabelList(items) => items.iter().collect(),
            _ => return Vec::new(),
        };
        labels
            .into_iter()
            .filter_map(|l| self.deps.get(l))
            .flat_map(|info| info.files.iter().cloned())
            .collect()
    }

    fn file<'v>(&self, heap: Heap<'v>, artifact: Artifact) -> Value<'v> {
        let owner = self.label.clone();
        alloc_file(heap, artifact, owner)
    }
}

#[derive(ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct CtxValue {
    #[allocative(skip)]
    pub(crate) state: Arc<CtxState>,
}

starlark_simple_value!(CtxValue);

impl fmt::Debug for CtxValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ctx")
    }
}

impl fmt::Display for CtxValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<ctx for {}>", self.state.label)
    }
}

fn state<'v>(this: Value<'v>) -> &'v Arc<CtxState> {
    &this.downcast_ref::<CtxValue>().expect("a ctx").state
}

#[starlark_value(type = "ctx")]
impl<'v> StarlarkValue<'v> for CtxValue {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("ctx", ctx_members);
        Some(RES.methods())
    }
}

#[starlark_module]
fn ctx_members(builder: &mut MethodsBuilder) {
    #[starlark(attribute)]
    fn label<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        Ok(heap.alloc(StarlarkLabel::from(state(this).label.clone())))
    }

    #[starlark(attribute)]
    fn build_setting_value<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        use fjfj_graph::SettingValue;
        match &state(this).build_setting_value {
            Some(SettingValue::Bool(b)) => Ok(Value::new_bool(*b)),
            Some(SettingValue::Int(i)) => Ok(heap.alloc(*i)),
            Some(SettingValue::Str(s)) => Ok(heap.alloc(s.as_str())),
            Some(SettingValue::List(items)) => {
                Ok(heap.alloc(AllocList(items.iter().map(String::as_str))))
            }
            None => Err(fatal(format!(
                "attempting to access 'build_setting_value' of non-build setting {}",
                fjfj_graph::expand::label_text(&state(this).label)
            ))),
        }
    }

    /// The configuration fragments.
    #[starlark(attribute)]
    fn fragments<'v>(this: Value<'v>) -> starlark::Result<Value<'v>> {
        let config = &state(this).configuration;
        super::fragments::fragments_of(&config.cpu, config.compilation_mode.name())
            .map(|made| made.to_value())
            .map_err(fatal)
    }

    #[starlark(attribute)]
    fn workspace_name<'v>(this: Value<'v>) -> starlark::Result<String> {
        Ok(state(this).main_repo_name.clone())
    }

    #[starlark(attribute)]
    fn build_file_path<'v>(this: Value<'v>) -> starlark::Result<String> {
        Ok(state(this).build_file.clone())
    }

    #[starlark(attribute)]
    fn bin_dir<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        Ok(heap.alloc(super::file::RootValue {
            path: state(this).bin_dir(),
            source: false,
        }))
    }

    #[starlark(attribute)]
    fn genfiles_dir<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        Ok(heap.alloc(super::file::RootValue {
            path: state(this).bin_dir(),
            source: false,
        }))
    }

    #[starlark(attribute)]
    fn var<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let s = state(this);
        let mut vars = vec![
            ("TARGET_CPU", target_cpu(&s.configuration.cpu).to_owned()),
            (
                "COMPILATION_MODE",
                s.configuration.compilation_mode.name().to_owned(),
            ),
            ("BINDIR", s.bin_dir()),
            ("GENDIR", s.bin_dir()),
        ];
        for (k, v) in &s.configuration.defines {
            vars.push((k.as_str(), v.clone()));
        }
        Ok(heap.alloc(AllocDict(
            vars.into_iter()
                .map(|(k, v)| (heap.alloc(k), heap.alloc(v.as_str()))),
        )))
    }

    #[starlark(attribute)]
    fn toolchains<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        Ok(heap.alloc(ToolchainsValue {
            state: state(this).clone(),
        }))
    }

    #[starlark(attribute)]
    fn actions<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        Ok(heap.alloc(ActionsValue {
            state: state(this).clone(),
        }))
    }

    #[starlark(attribute)]
    fn attr<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let s = state(this);
        let mut fields: Vec<(String, Value<'v>)> = s
            .attrs
            .iter()
            .filter(|(name, _)| !s.schema_hidden(name))
            .map(|(name, value)| (name.clone(), s.attr_value(heap, value)))
            .collect();
        // A label attribute that has no value is `None`.
        for attr in &s.schema.attrs {
            if matches!(attr.def.ty, fjfj_graph::rule::AttrType::Label)
                && !fields.iter().any(|(n, _)| *n == attr.name)
                && !s.schema_hidden(&attr.name)
            {
                fields.push((attr.name.clone(), Value::new_none()));
            }
        }
        Ok(new_struct(heap, fields))
    }

    #[starlark(attribute)]
    fn files<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let s = state(this);
        let mut fields = Vec::new();
        for (name, value) in &s.attrs {
            if s.takes_files(name) {
                let files = s.files_of(value);
                fields.push((
                    name.clone(),
                    heap.alloc(AllocList(files.into_iter().map(|a| s.file(heap, a)))),
                ));
            }
        }
        Ok(new_struct(heap, fields))
    }

    #[starlark(attribute)]
    fn file<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let s = state(this);
        let mut fields = Vec::new();
        for (name, value) in &s.attrs {
            if s.takes_single_file(name) {
                let files = s.files_of(value);
                let one = match files.as_slice() {
                    [one] => s.file(heap, one.clone()),
                    [] => Value::new_none(),
                    _ => {
                        return Err(fatal(format!(
                            "attribute '{name}' of {} produced more than one file",
                            s.label
                        )));
                    }
                };
                fields.push((name.clone(), one));
            }
        }
        Ok(new_struct(heap, fields))
    }

    #[starlark(attribute)]
    fn executable<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let s = state(this);
        let mut fields = Vec::new();
        for (name, value) in &s.attrs {
            if s.is_executable(name) {
                let exe = match value {
                    AttrValue::Label(l) => s
                        .deps
                        .get(l)
                        .and_then(|d| d.executable.clone().or_else(|| d.files.first().cloned()))
                        .map(|a| s.file(heap, a))
                        .unwrap_or_else(Value::new_none),
                    _ => Value::new_none(),
                };
                fields.push((name.clone(), exe));
            }
        }
        Ok(new_struct(heap, fields))
    }

    #[starlark(attribute)]
    fn outputs<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let s = state(this);
        let fields = s
            .outputs
            .iter()
            .map(|(name, artifact)| (name.clone(), s.file(heap, artifact.clone())))
            .collect();
        Ok(new_struct(heap, fields))
    }

    /// `ctx.runfiles(files, transitive_files, collect_data, collect_default,
    /// symlinks, root_symlinks)`.
    fn runfiles<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let s = state(this);
        let bound = bind(
            "runfiles",
            Wording::Signature,
            &[
                param("files", true, false),
                param("transitive_files", true, false),
                param("collect_data", true, false),
                param("collect_default", true, false),
                param("symlinks", true, false),
                param("root_symlinks", true, false),
                param("skip_conflict_checking", false, false),
            ],
            args,
            eval,
        )?;
        let mut runfiles = fjfj_graph::Runfiles::default();
        if let Some(v) = bound[0].filter(|v| !v.is_none()) {
            runfiles.files = super::runfiles::files_in(v, "files")?;
        }
        if let Some(v) = bound[1].filter(|v| !v.is_none()) {
            for f in super::runfiles::files_in(v, "transitive_files")? {
                if !runfiles.files.contains(&f) {
                    runfiles.files.push(f);
                }
            }
        }
        let flag = |i: usize| bound[i].and_then(|v| v.unpack_bool()).unwrap_or(false);
        if flag(2) || flag(3) {
            // The files and runfiles of what the rule reads in srcs, deps and data.
            for (name, value) in &s.attrs {
                if !matches!(name.as_str(), "srcs" | "deps" | "data") {
                    continue;
                }
                let mut labels = Vec::new();
                value.labels(&mut labels);
                for dep in labels.into_iter().filter_map(|l| s.deps.get(l)) {
                    let mut contributed = fjfj_graph::Runfiles {
                        files: dep.files.clone(),
                        ..fjfj_graph::Runfiles::default()
                    };
                    contributed = contributed.merge(&dep.runfiles);
                    runfiles = runfiles.merge(&contributed);
                }
            }
        }
        let entries = |i: usize| -> starlark::Result<Vec<(String, Artifact)>> {
            let Some(dict) = bound[i].and_then(starlark::values::dict::DictRef::from_value) else {
                return Ok(Vec::new());
            };
            dict.iter()
                .map(|(k, v)| {
                    Ok((
                        k.unpack_str()
                            .ok_or_else(|| fatal("runfiles symlink paths must be strings"))?
                            .to_owned(),
                        super::file::artifact_of(v)
                            .ok_or_else(|| fatal("runfiles symlink targets must be Files"))?,
                    ))
                })
                .collect()
        };
        runfiles.symlinks = entries(4)?;
        runfiles.root_symlinks = entries(5)?;
        Ok(super::runfiles::alloc_runfiles(
            eval.heap(),
            runfiles,
            s.label.clone(),
        ))
    }

    /// `ctx.target_platform_has_constraint(constraint_value)`.
    fn target_platform_has_constraint<'v>(
        this: Value<'v>,
        constraint_value: Value<'v>,
    ) -> starlark::Result<bool> {
        let s = state(this);
        let label = crate::structs::fields_of(constraint_value)
            .and_then(|fields| {
                fields
                    .into_iter()
                    .find(|(n, _)| *n == "label")
                    .and_then(|(_, v)| crate::label::label_of_value(v))
            })
            .ok_or_else(|| {
                fatal(format!(
                    "in call to target_platform_has_constraint(), parameter 'constraint_value' got value of type '{}', want 'ConstraintValueInfo'",
                    constraint_value.get_type()
                ))
            })?;
        Ok(s.configuration.constraints.contains(&label))
    }

    /// `ctx.expand_location(input, targets = [])`.
    fn expand_location<'v>(
        this: Value<'v>,
        input: &str,
        targets: Option<Value<'v>>,
        heap: Heap<'v>,
    ) -> starlark::Result<String> {
        let s = state(this);
        let mut prerequisites = Vec::new();
        for t in targets
            .and_then(|t| crate::attr::sequence(t, heap))
            .unwrap_or_default()
        {
            if let Some(target) = t.downcast_ref::<super::target::TargetValue>() {
                prerequisites.push(Prerequisite {
                    label: target.info.label.clone(),
                    files: target.info.files.clone(),
                });
            }
        }
        let outs: Vec<Artifact> = s.outputs.iter().map(|(_, a)| a.clone()).collect();
        let defines = s.configuration.defines.clone();
        let expander = Expander {
            rule_class: &s.rule_kind,
            attribute: "args",
            label: &s.label,
            srcs: &[],
            outs: &outs,
            prerequisites: &prerequisites,
            bin_dir: &s.bin_dir(),
            target_cpu: target_cpu(&s.configuration.cpu),
            compilation_mode: s.configuration.compilation_mode.name(),
            defines: &defines,
            main_repo_name: &s.main_repo_name,
            context: LabelContext {
                repo: &s.label.repo,
                package: &s.label.package,
            },
        };
        expander
            .expand_locations(input)
            .map_err(|e| fatal(e.message))
    }
}

impl CtxState {
    fn attr_def(&self, name: &str) -> Option<&fjfj_graph::rule::AttrDef> {
        self.schema
            .attrs
            .iter()
            .find(|a| a.name == name)
            .map(|a| &a.def)
    }

    /// `ctx.attr` does not show private attributes of the rule.
    fn schema_hidden(&self, name: &str) -> bool {
        name == "visibility" && false
    }

    fn takes_files(&self, name: &str) -> bool {
        self.attr_def(name).is_some_and(|d| {
            matches!(d.ty, AttrType::Label | AttrType::LabelList)
                && !matches!(d.files, fjfj_graph::rule::FileTypes::None)
        })
    }

    fn takes_single_file(&self, name: &str) -> bool {
        self.attr_def(name).is_some_and(|d| {
            d.single_file() && matches!(d.ty, AttrType::Label | AttrType::LabelList)
        })
    }

    fn is_executable(&self, name: &str) -> bool {
        self.attr_def(name)
            .is_some_and(|d| d.executable() && matches!(d.ty, AttrType::Label))
    }
}

/// What `$(TARGET_CPU)` is for a `--cpu`.
pub(crate) fn target_cpu(cpu: &str) -> &str {
    match cpu {
        "k8" => "x86_64",
        other => other,
    }
}

/// `ctx.toolchains`: indexed by a toolchain type, gives the `ToolchainInfo`
/// of the toolchain resolved for it.
#[derive(ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct ToolchainsValue {
    #[allocative(skip)]
    state: Arc<CtxState>,
}

starlark_simple_value!(ToolchainsValue);

impl fmt::Debug for ToolchainsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("toolchains")
    }
}

impl fmt::Display for ToolchainsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<toolchain context>")
    }
}

#[starlark_value(type = "ToolchainContext")]
impl<'v> StarlarkValue<'v> for ToolchainsValue {
    fn at(&self, index: Value<'v>, _heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let wanted = match crate::label::label_of_value(index) {
            Some(label) => label,
            None => {
                let text = index.unpack_str().ok_or_else(|| {
                    fatal(format!(
                        "in index, got a {} for the toolchain type",
                        index.get_type()
                    ))
                })?;
                Label::parse(
                    text,
                    LabelContext {
                        repo: &self.state.label.repo,
                        package: &self.state.label.package,
                    },
                )
                .map_err(|e| fatal(format!("invalid toolchain type '{text}': {e}")))?
            }
        };
        let Some((_, resolved)) = self.state.toolchains.iter().find(|(l, _)| *l == wanted) else {
            return Err(fatal(format!(
                "In {} rule {}, toolchain type {} was requested but only types [{}] are configured",
                self.state.rule_kind,
                crate::label::display_label(&self.state.label),
                crate::label::display_label(&wanted),
                self.state
                    .toolchains
                    .iter()
                    .map(|(l, _)| crate::label::display_label(l))
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        };
        let Some(info) = resolved else {
            return Ok(Value::new_none());
        };
        let tool = super::target::builtin_by_path("platform_common.ToolchainInfo");
        info.providers
            .iter()
            // SAFETY: the evaluation took a reference to the heap that owns
            // each provider of a dependency (`run_rule`).
            .map(|p| unsafe { p.value.unchecked_frozen_value().to_value() })
            .find(|instance| {
                crate::structs::provider_of(*instance)
                    .is_some_and(|q| crate::provider::same_provider(q, tool))
            })
            .ok_or_else(|| {
                fatal(format!(
                    "toolchain {} does not give a platform_common.ToolchainInfo",
                    crate::label::display_label(&info.label)
                ))
            })
    }

    fn is_in(&self, other: Value<'v>) -> starlark::Result<bool> {
        let label = crate::label::label_of_value(other);
        Ok(self
            .state
            .toolchains
            .iter()
            .any(|(l, r)| Some(l) == label.as_ref() && r.is_some()))
    }
}
