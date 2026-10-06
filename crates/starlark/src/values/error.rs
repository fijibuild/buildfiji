/*
 * Copyright 2018 The Starlark in Rust Authors.
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

//! Define a common set of errors.

use thiserror::Error;

use crate::values::StarlarkValue;
use crate::values::Value;

/// Common errors returned by Starlark evaluation.
#[derive(Debug, Error)]
#[allow(missing_docs)] // Self-explanatory.
pub enum ValueError {
    #[error("{}", unary_message(op, typ))]
    OperationNotSupported { op: String, typ: String },
    #[error("{}", binary_message(op, left, right))]
    OperationNotSupportedBinary {
        op: String,
        left: String,
        right: String,
    },
    #[error("floating-point division by zero")]
    DivisionByZero,
    #[error("Integer overflow")]
    IntegerOverflow,
    #[error("Negative shift count")]
    NegativeShiftCount,
    #[error("Type of parameters mismatch")]
    IncorrectParameterType,
    #[error("Type of parameter `{0}` doesn't match")]
    IncorrectParameterTypeNamed(String),
    #[error("Missing this parameter")]
    MissingThis,
    #[error("Missing required parameter `{0}`")]
    MissingRequired(String),
    #[error("Index `{0}` is out of bound")]
    IndexOutOfBound(i32),
    #[error("index out of range (index is {index}, but sequence has {len} elements)")]
    SequenceIndex { index: i32, len: usize },
    #[error("slice step cannot be zero")]
    SliceStepZero,
    #[error("key {0} not found in dictionary")]
    KeyNotFound(String),
    #[error("Immutable")]
    CannotMutateImmutableValue,
    #[error("This operation mutates an iterable for an iterator while iterating.")]
    MutationDuringIteration,
    #[error("'{0}' value has no field or method '{1}'")]
    NoAttr(String, String),
    #[error("'{0}' value has no field or method '{1}', did you mean '{2}'?")]
    NoAttrDidYouMean(String, String, String),
}

/// How Bazel words an operation a type does not have (fjfj).
fn unary_message(op: &str, typ: &str) -> String {
    match op {
        "-" | "+" | "~" => format!("unsupported unary operation: {op}{typ}"),
        "(iter)" => format!("type '{typ}' is not iterable"),
        "call()" => format!("'{typ}' object is not callable"),
        _ => format!("Operation `{op}` not supported on type `{typ}`"),
    }
}

/// How Bazel words a binary operation or a comparison two types do not have (fjfj).
fn binary_message(op: &str, left: &str, right: &str) -> String {
    match op {
        "compare" => format!("unsupported comparison: {left} <=> {right}"),
        "[]" => format!("type '{left}' has no operator []({right})"),
        _ => format!("unsupported binary operation: {left} {op} {right}"),
    }
}

impl From<ValueError> for crate::Error {
    fn from(e: ValueError) -> Self {
        crate::Error::new_kind(crate::ErrorKind::Value(anyhow::Error::new(e)))
    }
}

#[derive(Debug, Error)]
pub(crate) enum ControlError {
    #[error("unhashable type: '{0}'")]
    NotHashableValue(String),
    #[error("Too many recursion levels")]
    TooManyRecursionLevel,
}

impl ValueError {
    #[cold]
    pub(crate) fn unsupported_owned<T>(
        left: &str,
        op: &str,
        right: Option<&str>,
    ) -> crate::Result<T> {
        match right {
            None => Err(ValueError::OperationNotSupported {
                op: op.to_owned(),
                typ: left.to_owned(),
            }
            .into()),
            Some(right) => Err(ValueError::OperationNotSupportedBinary {
                op: op.to_owned(),
                left: left.to_owned(),
                right: right.to_owned(),
            }
            .into()),
        }
    }

    /// Helper to create an [`OperationNotSupported`](ValueError::OperationNotSupported) error.
    #[cold]
    pub fn unsupported<'v, T, V: StarlarkValue<'v>>(_left: &V, op: &str) -> crate::Result<T> {
        Self::unsupported_owned(V::TYPE, op, None)
    }

    #[cold]
    pub(crate) fn unsupported_type<T>(left: Value, op: &str) -> crate::Result<T> {
        Self::unsupported_owned(left.get_type(), op, None)
    }

    /// Helper to create an [`OperationNotSupported`](ValueError::OperationNotSupportedBinary) error.
    #[cold]
    pub fn unsupported_with<'v, T, V: StarlarkValue<'v>>(
        _left: &V,
        op: &str,
        right: Value,
    ) -> crate::Result<T> {
        Self::unsupported_owned(V::TYPE, op, Some(right.get_type()))
    }
}
