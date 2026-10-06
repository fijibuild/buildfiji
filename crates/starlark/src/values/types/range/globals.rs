/*
 * Copyright 2019 The Starlark in Rust Authors.
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::num::NonZeroI32;

use starlark_derive::starlark_module;

use crate as starlark;
use crate::environment::GlobalsBuilder;
use crate::values::Value;
use crate::values::range::Range;
use crate::values::types::num::value::NumRef;
use crate::values::unpack::ParameterType;

#[starlark_module]
pub(crate) fn register_range(globals: &mut GlobalsBuilder) {
    /// [range](
    /// https://github.com/bazelbuild/starlark/blob/master/spec.md#range
    /// ): return a range of integers
    ///
    /// `range` returns a tuple of integers defined by the specified interval
    /// and stride.
    ///
    /// ```python
    /// range(stop)                             # equivalent to range(0, stop)
    /// range(start, stop)                      # equivalent to range(start, stop, 1)
    /// range(start, stop, step)
    /// ```
    ///
    /// `range` requires between one and three integer arguments.
    /// With one argument, `range(stop)` returns the ascending sequence of
    /// non-negative integers less than `stop`.
    /// With two arguments, `range(start, stop)` returns only integers not less
    /// than `start`.
    ///
    /// With three arguments, `range(start, stop, step)` returns integers
    /// formed by successively adding `step` to `start` until the value meets or
    /// passes `stop`. A call to `range` fails if the value of `step` is
    /// zero.
    ///
    /// ```
    /// # starlark::assert::all_true(r#"
    /// list(range(10))                         == [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
    /// list(range(3, 10))                      == [3, 4, 5, 6, 7, 8, 9]
    /// list(range(3, 10, 2))                   == [3, 5, 7, 9]
    /// list(range(10, 3, -2))                  == [10, 8, 6, 4]
    /// # "#);
    /// ```
    #[starlark(as_type = Range, speculative_exec_safe)]
    fn range<'v>(
        #[starlark(require = pos)] a1: Value<'v>,
        #[starlark(require = pos)] a2: Option<Value<'v>>,
        #[starlark(require = pos)] step: Option<Value<'v>>,
    ) -> starlark::Result<Range> {
        // Bazel takes 32-bit arguments, and says which one is not.
        let int = |value: Value, what: &str, param: &str| -> starlark::Result<i32> {
            match value.unpack_num() {
                Some(NumRef::Int(i)) => i.to_i32().ok_or_else(|| {
                    crate::Error::new_native(anyhow::anyhow!(
                        "got {} for {what}, want value in signed 32-bit range",
                        value.to_str()
                    ))
                }),
                _ => Err(crate::Error::new_value(ParameterType {
                    function: Some("range".to_owned()),
                    param: param.to_owned(),
                    want: "int".to_owned(),
                    actual: value.get_type().to_owned(),
                })),
            }
        };
        let (start, stop) = match a2 {
            None => (0, int(a1, "stop", "start_or_stop")?),
            Some(a2) => (int(a1, "start", "start_or_stop")?, int(a2, "stop", "stop")?),
        };
        let step = match step {
            None => 1,
            Some(step) => int(step, "step", "step")?,
        };
        let Some(step) = NonZeroI32::new(step) else {
            return Err(crate::Error::new_native(anyhow::anyhow!(
                "step cannot be 0"
            )));
        };
        let range = Range::new(start, stop, step);
        if crate::values::StarlarkValue::length(&range).is_err() {
            return Err(crate::Error::new_native(anyhow::anyhow!(
                "len(range({start}, {stop}{})) exceeds signed 32-bit range",
                if step.get() == 1 {
                    String::new()
                } else {
                    format!(", {step}")
                }
            )));
        }
        Ok(range)
    }
}
