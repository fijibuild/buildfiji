//! `fjfj clean`: what Bazel's `clean` does to the output tree.
//!
//! The execroot, the action cache and the convenience links go;
//! `--expunge` takes the whole output base. With `--async` (or
//! `--expunge_async`) the tree is renamed and a process of its own deletes
//! it, and the command says where it went. Probed on Bazel 9.2.0.

use std::io::Write;

use fjfj_bazel_compat::{QueryArgs, build_flags, clap_flags};
use fjfj_exec::execroot::{Cleaned, Layout};

use crate::{CliError, fetch_command};

/// The flags `clean` reads besides the build flags.
const IMPLEMENTED: &[&str] = &["expunge", "async", "expunge_async"];

/// What the flags ask for, the last of each winning.
#[derive(Debug, Default, PartialEq, Eq)]
struct Flags {
    expunge: bool,
    asynchronous: bool,
}

fn extract(args: &[String]) -> (Flags, Vec<String>) {
    let mut flags = Flags::default();
    let mut rest = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--expunge" => flags.expunge = true,
            "--noexpunge" => flags.expunge = false,
            "--async" => flags.asynchronous = true,
            "--noasync" => flags.asynchronous = false,
            "--expunge_async" => {
                flags.expunge = true;
                flags.asynchronous = true;
            }
            other => rest.push(other.to_owned()),
        }
    }
    (flags, rest)
}

/// A name that tells two cleans apart, shaped like the UUID Bazel uses.
fn unique() -> String {
    use sha2::Digest;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let id = std::process::id();
    let hex = hex::encode(sha2::Sha256::digest(format!("{now}-{id}")));
    format!(
        "{id}_{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// What `clean` says, given what it did.
fn say(out: &mut dyn Write, flags: &Flags, cleaned: &Cleaned) -> std::io::Result<()> {
    match cleaned {
        Cleaned::Moved(to) => {
            writeln!(out, "INFO: Starting clean.")?;
            writeln!(
                out,
                "INFO: {} moved to {} for deletion",
                if flags.expunge {
                    "Output base"
                } else {
                    "Output tree"
                },
                to.display()
            )
        }
        Cleaned::Removed => Ok(()),
    }
}

pub(crate) async fn run(args: QueryArgs) -> Result<(), CliError> {
    let with_rc: Vec<String> = crate::rc_flags("clean")?
        .into_iter()
        .chain(args.expr.iter().cloned())
        .collect();
    let rest = crate::drop_build_family_flags(with_rc, "clean")?;
    let (fetch, rest) = fetch_command::extract(&rest)?;
    let mut implemented: Vec<&'static str> = build_flags::IMPLEMENTED.to_vec();
    implemented.extend_from_slice(IMPLEMENTED);
    implemented.extend_from_slice(fetch_command::IMPLEMENTED);
    clap_flags::validate(&rest, "clean", &implemented)
        .map_err(|e| CliError::CommandLine(anyhow::Error::from(e)))?;
    let (flags, rest) = extract(&rest);
    let (build, rest) = build_flags::extract(&rest, "clean");
    let _ = (fetch, rest);
    let workspace = crate::locate_workspace_root("clean")?;
    let layout = Layout {
        output_base: fetch_command::default_output_base(&workspace),
        workspace,
    };
    let prefix = build
        .symlink_prefix
        .clone()
        .unwrap_or_else(|| "bazel-".to_owned());
    let mut err = std::io::stderr();
    if !flags.asynchronous {
        let _ = writeln!(
            err,
            "INFO: Starting clean (this may take a while). Use --async if the clean takes more than several minutes."
        );
    }
    let cleaned = layout
        .clean(&prefix, flags.expunge, flags.asynchronous, &unique())
        .map_err(|e| CliError::Build(anyhow::anyhow!("clean failed: {e}")))?;
    let _ = say(&mut err, &flags, &cleaned);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn the_flags_come_out_and_the_last_of_each_wins() {
        let (flags, rest) = extract(&args(&["--expunge", "--noexpunge", "--async", "x"]));
        assert_eq!(
            flags,
            Flags {
                expunge: false,
                asynchronous: true
            }
        );
        assert_eq!(rest, ["x"]);
        let (flags, _) = extract(&args(&["--expunge_async"]));
        assert_eq!(
            flags,
            Flags {
                expunge: true,
                asynchronous: true
            }
        );
    }

    /// Probed on Bazel 9.2.0: what a clean leaves of an output base, and the
    /// two lines an asynchronous one says.
    #[test]
    fn a_clean_removes_the_execroot_the_cache_and_the_links() {
        let dir = tempfile::tempdir().unwrap();
        let workspace = dir.path().join("ws");
        let ob = dir.path().join("ob");
        std::fs::create_dir_all(&workspace).unwrap();
        let layout = Layout {
            workspace: workspace.clone(),
            output_base: ob.clone(),
        };
        let make = || {
            std::fs::create_dir_all(layout.bazel_out().join("k8-fastbuild/bin")).unwrap();
            std::fs::create_dir_all(ob.join("external/r")).unwrap();
            std::fs::write(ob.join("fjfj-action-cache.json"), "{}").unwrap();
            layout.convenience_links("bazel-", "k8-fastbuild").unwrap();
        };
        make();
        assert!(workspace.join("bazel-bin").exists());
        assert_eq!(
            layout.clean("bazel-", false, false, "x").unwrap(),
            Cleaned::Removed
        );
        assert!(!layout.execroot().exists());
        assert!(ob.join("execroot").is_dir());
        assert!(ob.join("external/r").is_dir());
        assert!(!ob.join("fjfj-action-cache.json").exists());
        assert!(std::fs::symlink_metadata(workspace.join("bazel-bin")).is_err());
        assert!(std::fs::symlink_metadata(workspace.join("bazel-out")).is_err());

        make();
        let moved = layout.clean("bazel-", false, true, "7_u").unwrap();
        assert_eq!(moved, Cleaned::Moved(ob.join("execroot_tmp_7_u")));
        let mut said = Vec::new();
        say(&mut said, &Flags::default(), &moved).unwrap();
        assert_eq!(
            String::from_utf8(said).unwrap(),
            format!(
                "INFO: Starting clean.\nINFO: Output tree moved to {} for deletion\n",
                ob.join("execroot_tmp_7_u").display()
            )
        );
        assert!(!ob.join("execroot").exists());

        make();
        assert_eq!(
            layout.clean("bazel-", true, false, "x").unwrap(),
            Cleaned::Removed
        );
        assert!(!ob.exists());
    }
}
