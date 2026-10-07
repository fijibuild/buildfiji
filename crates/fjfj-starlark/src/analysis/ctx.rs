//! `ctx`, the first argument of a rule's `implementation`.

use super::actions::ActionsValue;
use super::file::alloc_file;
use super::target::{DepInfo, alloc_target, template_variables};
use crate::args::{Wording, bind, fatal, param};
use crate::label::StarlarkLabel;
use crate::structs::new_struct;
use allocative::Allocative;
use fjfj_graph::config::COMMAND_LINE_OPTION;
use fjfj_graph::expand::{Expander, Prerequisite};
use fjfj_graph::rule::{AttrType, AttrValue};
use fjfj_graph::schema::RuleSchema;
use fjfj_graph::{Action, Artifact, Configuration, Label, LabelContext, SettingValue};
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_module;
use starlark::starlark_simple_value;
use starlark::values::dict::{AllocDict, DictRef};
use starlark::values::list::AllocList;
use starlark::values::{Heap, NoSerialize, ProvidesStaticType, StarlarkValue, Value, ValueLike};
use starlark_derive::starlark_value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::{Arc, Mutex};

/// A toolchain type and the toolchain resolved for it.
pub(crate) type ResolvedType = (Label, Option<Arc<DepInfo>>);

/// One branch of a split: its key and the targets in it.
pub(crate) type SplitBranch = (String, Vec<(Label, Arc<DepInfo>)>);

/// Everything one run of an `implementation` knows and builds.
pub(crate) struct CtxState {
    pub(crate) label: Label,
    pub(crate) rule_kind: String,
    /// `a/BUILD.bazel:3:8`: where the target is declared.
    pub(crate) location: String,
    pub(crate) build_file: String,
    pub(crate) configuration: Configuration,
    pub(crate) main_repo_name: String,
    /// What `@name` means in each repository, for labels written as text.
    pub(crate) mappings: Arc<crate::label::RepoMappings>,
    pub(crate) schema: Arc<RuleSchema>,
    /// `ctx.build_setting_value`, for a build setting.
    pub(crate) build_setting_value: Option<fjfj_graph::SettingValue>,
    /// Every attribute, set or defaulted.
    pub(crate) attrs: Vec<(String, AttrValue)>,
    pub(crate) deps: BTreeMap<Label, Arc<DepInfo>>,
    /// An attribute with a split transition: for each branch of the split, by
    /// its key, the targets the attribute names in the configuration it made.
    pub(crate) splits: BTreeMap<String, Vec<SplitBranch>>,
    /// Predeclared outputs by the name `ctx.outputs` gives them.
    pub(crate) outputs: Vec<(String, Artifact)>,
    /// `ctx.toolchains`: each type and the implementation resolved for it.
    pub(crate) toolchains: Vec<(Label, Option<Arc<DepInfo>>)>,
    /// `ctx.exec_groups`: each declared group and its own toolchains.
    pub(crate) exec_groups: Vec<(String, Vec<ResolvedType>)>,
    /// For an aspect: the rule it is looking at, as a `ctx` of the rule's own
    /// attributes (`ctx.rule`).
    pub(crate) rule: Option<Arc<CtxState>>,
    /// For an aspect: the aspects applied to this target so far, then this one.
    pub(crate) aspect_ids: Vec<String>,
    pub(crate) actions: Mutex<Vec<Action>>,
    /// The nested sets made of the depsets actions took, by depset, so a
    /// depset several actions take is one set.
    pub(crate) nested: Mutex<std::collections::HashMap<u64, Arc<fjfj_graph::NestedSet<Artifact>>>>,
    /// Exec paths declared so far, which another declaration may not repeat.
    pub(crate) declared: Mutex<BTreeSet<String>>,
    /// Errors in an attribute the rule went on after, as Bazel's
    /// `attributeError` does: each is `in <attribute> attribute of <rule>
    /// rule <label>: <message>`, once however often it is raised, and the
    /// rule fails when its implementation returns.
    pub(crate) errors: Mutex<Vec<String>>,
}

/// The Make variables of a target (probed on Bazel 9.2.0): the
/// configuration's own, the `TemplateVariableInfo` of each target of the
/// `toolchains` attribute, and the `--define`s.
pub(crate) struct MakeVariables {
    builtin: Vec<(String, String)>,
    toolchains: Vec<Vec<(String, String)>>,
    defines: Vec<(String, String)>,
}

impl MakeVariables {
    /// `ctx.var`: one dict, where a toolchain overrides the configuration,
    /// a later toolchain an earlier one, and a `--define` anything.
    fn dict(&self) -> Vec<(String, String)> {
        let mut vars: Vec<(String, String)> = Vec::new();
        for (k, v) in self
            .builtin
            .iter()
            .chain(self.toolchains.iter().flatten())
            .chain(&self.defines)
        {
            match vars.iter_mut().find(|(name, _)| name == k) {
                Some(entry) => entry.1 = v.clone(),
                None => vars.push((k.clone(), v.clone())),
            }
        }
        vars
    }

    /// `$(NAME)` in `expand_make_variables`: a `--define` first, then the
    /// toolchains in the order the attribute lists them, then the
    /// configuration's.
    fn lookup(&self, name: &str) -> Option<String> {
        let find = |list: &Vec<(String, String)>| {
            list.iter()
                .rev()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.clone())
        };
        find(&self.defines)
            .or_else(|| self.toolchains.iter().find_map(find))
            .or_else(|| find(&self.builtin))
    }
}

