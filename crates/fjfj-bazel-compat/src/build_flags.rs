//! The flags that shape a build's configuration and how it runs
//! (buildfiji-gwl.13): `-c`/`--compilation_mode`, `--cpu`, `--define`,
//! `--jobs`/`-j`, `--symlink_prefix`, `--show_result`, the tool option flags
//! (`--copt`, `--linkopt`, ...) and Starlark flags (`--//pkg:flag=value`).

use crate::flag_registry::FlagRegistry;

/// Flag names this module reads, for `clap_flags::validate`'s
/// unimplemented-flag gate.
pub const IMPLEMENTED: &[&str] = &[
    "compilation_mode",
    "platforms",
    "extra_toolchains",
    "extra_execution_platforms",
    "host_platform",
    "toolchain_resolution_debug",
    "action_env",
    "aspects",
    "output_groups",
    "cpu",
    "define",
    "jobs",
    "spawn_strategy",
    "symlink_prefix",
    "show_result",
    "build",
    "skip_incompatible_explicit_targets",
    "experimental_platform_in_output_dir",
    "experimental_override_platform_cpu_name",
    "experimental_convenience_symlinks",
    "expand_test_suites",
    "enable_runfiles",
    "build_runfile_links",
    "legacy_external_runfiles",
    "strip",
    "stamp",
    "collect_code_coverage",
    "force_pic",
    "save_temps",
    "fission",
    "host_javacopt",
    "java_runtime_version",
    "tool_java_runtime_version",
    "java_language_version",
    "tool_java_language_version",
    "copt",
    "cxxopt",
    "conlyopt",
    "linkopt",
    "host_copt",
    "javacopt",
    "custom_malloc",
    "proto_compiler",
    "proto_toolchain_for_cc",
    "proto_toolchain_for_java",
    "java_launcher",
    "fdo_profile",
    "cs_fdo_profile",
    "xbinary_fdo",
    "memprof_profile",
    "propeller_optimize",
    "fdo_prefetch_hints",
    "grte_top",
    "fdo_optimize",
];

/// The boolean flags that reach the configuration, with their negations.
const SWITCHES: &[&str] = &[
    "stamp",
    "collect_code_coverage",
    "force_pic",
    "save_temps",
    "enable_runfiles",
    "build_runfile_links",
    "legacy_external_runfiles",
];

/// The flags that are lists of options, kept in `options` joined by a space
/// in the order given.
const LIST_OPTIONS: &[&str] = &[
    "copt",
    "cxxopt",
    "conlyopt",
    "linkopt",
    "host_copt",
    "javacopt",
    "host_javacopt",
];

/// The flags that hold a string, the last one given winning: what a
/// `config_setting(values = ...)` or a `select` on them reads.
const STRING_OPTIONS: &[&str] = &[
    "java_runtime_version",
    "tool_java_runtime_version",
    "java_language_version",
    "tool_java_language_version",
];

