//! Stub Bazel environment for the evaluation phase of the spike.
//!
//! fjfj has no Bazel builtins yet, so a corpus BUILD file cannot be evaluated
//! for real. To still get a parse-versus-evaluate split, every name the corpus
//! calls (`cc_library`, `glob`, `select`, ...) is bound to one no-op native
//! function that ignores its arguments and returns an empty list, every
//! `a.b(...)` root becomes a namespace of the same stubs, and `load()` resolves
//! to a frozen module holding a stub for every symbol the corpus loads.
//!
//! Stub evaluation therefore does all the pure-Starlark work of a real load
//! (compile, control flow, string and list building, comprehensions, argument
//! passing) and none of the builtin work (globbing, rule instantiation,
//! depsets, providers). That makes the measured evaluate time a lower bound,
//! and the parse share of load an upper bound.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use starlark::environment::FrozenModule;
use starlark::environment::Globals;
use starlark::environment::GlobalsBuilder;
use starlark::eval::FileLoader;
use starlark::starlark_module;
use starlark::values::FrozenValue;
use starlark::values::Heap;
use starlark::values::Value;
use starlark::values::tuple::UnpackTuple;

/// Names a corpus scan found, used to shape the stub environment.
#[derive(Default)]
pub struct Names {
    /// Identifiers used as `name(...)`.
    pub callables: BTreeSet<String>,
    /// Roots of `root.member` accesses, mapped to the members seen.
    pub namespaces: BTreeMap<String, BTreeSet<String>>,
    /// Symbols named in `load()` statements (the source name, not the alias).
    pub loaded: BTreeSet<String>,
}

#[starlark_module]
fn stub_global(builder: &mut GlobalsBuilder) {
    fn __fjfj_stub__<'v>(
        #[starlark(args)] _args: UnpackTuple<Value<'v>>,
        #[starlark(kwargs)] _kwargs: Value<'v>,
        heap: Heap<'v>,
    ) -> anyhow::Result<Value<'v>> {
        Ok(heap.alloc(Vec::<Value<'v>>::new()))
    }
}

/// A stub environment: globals plus the module every `load()` resolves to.
pub struct StubEnv {
    /// Kept alive because `globals` and `loaded_module` reference its heap.
    _base: Globals,
    pub globals: Globals,
    loaded_module: FrozenModule,
}

impl StubEnv {
    pub fn new(names: &Names) -> anyhow::Result<StubEnv> {
        let base = GlobalsBuilder::standard().with(stub_global).build();
        let stub: FrozenValue = base
            .iter()
            .find(|(n, _)| *n == "__fjfj_stub__")
            .expect("stub global is registered")
            .1;
        let standard: BTreeSet<String> = Globals::standard()
            .names()
            .map(|n| n.as_str().to_owned())
            .collect();

        let mut builder = GlobalsBuilder::standard();
        builder.frozen_heap().add_reference(base.heap());
        for name in &names.callables {
            if !standard.contains(name) {
                builder.set(name, stub);
            }
        }
        for (root, members) in &names.namespaces {
            // A root that is also called has to stay a function; `a.b` on it
            // fails, which the evaluate phase counts as a stub-environment miss.
            if standard.contains(root) || names.callables.contains(root) {
                continue;
            }
            builder.namespace(root, |ns| {
                for member in members {
                    ns.set(member, stub);
                }
            });
        }
        let globals = builder.build();

        // Every `load()` resolves to this module: one stub per symbol the
        // corpus loads, built as globals because `FrozenModule` can only be
        // made from a `Globals` or a heap-bound `Module`.
        let mut loaded = GlobalsBuilder::new();
        loaded.frozen_heap().add_reference(base.heap());
        for name in &names.loaded {
            loaded.set(name, stub);
        }
        let loaded_module = FrozenModule::from_globals(&loaded.build())?;

        Ok(StubEnv {
            _base: base,
            globals,
            loaded_module,
        })
    }
}

impl FileLoader for StubEnv {
    fn load(&self, _path: &str) -> starlark::Result<FrozenModule> {
        Ok(self.loaded_module.clone())
    }
}
