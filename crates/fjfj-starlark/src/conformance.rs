//! Bazel's Starlark script tests (buildfiji-mum.10), run the way Bazel's
//! `ScriptTest.java` runs them (see `testdata/bazel-starlark/README.md`).
//!
//! A chunk of a file is a program. It fails if it does not parse, if it
//! raises an error that no `### regexp` comment of the chunk matches, if a
//! comment's regexp matches no error, or if `assert_`, `assert_eq` or
//! `assert_fails` says so. The chunks that fail here are listed in
//! `testdata/conformance_known.txt` by file and first line, each a difference from
//! Bazel that is owned by a bead; the test fails when a chunk starts to pass
//! (to be removed from the list) or one that passed starts to fail.

use crate::native::module_globals;
use crate::set::set_globals;
use crate::structs::struct_globals;
use num_bigint::BigInt;
use regex::Regex;
use starlark::PrintHandler;
use starlark::environment::{Globals, GlobalsBuilder, Module};
use starlark::eval::Evaluator;
use starlark::starlark_module;
use starlark::syntax::{AstModule, Dialect, DialectTypes};
use starlark::values::none::NoneType;
use starlark::values::{ProvidesStaticType, Value};
use std::cell::RefCell;
use std::collections::BTreeSet;

const FILES: &[(&str, &str)] = &[
    (
        "all_any.star",
        include_str!("../testdata/bazel-starlark/all_any.star"),
    ),
    (
        "and_or_not.star",
        include_str!("../testdata/bazel-starlark/and_or_not.star"),
    ),
    (
        "assign.star",
        include_str!("../testdata/bazel-starlark/assign.star"),
    ),
    (
        "bench_call.star",
        include_str!("../testdata/bazel-starlark/bench_call.star"),
    ),
    (
        "bench_dict.star",
        include_str!("../testdata/bazel-starlark/bench_dict.star"),
    ),
    (
        "bench_int.star",
        include_str!("../testdata/bazel-starlark/bench_int.star"),
    ),
    (
        "bench_list.star",
        include_str!("../testdata/bazel-starlark/bench_list.star"),
    ),
    (
        "bench_sorted.star",
        include_str!("../testdata/bazel-starlark/bench_sorted.star"),
    ),
    (
        "bench_string.star",
        include_str!("../testdata/bazel-starlark/bench_string.star"),
    ),
    (
        "comprehension.star",
        include_str!("../testdata/bazel-starlark/comprehension.star"),
    ),
    (
        "cycles.star",
        include_str!("../testdata/bazel-starlark/cycles.star"),
    ),
    (
        "dict.star",
        include_str!("../testdata/bazel-starlark/dict.star"),
    ),
    (
        "equality.star",
        include_str!("../testdata/bazel-starlark/equality.star"),
    ),
    (
        "fields.star",
        include_str!("../testdata/bazel-starlark/fields.star"),
    ),
    (
        "float.star",
        include_str!("../testdata/bazel-starlark/float.star"),
    ),
    (
        "function.star",
        include_str!("../testdata/bazel-starlark/function.star"),
    ),
    (
        "int.star",
        include_str!("../testdata/bazel-starlark/int.star"),
    ),
    (
        "int_constructor.star",
        include_str!("../testdata/bazel-starlark/int_constructor.star"),
    ),
    (
        "json.star",
        include_str!("../testdata/bazel-starlark/json.star"),
    ),
    (
        "list.star",
        include_str!("../testdata/bazel-starlark/list.star"),
    ),
    (
        "list_mutation.star",
        include_str!("../testdata/bazel-starlark/list_mutation.star"),
    ),
    (
        "list_slices.star",
        include_str!("../testdata/bazel-starlark/list_slices.star"),
    ),
    (
        "loop.star",
        include_str!("../testdata/bazel-starlark/loop.star"),
    ),
    (
        "min_max.star",
        include_str!("../testdata/bazel-starlark/min_max.star"),
    ),
    (
        "range.star",
        include_str!("../testdata/bazel-starlark/range.star"),
    ),
    (
        "reversed.star",
        include_str!("../testdata/bazel-starlark/reversed.star"),
    ),
    (
        "set.star",
        include_str!("../testdata/bazel-starlark/set.star"),
    ),
    (
        "sorted.star",
        include_str!("../testdata/bazel-starlark/sorted.star"),
    ),
    (
        "string_elems.star",
        include_str!("../testdata/bazel-starlark/string_elems.star"),
    ),
    (
        "string_find.star",
        include_str!("../testdata/bazel-starlark/string_find.star"),
    ),
    (
        "string_format.star",
        include_str!("../testdata/bazel-starlark/string_format.star"),
    ),
    (
        "string_misc.star",
        include_str!("../testdata/bazel-starlark/string_misc.star"),
    ),
    (
        "string_partition.star",
        include_str!("../testdata/bazel-starlark/string_partition.star"),
    ),
    (
        "string_slice_index.star",
        include_str!("../testdata/bazel-starlark/string_slice_index.star"),
    ),
    (
        "string_split.star",
        include_str!("../testdata/bazel-starlark/string_split.star"),
    ),
    (
        "string_splitlines.star",
        include_str!("../testdata/bazel-starlark/string_splitlines.star"),
    ),
    (
        "string_test_characters.star",
        include_str!("../testdata/bazel-starlark/string_test_characters.star"),
    ),
    (
        "tuple.star",
        include_str!("../testdata/bazel-starlark/tuple.star"),
    ),
];

