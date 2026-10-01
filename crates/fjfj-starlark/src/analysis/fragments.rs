//! `ctx.fragments` and `ctx.configuration` (buildfiji-136.18): structs the
//! builtins make (`_make_fragments`, `_make_configuration`) from the options
//! of a configuration. They are frozen once per set of options and live as long
//! as the process.

use super::run::builtins_owner;
use starlark::environment::{FrozenModule, Module};
use starlark::eval::Evaluator;
use starlark::values::FrozenValue;
use starlark::values::dict::AllocDict;
use std::collections::BTreeMap;
use std::sync::Mutex;

type Key = (String, Vec<(String, String)>);

/// What `function` of the builtins makes of `options`.
pub(super) fn made_by(function: &str, options: &[(&str, &str)]) -> Result<FrozenValue, String> {
    static MADE: Mutex<BTreeMap<Key, FrozenValue>> = Mutex::new(BTreeMap::new());
    let key: Key = (
        function.to_owned(),
        options
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect(),
    );
    if let Some(made) = MADE.lock().unwrap().get(&key) {
        return Ok(*made);
    }
    let frozen = Module::with_temp_heap(|module| -> Result<_, String> {
        let (function, _) = crate::label::builtins()
            .get_any_visibility(function)
            .map_err(|_| format!("the builtins have no {function}"))?;
        module.frozen_heap().add_reference(function.owner());
        module.frozen_heap().add_reference(builtins_owner());
        let heap = module.heap();
        let options = heap.alloc(AllocDict(
            options
                .iter()
                .map(|(k, v)| (heap.alloc(*k), heap.alloc(*v))),
        ));
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
        module.set("made", made);
        module.freeze().map_err(|e| format!("{e:?}"))
    })?;
    // The values point into the module's heap, which has to stay.
    let frozen: &'static FrozenModule = Box::leak(Box::new(frozen));
    let (made, _) = frozen
        .get_any_visibility("made")
        .map_err(|e| format!("{e:?}"))?;
    let made = made.value().unpack_frozen().expect("a global is frozen");
    MADE.lock().unwrap().insert(key, made);
    Ok(made)
}