impl CtxState {
    /// The Make variables of this target, in the layers Bazel keeps them.
    pub(crate) fn make_variables<'v>(&self, heap: Heap<'v>) -> MakeVariables {
        let builtin = vec![
            (
                "TARGET_CPU".to_owned(),
                target_cpu(&self.configuration.cpu).to_owned(),
            ),
            (
                "COMPILATION_MODE".to_owned(),
                self.configuration.compilation_mode.name().to_owned(),
            ),
            ("BINDIR".to_owned(), self.bin_dir()),
            ("GENDIR".to_owned(), self.bin_dir()),
        ];
        let toolchains = self.attrs.iter().find_map(|(name, value)| match value {
            AttrValue::LabelList(labels) if name == "toolchains" => Some(labels),
            _ => None,
        });
        MakeVariables {
            builtin,
            toolchains: toolchains
                .into_iter()
                .flatten()
                .filter_map(|label| self.deps.get(label))
                .map(|info| template_variables(info, heap))
                .collect(),
            defines: self
                .configuration
                .defines
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }

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
    pub(crate) fn attr_value<'v>(&self, heap: Heap<'v>, value: &AttrValue) -> Value<'v> {
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

    /// `attr_value`, except that an attribute with a transition is the list of
    /// the targets of every branch, one if the transition is no split.
    fn attr_or_split<'v>(&self, heap: Heap<'v>, name: &str, value: &AttrValue) -> Value<'v> {
        match self.splits.get(name) {
            Some(branches) => heap.alloc(AllocList(
                branches
                    .iter()
                    .flat_map(|(_, deps)| deps)
                    .map(|(_, d)| alloc_target(heap, d.clone())),
            )),
            None => self.attr_value(heap, value),
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

    /// What the builtins make `ctx.fragments` and `ctx.configuration` of.
    fn configuration_options(&self) -> Vec<(&'static str, String)> {
        let config = &self.configuration;
        // The OS of the target platform, which `cc_common` asks for the exec one.
        let has = |os: &str| {
            config
                .constraints
                .iter()
                .any(|c| c.repo == "platforms" && c.package == "os" && c.name == os)
        };
        let os = if has("osx") {
            "macos"
        } else if has("windows") {
            "windows"
        } else {
            "linux"
        };
        let option = |name: &str| config.options.get(name).cloned().unwrap_or_default();
        let switch = |name: &str| {
            let on = config.settings.get(&format!("{COMMAND_LINE_OPTION}{name}"))
                == Some(&SettingValue::Bool(true));
            if on { "1" } else { "" }.to_owned()
        };
        vec![
            ("copt", option("copt")),
            ("cxxopt", option("cxxopt")),
            ("conlyopt", option("conlyopt")),
            ("linkopt", option("linkopt")),
            ("strip", option("strip")),
            ("cpu", config.cpu.clone()),
            (
                "compilation_mode",
                config.compilation_mode.name().to_owned(),
            ),
            ("os", os.to_owned()),
            (
                "default_shell_env",
                config
                    .default_shell_env()
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            // Only a test rule sees `--test_env`,
            // and not the names it takes from the client.
            (
                "test_env",
                if self.schema.test {
                    config
                        .test_env
                        .iter()
                        .filter(|(k, _)| !config.test_env_inherited.contains(*k))
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    String::new()
                },
            ),
            // The switches `--stamp`, `--collect_code_coverage`, `--force_pic`
            // and `--save_temps`, and whether `--fission` is on in this mode.
            ("stamp", switch("stamp")),
            ("collect_code_coverage", switch("collect_code_coverage")),
            ("force_pic", switch("force_pic")),
            ("save_temps", switch("save_temps")),
            (
                "fission",
                match config
                    .settings
                    .get(&format!("{COMMAND_LINE_OPTION}fission"))
                {
                    Some(SettingValue::List(modes))
                        if modes.iter().any(|m| m == config.compilation_mode.name()) =>
                    {
                        "1".to_owned()
                    }
                    _ => String::new(),
                },
            ),
            ("short_id", config.checksum()[..7].to_owned()),
            ("bin_dir", config.bin_dir()),
            ("exec", if config.exec { "1" } else { "" }.to_owned()),
        ]
    }

    fn file<'v>(&self, heap: Heap<'v>, artifact: Artifact) -> Value<'v> {
        // A source file is owned by its own target, where the rule has that
        // target among those it reads (probed on 9.2.0, buildfiji-136.32).
        let owner = if artifact.is_source() {
            self.deps
                .values()
                .find(|d| d.rule_class.is_none() && !d.generated && d.files.contains(&artifact))
                .map_or_else(|| self.label.clone(), |d| d.label.clone())
        } else {
            self.label.clone()
        };
        alloc_file(heap, artifact, owner)
    }
}

/// The `ctx` of the rule being analysed, for code that runs in it (a subrule).
pub(crate) fn alloc_ctx<'v>(heap: Heap<'v>, state: Arc<CtxState>) -> Value<'v> {
    heap.alloc(CtxValue { state })
}

/// The value of the attribute `name` of the rule being analysed, None if it has none.
pub(crate) fn attr_named<'v>(state: &CtxState, heap: Heap<'v>, name: &str) -> Value<'v> {
    state
        .attrs
        .iter()
        .find(|(n, _)| n == name)
        .map_or_else(Value::new_none, |(_, v)| state.attr_value(heap, v))
}