/// The flags that name a label, the last one given winning: the options the
/// late-bound `configuration_field` defaults follow.
pub const LABEL_OPTIONS: &[&str] = &[
    "custom_malloc",
    "proto_compiler",
    "proto_toolchain_for_cc",
    "proto_toolchain_for_java",
    "java_launcher",
    "fdo_profile",
    "cs_fdo_profile",
    "xbinary_fdo",
    "memprof_profile",
    "propeller_optimize",
    "fdo_prefetch_hints",
    "grte_top",
    "fdo_optimize",
];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildFlags {
    /// `-c`: `fastbuild`, `dbg` or `opt`, checked by whoever uses it.
    pub compilation_mode: Option<String>,
    pub cpu: Option<String>,
    /// `--define k=v`, later ones for a name replacing earlier ones.
    pub defines: Vec<(String, String)>,
    /// `--jobs`: a number, `auto`, or `HOST_CPUS*0.5`.
    pub jobs: Option<String>,
    /// `--spawn_strategy`: strategy names, comma-separated, in the order given.
    pub spawn_strategy: Option<String>,
    pub symlink_prefix: Option<String>,
    /// `--platforms`: the target platform.
    pub platforms: Option<String>,
    /// `--extra_toolchains`, in order.
    pub extra_toolchains: Vec<String>,
    /// `--extra_execution_platforms`, in order.
    pub extra_execution_platforms: Vec<String>,
    /// `--host_platform`: the platform the host is.
    pub host_platform: Option<String>,
    /// `--toolchain_resolution_debug`: comma-separated regular expressions, a
    /// `-` in front of one excluding what it finds.
    pub toolchain_resolution_debug: Option<String>,
    /// `--action_env`: `NAME=VALUE` or `NAME`, in order.
    pub action_env: Vec<String>,
    /// `--aspects`: `<bzl label>%<aspect name>` for each, in order.
    pub aspects: Vec<String>,
    /// `--output_groups`: the groups to build, each as written (`+name`, `-name`).
    pub output_groups: Vec<String>,
    pub show_result: Option<String>,
    /// `--build` (default true): `--nobuild` stops after analysis.
    pub build: Option<bool>,
    /// `--skip_incompatible_explicit_targets`: a target named outright that
    /// the platform cannot build is skipped, as one a wildcard selects is.
    pub skip_incompatible_explicit_targets: bool,
    /// `--experimental_platform_in_output_dir`: the output directory is named
    /// for the target platform, not the cpu.
    pub platform_in_output_dir: bool,
    /// `--experimental_override_name_platform_in_output_dir`: `LABEL=NAME`
    /// for each, as written, in order.
    pub platform_name_overrides: Vec<String>,
    /// `--stamp`, `--collect_code_coverage`, `--force_pic` and `--save_temps`,
    /// each as the last of its spellings gave it.
    pub switches: Vec<(String, bool)>,
    /// `--experimental_convenience_symlinks`: `normal`, `clean`, `ignore` or
    /// `log_only`, checked by whoever uses it.
    pub convenience_symlinks: Option<String>,
    /// `--[no]expand_test_suites` (default on).
    pub expand_test_suites: Option<bool>,
    /// `--strip`: `always`, `sometimes` or `never`, checked by whoever uses it.
    pub strip: Option<String>,
    /// `--fission`: the modes it is on in, as written.
    pub fission: Option<String>,
    /// `--copt` and the like, by flag name, each value in order.
    pub options: Vec<(String, String)>,
    /// `--//pkg:flag=value`, in order.
    pub starlark_flags: Vec<(String, String)>,
}

