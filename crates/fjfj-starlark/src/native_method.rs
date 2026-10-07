//! A method of a module value that Bazel implements natively
//! (buildfiji-eczg): `testing.TestEnvironment`, `coverage_common.
//! instrumented_files_info` and the like are Starlark functions here, and
//! this makes one print and type as Bazel's do:
//! `<built-in method NAME of MODULE value>` and `builtin_function_or_method`.

use allocative::Allocative;
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_complex_value;
use starlark::values::{
    Coerce, Freeze, FreezeResult, Freezer, NoSerialize, ProvidesStaticType, StarlarkPagablePanic,
    StarlarkValue, Trace, Value, ValueLike,
};
use starlark_derive::starlark_value;
use std::fmt;

#[derive(
    Debug, Trace, Coerce, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative,
)]
#[repr(C)]
pub(crate) struct NativeMethodGen<V> {
    #[trace(static)]
    module: String,
    #[trace(static)]
    name: String,
    function: V,
}

starlark_complex_value!(pub(crate) NativeMethod);

impl<'v> Freeze for NativeMethod<'v> {
    type Frozen = FrozenNativeMethod;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenNativeMethod> {
        Ok(NativeMethodGen {
            module: self.module,
            name: self.name,
            function: self.function.freeze(freezer)?,
        })
    }
}

impl<V> fmt::Display for NativeMethodGen<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<built-in method {} of {} value>",
            self.name, self.module
        )
    }
}

#[starlark_value(type = "builtin_function_or_method")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for NativeMethodGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn invoke(
        &self,
        _me: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        self.function.to_value().invoke(args, eval)
    }

    fn name_for_call_stack(&self, _me: Value<'v>) -> String {
        self.name.clone()
    }
}

/// `function` as the method `name` of the module value `module`.
pub(crate) fn wrap<'v>(
    heap: starlark::values::Heap<'v>,
    module: &str,
    name: &str,
    function: Value<'v>,
) -> Value<'v> {
    heap.alloc_complex(NativeMethodGen {
        module: module.to_owned(),
        name: name.to_owned(),
        function,
    })
}

/// A function Bazel defines in its `@_builtins` Starlark, which prints with
/// the file it is in: `<function name from @@_builtins//:path.bzl>`.
#[derive(
    Debug, Trace, Coerce, ProvidesStaticType, NoSerialize, StarlarkPagablePanic, Allocative,
)]
#[repr(C)]
pub(crate) struct BuiltinsFunctionGen<V> {
    #[trace(static)]
    name: String,
    #[trace(static)]
    file: String,
    function: V,
}

starlark_complex_value!(pub(crate) BuiltinsFunction);

impl<'v> Freeze for BuiltinsFunction<'v> {
    type Frozen = FrozenBuiltinsFunction;

    fn freeze(self, freezer: &Freezer) -> FreezeResult<FrozenBuiltinsFunction> {
        Ok(BuiltinsFunctionGen {
            name: self.name,
            file: self.file,
            function: self.function.freeze(freezer)?,
        })
    }
}

impl<V> fmt::Display for BuiltinsFunctionGen<V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "<function {} from @@_builtins//:{}>",
            self.name, self.file
        )
    }
}

#[starlark_value(type = "function")]
impl<'v, V: ValueLike<'v>> StarlarkValue<'v> for BuiltinsFunctionGen<V>
where
    Self: ProvidesStaticType<'v>,
{
    fn invoke(
        &self,
        _me: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        self.function.to_value().invoke(args, eval)
    }

    fn name_for_call_stack(&self, _me: Value<'v>) -> String {
        self.name.clone()
    }
}

/// `function` as the function `name` of the file `file` of Bazel's builtins.
pub(crate) fn wrap_function<'v>(
    heap: starlark::values::Heap<'v>,
    name: &str,
    file: &str,
    function: Value<'v>,
) -> Value<'v> {
    heap.alloc_complex(BuiltinsFunctionGen {
        name: name.to_owned(),
        file: file.to_owned(),
        function,
    })
}
