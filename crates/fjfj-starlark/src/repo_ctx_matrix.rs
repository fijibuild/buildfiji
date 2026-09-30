//! Generated from Bazel 9.2.0 probes: what repository rules do to their repository.

use crate::repo_ctx_tests::RepoRow;

pub(crate) const REPO_CASES: &[RepoRow] = &[
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("BUILD.bazel", "x")
    print(ctx.name, ctx.original_name, type(ctx.attr), ctx.workspace_root)
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"+r+NAME NAME RepoDefinition <ws>"#],
        tree: &[(r#"BUILD.bazel"#, r#"x x"#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(str(ctx.path("a")))
    print(type(ctx.path("a")))
    print(ctx.path("a").basename, ctx.path("a/b/c").dirname)
    print(ctx.path("a").exists)
    print(dir(ctx.path("a")))
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"+r+NAME must create a directory"#),
        printed: &[
            r#"<repo>/a"#,
            r#"path"#,
            r#"a <repo>/a/b"#,
            r#"False"#,
            r#"["basename", "dirname", "exists", "get_child", "is_dir", "readdir", "realpath"]"#,
        ],
        tree: &[],
    },
    RepoRow {
        bzl: r##"def _impl(ctx):
    ctx.file("a/b.txt", "hello")
    ctx.file("c.txt")
    ctx.file("d.sh", "#!/bin/sh", executable=True)
    ctx.file("e.txt", "x", executable=False)
    ctx.file("BUILD.bazel", "")
r = repository_rule(_impl)
"##,
        files: &[],
        error: None,
        printed: &[],
        tree: &[
            (r#"BUILD.bazel"#, r#"x "#),
            (r#"a"#, r#"<dir>"#),
            (r#"a/b.txt"#, r#"x hello"#),
            (r#"c.txt"#, r#"x "#),
            (r#"d.sh"#, r#"x #!/bin/sh"#),
            (r#"e.txt"#, r#"x"#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a.txt", "one")
    ctx.file("a.txt", "two")
    print(ctx.read("a.txt"))
    ctx.file("b.txt", "x")
    ctx.file("b.txt", "y", legacy_utf8=False)
    print(ctx.read("b.txt"))
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"two"#, r#"y"#],
        tree: &[(r#"a.txt"#, r#"x two"#), (r#"b.txt"#, r#"x y"#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("../escape.txt", "x")
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"Cannot write outside of the repository directory for path <ext>/escape.txt"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("/abs.txt", "x")
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"Cannot write outside of the repository directory for path /abs.txt"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file(1, "x")
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to file(), parameter 'path' got value of type 'int', want 'string, Label, or path'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", 1)
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to file(), parameter 'content' got value of type 'int', want 'string'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "x", bogus=1)
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"file() got unexpected keyword argument 'bogus'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file()
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"file() missing 1 required positional argument: path"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.template("out.txt", Label("//:t.tmpl"), {"%{A}": "1", "%{B}": "two"})
    print(ctx.read("out.txt"))
r = repository_rule(_impl)
"#,
        files: &[(
            r#"t.tmpl"#,
            r#"a=%{A} b=%{B} c=%{C}
"#,
        )],
        error: None,
        printed: &[r#"a=1 b=two c=%{C}"#],
        tree: &[(
            r#"out.txt"#,
            r#"x a=1 b=two c=%{C}
"#,
        )],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.template("out.txt", "t.tmpl", {"%{A}": "1"})
    print(ctx.read("out.txt"))
r = repository_rule(_impl)
"#,
        files: &[(
            r#"t.tmpl"#,
            r#"a=%{A}
"#,
        )],
        error: Some(r#"java.io.FileNotFoundException: <repo>/t.tmpl (No such file or directory)"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.template("out.sh", Label("//:t.tmpl"), {}, executable=True)
r = repository_rule(_impl)
"#,
        files: &[(r#"t.tmpl"#, r#"x"#)],
        error: None,
        printed: &[],
        tree: &[(r#"out.sh"#, r#"x x"#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.template("out.sh", Label("//:missing"), {})
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"java.io.FileNotFoundException: missing (No such file or directory)"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.read("nothing"))
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"java.io.FileNotFoundException: <repo>/nothing (No such file or directory)"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("f", "x")
    print(ctx.read("f"))
    print(ctx.read(ctx.path("f")))
    print(ctx.read(Label("//:f1.txt")))
r = repository_rule(_impl)
"#,
        files: &[(r#"f1.txt"#, r#"from ws"#)],
        error: None,
        printed: &[r#"x"#, r#"x"#, r#"from ws"#],
        tree: &[(r#"f"#, r#"x x"#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("d/x", "1")
    ctx.delete("d")
    print(ctx.path("d").exists)
    print(ctx.delete("nothing"))
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"False"#, r#"False"#],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "1")
    ctx.rename("a", "b")
    print(ctx.path("a").exists, ctx.read("b"))
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"False 1"#],
        tree: &[(r#"b"#, r#"x 1"#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "1")
    ctx.symlink("t", "l")
    print(ctx.read("l"))
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"1"#],
        tree: &[(r#"l"#, r#"-> <repo>/t"#), (r#"t"#, r#"x 1"#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.symlink(Label("//:f1.txt"), "l")
    print(ctx.read("l"))
r = repository_rule(_impl)
"#,
        files: &[(r#"f1.txt"#, r#"hi"#)],
        error: None,
        printed: &[r#"hi"#],
        tree: &[(r#"l"#, r#"-> <ws>/f1.txt"#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["echo", "hi"])
    print(r.return_code, repr(r.stdout), repr(r.stderr))
    print(type(r))
    print(dir(r))
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[
            r#"0 "hi\n" """#,
            r#"exec_result"#,
            r#"["return_code", "stderr", "stdout"]"#,
        ],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["sh", "-c", "echo out; echo err >&2; exit 3"])
    print(r.return_code, repr(r.stdout), repr(r.stderr))
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"3 "out\n" "err\n""#],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["sleep", "5"], timeout=1)
    print(r.return_code, repr(r.stdout), repr(r.stderr))
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"256 "" "Timed out""#],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["sh", "-c", "echo $FOO; pwd"], environment={"FOO": "bar"})
    print(repr(r.stdout))
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#""bar\n<repo>\n""#],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("sub/x", "1")
    r = ctx.execute(["pwd"], working_directory="sub")
    print(repr(r.stdout))
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#""<repo>/sub\n""#],
        tree: &[(r#"sub"#, r#"<dir>"#), (r#"sub/x"#, r#"x 1"#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["nonexistent-binary"])
    print(r.return_code, repr(r.stdout), repr(r.stderr))
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[
            r#"1 "" "src/main/tools/process-wrapper-legacy.cc:80: \"execvp(nonexistent-binary, ...)\": No such file or directory\n""#,
        ],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute([])
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute("echo")
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to execute(), parameter 'arguments' got value of type 'string', want 'sequence'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["echo", 1])
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"Argument 1 of execute is neither a path, label, nor string."#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.report_progress("working")
    print(1)
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"+r+NAME must create a directory"#),
        printed: &[r#"1"#],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.attr.nothing)
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"unknown attribute nothing"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(dir(ctx.attr))
    print(str(ctx.attr))
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"+r+NAME must create a directory"#),
        printed: &[
            r#"["name"]"#,
            r#"<unknown object com.google.devtools.build.lib.bazel.repository.RepoDefinition>"#,
        ],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    fail("boom")
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"boom"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    x = 1 + "a"
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"unsupported binary operation: int + string"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("BUILD.bazel", "filegroup(name='all')")
    print("ok")
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"ok"#],
        tree: &[(r#"BUILD.bazel"#, r#"x filegroup(name='all')"#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    pass
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"+r+NAME must create a directory"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    p = ctx.path("a")
    print(p == ctx.path("a"), p == ctx.path("b"), hash(p) == hash(ctx.path("a")))
    print(p.get_child("b"), p.get_child("b", "c"), p.get_child())
    print(repr(p), str(p))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to hash(), parameter 'value' got value of type 'path', want 'string'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("f", "x")
    print(ctx.path("f").realpath)
    ctx.symlink("f", "l")
    print(ctx.path("l").realpath)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"<repo>/f"#, r#"<repo>/f"#],
        tree: &[
            (r#"f"#, r#"x x"#),
            (r#"l"#, r#"-> <repo>/f"#),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.path("/abs/x"), ctx.path("../x"), ctx.path(""), ctx.path("a/../b"), ctx.path("./a"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"/abs/x <ext>/x <repo> <repo>/b <repo>/a"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.path(Label("//:f1.txt")))
    print(ctx.path(ctx.path("a")))
    print(ctx.path(ctx.workspace_root))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[(r#"f1.txt"#, r#""#)],
        error: None,
        printed: &[r#"<ws>/f1.txt"#, r#"<repo>/a"#, r#"<ws>"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.path(1))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to path(), parameter 'path' got value of type 'int', want 'string, Label, or path'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.path(None))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to path(), parameter 'path' got value of type 'NoneType', want 'string, Label, or path'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.path())
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"path() missing 1 required positional argument: path"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.file("a", "x"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"None"#],
        tree: &[(r#"a"#, r#"x x"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "x", executable=1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to file(), parameter 'executable' got value of type 'int', want 'bool'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "x", legacy_utf8=1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to file(), parameter 'legacy_utf8' got value of type 'int', want 'bool'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a/b/c/d.txt", "x")
    print(ctx.path("a/b/c").is_dir)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"True"#],
        tree: &[
            (r#"a"#, r#"<dir>"#),
            (r#"a/b"#, r#"<dir>"#),
            (r#"a/b/c"#, r#"<dir>"#),
            (r#"a/b/c/d.txt"#, r#"x x"#),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "x")
    ctx.file("a/b", "y")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"java.io.IOException: <repo>/a (File exists)"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file(".", "x")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"java.io.IOException: <repo> (File exists)"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("", "x")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"java.io.IOException: <repo> (File exists)"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file(Label("//:f1.txt"), "x")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[(r#"f1.txt"#, r#""#)],
        error: Some(r#"Cannot write outside of the repository directory for path f1.txt"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file(ctx.path("p.txt"), "via path")
    print(ctx.read("p.txt"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"via path"#],
        tree: &[(r#"p.txt"#, r#"x via path"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.read("a", watch="no"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"java.io.FileNotFoundException: <repo>/a (No such file or directory)"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "x")
    print(ctx.read("a", watch="yes"))
    print(ctx.read("a", watch="auto"))
    print(ctx.read("a", watch="bogus"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"attempted to watch path under working directory"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "x")
    print(ctx.delete("a"), ctx.delete("a"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"True False"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "x")
    print(ctx.delete(ctx.path("a")))
    print(ctx.delete(Label("//:f1.txt")))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[(r#"f1.txt"#, r#""#)],
        error: Some(
            r#"in call to delete(), parameter 'path' got value of type 'Label', want 'string or path'"#,
        ),
        printed: &[r#"True"#],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.delete(1))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to delete(), parameter 'path' got value of type 'int', want 'string or path'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.delete("/nonexistent/x"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"False"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "1")
    ctx.file("b", "2")
    ctx.rename("a", "b")
    print(ctx.read("b"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"java.io.IOException: Could not rename <repo>/a to <repo>/b: already exists"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.rename("missing", "b")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"java.io.IOException: Could not rename <repo>/missing to <repo>/b: [unix_jni.cc:638] <repo>/missing -> <repo>/b (No such file or directory)"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "1")
    ctx.rename("a", "sub/b")
    print(ctx.read("sub/b"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"1"#],
        tree: &[
            (r#"sub"#, r#"<dir>"#),
            (r#"sub/b"#, r#"x 1"#),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "1")
    ctx.rename("a", "../b")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"Cannot write outside of the repository directory for path <ext>/b"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "1")
    ctx.symlink("a", "l")
    ctx.symlink("a", "l")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"java.io.IOException: Could not create symlink from <repo>/a to <repo>/l: [unix_jni.cc:297] <repo>/l (File exists)"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.symlink("missing", "l")
    print(ctx.path("l").exists)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"False"#],
        tree: &[(r#"l"#, r#"-> <repo>/missing"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "1")
    ctx.symlink("a", "sub/l")
    print(ctx.read("sub/l"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"1"#],
        tree: &[
            (r#"a"#, r#"x 1"#),
            (r#"sub"#, r#"<dir>"#),
            (r#"sub/l"#, r#"-> <repo>/a"#),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "1")
    ctx.symlink("a", "../l")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"Cannot write outside of the repository directory for path <ext>/l"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.symlink("a", "l"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"None"#],
        tree: &[(r#"l"#, r#"-> <repo>/a"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.template("o", "t", {"%{A}": "1"})
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[(r#"t"#, r#"x"#)],
        error: Some(r#"java.io.FileNotFoundException: <repo>/t (No such file or directory)"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "a=%{A} %{A}\n")
    ctx.template("o", "t", {"%{A}": "1"})
    print(repr(ctx.read("o")))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#""a=1 1\n""#],
        tree: &[
            (
                r#"o"#,
                r#"x a=1 1
"#,
            ),
            (
                r#"t"#,
                r#"x a=%{A} %{A}
"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.template("o", "t", {1: "a"})
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"got dict<int, string> for 'substitutions', want dict<string, string>"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.template("o", "t", {"a": 1})
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"got dict<string, int> for 'substitutions', want dict<string, string>"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.template("o", "t", {"": "a"})
    print(ctx.read("o"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"x"#],
        tree: &[
            (r#"o"#, r#"x x"#),
            (r#"t"#, r#"x x"#),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.template("o", "t", None)
    print(ctx.read("o"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to template(), parameter 'substitutions' got value of type 'NoneType', want 'dict'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["echo", "a", "b"], quiet=False)
    print(r.stdout)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"a b"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["sh", "-c", "exit 0"], timeout=1.5)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to execute(), parameter 'timeout' got value of type 'float', want 'int'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["sh", "-c", "exit 0"], timeout="1")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to execute(), parameter 'timeout' got value of type 'string', want 'int'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["sh", "-c", "echo $FOO"], environment={"FOO": None})
    print(repr(r.stdout))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#""\n""#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["sh", "-c", "echo $FOO"], environment={"FOO": 1})
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"environment values must be strings or None, got 1"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["sh", "-c", "echo $FOO"], environment=1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to execute(), parameter 'environment' got value of type 'int', want 'dict'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r##"def _impl(ctx):
    ctx.file("x.sh", "#!/bin/sh\necho script", executable=True)
    r = ctx.execute([ctx.path("x.sh")])
    print(repr(r.stdout))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"##,
        files: &[],
        error: None,
        printed: &[r#""script\n""#],
        tree: &[
            (
                r#"x.sh"#,
                r#"x #!/bin/sh
echo script"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r##"def _impl(ctx):
    ctx.file("x.sh", "#!/bin/sh\necho script", executable=True)
    r = ctx.execute(["./x.sh"])
    print(repr(r.stdout), r.return_code)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"##,
        files: &[],
        error: None,
        printed: &[r#""script\n" 0"#],
        tree: &[
            (
                r#"x.sh"#,
                r#"x #!/bin/sh
echo script"#,
            ),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute([Label("//:f1.txt")])
    print(r.return_code)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[(r#"f1.txt"#, r#""#)],
        error: None,
        printed: &[r#"1"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["sh", "-c", "cat"], working_directory="/tmp")
    print(r.return_code)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"0"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["pwd"], working_directory="nodir")
    print(repr(r.stdout), r.return_code)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#""<repo>/nodir\n" 0"#],
        tree: &[(r#"nodir"#, r#"<dir>"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["sh", "-c", "printf abc; printf def >&2"])
    print(repr(r.stdout), repr(r.stderr))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#""abc" "def""#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.os.environ.get("PATH") != None, type(ctx.os.environ))
    print(ctx.getenv("PATH") == ctx.os.environ["PATH"])
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"True dict"#, r#"True"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.os.environ.get("FJFJ_PROBE_VAR"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"None"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.getenv(1))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to getenv(), parameter 'name' got value of type 'int', want 'string'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.getenv("A", 1))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to getenv(), parameter 'default' got value of type 'int', want 'string or NoneType'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.which(1))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to which(), parameter 'program' got value of type 'int', want 'string'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.which("/bin/sh"), ctx.which("./nothing"), ctx.which(""))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"Program argument of which() may not contain a / or a \ ('/bin/sh' given)"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.report_progress()
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.report_progress(1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to report_progress(), parameter 'status' got value of type 'int', want 'string'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.report_progress("x"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"None"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a","x")
    print(ctx.watch("a"))
    print(ctx.watch_tree("."))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"attempted to watch path under working directory"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.repo_metadata(reproducible=True))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[
            r#"<unknown object com.google.devtools.build.lib.bazel.repository.starlark.RepoMetadata>"#,
        ],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(type(ctx.repo_metadata()))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"repo_metadata"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.original_name, ctx.name)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"NAME +r+NAME"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.attr.x = 1
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"RepoDefinition value does not support field assignment"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.foo
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"'repository_ctx' value has no field or method 'foo'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(hasattr(ctx, "foo"), hasattr(ctx, "file"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"False True"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[
            r#"<unknown object com.google.devtools.build.lib.bazel.repository.starlark.StarlarkRepositoryContext>"#,
        ],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(repr(ctx), str(ctx))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[
            r#"<unknown object com.google.devtools.build.lib.bazel.repository.starlark.StarlarkRepositoryContext> <unknown object com.google.devtools.build.lib.bazel.repository.starlark.StarlarkRepositoryContext>"#,
        ],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.workspace_root == ctx.path("."))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"False"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.path("a") + "b")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"unsupported binary operation: path + string"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.path("a") < ctx.path("b"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"unsupported comparison: path <=> path"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print("x" in ctx.path("a"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"unsupported binary operation: string in path"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(len(ctx.path("a")))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to len(), parameter 'x' got value of type 'path', want 'iterable or string'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print({ctx.path("a"): 1})
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"{"<repo>/a": 1}"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(bool(ctx.path("a")))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"True"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(json.encode(ctx.path("a")))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"cannot encode path as JSON"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    x = ctx.path("a").foo
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"'path' value has no field or method 'foo'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.attr.d, ctx.attr.b, ctx.attr.i, ctx.attr.s, ctx.attr.sl, ctx.attr.sd, ctx.attr.ll)
    ctx.file('z.marker', '')
r = repository_rule(_impl, attrs={"d": attr.string_dict(), "b": attr.bool(), "i": attr.int(), "s": attr.string(), "sl": attr.string_list(), "sd": attr.string_list_dict(), "ll": attr.label_list()})
"#,
        files: &[],
        error: None,
        printed: &[r#"{} False 0  [] {} []"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.path(".").exists, ctx.path(".").is_dir)
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"+r+NAME must create a directory"#),
        printed: &[r#"False False"#],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.path(".").exists, ctx.path(".").is_dir)
    ctx.execute(["true"])
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"False False"#],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.execute(["true"])
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.execute(["true"], working_directory="sub")
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[(r#"sub"#, r#"<dir>"#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.symlink("/tmp", "l")
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[(r#"l"#, r#"-> /tmp"#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("d/.keep", "")
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[(r#"d"#, r#"<dir>"#), (r#"d/.keep"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.delete("nothing")
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"+r+NAME must create a directory"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.execute(["sh", "-c", "mkdir d"])
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[(r#"d"#, r#"<dir>"#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.execute(["sh", "-c", "ls -la . | wc -l; pwd"], quiet=False)
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.execute(["sh", "-c", "ls -A . | wc -l"]).stdout)
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"0"#],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "")
    ctx.delete("a")
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "")
    ctx.delete(".")
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"+r+NAME must create a directory"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "")
    print(ctx.delete(ctx.path(".")))
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"+r+NAME must create a directory"#),
        printed: &[r#"True"#],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.delete("/tmp/fjfj_nonexistent_dir_xyz"))
    ctx.file("a", "")
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"False"#],
        tree: &[(r#"a"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "")
    ctx.symlink("a", ".")
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"java.io.IOException: Could not create symlink from <repo>/a to <repo>: [unix_jni.cc:297] <repo> (File exists)"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "")
    ctx.rename(".", "b")
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"java.io.IOException: Could not rename <repo> to <repo>/b: [unix_jni.cc:638] <repo> -> <repo>/b (Invalid argument)"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.path("/").dirname, ctx.path("/").basename == "")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"None True"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.symlink("/nonexistent", "dangling")
    print(ctx.path("dangling").exists, ctx.path("dangling").is_dir)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"False False"#],
        tree: &[
            (r#"dangling"#, r#"-> /nonexistent"#),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("d/f", "x")
    ctx.symlink("d", "ld")
    print(ctx.path("ld").is_dir, ctx.path("ld").exists)
    print(ctx.read("ld/f"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"True True"#, r#"x"#],
        tree: &[
            (r#"d"#, r#"<dir>"#),
            (r#"d/f"#, r#"x x"#),
            (r#"ld"#, r#"-> <repo>/d"#),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.path("nothing").realpath)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"[unix_jni.cc:382] <repo> (No such file or directory)"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("d/a", "1")
    ctx.file("d/s/b", "2")
    print(ctx.delete("d"))
    print(ctx.path("d").exists)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"True"#, r#"False"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("f", "1")
    ctx.symlink("f", "l")
    print(ctx.delete("l"))
    print(ctx.path("f").exists, ctx.path("l").exists)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"True"#, r#"True False"#],
        tree: &[(r#"f"#, r#"x 1"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", None)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to file(), parameter 'content' got value of type 'NoneType', want 'string'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", content="x", executable=False)
    print(ctx.read("a"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"x"#],
        tree: &[(r#"a"#, r#"x"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.symlink("rel", "l")
    print(ctx.path("l").exists)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"False"#],
        tree: &[(r#"l"#, r#"-> <repo>/rel"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("rel", "x")
    ctx.symlink("rel", "l")
    print(ctx.path("l").realpath)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"<repo>/rel"#],
        tree: &[
            (r#"l"#, r#"-> <repo>/rel"#),
            (r#"rel"#, r#"x x"#),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("d/x", "1")
    ctx.rename("d", "e")
    print(ctx.read("e/x"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"1"#],
        tree: &[
            (r#"e"#, r#"<dir>"#),
            (r#"e/x"#, r#"x 1"#),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("d/x", "1")
    ctx.file("e/y", "2")
    ctx.rename("d", "e")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"java.io.IOException: Could not rename <repo>/d to <repo>/e: already exists"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("d/x", "1")
    print(ctx.read("d"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"attempting to read() a directory: <repo>/d"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "x")
    ctx.file("b", "y")
    ctx.symlink("a", "b")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"java.io.IOException: Could not create symlink from <repo>/a to <repo>/b: [unix_jni.cc:297] <repo>/b (File exists)"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.template("o", Label("//pkg:t.tmpl"), {"%{A}": "1"})
    print(ctx.read("o"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[
            (r#"pkg/t.tmpl"#, r#"v=%{A}"#),
            (r#"pkg/BUILD.bazel"#, r#""#),
        ],
        error: None,
        printed: &[r#"v=1"#],
        tree: &[(r#"o"#, r#"x v=1"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.path(Label("//pkg:file.txt")))
    print(ctx.read(Label("//pkg:file.txt")))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[
            (r#"pkg/file.txt"#, r#"content"#),
            (r#"pkg/BUILD.bazel"#, r#""#),
        ],
        error: None,
        printed: &[r#"<ws>/pkg/file.txt"#, r#"content"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["sh", "-c", "head -c 100000 /dev/zero | tr \\0 a"])
    print(len(r.stdout), r.return_code)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"100000 0"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["sh", "-c", "kill -9 $$"])
    print(r.return_code, repr(r.stderr))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"137 """#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["echo", "x"], environment={"PATH": "/nonexistent"})
    print(r.return_code, repr(r.stdout))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"1 """#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["sh", "-c", "echo $0 $#", "a", "b"])
    print(repr(r.stdout))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#""a 1\n""#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["sh", "-c", "sleep 3; echo late"], timeout=1)
    print(r.return_code, repr(r.stdout), repr(r.stderr))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"256 "" "Timed out""#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    r = ctx.execute(["sh", "-c", "echo part; sleep 3"], timeout=1)
    print(r.return_code, repr(r.stdout), repr(r.stderr))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"256 "" "Timed out""#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.getenv("FOO"), ctx.os.environ.get("FOO"))
    r = ctx.execute(["sh", "-c", "echo $FOO"])
    print(repr(r.stdout))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"None None"#, r#""\n""#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl2(ctx, extra): pass
def _impl(ctx):
    print(1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"1"#],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "x")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[(r#"a"#, r#"x x"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(type(ctx.repo_metadata(reproducible=True)))
    ctx.file("a", "x")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"repo_metadata"#],
        tree: &[(r#"a"#, r#"x x"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.repo_metadata(attrs_for_reproducibility={"a": 1}))
    ctx.file("a", "x")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[
            r#"<unknown object com.google.devtools.build.lib.bazel.repository.starlark.RepoMetadata>"#,
        ],
        tree: &[(r#"a"#, r#"x x"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.repo_metadata(reproducible=1))
    ctx.file("a", "x")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to repo_metadata(), parameter 'reproducible' got value of type 'int', want 'bool'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    fail("a", 1, [2])
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"a 1 [2]"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    fail()
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#""#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "x")
    print(ctx.path("a").get_child("b", "..", "c"), ctx.path("a").get_child("/abs"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"<repo>/a/c /abs"#],
        tree: &[(r#"a"#, r#"x x"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "x")
    print(ctx.path("a").get_child(1))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"at index 0 of relative_paths, got element of type int, want string"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.path("a").basename, ctx.path("a/b.txt").basename, ctx.path(".").basename.startswith("+r+"))
    ctx.file("z", "")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"a b.txt True"#],
        tree: &[(r#"z"#, r#"x "#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("z", "")
    print(ctx.path("z").dirname == ctx.path("."))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"True"#],
        tree: &[(r#"z"#, r#"x "#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("z", "")
    print(str(ctx.path("z").dirname) == str(ctx.path(".")))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"True"#],
        tree: &[(r#"z"#, r#"x "#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.execute(["sh", "-c", "printf '\\377\\376' > bin"])
    print(len(ctx.read("bin")))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[r#"2"#],
        tree: &[(r#"bin"#, r#"��"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "x")
    print(ctx.read("a", watch="no"), ctx.read(path="a"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"read() got named argument for positional-only parameter 'path'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file(path="a", content="b")
    print(ctx.read("a"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"file() got named argument for positional-only parameter 'path'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "x", True)
    ctx.file("b", "x", False)
    ctx.file("c", "x", True, False)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[
            (r#"a"#, r#"x x"#),
            (r#"b"#, r#"x"#),
            (r#"c"#, r#"x x"#),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "x", True, False, 1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"file() accepts no more than 4 positional arguments but got 5"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.template("o", "t", {}, True, "auto", 1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[(r#"t"#, r#"x"#)],
        error: Some(r#"template() accepts no more than 4 positional arguments but got 6"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.rename("a")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"rename() missing 1 required positional argument: dst"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.symlink("a")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"symlink() missing 1 required positional argument: link_name"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.delete()
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"delete() missing 1 required positional argument: path"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.read()
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"read() missing 1 required positional argument: path"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.which()
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"which() missing 1 required positional argument: program"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.getenv()
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"getenv() missing 1 required positional argument: name"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.execute()
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"execute() missing 1 required positional argument: arguments"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.execute(["true"], 5)
    ctx.execute(["true"], 5, {})
    ctx.execute(["true"], 5, {}, True, "")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[(r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.execute(["true"], 5, {}, True, "", 1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"execute() accepts no more than 5 positional arguments but got 6"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("a", "x")
    ctx.watch(ctx.workspace_root.get_child("f1.txt"))
    ctx.watch_tree(ctx.workspace_root)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[(r#"f1.txt"#, r#""#)],
        error: None,
        printed: &[],
        tree: &[(r#"a"#, r#"x x"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.watch(Label("//:f1.txt"))
    ctx.file("a", "x")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[(r#"f1.txt"#, r#""#)],
        error: None,
        printed: &[],
        tree: &[(r#"a"#, r#"x x"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.watch("/tmp")
    ctx.file("a", "x")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[(r#"a"#, r#"x x"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.watch(1)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"in call to watch(), parameter 'path' got value of type 'int', want 'string, Label, or path'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    print(ctx.report_progress("a", "b"))
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"report_progress() accepts no more than 1 positional argument but got 2"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.file(path="a")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"file() got named argument for positional-only parameter 'path'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.file("a", content="x")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[
            (r#"a"#, r#"x x"#),
            (r#"t"#, r#"x x"#),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.file("a", "x", executable=True)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[
            (r#"a"#, r#"x x"#),
            (r#"t"#, r#"x x"#),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.file("a", "x", True, legacy_utf8=True)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[
            (r#"a"#, r#"x x"#),
            (r#"t"#, r#"x x"#),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.template(path="o")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"template() got named argument for positional-only parameter 'path'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.template("o", template="t")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"template() got named argument for positional-only parameter 'template'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.template("o", "t", substitutions={})
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[
            (r#"o"#, r#"x x"#),
            (r#"t"#, r#"x x"#),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.template("o", "t", {}, executable=True)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[
            (r#"o"#, r#"x x"#),
            (r#"t"#, r#"x x"#),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.template("o", "t", {}, True, watch_template="auto")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[
            (r#"o"#, r#"x x"#),
            (r#"t"#, r#"x x"#),
            (r#"z.marker"#, r#"x "#),
        ],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.read(path="t")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"read() got named argument for positional-only parameter 'path'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.read("t", watch="auto")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[(r#"t"#, r#"x x"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.delete(path="t")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"delete() got named argument for positional-only parameter 'path'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.rename(src="t")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"rename() got named argument for positional-only parameter 'src'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.rename("t", dst="u")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"rename() got named argument for positional-only parameter 'dst'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.symlink(target="t")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"symlink() got named argument for positional-only parameter 'target'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.symlink("t", link_name="l")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"symlink() got named argument for positional-only parameter 'link_name'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.execute(arguments=["true"])
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"execute() got named argument for positional-only parameter 'arguments'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.execute(["true"], timeout=5)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[(r#"t"#, r#"x x"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.execute(["true"], 5, environment={})
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[(r#"t"#, r#"x x"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.execute(["true"], 5, {}, quiet=True)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[(r#"t"#, r#"x x"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.execute(["true"], 5, {}, True, working_directory="")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[(r#"t"#, r#"x x"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.which(program="sh")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"which() got named argument for positional-only parameter 'program'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.getenv(name="A")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"getenv() got named argument for positional-only parameter 'name'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.getenv("A", default="d")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"getenv() got named argument for positional-only parameter 'default'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.path(path="t")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"path() got named argument for positional-only parameter 'path'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.report_progress(status="s")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(
            r#"report_progress() got named argument for positional-only parameter 'status'"#,
        ),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.watch(path="/tmp")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"watch() got named argument for positional-only parameter 'path'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.watch_tree(path="/tmp")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"watch_tree() got named argument for positional-only parameter 'path'"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.repo_metadata(reproducible=True)
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: None,
        printed: &[],
        tree: &[(r#"t"#, r#"x x"#), (r#"z.marker"#, r#"x "#)],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.repo_metadata(True, attrs_for_reproducibility={})
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"repo_metadata() got unexpected positional argument"#),
        printed: &[],
        tree: &[],
    },
    RepoRow {
        bzl: r#"def _impl(ctx):
    ctx.file("t", "x")
    ctx.path("t").get_child(relative_paths="a")
    ctx.file('z.marker', '')
r = repository_rule(_impl)
"#,
        files: &[],
        error: Some(r#"get_child() got unexpected keyword argument 'relative_paths'"#),
        printed: &[],
        tree: &[],
    },
];
