//! Generated from Bazel 9.2.0 probes: what repository rules that download, extract and patch do.

use crate::repo_download_tests::{DlRow, Serve};

pub(crate) const DL_ROWS: &[DlRow] = &[
    DlRow {
        bzl: r#"def _impl(ctx):
    r = ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@")
    print(r)
    print(type(r), dir(r))
    print(r.success, r.sha256, r.integrity)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[
            r#"struct(integrity = "sha256-LPJNul+wow4m6DsqxbninhsWHlwfp0JecwQzYpOLmCQ=", sha256 = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824", success = True)"#,
            r#"struct ["integrity", "sha256", "success"]"#,
            r#"True 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824 sha256-LPJNul+wow4m6DsqxbninhsWHlwfp0JecwQzYpOLmCQ="#,
        ],
        tree: &[(r#"out.txt"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download(["@URL@/nothing", "@URL@/f.txt"], "out.txt", sha256="@SHA:f.txt@")
    print(ctx.read("out.txt"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[r#"hello"#],
        tree: &[(r#"out.txt"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/nothing"#, r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", sha256="@SHA:f.txt@")
    print(ctx.path("f.txt").exists)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: Some(r#"java.io.IOException: <repo> (File exists)"#),
        printed: &[r#"False"#],
        tree: &[],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "dir/", sha256="@SHA:f.txt@")
    print(ctx.path("dir").is_dir, ctx.path("dir/f.txt").exists)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[r#"False False"#],
        tree: &[(r#"dir"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("d/x", "1")
    ctx.download("@URL@/f.txt", "d", sha256="@SHA:f.txt@")
    print(ctx.path("d/f.txt").exists)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error downloading [@URL@/f.txt] to <repo>/d: <repo>/d (Is a directory)"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "a/b/c.txt", sha256="@SHA:f.txt@")
    print(ctx.read("a/b/c.txt"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[r#"hello"#],
        tree: &[
            (r#"a"#, r#"<dir>"#),
            (r#"a/b"#, r#"<dir>"#),
            (r#"a/b/c.txt"#, r#"hello"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.sh", sha256="@SHA:f.txt@", executable=True)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[],
        tree: &[(r#"out.sh"#, r#"x hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@", executable=False)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[],
        tree: &[(r#"out.txt"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[],
        tree: &[(r#"out.txt"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "../out.txt", sha256="@SHA:f.txt@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: Some(r#"Cannot write outside of the repository directory for path <ext>/out.txt"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@")
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@")
    print(ctx.read("out.txt"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[r#"hello"#],
        tree: &[(r#"out.txt"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@", canonical_id="abc")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[],
        tree: &[(r#"out.txt"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@", headers={"X-A": "b"}, auth={})
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[],
        tree: &[(r#"out.txt"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@", headers={"X-A": ["b", "c"]})
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[],
        tree: &[(r#"out.txt"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@", auth={"@URL@/f.txt": {"type": "basic", "login": "u", "password": "p"}})
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[],
        tree: &[(r#"out.txt"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@", auth=1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: Some(
            r#"in call to download(), parameter 'auth' got value of type 'int', want 'dict'"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@", block=False)
    print(1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[r#"1"#],
        tree: &[(r#"out.txt"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    t = ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@", block=False)
    print(type(t), dir(t))
    r = t.wait()
    print(r.success, r.sha256)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[
            r#"PendingDownload ["wait"]"#,
            r#"True 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"#,
        ],
        tree: &[(r#"out.txt"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@", bogus=1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: Some(r#"download() got unexpected keyword argument 'bogus'"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@", allow_fail=1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: Some(
            r#"in call to download(), parameter 'allow_fail' got value of type 'int', want 'bool'"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@", canonical_id=1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: Some(
            r#"in call to download(), parameter 'canonical_id' got value of type 'int', want 'string'"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@".upper())
    print(ctx.read("out.txt"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[r#"hello"#],
        tree: &[(r#"out.txt"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", integrity="sha512-" + "A"*86 + "==")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error downloading [@URL@/f.txt] to <repo>/out.txt: Checksum was sha512-m3HSJL1i83hdltRq0+o9czGb+8KJDKra4t/3JRlnPKcjI8PZm6XBHXx6zG4UuMXaDEZjR1wuXDre9G9zvN7AQw== but wanted sha512-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    r = ctx.download("@URL@/f.txt", "out.txt", integrity="@INT:f.txt@")
    print(r.integrity, r.sha256)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[
            r#"sha256-LPJNul+wow4m6DsqxbninhsWHlwfp0JecwQzYpOLmCQ= 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"#,
        ],
        tree: &[(r#"out.txt"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("file:///nonexistent/x", "out.txt")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error downloading [file:///nonexistent/x] to <repo>/out.txt: /nonexistent/x (No such file or directory)"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    r = ctx.download("file:///nonexistent/x", "out.txt", allow_fail=True)
    print(r)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: None,
        printed: &[r#"struct(success = False)"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    r = ctx.download("@URL@/gone", "out.txt", sha256="@SHA:f.txt@", allow_fail=True)
    print(r)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[r#"struct(success = False)"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
        requests: &[r#"/gone"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/gone", "out.txt", sha256="@SHA:f.txt@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error downloading [@URL@/gone] to <repo>/out.txt: GET returned 404 Not Found"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/gone"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Status(500))],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error downloading [@URL@/f.txt] to <repo>/out.txt: GET returned 500 Internal Server Error"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Status(403))],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error downloading [@URL@/f.txt] to <repo>/out.txt: GET returned 403 Forbidden"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@")
    print(ctx.read("out.txt") == "")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#""#))],
        twice: false,
        error: None,
        printed: &[r#"True"#],
        tree: &[(r#"out.txt"#, r#""#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/top"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.jar", "out", sha256="@SHA:a.jar@")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.jar"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/top"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.jar"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.whl", "out", sha256="@SHA:a.whl@")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.whl"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/top"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.whl"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tar", "out", sha256="@SHA:a.tar@")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar"#,
            Serve::Archive(
                r#"tar"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/top"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.tar"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tar.gz", "out", sha256="@SHA:a.tar.gz@")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar.gz"#,
            Serve::Archive(
                r#"tar.gz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/top"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.tar.gz"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tgz", "out", sha256="@SHA:a.tgz@")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tgz"#,
            Serve::Archive(
                r#"tar.gz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/top"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.tgz"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tar.bz2", "out", sha256="@SHA:a.tar.bz2@")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar.bz2"#,
            Serve::Archive(
                r#"tar.bz2"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/top"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.tar.bz2"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tar.xz", "out", sha256="@SHA:a.tar.xz@")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar.xz"#,
            Serve::Archive(
                r#"tar.xz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/top"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.tar.xz"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tar.zst", "out", sha256="@SHA:a.tar.zst@")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar.zst"#,
            Serve::Archive(
                r#"tar.zst"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/top"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.tar.zst"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.txz", "out", sha256="@SHA:a.txz@")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.txz"#,
            Serve::Archive(
                r#"tar.xz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/top"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.txz"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tzst", "out", sha256="@SHA:a.tzst@")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tzst"#,
            Serve::Archive(
                r#"tar.zst"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/top"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.tzst"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tbz", "out", sha256="@SHA:a.tbz@")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tbz"#,
            Serve::Archive(
                r#"tar.bz2"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/top"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.tbz"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_prefix="top")
    print(sorted(ctx.path("out").readdir()))
    print(ctx.read("out/a.txt"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(r#"unsupported comparison: path <=> path"#),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tar.gz", "out", sha256="@SHA:a.tar.gz@", strip_prefix="top")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar.gz"#,
            Serve::Archive(
                r#"tar.gz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(r#"unsupported comparison: path <=> path"#),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.tar.gz"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tar.gz", "out", sha256="@SHA:a.tar.gz@", strip_prefix="top/sub")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar.gz"#,
            Serve::Archive(
                r#"tar.gz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/b.txt"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.tar.gz"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_prefix="nothere")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error extracting <repo>/out/tempN/a.zip to <repo>/out/tempN: Prefix "nothere" was given, but not found in the archive. Here are possible prefixes for this archive: "top"."#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tar.gz", "out", sha256="@SHA:a.tar.gz@", strip_prefix="nothere")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar.gz"#,
            Serve::Archive(
                r#"tar.gz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error extracting <repo>/out/tempN/a.tar.gz to <repo>/out/tempN: Prefix "nothere" was given, but not found in the archive. Here are possible prefixes for this archive: "top"."#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.tar.gz"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_prefix="top/")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(r#"unsupported comparison: path <=> path"#),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_prefix="/top")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(r#"unsupported comparison: path <=> path"#),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", rename_files={"top/a.txt": "top/renamed.txt"})
    print(sorted(ctx.path("out/top").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(r#"unsupported comparison: path <=> path"#),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", rename_files={"nothere": "x"})
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/top"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_prefix="top", rename_files={"a.txt": "moved.txt"})
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(r#"unsupported comparison: path <=> path"#),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/blob", "out", sha256="@SHA:blob@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"blob"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(
            r#"Expected a file with a .zip, .jar, .war, .aar, .nupkg, .whl, .tar, .tar.gz, .tgz, .gz, .tar.xz, .txz, .xz, .tar.zst, .tzst, .zst, .tar.bz2, .tbz, .bz2, .ar, .deb or .7z suffix (got <repo>/out/tempN/blob)"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/blob"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/blob", "out", sha256="@SHA:blob@", type="zip")
    print(sorted(ctx.path("out/top").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"blob"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(r#"unsupported comparison: path <=> path"#),
        printed: &[],
        tree: &[],
        requests: &[r#"/blob"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/blob.dat", "out", sha256="@SHA:blob.dat@", type="tar.gz")
    print(sorted(ctx.path("out/top").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"blob.dat"#,
            Serve::Archive(
                r#"tar.gz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(r#"unsupported comparison: path <=> path"#),
        printed: &[],
        tree: &[],
        requests: &[r#"/blob.dat"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/blob.dat", "out", sha256="@SHA:blob.dat@", type="bogus")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"blob.dat"#,
            Serve::Archive(
                r#"tar.gz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(
            r#"Expected a file with a .zip, .jar, .war, .aar, .nupkg, .whl, .tar, .tar.gz, .tgz, .gz, .tar.xz, .txz, .xz, .tar.zst, .tzst, .zst, .tar.bz2, .tbz, .bz2, .ar, .deb or .7z suffix (got <repo>/out/tempN/blob.dat.bogus)"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/blob.dat"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip?x=1", "out", sha256="@SHA:a.zip@")
    print(sorted(ctx.path("out/top").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(r#"unsupported comparison: path <=> path"#),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.zip?x=1"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", sha256="@SHA:a.zip@")
    print(sorted(ctx.path("top").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(r#"unsupported comparison: path <=> path"#),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    r = ctx.download_and_extract("@URL@/nothing.zip", "out", sha256="0000000000000000000000000000000000000000000000000000000000000000", allow_fail=True)
    print(r)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: None,
        printed: &[r#"struct(success = False)"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/tempN"#, r#"<dir>"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/nothing.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@")
    print(ctx.path("out/a.zip").exists)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"False"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/bad.zip", "out", sha256="@SHA:bad.zip@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"bad.zip"#, Serve::Text(r#"not a zip"#))],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error extracting <repo>/out/tempN/bad.zip to <repo>/out/tempN: Zip file 'bad.zip' is malformed. It does not contain an end of central directory record."#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/bad.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/bad.tar.gz", "out", sha256="@SHA:bad.tar.gz@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"bad.tar.gz"#, Serve::Text(r#"not gzip"#))],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error extracting <repo>/out/tempN/bad.tar.gz to <repo>/out/tempN: Input is not in the .gz format"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/bad.tar.gz"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"../escape.txt"#, "f", r#"x"#)]),
        )],
        twice: false,
        error: None,
        printed: &[],
        tree: &[
            (r#"escape.txt"#, r#"x"#),
            (r#"out"#, r#"<dir>"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"/abs.txt"#, "f", r#"x"#)]),
        )],
        twice: false,
        error: None,
        printed: &[],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/abs.txt"#, r#"x"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tar", "out", sha256="@SHA:a.tar@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar"#,
            Serve::Archive(r#"tar"#, &[(r#"../escape.txt"#, "f", r#"x"#)]),
        )],
        twice: false,
        error: None,
        printed: &[],
        tree: &[
            (r#"escape.txt"#, r#"x"#),
            (r#"out"#, r#"<dir>"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.tar"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tar", "out", sha256="@SHA:a.tar@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar"#,
            Serve::Archive(r#"tar"#, &[(r#"/abs.txt"#, "f", r#"x"#)]),
        )],
        twice: false,
        error: None,
        printed: &[],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/abs.txt"#, r#"x"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.tar"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tar", "out", sha256="@SHA:a.tar@")
    print(sorted(ctx.path("out").readdir()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar"#,
            Serve::Archive(
                r#"tar"#,
                &[
                    (r#"l"#, "l", r#"../../etc/passwd"#),
                    (r#"m"#, "l", r#"/etc/passwd"#),
                    (r#"n"#, "h", r#"l"#),
                ],
            ),
        )],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error extracting <repo>/out/tempN/a.tar to <repo>/out/tempN: File "l" linked from "n" does not exist"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.tar"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tar", "out", sha256="@SHA:a.tar@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar"#,
            Serve::Archive(r#"tar"#, &[(r#"x"#, "f", r#"1"#)]),
        )],
        twice: false,
        error: None,
        printed: &[],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/x"#, r#"1"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.tar"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("out/x", "old")
    ctx.download_and_extract("@URL@/a.tar", "out", sha256="@SHA:a.tar@")
    print(ctx.read("out/x"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar"#,
            Serve::Archive(r#"tar"#, &[(r#"x"#, "f", r#"new"#)]),
        )],
        twice: false,
        error: None,
        printed: &[r#"new"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/x"#, r#"new"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.tar"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("out/x/y", "old")
    ctx.download_and_extract("@URL@/a.tar", "out", sha256="@SHA:a.tar@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar"#,
            Serve::Archive(r#"tar"#, &[(r#"x"#, "f", r#"new"#)]),
        )],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error extracting <repo>/out/tempN/a.tar to <repo>/out/tempN: <repo>/out/x (Is a directory)"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.tar"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tar", "out", sha256="@SHA:a.tar@")
    print(ctx.read("out/x"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar"#,
            Serve::Archive(r#"tar"#, &[(r#"x"#, "f", r#"1"#), (r#"y/"#, "d", "")]),
        )],
        twice: false,
        error: None,
        printed: &[r#"1"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/x"#, r#"1"#),
            (r#"out/y"#, r#"<dir>"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.tar"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tar", "../out", sha256="@SHA:a.tar@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar"#,
            Serve::Archive(r#"tar"#, &[(r#"x"#, "f", r#"1"#)]),
        )],
        twice: false,
        error: Some(r#"Cannot write outside of the repository directory for path <ext>/out"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@")
    print(ctx.read("out/some dir/f é.txt"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"some dir/f é.txt"#, "f", r#"x"#)]),
        )],
        twice: false,
        error: None,
        printed: &[r#"x"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/some dir"#, r#"<dir>"#),
            (r#"out/some dir/f é.txt"#, r#"x"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_prefix="top")
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/a.txt", "<repo>/out/empty", "<repo>/out/link", "<repo>/out/sub"]"#,
        ],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/a.txt"#, r#"A"#),
            (r#"out/empty"#, r#"<dir>"#),
            (r#"out/link"#, r#"-> a.txt"#),
            (r#"out/sub"#, r#"<dir>"#),
            (r#"out/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tar.gz", "out", sha256="@SHA:a.tar.gz@", strip_prefix="top")
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar.gz"#,
            Serve::Archive(
                r#"tar.gz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/a.txt", "<repo>/out/empty", "<repo>/out/link", "<repo>/out/sub"]"#,
        ],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/a.txt"#, r#"A"#),
            (r#"out/empty"#, r#"<dir>"#),
            (r#"out/link"#, r#"-> a.txt"#),
            (r#"out/sub"#, r#"<dir>"#),
            (r#"out/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.tar.gz"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_prefix="top/")
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/a.txt", "<repo>/out/empty", "<repo>/out/link", "<repo>/out/sub"]"#,
        ],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/a.txt"#, r#"A"#),
            (r#"out/empty"#, r#"<dir>"#),
            (r#"out/link"#, r#"-> a.txt"#),
            (r#"out/sub"#, r#"<dir>"#),
            (r#"out/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_prefix="/top")
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/a.txt", "<repo>/out/empty", "<repo>/out/link", "<repo>/out/sub"]"#,
        ],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/a.txt"#, r#"A"#),
            (r#"out/empty"#, r#"<dir>"#),
            (r#"out/link"#, r#"-> a.txt"#),
            (r#"out/sub"#, r#"<dir>"#),
            (r#"out/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", rename_files={"top/a.txt": "top/renamed.txt"})
    print(sorted([str(p) for p in ctx.path("out/top").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/top/empty", "<repo>/out/top/link", "<repo>/out/top/renamed.txt", "<repo>/out/top/sub"]"#,
        ],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/renamed.txt"#, r#"A"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_prefix="top", rename_files={"a.txt": "moved.txt"})
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/a.txt", "<repo>/out/empty", "<repo>/out/link", "<repo>/out/sub"]"#,
        ],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/a.txt"#, r#"A"#),
            (r#"out/empty"#, r#"<dir>"#),
            (r#"out/link"#, r#"-> a.txt"#),
            (r#"out/sub"#, r#"<dir>"#),
            (r#"out/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_prefix="top", rename_files={"top/a.txt": "moved.txt"})
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/empty", "<repo>/out/link", "<repo>/out/sub"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/empty"#, r#"<dir>"#),
            (r#"out/link"#, r#"-> a.txt"#),
            (r#"out/sub"#, r#"<dir>"#),
            (r#"out/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", rename_files={"top/a.txt": "../x"})
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/top"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"x"#, r#"A"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", rename_files={"top/a.txt": 1})
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(r#"got dict<string, int> for 'rename_files', want dict<string, string>"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_prefix="nothere")
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error extracting <repo>/out/tempN/a.zip to <repo>/out/tempN: Prefix "nothere" was given, but not found in the archive. Here are possible prefixes for this archive: "top"."#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_prefix="to")
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error extracting <repo>/out/tempN/a.zip to <repo>/out/tempN: Prefix "to" was given, but not found in the archive. Here are possible prefixes for this archive: "top"."#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tar.gz", "out", sha256="@SHA:a.tar.gz@", strip_prefix="top/su")
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar.gz"#,
            Serve::Archive(
                r#"tar.gz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error extracting <repo>/out/tempN/a.tar.gz to <repo>/out/tempN: Prefix "top/su" was given, but not found in the archive. Here are possible prefixes for this archive: "top"."#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.tar.gz"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/blob", "out", sha256="@SHA:blob@", type="zip")
    print(sorted([str(p) for p in ctx.path("out/top").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"blob"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/top/a.txt", "<repo>/out/top/empty", "<repo>/out/top/link", "<repo>/out/top/sub"]"#,
        ],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/blob"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/blob.dat", "out", sha256="@SHA:blob.dat@", type="tar.gz")
    print(sorted([str(p) for p in ctx.path("out/top").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"blob.dat"#,
            Serve::Archive(
                r#"tar.gz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/top/a.txt", "<repo>/out/top/empty", "<repo>/out/top/link", "<repo>/out/top/sub"]"#,
        ],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/blob.dat"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/blob.dat", "out", sha256="@SHA:blob.dat@", type=".tar.gz")
    print(sorted([str(p) for p in ctx.path("out/top").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"blob.dat"#,
            Serve::Archive(
                r#"tar.gz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/top/a.txt", "<repo>/out/top/empty", "<repo>/out/top/link", "<repo>/out/top/sub"]"#,
        ],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/blob.dat"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/blob.dat", "out", sha256="@SHA:blob.dat@", type="TAR.GZ")
    print(sorted([str(p) for p in ctx.path("out/top").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"blob.dat"#,
            Serve::Archive(
                r#"tar.gz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(
            r#"Expected a file with a .zip, .jar, .war, .aar, .nupkg, .whl, .tar, .tar.gz, .tgz, .gz, .tar.xz, .txz, .xz, .tar.zst, .tzst, .zst, .tar.bz2, .tbz, .bz2, .ar, .deb or .7z suffix (got <repo>/out/tempN/blob.dat.TAR.GZ)"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/blob.dat"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip?x=1", "out", sha256="@SHA:a.zip@")
    print(sorted([str(p) for p in ctx.path("out/top").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/top/a.txt", "<repo>/out/top/empty", "<repo>/out/top/link", "<repo>/out/top/sub"]"#,
        ],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip?x=1"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", sha256="@SHA:a.zip@")
    print(sorted([str(p) for p in ctx.path("top").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/top/a.txt", "<repo>/top/empty", "<repo>/top/link", "<repo>/top/sub"]"#,
        ],
        tree: &[
            (r#"top"#, r#"<dir>"#),
            (r#"top/a.txt"#, r#"A"#),
            (r#"top/empty"#, r#"<dir>"#),
            (r#"top/link"#, r#"-> a.txt"#),
            (r#"top/sub"#, r#"<dir>"#),
            (r#"top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.deb", "out", sha256="@SHA:a.deb@")
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.deb"#,
            Serve::Archive(
                r#"deb:tar.xz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/control.tar.gz", "<repo>/out/data.tar.xz", "<repo>/out/debian-binary"]"#,
        ],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/control.tar.gz"#, r#"<binary>"#),
            (r#"out/data.tar.xz"#, r#"<binary>"#),
            (
                r#"out/debian-binary"#,
                r#"2.0
"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.deb"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.deb", "out", sha256="@SHA:a.deb@")
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.deb"#,
            Serve::Archive(
                r#"deb:tar.gz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/control.tar.gz", "<repo>/out/data.tar.gz", "<repo>/out/debian-binary"]"#,
        ],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/control.tar.gz"#, r#"<binary>"#),
            (r#"out/data.tar.gz"#, r#"<binary>"#),
            (
                r#"out/debian-binary"#,
                r#"2.0
"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.deb"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.deb", "out", sha256="@SHA:a.deb@")
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.deb"#,
            Serve::Archive(
                r#"deb:tar.zst"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/control.tar.gz", "<repo>/out/data.tar.zst", "<repo>/out/debian-binary"]"#,
        ],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/control.tar.gz"#, r#"<binary>"#),
            (r#"out/data.tar.zst"#, r#"<binary>"#),
            (
                r#"out/debian-binary"#,
                r#"2.0
"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.deb"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.gz", "out", sha256="@SHA:a.gz@")
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.gz"#,
            Serve::Hex(r#"1f8b0800000000000003cb48cdc9c907002d3b08af05000000"#),
        )],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error extracting <repo>/out/tempN/a.gz to <repo>/out/tempN: Gzip-compressed data is corrupt (CRC32 error)"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.gz"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.ar", "out", sha256="@SHA:a.ar@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.ar"#,
            Serve::Text(
                r#"!<arch>
"#,
            ),
        )],
        twice: false,
        error: None,
        printed: &[],
        tree: &[(r#"out"#, r#"<dir>"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/a.ar"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/a.zip", "a.zip", sha256="@SHA:a.zip@")
    ctx.extract("a.zip")
    print(sorted([str(p) for p in ctx.path("top").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/top/a.txt", "<repo>/top/empty", "<repo>/top/link", "<repo>/top/sub"]"#,
        ],
        tree: &[
            (r#"a.zip"#, r#"<binary>"#),
            (r#"top"#, r#"<dir>"#),
            (r#"top/a.txt"#, r#"A"#),
            (r#"top/empty"#, r#"<dir>"#),
            (r#"top/link"#, r#"-> a.txt"#),
            (r#"top/sub"#, r#"<dir>"#),
            (r#"top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/a.zip", "a.zip", sha256="@SHA:a.zip@")
    ctx.extract("a.zip", "out", "top")
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/a.txt", "<repo>/out/empty", "<repo>/out/link", "<repo>/out/sub"]"#,
        ],
        tree: &[
            (r#"a.zip"#, r#"<binary>"#),
            (r#"out"#, r#"<dir>"#),
            (r#"out/a.txt"#, r#"A"#),
            (r#"out/empty"#, r#"<dir>"#),
            (r#"out/link"#, r#"-> a.txt"#),
            (r#"out/sub"#, r#"<dir>"#),
            (r#"out/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/a.zip", "a.zip", sha256="@SHA:a.zip@")
    ctx.extract("a.zip", output="out", strip_prefix="top", rename_files={"a.txt": "z.txt"})
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/a.txt", "<repo>/out/empty", "<repo>/out/link", "<repo>/out/sub"]"#,
        ],
        tree: &[
            (r#"a.zip"#, r#"<binary>"#),
            (r#"out"#, r#"<dir>"#),
            (r#"out/a.txt"#, r#"A"#),
            (r#"out/empty"#, r#"<dir>"#),
            (r#"out/link"#, r#"-> a.txt"#),
            (r#"out/sub"#, r#"<dir>"#),
            (r#"out/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.extract("missing.zip")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(r#"Archive path '<repo>/missing.zip' does not exist."#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("x.zip", "not a zip")
    ctx.extract("x.zip")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error extracting <repo>/x.zip to <repo>: Zip file 'x.zip' is malformed. It does not contain an end of central directory record."#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("x.txt", "hello")
    ctx.extract("x.txt")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(
            r#"Expected a file with a .zip, .jar, .war, .aar, .nupkg, .whl, .tar, .tar.gz, .tgz, .gz, .tar.xz, .txz, .xz, .tar.zst, .tzst, .zst, .tar.bz2, .tbz, .bz2, .ar, .deb or .7z suffix (got <repo>/x.txt)"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/a.zip", "a.zip", sha256="@SHA:a.zip@")
    print(ctx.extract("a.zip"))
    print(ctx.path("a.zip").exists)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"None"#, r#"True"#],
        tree: &[
            (r#"a.zip"#, r#"<binary>"#),
            (r#"top"#, r#"<dir>"#),
            (r#"top/a.txt"#, r#"A"#),
            (r#"top/empty"#, r#"<dir>"#),
            (r#"top/link"#, r#"-> a.txt"#),
            (r#"top/sub"#, r#"<dir>"#),
            (r#"top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/a.zip", "a.zip", sha256="@SHA:a.zip@")
    ctx.extract(ctx.path("a.zip"), "out")
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/top"]"#],
        tree: &[
            (r#"a.zip"#, r#"<binary>"#),
            (r#"out"#, r#"<dir>"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/a.zip", "a.zip", sha256="@SHA:a.zip@")
    ctx.extract("a.zip", watch_archive="no")
    ctx.extract("a.zip", watch_archive="bogus")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: Some(r#"bad value for 'watch' parameter; want 'yes', 'no', or 'auto', got bogus"#),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/a.zip", "a.zip", sha256="@SHA:a.zip@")
    ctx.extract(archive="a.zip")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(
                r#"zip"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[],
        tree: &[
            (r#"a.zip"#, r#"<binary>"#),
            (r#"top"#, r#"<dir>"#),
            (r#"top/a.txt"#, r#"A"#),
            (r#"top/empty"#, r#"<dir>"#),
            (r#"top/link"#, r#"-> a.txt"#),
            (r#"top/sub"#, r#"<dir>"#),
            (r#"top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/a.tar.gz", "a.tar.gz", sha256="@SHA:a.tar.gz@")
    ctx.extract("a.tar.gz", "out", strip_prefix="top", rename_files={"a.txt": "z.txt"})
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.tar.gz"#,
            Serve::Archive(
                r#"tar.gz"#,
                &[
                    (r#"top/a.txt"#, "f", r#"A"#),
                    (r#"top/sub/b.txt"#, "x", r#"B"#),
                    (r#"top/sub/"#, "d", ""),
                    (r#"top/link"#, "l", r#"a.txt"#),
                    (r#"top/empty/"#, "d", ""),
                ],
            ),
        )],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/a.txt", "<repo>/out/empty", "<repo>/out/link", "<repo>/out/sub"]"#,
        ],
        tree: &[
            (r#"a.tar.gz"#, r#"<binary>"#),
            (r#"out"#, r#"<dir>"#),
            (r#"out/a.txt"#, r#"A"#),
            (r#"out/empty"#, r#"<dir>"#),
            (r#"out/link"#, r#"-> a.txt"#),
            (r#"out/sub"#, r#"<dir>"#),
            (r#"out/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.tar.gz"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.extract(1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(
            r#"in call to extract(), parameter 'archive' got value of type 'int', want 'string, Label, or path'"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.extract("a.zip", 1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(
            r#"in call to extract(), parameter 'output' got value of type 'int', want 'string, Label, or path'"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.extract("a.zip", "out", 1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(
            r#"in call to extract(), parameter 'strip_prefix' got value of type 'int', want 'string'"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("f.txt", "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\n")
    ctx.file("x.patch", """--- a/f.txt\n+++ b/f.txt\n@@ -2,3 +2,3 @@\n line2\n-line3\n+LINE3\n line4\n""")
    ctx.patch("x.patch", strip=1)
    print(ctx.read("f.txt"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: None,
        printed: &[r#"line1"#],
        tree: &[
            (
                r#"f.txt"#,
                r#"x line1
line2
LINE3
line4
line5
line6
line7
line8
line9
line10
"#,
            ),
            (
                r#"x.patch"#,
                r#"x --- a/f.txt
+++ b/f.txt
@@ -2,3 +2,3 @@
 line2
-line3
+LINE3
 line4
"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("f.txt", "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\n")
    ctx.file("x.patch", "--- a/f.txt\n+++ b/f.txt\n@@ -2,3 +2,3 @@\n line2\n-line3\n+LINE3\n line4\n")
    ctx.patch("x.patch", 1)
    print(ctx.read("f.txt"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: None,
        printed: &[r#"line1"#],
        tree: &[
            (
                r#"f.txt"#,
                r#"x line1
line2
LINE3
line4
line5
line6
line7
line8
line9
line10
"#,
            ),
            (
                r#"x.patch"#,
                r#"x --- a/f.txt
+++ b/f.txt
@@ -2,3 +2,3 @@
 line2
-line3
+LINE3
 line4
"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("f.txt", "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\n")
    ctx.file("x.patch", "--- a/f.txt\n+++ b/f.txt\n@@ -2,3 +2,3 @@\n line2\n-line3\n+LINE3\n line4\n")
    ctx.patch("x.patch")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(
            r#"Error applying patch <repo>/x.patch: Cannot find file to patch (near line 1), old file name (a/f.txt) doesn't exist, new file name (b/f.txt) doesn't exist."#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("f.txt", "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\n")
    ctx.file("x.patch", "--- a/f.txt\n+++ b/f.txt\n@@ -2,3 +2,3 @@\n line2\n-line3\n+LINE3\n line4\n".replace("a/f.txt", "f.txt").replace("b/f.txt", "f.txt"))
    ctx.patch("x.patch", strip=0)
    print(ctx.read("f.txt"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: None,
        printed: &[r#"line1"#],
        tree: &[
            (
                r#"f.txt"#,
                r#"x line1
line2
LINE3
line4
line5
line6
line7
line8
line9
line10
"#,
            ),
            (
                r#"x.patch"#,
                r#"x --- f.txt
+++ f.txt
@@ -2,3 +2,3 @@
 line2
-line3
+LINE3
 line4
"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("f.txt", "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\n")
    ctx.file("x.patch", "--- a/f.txt\n+++ b/f.txt\n@@ -2,3 +2,3 @@\n line2\n-line3\n+LINE3\n line4\n")
    ctx.patch("x.patch", strip=2)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(
            r#"Error applying patch <repo>/x.patch: Cannot determine file name with strip = 2 at line 1:"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("f.txt", "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\n")
    ctx.file("x.patch", "--- a/f.txt\n+++ b/f.txt\n@@ -4,3 +4,3 @@\n line2\n-line3\n+LINE3\n line4\n")
    ctx.patch("x.patch", strip=1)
    print(ctx.read("f.txt"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: None,
        printed: &[r#"line1"#],
        tree: &[
            (
                r#"f.txt"#,
                r#"x line1
line2
LINE3
line4
line5
line6
line7
line8
line9
line10
"#,
            ),
            (
                r#"x.patch"#,
                r#"x --- a/f.txt
+++ b/f.txt
@@ -4,3 +4,3 @@
 line2
-line3
+LINE3
 line4
"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.patch("missing.patch")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(
            r#"Error applying patch <repo>/missing.patch: Cannot find patch file: <repo>/missing.patch"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("x.patch", "")
    ctx.patch("x.patch")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: None,
        printed: &[],
        tree: &[(r#"x.patch"#, r#"x "#), (r#"z.marker"#, r#"x "#)],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("x.patch", "not a patch\n")
    ctx.patch("x.patch")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: None,
        printed: &[],
        tree: &[
            (
                r#"x.patch"#,
                r#"x not a patch
"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("x.patch", "--- /dev/null\n+++ b/new.txt\n@@ -0,0 +1,2 @@\n+hello\n+world\n")
    ctx.patch("x.patch", strip=1)
    print(ctx.read("new.txt"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: None,
        printed: &[r#"hello"#],
        tree: &[
            (
                r#"new.txt"#,
                r#"hello
world
"#,
            ),
            (
                r#"x.patch"#,
                r#"x --- /dev/null
+++ b/new.txt
@@ -0,0 +1,2 @@
+hello
+world
"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("old.txt", "a\nb\n")
    ctx.file("x.patch", "--- a/old.txt\n+++ /dev/null\n@@ -1,2 +0,0 @@\n-a\n-b\n")
    ctx.patch("x.patch", strip=1)
    print(ctx.path("old.txt").exists)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: None,
        printed: &[r#"False"#],
        tree: &[
            (
                r#"x.patch"#,
                r#"x --- a/old.txt
+++ /dev/null
@@ -1,2 +0,0 @@
-a
-b
"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a.txt", "1\n")
    ctx.file("b.txt", "2\n")
    ctx.file("x.patch", "--- a/a.txt\n+++ b/a.txt\n@@ -1 +1 @@\n-1\n+one\n--- a/b.txt\n+++ b/b.txt\n@@ -1 +1 @@\n-2\n+two\n")
    ctx.patch("x.patch", strip=1)
    print(ctx.read("a.txt"), ctx.read("b.txt"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: None,
        printed: &[r#"1"#],
        tree: &[
            (
                r#"a.txt"#, r#"x 1
"#,
            ),
            (
                r#"b.txt"#,
                r#"x two
"#,
            ),
            (
                r#"x.patch"#,
                r#"x --- a/a.txt
+++ b/a.txt
@@ -1 +1 @@
-1
+one
--- a/b.txt
+++ b/b.txt
@@ -1 +1 @@
-2
+two
"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a.txt", "1\n")
    ctx.file("x.patch", "diff --git a/a.txt b/a.txt\nindex 111..222 100644\n--- a/a.txt\n+++ b/a.txt\n@@ -1 +1 @@\n-1\n+one\n")
    ctx.patch("x.patch", strip=1)
    print(ctx.read("a.txt"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: None,
        printed: &[r#"one"#],
        tree: &[
            (
                r#"a.txt"#,
                r#"x one
"#,
            ),
            (
                r#"x.patch"#,
                r#"x diff --git a/a.txt b/a.txt
index 111..222 100644
--- a/a.txt
+++ b/a.txt
@@ -1 +1 @@
-1
+one
"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a.txt", "1\n")
    ctx.file("x.patch", "diff --git a/a.txt b/b.txt\nsimilarity index 100%\nrename from a.txt\nrename to b.txt\n")
    ctx.patch("x.patch", strip=1)
    print(ctx.path("a.txt").exists, ctx.path("b.txt").exists)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: None,
        printed: &[r#"False True"#],
        tree: &[
            (
                r#"b.txt"#, r#"x 1
"#,
            ),
            (
                r#"x.patch"#,
                r#"x diff --git a/a.txt b/b.txt
similarity index 100%
rename from a.txt
rename to b.txt
"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("x.patch", "diff --git a/n.sh b/n.sh\nnew file mode 100755\n--- /dev/null\n+++ b/n.sh\n@@ -0,0 +1 @@\n+echo\n")
    ctx.patch("x.patch", strip=1)
    print(ctx.read("n.sh"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: None,
        printed: &[r#"echo"#],
        tree: &[
            (
                r#"n.sh"#,
                r#"x echo
"#,
            ),
            (
                r#"x.patch"#,
                r#"x diff --git a/n.sh b/n.sh
new file mode 100755
--- /dev/null
+++ b/n.sh
@@ -0,0 +1 @@
+echo
"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a.txt", "1\n2")
    ctx.file("x.patch", "--- a/a.txt\n+++ b/a.txt\n@@ -1,2 +1,2 @@\n 1\n-2\n\\ No newline at end of file\n+two\n\\ No newline at end of file\n")
    ctx.patch("x.patch", strip=1)
    print(repr(ctx.read("a.txt")))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(r#"Error applying patch <repo>/x.patch: Expecting more chunk line at line 6"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a.txt", "1\n")
    ctx.file("x.patch", "--- a/a.txt\n+++ b/a.txt\n@@ -1 +1 @@\n-1\n+one\n")
    ctx.patch(ctx.path("x.patch"), strip=1)
    print(ctx.read("a.txt"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: None,
        printed: &[r#"one"#],
        tree: &[
            (
                r#"a.txt"#,
                r#"x one
"#,
            ),
            (
                r#"x.patch"#,
                r#"x --- a/a.txt
+++ b/a.txt
@@ -1 +1 @@
-1
+one
"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.patch(1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(
            r#"in call to patch(), parameter 'patch_file' got value of type 'int', want 'string, Label, or path'"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("x.patch", "")
    ctx.patch("x.patch", strip="1")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(
            r#"in call to patch(), parameter 'strip' got value of type 'string', want 'int'"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("x.patch", "")
    ctx.patch("x.patch", strip=-1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: None,
        printed: &[],
        tree: &[(r#"x.patch"#, r#"x "#), (r#"z.marker"#, r#"x "#)],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("x.patch", "--- a/zz/missing.txt\n+++ b/zz/missing.txt\n@@ -1 +1 @@\n-1\n+2\n")
    ctx.patch("x.patch", strip=1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(
            r#"Error applying patch <repo>/x.patch: Cannot find file to patch (near line 1), old file name (zz/missing.txt) doesn't exist, new file name (zz/missing.txt) doesn't exist."#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a.txt", "1\n")
    ctx.file("x.patch", "--- a/a.txt\n+++ b/a.txt\n@@ -1 +1 @@\n-1\n+one\n")
    ctx.patch("x.patch", strip=1, watch_patch="no")
    ctx.patch("x.patch", strip=1, watch_patch="bogus")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(r#"bad value for 'watch' parameter; want 'yes', 'no', or 'auto', got bogus"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a.txt", "1\n")
    ctx.file("x.patch", "--- a/../a.txt\n+++ b/../a.txt\n@@ -1 +1 @@\n-1\n+one\n")
    ctx.patch("x.patch", strip=1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(
            r#"Error applying patch <repo>/x.patch: Cannot patch file outside of external repository (<repo>), file path = "../a.txt" at line 1"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download(url="@URL@/f.txt", output="out.txt", sha256="@SHA:f.txt@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[],
        tree: &[(r#"out.txt"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", "@SHA:f.txt@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[],
        tree: &[(r#"out.txt"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", "@SHA:f.txt@", True, False, "cid", {}, {})
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[],
        tree: &[(r#"out.txt"#, r#"x hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", "@SHA:f.txt@", True, False, "cid", {}, {}, "")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: Some(r#"download() accepts no more than 8 positional arguments but got 9"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", "@SHA:f.txt@", True, False, "cid", {}, {}, "", True, 1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: Some(r#"download() accepts no more than 8 positional arguments but got 11"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract(url="@URL@/a.zip", output="out", sha256="@SHA:a.zip@")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"x"#, "f", r#"1"#)]),
        )],
        twice: false,
        error: None,
        printed: &[],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/x"#, r#"1"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", "@SHA:a.zip@", "zip", "", False, "", {}, {}, "", {})
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"x"#, "f", r#"1"#)]),
        )],
        twice: false,
        error: Some(
            r#"download_and_extract() accepts no more than 9 positional arguments but got 11"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", "@SHA:a.zip@", "zip", "", False, "", {}, {}, "", {}, 1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"x"#, "f", r#"1"#)]),
        )],
        twice: false,
        error: Some(
            r#"download_and_extract() accepts no more than 9 positional arguments but got 12"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("x.patch", "")
    ctx.patch(patch_file="x.patch")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(r#"patch() got named argument for positional-only parameter 'patch_file'"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("x.patch", "")
    ctx.patch("x.patch", 0, "auto")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(r#"patch() accepts no more than 2 positional arguments but got 3"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.file("x.patch", "")
    ctx.patch("x.patch", 0, "auto", 1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(r#"patch() accepts no more than 2 positional arguments but got 4"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.extract("a.zip", "out", "", {}, "auto", 1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[],
        twice: false,
        error: Some(r#"extract() accepts no more than 3 positional arguments but got 6"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@", allow_fail=True)
    print(ctx.read("out.txt"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[r#"hello"#],
        tree: &[(r#"out.txt"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    r = ctx.download("@URL@/f.txt", "out.txt", sha256="@SHA:f.txt@", block=False)
    print(r.wait())
    print(r.wait())
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: None,
        printed: &[
            r#"struct(integrity = "sha256-LPJNul+wow4m6DsqxbninhsWHlwfp0JecwQzYpOLmCQ=", sha256 = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824", success = True)"#,
            r#"struct(integrity = "sha256-LPJNul+wow4m6DsqxbninhsWHlwfp0JecwQzYpOLmCQ=", sha256 = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824", success = True)"#,
        ],
        tree: &[(r#"out.txt"#, r#"hello"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/f.txt"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    r = ctx.download("@URL@/gone", "out.txt", sha256="@SHA:f.txt@", block=False)
    print(1)
    r.wait()
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(r#"f.txt"#, Serve::Text(r#"hello"#))],
        twice: false,
        error: Some(
            r#"java.io.IOException: Error downloading [@URL@/gone] to <repo>/out.txt: GET returned 404 Not Found"#,
        ),
        printed: &[r#"1"#],
        tree: &[],
        requests: &[r#"/gone"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    r = ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", block=False)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[(
            r#"a.zip"#,
            Serve::Archive(r#"zip"#, &[(r#"x"#, "f", r#"1"#)]),
        )],
        twice: false,
        error: Some(r#"download_and_extract() got unexpected keyword argument 'block'"#),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_components=1)
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[
            (
                r#"a.zip"#,
                Serve::Archive(
                    r#"zip"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
            (
                r#"a.tar.gz"#,
                Serve::Archive(
                    r#"tar.gz"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
        ],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/a.txt", "<repo>/out/empty", "<repo>/out/link", "<repo>/out/sub"]"#,
        ],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/a.txt"#, r#"A"#),
            (r#"out/empty"#, r#"<dir>"#),
            (r#"out/link"#, r#"-> a.txt"#),
            (r#"out/sub"#, r#"<dir>"#),
            (r#"out/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.tar.gz", "out", sha256="@SHA:a.tar.gz@", strip_components=1)
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[
            (
                r#"a.zip"#,
                Serve::Archive(
                    r#"zip"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
            (
                r#"a.tar.gz"#,
                Serve::Archive(
                    r#"tar.gz"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
        ],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/a.txt", "<repo>/out/empty", "<repo>/out/link", "<repo>/out/sub"]"#,
        ],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/a.txt"#, r#"A"#),
            (r#"out/empty"#, r#"<dir>"#),
            (r#"out/link"#, r#"-> a.txt"#),
            (r#"out/sub"#, r#"<dir>"#),
            (r#"out/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.tar.gz"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_components=0)
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[
            (
                r#"a.zip"#,
                Serve::Archive(
                    r#"zip"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
            (
                r#"a.tar.gz"#,
                Serve::Archive(
                    r#"tar.gz"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
        ],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/root.txt", "<repo>/out/top"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/root.txt"#, r#"R"#),
            (r#"out/top"#, r#"<dir>"#),
            (r#"out/top/a.txt"#, r#"A"#),
            (r#"out/top/empty"#, r#"<dir>"#),
            (r#"out/top/link"#, r#"-> a.txt"#),
            (r#"out/top/sub"#, r#"<dir>"#),
            (r#"out/top/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_components=-1)
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[
            (
                r#"a.zip"#,
                Serve::Archive(
                    r#"zip"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
            (
                r#"a.tar.gz"#,
                Serve::Archive(
                    r#"tar.gz"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
        ],
        twice: false,
        error: Some(
            r#"download_and_extract() has an invalid argument for 'strip_components': -1. Must be non-negative."#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_components="1")
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[
            (
                r#"a.zip"#,
                Serve::Archive(
                    r#"zip"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
            (
                r#"a.tar.gz"#,
                Serve::Archive(
                    r#"tar.gz"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
        ],
        twice: false,
        error: Some(
            r#"in call to download_and_extract(), parameter 'strip_components' got value of type 'string', want 'int'"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_components=1, strip_prefix="top")
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[
            (
                r#"a.zip"#,
                Serve::Archive(
                    r#"zip"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
            (
                r#"a.tar.gz"#,
                Serve::Archive(
                    r#"tar.gz"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
        ],
        twice: false,
        error: Some(
            r#"download_and_extract() got multiple strip values. Only one of 'strip_prefix' or 'strip_components' can be set"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_components=1, strip_prefix="nothere")
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[
            (
                r#"a.zip"#,
                Serve::Archive(
                    r#"zip"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
            (
                r#"a.tar.gz"#,
                Serve::Archive(
                    r#"tar.gz"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
        ],
        twice: false,
        error: Some(
            r#"download_and_extract() got multiple strip values. Only one of 'strip_prefix' or 'strip_components' can be set"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_components=3)
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[
            (
                r#"a.zip"#,
                Serve::Archive(
                    r#"zip"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
            (
                r#"a.tar.gz"#,
                Serve::Archive(
                    r#"tar.gz"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
        ],
        twice: false,
        error: None,
        printed: &[r#"[]"#],
        tree: &[(r#"out"#, r#"<dir>"#), (r#"z.marker"#, r#"x "#)],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", rename_files={"top/a.txt": "x/y/renamed.txt"}, strip_components=1)
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[
            (
                r#"a.zip"#,
                Serve::Archive(
                    r#"zip"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
            (
                r#"a.tar.gz"#,
                Serve::Archive(
                    r#"tar.gz"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
        ],
        twice: false,
        error: None,
        printed: &[r#"["<repo>/out/empty", "<repo>/out/link", "<repo>/out/sub", "<repo>/out/y"]"#],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/empty"#, r#"<dir>"#),
            (r#"out/link"#, r#"-> a.txt"#),
            (r#"out/sub"#, r#"<dir>"#),
            (r#"out/sub/b.txt"#, r#"x B"#),
            (r#"out/y"#, r#"<dir>"#),
            (r#"out/y/renamed.txt"#, r#"A"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/a.zip", "a.zip", sha256="@SHA:a.zip@")
    ctx.extract("a.zip", "out", strip_components=1)
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[
            (
                r#"a.zip"#,
                Serve::Archive(
                    r#"zip"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
            (
                r#"a.tar.gz"#,
                Serve::Archive(
                    r#"tar.gz"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
        ],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/a.txt", "<repo>/out/empty", "<repo>/out/link", "<repo>/out/sub"]"#,
        ],
        tree: &[
            (r#"a.zip"#, r#"<binary>"#),
            (r#"out"#, r#"<dir>"#),
            (r#"out/a.txt"#, r#"A"#),
            (r#"out/empty"#, r#"<dir>"#),
            (r#"out/link"#, r#"-> a.txt"#),
            (r#"out/sub"#, r#"<dir>"#),
            (r#"out/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/a.zip", "a.zip", sha256="@SHA:a.zip@")
    ctx.extract("a.zip", "out", "", {}, strip_components=1)
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[
            (
                r#"a.zip"#,
                Serve::Archive(
                    r#"zip"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
            (
                r#"a.tar.gz"#,
                Serve::Archive(
                    r#"tar.gz"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
        ],
        twice: false,
        error: Some(r#"extract() accepts no more than 3 positional arguments but got 4"#),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download("@URL@/a.zip", "a.zip", sha256="@SHA:a.zip@")
    ctx.extract("a.zip", "out", "", {}, "auto", 1)
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[
            (
                r#"a.zip"#,
                Serve::Archive(
                    r#"zip"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
            (
                r#"a.tar.gz"#,
                Serve::Archive(
                    r#"tar.gz"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
        ],
        twice: false,
        error: Some(r#"extract() accepts no more than 3 positional arguments but got 6"#),
        printed: &[],
        tree: &[],
        requests: &[r#"/a.zip"#],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", "@SHA:a.zip@", "", "", False, "", {}, {}, 1)
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[
            (
                r#"a.zip"#,
                Serve::Archive(
                    r#"zip"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
            (
                r#"a.tar.gz"#,
                Serve::Archive(
                    r#"tar.gz"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
        ],
        twice: false,
        error: Some(
            r#"download_and_extract() accepts no more than 9 positional arguments but got 10"#,
        ),
        printed: &[],
        tree: &[],
        requests: &[],
    },
    DlRow {
        bzl: r#"def _impl(ctx):
    ctx.download_and_extract("@URL@/a.zip", "out", sha256="@SHA:a.zip@", strip_components=1, strip_prefix="")
    print(sorted([str(p) for p in ctx.path("out").readdir()]))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        serve: &[
            (
                r#"a.zip"#,
                Serve::Archive(
                    r#"zip"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
            (
                r#"a.tar.gz"#,
                Serve::Archive(
                    r#"tar.gz"#,
                    &[
                        (r#"top/a.txt"#, "f", r#"A"#),
                        (r#"top/sub/b.txt"#, "x", r#"B"#),
                        (r#"top/sub/"#, "d", ""),
                        (r#"top/link"#, "l", r#"a.txt"#),
                        (r#"top/empty/"#, "d", ""),
                        (r#"root.txt"#, "f", r#"R"#),
                    ],
                ),
            ),
        ],
        twice: false,
        error: None,
        printed: &[
            r#"["<repo>/out/a.txt", "<repo>/out/empty", "<repo>/out/link", "<repo>/out/sub"]"#,
        ],
        tree: &[
            (r#"out"#, r#"<dir>"#),
            (r#"out/a.txt"#, r#"A"#),
            (r#"out/empty"#, r#"<dir>"#),
            (r#"out/link"#, r#"-> a.txt"#),
            (r#"out/sub"#, r#"<dir>"#),
            (r#"out/sub/b.txt"#, r#"x B"#),
            (r#"z.marker"#, r#"x "#),
        ],
        requests: &[r#"/a.zip"#],
    },
];