/// The label lists every rule has, which `ctx.attr` and `ctx.files` show.
const IMPLICIT_LABEL_LISTS: [&str; 2] = ["_action_listener", "_config_dependencies"];

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
        write!(f, "<rule context for {}>", self.state.label)
    }
}

/// `(enabled, name)` of each entry of the rule's `features` attribute.
fn features_of(state: &CtxState) -> Vec<(bool, String)> {
    match state.attrs.iter().find(|(n, _)| n == "features") {
        Some((_, AttrValue::StringList(items))) => items
            .iter()
            .map(|f| match f.strip_prefix('-') {
                Some(off) => (false, off.to_owned()),
                None => (true, f.clone()),
            })
            .collect(),
        _ => Vec::new(),
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
            Some(SettingValue::Str(s) | SettingValue::Label(s)) => Ok(heap.alloc(s.as_str())),
            Some(SettingValue::None) => Ok(Value::new_none()),
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
        let options = state(this).configuration_options();
        let options: Vec<(&str, &str)> = options.iter().map(|(k, v)| (*k, v.as_str())).collect();
        super::fragments::made_by("_make_fragments", &options)
            .map(|made| made.to_value())
            .map_err(fatal)
    }

    /// `ctx.exec_groups`: the groups `rule(exec_groups = ...)` declared.
    #[starlark(attribute)]
    fn exec_groups<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        Ok(heap.alloc(ExecGroupCollection {
            state: state(this).clone(),
        }))
    }

    /// The features the rule asks for: its `features` attribute (the package's
    /// and `--features` are not merged in yet).
    #[starlark(attribute)]
    fn features<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        Ok(heap.alloc(AllocList(
            features_of(state(this))
                .into_iter()
                .filter_map(|(on, f)| on.then_some(f)),
        )))
    }

    /// The features it turned off (`-name`).
    #[starlark(attribute)]
    fn disabled_features<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        Ok(heap.alloc(AllocList(
            features_of(state(this))
                .into_iter()
                .filter_map(|(on, f)| (!on).then_some(f)),
        )))
    }

    /// `ctx.coverage_instrumented(target = None)`: coverage is not collected
    /// (buildfiji-fyz.9).
    fn coverage_instrumented<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
    ) -> starlark::Result<bool> {
        let _ = (this, args);
        Ok(false)
    }

    /// The configuration: `ctx.configuration`.
    #[starlark(attribute)]
    fn configuration<'v>(this: Value<'v>) -> starlark::Result<Value<'v>> {
        let options = state(this).configuration_options();
        let options: Vec<(&str, &str)> = options.iter().map(|(k, v)| (*k, v.as_str())).collect();
        super::fragments::made_by("_make_configuration", &options)
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

    /// `bazel-out/stable-status.txt`, which the build writes before any action
    /// reads it: the keys of the workspace status that are stable.
    #[starlark(attribute)]
    fn info_file<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let _ = this;
        Ok(status_file(heap, "stable-status.txt"))
    }

    /// `bazel-out/volatile-status.txt`: the keys that change from build to
    /// build, which never make an action that reads them run again.
    #[starlark(attribute)]
    fn version_file<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let _ = this;
        Ok(status_file(heap, "volatile-status.txt"))
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
        let vars = state(this).make_variables(heap).dict();
        Ok(heap
            .alloc(AllocDict(vars.into_iter().map(|(k, v)| {
                (heap.alloc(k.as_str()), heap.alloc(v.as_str()))
            }))))
    }

    #[starlark(attribute)]
    fn toolchains<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        Ok(heap.alloc(ToolchainsValue {
            state: state(this).clone(),
            group: None,
        }))
    }

    #[starlark(attribute)]
    fn actions<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        Ok(heap.alloc(ActionsValue {
            state: state(this).clone(),
        }))
    }

    /// `ctx.rule`, in an aspect: the attributes of the target it looks at.
    #[starlark(attribute)]
    fn rule<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        match &state(this).rule {
            Some(rule) => Ok(alloc_ctx(heap, rule.clone())),
            None => Err(fatal("'rule' is only available in aspect implementations")),
        }
    }

    /// `ctx.kind` (`ctx.rule.kind` in an aspect): the rule class.
    #[starlark(attribute)]
    fn kind<'v>(this: Value<'v>) -> starlark::Result<String> {
        Ok(state(this).rule_kind.clone())
    }

    #[starlark(attribute)]
    fn aspect_ids<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let s = state(this);
        if s.rule.is_none() {
            return Err(fatal(
                "'aspect_ids' is only available in aspect implementations",
            ));
        }
        Ok(heap.alloc(AllocList(s.aspect_ids.iter().map(String::as_str))))
    }

    #[starlark(attribute)]
    fn attr<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let s = state(this);
        let mut fields: Vec<(String, Value<'v>)> = s
            .attrs
            .iter()
            .filter(|(name, _)| !s.schema_hidden(name))
            .map(|(name, value)| (name.clone(), s.attr_or_split(heap, name, value)))
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
        // `name` is the rule's.
        match fields.iter_mut().find(|(n, _)| n == "name") {
            Some(field) => field.1 = heap.alloc(s.label.name.as_str()),
            None => fields.push(("name".to_owned(), heap.alloc(s.label.name.as_str()))),
        }
        // What every rule has that nothing sets: Bazel's defaults for the
        // package's.
        let mut absent = |name: &str, value: Value<'v>| {
            if !fields.iter().any(|(n, _)| n == name) {
                fields.push((name.to_owned(), value));
            }
        };
        absent("testonly", Value::new_bool(false));
        absent("deprecation", Value::new_none());
        absent(
            "package_metadata",
            heap.alloc(AllocList(Vec::<Value<'v>>::new())),
        );
        for name in IMPLICIT_LABEL_LISTS {
            absent(name, heap.alloc(AllocList(Vec::<Value<'v>>::new())));
        }
        Ok(new_struct(heap, fields))
    }

    /// An attribute with a transition, by the key of each branch (`None` for
    /// one that is no split): a target (a label) or a list of them (a list of
    /// labels).
    #[starlark(attribute)]
    fn split_attr<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let s = state(this);
        let mut fields = Vec::new();
        for (name, branches) in &s.splits {
            let one = matches!(
                s.attrs.iter().find(|(n, _)| n == name),
                Some((_, AttrValue::Label(_)))
            );
            let entries: Vec<(Value<'v>, Value<'v>)> = branches
                .iter()
                .map(|(key, deps)| {
                    let mut targets = deps.iter().map(|(_, d)| alloc_target(heap, d.clone()));
                    let value = if one {
                        targets.next_back().unwrap_or_else(Value::new_none)
                    } else {
                        heap.alloc(AllocList(targets))
                    };
                    // A transition that is no split has no key.
                    let key = if key.is_empty() {
                        Value::new_none()
                    } else {
                        heap.alloc(key.as_str())
                    };
                    (key, value)
                })
                .collect();
            fields.push((name.clone(), heap.alloc(AllocDict(entries))));
        }
        Ok(new_struct(heap, fields))
    }

    #[starlark(attribute)]
    fn files<'v>(this: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let s = state(this);
        let mut fields = Vec::new();
        // The private label lists every rule has, which nothing sets.
        for name in IMPLICIT_LABEL_LISTS {
            fields.push((
                name.to_owned(),
                heap.alloc(AllocList(Vec::<Value<'v>>::new())),
            ));
        }
        for name in s.schema.attrs.iter().map(|a| &a.name) {
            if s.takes_files(name) && !s.hidden_from_files(name) {
                // An attribute with no value is an empty list.
                let files = s
                    .attrs
                    .iter()
                    .find(|(n, _)| n == name)
                    .map_or_else(Vec::new, |(_, value)| s.files_of(value));
                let files = match s.splits.get(name) {
                    Some(branches) => branches
                        .iter()
                        .flat_map(|(_, deps)| deps)
                        .flat_map(|(_, d)| d.files.iter().cloned())
                        .collect(),
                    None => files,
                };
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
        for name in s.schema.attrs.iter().map(|a| &a.name) {
            if s.takes_single_file(name) {
                // An attribute with no value is None.
                let files = s
                    .attrs
                    .iter()
                    .find(|(n, _)| n == name)
                    .map_or_else(Vec::new, |(_, value)| s.files_of(value));
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
        for name in s.schema.attrs.iter().map(|a| &a.name) {
            if s.is_executable(name) {
                let value = s.attrs.iter().find(|(n, _)| n == name).map(|(_, v)| v);
                let exe = match value {
                    Some(AttrValue::Label(l)) => s
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
        // An `attr.output_list()` is a list of its files, empty if the target
        // set none; an `attr.output()` it did not set is None.
        let lists: Vec<&str> = s
            .schema
            .attrs
            .iter()
            .filter(|a| a.def.ty == AttrType::OutputList)
            .map(|a| a.name.as_str())
            .collect();
        let mut fields: Vec<(String, Value<'v>)> = Vec::new();
        for (name, artifact) in &s.outputs {
            if !lists.contains(&name.as_str()) {
                fields.push((name.clone(), s.file(heap, artifact.clone())));
            }
        }
        for attr in &s.schema.attrs {
            match attr.def.ty {
                AttrType::OutputList => {
                    let files: Vec<Value<'v>> = s
                        .outputs
                        .iter()
                        .filter(|(n, _)| *n == attr.name)
                        .map(|(_, a)| s.file(heap, a.clone()))
                        .collect();
                    fields.push((attr.name.clone(), heap.alloc(AllocList(files))));
                }
                AttrType::Output if !fields.iter().any(|(n, _)| *n == attr.name) => {
                    fields.push((attr.name.clone(), Value::new_none()));
                }
                _ => {}
            }
        }
        Ok(new_struct(heap, fields))
    }

    /// `ctx.package_relative_label(input)`: `input` as written in the BUILD
    /// file of the rule's package.
    fn package_relative_label<'v>(
        this: Value<'v>,
        input: Value<'v>,
        heap: Heap<'v>,
    ) -> starlark::Result<Value<'v>> {
        let s = state(this);
        crate::label::relative_to_package_as(
            input,
            LabelContext {
                repo: &s.label.repo,
                package: &s.label.package,
            },
            &s.mappings,
            heap,
            "ctx.package_relative_label",
        )
    }

    /// `ctx.tokenize(option)`: the words of a shell command line.
    fn tokenize<'v>(this: Value<'v>, option: Value<'v>) -> starlark::Result<Vec<String>> {
        let _ = this;
        let text = option.unpack_str().ok_or_else(|| {
            fatal(format!(
                "in call to tokenize(), parameter 'option' got value of type '{}', want 'string'",
                option.get_type()
            ))
        })?;
        fjfj_graph::command_line::tokenize(text)
            .map_err(|e| fatal(format!("{e} while tokenizing '{text}'")))
    }

    /// `ctx.check_placeholders(template, allowed_placeholders)`: whether the
    /// template uses no `%{name}` that is not allowed.
    fn check_placeholders<'v>(
        this: Value<'v>,
        template: Value<'v>,
        allowed_placeholders: Value<'v>,
    ) -> starlark::Result<bool> {
        let _ = this;
        let text = template.unpack_str().ok_or_else(|| {
            fatal(format!(
                "in call to check_placeholders(), parameter 'template' got value of type '{}', want 'string'",
                template.get_type()
            ))
        })?;
        let allowed: Vec<String> = crate::args::sequence(allowed_placeholders)
            .ok_or_else(|| {
                fatal(format!(
                    "in call to check_placeholders(), parameter 'allowed_placeholders' got value of type '{}', want 'sequence'",
                    allowed_placeholders.get_type()
                ))
            })?
            .iter()
            .filter_map(|v| v.unpack_str().map(str::to_owned))
            .collect();
        Ok(fjfj_graph::command_line::placeholders_are_allowed(
            text, &allowed,
        ))
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
                // Builtins only: python's legacy `__init__.py` files.
                param("_python_inits", false, false),
            ],
            args,
            eval,
        )?;
        let mut runfiles = fjfj_graph::Runfiles {
            python_inits: bound[7].and_then(|v| v.unpack_bool()).unwrap_or(false),
            ..fjfj_graph::Runfiles::default()
        };
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
        let flag_value = |i: usize, name: &str| -> starlark::Result<bool> {
            match bound[i] {
                None => Ok(false),
                Some(v) => v.unpack_bool().ok_or_else(|| {
                    fatal(format!(
                        "in call to runfiles(), parameter '{name}' got value of type '{}', want 'bool'",
                        v.get_type()
                    ))
                }),
            }
        };
        let (collect_data, collect_default) = (
            flag_value(2, "collect_data")?,
            flag_value(3, "collect_default")?,
        );
        if collect_data || collect_default {
            // The files and runfiles of what the rule reads in srcs, deps and data.
            for (name, value) in &s.attrs {
                if !matches!(name.as_str(), "srcs" | "deps" | "data") {
                    continue;
                }
                let mut labels = Vec::new();
                value.labels(&mut labels);
                for dep in labels.into_iter().filter_map(|l| s.deps.get(l)) {
                    // What a rule makes is runfiles when the rule reads it in
                    // `data`; from `srcs` and `deps` only its runfiles are.
                    let mut contributed = fjfj_graph::Runfiles {
                        files: if name == "data" {
                            dep.files.clone()
                        } else {
                            Vec::new()
                        },
                        ..fjfj_graph::Runfiles::default()
                    };
                    contributed = contributed.merge(&dep.runfiles);
                    runfiles = runfiles.merge(&contributed);
                }
            }
        }
        let entries = |i: usize| -> starlark::Result<Vec<(String, Artifact)>> {
            let name = if i == 4 { "symlinks" } else { "root_symlinks" };
            let Some(value) = bound[i].filter(|v| !v.is_none()) else {
                return Ok(Vec::new());
            };
            let Some(dict) = starlark::values::dict::DictRef::from_value(value) else {
                if crate::depset::is_depset(value) {
                    return Ok(Vec::new());
                }
                return Err(fatal(format!(
                    "in call to runfiles(), parameter '{name}' got value of type '{}', want 'dict or depset'",
                    value.get_type()
                )));
            };
            dict.iter()
                .map(
                    |(k, v)| match (k.unpack_str(), super::file::artifact_of(v)) {
                        (Some(path), Some(target)) => Ok((path.to_owned(), target)),
                        _ => Err(fatal(format!(
                            "got dict<{}, {}> for '{name}', want dict<string, File>",
                            k.get_type(),
                            v.get_type()
                        ))),
                    },
                )
                .collect()
        };
        // After what `collect_data` and `collect_default` gathered, not in place of it.
        runfiles.symlinks.extend(entries(4)?);
        runfiles.root_symlinks.extend(entries(5)?);
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
                    "in call to target_platform_has_constraint(), parameter 'constraintValue' got value of type '{}', want 'ConstraintValueInfo'",
                    constraint_value.get_type()
                ))
            })?;
        Ok(s.configuration.constraints.contains(&label))
    }

    /// `ctx.expand_make_variables(attribute_name, command,
    /// additional_substitutions)`: `$(VAR)` read from the substitutions, then
    /// the variables of the `toolchains` attribute, then the configuration's.
    /// A reference that cannot be expanded is an error of the attribute the
    /// rule goes on after: the command comes back as it was and the rule
    /// fails when its implementation returns.
    fn expand_make_variables<'v>(
        this: Value<'v>,
        attribute_name: &str,
        command: &str,
        additional_substitutions: Value<'v>,
        heap: Heap<'v>,
    ) -> starlark::Result<String> {
        let s = state(this);
        let extra = DictRef::from_value(additional_substitutions).ok_or_else(|| {
            fatal(format!(
                "in call to expand_make_variables(), parameter 'additional_substitutions' got \
                 value of type '{}', want 'dict'",
                additional_substitutions.get_type()
            ))
        })?;
        let mut given: BTreeMap<String, String> = BTreeMap::new();
        for (k, v) in extra.iter() {
            match (k.unpack_str(), v.unpack_str()) {
                (Some(k), Some(v)) => {
                    given.insert(k.to_owned(), v.to_owned());
                }
                _ => {
                    return Err(fatal(format!(
                        "expected a dict of string to string for 'additional_substitutions', \
                         got {}",
                        crate::args::describe(additional_substitutions)
                    )));
                }
            }
        }
        let vars = s.make_variables(heap);
        let lookup = |name: &str| given.get(name).cloned().or_else(|| vars.lookup(name));
        match fjfj_graph::expand::expand_make_variables(command, &lookup) {
            Ok(expanded) => Ok(expanded),
            Err(message) => {
                let error = fjfj_graph::expand::ExpandError {
                    attribute: attribute_name.to_owned(),
                    rule_class: s.rule_kind.clone(),
                    label: fjfj_graph::expand::label_text(&s.label),
                    message,
                }
                .to_string();
                let mut errors = s.errors.lock().unwrap();
                if !errors.contains(&error) {
                    errors.push(error);
                }
                Ok(command.to_owned())
            }
        }
    }

    /// `ctx.expand_location(input, targets = [])`. A reference that cannot be
    /// expanded is an error of the rule, which goes on after: the text comes
    /// back as it was and the rule fails when its implementation returns.
    fn expand_location<'v>(
        this: Value<'v>,
        input: &str,
        targets: Option<Value<'v>>,
        heap: Heap<'v>,
    ) -> starlark::Result<String> {
        let s = state(this);
        // What `$(location)` can name: the targets given, and what the rule
        // reads through `srcs`, `deps` and `tools` (not `data`, probed).
        let mut prerequisites = s.location_scope();
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
        Ok(s.expand_locations_or_record(input, &prerequisites, None))
    }

    /// `ctx.resolve_command(*, command, attribute, expand_locations,
    /// make_variables, tools, label_dict, execution_requirements)`: the
    /// inputs a command needs (the files of its tools), the argv that runs
    /// it with bash, and no manifests. `$(location)` is expanded only if
    /// asked to be, from the `label_dict` and the tools; make variables only
    /// if some were given.
    fn resolve_command<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let s = state(this);
        let heap = eval.heap();
        let bound = bind(
            "resolve_command",
            Wording::Signature,
            &[
                param("command", false, false),
                param("attribute", false, false),
                param("expand_locations", false, false),
                param("make_variables", false, false),
                param("tools", false, false),
                param("label_dict", false, false),
                param("execution_requirements", false, false),
            ],
            args,
            eval,
        )?;
        let mut command = bound[0]
            .and_then(|v| v.unpack_str())
            .unwrap_or_default()
            .to_owned();
        let attribute = bound[1].and_then(|v| v.unpack_str()).map(str::to_owned);
        let expand_locations = bound[2].and_then(|v| v.unpack_bool()).unwrap_or(false);
        let mut prerequisites = Vec::new();
        let mut inputs: Vec<Artifact> = Vec::new();
        let mut runfiles: Vec<Artifact> = Vec::new();
        for tool in bound[4]
            .and_then(|t| crate::attr::sequence(t, heap))
            .unwrap_or_default()
        {
            if let Some(target) = tool.downcast_ref::<super::target::TargetValue>() {
                let info = &target.info;
                prerequisites.push(Prerequisite {
                    label: info.label.clone(),
                    files: info.files.clone(),
                });
                let mut files = info.files.clone();
                if let Some(exe) = &info.executable {
                    if !files.contains(exe) {
                        files.push(exe.clone());
                    }
                    runfiles.extend(s.executable_runfiles(exe));
                }
                for file in files {
                    if !inputs.contains(&file) {
                        inputs.push(file);
                    }
                }
            }
        }
        for tree in runfiles {
            if !inputs.contains(&tree) {
                inputs.push(tree);
            }
        }
        if let Some(dict) = bound[5].and_then(DictRef::from_value) {
            for (k, v) in dict.iter() {
                let (Some(label), Some(files)) = (
                    crate::label::label_of_value(k),
                    crate::attr::sequence(v, heap),
                ) else {
                    continue;
                };
                prerequisites.push(Prerequisite {
                    label,
                    files: files
                        .into_iter()
                        .filter_map(super::file::artifact_of)
                        .collect(),
                });
            }
        }
        if expand_locations {
            command = s.expand_locations_or_record(&command, &prerequisites, attribute.as_deref());
        }
        let variables: BTreeMap<String, String> = bound[3]
            .filter(|v| !v.is_none())
            .and_then(DictRef::from_value)
            .map(|d| {
                d.iter()
                    .filter_map(|(k, v)| {
                        Some((k.unpack_str()?.to_owned(), v.unpack_str()?.to_owned()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        if !variables.is_empty() {
            let vars = s.make_variables(heap);
            let lookup = |name: &str| variables.get(name).cloned().or_else(|| vars.lookup(name));
            match fjfj_graph::expand::expand_make_variables(&command, &lookup) {
                Ok(expanded) => command = expanded,
                Err(message) => s.record_error(&message, attribute.as_deref()),
            }
        }
        let files: Vec<Value<'v>> = inputs.into_iter().map(|a| s.file(heap, a)).collect();
        let argv: Vec<Value<'v>> = ["/bin/bash", "-c", &command]
            .iter()
            .map(|a| heap.alloc(*a))
            .collect();
        Ok(heap.alloc((
            heap.alloc(AllocList(files)),
            heap.alloc(AllocList(argv)),
            heap.alloc(AllocList(Vec::<Value<'v>>::new())),
        )))
    }

    /// `ctx.resolve_tools(tools = [])`: Bazel 9 refuses it.
    fn resolve_tools<'v>(this: Value<'v>, args: &Arguments<'v, '_>) -> starlark::Result<Value<'v>> {
        let _ = (this, args);
        Err(fatal(
            "Pass an executable or tools argument to ctx.actions.run or ctx.actions.run_shell instead of calling ctx.resolve_tools.\nUse --noincompatible_disallow_ctx_resolve_tools to temporarily disable this check.",
        ))
    }
}

impl CtxState {
    /// A problem the rule reports and goes on after, once: `in <attribute>
    /// attribute of <rule> rule <label>: <message>`, or without the attribute
    /// for a call that has none.
    fn record_error(&self, message: &str, attribute: Option<&str>) {
        let label = fjfj_graph::expand::label_text(&self.label);
        let error = match attribute {
            Some(attribute) => fjfj_graph::expand::ExpandError {
                attribute: attribute.to_owned(),
                rule_class: self.rule_kind.clone(),
                label,
                message: message.to_owned(),
            }
            .to_string(),
            None => format!("in {} rule {label}: {message}", self.rule_kind),
        };
        let mut errors = self.errors.lock().unwrap();
        if !errors.contains(&error) {
            errors.push(error);
        }
    }

    /// `input` with its `$(location)` references expanded from
    /// `prerequisites`, or as it was, with the problem recorded.
    fn expand_locations_or_record(
        &self,
        input: &str,
        prerequisites: &[Prerequisite],
        attribute: Option<&str>,
    ) -> String {
        let outs: Vec<Artifact> = self.outputs.iter().map(|(_, a)| a.clone()).collect();
        let defines = self.configuration.defines.clone();
        let expander = Expander {
            rule_class: &self.rule_kind,
            attribute: attribute.unwrap_or("args"),
            label: &self.label,
            srcs: &[],
            outs: &outs,
            prerequisites,
            bin_dir: &self.bin_dir(),
            target_cpu: target_cpu(&self.configuration.cpu),
            compilation_mode: self.configuration.compilation_mode.name(),
            defines: &defines,
            toolchain_variables: &[],
            main_repo_name: &self.main_repo_name,
            context: LabelContext {
                repo: &self.label.repo,
                package: &self.label.package,
            },
        };
        match expander.expand_locations(input) {
            Ok(expanded) => expanded,
            Err(e) => {
                self.record_error(&e.message, attribute);
                input.to_owned()
            }
        }
    }

    /// The targets of `srcs`, `deps` and `tools` of the rule, which a
    /// `$(location)` in `ctx.expand_location` can name.
    fn location_scope(&self) -> Vec<Prerequisite> {
        let mut out: Vec<Prerequisite> = Vec::new();
        for (name, value) in &self.attrs {
            if !["srcs", "deps", "tools"].contains(&name.as_str()) {
                continue;
            }
            let mut labels = Vec::new();
            value.labels(&mut labels);
            for label in labels {
                if let Some(dep) = self.deps.get(label)
                    && !out.iter().any(|p| p.label == dep.label)
                {
                    out.push(Prerequisite {
                        label: dep.label.clone(),
                        files: dep.files.clone(),
                    });
                }
            }
        }
        out
    }

    fn attr_def(&self, name: &str) -> Option<&fjfj_graph::rule::AttrDef> {
        self.schema
            .attrs
            .iter()
            .find(|a| a.name == name)
            .map(|a| &a.def)
    }

    /// `ctx.attr` does not show these attributes of the rule that Bazel's
    /// has none of (probed on 9.2.0).
    fn schema_hidden(&self, name: &str) -> bool {
        [
            "aspect_hints",
            "exec_group_compatible_with",
            "applicable_licenses",
        ]
        .contains(&name)
    }

    /// `ctx.files` leaves out these label attributes too.
    fn hidden_from_files(&self, name: &str) -> bool {
        self.schema_hidden(name) || ["visibility", "transitive_configs"].contains(&name)
    }

    /// Every label attribute has its files in `ctx.files` (probed on Bazel
    /// 9.2.0), whether or not it allows source files.
    fn takes_files(&self, name: &str) -> bool {
        self.attr_def(name)
            .is_some_and(|d| matches!(d.ty, AttrType::Label | AttrType::LabelList))
    }

    fn takes_single_file(&self, name: &str) -> bool {
        self.attr_def(name).is_some_and(|d| {
            d.single_file() && matches!(d.ty, AttrType::Label | AttrType::LabelList)
        })
    }

    /// The runfiles tree of `file` if it is a `ctx.executable` of this rule (or,
    /// for an aspect, of the rule it looks at): Bazel runs such a file with
    /// the runfiles of the target it came from, as a `files_to_run` is.
    pub(crate) fn executable_runfiles(&self, file: &Artifact) -> Option<Artifact> {
        let own = self.schema.attrs.iter().any(|a| {
            self.is_executable(&a.name)
                && self.attrs.iter().any(|(n, v)| {
                    *n == a.name
                        && matches!(v, AttrValue::Label(l)
                            if self.deps.get(l).is_some_and(|d| d.executable.as_ref() == Some(file)))
                })
        });
        if own {
            return Some(Artifact {
                root: file.root.clone(),
                path: format!("{}.runfiles", file.path),
                tree: false,
                symlink: false,
            });
        }
        self.rule
            .as_ref()
            .and_then(|rule| rule.executable_runfiles(file))
    }

    fn is_executable(&self, name: &str) -> bool {
        self.attr_def(name)
            .is_some_and(|d| d.executable() && matches!(d.ty, AttrType::Label))
    }
}

/// What `$(TARGET_CPU)` is for a `--cpu`.
/// A file of the workspace status, which the build writes under `bazel-out`
/// itself: owned by no target.
fn status_file<'v>(heap: Heap<'v>, name: &str) -> Value<'v> {
    alloc_file(
        heap,
        Artifact {
            root: fjfj_graph::Root::derived("bazel-out"),
            path: name.to_owned(),
            tree: false,
            symlink: false,
        },
        Label {
            repo: String::new(),
            package: String::new(),
            name: String::new(),
        },
    )
}

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
    /// The exec group these are of; the rule's own when none.
    group: Option<usize>,
}

impl ToolchainsValue {
    fn types(&self) -> &[(Label, Option<Arc<DepInfo>>)] {
        match self.group {
            Some(i) => &self.state.exec_groups[i].1,
            None => &self.state.toolchains,
        }
    }
}

starlark_simple_value!(ToolchainsValue);

impl fmt::Debug for ToolchainsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("toolchains")
    }
}

