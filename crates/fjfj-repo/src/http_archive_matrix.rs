//! Generated from Bazel 9.2.0 probes: what Bazel's own `http_archive` and `http_file` do.

use crate::http_archive_tests::{HaRow, Serve};

pub(crate) const HA_ROWS: &[HaRow] = &[
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", build_file_content="filegroup(name='all')")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (
                r#"+http_archive+x/BUILD.bazel"#,
                r#"x filegroup(name='all')"#,
            ),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+x/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], integrity="@INT:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+x/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", url="@URL@/d.zip", sha256 = "@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+x/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/gone.zip", "@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+x/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/gone.zip"#, r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", build_file_content="x")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/top"#, r#"<dir>"#),
            (r#"+http_archive+x/top/a.txt"#, r#"A"#),
            (r#"+http_archive+x/top/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/top/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/top/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="nothere", build_file_content="x")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: Some(
            r#"java.io.IOException: Error extracting <ext>/+http_archive+x/tempN/d.zip to <ext>/+http_archive+x/tempN: Prefix "nothere" was given, but not found in the archive. Here are possible prefixes for this archive: "top"."#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", add_prefix="pre/fix", build_file_content="x")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/pre"#, r#"<dir>"#),
            (r#"+http_archive+x/pre/fix"#, r#"<dir>"#),
            (r#"+http_archive+x/pre/fix/a.txt"#, r#"A"#),
            (r#"+http_archive+x/pre/fix/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/pre/fix/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/pre/fix/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], build_file_content="x")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: Some(
            r#"java.io.IOException: No URLs left after removing plain http URLs due to missing checksum. Please provide either a checksum or an https download location."#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", build_file_content="x")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: Some(r#"At least one of url and urls must be provided"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], url="@URL@/d.zip", sha256 = "@SHA:d.zip@", build_file_content="x")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/top"#, r#"<dir>"#),
            (r#"+http_archive+x/top/a.txt"#, r#"A"#),
            (r#"+http_archive+x/top/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/top/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/top/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", build_file="//:BUILD.dep")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[
            (
                r#"BUILD.dep"#,
                r#"filegroup(name='dep')
"#,
            ),
            (r#"BUILD.bazel"#, r#""#),
        ],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (
                r#"+http_archive+x/BUILD.bazel"#,
                r#"x filegroup(name='dep')
"#,
            ),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+x/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", build_file="//:BUILD.dep", build_file_content="x")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(r#"BUILD.dep"#, r#"x"#)],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: Some(r#"Only one of build_file and build_file_content can be provided."#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+x/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", build_file_content="x", type="zip")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+x/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/blob"], sha256="@SHA:blob@", strip_prefix="top", build_file_content="x", type="tar.gz")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"blob"#,
            Serve::Archive(
                r#"tar.gz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+x/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/blob"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/blob"], sha256="@SHA:blob@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"blob"#,
            Serve::Archive(
                r#"tar.gz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: Some(
            r#"Expected a file with a .zip, .jar, .war, .aar, .nupkg, .whl, .tar, .tar.gz, .tgz, .gz, .tar.xz, .txz, .xz, .tar.zst, .tzst, .zst, .tar.bz2, .tbz, .bz2, .ar, .deb or .7z suffix (got <ext>/+http_archive+x/tempN/blob)"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/blob"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.tar.gz"], sha256="@SHA:d.tar.gz@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.tar.gz"#,
            Serve::Archive(
                r#"tar.gz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+x/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.tar.gz"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", build_file_content="x", patches=["//:fix.patch"], patch_args=["-p1"])
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[
            (
                r#"fix.patch"#,
                r#"--- a/a.txt
+++ b/a.txt
@@ -1 +1 @@
-A
+PATCHED
"#,
            ),
            (r#"BUILD.bazel"#, r#""#),
        ],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (
                r#"+http_archive+x/a.txt"#,
                r#"PATCHED
"#,
            ),
            (r#"+http_archive+x/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", build_file_content="x", patches=["//:fix.patch"])
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[
            (
                r#"fix.patch"#,
                r#"--- a/a.txt
+++ b/a.txt
@@ -1 +1 @@
-A
+PATCHED
"#,
            ),
            (r#"BUILD.bazel"#, r#""#),
        ],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: Some(
            r#"Error applying patch <ws>/fix.patch: Cannot find file to patch (near line 1), old file name (a/a.txt) doesn't exist, new file name (b/a.txt) doesn't exist."#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", build_file_content="x", patch_cmds=["echo hi > made.txt", "echo again >> made.txt"])
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+x/link"#, r#"-> a.txt"#),
            (
                r#"+http_archive+x/made.txt"#,
                r#"hi
again
"#,
            ),
            (r#"+http_archive+x/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", build_file_content="x", patch_cmds=["exit 3"])
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: Some(r#"Error applying patch command exit 3:"#),
        printed: &[],
        tree: &[],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", build_file_content="x", remote_file_urls={"extra.txt": ["@URL@/f.txt"]}, remote_file_integrity={"extra.txt": "@INT:f.txt@"})
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(
                    r#"zip"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/link"#, "l", r#"a.txt"#),
                    ],
                ),
            ),
            (r#"f.txt"#, Serve::Text(r#"hello"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+x/extra.txt"#, r#"x hello"#),
            (r#"+http_archive+x/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.zip"#, r#"/f.txt"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", build_file_content="x", files={"copied.txt": "//:src.txt"})
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[
            (
                r#"src.txt"#,
                r#"source
"#,
            ),
            (r#"BUILD.bazel"#, r#""#),
        ],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+x/copied.txt"#, r#"-> <ws>/src.txt"#),
            (r#"+http_archive+x/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", build_file_content="x", remote_patches={"@URL@/p.patch": "@INT:p.patch@"}, remote_patch_strip=1)
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(
                    r#"zip"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/link"#, "l", r#"a.txt"#),
                    ],
                ),
            ),
            (
                r#"p.patch"#,
                Serve::Text(
                    r#"--- a/a.txt
+++ b/a.txt
@@ -1 +1 @@
-A
+REMOTE
"#,
                ),
            ),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (
                r#"+http_archive+x/a.txt"#,
                r#"REMOTE
"#,
            ),
            (r#"+http_archive+x/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.zip"#, r#"/p.patch"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", build_file_content="x", canonical_id="cid")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+x/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", build_file_content="x", workspace_file_content="workspace(name='x')")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+x/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=[], sha256 = "@SHA:d.zip@", build_file_content="x")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: Some(r#"At least one of url and urls must be provided"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="ABC", build_file_content="x")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: Some(
            r#"Checksum error in repository @@+http_archive+x: Invalid SHA-256 checksum 'ABC'"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", build_file_content="x", netrc="/nonexistent")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: Some(r#"java.io.FileNotFoundException: /nonexistent (No such file or directory)"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", build_file_content="x", auth_patterns={"@URL@/*": "Bearer <password>"})
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/link"#, "l", r#"a.txt"#),
                ],
            ),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+x/link"#, r#"-> a.txt"#),
            (r#"+http_archive+x/sub"#, r#"<dir>"#),
            (r#"+http_archive+x/sub/b.txt"#, r#"x B"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_file(name="x", urls=["@URL@/f.txt"], sha256="@SHA:f.txt@")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (
                r#"+http_file+x/WORKSPACE"#,
                r#"x workspace(name = "+http_file+x")"#,
            ),
            (r#"+http_file+x/file"#, r#"<dir>"#),
            (
                r#"+http_file+x/file/BUILD"#,
                r#"x package(default_visibility = ["//visibility:public"])

exports_files(["downloaded"])

filegroup(
    name = "file",
    srcs = ["downloaded"],
)
"#,
            ),
            (r#"+http_file+x/file/downloaded"#, r#"hello"#),
        ],
        requests: &[r#"/f.txt"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_file(name="x", urls=["@URL@/f.txt"], sha256="@SHA:f.txt@", downloaded_file_path="renamed.dat", executable=True)
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (
                r#"+http_file+x/WORKSPACE"#,
                r#"x workspace(name = "+http_file+x")"#,
            ),
            (r#"+http_file+x/file"#, r#"<dir>"#),
            (
                r#"+http_file+x/file/BUILD"#,
                r#"x package(default_visibility = ["//visibility:public"])

exports_files(["renamed.dat"])

filegroup(
    name = "file",
    srcs = ["renamed.dat"],
)
"#,
            ),
            (r#"+http_file+x/file/renamed.dat"#, r#"x hello"#),
        ],
        requests: &[r#"/f.txt"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_file(name="x", urls=["@URL@/f.txt"], integrity="@INT:f.txt@")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (
                r#"+http_file+x/WORKSPACE"#,
                r#"x workspace(name = "+http_file+x")"#,
            ),
            (r#"+http_file+x/file"#, r#"<dir>"#),
            (
                r#"+http_file+x/file/BUILD"#,
                r#"x package(default_visibility = ["//visibility:public"])

exports_files(["downloaded"])

filegroup(
    name = "file",
    srcs = ["downloaded"],
)
"#,
            ),
            (r#"+http_file+x/file/downloaded"#, r#"hello"#),
        ],
        requests: &[r#"/f.txt"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_file(name="x", urls=["@URL@/f.txt"])
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        fetch: &[r#"@x"#],
        error: Some(
            r#"java.io.IOException: No URLs left after removing plain http URLs due to missing checksum. Please provide either a checksum or an https download location."#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_file(name="x", urls=["@URL@/f.txt"], sha256="@SHA:f.txt@", downloaded_file_path="a/b/c.txt")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (
                r#"+http_file+x/WORKSPACE"#,
                r#"x workspace(name = "+http_file+x")"#,
            ),
            (r#"+http_file+x/file"#, r#"<dir>"#),
            (
                r#"+http_file+x/file/BUILD"#,
                r#"x package(default_visibility = ["//visibility:public"])

exports_files(["a/b/c.txt"])

filegroup(
    name = "file",
    srcs = ["a/b/c.txt"],
)
"#,
            ),
            (r#"+http_file+x/file/a"#, r#"<dir>"#),
            (r#"+http_file+x/file/a/b"#, r#"<dir>"#),
            (r#"+http_file+x/file/a/b/c.txt"#, r#"hello"#),
        ],
        requests: &[r#"/f.txt"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
git_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")
new_git_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:git.bzl", "new_git_repository")
git_repository(name="x", remote="@GIT:r@", commit="@COMMIT:r@", tag="v1")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[(
            r#"r"#,
            &[
                &[
                    (
                        r#"a.txt"#, r#"one
"#,
                    ),
                    (
                        r#"sub/b.txt"#,
                        r#"B
"#,
                    ),
                ],
                &[(
                    r#"a.txt"#, r#"two
"#,
                )],
            ],
            &[(0, r#"v1"#)],
        )],
        files: &[],
        serve: &[],
        fetch: &[r#"@x"#],
        error: Some(r#"At most one of commit, tag, or branch may be provided"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
git_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")
new_git_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:git.bzl", "new_git_repository")
git_repository(name="x", remote="@GIT:r@", commit="0000000000000000000000000000000000000001")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[(
            r#"r"#,
            &[
                &[
                    (
                        r#"a.txt"#, r#"one
"#,
                    ),
                    (
                        r#"sub/b.txt"#,
                        r#"B
"#,
                    ),
                ],
                &[(
                    r#"a.txt"#, r#"two
"#,
                )],
            ],
            &[(0, r#"v1"#)],
        )],
        files: &[],
        serve: &[],
        fetch: &[r#"@x"#],
        error: Some(
            r#"error running 'git reset --hard <commit>' while working with @+git_repository+x:"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
git_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")
new_git_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:git.bzl", "new_git_repository")
git_repository(name="x", remote="file:///nonexistent/repo", commit="@COMMIT:r@")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[(
            r#"r"#,
            &[
                &[
                    (
                        r#"a.txt"#, r#"one
"#,
                    ),
                    (
                        r#"sub/b.txt"#,
                        r#"B
"#,
                    ),
                ],
                &[(
                    r#"a.txt"#, r#"two
"#,
                )],
            ],
            &[(0, r#"v1"#)],
        )],
        files: &[],
        serve: &[],
        fetch: &[r#"@x"#],
        error: Some(
            r#"error running 'git fetch origin refs/heads/*:refs/remotes/origin/* refs/tags/*:refs/tags/*' while working with @+git_repository+x:"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
git_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")
new_git_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:git.bzl", "new_git_repository")
git_repository(name="x", remote="@GIT:r@", tag="nope")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[(
            r#"r"#,
            &[
                &[
                    (
                        r#"a.txt"#, r#"one
"#,
                    ),
                    (
                        r#"sub/b.txt"#,
                        r#"B
"#,
                    ),
                ],
                &[(
                    r#"a.txt"#, r#"two
"#,
                )],
            ],
            &[(0, r#"v1"#)],
        )],
        files: &[],
        serve: &[],
        fetch: &[r#"@x"#],
        error: Some(
            r#"error running 'git fetch origin tags/nope:tags/nope' while working with @+git_repository+x:"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "local_repository")
new_local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "new_local_repository")
local_repository(name="x", path="sub")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[
            (
                r#"sub/BUILD.bazel"#,
                r#"filegroup(name='all', srcs=glob(['**']))"#,
            ),
            (r#"sub/a.txt"#, r#"A"#),
            (r#"sub/d/b.txt"#, r#"B"#),
            (
                r#"sub/MODULE.bazel"#,
                r#"module(name='x')
"#,
            ),
        ],
        serve: &[],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (
                r#"+local_repository+x/BUILD.bazel"#,
                r#"filegroup(name='all', srcs=glob(['**']))"#,
            ),
            (
                r#"+local_repository+x/MODULE.bazel"#,
                r#"module(name='x')
"#,
            ),
            (r#"+local_repository+x/a.txt"#, r#"A"#),
            (r#"+local_repository+x/d"#, r#"<dir>"#),
            (r#"+local_repository+x/d/b.txt"#, r#"B"#),
        ],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "local_repository")
new_local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "new_local_repository")
local_repository(name="x", path="@WS@/sub")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[
            (
                r#"sub/BUILD.bazel"#,
                r#"filegroup(name='all', srcs=glob(['**']))"#,
            ),
            (r#"sub/a.txt"#, r#"A"#),
            (r#"sub/d/b.txt"#, r#"B"#),
            (
                r#"sub/MODULE.bazel"#,
                r#"module(name='x')
"#,
            ),
        ],
        serve: &[],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (
                r#"+local_repository+x/BUILD.bazel"#,
                r#"filegroup(name='all', srcs=glob(['**']))"#,
            ),
            (
                r#"+local_repository+x/MODULE.bazel"#,
                r#"module(name='x')
"#,
            ),
            (r#"+local_repository+x/a.txt"#, r#"A"#),
            (r#"+local_repository+x/d"#, r#"<dir>"#),
            (r#"+local_repository+x/d/b.txt"#, r#"B"#),
        ],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "local_repository")
new_local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "new_local_repository")
local_repository(name="x", path="nope")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[
            (
                r#"sub/BUILD.bazel"#,
                r#"filegroup(name='all', srcs=glob(['**']))"#,
            ),
            (r#"sub/a.txt"#, r#"A"#),
            (r#"sub/d/b.txt"#, r#"B"#),
            (
                r#"sub/MODULE.bazel"#,
                r#"module(name='x')
"#,
            ),
        ],
        serve: &[],
        fetch: &[r#"@x"#],
        error: Some(
            r#"The repository's path is "nope" (absolute: "<ws>/nope") but it does not exist or is not a directory."#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "local_repository")
new_local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "new_local_repository")
local_repository(name="x", path="sub/a.txt")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[
            (
                r#"sub/BUILD.bazel"#,
                r#"filegroup(name='all', srcs=glob(['**']))"#,
            ),
            (r#"sub/a.txt"#, r#"A"#),
            (r#"sub/d/b.txt"#, r#"B"#),
            (
                r#"sub/MODULE.bazel"#,
                r#"module(name='x')
"#,
            ),
        ],
        serve: &[],
        fetch: &[r#"@x"#],
        error: Some(
            r#"The repository's path is "sub/a.txt" (absolute: "<ws>/sub/a.txt") but it does not exist or is not a directory."#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "local_repository")
new_local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "new_local_repository")
new_local_repository(name="x", path="sub", build_file_content="filegroup(name='f')")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[
            (
                r#"sub/BUILD.bazel"#,
                r#"filegroup(name='all', srcs=glob(['**']))"#,
            ),
            (r#"sub/a.txt"#, r#"A"#),
            (r#"sub/d/b.txt"#, r#"B"#),
            (
                r#"sub/MODULE.bazel"#,
                r#"module(name='x')
"#,
            ),
        ],
        serve: &[],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (
                r#"+new_local_repository+x/BUILD.bazel"#,
                r#"x filegroup(name='f')"#,
            ),
            (
                r#"+new_local_repository+x/MODULE.bazel"#,
                r#"-> <ws>/sub/MODULE.bazel"#,
            ),
            (r#"+new_local_repository+x/a.txt"#, r#"-> <ws>/sub/a.txt"#),
            (r#"+new_local_repository+x/d"#, r#"-> <ws>/sub/d"#),
        ],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "local_repository")
new_local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "new_local_repository")
new_local_repository(name="x", path="sub", build_file="//:b.BUILD")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[
            (
                r#"sub/BUILD.bazel"#,
                r#"filegroup(name='all', srcs=glob(['**']))"#,
            ),
            (r#"sub/a.txt"#, r#"A"#),
            (r#"sub/d/b.txt"#, r#"B"#),
            (
                r#"sub/MODULE.bazel"#,
                r#"module(name='x')
"#,
            ),
            (r#"b.BUILD"#, r#"filegroup(name='fromfile')"#),
        ],
        serve: &[],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (
                r#"+new_local_repository+x/BUILD.bazel"#,
                r#"-> <ws>/b.BUILD"#,
            ),
            (
                r#"+new_local_repository+x/MODULE.bazel"#,
                r#"-> <ws>/sub/MODULE.bazel"#,
            ),
            (r#"+new_local_repository+x/a.txt"#, r#"-> <ws>/sub/a.txt"#),
            (r#"+new_local_repository+x/d"#, r#"-> <ws>/sub/d"#),
        ],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "local_repository")
new_local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "new_local_repository")
new_local_repository(name="x", path="sub")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[
            (
                r#"sub/BUILD.bazel"#,
                r#"filegroup(name='all', srcs=glob(['**']))"#,
            ),
            (r#"sub/a.txt"#, r#"A"#),
            (r#"sub/d/b.txt"#, r#"B"#),
            (
                r#"sub/MODULE.bazel"#,
                r#"module(name='x')
"#,
            ),
        ],
        serve: &[],
        fetch: &[r#"@x"#],
        error: Some(r#"exactly one of `build_file` and `build_file_content` must be specified"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "local_repository")
new_local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "new_local_repository")
new_local_repository(name="x", path="sub", build_file="//:b.BUILD", build_file_content="x")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[
            (
                r#"sub/BUILD.bazel"#,
                r#"filegroup(name='all', srcs=glob(['**']))"#,
            ),
            (r#"sub/a.txt"#, r#"A"#),
            (r#"sub/d/b.txt"#, r#"B"#),
            (
                r#"sub/MODULE.bazel"#,
                r#"module(name='x')
"#,
            ),
            (r#"b.BUILD"#, r#"y"#),
        ],
        serve: &[],
        fetch: &[r#"@x"#],
        error: Some(r#"exactly one of `build_file` and `build_file_content` must be specified"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "local_repository")
new_local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "new_local_repository")
new_local_repository(name="x", path="nope", build_file_content="x")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[
            (
                r#"sub/BUILD.bazel"#,
                r#"filegroup(name='all', srcs=glob(['**']))"#,
            ),
            (r#"sub/a.txt"#, r#"A"#),
            (r#"sub/d/b.txt"#, r#"B"#),
            (
                r#"sub/MODULE.bazel"#,
                r#"module(name='x')
"#,
            ),
        ],
        serve: &[],
        fetch: &[r#"@x"#],
        error: Some(
            r#"The repository's path is "nope" (absolute: "<ws>/nope") but it does not exist or is not a directory."#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "local_repository")
new_local_repository = use_repo_rule("@bazel_tools//tools/build_defs/repo:local.bzl", "new_local_repository")
new_local_repository(name="x", path="sub", build_file_content="x", build_file="//:b.BUILD")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[
            (
                r#"sub/BUILD.bazel"#,
                r#"filegroup(name='all', srcs=glob(['**']))"#,
            ),
            (r#"sub/a.txt"#, r#"A"#),
            (r#"sub/d/b.txt"#, r#"B"#),
            (
                r#"sub/MODULE.bazel"#,
                r#"module(name='x')
"#,
            ),
            (r#"b.BUILD"#, r#"y"#),
        ],
        serve: &[],
        fetch: &[r#"@x"#],
        error: Some(r#"exactly one of `build_file` and `build_file_content` must be specified"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", netrc="@WS@/netrc")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"netrc"#,
            r#"machine 127.0.0.1 login u password p
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Basic dTpw"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", netrc="@WS@/netrc")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"netrc"#,
            r#"machine other login u password p
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", netrc="@WS@/netrc")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"netrc"#,
            r#"default login dl password dp
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Basic ZGw6ZHA="#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", netrc="@WS@/netrc")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"netrc"#,
            r#"machine 127.0.0.1
  login u2
  password p2
machine x login a password b
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Basic dTI6cDI="#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", netrc="@WS@/netrc")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"netrc"#,
            r#"machine 127.0.0.1 login u
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[r#"WARNING: Found machine in .netrc for URL @URL@/d.zip, but no password."#],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", netrc="@WS@/nonexistent")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: Some(
            r#"java.io.FileNotFoundException: <ws>/nonexistent (No such file or directory)"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", netrc="@WS@/netrc", auth_patterns={"127.0.0.1":"Bearer <password>"})
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"netrc"#,
            r#"machine 127.0.0.1 login u password tok
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Bearer tok"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", netrc="@WS@/netrc", auth_patterns={"127.0.0.1":"Token <login>:<password>"})
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"netrc"#,
            r#"machine 127.0.0.1 login u password tok
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Token u:tok"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", netrc="@WS@/netrc", auth_patterns={"127.0.0.1":"Bearer <password>"})
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"netrc"#,
            r#"machine other login u password tok
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", auth_patterns={"127.0.0.1":"Bearer <password>"})
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_file(name="x", urls=["@URL@/f.txt"], sha256="@SHA:f.txt@", netrc="@WS@/netrc")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"netrc"#,
            r#"machine 127.0.0.1 login u password p
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (
                r#"+http_file+x/WORKSPACE"#,
                r#"x workspace(name = "+http_file+x")"#,
            ),
            (r#"+http_file+x/file"#, r#"<dir>"#),
            (
                r#"+http_file+x/file/BUILD"#,
                r#"x package(default_visibility = ["//visibility:public"])

exports_files(["downloaded"])

filegroup(
    name = "file",
    srcs = ["downloaded"],
)
"#,
            ),
            (r#"+http_file+x/file/downloaded"#, r#"F"#),
        ],
        requests: &[r#"/f.txt auth=Basic dTpw"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", netrc="@WS@/netrc")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"netrc"#,
            r#"# comment
macdef init
  echo hi

machine 127.0.0.1 login u password p
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Basic dTpw"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", netrc="@WS@/netrc")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"netrc"#,
            r#"machine 127.0.0.1 login "quoted user" password "p w"
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: Some(r#"Unexpected token 'user"' while reading <ws>/netrc"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", netrc="@WS@/netrc")
"#,
        flags: &[],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"netrc"#,
            r#"machine 127.0.0.1:1 login u password p
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper=@WS@/h.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[(
            r#"h.sh"#,
            r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer helpertok"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Bearer helpertok"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper=127.0.0.1=@WS@/h.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[(
            r#"h.sh"#,
            r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer helpertok"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Bearer helpertok"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper=example.com=@WS@/h.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"h.sh"#,
            r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer helpertok"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper=*.0.0.1=@WS@/h.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[(
            r#"h.sh"#,
            r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer helpertok"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Bearer helpertok"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper=*.0.1=@WS@/h.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[(
            r#"h.sh"#,
            r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer helpertok"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Bearer helpertok"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper=*.1=@WS@/h.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[(
            r#"h.sh"#,
            r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer helpertok"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Bearer helpertok"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", netrc="@WS@/netrc")
"#,
        flags: &[r#"--credential_helper=127.0.0.1=@WS@/h.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[
            (
                r#"h.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer helpertok"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"netrc"#,
                r#"machine 127.0.0.1 login u password p
"#,
            ),
        ],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Bearer helpertok"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper=127.0.0.1=@WS@/h.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[(
            r#"h.sh"#,
            r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo ''
    exit 1
    ;;
  *) echo '{}' ;;
esac
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper=127.0.0.1=@WS@/h.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[(
            r#"h.sh"#,
            r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo 'not json'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper=127.0.0.1=@WS@/h.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[(
            r#"h.sh"#,
            r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"X-Token":["a","b"],"Authorization":["Bearer t1"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Bearer t1"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper=127.0.0.1=@WS@/h.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[(
            r#"h.sh"#,
            r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper=127.0.0.1=@WS@/missing.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[
            r#"--credential_helper=@WS@/h.sh"#,
            r#"--credential_helper=127.0.0.1=@WS@/g.sh"#,
        ],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[
            (
                r#"h.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer helpertok"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"g.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer second"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
        ],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Bearer second"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[
            r#"--credential_helper=127.0.0.1=@WS@/h.sh"#,
            r#"--credential_helper=127.0.0.1=@WS@/g.sh"#,
        ],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[
            (
                r#"h.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer helpertok"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"g.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer second"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
        ],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Bearer second"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper=http://127.0.0.1=@WS@/h.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"h.sh"#,
            r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer helpertok"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: Some(
            r#"While parsing option --credential_helper=http://127.0.0.1=<ws>/h.sh: Credential helper scope 'http://127.0.0.1' must be a valid domain name with an optional leading '*.' wildcard"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper=127.0.0.1:1234=@WS@/h.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"h.sh"#,
            r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer helpertok"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: Some(
            r#"While parsing option --credential_helper=127.0.0.1:1234=<ws>/h.sh: Credential helper scope '127.0.0.1:1234' must be a valid domain name with an optional leading '*.' wildcard"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[
            r#"--credential_helper=@WS@/h.sh"#,
            r#"--credential_helper=no_such_host_x=@WS@/missing.sh"#,
        ],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"h.sh"#,
            r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer helpertok"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: Some(
            r#"While parsing option --credential_helper=no_such_host_x=<ws>/missing.sh: Credential helper scope 'no_such_host_x' must be a valid domain name with an optional leading '*.' wildcard"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper=127.0.0.1=rel/h.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"h.sh"#,
            r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer helpertok"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: Some(
            r#"Path 'rel/h.sh' must either be absolute or not contain any path separators"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper="#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"h.sh"#,
            r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer helpertok"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: Some(
            r#"While parsing option --credential_helper=: Credential helper path must not be empty"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper==@WS@/h.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[(
            r#"h.sh"#,
            r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer helpertok"]}}'
    exit 0
    ;;
  *) echo '{}' ;;
esac
"#,
        )],
        serve: &[
            (
                r#"d.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
            ),
            (
                r#"e.zip"#,
                Serve::Archive(r#"zip"#, &[(r#"top/b.txt"#, "f", r#"B"#)]),
            ),
            (r#"f.txt"#, Serve::Text(r#"F"#)),
        ],
        fetch: &[r#"@x"#],
        error: Some(
            r#"While parsing option --credential_helper==<ws>/h.sh: Credential helper scope must not be empty"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper=*.127.0.0.1=@WS@/a.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get a {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[
            (
                r#"a.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 a $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer a"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"b.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 b $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer b"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"c.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 c $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer c"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
        ],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Bearer a"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[
            r#"--credential_helper=127.0.0.1=@WS@/a.sh"#,
            r#"--credential_helper=*.0.1=@WS@/b.sh"#,
            r#"--credential_helper=@WS@/c.sh"#,
        ],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get a {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[
            (
                r#"a.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 a $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer a"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"b.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 b $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer b"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"c.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 c $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer c"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
        ],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Bearer a"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[
            r#"--credential_helper=*.0.1=@WS@/b.sh"#,
            r#"--credential_helper=*.1=@WS@/c.sh"#,
        ],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get b {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[
            (
                r#"a.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 a $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer a"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"b.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 b $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer b"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"c.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 c $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer c"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
        ],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Bearer b"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[
            r#"--credential_helper=*.1=@WS@/c.sh"#,
            r#"--credential_helper=*.0.1=@WS@/b.sh"#,
        ],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get b {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[
            (
                r#"a.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 a $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer a"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"b.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 b $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer b"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"c.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 c $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer c"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
        ],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Bearer b"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[
            r#"--credential_helper=*.1=@WS@/c.sh"#,
            r#"--credential_helper=@WS@/a.sh"#,
        ],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get c {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[
            (
                r#"a.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 a $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer a"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"b.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 b $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer b"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"c.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 c $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer c"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
        ],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Bearer c"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper=*=@WS@/a.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[
            (
                r#"a.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 a $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer a"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"b.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 b $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer b"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"c.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 c $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer c"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
        ],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
        )],
        fetch: &[r#"@x"#],
        error: Some(
            r#"While parsing option --credential_helper=*=<ws>/a.sh: Credential helper scope '*' must be a valid domain name with an optional leading '*.' wildcard"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--credential_helper=*.=@WS@/a.sh"#],
        then_module: None,
        then_fetch: &[],
        helper_log: &[],
        git: &[],
        files: &[
            (
                r#"a.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 a $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer a"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"b.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 b $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer b"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"c.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 c $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer c"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
        ],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
        )],
        fetch: &[r#"@x"#],
        error: Some(
            r#"While parsing option --credential_helper=*.=<ws>/a.sh: Credential helper scope '*.' must be a valid domain name with an optional leading '*.' wildcard"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[
            r#"--credential_helper=@WS@/a.sh"#,
            r#"--credential_helper=@WS@/b.sh"#,
        ],
        then_module: None,
        then_fetch: &[],
        helper_log: &[r#"get b {"uri":"@URL@/d.zip"}"#],
        git: &[],
        files: &[
            (
                r#"a.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 a $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer a"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"b.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 b $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer b"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
            (
                r#"c.sh"#,
                r#"#!/bin/sh
read in
case "$in" in
  *127.0.0.1*)
    echo "$1 c $in" >> @WS@/helper.log
    echo '{"headers":{"Authorization":["Bearer c"]}}'
    ;;
  *) echo '{}' ;;
esac
"#,
            ),
        ],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip auth=Bearer b"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[],
        then_module: Some(
            r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
http_archive(name="y", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        ),
        then_fetch: &[r#"@y"#],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+y/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+y/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#, r#"--then--"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[],
        then_module: Some(
            r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
http_archive(name="y", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", canonical_id="a")
"#,
        ),
        then_fetch: &[r#"@y"#],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+y/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+y/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#, r#"--then--"#, r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", canonical_id="a")
"#,
        flags: &[],
        then_module: Some(
            r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", canonical_id="a")
http_archive(name="y", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", canonical_id="a")
"#,
        ),
        then_fetch: &[r#"@y"#],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+y/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+y/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#, r#"--then--"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", canonical_id="a")
"#,
        flags: &[],
        then_module: Some(
            r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", canonical_id="a")
http_archive(name="y", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", canonical_id="b")
"#,
        ),
        then_fetch: &[r#"@y"#],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+y/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+y/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#, r#"--then--"#, r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", canonical_id="a")
"#,
        flags: &[],
        then_module: Some(
            r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x", canonical_id="a")
http_archive(name="y", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        ),
        then_fetch: &[r#"@y"#],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+y/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+y/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#, r#"--then--"#, r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--repository_cache="#],
        then_module: Some(
            r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
http_archive(name="y", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        ),
        then_fetch: &[r#"@y"#],
        helper_log: &[],
        git: &[],
        files: &[],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+y/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+y/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#, r#"--then--"#, r#"/d.zip"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--distdir=@WS@/dist"#],
        then_module: Some(
            r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
http_archive(name="y", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        ),
        then_fetch: &[r#"@y"#],
        helper_log: &[],
        git: &[],
        files: &[(r#"dist/d.zip"#, r#"@SERVE:d.zip@"#)],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+y/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+y/a.txt"#, r#"A"#),
        ],
        requests: &[r#"--then--"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--distdir=@WS@/dist"#],
        then_module: Some(
            r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
http_archive(name="y", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        ),
        then_fetch: &[r#"@y"#],
        helper_log: &[],
        git: &[],
        files: &[(r#"dist/d.zip"#, r#"not the archive"#)],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+y/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+y/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#, r#"--then--"#],
    },
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        flags: &[r#"--distdir=@WS@/dist"#],
        then_module: Some(
            r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(name="x", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
http_archive(name="y", urls=["@URL@/d.zip"], sha256="@SHA:d.zip@", strip_prefix="top", build_file_content="x")
"#,
        ),
        then_fetch: &[r#"@y"#],
        helper_log: &[],
        git: &[],
        files: &[(r#"dist/other.zip"#, r#"@SERVE:d.zip@"#)],
        serve: &[(
            r#"d.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"top/a.txt"#, "f", r#"A"#)]),
        )],
        fetch: &[r#"@x"#],
        error: None,
        printed: &[],
        tree: &[
            (r#"+http_archive+x/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+x/a.txt"#, r#"A"#),
            (r#"+http_archive+y/BUILD.bazel"#, r#"x x"#),
            (r#"+http_archive+y/a.txt"#, r#"A"#),
        ],
        requests: &[r#"/d.zip"#, r#"--then--"#],
    },
];
