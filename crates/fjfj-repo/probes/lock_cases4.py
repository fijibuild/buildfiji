import json
exec(open("lock_cases2.py").read().split("cases=[")[0])
def meta(args, imports='"x", "y"'):
    return case("m_"+str(abs(hash(args)))[:5], '    made(name = "x")\n    made(name = "y")\n    return mctx.extension_metadata(%s)' % args, module_extra="")
def mcase(name, args, imports):
    c=case(name,'    made(name = "x")\n    made(name = "y")\n    return mctx.extension_metadata(%s)' % args, fetch=["+collect+x"])
    c["root"]["MODULE.bazel"]='module(name = "root")\ne = use_extension("//:ext.bzl", "collect")\nuse_repo(e, %s)\n' % imports
    return c
cases=[
 mcase("deps_match",'root_module_direct_deps = ["x", "y"], root_module_direct_dev_deps = []','"x", "y"'),
 mcase("deps_missing_import",'root_module_direct_deps = ["x", "y"], root_module_direct_dev_deps = []','"x"'),
 mcase("deps_extra_import",'root_module_direct_deps = ["x"], root_module_direct_dev_deps = []','"x", "y"'),
 mcase("deps_all",'root_module_direct_deps = "all"','"x"'),
 mcase("deps_all_dev",'root_module_direct_deps = [], root_module_direct_dev_deps = "all"','"x"'),
 mcase("deps_not_generated",'root_module_direct_deps = ["x", "zzz"], root_module_direct_dev_deps = []','"x"'),
 mcase("only_direct",'root_module_direct_deps = ["x", "y"]','"x", "y"'),
 mcase("only_dev",'root_module_direct_dev_deps = ["x", "y"]','"x", "y"'),
 mcase("bad_type",'root_module_direct_deps = 3','"x"'),
 mcase("deps_dev_overlap",'root_module_direct_deps = ["x"], root_module_direct_dev_deps = ["x"]','"x"'),
 mcase("facts_set",'facts = {"a": 1}','"x"'),
]
json.dump(cases,open("lock_cases4.json","w"))