impl fmt::Display for ToolchainsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<toolchain_context.resolved_labels: {}>",
            self.types()
                .iter()
                .map(|(l, _)| crate::label::display_label(l))
                .collect::<Vec<_>>()
                .join(", ")
        )
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
                // Written in the rule's repository, so `@name` is as it maps it.
                Label::parse_mapped(
                    text,
                    LabelContext {
                        repo: &self.state.label.repo,
                        package: &self.state.label.package,
                    },
                    &mut |apparent| {
                        self.state
                            .mappings
                            .resolve_apparent(&self.state.label.repo, apparent)
                    },
                )
                .map_err(|e| fatal(format!("invalid toolchain type '{text}': {e}")))?
            }
        };
        let Some((_, resolved)) = self.types().iter().find(|(l, _)| *l == wanted) else {
            return Err(fatal(format!(
                "In {} rule {}, toolchain type {} was requested but only types [{}] are configured",
                self.state.rule_kind,
                crate::label::display_label(&self.state.label),
                crate::label::display_label(&wanted),
                self.types()
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
            .types()
            .iter()
            .any(|(l, r)| Some(l) == label.as_ref() && r.is_some()))
    }
}

/// `ctx.exec_groups`: indexed by a group's name.
#[derive(ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct ExecGroupCollection {
    #[allocative(skip)]
    state: Arc<CtxState>,
}

