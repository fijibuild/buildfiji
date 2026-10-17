//! The Xcode half of a command's environment (buildfiji-4sm9).
//!
//! An action that needs Xcode names the one it wants, and the SDK, in
//! `XCODE_VERSION_OVERRIDE`, `APPLE_SDK_VERSION_OVERRIDE` and
//! `APPLE_SDK_PLATFORM` (`apple_common.apple_host_system_env` and
//! `target_apple_env`). Just before it runs, Bazel turns those into the
//! variables the Apple tools read: `DEVELOPER_DIR`, the directory of that
//! Xcode, found with `xcode-locator`, and `SDKROOT`, the SDK's directory, found
//! with `xcrun`. The names stay in the environment, as Bazel leaves them.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;

const XCODE_VERSION: &str = "XCODE_VERSION_OVERRIDE";
const SDK_VERSION: &str = "APPLE_SDK_VERSION_OVERRIDE";
const SDK_PLATFORM: &str = "APPLE_SDK_PLATFORM";

/// `env` with `DEVELOPER_DIR` and `SDKROOT` added when it asks for an Xcode and
/// an SDK. Only macOS has either, so elsewhere it is returned as it is.
pub(crate) fn local_env(
    env: &BTreeMap<String, String>,
    output_base: &Path,
) -> Result<BTreeMap<String, String>, String> {
    if !cfg!(target_os = "macos") {
        return Ok(env.clone());
    }
    rewrite(
        env,
        &|version| developer_dir(output_base, version),
        &|developer_dir, sdk| sdk_root(developer_dir, sdk),
    )
}

/// Finds the SDK's directory from an Xcode directory (if any) and an SDK name.
type SdkRootLookup = dyn Fn(Option<&str>, &str) -> Result<String, String>;

/// The rewrite itself, with the two lookups given.
fn rewrite(
    env: &BTreeMap<String, String>,
    developer_dir: &dyn Fn(&str) -> Result<String, String>,
    sdk_root: &SdkRootLookup,
) -> Result<BTreeMap<String, String>, String> {
    let (xcode, sdk_version) = (env.get(XCODE_VERSION), env.get(SDK_VERSION));
    if xcode.is_none() && sdk_version.is_none() {
        return Ok(env.clone());
    }
    let mut out = env.clone();
    let mut dir = None;
    if let Some(version) = xcode {
        let found = developer_dir(version)?;
        out.insert("DEVELOPER_DIR".to_owned(), found.clone());
        dir = Some(found);
    }
    if let (Some(version), Some(platform)) = (sdk_version, env.get(SDK_PLATFORM)) {
        let sdk = format!("{}{version}", platform.to_lowercase());
        out.insert("SDKROOT".to_owned(), sdk_root(dir.as_deref(), &sdk)?);
    }
    Ok(out)
}

