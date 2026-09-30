//! Generated from Bazel 9.2.0 probes: what Bazel's own `http_archive` and `http_file` do.

use crate::http_archive_tests::{HaRow, Serve};

pub(crate) const HA_ROWS: &[HaRow] = &[
    HaRow {
        module: r#"module(name="probe")
http_archive = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_file = use_repo_rule("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")
http_archive(name="x", urls=["@URL@/d.zip"], sha256 = "@SHA:d.zip@", strip_prefix="top", build_file_content="filegroup(name='all')")
"#,
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
http_file(name="x", urls=["@URL@/f.txt"], sha256="@SHA:f.txt@")
"#,
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
];
