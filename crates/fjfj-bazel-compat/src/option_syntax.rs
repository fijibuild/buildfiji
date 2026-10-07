//! What Bazel does with the shape of an option before any command reads it
//! (probed on 9.2.0):
//!
//! - a boolean may be written `--flag`, `--noflag` or `--flag=<value>` with
//!   `true`, `yes` or `1` (any case) or `false`, `no` or `0`; those with a
//!   value are rewritten here as `--flag` or `--noflag`;
//! - `--noflag=value` and `--noflag` of a flag that is no boolean are errors;
//! - a value that is no int, long, double or percentage says so, naming the
//!   option as it was written and the value, which for a flag given without
//!   `=` is the next word;
//! - `--define` and `--action_env` and the like need `name=value`.
//!
//! [`normalize`] does that to the words before `--` and returns what the
//! rest of the pipeline reads, or the error as Bazel words it (without
//! `ERROR: `).

use crate::flag_registry::FlagRegistry;

/// The value of a boolean option, if it is one.
fn boolean(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "yes" | "1" => Some(true),
        "false" | "no" | "0" => Some(false),
        _ => None,
    }
}

/// What is wrong with `value` for an option of the converter `converter`,
/// in Bazel's words.
fn wrong_value(converter: &str, name: &str, value: &str) -> Option<String> {
    let int = |what: &str| {
        value
            .parse::<i32>()
            .is_err()
            .then(|| format!("'{value}' is not {what}"))
    };
    match converter {
        "Integer" => int("an int"),
        "Long" => value
            .parse::<i64>()
            .is_err()
            .then(|| format!("'{value}' is not a long")),
        "Double" => value
            .parse::<f64>()
            .is_err()
            .then(|| format!("'{value}' is not a double")),
        "Percentage" => match value.parse::<i32>() {
            Err(_) => Some(format!("'{value}' is not an int")),
            Ok(n) if n < 0 => Some(format!("'{value}' should be >= 0")),
            Ok(n) if n > 100 => Some(format!("'{value}' should be <= 100")),
            Ok(_) => None,
        },
        "Assignment" if name == "define" => (!value
            .split_once('=')
            .is_some_and(|(k, v)| !k.is_empty() && !v.is_empty() && !v.contains('=')))
        .then(|| "Variable definitions must be in the form of a 'name=value' assignment".to_owned()),
        "EnvVars" => (value.is_empty() || value == "=").then(|| {
            "Variable definitions must be in the form of a 'name=value', 'name', or '=name' assignment"
                .to_owned()
        }),
        _ => None,
    }
}

