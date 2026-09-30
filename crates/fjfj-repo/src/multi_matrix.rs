//! Generated from probes of Bazel 9.2.0 (`bazel build` against a local registry
//! of local-path modules); see `multi_tests.rs`.

use crate::multi_tests::MultiRow;

pub(crate) const MULTI_ROWS: &[MultiRow] = &[
    MultiRow {
        name: r#"every_module_in_order"#,
        registry: &[
            (
                r#"lib"#,
                r#"1.0"#,
                &[
                    (
                        r#"MODULE.bazel"#,
                        r##"module(name = "lib", version = "1.0")
"##,
                    ),
                    (
                        r#"ext.bzl"#,
                        r###"def _r(rctx):
    rctx.file("BUILD.bazel", "# " + rctx.attr.info + "\nfilegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"info": attr.string()})

_add = tag_class(attrs = {"name": attr.string(mandatory = True)})

def _impl(mctx):
    lines = []
    for m in mctx.modules:
        lines.append("%s@%s root=%s tags=%s" % (m.name, m.version, m.is_root, [t.name for t in m.tags.add]))
    info = "; ".join(lines) + " nd=" + str(mctx.root_module_has_non_dev_dependency)
    print(info)
    for m in mctx.modules:
        for t in m.tags.add:
            made(name = t.name, info = info)

collect = module_extension(implementation = _impl, tag_classes = {"add": _add})
"###,
                    ),
                ],
            ),
            (
                r#"a"#,
                r#"1.0"#,
                &[(
                    r#"MODULE.bazel"#,
                    r##"module(name = "a", version = "1.0")
bazel_dep(name = "lib", version = "1.0")
e = use_extension("@lib//:ext.bzl", "collect")
e.add(name = "from_a")
use_repo(e, "from_a")
"##,
                )],
            ),
            (
                r#"b"#,
                r#"1.0"#,
                &[(
                    r#"MODULE.bazel"#,
                    r##"module(name = "b", version = "1.0")
bazel_dep(name = "lib", version = "1.0")
e = use_extension("@lib//:ext.bzl", "collect")
e.add(name = "from_b")
use_repo(e, "from_b")
d = use_extension("@lib//:ext.bzl", "collect", dev_dependency = True)
d.add(name = "from_b_dev")
"##,
                )],
            ),
        ],
        root: &[(
            r#"MODULE.bazel"#,
            r##"module(name = "root", version = "0")
bazel_dep(name = "b", version = "1.0")
bazel_dep(name = "a", version = "1.0")
bazel_dep(name = "lib", version = "1.0")
e = use_extension("@lib//:ext.bzl", "collect")
e.add(name = "from_root")
use_repo(e, "from_root", "from_a", "from_b")
d = use_extension("@lib//:ext.bzl", "collect", dev_dependency = True)
d.add(name = "dev_root")
use_repo(d, "dev_root")
"##,
        )],
        fetch: &[
            r#"lib++collect+from_a"#,
            r#"lib++collect+from_b"#,
            r#"lib++collect+from_root"#,
            r#"lib++collect+dev_root"#,
        ],
        error: None,
        printed: &[
            r##"root@0 root=True tags=["from_root", "dev_root"]; b@1.0 root=False tags=["from_b"]; a@1.0 root=False tags=["from_a"] nd=True"##,
        ],
        builds: &[
            (
                r#"lib++collect+dev_root"#,
                r##"# root@0 root=True tags=["from_root", "dev_root"]; b@1.0 root=False tags=["from_b"]; a@1.0 root=False tags=["from_a"] nd=True
filegroup(name = 'f')"##,
            ),
            (
                r#"lib++collect+from_a"#,
                r##"# root@0 root=True tags=["from_root", "dev_root"]; b@1.0 root=False tags=["from_b"]; a@1.0 root=False tags=["from_a"] nd=True
filegroup(name = 'f')"##,
            ),
            (
                r#"lib++collect+from_b"#,
                r##"# root@0 root=True tags=["from_root", "dev_root"]; b@1.0 root=False tags=["from_b"]; a@1.0 root=False tags=["from_a"] nd=True
filegroup(name = 'f')"##,
            ),
            (
                r#"lib++collect+from_root"#,
                r##"# root@0 root=True tags=["from_root", "dev_root"]; b@1.0 root=False tags=["from_b"]; a@1.0 root=False tags=["from_a"] nd=True
filegroup(name = 'f')"##,
            ),
        ],
    },
    MultiRow {
        name: r#"root_does_not_use_it"#,
        registry: &[
            (
                r#"lib"#,
                r#"1.0"#,
                &[
                    (
                        r#"MODULE.bazel"#,
                        r##"module(name = "lib", version = "1.0")
"##,
                    ),
                    (
                        r#"ext.bzl"#,
                        r###"def _r(rctx):
    rctx.file("BUILD.bazel", "# " + rctx.attr.info + "\nfilegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"info": attr.string()})

_add = tag_class(attrs = {"name": attr.string(mandatory = True)})

def _impl(mctx):
    lines = []
    for m in mctx.modules:
        lines.append("%s@%s root=%s tags=%s" % (m.name, m.version, m.is_root, [t.name for t in m.tags.add]))
    info = "; ".join(lines) + " nd=" + str(mctx.root_module_has_non_dev_dependency)
    print(info)
    for m in mctx.modules:
        for t in m.tags.add:
            made(name = t.name, info = info)

collect = module_extension(implementation = _impl, tag_classes = {"add": _add})
"###,
                    ),
                ],
            ),
            (
                r#"a"#,
                r#"1.0"#,
                &[(
                    r#"MODULE.bazel"#,
                    r##"module(name = "a", version = "1.0")
bazel_dep(name = "lib", version = "1.0")
e = use_extension("@lib//:ext.bzl", "collect")
e.add(name = "from_a")
use_repo(e, "from_a")
"##,
                )],
            ),
        ],
        root: &[(
            r#"MODULE.bazel"#,
            r##"module(name = "root", version = "0")
bazel_dep(name = "a", version = "1.0")
"##,
        )],
        fetch: &[r#"lib++collect+from_a"#],
        error: None,
        printed: &[r##"a@1.0 root=False tags=["from_a"] nd=False"##],
        builds: &[(
            r#"lib++collect+from_a"#,
            r##"# a@1.0 root=False tags=["from_a"] nd=False
filegroup(name = 'f')"##,
        )],
    },
    MultiRow {
        name: r#"root_only_dev"#,
        registry: &[
            (
                r#"lib"#,
                r#"1.0"#,
                &[
                    (
                        r#"MODULE.bazel"#,
                        r##"module(name = "lib", version = "1.0")
"##,
                    ),
                    (
                        r#"ext.bzl"#,
                        r###"def _r(rctx):
    rctx.file("BUILD.bazel", "# " + rctx.attr.info + "\nfilegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"info": attr.string()})

_add = tag_class(attrs = {"name": attr.string(mandatory = True)})

def _impl(mctx):
    lines = []
    for m in mctx.modules:
        lines.append("%s@%s root=%s tags=%s" % (m.name, m.version, m.is_root, [t.name for t in m.tags.add]))
    info = "; ".join(lines) + " nd=" + str(mctx.root_module_has_non_dev_dependency)
    print(info)
    for m in mctx.modules:
        for t in m.tags.add:
            made(name = t.name, info = info)

collect = module_extension(implementation = _impl, tag_classes = {"add": _add})
"###,
                    ),
                ],
            ),
            (
                r#"a"#,
                r#"1.0"#,
                &[(
                    r#"MODULE.bazel"#,
                    r##"module(name = "a", version = "1.0")
bazel_dep(name = "lib", version = "1.0")
e = use_extension("@lib//:ext.bzl", "collect")
e.add(name = "from_a")
use_repo(e, "from_a")
"##,
                )],
            ),
        ],
        root: &[(
            r#"MODULE.bazel"#,
            r##"module(name = "root", version = "0")
bazel_dep(name = "a", version = "1.0")
bazel_dep(name = "lib", version = "1.0")
e = use_extension("@lib//:ext.bzl", "collect", dev_dependency = True)
e.add(name = "dev_root")
use_repo(e, "dev_root", "from_a")
"##,
        )],
        fetch: &[r#"lib++collect+from_a"#, r#"lib++collect+dev_root"#],
        error: None,
        printed: &[
            r##"root@0 root=True tags=["dev_root"]; a@1.0 root=False tags=["from_a"] nd=False"##,
        ],
        builds: &[
            (
                r#"lib++collect+dev_root"#,
                r##"# root@0 root=True tags=["dev_root"]; a@1.0 root=False tags=["from_a"] nd=False
filegroup(name = 'f')"##,
            ),
            (
                r#"lib++collect+from_a"#,
                r##"# root@0 root=True tags=["dev_root"]; a@1.0 root=False tags=["from_a"] nd=False
filegroup(name = 'f')"##,
            ),
        ],
    },
    MultiRow {
        name: r#"import_of_a_repo_nobody_makes"#,
        registry: &[
            (
                r#"lib"#,
                r#"1.0"#,
                &[
                    (
                        r#"MODULE.bazel"#,
                        r##"module(name = "lib", version = "1.0")
"##,
                    ),
                    (
                        r#"ext.bzl"#,
                        r###"def _r(rctx):
    rctx.file("BUILD.bazel", "# " + rctx.attr.info + "\nfilegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"info": attr.string()})

_add = tag_class(attrs = {"name": attr.string(mandatory = True)})

def _impl(mctx):
    lines = []
    for m in mctx.modules:
        lines.append("%s@%s root=%s tags=%s" % (m.name, m.version, m.is_root, [t.name for t in m.tags.add]))
    info = "; ".join(lines) + " nd=" + str(mctx.root_module_has_non_dev_dependency)
    print(info)
    for m in mctx.modules:
        for t in m.tags.add:
            made(name = t.name, info = info)

collect = module_extension(implementation = _impl, tag_classes = {"add": _add})
"###,
                    ),
                ],
            ),
            (
                r#"a"#,
                r#"1.0"#,
                &[(
                    r#"MODULE.bazel"#,
                    r##"module(name = "a", version = "1.0")
bazel_dep(name = "lib", version = "1.0")
e = use_extension("@lib//:ext.bzl", "collect")
e.add(name = "from_a")
use_repo(e, "from_a", "nope")
"##,
                )],
            ),
        ],
        root: &[(
            r#"MODULE.bazel"#,
            r##"module(name = "root", version = "0")
bazel_dep(name = "a", version = "1.0")
"##,
        )],
        fetch: &[r#"lib++collect+from_a"#],
        error: Some(
            r##"module extension @@lib+//:ext.bzl%collect does not generate repository "nope", yet it is imported as "nope" in the usage at <reg>/modules/a/1.0/MODULE.bazel:3:18"##,
        ),
        printed: &[r##"a@1.0 root=False tags=["from_a"] nd=False"##],
        builds: &[],
    },
    MultiRow {
        name: r#"root_imports_a_repo_nobody_makes"#,
        registry: &[
            (
                r#"lib"#,
                r#"1.0"#,
                &[
                    (
                        r#"MODULE.bazel"#,
                        r##"module(name = "lib", version = "1.0")
"##,
                    ),
                    (
                        r#"ext.bzl"#,
                        r###"def _r(rctx):
    rctx.file("BUILD.bazel", "# " + rctx.attr.info + "\nfilegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"info": attr.string()})

_add = tag_class(attrs = {"name": attr.string(mandatory = True)})

def _impl(mctx):
    lines = []
    for m in mctx.modules:
        lines.append("%s@%s root=%s tags=%s" % (m.name, m.version, m.is_root, [t.name for t in m.tags.add]))
    info = "; ".join(lines) + " nd=" + str(mctx.root_module_has_non_dev_dependency)
    print(info)
    for m in mctx.modules:
        for t in m.tags.add:
            made(name = t.name, info = info)

collect = module_extension(implementation = _impl, tag_classes = {"add": _add})
"###,
                    ),
                ],
            ),
            (
                r#"a"#,
                r#"1.0"#,
                &[(
                    r#"MODULE.bazel"#,
                    r##"module(name = "a", version = "1.0")
bazel_dep(name = "lib", version = "1.0")
e = use_extension("@lib//:ext.bzl", "collect")
e.add(name = "from_a")
use_repo(e, "from_a")
"##,
                )],
            ),
        ],
        root: &[(
            r#"MODULE.bazel"#,
            r##"module(name = "root", version = "0")
bazel_dep(name = "a", version = "1.0")
bazel_dep(name = "lib", version = "1.0")
e = use_extension("@lib//:ext.bzl", "collect")
e.add(name = "from_root")
use_repo(e, "from_root", "zzz")
"##,
        )],
        fetch: &[r#"lib++collect+from_a"#],
        error: Some(
            r##"module extension @@lib+//:ext.bzl%collect does not generate repository "zzz", yet it is imported as "zzz" in the usage at MODULE.bazel:4:18"##,
        ),
        printed: &[
            r##"root@0 root=True tags=["from_root"]; a@1.0 root=False tags=["from_a"] nd=True"##,
        ],
        builds: &[],
    },
    MultiRow {
        name: r#"extension_loads_from_a_module_it_depends_on"#,
        registry: &[
            (
                r#"helper"#,
                r#"1.0"#,
                &[
                    (
                        r#"MODULE.bazel"#,
                        r##"module(name = "helper", version = "1.0")
"##,
                    ),
                    (
                        r#"h.bzl"#,
                        r##"TAG = "from_helper"
"##,
                    ),
                ],
            ),
            (
                r#"lib"#,
                r#"1.0"#,
                &[
                    (
                        r#"MODULE.bazel"#,
                        r##"module(name = "lib", version = "1.0")
bazel_dep(name = "helper", version = "1.0")
"##,
                    ),
                    (
                        r#"ext.bzl"#,
                        r###"load("@helper//:h.bzl", "TAG")
def _r(rctx):
    rctx.file("BUILD.bazel", "# " + rctx.attr.info + "\nfilegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"info": attr.string()})

_add = tag_class(attrs = {"name": attr.string(mandatory = True)})

def _impl(mctx):
    lines = []
    for m in mctx.modules:
        lines.append("%s@%s root=%s tags=%s" % (m.name, m.version, m.is_root, [t.name for t in m.tags.add]))
    info = "; ".join(lines) + " nd=" + str(mctx.root_module_has_non_dev_dependency)
    print(info + " " + TAG)
    for m in mctx.modules:
        for t in m.tags.add:
            made(name = t.name, info = info)

collect = module_extension(implementation = _impl, tag_classes = {"add": _add})
"###,
                    ),
                ],
            ),
        ],
        root: &[(
            r#"MODULE.bazel"#,
            r##"module(name = "root", version = "0")
bazel_dep(name = "lib", version = "1.0")
e = use_extension("@lib//:ext.bzl", "collect")
e.add(name = "x")
use_repo(e, "x")
"##,
        )],
        fetch: &[r#"lib++collect+x"#],
        error: None,
        printed: &[r##"root@0 root=True tags=["x"] nd=True from_helper"##],
        builds: &[(
            r#"lib++collect+x"#,
            r##"# root@0 root=True tags=["x"] nd=True
filegroup(name = 'f')"##,
        )],
    },
    MultiRow {
        name: r#"extension_loads_from_a_repository_another_extension_made"#,
        registry: &[(
            r#"lib"#,
            r#"1.0"#,
            &[
                (
                    r#"MODULE.bazel"#,
                    r##"module(name = "lib", version = "1.0")
g = use_extension("//:gen.bzl", "genx")
use_repo(g, "gen")
"##,
                ),
                (
                    r#"gen.bzl"#,
                    r##"def _r(rctx):
    rctx.file("BUILD.bazel", "")
    rctx.file("defs.bzl", "VALUE = 'generated'\n")

maker = repository_rule(implementation = _r)

def _impl(mctx):
    print("genx runs")
    maker(name = "gen")

genx = module_extension(implementation = _impl)
"##,
                ),
                (
                    r#"ext.bzl"#,
                    r###"load("@gen//:defs.bzl", "VALUE")
def _r(rctx):
    rctx.file("BUILD.bazel", "# " + rctx.attr.info + "\nfilegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"info": attr.string()})

_add = tag_class(attrs = {"name": attr.string(mandatory = True)})

def _impl(mctx):
    lines = []
    for m in mctx.modules:
        lines.append("%s@%s root=%s tags=%s" % (m.name, m.version, m.is_root, [t.name for t in m.tags.add]))
    info = "; ".join(lines) + " nd=" + str(mctx.root_module_has_non_dev_dependency)
    print(info + " " + VALUE)
    for m in mctx.modules:
        for t in m.tags.add:
            made(name = t.name, info = info)

collect = module_extension(implementation = _impl, tag_classes = {"add": _add})
"###,
                ),
            ],
        )],
        root: &[(
            r#"MODULE.bazel"#,
            r##"module(name = "root", version = "0")
bazel_dep(name = "lib", version = "1.0")
e = use_extension("@lib//:ext.bzl", "collect")
e.add(name = "x")
use_repo(e, "x")
"##,
        )],
        fetch: &[r#"lib++collect+x"#],
        error: None,
        printed: &[
            r#"genx runs"#,
            r##"root@0 root=True tags=["x"] nd=True generated"##,
        ],
        builds: &[(
            r#"lib++collect+x"#,
            r##"# root@0 root=True tags=["x"] nd=True
filegroup(name = 'f')"##,
        )],
    },
    MultiRow {
        name: r#"a_dependency_calls_a_repository_rule_of_another_module"#,
        registry: &[
            (
                r#"lib"#,
                r#"1.0"#,
                &[
                    (
                        r#"MODULE.bazel"#,
                        r##"module(name = "lib", version = "1.0")
"##,
                    ),
                    (
                        r#"rules.bzl"#,
                        r###"def _r(rctx):
    rctx.file("BUILD.bazel", "# " + rctx.attr.info + " " + rctx.name + " " + rctx.original_name + "\nfilegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"info": attr.string()})
"###,
                    ),
                ],
            ),
            (
                r#"a"#,
                r#"1.0"#,
                &[(
                    r#"MODULE.bazel"#,
                    r##"module(name = "a", version = "1.0")
bazel_dep(name = "lib", version = "1.0")
made = use_repo_rule("@lib//:rules.bzl", "made")
made(name = "mine", info = "from a")
"##,
                )],
            ),
        ],
        root: &[(
            r#"MODULE.bazel"#,
            r##"module(name = "root", version = "0")
bazel_dep(name = "a", version = "1.0")
"##,
        )],
        fetch: &[r#"a++made+mine"#],
        error: None,
        printed: &[],
        builds: &[(
            r#"a++made+mine"#,
            r#"# from a a++made+mine mine
filegroup(name = 'f')"#,
        )],
    },
];
