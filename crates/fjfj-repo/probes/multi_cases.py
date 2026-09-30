import json
EXT='''def _r(rctx):
    rctx.file("BUILD.bazel", "# " + rctx.attr.info + "\\nfilegroup(name = 'f')")

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
'''
def use(mod, tag, imports, dev=False):
    return f'''e = use_extension("@lib//:ext.bzl", "collect"{", dev_dependency = True" if dev else ""})
e.add(name = "{tag}")
use_repo(e, {", ".join('"%s"' % i for i in imports)})
'''
LIB={"1.0":{"MODULE.bazel":'module(name = "lib", version = "1.0")\n',"ext.bzl":EXT}}
def dep(n): return f'bazel_dep(name = "{n}", version = "1.0")\n'
def m(name, body): return {"1.0":{"MODULE.bazel":f'module(name = "{name}", version = "1.0")\n'+dep("lib")+body}}
ROOT='module(name = "root", version = "0")\n'

ROOT_DEPS=ROOT
cases=[
 {"name":"every_module_in_order","registry":{"lib":LIB,"a":m("a",use("a","from_a",["from_a"])),
   "b":m("b",use("b","from_b",["from_b"])+'d = use_extension("@lib//:ext.bzl", "collect", dev_dependency = True)\nd.add(name = "from_b_dev")\n')},
  "root":{"MODULE.bazel":ROOT+dep("b")+dep("a")+dep("lib")+use("r","from_root",["from_root","from_a","from_b"])+'d = use_extension("@lib//:ext.bzl", "collect", dev_dependency = True)\nd.add(name = "dev_root")\nuse_repo(d, "dev_root")\n'},
  "fetch":["lib++collect+from_a","lib++collect+from_b","lib++collect+from_root","lib++collect+dev_root"]},
 {"name":"root_does_not_use_it","registry":{"lib":LIB,"a":m("a",use("a","from_a",["from_a"]))},
  "root":{"MODULE.bazel":ROOT+dep("a")},
  "fetch":["lib++collect+from_a"]},
 {"name":"root_only_dev","registry":{"lib":LIB,"a":m("a",use("a","from_a",["from_a"]))},
  "root":{"MODULE.bazel":ROOT+dep("a")+dep("lib")+use("r","dev_root",["dev_root","from_a"],True)},
  "fetch":["lib++collect+from_a","lib++collect+dev_root"]},
 {"name":"import_of_a_repo_nobody_makes","registry":{"lib":LIB,"a":m("a",use("a","from_a",["from_a","nope"]))},
  "root":{"MODULE.bazel":ROOT+dep("a")},
  "fetch":["lib++collect+from_a"]},
 {"name":"root_imports_a_repo_nobody_makes","registry":{"lib":LIB,"a":m("a",use("a","from_a",["from_a"]))},
  "root":{"MODULE.bazel":ROOT+dep("a")+dep("lib")+use("r","from_root",["from_root","zzz"])},
  "fetch":["lib++collect+from_a"]},
 {"name":"extension_loads_from_a_module_it_depends_on","registry":{
    "helper":{"1.0":{"MODULE.bazel":'module(name = "helper", version = "1.0")\n',"h.bzl":'TAG = "from_helper"\n'}},
    "lib":{"1.0":{"MODULE.bazel":'module(name = "lib", version = "1.0")\nbazel_dep(name = "helper", version = "1.0")\n',"ext.bzl":'load("@helper//:h.bzl", "TAG")\n'+EXT.replace('print(info)','print(info + " " + TAG)')}}},
  "root":{"MODULE.bazel":ROOT+dep("lib")+use("r","x",["x"])},
  "fetch":["lib++collect+x"]},
 {"name":"extension_loads_from_a_repository_another_extension_made","registry":{
    "lib":{"1.0":{"MODULE.bazel":'module(name = "lib", version = "1.0")\ng = use_extension("//:gen.bzl", "genx")\nuse_repo(g, "gen")\n',
       "gen.bzl":'def _r(rctx):\n    rctx.file("BUILD.bazel", "")\n    rctx.file("defs.bzl", "VALUE = \'generated\'\\n")\n\nmaker = repository_rule(implementation = _r)\n\ndef _impl(mctx):\n    print("genx runs")\n    maker(name = "gen")\n\ngenx = module_extension(implementation = _impl)\n',
       "ext.bzl":'load("@gen//:defs.bzl", "VALUE")\n'+EXT.replace('print(info)','print(info + " " + VALUE)')}}},
  "root":{"MODULE.bazel":ROOT+dep("lib")+use("r","x",["x"])},
  "fetch":["lib++collect+x"]},
]
cases.append({"name":"a_dependency_calls_a_repository_rule_of_another_module","registry":{
    "lib":{"1.0":{"MODULE.bazel":'module(name = "lib", version = "1.0")\n',"rules.bzl":'def _r(rctx):\n    rctx.file("BUILD.bazel", "# " + rctx.attr.info + " " + rctx.name + " " + rctx.original_name + "\\nfilegroup(name = \'f\')")\n\nmade = repository_rule(implementation = _r, attrs = {"info": attr.string()})\n'}},
    "a":m("a",'made = use_repo_rule("@lib//:rules.bzl", "made")\nmade(name = "mine", info = "from a")\n')},
  "root":{"MODULE.bazel":ROOT+dep("a")},
  "fetch":["a++made+mine"]})