const KNOWN: &str = include_str!("../testdata/conformance_known.txt");

/// Chunks that are not run.
const NOT_RUN: &[&str] = &[];

/// What the asserts report: one line for each failure.
#[derive(ProvidesStaticType, Default)]
struct Reporter(RefCell<Vec<String>>);

fn report(eval: &Evaluator<'_, '_, '_>, message: String) {
    if let Some(reporter) = eval.extra.and_then(|e| e.downcast_ref::<Reporter>()) {
        reporter.0.borrow_mut().push(message);
    }
}

/// An error's message, without the location and stack.
fn message(error: &starlark::Error) -> String {
    error.kind().to_string()
}

#[starlark_module]
fn script_functions(builder: &mut GlobalsBuilder) {
    fn assert_<'v>(
        cond: Value<'v>,
        #[starlark(default = "assertion failed")] msg: &str,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        if !cond.to_bool() {
            report(eval, format!("assert_: {msg}"));
        }
        Ok(NoneType)
    }

    fn assert_eq<'v>(
        x: Value<'v>,
        y: Value<'v>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        if !x.equals(y)? {
            report(
                eval,
                format!("assert_eq: {} != {}", x.to_repr(), y.to_repr()),
            );
        }
        Ok(NoneType)
    }

    fn assert_fails<'v>(
        f: Value<'v>,
        want: &str,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<NoneType> {
        let pattern = Regex::new(want)
            .map_err(|_| starlark::Error::new_other(anyhow::anyhow!("invalid regexp: {want}")))?;
        match eval.eval_function(f, &[], &[]) {
            Ok(_) => report(
                eval,
                format!("evaluation succeeded unexpectedly (want error matching {want})"),
            ),
            Err(e) => {
                let got = message(&e);
                if !pattern.is_match(&got) {
                    report(
                        eval,
                        format!("regular expression ({want}) did not match error ({got})"),
                    );
                }
            }
        }
        Ok(NoneType)
    }

    /// Shallow-freezing is not something the crate's values can be asked for.
    fn freeze<'v>(#[starlark(default = NoneType)] _x: Value<'v>) -> starlark::Result<NoneType> {
        Ok(NoneType)
    }

    fn int_mul_slow<'v>(
        x: Value<'v>,
        y: Value<'v>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> starlark::Result<Value<'v>> {
        let parse = |v: Value<'v>| {
            BigInt::parse_bytes(v.to_str().as_bytes(), 10).ok_or_else(|| {
                starlark::Error::new_other(anyhow::anyhow!("not an int: {}", v.to_repr()))
            })
        };
        Ok(eval.heap().alloc(parse(x)? * parse(y)?))
    }
}

fn globals() -> Globals {
    let mut builder =
        GlobalsBuilder::extended_by(&[starlark::environment::LibraryExtension::Print]);
    builder.set("_utf8_byte_strings", false);
    builder
        .with(script_functions)
        .with(struct_globals)
        .with(module_globals)
        .with(set_globals)
        .build()
}

