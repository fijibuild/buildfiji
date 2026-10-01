//! Generated from probes of Bazel 9.2.0 (`bazel build` with `--lockfile_mode=update`):
//! the `moduleExtensions` it wrote; see `lock_tests.rs`.

use crate::lock_tests::LockRow;

pub(crate) const LOCK_ROWS: &[LockRow] = &[
    LockRow {
        name: r#"basic"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
e.t(name = "x")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict()})

_t = tag_class(attrs = {"name": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "o": attr.string(), "lab": attr.label(), "sld": attr.string_list_dict()})

def _impl(mctx):
    for m in mctx.modules:
        for t in m.tags.t:
            made(name = t.name, v = "x")

collect = module_extension(implementation = _impl, tag_classes = {"t": _t})
"#,
            ),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"QbRTdUSjsmE1Hre6G2+tOwHQlQXHHXK2RrApE+2f1w4=","usagesDigest":"6MuuqwoDWLhVH1uyM4cECDqVoDPWWaU5TPbGH+qEw0w=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{"v":"x"}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"repo_name"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root", repo_name = "rr")
e = use_extension("//:ext.bzl", "collect")
e.t(name = "x")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict()})

_t = tag_class(attrs = {"name": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "o": attr.string(), "lab": attr.label(), "sld": attr.string_list_dict()})

def _impl(mctx):
    for m in mctx.modules:
        for t in m.tags.t:
            made(name = t.name, v = "x")

collect = module_extension(implementation = _impl, tag_classes = {"t": _t})
"#,
            ),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"QbRTdUSjsmE1Hre6G2+tOwHQlQXHHXK2RrApE+2f1w4=","usagesDigest":"FDgQkyxqxOyFRBP2Aaa/rJTPGdtoDwS8ytVwgZwLMgg=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{"v":"x"}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"colon_label"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension(":ext.bzl", "collect")
e.t(name = "x")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict()})

_t = tag_class(attrs = {"name": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "o": attr.string(), "lab": attr.label(), "sld": attr.string_list_dict()})

def _impl(mctx):
    for m in mctx.modules:
        for t in m.tags.t:
            made(name = t.name, v = "x")

collect = module_extension(implementation = _impl, tag_classes = {"t": _t})
"#,
            ),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"QbRTdUSjsmE1Hre6G2+tOwHQlQXHHXK2RrApE+2f1w4=","usagesDigest":"6MuuqwoDWLhVH1uyM4cECDqVoDPWWaU5TPbGH+qEw0w=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{"v":"x"}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"pkg_label"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//sub:ext.bzl", "collect")
e.t(name = "x")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict()})

_t = tag_class(attrs = {"name": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "o": attr.string(), "lab": attr.label(), "sld": attr.string_list_dict()})

def _impl(mctx):
    for m in mctx.modules:
        for t in m.tags.t:
            made(name = t.name, v = "x")

collect = module_extension(implementation = _impl, tag_classes = {"t": _t})
"#,
            ),
            (
                r#"sub/ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict()})

_t = tag_class(attrs = {"name": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "o": attr.string(), "lab": attr.label(), "sld": attr.string_list_dict()})

def _impl(mctx):
    for m in mctx.modules:
        for t in m.tags.t:
            made(name = t.name, v = "x")

collect = module_extension(implementation = _impl, tag_classes = {"t": _t})
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//sub:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"QbRTdUSjsmE1Hre6G2+tOwHQlQXHHXK2RrApE+2f1w4=","usagesDigest":"PbTSQp85kX+jit7Rfy6/9Lk140Fal+HqweOsmHUW55A=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//sub:ext.bzl%made","attributes":{"v":"x"}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"two_extensions"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
e.t(name = "x")
use_repo(e, "x")
f = use_extension("//:ext.bzl", "collect")
f.t(name = "y")
use_repo(f, "y")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict()})

_t = tag_class(attrs = {"name": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "o": attr.string(), "lab": attr.label(), "sld": attr.string_list_dict()})

def _impl(mctx):
    for m in mctx.modules:
        for t in m.tags.t:
            made(name = t.name, v = "x")

collect = module_extension(implementation = _impl, tag_classes = {"t": _t})
"#,
            ),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"QbRTdUSjsmE1Hre6G2+tOwHQlQXHHXK2RrApE+2f1w4=","usagesDigest":"alExdup/PeZb57bdXp8tGSBVruOn/QgjX6PfmJvjKlc=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{"v":"x"}},"y":{"repoRuleId":"@@//:ext.bzl%made","attributes":{"v":"x"}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"tag_values"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