/// Pull [`BuildFlags`] out of `args`, returning the rest in their original
/// order.
pub fn extract(args: &[String], command: &str) -> (BuildFlags, Vec<String>) {
    let registry = FlagRegistry::global();
    let mut flags = BuildFlags::default();
    let mut rest = Vec::with_capacity(args.len());
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if !arg.starts_with('-') {
            rest.push(arg.clone());
            continue;
        }
        // A boolean Starlark flag turned off: `--no//pkg:name`.
        if let Some(body) = arg
            .strip_prefix("--no")
            .filter(|b| b.starts_with("//") || b.starts_with('@'))
        {
            flags
                .starlark_flags
                .push((body.to_owned(), "false".to_owned()));
            continue;
        }
        // A Starlark flag: `--//pkg:name=value`, `--@repo//pkg:name=value`.
        if let Some(body) = arg
            .strip_prefix("--")
            .filter(|b| b.starts_with("//") || b.starts_with('@'))
        {
            match body.split_once('=') {
                Some((name, value)) => flags
                    .starlark_flags
                    .push((name.to_owned(), value.to_owned())),
                None => flags
                    .starlark_flags
                    .push((body.to_owned(), "true".to_owned())),
            }
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
        if name == "build" {
            flags.build = Some(!m.negated);
            continue;
        }
        if name == "skip_incompatible_explicit_targets" {
            flags.skip_incompatible_explicit_targets = !m.negated;
            continue;
        }
        if name == "expand_test_suites" {
            flags.expand_test_suites = Some(!m.negated);
            continue;
        }
        if SWITCHES.contains(&name) {
            flags.switches.retain(|(n, _)| n != name);
            flags.switches.push((name.to_owned(), !m.negated));
            continue;
        }
        if name == "experimental_platform_in_output_dir" {
            flags.platform_in_output_dir = !m.negated;
            continue;
        }
        let Some(value) = m.value.map(str::to_string).or_else(|| iter.next().cloned()) else {
            rest.push(arg.clone());
            continue;
        };
        match name {
            "compilation_mode" => flags.compilation_mode = Some(value),
            "cpu" => flags.cpu = Some(value),
            "platforms" => flags.platforms = Some(value),
            "aspects" => flags.aspects.extend(
                value
                    .split(',')
                    .filter(|p| !p.is_empty())
                    .map(str::to_owned),
            ),
            "output_groups" => flags.output_groups.extend(
                value
                    .split(',')
                    .filter(|p| !p.is_empty())
                    .map(str::to_owned),
            ),
            "extra_execution_platforms" => flags.extra_execution_platforms.extend(
                value
                    .split(',')
                    .filter(|p| !p.is_empty())
                    .map(str::to_owned),
            ),
            "host_platform" => flags.host_platform = Some(value),
            "strip" => flags.strip = Some(value),
            "experimental_convenience_symlinks" => flags.convenience_symlinks = Some(value),
            "fission" => flags.fission = Some(value),
            "experimental_override_platform_cpu_name" => flags.platform_name_overrides.push(value),
            "toolchain_resolution_debug" => flags.toolchain_resolution_debug = Some(value),
            "action_env" => flags.action_env.push(value),
            "extra_toolchains" => flags.extra_toolchains.extend(
                value
                    .split(',')
                    .filter(|p| !p.is_empty())
                    .map(str::to_owned),
            ),
            "define" => match value.split_once('=') {
                Some((k, v)) => {
                    flags.defines.retain(|(name, _)| name != k);
                    flags.defines.push((k.to_owned(), v.to_owned()));
                }
                None => rest.push(arg.clone()),
            },
            "jobs" => flags.jobs = Some(value),
            "spawn_strategy" => {
                flags.spawn_strategy = Some(match flags.spawn_strategy.take() {
                    Some(earlier) => format!("{earlier},{value}"),
                    None => value,
                });
            }
            "symlink_prefix" => flags.symlink_prefix = Some(value),
            "show_result" => flags.show_result = Some(value),
            other if LABEL_OPTIONS.contains(&other) || STRING_OPTIONS.contains(&other) => {
                flags.options.retain(|(name, _)| name != other);
                flags.options.push((other.to_owned(), value));
            }
            other if LIST_OPTIONS.contains(&other) => {
                flags.options.push((other.to_owned(), value));
            }
            _ => rest.push(arg.clone()),
        }
    }
    (flags, rest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> Vec<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn the_platform_in_the_output_dir_flags_are_taken() {
        let (flags, rest) = extract(
            &args(&[
                "--experimental_platform_in_output_dir",
                "--experimental_override_name_platform_in_output_dir=//:q=a",
                "--experimental_override_name_platform_in_output_dir=//:p=b",
                "//a:a",
            ]),
            "build",
        );
        assert!(flags.platform_in_output_dir);
        assert_eq!(flags.platform_name_overrides, ["//:q=a", "//:p=b"]);
        assert_eq!(rest, ["//a:a"]);
        let (flags, _) = extract(
            &args(&[
                "--experimental_platform_in_output_dir",
                "--noexperimental_platform_in_output_dir",
            ]),
            "build",
        );
        assert!(!flags.platform_in_output_dir);
    }

    #[test]
    fn the_switches_the_fragments_read_are_taken_last_spelling_wins() {
        let (flags, rest) = extract(
            &args(&[
                "--stamp",
                "--collect_code_coverage",
                "--nostamp",
                "--stamp",
                "--noforce_pic",
                "--strip=always",
                "--fission",
                "dbg,opt",
                "//a:a",
            ]),
            "build",
        );
        assert_eq!(
            flags.switches,
            [
                ("collect_code_coverage".to_owned(), true),
                ("stamp".to_owned(), true),
                ("force_pic".to_owned(), false),
            ]
        );
        assert_eq!(flags.strip.as_deref(), Some("always"));
        assert_eq!(flags.fission.as_deref(), Some("dbg,opt"));
        assert_eq!(rest, ["//a:a"]);
    }

    #[test]
    fn the_java_version_flags_keep_the_last_value_and_javacopts_add_up() {
        let (flags, _) = extract(
            &args(&[
                "--java_runtime_version=8",
                "--java_runtime_version=17",
                "--host_javacopt=-a",
                "--host_javacopt=-b",
            ]),
            "build",
        );
        assert_eq!(
            flags.options,
            [
                ("java_runtime_version".to_owned(), "17".to_owned()),
                ("host_javacopt".to_owned(), "-a".to_owned()),
                ("host_javacopt".to_owned(), "-b".to_owned()),
            ]
        );
    }

    #[test]
    fn skip_incompatible_explicit_targets_is_a_switch() {
        let (flags, rest) = extract(
            &args(&["--skip_incompatible_explicit_targets", "//a:a"]),
            "build",
        );
        assert!(flags.skip_incompatible_explicit_targets);
        assert_eq!(rest, ["//a:a"]);
        let (flags, _) = extract(
            &args(&[
                "--skip_incompatible_explicit_targets",
                "--noskip_incompatible_explicit_targets",
            ]),
            "build",
        );
        assert!(!flags.skip_incompatible_explicit_targets);
    }

    #[test]
    fn the_flags_come_out_in_both_spellings() {
        let (flags, rest) = extract(
            &args(&[
                "-c",
                "opt",
                "--cpu=k8",
                "--define",
                "a=b",
                "--define=c=d",
                "--define=a=z",
                "-j",
                "4",
                "--symlink_prefix=out-",
                "--copt=-O2",
                "--copt",
                "-g",
                "//pkg:x",
                "--//flags:f=v",
                "--@dep//f:g",
            ]),
            "build",
        );
        assert_eq!(flags.compilation_mode.as_deref(), Some("opt"));
        assert_eq!(flags.cpu.as_deref(), Some("k8"));
        assert_eq!(
            flags.defines,
            [
                ("c".to_owned(), "d".to_owned()),
                ("a".to_owned(), "z".to_owned())
            ]
        );
        assert_eq!(flags.jobs.as_deref(), Some("4"));
        assert_eq!(flags.symlink_prefix.as_deref(), Some("out-"));
        assert_eq!(
            flags.options,
            [
                ("copt".to_owned(), "-O2".to_owned()),
                ("copt".to_owned(), "-g".to_owned())
            ]
        );
        assert_eq!(
            flags.starlark_flags,
            [
                ("//flags:f".to_owned(), "v".to_owned()),
                ("@dep//f:g".to_owned(), "true".to_owned())
            ]
        );
        assert_eq!(rest, ["//pkg:x"]);
    }

    #[test]
    fn action_env_keeps_each_entry_in_order() {
        let (flags, rest) = extract(
            &args(&["--action_env=A=1", "--action_env", "B", "//x"]),
            "build",
        );
        assert_eq!(flags.action_env, ["A=1", "B"]);
        assert_eq!(rest, ["//x"]);
    }

    #[test]
    fn other_flags_pass_through() {
        let (flags, rest) = extract(&args(&["--keep_going", "//a"]), "build");
        assert_eq!(flags, BuildFlags::default());
        assert_eq!(rest, ["--keep_going", "//a"]);
    }
}
