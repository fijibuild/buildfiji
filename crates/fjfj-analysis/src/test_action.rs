//! The action that runs a test (buildfiji-fyz.9), as Bazel 9.2.0 registers it:
//! `external/bazel_tools/tools/test/test-setup.sh <test> <args...>` in the
//! execroot, with the environment `test-setup.sh` documents. The script
//! changes into the runfiles tree, runs the test, and leaves `test.log` and
//! `test.xml` in `bazel-out/<config>/testlogs/<package>/<name>/`.

use crate::target::{ConfiguredTarget, ConfiguredTargetKey};
use fjfj_engine::{Ctx, Error};
use fjfj_graph::rule::AttrValue;
use fjfj_graph::{Action, ActionKind, Artifact, Label, TestInfo};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// `size` to the seconds a test may take, and `timeout` to the same.
fn seconds(size: &str, timeout: &str) -> u64 {
    match timeout {
        "short" => 60,
        "moderate" => 300,
        "long" => 900,
        "eternal" => 3600,
        _ => match size {
            "small" => 60,
            "large" => 900,
            "enormous" => 3600,
            _ => 300,
        },
    }
}

/// Split a command line the way a shell reads words: spaces separate, quotes
/// group, a backslash escapes.
pub(crate) fn tokenize(text: &str) -> Result<Vec<String>, String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_word = false;
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        match c {
            c if c.is_whitespace() => {
                if in_word {
                    words.push(std::mem::take(&mut current));
                    in_word = false;
                }
            }
            '\'' => {
                in_word = true;
                loop {
                    match chars.next() {
                        Some('\'') => break,
                        Some(c) => current.push(c),
                        None => return Err("unterminated quotation".to_owned()),
                    }
                }
            }
            '"' => {
                in_word = true;
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('\\') => match chars.next() {
                            Some(c) => current.push(c),
                            None => return Err("unterminated quotation".to_owned()),
                        },
                        Some(c) => current.push(c),
                        None => return Err("unterminated quotation".to_owned()),
                    }
                }
            }
            '\\' => {
                in_word = true;
                match chars.next() {
                    Some(c) => current.push(c),
                    None => return Err("escape at end of line".to_owned()),
                }
            }
            c => {
                in_word = true;
                current.push(c);
            }
        }
    }
    if in_word {
        words.push(current);
    }
    Ok(words)
}

fn string_attr<'a>(attrs: &'a [(String, AttrValue)], name: &str) -> Option<&'a str> {
    attrs.iter().find_map(|(n, v)| match (n == name, v) {
        (true, AttrValue::String(s)) => Some(s.as_str()),
        _ => None,
    })
}