e.t(name = "x", n = 3, b = True, l = ["a", "b"], d = {"k": "v", "k2": "v2"}, o = "s", lab = "//:ext.bzl", sld = {"a": ["1", "2"]})
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict()})

_t = tag_class(attrs = {"name": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "o": attr.string(), "lab": attr.label(), "sld": attr.string_list_dict()})

def _impl(mctx):
    for m in mctx.modules:
        for t in m.tags.t:
            made(name = t.name, v = "x")

collect = module_extension(implementation = _impl, tag_classes = {"t": _t})
"#,
            ),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"QbRTdUSjsmE1Hre6G2+tOwHQlQXHHXK2RrApE+2f1w4=","usagesDigest":"4AqoH8SLoW8XyCzR2PVcgHCWeByJvWQJXOCCQIZeu0M=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{"v":"x"}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"dev_tag"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
e.t(name = "x")
use_repo(e, "x")
d = use_extension("//:ext.bzl", "collect", dev_dependency = True)
d.t(name = "y")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict()})

_t = tag_class(attrs = {"name": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "o": attr.string(), "lab": attr.label(), "sld": attr.string_list_dict()})

def _impl(mctx):
    for m in mctx.modules:
        for t in m.tags.t:
            made(name = t.name, v = "x")

collect = module_extension(implementation = _impl, tag_classes = {"t": _t})
"#,
            ),
        ],
        fetch: &[r#"+collect+x"#, r#"+collect+y"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"QbRTdUSjsmE1Hre6G2+tOwHQlQXHHXK2RrApE+2f1w4=","usagesDigest":"J7VjBGgXDjVoBI99KYwN7R9LLqSnPBOmr7u80Dep0gM=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{"v":"x"}},"y":{"repoRuleId":"@@//:ext.bzl%made","attributes":{"v":"x"}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"root_version"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root", version = "2.5")
e = use_extension("//:ext.bzl", "collect")
e.t(name = "x")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict()})

_t = tag_class(attrs = {"name": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "o": attr.string(), "lab": attr.label(), "sld": attr.string_list_dict()})

def _impl(mctx):
    for m in mctx.modules:
        for t in m.tags.t:
            made(name = t.name, v = "x")

