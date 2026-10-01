//! Providers a native rule gives (buildfiji-136.7): built in Rust, frozen,
//! and stored like the ones a rule's code returns.

use super::target::{StoredProvider, builtin};
use crate::label::StarlarkLabel;
use crate::structs::{fields_of, new_instance};
use fjfj_graph::Label;
use starlark::environment::Module;
use starlark::values::Value;

/// A field of a native provider.
pub enum Field {
    Label(Label),
    None,
    String(String),
    /// Another provider instance, kept.
    Provider(StoredProvider),
    List(Vec<Field>),
}

/// An instance of the builtin provider `path` (`platform_common.ConstraintValueInfo`)
/// with these fields.
pub fn native_provider(path: &str, fields: Vec<(String, Field)>) -> Result<StoredProvider, String> {
    let mut names = path.split('.');
    let first = names.next().expect("a path");
    Module::with_temp_heap(|module| -> Result<StoredProvider, String> {
        module
            .frozen_heap()
            .add_reference(crate::analysis::run::builtins_owner());
        let mut provider = builtin(first).ok_or_else(|| format!("the builtins have no {first}"))?;
        for name in names {
            provider = fields_of(provider)
                .and_then(|f| f.into_iter().find(|(n, _)| *n == name).map(|(_, v)| v))
                .ok_or_else(|| format!("{path} is not a builtin provider"))?;
        }
        let heap = module.heap();
        let mut values = Vec::new();
        for (name, field) in fields {
            values.push((name, value_of(&module, field)));
        }
        let instance = new_instance(heap, provider, values);
        module.set("instance", instance);
        let frozen = module.freeze().map_err(|e| format!("{e:?}"))?;
        let (value, _) = frozen
            .get_any_visibility("instance")
            .map_err(|e| format!("{e:?}"))?;
        Ok(StoredProvider { value })
    })
}

fn value_of<'v>(module: &Module<'v>, field: Field) -> Value<'v> {
    let heap = module.heap();
    match field {
        Field::Label(label) => heap.alloc(StarlarkLabel::from(label)),
        Field::None => Value::new_none(),
        Field::String(s) => heap.alloc(s.as_str()),
        Field::Provider(stored) => {
            module.frozen_heap().add_reference(stored.value.owner());
            // SAFETY: the module holds a reference to the heap that owns it.
            unsafe { stored.value.unchecked_frozen_value().to_value() }
        }
        Field::List(items) => {
            let values: Vec<Value<'v>> = items.into_iter().map(|f| value_of(module, f)).collect();
            heap.alloc(values)
        }
    }
}
