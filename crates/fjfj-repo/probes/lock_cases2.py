import json
def ext(body, extra_args="", attrs=""):
    return '''def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

made = repository_rule(implementation = _r, attrs = {"v": attr.string(), "n": attr.int(), "b": attr.bool(), "l": attr.string_list(), "d": attr.string_dict(), "lab": attr.label(), "labs": attr.label_list(), "ld": attr.label_keyed_string_dict(), "sld": attr.string_list_dict(), "il": attr.int_list(), "o": attr.output()})

def _impl(mctx):
%s

collect = module_extension(implementation = _impl%s)
''' % (body, extra_args)
def case(name, body, extra_args="", extra_files=None, module_extra="", fetch=None, flags=None):
    files={"MODULE.bazel":'module(name = "root")\ne = use_extension("//:ext.bzl", "collect")\nuse_repo(e, "x")\n'+module_extra,"ext.bzl":ext(body,extra_args),"data.txt":"hello\n","sub/BUILD.bazel":""}
    if extra_files: files.update(extra_files)
    c=dict(name=name,registry={},root=files,fetch=fetch or ["+collect+x"],lock=True)
    if flags: c["flags"]=flags
    return c
cases=[
 case("attr_types",'    made(name = "x", v = "s", n = 3, b = True, l = ["a", "b"], d = {"k": "v"}, lab = "//:data.txt", labs = ["//:data.txt", "@bazel_tools//:x"], ld = {"//:data.txt": "val"}, sld = {"a": ["1", "2"]}, il = [1, 2])'),
 case("label_objects",'    made(name = "x", lab = Label("//:data.txt"), labs = [Label("//sub:y")])'),
 case("none_and_empty",'    made(name = "x", v = None, l = [], d = {})'),
 case("read_file",'    mctx.read(mctx.path(Label("//:data.txt")))\n    made(name = "x")'),
 case("read_path_str",'    mctx.read("/etc/hostname")\n    made(name = "x")'),
 case("getenv",'    mctx.getenv("FJFJ_PROBE_A")\n    made(name = "x")'),
 case("getenv_default",'    mctx.getenv("FJFJ_PROBE_B", "dflt")\n    made(name = "x")'),
 case("environ",'    mctx.os.environ.get("FJFJ_PROBE_C")\n    made(name = "x")'),
 case("file_exists",'    mctx.path(Label("//:data.txt")).exists\n    made(name = "x")'),
 case("readdir",'    mctx.path(Label("//sub:BUILD.bazel")).dirname.readdir()\n    made(name = "x")'),
 case("watch",'    mctx.watch(mctx.path(Label("//:data.txt")))\n    made(name = "x")'),
 case("watch_tree",'    mctx.watch_tree(mctx.path(Label("//sub:BUILD.bazel")).dirname)\n    made(name = "x")'),
 case("execute",'    mctx.execute(["true"])\n    made(name = "x")'),
 case("os_dependent",'    made(name = "x")',", os_dependent = True"),
 case("arch_dependent",'    made(name = "x")',", arch_dependent = True"),
 case("both_dependent",'    made(name = "x")',", os_dependent = True, arch_dependent = True"),
 case("reproducible",'    made(name = "x")\n    return mctx.extension_metadata(reproducible = True)'),
 case("which",'    mctx.which("sh")\n    made(name = "x")'),
 case("download_none",'    made(name = "x")\n    print(mctx.os.name)'),
]
json.dump(cases,open("lock_cases2.json","w"))
