//! `Args`, what `ctx.actions.args()` builds (buildfiji-136.3).
//!
//! What each method adds was read off `bazel aquery` of a rule that used all
//! of them (Bazel 9.2.0). The arguments are expanded as they are added, not
//! when the action runs, so a `map_each` is called during analysis.

use super::file::artifact_of;
use crate::args::{Wording, bind, fatal, param};
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

/// An argument that is not a string, int, File or Label, as Bazel puts it on
/// the command line: its Java string, which is `str()` but for a bool, that is
/// `true` or `false`.
fn other_text(value: Value<'_>) -> String {
    match value.unpack_bool() {
        Some(b) => b.to_string(),
        None => value.to_str(),
    }
}

fn arg_text(value: Value<'_>) -> String {
    text_of(value).unwrap_or_else(|| other_text(value))
}

/// What `map_each` may return is a string, None or a list of strings.
fn mapped_wrong(value: Value<'_>) -> starlark::Error {
    let found = match crate::args::sequence(value)
        .and_then(|items| items.into_iter().find(|i| i.unpack_str().is_none()))
    {
        Some(bad) if value.get_type() == "list" => format!("list containing {}", bad.get_type()),
        _ => value.get_type().to_owned(),
    };
    fatal(format!(
        "Expected map_each to return string, None, or list of strings, found {found}"
    ))
}

/// Whether `pattern` has one `%s` (`%%` is a `%`), else Bazel's error naming
/// the parameter.
fn check_format(name: &str, pattern: &str) -> starlark::Result<()> {
    let mut holes = 0;
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, chars.peek()) {
            ('%', Some('s')) => {
                chars.next();
                holes += 1;
            }
            ('%', Some('%')) => {
                chars.next();
            }
            _ => {}
        }
    }
    if holes == 1 {
        Ok(())
    } else {
        Err(fatal(format!(
            "Invalid value for parameter \"{name}\": Expected string with a single \"%s\""
        )))
    }
}

/// `pattern` with `%s` replaced, already checked by [`check_format`]; `%%` is
/// a `%`.
fn formatted(pattern: &str, text: &str) -> String {
    let mut out = String::with_capacity(pattern.len() + text.len());
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, chars.peek()) {
            ('%', Some('s')) => {
                chars.next();
                out.push_str(text);
            }
            ('%', Some('%')) => {
                chars.next();
                out.push('%');
            }
            _ => out.push(c),
        }
    }
    out
}