/// Register how to run `target`, a test with an executable.
pub(crate) async fn register(
    ctx: &Ctx,
    key: &ConfiguredTargetKey,
    attrs: &[(String, AttrValue)],
    target: &mut ConfiguredTarget,
) -> Result<(), Error> {
    let Some(exe) = target.executable.clone() else {
        return Err(Error::msg(format!(
            "{}: a test rule must give an executable",
            fjfj_graph::expand::label_text(&key.label)
        )));
    };
    let tool = |name: &str| ConfiguredTargetKey {
        label: Label {
            repo: "bazel_tools".into(),
            package: "tools/test".into(),
            name: name.into(),
        },
        configuration: key.configuration.clone(),
    };
    let setup = ctx.get(tool("test-setup.sh")).await?;
    let xml_tool = ctx.get(tool("generate-xml.sh")).await?;
    let tool_files: Vec<Artifact> = setup
        .files
        .to_vec()
        .into_iter()
        .chain(xml_tool.files.to_vec())
        .collect();
    let script = setup
        .files
        .to_vec()
        .first()
        .map(Artifact::exec_path)
        .ok_or_else(|| Error::msg("@bazel_tools has no test-setup.sh"))?;

    let label = &key.label;
    let size = string_attr(attrs, "size")
        .filter(|s| !s.is_empty())
        .unwrap_or("medium");
    let timeout = string_attr(attrs, "timeout").unwrap_or("");
    let seconds = seconds(size, timeout);

    // The test as the runfiles tree names it: `pkg/name`.
    let short = match exe.path.strip_prefix("external/") {
        Some(rest) => rest.split_once('/').map_or(rest, |(_, r)| r).to_owned(),
        None => exe.path.clone(),
    };
    let mut argv = vec![script, short.clone()];
    if let Some(AttrValue::StringList(args)) =
        attrs.iter().find(|(n, _)| n == "args").map(|(_, v)| v)
    {
        for arg in args {
            argv.extend(
                tokenize(arg).map_err(|e| Error::msg(format!("{}: in args: {e}", label.name)))?,
            );
        }
    }

    let testlogs = key.configuration.testlogs_dir();
    let dir = if label.package.is_empty() {
        format!("{testlogs}/{}", label.name)
    } else {
        format!("{testlogs}/{}/{}", label.package, label.name)
    };
    let root = fjfj_graph::Root::derived("");
    let file = |name: &str| Artifact {
        root: root.clone(),
        path: format!("{dir}/{name}"),
    };
    let (log, xml) = (file("test.log"), file("test.xml"));
    let tmp = {
        let hash = Sha256::digest(fjfj_graph::expand::label_text(label).as_bytes());
        format!("_tmp/{}", &hex::encode(hash)[..32])
    };
    let runfiles_dir = target
        .extra_outputs
        .first()
        .map(Artifact::exec_path)
        .unwrap_or_default();
    let mut env: BTreeMap<String, String> = BTreeMap::new();
    for (name, value) in [
        ("PATH", "/bin:/usr/bin:/usr/local/bin".to_owned()),
        ("TZ", "UTC".to_owned()),
        ("TEST_BINARY", short),
        (
            "TEST_INFRASTRUCTURE_FAILURE_FILE",
            format!("{dir}/test.infrastructure_failure"),
        ),
        (
            "TEST_LOGSPLITTER_OUTPUT_FILE",
            format!("{dir}/test.raw_splitlogs/test.splitlogs"),
        ),
        (
            "TEST_PREMATURE_EXIT_FILE",
            format!("{dir}/test.exited_prematurely"),
        ),
        ("TEST_SIZE", size.to_owned()),
        ("TEST_SRCDIR", runfiles_dir),
        ("TEST_TARGET", fjfj_graph::expand::label_text(label)),
        ("TEST_TIMEOUT", seconds.to_string()),
        ("TEST_TMPDIR", tmp),
        (
            "TEST_UNDECLARED_OUTPUTS_ANNOTATIONS_DIR",
            format!("{dir}/test.outputs_manifest"),
        ),
        ("TEST_UNDECLARED_OUTPUTS_DIR", format!("{dir}/test.outputs")),
        (
            "TEST_UNUSED_RUNFILES_LOG_FILE",
            format!("{dir}/test.unused_runfiles_log"),
        ),
        ("TEST_WARNINGS_OUTPUT_FILE", format!("{dir}/test.warnings")),
        ("TEST_WORKSPACE", "_main".to_owned()),
        ("XML_OUTPUT_FILE", xml.exec_path()),
    ] {
        env.insert(name.to_owned(), value);
    }
    env.extend(key.configuration.test_env.clone());
    argv.extend(key.configuration.test_args.iter().cloned());
    let mut inputs = tool_files;
    inputs.extend(target.extra_outputs.iter().take(1).cloned());
    let action = Action {
        owner: label.clone(),
        owner_kind: target.rule_class.clone().unwrap_or_default(),
        location: String::new(),
        configuration: key.configuration.mnemonic(),
        mnemonic: "TestRunner".to_owned(),
        progress_message: Some(format!("Testing {}", fjfj_graph::expand::label_text(label))),
        kind: ActionKind::Spawn {
            argv,
            env,
            execution_requirements: BTreeMap::from([("timeout".to_owned(), seconds.to_string())]),
        },
        inputs,
        outputs: vec![log.clone(), xml.clone()],
    };
    target.test = Some(TestInfo {
        action,
        log,
        xml,
        size: size.to_owned(),
        timeout_seconds: seconds,
    });
    target.deps.push(tool("test-setup.sh"));
    target.deps.push(tool("generate-xml.sh"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_split_like_a_shell_reads_them() {
        assert_eq!(
            tokenize("a1 'a 2' \"b c\" d\\ e").unwrap(),
            ["a1", "a 2", "b c", "d e"]
        );
        assert_eq!(tokenize("a 2").unwrap(), ["a", "2"]);
        assert!(tokenize("'open").is_err());
        assert_eq!(tokenize("").unwrap(), Vec::<String>::new());
        assert_eq!(tokenize("x ''").unwrap(), ["x", ""]);
    }

    #[test]
    fn size_and_timeout_give_the_seconds_bazel_gives() {
        assert_eq!(seconds("small", ""), 60);
        assert_eq!(seconds("medium", ""), 300);
        assert_eq!(seconds("large", ""), 900);
        assert_eq!(seconds("enormous", ""), 3600);
        assert_eq!(seconds("small", "long"), 900);
        assert_eq!(seconds("", "short"), 60);
    }
}
