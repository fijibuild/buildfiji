//! `Args`, what `ctx.actions.args()` builds (buildfiji-136.3).
//!
//! What each method adds was read off `bazel aquery` of a rule that used all
//! of them (Bazel 9.2.0). The arguments are expanded as they are added, not
//! when the action runs, so a `map_each` is called during analysis.

use super::file::artifact_of;
use crate::args::{Wording, bind, describe, fatal, param};
use crate::depset::{depset_to_list, is_depset};
use allocative::Allocative;
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::eval::{Arguments, Evaluator};
use starlark::starlark_module;
use starlark::starlark_simple_value;
use starlark::values::{NoSerialize, ProvidesStaticType, StarlarkValue, Value, ValueLike};
use starlark_derive::starlark_value;
use std::fmt;
use std::sync::Mutex;

/// How a parameter file lists its arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParamFormat {
    /// Each argument on a line, quoted as a shell would need.
    Shell,
    /// Each argument on a line, as it is.
    Multiline,
    /// `--flag=value` arguments split at the `=`, each half on a line.
    FlagPerLine,
}

#[derive(Debug, Clone)]
pub(crate) struct ParamFile {
    /// `@%s`: how the file is named on the command line.
    pub(crate) pattern: String,
    pub(crate) use_always: bool,
    pub(crate) format: ParamFormat,
}

#[derive(Debug, Default)]
pub(crate) struct ArgsState {
    pub(crate) items: Vec<String>,
    pub(crate) param_file: Option<ParamFile>,
    pub(crate) format: Option<ParamFormat>,
}

#[derive(ProvidesStaticType, NoSerialize, Allocative)]
pub(crate) struct ArgsValue {
    #[allocative(skip)]
    pub(crate) state: Mutex<ArgsState>,
}

starlark_simple_value!(ArgsValue);

impl fmt::Debug for ArgsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Args")
    }
}

impl fmt::Display for ArgsValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Args")
    }
}

#[starlark_value(type = "Args")]
impl<'v> StarlarkValue<'v> for ArgsValue {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("Args", args_members);
        Some(RES.methods())
    }
}

impl ArgsValue {
    pub(crate) fn new() -> ArgsValue {
        ArgsValue {
            state: Mutex::new(ArgsState::default()),
        }
    }
}

fn args_of<'v>(this: Value<'v>) -> &'v ArgsValue {
    this.downcast_ref::<ArgsValue>().expect("Args")
}

/// An argument value as text.
fn text_of(value: Value<'_>) -> Option<String> {
    if let Some(s) = value.unpack_str() {
        return Some(s.to_owned());
    }
    if let Some(file) = artifact_of(value) {
        return Some(file.exec_path());
    }
    if value.get_type() == "int" {
        return Some(value.to_str());
    }
    if value.get_type() == "Label" {
        return Some(value.to_str());
    }
    None
}

fn wrong(function: &str, value: Value<'_>) -> starlark::Error {
    fatal(format!(
        "{function}: expected a string, int, File or Label, got {}",
        describe(value)
    ))
}

/// `pattern` with `%s` replaced; `%%` is a `%`.
fn formatted(function: &str, pattern: &str, text: &str) -> starlark::Result<String> {
    let mut out = String::with_capacity(pattern.len() + text.len());
    let mut holes = 0;
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, chars.peek()) {
            ('%', Some('s')) => {
                chars.next();
                holes += 1;
                out.push_str(text);
            }
            ('%', Some('%')) => {
                chars.next();
                out.push('%');
            }
            _ => out.push(c),
        }
    }
    if holes != 1 {
        return Err(fatal(format!(
            "{function}: format '{pattern}' must contain exactly one '%s'"
        )));
    }
    Ok(out)
}

fn flag(
    function: &str,
    name: &str,
    value: Option<Value<'_>>,
    default: bool,
) -> starlark::Result<bool> {
    match value {
        None => Ok(default),
        Some(v) => v.unpack_bool().ok_or_else(|| {
            fatal(format!(
                "in call to {function}(), parameter '{name}' got value of type '{}', want 'bool'",
                v.get_type()
            ))
        }),
    }
}

fn string_opt(
    function: &str,
    name: &str,
    value: Option<Value<'_>>,
) -> starlark::Result<Option<String>> {
    match value.filter(|v| !v.is_none()) {
        None => Ok(None),
        Some(v) => v.unpack_str().map(|s| Some(s.to_owned())).ok_or_else(|| {
            fatal(format!(
                "in call to {function}(), parameter '{name}' got value of type '{}', want 'string or NoneType'",
                v.get_type()
            ))
        }),
    }
}