/// A name given before values: a string, or Bazel's error.
fn arg_name(value: Option<Value<'_>>) -> starlark::Result<String> {
    let value = value.expect("bound");
    value.unpack_str().map(str::to_owned).ok_or_else(|| {
        fatal(format!(
            "expected value of type 'string' for arg name, got '{}'",
            value.get_type()
        ))
    })
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
    named: bool,
    map_each: Option<Value<'v>>,
    allow_closure: bool,
    format_each: Option<&str>,
    uniquify: bool,
    eval: &mut Evaluator<'v, '_, '_>,
) -> starlark::Result<Vec<String>> {
    let items: Vec<Value<'v>> = if is_depset(values) {
        depset_to_list(values).expect("a depset")?
    } else if let Some(items) = crate::args::sequence(values) {
        items
    } else {
        // Given after a name it is a parameter of its own, which Starlark
        // checks; alone it is the first parameter, which the method checks.
        return Err(fatal(if named {
            format!(
                "in call to {function}(), parameter 'values' got value of type '{}', want 'sequence or depset'",
                values.get_type()
            )
        } else {
            format!(
                "expected value of type 'sequence or depset' for values, got '{}'",
                values.get_type()
            )
        }));
    };
    let mut out: Vec<String> = Vec::new();
    if let Some(f) = map_each.filter(|f| !f.is_none())
        && !f.get_type().contains("function")
    {
        return Err(fatal(format!(
            "in call to {function}(), parameter 'map_each' got value of type '{}', want 'callable or NoneType'",
            f.get_type()
        )));
    }
    if let Some(f) = map_each.filter(|f| !f.is_none())
        && !allow_closure
        && !starlark::eval::is_global_definition(f)
        && let Some(span) = starlark::eval::definition_span(f)
    {
        let at = span.resolve();
        return Err(fatal(format!(
            "to avoid unintended retention of analysis data structures, the map_each function (declared at {}:{}:{}) must be declared by a top-level def statement",
            at.file,
            at.span.begin.line + 1,
            at.span.begin.column + 1
        )));
    }
    for item in items {
        let mut produced: Vec<String> = Vec::new();
        match map_each.filter(|f| !f.is_none()) {
            Some(f) => {
                let mapped = eval.eval_function(f, &[item], &[])?;
                if mapped.is_none() {
                    continue;
                }
                if let Some(text) = mapped.unpack_str() {
                    produced.push(text.to_owned());
                } else if mapped.get_type() == "list" {
                    for each in crate::args::sequence(mapped).unwrap_or_default() {
                        produced.push(
                            each.unpack_str()
                                .ok_or_else(|| mapped_wrong(mapped))?
                                .to_owned(),
                        );
                    }
                } else {
                    return Err(mapped_wrong(mapped));
                }
            }
            None => produced.push(arg_text(item)),
        }
        for text in produced {
            let text = match format_each {
                Some(pattern) => formatted(pattern, &text),
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
        if let Some(pattern) = &format {
            check_format("format", pattern)?;
        }
        // One value, which is not a collection: those are for `add_all`.
        let single = |value: Value<'v>| -> starlark::Result<String> {
            if let Some(text) = text_of(value) {
                return Ok(text);
            }
            if is_depset(value) || crate::args::sequence(value).is_some() {
                return Err(fatal(
                    "Args.add() doesn't accept vectorized arguments. Please use Args.add_all() or Args.add_joined() instead.",
                ));
            }
            Ok(other_text(value))
        };
        let mut state = args_of(this).state.lock().unwrap();
        match bound[1] {
            None => {
                let text = single(first)?;
                state.items.push(match &format {
                    Some(p) => formatted(p, &text),
                    None => text,
                });
            }
            Some(value) => {
                let name = arg_name(Some(first))?;
                state.items.push(name);
                let text = single(value)?;
                state.items.push(match &format {
                    Some(p) => formatted(p, &text),
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
            Some(values) => (Some(arg_name(bound[0])?), values),
            None => (None, bound[0].expect("required")),
        };
        let format_each = string_opt("add_all", "format_each", bound[3])?;
        if let Some(pattern) = &format_each {
            check_format("format_each", pattern)?;
        }
        let before_each = string_opt("add_all", "before_each", bound[4])?;
        let omit_if_empty = flag("add_all", "omit_if_empty", bound[5], true)?;
        let uniquify = flag("add_all", "uniquify", bound[6], false)?;
        flag("add_all", "expand_directories", bound[7], true)?;
        let terminate_with = string_opt("add_all", "terminate_with", bound[8])?;
        let allow_closure = flag("add_all", "allow_closure", bound[9], false)?;
        let items = expand(
            "add_all",
            values,
            name.is_some(),
            bound[2],
            allow_closure,
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
            Some(values) => (Some(arg_name(bound[0])?), values),
            None => (None, bound[0].expect("required")),
        };
        let join = bound[2].expect("required");
        let join_with = join
            .unpack_str()
            .ok_or_else(|| {
                fatal(format!(
                    "in call to add_joined(), parameter 'join_with' got value of type '{}', want 'string'",
                    join.get_type()
                ))
            })?
            .to_owned();
        let format_each = string_opt("add_joined", "format_each", bound[4])?;
        if let Some(pattern) = &format_each {
            check_format("format_each", pattern)?;
        }
        let format_joined = string_opt("add_joined", "format_joined", bound[5])?;
        if let Some(pattern) = &format_joined {
            check_format("format_joined", pattern)?;
        }
        let omit_if_empty = flag("add_joined", "omit_if_empty", bound[6], true)?;
        let uniquify = flag("add_joined", "uniquify", bound[7], false)?;
        flag("add_joined", "expand_directories", bound[8], true)?;
        let allow_closure = flag("add_joined", "allow_closure", bound[9], false)?;
        let items = expand(
            "add_joined",
            values,
            name.is_some(),
            bound[3],
            allow_closure,
            format_each.as_deref(),
            uniquify,
            eval,
        )?;
        if items.is_empty() && omit_if_empty {
            return Ok(this);
        }
        let mut joined = items.join(&join_with);
        if let Some(pattern) = format_joined {
            joined = formatted(&pattern, &joined);
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
        let given = bound[0].expect("required");
        let pattern = given
            .unpack_str()
            .ok_or_else(|| {
                fatal(format!(
                    "in call to use_param_file(), parameter 'param_file_arg' got value of type '{}', want 'string'",
                    given.get_type()
                ))
            })?
            .to_owned();
        if check_format("param_file_arg", &pattern).is_err() {
            return Err(fatal(format!(
                "Invalid value for parameter \"param_file_arg\": Expected string with a single \"%s\", got \"{pattern}\""
            )));
        }
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
                let _ = other;
                return Err(fatal(
                    "Invalid value for parameter \"format\": Expected one of \"shell\", \"multiline\", \"flag_per_line\"",
                ));
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
            .all(|c| c.is_ascii_alphanumeric() || "-_./:,+@%".contains(c));
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