collect = module_extension(implementation = _impl, tag_classes = {"t": _t})
"#,
            ),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"QbRTdUSjsmE1Hre6G2+tOwHQlQXHHXK2RrApE+2f1w4=","usagesDigest":"3gK6WlmvWGMk+JvXefJuXUWsKP+bXIOsAguh9HCV+uA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{"v":"x"}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"two_modules"#,
        registry: &[
            (
                r#"lib"#,
                r#"1.0"#,
                &[
                    (
                        r#"MODULE.bazel"#,
                        r#"module(name = "lib", version = "1.0")
"#,
                    ),
                    (
                        r#"ext.bzl"#,
                        r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict()})

_t = tag_class(attrs = {"name": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "o": attr.string(), "lab": attr.label(), "sld": attr.string_list_dict()})

def _impl(mctx):
    for m in mctx.modules:
        for t in m.tags.t:
            made(name = t.name, v = "x")

collect = module_extension(implementation = _impl, tag_classes = {"t": _t})
"#,
                    ),
                ],
            ),
            (
                r#"a"#,
                r#"1.0"#,
                &[(
                    r#"MODULE.bazel"#,
                    r#"module(name = "a", version = "1.0")
bazel_dep(name = "lib", version = "1.0")
e = use_extension("@lib//:ext.bzl", "collect")
e.t(name = "from_a")
use_repo(e, "from_a")
"#,
                )],
            ),
        ],
        root: &[(
            r#"MODULE.bazel"#,
            r#"module(name = "root")
bazel_dep(name = "lib", version = "1.0")
bazel_dep(name = "a", version = "1.0")
e = use_extension("@lib//:ext.bzl", "collect")
e.t(name = "from_root")
use_repo(e, "from_root")
"#,
        )],
        fetch: &[r#"lib++collect+from_root"#],
        env: &[],
        expected: r#"{"@@lib+//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"QbRTdUSjsmE1Hre6G2+tOwHQlQXHHXK2RrApE+2f1w4=","usagesDigest":"o4ZqIAqxfLaa6WQnmcQZXXoM1C0ahqvlowzwJhMNv3g=","recordedInputs":[],"generatedRepoSpecs":{"from_root":{"repoRuleId":"@@lib+//:ext.bzl%made","attributes":{"v":"x"}},"from_a":{"repoRuleId":"@@lib+//:ext.bzl%made","attributes":{"v":"x"}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"aliased_dep"#,
        registry: &[(
            r#"lib"#,
            r#"1.0"#,
            &[
                (
                    r#"MODULE.bazel"#,
                    r#"module(name = "lib", version = "1.0")
"#,
                ),
                (
                    r#"ext.bzl"#,
                    r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict()})

_t = tag_class(attrs = {"name": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "o": attr.string(), "lab": attr.label(), "sld": attr.string_list_dict()})

def _impl(mctx):
    for m in mctx.modules:
        for t in m.tags.t:
            made(name = t.name, v = "x")

collect = module_extension(implementation = _impl, tag_classes = {"t": _t})
"#,
                ),
            ],
        )],
        root: &[(
            r#"MODULE.bazel"#,
            r#"module(name = "root")
bazel_dep(name = "lib", version = "1.0", repo_name = "ll")
e = use_extension("@ll//:ext.bzl", "collect")
e.t(name = "x")
use_repo(e, "x")
"#,
        )],
        fetch: &[r#"lib++collect+x"#],
        env: &[],
        expected: r#"{"@@lib+//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"QbRTdUSjsmE1Hre6G2+tOwHQlQXHHXK2RrApE+2f1w4=","usagesDigest":"flsQXN7ZtRz9uR/yuVr/aXZNiTKjHLN6c3e+1QcSAok=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@lib+//:ext.bzl%made","attributes":{"v":"x"}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"dep_module_repo_name"#,
        registry: &[
            (
                r#"lib"#,
                r#"1.0"#,
                &[
                    (
                        r#"MODULE.bazel"#,
                        r#"module(name = "lib", version = "1.0")
"#,
                    ),
                    (
                        r#"ext.bzl"#,
                        r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict()})

_t = tag_class(attrs = {"name": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "o": attr.string(), "lab": attr.label(), "sld": attr.string_list_dict()})

def _impl(mctx):
    for m in mctx.modules:
        for t in m.tags.t:
            made(name = t.name, v = "x")

collect = module_extension(implementation = _impl, tag_classes = {"t": _t})
"#,
                    ),
                ],
            ),
            (
                r#"a"#,
                r#"1.0"#,
                &[
                    (
                        r#"MODULE.bazel"#,
                        r#"module(name = "a", version = "1.0", repo_name = "aa")
bazel_dep(name = "lib", version = "1.0")
e = use_extension("@lib//:ext.bzl", "collect")
e.t(name = "from_a")
use_repo(e, "from_a")
"#,
                    ),
                    (
                        r#"ext.bzl"#,
                        r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict()})

_t = tag_class(attrs = {"name": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "o": attr.string(), "lab": attr.label(), "sld": attr.string_list_dict()})

def _impl(mctx):
    for m in mctx.modules:
        for t in m.tags.t:
            made(name = t.name, v = "x")

collect = module_extension(implementation = _impl, tag_classes = {"t": _t})
"#,
                    ),
                ],
            ),
        ],
        root: &[(
            r#"MODULE.bazel"#,
            r#"module(name = "root")
bazel_dep(name = "lib", version = "1.0")
bazel_dep(name = "a", version = "1.0")
"#,
        )],
        fetch: &[r#"lib++collect+from_a"#],
        env: &[],
        expected: r#"{"@@lib+//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"QbRTdUSjsmE1Hre6G2+tOwHQlQXHHXK2RrApE+2f1w4=","usagesDigest":"UcgUFcGfrZos6nEdEConv9LlmFtwetGz5hOJJGlL2tk=","recordedInputs":[],"generatedRepoSpecs":{"from_a":{"repoRuleId":"@@lib+//:ext.bzl%made","attributes":{"v":"x"}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"dep_local_ext"#,
        registry: &[(
            r#"a"#,
            r#"1.0"#,
            &[
                (
                    r#"MODULE.bazel"#,
                    r#"module(name = "a", version = "1.0")
e = use_extension("//:ext.bzl", "collect")
e.t(name = "from_a")
use_repo(e, "from_a")
"#,
                ),
                (
                    r#"ext.bzl"#,
                    r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict()})

_t = tag_class(attrs = {"name": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "o": attr.string(), "lab": attr.label(), "sld": attr.string_list_dict()})

def _impl(mctx):
    for m in mctx.modules:
        for t in m.tags.t:
            made(name = t.name, v = "x")

collect = module_extension(implementation = _impl, tag_classes = {"t": _t})
"#,
                ),
            ],
        )],
        root: &[(
            r#"MODULE.bazel"#,
            r#"module(name = "root")
bazel_dep(name = "a", version = "1.0")
"#,
        )],
        fetch: &[r#"a++collect+from_a"#],
        env: &[],
        expected: r#"{"@@a+//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"QbRTdUSjsmE1Hre6G2+tOwHQlQXHHXK2RrApE+2f1w4=","usagesDigest":"vobFnHFi0rHonxADVOhHyhGMLE4u0nXSjgPFr0WlQes=","recordedInputs":[],"generatedRepoSpecs":{"from_a":{"repoRuleId":"@@a+//:ext.bzl%made","attributes":{"v":"x"}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"attr_types"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x", v = "s", n = 3, b = True, l = ["a", "b"], d = {"k": "v"}, lab = "//:data.txt", labs = ["//:data.txt", "@bazel_tools//:x"], ld = {"//:data.txt": "val"}, sld = {"a": ["1", "2"]}, il = [1, 2])

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"3nsNkXi13KM5OvQwdfU9QVXXB8L/ZVqNIRhdTWHGST0=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{"v":"s","n":3,"b":true,"l":["a","b"],"d":{"k":"v"},"lab":"@@//:data.txt","labs":["@@//:data.txt","@@bazel_tools//:x"],"ld":{"@@//:data.txt":"val"},"sld":{"a":["1","2"]},"il":[1,2]}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"label_objects"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x", lab = Label("//:data.txt"), labs = [Label("//sub:y")])

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"DBZABzODvJHPRw9CiUwfsiIgyYhCjazscGVnSOBl8Sg=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{"lab":"@@//:data.txt","labs":["@@//sub:y"]}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"none_and_empty"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x", v = None, l = [], d = {})

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"9g+1vLNYNoNFhdmGkv9u4botZyai+f9vyg3t+E5rX3I=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{"l":[],"d":{}}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"read_file"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    mctx.read(mctx.path(Label("//:data.txt")))
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"xEVc84ToLgld6VE8KmKbe3NiRIhvRydbqMstDwPxxYg=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":["FILE:@@//data.txt 5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"read_path_str"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    mctx.read("/etc/hostname")
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"EiSoidYyZRGUF5DSEBbeDhp4BB+lPdKJal658SQokRw=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"getenv"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    mctx.getenv("FJFJ_PROBE_A")
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[(r#"FJFJ_PROBE_A"#, r#"1"#)],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"CXNkiwetlsIWeiTG0/WF76ftEVIZ2a6/jnc3UEPyCec=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":["ENV:FJFJ_PROBE_A 1"],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"getenv_default"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    mctx.getenv("FJFJ_PROBE_B", "dflt")
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"+wz6OIn2DygPtp4+MA4GbJIZVpGqR5n6wMv5cwHAfHc=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":["ENV:FJFJ_PROBE_B \\0"],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"environ"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    mctx.os.environ.get("FJFJ_PROBE_C")
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"wTmN4sBmYQQpJQ2Pd4GDbc/2GkFnScHAbKDRp8mXjYI=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"file_exists"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    mctx.path(Label("//:data.txt")).exists
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"FTXOLyPN0MLtwdSTsBFKJmRDeaAmHqmjuIs15YdoY+Q=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"readdir"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    mctx.path(Label("//sub:BUILD.bazel")).dirname.readdir()
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"thmE150jxvsAkiyygMgu3uHlAULj77KbBAzubYJ9oAI=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":["DIRENTS:@@//sub 1ac7d3bfc6cdfc8f606b974455a519dfb62c18390c8c446c344937ae429fc122"],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"watch"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    mctx.watch(mctx.path(Label("//:data.txt")))
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"DMN01lrecX39/zJxp7zeWDUAZBBEJ/nWRh8b5BMmgWE=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":["FILE:@@//data.txt 5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"execute"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    mctx.execute(["true"])
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"7Tpu1iCmwV/bsDZXPV0Yt9YgWUqgBvm/XjGSrO6Bqk4=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"os_dependent"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")

collect = module_extension(implementation = _impl, os_dependent = True)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"os:linux":{"bzlTransitiveDigest":"EgvYQgIIPn3mZJN5YbYzmVtux5vw6A04m8/MDWnlFyo=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"arch_dependent"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")

collect = module_extension(implementation = _impl, arch_dependent = True)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"arch:amd64":{"bzlTransitiveDigest":"T9SM0NKALT5F+RAfU2cxbBxk3lYrByUTA6hm9ytGXhM=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"both_dependent"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")

collect = module_extension(implementation = _impl, os_dependent = True, arch_dependent = True)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"os:linux,arch:amd64":{"bzlTransitiveDigest":"qU8U6spxfKyBFqQpx7cIiK2qA/EkLlycRvCaQrvl/MY=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"reproducible"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    return mctx.extension_metadata(reproducible = True)

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"which"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    mctx.which("sh")
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"rT5fj63YuRADKoU2fu4fhXKI9lP4FxUOWYjmed6IgXc=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"download_none"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    print(mctx.os.name)

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"Knttoytcdt9Jpf+caVZzx0jk5UtWmIKwA9svbjR3Fm4=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"order"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    mctx.read(mctx.path(Label("//:b.txt")))
    mctx.getenv("ZZ_P")
    mctx.getenv("AA_P")
    mctx.read(mctx.path(Label("//:a.txt")))
    mctx.path(Label("//sub:BUILD.bazel")).dirname.readdir()
    mctx.read(mctx.path(Label("//:b.txt")))
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
            (
                r#"a.txt"#, r#"A
"#,
            ),
            (
                r#"b.txt"#, r#"B
"#,
            ),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"d92K2C+bf3xlJH1HBpeCyuemzwvFhtgncD9wF2A+dqk=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":["FILE:@@//b.txt c0cde77fa8fef97d476c10aad3d2d54fcc2f336140d073651c2dcccf1e379fd6","ENV:ZZ_P \\0","ENV:AA_P \\0","FILE:@@//a.txt 06f961b802bc46ee168555f066d28f4f0e9afdf3f88174c1ee6f9de004fc30a0","DIRENTS:@@//sub 1ac7d3bfc6cdfc8f606b974455a519dfb62c18390c8c446c344937ae429fc122"],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"abs_path_in_ws"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    mctx.read(mctx.path(Label("//:a.txt")).realpath)
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
            (
                r#"a.txt"#, r#"A
"#,
            ),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"SwZP2TyUPb03BSUEFhMw7OS2+ctFyhHe30oH2UsCQwM=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":["FILE:@@//a.txt 06f961b802bc46ee168555f066d28f4f0e9afdf3f88174c1ee6f9de004fc30a0"],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"subdir_file"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    mctx.read(mctx.path(Label("//sub:f.txt")))
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
            (
                r#"sub/f.txt"#,
                r#"F
"#,
            ),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"snp1CMOy8+zt5IUQEf14LS0zu1F+vPnFnNds3WSV9VU=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":["FILE:@@//sub/f.txt e2ca2771fc7c542bcdeeb6065a6e872ff2f2d263a19005b55f63311c0a8f1fa9"],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"missing_file_exists"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    mctx.path(Label("//:none.txt")).exists
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"+KWT56jNuHlnx2rI+tQSQfU5sbj9qnfRMF40iyjfJEE=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"getenv_unset_nodefault"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    mctx.getenv("FJFJ_UNSET_Q")
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"JMUV+495kBi1DkFK9inyXk3kmfEelyoX9npN/rSxYYs=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":["ENV:FJFJ_UNSET_Q \\0"],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"getenv_value_with_space"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    mctx.getenv("FJFJ_SP")
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[(r#"FJFJ_SP"#, r#"a b  c"#)],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"Aebwkw1eYUgKlTvYoqQCrdfMMfJSzJD+ZjcxDvMZA6M=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":["ENV:FJFJ_SP a\\sb\\s\\sc"],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"deps_match"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x", "y")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = ["x", "y"], root_module_direct_dev_deps = [])

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"WEnKURugwFtGw10qa70UMO3p3B9UUXPpO1VluNEbKi4=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}},"y":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}},"moduleExtensionMetadata":{"explicitRootModuleDirectDeps":["x","y"],"explicitRootModuleDirectDevDeps":[],"useAllRepos":"NO","reproducible":false}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"deps_missing_import"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = ["x", "y"], root_module_direct_dev_deps = [])

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"WEnKURugwFtGw10qa70UMO3p3B9UUXPpO1VluNEbKi4=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}},"y":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}},"moduleExtensionMetadata":{"explicitRootModuleDirectDeps":["x","y"],"explicitRootModuleDirectDevDeps":[],"useAllRepos":"NO","reproducible":false}}}}"#,
        facts: r#"{}"#,
        warnings: &[
            r#"MODULE.bazel:2:18: The module extension collect defined in @root//:ext.bzl reported incorrect imports of repositories via use_repo():

Not imported, but reported as direct dependencies by the extension (may cause the build to fail):
    y

Fix the use_repo calls by running 'bazel mod tidy'."#,
        ],
        error: None,
    },
    LockRow {
        name: r#"deps_extra_import"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x", "y")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = ["x"], root_module_direct_dev_deps = [])

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"0vxamfm0nt42zLS0hM3RLkm1VV1sVRSapTSQ546hOh0=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}},"y":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}},"moduleExtensionMetadata":{"explicitRootModuleDirectDeps":["x"],"explicitRootModuleDirectDevDeps":[],"useAllRepos":"NO","reproducible":false}}}}"#,
        facts: r#"{}"#,
        warnings: &[
            r#"MODULE.bazel:2:18: The module extension collect defined in @root//:ext.bzl reported incorrect imports of repositories via use_repo():

Imported, but reported as indirect dependencies by the extension:
    y

Fix the use_repo calls by running 'bazel mod tidy'."#,
        ],
        error: None,
    },
    LockRow {
        name: r#"deps_all"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = "all")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: Some(
            r#"if one of root_module_direct_deps and root_module_direct_dev_deps is "all", the other must be an empty list"#,
        ),
    },
    LockRow {
        name: r#"deps_all_dev"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = [], root_module_direct_dev_deps = "all")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: Some(
            r#"root_module_direct_dev_deps must be empty if the root module contains no usages with dev_dependency = True"#,
        ),
    },
    LockRow {
        name: r#"deps_not_generated"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = ["x", "zzz"], root_module_direct_dev_deps = [])

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: Some(
            r#"root_module_direct_deps contained the following repositories not generated by the extension: zzz"#,
        ),
    },
    LockRow {
        name: r#"only_direct"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x", "y")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = ["x", "y"])

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: Some(
            r#"root_module_direct_deps and root_module_direct_dev_deps must both be specified or both be unspecified"#,
        ),
    },
    LockRow {
        name: r#"only_dev"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x", "y")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_dev_deps = ["x", "y"])

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: Some(
            r#"root_module_direct_deps and root_module_direct_dev_deps must both be specified or both be unspecified"#,
        ),
    },
    LockRow {
        name: r#"bad_type"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = 3)

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: Some(
            r#"in call to extension_metadata(), parameter 'root_module_direct_deps' got value of type 'int', want 'sequence, string, or NoneType'"#,
        ),
    },
    LockRow {
        name: r#"deps_dev_overlap"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = ["x"], root_module_direct_dev_deps = ["x"])

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: Some(
            r#"in root_module_direct_dev_deps: entry 'x' is also in root_module_direct_deps"#,
        ),
    },
    LockRow {
        name: r#"facts_set"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(facts = {"a": 1})

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"km4JxbRsmeYYqRMPsxO2Nw+xqKONDbq3bhV7Y33lrS0=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}},"y":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}},"moduleExtensionMetadata":{"useAllRepos":"NO","reproducible":false}}}}"#,
        facts: r#"{"//:ext.bzl%collect":{"a":1}}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"deps_none"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = None, root_module_direct_dev_deps = None)

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"63QJIxXSUWy3Yb6XHFljjM2xOwXFhdNyXxatCQkQNww=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}},"y":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"deps_reproducible"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(reproducible = True)

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"deps_repro_false"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(reproducible = False)

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"xQx7mq656vlCp6GcUJfYF6+YneotFoTJYSQkgxcAa+M=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}},"y":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"deps_some_lists"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = ["y", "x"], root_module_direct_dev_deps = [])

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"dUkfl3upxOXjTZGVIwjYQFCzNN8dPKzp7qQsjk3UAto=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}},"y":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}},"moduleExtensionMetadata":{"explicitRootModuleDirectDeps":["y","x"],"explicitRootModuleDirectDevDeps":[],"useAllRepos":"NO","reproducible":false}}}}"#,
        facts: r#"{}"#,
        warnings: &[
            r#"MODULE.bazel:2:18: The module extension collect defined in @root//:ext.bzl reported incorrect imports of repositories via use_repo():

Not imported, but reported as direct dependencies by the extension (may cause the build to fail):
    y

Fix the use_repo calls by running 'bazel mod tidy'."#,
        ],
        error: None,
    },
    LockRow {
        name: r#"facts_only"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(facts = {"a": 1})

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"km4JxbRsmeYYqRMPsxO2Nw+xqKONDbq3bhV7Y33lrS0=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}},"y":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}},"moduleExtensionMetadata":{"useAllRepos":"NO","reproducible":false}}}}"#,
        facts: r#"{"//:ext.bzl%collect":{"a":1}}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"facts_empty"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(facts = {})

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"QB2a6g67NdXrql9JnLAJeshyeOEQeQFAkmSfkSJeKd8=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}},"y":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"facts_nested"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(facts = {"z": 1, "a": {"q": [1, 2, None], "b": "s"}, "m": 2.5})

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"RNahlpOWibrxvX9RO6NT9IoTvrH/cUzNrMvJ2uWFTAA=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}},"y":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}},"moduleExtensionMetadata":{"useAllRepos":"NO","reproducible":false}}}}"#,
        facts: r#"{"//:ext.bzl%collect":{"a":{"b":"s","q":[1,2,null]},"m":2.5,"z":1}}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"facts_not_dict"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(facts = 3)

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: Some(
            r#"in call to extension_metadata(), parameter 'facts' got value of type 'int', want 'dict'"#,
        ),
    },
    LockRow {
        name: r#"dev_dev_deps"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
d = use_extension("//:ext.bzl", "collect", dev_dependency = True)
use_repo(d, "y")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = ["x"], root_module_direct_dev_deps = ["y"])

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"acx9JcTyT0Qp9Xk1n6jEbx4KHkNq4ztLJVR16TknoOI=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}},"y":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}},"moduleExtensionMetadata":{"explicitRootModuleDirectDeps":["x"],"explicitRootModuleDirectDevDeps":["y"],"useAllRepos":"NO","reproducible":false}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"dev_swapped"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
d = use_extension("//:ext.bzl", "collect", dev_dependency = True)
use_repo(d, "y")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = ["y"], root_module_direct_dev_deps = ["x"])

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"LwRToEm0vvhbl3Zv+fTlSbP6g1E5rDUmnugUKjpaBlI=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}},"y":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}},"moduleExtensionMetadata":{"explicitRootModuleDirectDeps":["y"],"explicitRootModuleDirectDevDeps":["x"],"useAllRepos":"NO","reproducible":false}}}}"#,
        facts: r#"{}"#,
        warnings: &[
            r#"MODULE.bazel:2:18: The module extension collect defined in @root//:ext.bzl reported incorrect imports of repositories via use_repo():

Imported as a regular dependency, but reported as a dev dependency by the extension (may cause the build to fail when used by other modules):
    x

Imported as a dev dependency, but reported as a regular dependency by the extension (may cause the build to fail when used by other modules):
    y

Fix the use_repo calls by running 'bazel mod tidy'."#,
        ],
        error: None,
    },
    LockRow {
        name: r#"dev_all_dev"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
d = use_extension("//:ext.bzl", "collect", dev_dependency = True)
use_repo(d, "y")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = [], root_module_direct_dev_deps = "all")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"igEQ4Y4D46BbRKERbfc8KjdXcijGBNpdYZxZEZsrqlM=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}},"y":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}},"moduleExtensionMetadata":{"useAllRepos":"DEV","reproducible":false}}}}"#,
        facts: r#"{}"#,
        warnings: &[
            r#"MODULE.bazel:2:18: The module extension collect defined in @root//:ext.bzl reported incorrect imports of repositories via use_repo():

Imported as a regular dependency, but reported as a dev dependency by the extension (may cause the build to fail when used by other modules):
    x

Fix the use_repo calls by running 'bazel mod tidy'."#,
        ],
        error: None,
    },
    LockRow {
        name: r#"many_missing"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = ["y", "x", "b", "a"], root_module_direct_dev_deps = [])

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: Some(
            r#"root_module_direct_deps contained the following repositories not generated by the extension: b, a"#,
        ),
    },
    LockRow {
        name: r#"unordered_extra"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "y", "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = ["x"], root_module_direct_dev_deps = [])

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"0vxamfm0nt42zLS0hM3RLkm1VV1sVRSapTSQ546hOh0=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}},"y":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}},"moduleExtensionMetadata":{"explicitRootModuleDirectDeps":["x"],"explicitRootModuleDirectDevDeps":[],"useAllRepos":"NO","reproducible":false}}}}"#,
        facts: r#"{}"#,
        warnings: &[
            r#"MODULE.bazel:2:18: The module extension collect defined in @root//:ext.bzl reported incorrect imports of repositories via use_repo():

Imported, but reported as indirect dependencies by the extension:
    y

Fix the use_repo calls by running 'bazel mod tidy'."#,
        ],
        error: None,
    },
    LockRow {
        name: r#"dup_entry"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = ["x", "x"], root_module_direct_dev_deps = [])

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: Some(r#"in root_module_direct_deps: duplicate entry 'x'"#),
    },
    LockRow {
        name: r#"non_string_entry"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")
    made(name = "y")
    return mctx.extension_metadata(root_module_direct_deps = ["x", 3], root_module_direct_dev_deps = [])

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: Some(
            r#"at index 1 of root_module_direct_deps, got element of type int, want string"#,
        ),
    },
    LockRow {
        name: r#"mapping_label"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    Label("@bazel_tools//:x")
    Label("@nosuch//:y")
    Label("@root//:z")
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"EJX686tWW8Bw8m77UaGhdBrOTBhJZvYGOD+YXfrMZGY=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":["REPO_MAPPING:,bazel_tools bazel_tools","REPO_MAPPING:,nosuch \\0","REPO_MAPPING:,root "],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"mapping_load"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")
def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"vMas1xvo8adzEWhwlEBLgSKgVpC8bXHQX6xy9VukMR8=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":["REPO_MAPPING:,bazel_tools bazel_tools"],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"mapping_transitive"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"load("//:helper.bzl", "H")
def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
            (
                r#"helper.bzl"#,
                r#"load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")
H = Label("@nosuch2//:q")
H2 = Label("@@canon//:q")
H3 = Label("//:own")
"#,
            ),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"ODP5aF+lOY/w2ctAelNwaHAvqc3PEzBMswHtEkqUXh0=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":["REPO_MAPPING:,bazel_tools bazel_tools","REPO_MAPPING:,nosuch2 \\0"],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"mapping_with_env"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    mctx.getenv("FJFJ_PROBE_A")
    Label("@nosuch//:y")
    mctx.read(mctx.path(Label("//:data.txt")))
    made(name = "x")

collect = module_extension(implementation = _impl)
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[(r#"FJFJ_PROBE_A"#, r#"1"#)],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"eYN0IRr1w274l92BPhyjpFmcHbr+Nj2K8JmJC9pYpEM=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":["ENV:FJFJ_PROBE_A 1","REPO_MAPPING:,nosuch \\0","FILE:@@//data.txt 5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"mapping_attr_default"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")

collect = module_extension(implementation = _impl, tag_classes = {"t": tag_class(attrs = {"l": attr.label_list(default = ["@nosuch3//:x"])})})
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"Lnv/5lpcMVDY9v5pBrtUWmS76R5yFtey9uilj/DXgUs=","usagesDigest":"F+8iDTzTJ/5jCXyE+lMDbP4LtQig4r6YwvJE56X/qgA=","recordedInputs":[],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
    LockRow {
        name: r#"mapping_tag_label"#,
        registry: &[],
        root: &[
            (
                r#"MODULE.bazel"#,
                r#"module(name = "root")
e = use_extension("//:ext.bzl", "collect")
use_repo(e, "x")
e.t(lab = "@bazel_tools//:x", labs = ["@root//:y", "//:z"])
"#,
            ),
            (
                r#"ext.bzl"#,
                r#"def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
    made(name = "x")

collect = module_extension(implementation = _impl, tag_classes = {"t": tag_class(attrs = {"lab": attr.label(), "labs": attr.label_list()})})
"#,
            ),
            (
                r#"data.txt"#,
                r#"hello
"#,
            ),
            (r#"sub/BUILD.bazel"#, r#""#),
        ],
        fetch: &[r#"+collect+x"#],
        env: &[],
        expected: r#"{"//:ext.bzl%collect":{"general":{"bzlTransitiveDigest":"ICkc3oMJXpZ/Hol0pDXWJqFe+tYNCgrgoWOo7VexjsI=","usagesDigest":"Jr2TN3jqy4bswBcGO0MQCt0f3jEuvZwReR++qd5vTj4=","recordedInputs":["REPO_MAPPING:,bazel_tools bazel_tools","REPO_MAPPING:,root "],"generatedRepoSpecs":{"x":{"repoRuleId":"@@//:ext.bzl%made","attributes":{}}}}}}"#,
        facts: r#"{}"#,
        warnings: &[],
        error: None,
    },
];