/// The items to add for `values`, through `map_each` and `format_each`.
#[allow(clippy::too_many_arguments)]
fn expand<'v>(
    function: &str,
    values: Value<'v>,
    map_each: Option<Value<'v>>,
    format_each: Option<&str>,
    uniquify: bool,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Vec<String>> {
    let items: Vec<Value<'v>> = if is_depset(values) {
        depset_to_list(values).expect("a depset")?
    } else if let Some(items) = crate::args::sequence(values) {
        items
    } else {
        return Err(fatal(format!(
            "in call to {function}(), parameter 'values' got value of type '{}', want 'sequence or depset'",
            values.get_type()
        )));
    };
    let mut out: Vec<String> = Vec::new();
    for item in items {
        let mut produced: Vec<String> = Vec::new();
        match map_each.filter(|f| !f.is_none()) {
            Some(f) => {
                let mapped = eval.eval_function(f, &[item], &[])?;
                if mapped.is_none() {
                    continue;
                }
                if let Some(text) = text_of(mapped) {
                    produced.push(text);
                } else if let Some(list) = crate::args::sequence(mapped) {
                    for each in list {
                        produced.push(text_of(each).ok_or_else(|| wrong(function, each))?);
                    }
                } else {
                    return Err(wrong(function, mapped));
                }
            }
            None => produced.push(text_of(item).ok_or_else(|| wrong(function, item))?),
        }
        for text in produced {
            let text = match format_each {
                Some(pattern) => formatted(function, pattern, &text)?,
                None => text,
            };
            if !uniquify || !out.contains(&text) {
                out.push(text);
            }
        }
    }
    Ok(out)
}