ALT={"ovr/alt/MODULE.bazel":'module(name = "lib", version = "9.9")\n',"ovr/alt/BUILD.bazel":"filegroup(name = 'f')\n# alt\n"}
NOB={"ovr/nob/BUILD.bazel":"filegroup(name = 'f')\n# no boundary file\n"}
OEXT=EXT.replace("print(info)","print('ext runs')")
cases += [
 {"name":"override_repository_replaces_a_module_repo","registry":{"lib":LIB},"root":{"MODULE.bazel":ROOT+dep("lib"),**ALT},"overrides":["lib=ovr/alt"],"fetch":["lib+"]},
 {"name":"override_repository_takes_an_absolute_path","registry":{"lib":LIB},"root":{"MODULE.bazel":ROOT+dep("lib"),**ALT},"overrides":["lib=@WS@/ovr/alt"],"fetch":["lib+"]},
 {"name":"override_repository_names_the_repo_the_main_repository_sees","registry":{"lib":LIB},"root":{"MODULE.bazel":ROOT+'bazel_dep(name = "lib", version = "1.0", repo_name = "foo")\n',**ALT},"overrides":["foo=ovr/alt"],"fetch":["lib+"]},
 {"name":"override_repository_of_a_name_the_main_repository_does_not_see","registry":{"lib":LIB},"root":{"MODULE.bazel":ROOT+'bazel_dep(name = "lib", version = "1.0", repo_name = "foo")\n',**ALT},"overrides":["lib=ovr/alt"],"fetch":["lib+"]},
 {"name":"override_repository_to_a_directory_that_is_not_there","registry":{"lib":LIB},"root":{"MODULE.bazel":ROOT+dep("lib"),**ALT},"overrides":["lib=ovr/nope"],"fetch":["lib+"]},
 {"name":"override_repository_to_a_file","registry":{"lib":LIB},"root":{"MODULE.bazel":ROOT+dep("lib"),**ALT},"overrides":["lib=ovr/alt/BUILD.bazel"],"fetch":["lib+"]},
 {"name":"override_repository_to_a_directory_with_no_boundary_file","registry":{"lib":LIB},"root":{"MODULE.bazel":ROOT+dep("lib"),**NOB},"overrides":["lib=ovr/nob"],"fetch":["lib+"]},
 {"name":"override_repository_replaces_what_an_extension_generates","registry":{"lib":{"1.0":{"MODULE.bazel":'module(name = "lib", version = "1.0")\n',"ext.bzl":OEXT}}},"root":{"MODULE.bazel":ROOT+dep("lib")+use("r","gen",["gen"]),**ALT},"overrides":["gen=ovr/alt"],"fetch":["lib++collect+gen"]},
 {"name":"override_repository_replaces_a_use_repo_rule_repo","registry":{},"root":{"MODULE.bazel":ROOT+'made = use_repo_rule("//:r.bzl", "made")\nmade(name = "x")\n',"r.bzl":'def _r(rctx):\n    rctx.file("BUILD.bazel", "")\n\nmade = repository_rule(implementation = _r)\n',**ALT},"overrides":["x=ovr/alt"],"fetch":["+made+x"]},
]
json.dump(cases,open("multi_cases.json","w"))
