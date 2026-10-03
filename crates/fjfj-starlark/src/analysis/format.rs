//! `cquery --output=starlark` (buildfiji-tle.3): the user's `format(target)`
//! run on each configured target, its result printed.

use super::run::builtins_owner;
use super::target::{DepInfo, TargetValue, alloc_configured_target};
use super::transition::to_starlark;
use crate::args::fatal;
use crate::label::{BzlEval, BzlFile, RepoMappings, evaluate_bzl};
use fjfj_graph::{Label, SettingValue};
use starlark::environment::{FrozenModule, GlobalsBuilder, Module};
use starlark::eval::{Evaluator, FileLoader};
use starlark::starlark_module;
use starlark::values::{Heap, Value, ValueLike};
use std::sync::Arc;

/// A configured target to format.
pub struct FormatTarget {
    pub info: Arc<DepInfo>,
    /// The options of its configuration, by `//command_line_option:name` or
    /// the label of a build setting.
    pub build_options: Vec<(String, SettingValue)>,
}

#[starlark_module]
pub(crate) fn format_functions(builder: &mut GlobalsBuilder) {
    /// `providers(target)`: the providers the target gave, by name.
    fn providers<'v>(target: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let target = target
            .downcast_ref::<TargetValue>()
            .ok_or_else(|| fatal("providers() takes a target"))?;
        Ok(heap.alloc(starlark::values::dict::AllocDict(
            target
                .provider_instances(heap)
                .into_iter()
                .map(|(name, value)| (heap.alloc(name), value)),
        )))
    }

    /// `build_options(target)`: the options of the target's configuration.
    fn build_options<'v>(target: Value<'v>, heap: Heap<'v>) -> starlark::Result<Value<'v>> {
        let target = target
            .downcast_ref::<TargetValue>()
            .ok_or_else(|| fatal("build_options() takes a target"))?;
        let options = target.build_options.as_deref().map_or(&[][..], |o| &o[..]);
        Ok(
            heap.alloc(starlark::values::dict::AllocDict(options.iter().map(
                |(name, value)| (heap.alloc(name.as_str()), to_starlark(heap, name, value)),
            ))),
        )
    }
}

/// A `--starlark:file` cannot `load()`.
struct NoLoads;

impl FileLoader for NoLoads {
    fn load(&self, path: &str) -> starlark::Result<FrozenModule> {
        Err(starlark::Error::new_other(anyhow::anyhow!(
            "cannot load '{path}': a cquery format file cannot load other files"
        )))
    }
}

/// What each of `targets` makes of `format`, in order. `source` is a file
/// that defines `format(target)`, or with `expr` the expression it returns.
/// A string is printed as it is, anything else as `str` shows it.
pub fn format_targets(
    name: &str,
    source: &str,
    expr: bool,
    mappings: &RepoMappings,
    targets: &[FormatTarget],
) -> Result<Vec<String>, String> {
    let code = if expr {
        format!("def format(target):\n  return {source}\n")
    } else {
        source.to_owned()
    };
    let file = Label {
        repo: String::new(),
        package: String::new(),
        name: name.to_owned(),
    };
    let globals = crate::native::format_globals();
    let frozen = evaluate_bzl(&BzlFile {
        file: &file,
        source: &code,
        globals: &globals,
        mappings,
        loader: &NoLoads,
        print: None,
    })
    .map_err(|e| e.to_string())?;
    let (format, _) = frozen
        .get_any_visibility("format")
        .map_err(|_| "Starlark file must define a function named 'format'".to_owned())?;
    let mut out = Vec::with_capacity(targets.len());
    for target in targets {
        let text = Module::with_temp_heap(|module| -> Result<String, String> {
            module.frozen_heap().add_reference(format.owner());
            module.frozen_heap().add_reference(builtins_owner());
            for provider in &target.info.providers {
                module.frozen_heap().add_reference(provider.value.owner());
            }
            let function = format
                .value()
                .unpack_frozen()
                .expect("a global is frozen")
                .to_value();
            let value = alloc_configured_target(
                module.heap(),
                target.info.clone(),
                target.build_options.clone(),
            );
            let running = BzlEval::running(mappings);
            let mut eval = Evaluator::new(&module);
            eval.extra = Some(&running);
            let result = eval
                .eval_function(function, &[value], &[])
                .map_err(|e| format!("Starlark evaluation error for {}: {e}", target.info.label))?;
            Ok(result
                .unpack_str()
                .map_or_else(|| result.to_str(), str::to_owned))
        })?;
        out.push(text);
    }
    Ok(out)
}
