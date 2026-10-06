//! `fjfj info`: what Bazel's `info` says about the workspace and its output
//! tree, with the same keys.
//!
//! No keys lists every key as `key: value` in byte order; one key prints its
//! value alone; several print `key: value` in the order asked, and an unknown
//! one is said after the rest, exit 2. `--show_make_env` adds the variables
//! of the configuration (`BINDIR`, `COMPILATION_MODE`, `GENDIR`,
//! `TARGET_CPU`). Bazel's server keys (`server_pid`, the JVM's) are not here,
//! as fjfj has no server process yet.

use std::collections::BTreeMap;
use std::path::PathBuf;

use fjfj_bazel_compat::{QueryArgs, build_flags, clap_flags};

use crate::{CliError, fetch_command};

/// The flags `info` reads besides the build flags.
const IMPLEMENTED: &[&str] = &["show_make_env", "noshow_make_env", "package_path"];

/// What Bazel prints for `info starlark-semantics`; not in the full list.
const STARLARK_SEMANTICS: &str = "StarlarkSemantics{internal_bazel_only_utf_8_byte_strings=true}";

pub(crate) async fn run(args: QueryArgs) -> Result<(), CliError> {
    let bad = |message: String| CliError::CommandLine(anyhow::anyhow!(message));
    let with_rc: Vec<String> = crate::rc_flags("info")?
        .into_iter()
        .chain(args.expr.iter().cloned())
        .collect();
    let rest = crate::drop_build_family_flags(with_rc, "info")?;
    let (fetch, rest) = fetch_command::extract(&rest)?;
    let mut implemented: Vec<&'static str> = build_flags::IMPLEMENTED.to_vec();
    implemented.extend_from_slice(IMPLEMENTED);
    implemented.extend_from_slice(fetch_command::IMPLEMENTED);
    clap_flags::validate(&rest, "info", &implemented)
        .map_err(|e| CliError::CommandLine(anyhow::Error::from(e)))?;
    let (build, rest) = build_flags::extract(&rest, "info");
    let mut show_make_env = false;
    let mut package_path = "%workspace%".to_owned();
    let mut keys = Vec::new();
    for arg in &rest {
        match arg.as_str() {
            "--show_make_env" => show_make_env = true,
            "--noshow_make_env" => show_make_env = false,
            other => match other.strip_prefix("--package_path=") {
                Some(value) => package_path = value.to_owned(),
                None => keys.push(other.to_owned()),
            },
        }
    }
    let configuration = crate::build_command::configuration_from(&build).map_err(bad)?;
    let workspace = crate::locate_workspace_root("info")?;
    let output_base = fetch_command::default_output_base(&workspace);
    let layout = fjfj_exec::execroot::Layout {
        workspace: workspace.clone(),
        output_base: output_base.clone(),
    };
    let mnemonic = configuration.mnemonic();
    let out = layout.bazel_out();
    let show = |p: PathBuf| p.display().to_string();
    let bin = show(out.join(&mnemonic).join("bin"));
    let mut values: BTreeMap<&str, String> = BTreeMap::from([
        ("bazel-bin", bin.clone()),
        ("bazel-genfiles", bin),
        ("bazel-testlogs", show(out.join(&mnemonic).join("testlogs"))),
        ("execution_root", show(layout.execroot())),
        ("output_base", show(output_base)),
        ("output_path", show(out.clone())),
        ("package_path", package_path),
        (
            "release",
            format!("release {}", fjfj_bazel_compat::bazel_flags::BAZEL_VERSION),
        ),
        (
            "repository_cache",
            match fetch.repository_cache.clone() {
                None => show(fetch_command::default_repository_cache()),
                Some(cache) => cache.map(show).unwrap_or_default(),
            },
        ),
        ("workspace", show(workspace)),
    ]);
    let mut listed = values.clone();
    if show_make_env {
        let gen_dir = format!("bazel-out/{mnemonic}/bin");
        for (key, value) in [
            ("BINDIR", gen_dir.clone()),
            (
                "COMPILATION_MODE",
                configuration.compilation_mode.name().to_owned(),
            ),
            ("GENDIR", gen_dir),
            ("TARGET_CPU", std::env::consts::ARCH.to_owned()),
        ] {
            values.insert(key, value.clone());
            listed.insert(key, value);
        }
    }
    values.insert("starlark-semantics", STARLARK_SEMANTICS.to_owned());
    let mut text = String::new();
    let mut unknown = Vec::new();
    match keys.as_slice() {
        [] => {
            for (key, value) in &listed {
                text.push_str(&format!("{key}: {value}\n"));
            }
        }
        [key] => match values.get(key.as_str()) {
            Some(value) => text.push_str(&format!("{value}\n")),
            None => unknown.push(key.as_str()),
        },
        keys => {
            for key in keys {
                match values.get(key.as_str()) {
                    Some(value) => text.push_str(&format!("{key}: {value}\n")),
                    None => unknown.push(key.as_str()),
                }
            }
        }
    }
    print!("{text}");
    if unknown.is_empty() {
        return Ok(());
    }
    Err(bad(format!("unknown key(s): '{}'", unknown.join(" "))))
}