/// The directory of the Xcode with this full version, by `xcode-locator`
/// (built from `bazel_tools`' source the first time), remembered.
fn developer_dir(output_base: &Path, version: &str) -> Result<String, String> {
    static FOUND: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());
    let mut found = FOUND.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(dir) = found.get(version) {
        return Ok(dir.clone());
    }
    let locator = output_base.join("bin").join("xcode-locator");
    if !locator.is_file() {
        build_locator(output_base, &locator)?;
    }
    let output = Command::new(&locator)
        .arg(version)
        .output()
        .map_err(|e| format!("cannot run {}: {e}", locator.display()))?;
    let dir = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !output.status.success() || dir.is_empty() {
        return Err(format!(
            "Xcode {version} was not found by xcode-locator: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    found.insert(version.to_owned(), dir.clone());
    Ok(dir)
}

/// Compiles `xcode_locator.m`, as `@bazel_tools`' `xcode_configure` does.
fn build_locator(output_base: &Path, locator: &Path) -> Result<(), String> {
    let source = output_base.join("external/bazel_tools/tools/osx/xcode_locator.m");
    let bin = locator.parent().unwrap_or(output_base);
    std::fs::create_dir_all(bin).map_err(|e| format!("cannot make {}: {e}", bin.display()))?;
    // Built beside the result and moved into place, so a half-written file is
    // never taken for the tool.
    let partial = bin.join(format!("xcode-locator.{}", std::process::id()));
    let output = Command::new("env")
        .args(["-i"])
        .arg(format!(
            "DEVELOPER_DIR={}",
            std::env::var("DEVELOPER_DIR").unwrap_or_default()
        ))
        .args([
            "xcrun",
            "--sdk",
            "macosx",
            "clang",
            "-mmacosx-version-min=10.13",
        ])
        .args([
            "-fobjc-arc",
            "-framework",
            "CoreServices",
            "-framework",
            "Foundation",
        ])
        .arg("-o")
        .arg(&partial)
        .arg(&source)
        .output()
        .map_err(|e| format!("cannot run xcrun: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cannot build xcode-locator from {}: {}",
            source.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    std::fs::rename(&partial, locator)
        .map_err(|e| format!("cannot put xcode-locator in place: {e}"))
}

/// The directory of the SDK named like `macosx27.0`, by `xcrun`, in the given
/// Xcode.
fn sdk_root(developer_dir: Option<&str>, sdk: &str) -> Result<String, String> {
    let mut command = Command::new("/usr/bin/xcrun");
    command.args(["--sdk", sdk, "--show-sdk-path"]);
    if let Some(dir) = developer_dir {
        command.env("DEVELOPER_DIR", dir);
    }
    let output = command
        .output()
        .map_err(|e| format!("cannot run xcrun: {e}"))?;
    let path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !output.status.success() || path.is_empty() {
        return Err(format!(
            "xcrun cannot find the SDK {sdk}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    fn no_dir(_: &str) -> Result<String, String> {
        panic!("the Xcode directory was not asked for");
    }

    fn no_sdk(_: Option<&str>, _: &str) -> Result<String, String> {
        panic!("the SDK was not asked for");
    }

    /// Probed on Bazel 9.2.0 with an action whose environment was the three
    /// names: it ran with `DEVELOPER_DIR` and `SDKROOT` added and the names kept.
    #[test]
    fn xcode_names_become_a_developer_dir_and_an_sdk_root() {
        let given = env(&[
            ("APPLE_SDK_PLATFORM", "MacOSX"),
            ("APPLE_SDK_VERSION_OVERRIDE", "27.0"),
            ("PATH", "/bin"),
            ("XCODE_VERSION_OVERRIDE", "27.1.0.27A9275"),
        ]);
        let got = rewrite(
            &given,
            &|version| {
                assert_eq!(version, "27.1.0.27A9275");
                Ok("/X/Contents/Developer".to_owned())
            },
            &|dir, sdk| {
                assert_eq!(dir, Some("/X/Contents/Developer"));
                assert_eq!(sdk, "macosx27.0");
                Ok("/X/Contents/Developer/SDKs/MacOSX27.0.sdk".to_owned())
            },
        )
        .unwrap();
        let mut want = given.clone();
        want.insert("DEVELOPER_DIR".into(), "/X/Contents/Developer".into());
        want.insert(
            "SDKROOT".into(),
            "/X/Contents/Developer/SDKs/MacOSX27.0.sdk".into(),
        );
        assert_eq!(got, want);
    }

    #[test]
    fn an_environment_without_the_names_is_left_alone() {
        let given = env(&[("PATH", "/bin"), ("HOME", "/h")]);
        assert_eq!(rewrite(&given, &no_dir, &no_sdk).unwrap(), given);
    }

    #[test]
    fn an_sdk_without_a_platform_is_not_looked_up() {
        let given = env(&[("APPLE_SDK_VERSION_OVERRIDE", "27.0")]);
        assert_eq!(rewrite(&given, &no_dir, &no_sdk).unwrap(), given);
    }

    #[test]
    fn a_missing_xcode_is_an_error() {
        let given = env(&[("XCODE_VERSION_OVERRIDE", "1.0")]);
        let err = rewrite(&given, &|_| Err("not found".to_owned()), &no_sdk).unwrap_err();
        assert_eq!(err, "not found");
    }
}
