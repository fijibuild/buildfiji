//! The flags only `test` reads (buildfiji-fyz.12): `--test_output`,
//! `--test_env`, `--test_arg`, and `--cache_test_results`.

use crate::flag_registry::FlagRegistry;

pub const IMPLEMENTED: &[&str] = &["test_output", "test_env", "test_arg", "cache_test_results"];

/// How much of a test's log `test` shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TestOutput {
    /// Only the summary line of each test.
    #[default]
    Summary,
    /// The log of each test that failed.
    Errors,
    /// The log of every test.
    All,
    /// Logs as they are written (shown as `All` here).
    Streamed,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestFlags {
    pub output: TestOutput,
    /// `NAME=value`, or `NAME` for the client's value.
    pub env: Vec<String>,
    pub args: Vec<String>,
    /// `Some(false)` for `--nocache_test_results`.
    pub cache_results: Option<bool>,
}

/// Pull [`TestFlags`] out of `args`, returning the rest in order.
pub fn extract(args: &[String], command: &str) -> Result<(TestFlags, Vec<String>), String> {
    let registry = FlagRegistry::global();
    let mut flags = TestFlags::default();
    let mut rest = Vec::with_capacity(args.len());
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if !arg.starts_with('-') {
            rest.push(arg.clone());
            continue;
        }
        let Ok(m) = registry.resolve(arg, command) else {
            rest.push(arg.clone());
            continue;
        };
        let name = m.flag.name;
        if !IMPLEMENTED.contains(&name) {
            rest.push(arg.clone());
            continue;
        }
        if name == "cache_test_results" {
            flags.cache_results = Some(!m.negated);
            continue;
        }
        let Some(value) = m.value.map(str::to_string).or_else(|| iter.next().cloned()) else {
            rest.push(arg.clone());
            continue;
        };
        match name {
            "test_output" => {
                flags.output = match value.as_str() {
                    "summary" => TestOutput::Summary,
                    "errors" => TestOutput::Errors,
                    "all" => TestOutput::All,
                    "streamed" => TestOutput::Streamed,
                    other => {
                        return Err(format!(
                            "While parsing option --test_output={other}: Not a valid test output: {other}"
                        ));
                    }
                }
            }
            "test_env" => flags.env.push(value),
            "test_arg" => flags.args.push(value),
            _ => rest.push(arg.clone()),
        }
    }
    Ok((flags, rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn the_test_flags_come_out_in_both_spellings() {
        let (flags, rest) = extract(
            &args(&[
                "--test_output=errors",
                "--test_env",
                "A=b",
                "--test_env=HOME",
                "--test_arg=-x",
                "--nocache_test_results",
                "//a",
            ]),
            "test",
        )
        .unwrap();
        assert_eq!(flags.output, TestOutput::Errors);
        assert_eq!(flags.env, ["A=b", "HOME"]);
        assert_eq!(flags.args, ["-x"]);
        assert_eq!(flags.cache_results, Some(false));
        assert_eq!(rest, ["//a"]);
        assert!(extract(&args(&["--test_output=loud"]), "test").is_err());
    }
}
