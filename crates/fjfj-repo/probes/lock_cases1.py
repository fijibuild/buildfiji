import json
EXT='''def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict()})

_t = tag_class(attrs = {"name": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "o": attr.string(), "lab": attr.label(), "sld": attr.string_list_dict()})

def _impl(mctx):
    for m in mctx.modules:
        for t in m.tags.t:
            made(name = t.name, v = "x")

collect = module_extension(implementation = _impl, tag_classes = {"t": _t})
'''
LIB={"1.0":{"MODULE.bazel":'module(name = "lib", version = "1.0")\n',"ext.bzl":EXT}}
def C(name, registry, root, **kw):
    return dict(name=name, registry=registry, root=root, fetch=[], lock=True, **kw)
R='module(name = "root")\nbazel_dep(name = "lib", version = "1.0")\n'
cases=[]
def local(name, module, ext_files=None, fetch=None, registry=None):
    files={"MODULE.bazel":module,"ext.bzl":EXT}
    if ext_files: files.update(ext_files)
    c=C(name,registry or {},files)
    c['fetch']=fetch or ['+collect+x']
    return c
T='e = use_extension("//:ext.bzl", "collect")\ne.t(name = "x")\nuse_repo(e, "x")\n'
cases.append(local("basic",'module(name = "root")\n'+T))
cases.append(local("repo_name",'module(name = "root", repo_name = "rr")\n'+T))
cases.append(local("colon_label",'module(name = "root")\ne = use_extension(":ext.bzl", "collect")\ne.t(name = "x")\nuse_repo(e, "x")\n'))
cases.append(local("pkg_label",'module(name = "root")\ne = use_extension("//sub:ext.bzl", "collect")\ne.t(name = "x")\nuse_repo(e, "x")\n',{"sub/ext.bzl":EXT,"sub/BUILD.bazel":""}))
cases.append(local("two_extensions",'module(name = "root")\n'+T+'f = use_extension("//:ext.bzl", "collect")\nf.t(name = "y")\nuse_repo(f, "y")\n')) # same ext used twice: merged
cases.append(local("tag_values",'''module(name = "root")
e = use_extension("//:ext.bzl", "collect")
e.t(name = "x", n = 3, b = True, l = ["a", "b"], d = {"k": "v", "k2": "v2"}, o = "s", lab = "//:ext.bzl", sld = {"a": ["1", "2"]})
use_repo(e, "x")
'''))
cases.append(local("dev_tag",'''module(name = "root")
e = use_extension("//:ext.bzl", "collect")
e.t(name = "x")
use_repo(e, "x")
d = use_extension("//:ext.bzl", "collect", dev_dependency = True)
d.t(name = "y")
'''))
cases.append(local("root_version",'module(name = "root", version = "2.5")\n'+T))
cases.append(C("two_modules",{"lib":LIB,"a":{"1.0":{"MODULE.bazel":'module(name = "a", version = "1.0")\nbazel_dep(name = "lib", version = "1.0")\ne = use_extension("@lib//:ext.bzl", "collect")\ne.t(name = "from_a")\nuse_repo(e, "from_a")\n'}}},{"MODULE.bazel":R+'bazel_dep(name = "a", version = "1.0")\ne = use_extension("@lib//:ext.bzl", "collect")\ne.t(name = "from_root")\nuse_repo(e, "from_root")\n'}))
cases.append(C("aliased_dep",{"lib":LIB},{"MODULE.bazel":'module(name = "root")\nbazel_dep(name = "lib", version = "1.0", repo_name = "ll")\ne = use_extension("@ll//:ext.bzl", "collect")\ne.t(name = "x")\nuse_repo(e, "x")\n'}))
cases.append(C("dep_module_repo_name",{"lib":LIB,"a":{"1.0":{"MODULE.bazel":'module(name = "a", version = "1.0", repo_name = "aa")\nbazel_dep(name = "lib", version = "1.0")\ne = use_extension("@lib//:ext.bzl", "collect")\ne.t(name = "from_a")\nuse_repo(e, "from_a")\n',"ext.bzl":EXT}}},{"MODULE.bazel":R+'bazel_dep(name = "a", version = "1.0")\n'}))
cases.append(C("dep_local_ext",{"a":{"1.0":{"MODULE.bazel":'module(name = "a", version = "1.0")\ne = use_extension("//:ext.bzl", "collect")\ne.t(name = "from_a")\nuse_repo(e, "from_a")\n',"ext.bzl":EXT}}},{"MODULE.bazel":'module(name = "root")\nbazel_dep(name = "a", version = "1.0")\n'}))
cases.append(local("override_repo",'''module(name = "root")
e = use_extension("//:ext.bzl", "collect")
e.t(name = "x")
use_repo(e, "x")
r = use_repo_rule("//:ext.bzl", "made")
r(name = "mine", v = "1")
override_repo(e, x = "mine")
'''))
for x in cases:
    if x["name"]=="two_extensions": x["fetch"]=["+collect+x"]
    if x["name"] in ("two_modules",): x["fetch"]=["lib++collect+from_root"]
    if x["name"] in ("aliased_dep",): x["fetch"]=["lib++collect+x"]
    if x["name"]=="dep_module_repo_name": x["fetch"]=["lib++collect+from_a"]
    if x["name"]=="dep_local_ext": x["fetch"]=["a++collect+from_a"]
    if x["name"]=="override_repo": x["fetch"]=["+made+mine"]
    if x["name"]=="dev_tag": x["fetch"]=["+collect+x","+collect+y"]
json.dump(cases,open("lock_cases1.json","w"))
