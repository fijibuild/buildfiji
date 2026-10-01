import json
exec(open("lock_cases2.py").read().split("\ncases=[")[0])
UTILS = 'load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")\n'
def with_ext(c, prefix, extra_files=None):
    c["root"]["ext.bzl"] = prefix + c["root"]["ext.bzl"]
    if extra_files: c["root"].update(extra_files)
    return c
cases = [
 case("mapping_label", '    Label("@bazel_tools//:x")\n    Label("@nosuch//:y")\n    Label("@root//:z")\n    made(name = "x")'),
 with_ext(case("mapping_load", '    made(name = "x")'), UTILS),
 with_ext(case("mapping_transitive", '    made(name = "x")'), 'load("//:helper.bzl", "H")\n',
          {"helper.bzl": UTILS + 'H = Label("@nosuch2//:q")\nH2 = Label("@@canon//:q")\nH3 = Label("//:own")\n'}),
 case("mapping_with_env", '    mctx.getenv("FJFJ_PROBE_A")\n    Label("@nosuch//:y")\n    mctx.read(mctx.path(Label("//:data.txt")))\n    made(name = "x")'),
 case("mapping_attr_default", '    made(name = "x")', extra_args = ', tag_classes = {"t": tag_class(attrs = {"l": attr.label_list(default = ["@nosuch3//:x"])})}'),

 case("mapping_tag_label", '    made(name = "x")', extra_args = ', tag_classes = {"t": tag_class(attrs = {"lab": attr.label(), "labs": attr.label_list()})}', module_extra = 'e.t(lab = "@bazel_tools//:x", labs = ["@root//:y", "//:z"])\n'),
]
json.dump(cases, open("lock_cases6.json", "w"))