/// `args` with each boolean written with a value rewritten, or the first
/// thing Bazel would refuse.
pub fn normalize(args: &[String], command: &str) -> Result<Vec<String>, String> {
    let registry = FlagRegistry::global();
    let mut out = Vec::with_capacity(args.len());
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        if arg == "--" {
            out.push(arg.clone());
            out.extend(iter.cloned());
            break;
        }
        let Some(body) = arg.strip_prefix("--").filter(|b| !b.starts_with('/')) else {
            out.push(arg.clone());
            continue;
        };
        let (name, value) = match body.split_once('=') {
            Some((name, value)) => (name, Some(value)),
            None => (body, None),
        };
        let known = |n: &str| registry.named(n).filter(|f| f.commands.contains(&command));
        match known(name) {
            Some(flag) => {
                let converter = flag.type_converter.unwrap_or("");
                let switch = matches!(converter, "Boolean" | "TriState");
                if switch {
                    match value {
                        None => out.push(arg.clone()),
                        Some(v) => match boolean(v) {
                            // Every boolean has its `no` form.
                            Some(on) if flag.has_negative_flag => out.push(if on {
                                format!("--{name}")
                            } else {
                                format!("--no{name}")
                            }),
                            Some(_) => out.push(arg.clone()),
                            None if converter == "TriState" && v.eq_ignore_ascii_case("auto") => {}
                            None => {
                                return Err(format!(
                                    "While parsing option {arg}: '{v}' is not a boolean"
                                ));
                            }
                        },
                    }
                    continue;
                }
                if !flag.requires_value && !flag.enum_values.is_empty() {
                    // A flag of a few values, which may be given none.
                    match (name, value) {
                        ("subcommands", Some(v)) => match v.to_ascii_lowercase().as_str() {
                            "true" => out.push("--subcommands".to_owned()),
                            "false" => out.push("--nosubcommands".to_owned()),
                            "pretty_print" => out.push("--subcommands=pretty_print".to_owned()),
                            _ => {
                                return Err(format!(
                                    "While parsing option {arg}: Not a valid subcommand option: '{v}' (should be true, pretty_print or false)"
                                ));
                            }
                        },
                        _ => out.push(arg.clone()),
                    }
                    continue;
                }
                if flag.requires_value {
                    // The value is the next word when it is not attached.
                    let (shown, text) = match value {
                        Some(v) => (arg.clone(), Some(v.to_owned())),
                        None => match iter.peek() {
                            Some(next) => (format!("--{name} {next}"), Some((*next).clone())),
                            None => (arg.clone(), None),
                        },
                    };
                    if let Some(text) = &text
                        && let Some(why) = wrong_value(converter, name, text)
                    {
                        return Err(format!("While parsing option {shown}: {why}"));
                    }
                }
                out.push(arg.clone());
                if flag.requires_value
                    && value.is_none()
                    && let Some(next) = iter.next()
                {
                    out.push(next.clone());
                }
            }
            None => {
                // `--noflag` of a flag that is no boolean.
                if let Some(base) = name.strip_prefix("no")
                    && let Some(flag) = known(base)
                    && !matches!(flag.type_converter, Some("Boolean" | "TriState"))
                    && flag.requires_value
                {
                    return Err(format!(
                        "{arg} :: Illegal use of 'no' prefix on non-boolean option: {arg}"
                    ));
                }
                // `--noflag=value` of a boolean.
                if let Some(base) = name.strip_prefix("no")
                    && value.is_some()
                    && known(base).is_some_and(|f| f.has_negative_flag)
                {
                    return Err(format!(
                        "{arg} :: Unexpected value after boolean option: {arg}"
                    ));
                }
                out.push(arg.clone());
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    fn one(words: &[&str]) -> Result<Vec<String>, String> {
        normalize(&args(words), "build")
    }

    #[test]
    fn a_boolean_with_a_value_becomes_the_flag_or_its_negation() {
        for (word, want) in [
            ("--keep_going=true", "--keep_going"),
            ("--keep_going=TRUE", "--keep_going"),
            ("--keep_going=yes", "--keep_going"),
            ("--keep_going=1", "--keep_going"),
            ("--keep_going=false", "--nokeep_going"),
            ("--keep_going=No", "--nokeep_going"),
            ("--keep_going=0", "--nokeep_going"),
            ("--keep_going", "--keep_going"),
            ("--nokeep_going", "--nokeep_going"),
        ] {
            assert_eq!(one(&[word, "//a:b"]).unwrap(), [want, "//a:b"], "{word}");
        }
        // A tri-state `auto` is the default: nothing to say.
        assert_eq!(
            one(&["--enable_runfiles=auto"]).unwrap(),
            Vec::<String>::new()
        );
    }

    /// Probed on Bazel 9.2.0.
    #[test]
    fn the_errors_say_what_bazel_says() {
        for (words, want) in [
            (
                &["--keep_going=maybe"][..],
                "While parsing option --keep_going=maybe: 'maybe' is not a boolean",
            ),
            (
                &["--keep_going="],
                "While parsing option --keep_going=: '' is not a boolean",
            ),
            (
                &["--stamp=auto"],
                "While parsing option --stamp=auto: 'auto' is not a boolean",
            ),
            (
                &["--nokeep_going=true"],
                "--nokeep_going=true :: Unexpected value after boolean option: --nokeep_going=true",
            ),
            (
                &["--nojobs"],
                "--nojobs :: Illegal use of 'no' prefix on non-boolean option: --nojobs",
            ),
            (
                &["--noaction_env"],
                "--noaction_env :: Illegal use of 'no' prefix on non-boolean option: --noaction_env",
            ),
            (
                &["--analysis_testing_deps_limit=abc"],
                "While parsing option --analysis_testing_deps_limit=abc: 'abc' is not an int",
            ),
            (
                &["--analysis_testing_deps_limit", "//a:b"],
                "While parsing option --analysis_testing_deps_limit //a:b: '//a:b' is not an int",
            ),
            (
                &["--analysis_testing_deps_limit=99999999999"],
                "While parsing option --analysis_testing_deps_limit=99999999999: '99999999999' is not an int",
            ),
            (
                &["--max_computation_steps=abc"],
                "While parsing option --max_computation_steps=abc: 'abc' is not a long",
            ),
            (
                &["--http_timeout_scaling=abc"],
                "While parsing option --http_timeout_scaling=abc: 'abc' is not a double",
            ),
            (
                &["--experimental_remote_failure_rate_threshold=200"],
                "While parsing option --experimental_remote_failure_rate_threshold=200: '200' should be <= 100",
            ),
            (
                &["--define=abc"],
                "While parsing option --define=abc: Variable definitions must be in the form of a 'name=value' assignment",
            ),
            (
                &["--action_env="],
                "While parsing option --action_env=: Variable definitions must be in the form of a 'name=value', 'name', or '=name' assignment",
            ),
        ] {
            assert_eq!(one(words).unwrap_err(), want, "{words:?}");
        }
    }

    #[test]
    fn what_follows_the_marker_and_what_is_not_an_option_is_left() {
        assert_eq!(
            one(&["//a:b", "--", "--keep_going=maybe"]).unwrap(),
            ["//a:b", "--", "--keep_going=maybe"]
        );
        assert_eq!(
            one(&["--define", "a=b", "--keep_going=1", "-k"]).unwrap(),
            ["--define", "a=b", "--keep_going", "-k"]
        );
        // An option of another command is left to the check that names it.
        assert_eq!(one(&["--bogus=maybe"]).unwrap(), ["--bogus=maybe"]);
    }
}
