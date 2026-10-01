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
    "cpu",
    "define",
    "jobs",
    "symlink_prefix",
    "show_result",
    "copt",
    "cxxopt",
    "conlyopt",
    "linkopt",
    "host_copt",
    "javacopt",
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
    pub symlink_prefix: Option<String>,
    /// `--platforms`: the target platform.
    pub platforms: Option<String>,
    /// `--extra_toolchains`, in order.
    pub extra_toolchains: Vec<String>,
    pub show_result: Option<String>,
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
        let Some(value) = m.value.map(str::to_string).or_else(|| iter.next().cloned()) else {
            rest.push(arg.clone());
            continue;
        };
        match name {
            "compilation_mode" => flags.compilation_mode = Some(value),
            "cpu" => flags.cpu = Some(value),
            "platforms" => flags.platforms = Some(value),
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
            "symlink_prefix" => flags.symlink_prefix = Some(value),
            "show_result" => flags.show_result = Some(value),
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
    fn other_flags_pass_through() {
        let (flags, rest) = extract(&args(&["--keep_going", "//a"]), "build");
        assert_eq!(flags, BuildFlags::default());
        assert_eq!(rest, ["--keep_going", "//a"]);
    }
}