#[starlark_module]
fn args_members(builder: &mut MethodsBuilder) {
    /// `args.add(arg_name_or_value, value, *, format)`.
    fn add<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let bound = bind(
            "add",
            Wording::Signature,
            &[
                param("arg_name_or_value", true, true),
                param("value", true, false),
                param("format", false, false),
            ],
            args,
            eval,
        )?;
        let first = bound[0].expect("required");
        let format = string_opt("add", "format", bound[2])?;
        let mut state = args_of(this).state.lock().unwrap();
        match bound[1] {
            None => {
                let text = text_of(first).ok_or_else(|| wrong("add", first))?;
                state.items.push(match &format {
                    Some(p) => formatted("add", p, &text)?,
                    None => text,
                });
            }
            Some(value) => {
                let name = first
                    .unpack_str()
                    .ok_or_else(|| fatal("add: the argument name must be a string"))?;
                state.items.push(name.to_owned());
                let text = text_of(value).ok_or_else(|| wrong("add", value))?;
                state.items.push(match &format {
                    Some(p) => formatted("add", p, &text)?,
                    None => text,
                });
            }
        }
        Ok(this)
    }

    /// `args.add_all(arg_name_or_values, values, *, map_each, format_each,
    /// before_each, omit_if_empty, uniquify, expand_directories, terminate_with,
    /// allow_closure)`.
    fn add_all<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let bound = bind(
            "add_all",
            Wording::Signature,
            &[
                param("arg_name_or_values", true, true),
                param("values", true, false),
                param("map_each", false, false),
                param("format_each", false, false),
                param("before_each", false, false),
                param("omit_if_empty", false, false),
                param("uniquify", false, false),
                param("expand_directories", false, false),
                param("terminate_with", false, false),
                param("allow_closure", false, false),
            ],
            args,
            eval,
        )?;
        let (name, values) = match bound[1] {
            Some(values) => (
                Some(
                    bound[0]
                        .and_then(|v| v.unpack_str())
                        .ok_or_else(|| fatal("add_all: the argument name must be a string"))?
                        .to_owned(),
                ),
                values,
            ),
            None => (None, bound[0].expect("required")),
        };
        let format_each = string_opt("add_all", "format_each", bound[3])?;
        let before_each = string_opt("add_all", "before_each", bound[4])?;
        let omit_if_empty = flag("add_all", "omit_if_empty", bound[5], true)?;
        let uniquify = flag("add_all", "uniquify", bound[6], false)?;
        let terminate_with = string_opt("add_all", "terminate_with", bound[8])?;
        let items = expand(
            "add_all",
            values,
            bound[2],
            format_each.as_deref(),
            uniquify,
            eval,
        )?;
        if items.is_empty() && omit_if_empty {
            return Ok(this);
        }
        let mut state = args_of(this).state.lock().unwrap();
        if let Some(name) = name {
            state.items.push(name);
        }
        for item in items {
            if let Some(before) = &before_each {
                state.items.push(before.clone());
            }
            state.items.push(item);
        }
        if let Some(end) = terminate_with {
            state.items.push(end);
        }
        Ok(this)
    }

    /// `args.add_joined(arg_name_or_values, values, *, join_with, map_each,
    /// format_each, format_joined, omit_if_empty, uniquify, expand_directories,
    /// allow_closure)`.
    fn add_joined<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let bound = bind(
            "add_joined",
            Wording::Signature,
            &[
                param("arg_name_or_values", true, true),
                param("values", true, false),
                param("join_with", false, true),
                param("map_each", false, false),
                param("format_each", false, false),
                param("format_joined", false, false),
                param("omit_if_empty", false, false),
                param("uniquify", false, false),
                param("expand_directories", false, false),
                param("allow_closure", false, false),
            ],
            args,
            eval,
        )?;
        let (name, values) = match bound[1] {
            Some(values) => (
                Some(
                    bound[0]
                        .and_then(|v| v.unpack_str())
                        .ok_or_else(|| fatal("add_joined: the argument name must be a string"))?
                        .to_owned(),
                ),
                values,
            ),
            None => (None, bound[0].expect("required")),
        };
        let join_with = bound[2]
            .and_then(|v| v.unpack_str())
            .ok_or_else(|| fatal("in call to add_joined(), parameter 'join_with' got value of type that is not 'string'"))?
            .to_owned();
        let format_each = string_opt("add_joined", "format_each", bound[4])?;
        let format_joined = string_opt("add_joined", "format_joined", bound[5])?;
        let omit_if_empty = flag("add_joined", "omit_if_empty", bound[6], true)?;
        let uniquify = flag("add_joined", "uniquify", bound[7], false)?;
        let items = expand(
            "add_joined",
            values,
            bound[3],
            format_each.as_deref(),
            uniquify,
            eval,
        )?;
        if items.is_empty() && omit_if_empty {
            return Ok(this);
        }
        let mut joined = items.join(&join_with);
        if let Some(pattern) = format_joined {
            joined = formatted("add_joined", &pattern, &joined)?;
        }
        let mut state = args_of(this).state.lock().unwrap();
        if let Some(name) = name {
            state.items.push(name);
        }
        state.items.push(joined);
        Ok(this)
    }

    /// `args.use_param_file(param_file_arg, *, use_always = False)`.
    fn use_param_file<'v>(
        this: Value<'v>,
        args: &Arguments<'v, '_>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let bound = bind(
            "use_param_file",
            Wording::Signature,
            &[
                param("param_file_arg", true, true),
                param("use_always", false, false),
            ],
            args,
            eval,
        )?;
        let pattern = bound[0]
            .and_then(|v| v.unpack_str())
            .ok_or_else(|| fatal("in call to use_param_file(), parameter 'param_file_arg' got value of type that is not 'string'"))?
            .to_owned();
        let use_always = flag("use_param_file", "use_always", bound[1], false)?;
        let mut state = args_of(this).state.lock().unwrap();
        let format = state.format.unwrap_or(ParamFormat::Shell);
        state.param_file = Some(ParamFile {
            pattern,
            use_always,
            format,
        });
        Ok(this)
    }

    /// `args.set_param_file_format(format)`.
    fn set_param_file_format<'v>(this: Value<'v>, format: &str) -> starlark::Result<Value<'v>> {
        let parsed = match format {
            "shell" => ParamFormat::Shell,
            "multiline" => ParamFormat::Multiline,
            "flag_per_line" => ParamFormat::FlagPerLine,
            other => {
                return Err(fatal(format!(
                    "Invalid value for parameter \"format\": Allowed values are: \"shell\", \"multiline\", \"flag_per_line\", got \"{other}\""
                )));
            }
        };
        let mut state = args_of(this).state.lock().unwrap();
        state.format = Some(parsed);
        if let Some(file) = state.param_file.as_mut() {
            file.format = parsed;
        }
        Ok(this)
    }
}

/// An argument as a shell reads it back, quoted only when it needs it.
pub(crate) fn shell_quote(arg: &str) -> String {
    let plain = !arg.is_empty()
        && arg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./=:,+@%".contains(c));
    if plain {
        arg.to_owned()
    } else {
        format!("'{}'", arg.replace('\'', "'\\''"))
    }
}

/// The text of a parameter file for `items`.
pub(crate) fn param_file_contents(items: &[String], format: ParamFormat) -> String {
    let mut out = String::new();
    for item in items {
        match format {
            ParamFormat::Shell => {
                out.push_str(&shell_quote(item));
                out.push('\n');
            }
            ParamFormat::Multiline => {
                out.push_str(item);
                out.push('\n');
            }
            ParamFormat::FlagPerLine => {
                match item
                    .strip_prefix("--")
                    .and_then(|rest| rest.split_once('='))
                {
                    Some((flag, value)) => {
                        out.push_str(&format!("--{flag}\n{value}\n"));
                    }
                    None => {
                        out.push_str(item);
                        out.push('\n');
                    }
                }
            }
        }
    }
    out
}

/// The `Args` a value is, if it is one.
pub(crate) fn args_value<'v>(value: Value<'v>) -> Option<&'v ArgsValue> {
    value.downcast_ref::<ArgsValue>()
}