/// The dialect of the script tests: the language as Bazel's interpreter reads
/// it, with no file kind's restrictions.
fn dialect() -> Dialect {
    Dialect {
        enable_def: true,
        enable_lambda: true,
        enable_load: true,
        enable_keyword_only_arguments: true,
        enable_positional_only_arguments: false,
        enable_types: DialectTypes::Disable,
        enable_top_level_stmt: true,
        enable_f_strings: false,
        ..Dialect::Standard
    }
}

struct Quiet;
impl PrintHandler for Quiet {
    fn println(&self, _text: &str) -> starlark::Result<()> {
        Ok(())
    }
}

/// Run one chunk, which starts at line `first` of `file`; returns what is
/// wrong with it, one line each.
fn run_chunk(file: &str, first: usize, chunk: &str) -> Vec<String> {
    let mut problems = Vec::new();
    // `### regexp` comments.
    let mut expectations: Vec<(usize, Regex)> = Vec::new();
    for (n, line) in chunk.split('\n').enumerate() {
        if let Some(at) = line.find("###") {
            match Regex::new(line[at + 3..].trim()) {
                Ok(re) => expectations.push((first + n, re)),
                Err(e) => problems.push(format!("{file}:{}: invalid regexp: {e}", first + n)),
            }
        }
    }
    let padded = format!("{}{chunk}", "\n".repeat(first - 1));
    let ast = match AstModule::parse(file, padded, &dialect()) {
        Ok(ast) => ast,
        Err(e) => {
            problems.push(format!("{file}:{first}: {}", message(&e)));
            return problems;
        }
    };
    let reporter = Reporter::default();
    let globals = globals();
    let outcome = Module::with_temp_heap(|module| {
        let mut eval = Evaluator::new(&module);
        eval.extra = Some(&reporter);
        eval.set_print_handler(&Quiet);
        eval.eval_module(ast, &globals).map(|_| ())
    });
    problems.extend(
        reporter
            .0
            .into_inner()
            .into_iter()
            .map(|m| format!("{file}:{first}: {m}")),
    );
    if let Err(e) = outcome {
        let text = message(&e);
        match expectations.iter().position(|(_, re)| re.is_match(&text)) {
            Some(at) => {
                expectations.remove(at);
            }
            None => problems.push(format!("{file}:{first}: {text}")),
        }
    }
    for (line, re) in expectations {
        problems.push(format!("{file}:{line}: unmatched expectation: {re}"));
    }
    problems
}

/// Every chunk that fails, as `file:first line`, with why.
fn run_all() -> Vec<(String, String)> {
    let mut failed = Vec::new();
    for (name, content) in FILES {
        let mut line = 1;
        for chunk in content.split("\n---\n") {
            if NOT_RUN.contains(&format!("{name}:{line}").as_str()) {
                failed.push((format!("{name}:{line}"), "not run".to_owned()));
                line += chunk.matches('\n').count() + 2;
                continue;
            }
            let started = std::time::Instant::now();
            // A crate that panics on a chunk is a failure of that chunk.
            let problems = std::panic::catch_unwind(|| run_chunk(name, line, chunk))
                .unwrap_or_else(|panic| {
                    let why = panic
                        .downcast_ref::<String>()
                        .map(String::as_str)
                        .or_else(|| panic.downcast_ref::<&str>().copied())
                        .unwrap_or("panic");
                    vec![format!("{name}:{line}: panic: {why}")]
                });
            if started.elapsed().as_secs() >= 1 {
                eprintln!("SLOW {name}:{line} {:?}", started.elapsed());
            }
            if !problems.is_empty() {
                failed.push((format!("{name}:{line}"), problems.join("\n")));
            }
            line += chunk.matches('\n').count() + 2;
        }
    }
    failed
}

#[test]
fn the_script_tests_fail_where_the_known_list_says() {
    let failed = run_all();
    if std::env::var_os("FJFJ_CONFORMANCE_PRINT").is_some() {
        for (key, why) in &failed {
            eprintln!("KEY {key}\n{why}\n");
        }
    }
    let got: BTreeSet<&str> = failed.iter().map(|(k, _)| k.as_str()).collect();
    let known: BTreeSet<&str> = KNOWN
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    let new: Vec<&&str> = got.difference(&known).collect();
    let fixed: Vec<&&str> = known.difference(&got).collect();
    assert!(
        new.is_empty() && fixed.is_empty(),
        "newly failing: {new:?}\nnow passing (remove from conformance_known.txt): {fixed:?}"
    );
}
