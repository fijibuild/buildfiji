//! `ctx.fragments` (buildfiji-136.18): the structs of the configuration
//! fragments, made by `_make_fragments` of the builtins from the options of a
//! configuration. They are frozen once per configuration and live as long as
//! the process.

use super::run::builtins_owner;
use starlark::environment::Module;
use starlark::eval::Evaluator;
use starlark::values::FrozenValue;
use starlark::values::dict::AllocDict;
use std::collections::BTreeMap;
use std::sync::Mutex;

type Key = (String, String);

pub(super) fn fragments_of(cpu: &str, compilation_mode: &str) -> Result<FrozenValue, String> {
    static MADE: Mutex<BTreeMap<Key, FrozenValue>> = Mutex::new(BTreeMap::new());
    let key = (cpu.to_owned(), compilation_mode.to_owned());
    if let Some(made) = MADE.lock().unwrap().get(&key) {
        return Ok(*made);
    }
    let frozen = Module::with_temp_heap(|module| -> Result<_, String> {
        let (function, _) = crate::label::builtins()
            .get_any_visibility("_make_fragments")
            .map_err(|_| "the builtins have no _make_fragments".to_owned())?;
        module.frozen_heap().add_reference(function.owner());
        module.frozen_heap().add_reference(builtins_owner());
        let heap = module.heap();
        let options = heap.alloc(AllocDict([
            ("cpu", heap.alloc(cpu)),
            ("compilation_mode", heap.alloc(compilation_mode)),
        ]));
        let made = {
            let mut eval = Evaluator::new(&module);
            eval.eval_function(
                function
                    .value()
                    .unpack_frozen()
                    .expect("a global is frozen")
                    .to_value(),
                &[options],
                &[],
            )
            .map_err(|e| format!("{e}"))?
        };
        module.set("fragments", made);
        module.freeze().map_err(|e| format!("{e:?}"))
    })?;
    // The values point into the module's heap, which has to stay.
    let frozen: &'static starlark::environment::FrozenModule = Box::leak(Box::new(frozen));
    let (made, _) = frozen
        .get_any_visibility("fragments")
        .map_err(|e| format!("{e:?}"))?;
    let made = made.value().unpack_frozen().expect("a global is frozen");
    MADE.lock().unwrap().insert(key, made);
    Ok(made)
}
