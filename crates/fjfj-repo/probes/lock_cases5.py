import json
exec(open("lock_cases2.py").read().split("\ncases=[")[0])
def mcase(name, args, imports):
    c=case(name,'    made(name = "x")\n    made(name = "y")\n    return mctx.extension_metadata(%s)' % args, fetch=["+collect+x"])
    c["root"]["MODULE.bazel"]='module(name = "root")\ne = use_extension("//:ext.bzl", "collect")\nuse_repo(e, %s)\n' % imports
    return c
cases=[
 mcase("deps_match",'root_module_direct_deps = ["x", "y"], root_module_direct_dev_deps = []','"x", "y"'),
 mcase("deps_all",'root_module_direct_deps = "all", root_module_direct_dev_deps = []','"x"'),
 mcase("deps_none",'root_module_direct_deps = None, root_module_direct_dev_deps = None','"x"'),
 mcase("deps_reproducible",'reproducible = True','"x"'),
 mcase("deps_repro_false",'reproducible = False','"x"'),
 mcase("deps_some_lists",'root_module_direct_deps = ["y", "x"], root_module_direct_dev_deps = []','"x"'),
 mcase("facts_only",'facts = {"a": 1}','"x"'),
 mcase("facts_empty",'facts = {}','"x"'),
 mcase("facts_nested",'facts = {"z": 1, "a": {"q": [1, 2, None], "b": "s"}, "m": 2.5}','"x"'),
 mcase("facts_not_dict",'facts = 3','"x"'),
]
# dev usage variants
def dcase(name,args,module):
    c=case(name,'    made(name = "x")\n    made(name = "y")\n    return mctx.extension_metadata(%s)' % args, fetch=["+collect+x"])
    c["root"]["MODULE.bazel"]=module
    return c
cases+=[
 dcase("dev_dev_deps",'root_module_direct_deps = ["x"], root_module_direct_dev_deps = ["y"]','module(name = "root")\ne = use_extension("//:ext.bzl", "collect")\nuse_repo(e, "x")\nd = use_extension("//:ext.bzl", "collect", dev_dependency = True)\nuse_repo(d, "y")\n'),
 dcase("dev_swapped",'root_module_direct_deps = ["y"], root_module_direct_dev_deps = ["x"]','module(name = "root")\ne = use_extension("//:ext.bzl", "collect")\nuse_repo(e, "x")\nd = use_extension("//:ext.bzl", "collect", dev_dependency = True)\nuse_repo(d, "y")\n'),
 dcase("dev_all_dev",'root_module_direct_deps = [], root_module_direct_dev_deps = "all"','module(name = "root")\ne = use_extension("//:ext.bzl", "collect")\nuse_repo(e, "x")\nd = use_extension("//:ext.bzl", "collect", dev_dependency = True)\nuse_repo(d, "y")\n'),
 dcase("many_missing",'root_module_direct_deps = ["y", "x", "b", "a"], root_module_direct_dev_deps = []','module(name = "root")\ne = use_extension("//:ext.bzl", "collect")\nuse_repo(e, "x")\n'),
 dcase("unordered_extra",'root_module_direct_deps = ["x"], root_module_direct_dev_deps = []','module(name = "root")\ne = use_extension("//:ext.bzl", "collect")\nuse_repo(e, "y", "x")\n'),
 dcase("dup_entry",'root_module_direct_deps = ["x", "x"], root_module_direct_dev_deps = []','module(name = "root")\ne = use_extension("//:ext.bzl", "collect")\nuse_repo(e, "x")\n'),
 dcase("non_string_entry",'root_module_direct_deps = ["x", 3], root_module_direct_dev_deps = []','module(name = "root")\ne = use_extension("//:ext.bzl", "collect")\nuse_repo(e, "x")\n'),
]
json.dump(cases,open("lock_cases5.json","w"))