starlark_simple_value!(ExecGroupCollection);

impl fmt::Debug for ExecGroupCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("exec groups")
    }
}

impl fmt::Display for ExecGroupCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<exec_group_collection>")
    }
}

#[starlark_value(type = "ExecGroupCollection")]
impl<'v> StarlarkValue<'v> for ExecGroupCollection {
    fn at(&self, index: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let name = index.unpack_str().ok_or_else(|| {
            fatal(format!(
                "in index, got a {} for the exec group name",
                index.get_type()
            ))
        })?;
        match self.state.exec_groups.iter().position(|(n, _)| n == name) {
            Some(group) => Ok(heap.alloc(ExecGroupContext {
                state: self.state.clone(),
                group,
            })),
            None => Err(fatal(format!(
                "In {} rule {}, unrecognized exec group '{name}' requested. Available exec groups: [{}]",
                self.state.rule_kind,
                crate::label::display_label(&self.state.label),
                self.state
                    .exec_groups
                    .iter()
                    .map(|(n, _)| n.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))),
        }
    }

    fn is_in(&self, other: Value<'v>) -> starlark::Result<bool> {
        Ok(other
            .unpack_str()
            .is_some_and(|name| self.state.exec_groups.iter().any(|(n, _)| n == name)))
    }
}

/// One of `ctx.exec_groups`: what the group resolved.
#[derive(ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct ExecGroupContext {
    #[allocative(skip)]
    state: Arc<CtxState>,
    group: usize,
}

starlark_simple_value!(ExecGroupContext);

impl fmt::Debug for ExecGroupContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("exec group")
    }
}

impl fmt::Display for ExecGroupContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<exec_group_context>")
    }
}

#[starlark_value(type = "ExecGroupContext")]
impl<'v> StarlarkValue<'v> for ExecGroupContext {
    fn get_attr(&self, attribute: &str, heap: Heap<'v>) -> Option<Value<'v>> {
        (attribute == "toolchains").then(|| {
            heap.alloc(ToolchainsValue {
                state: self.state.clone(),
                group: Some(self.group),
            })
        })
    }

    fn has_attr(&self, attribute: &str, _heap: Heap<'v>) -> bool {
        attribute == "toolchains"
    }

    fn dir_attr(&self) -> Vec<String> {
        vec!["toolchains".to_owned()]
    }
}
