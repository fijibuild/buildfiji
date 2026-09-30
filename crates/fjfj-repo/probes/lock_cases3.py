import json
exec(open("lock_cases2.py").read().split("cases=[")[0])
cases=[
 case("order",'    mctx.read(mctx.path(Label("//:b.txt")))\n    mctx.getenv("ZZ_P")\n    mctx.getenv("AA_P")\n    mctx.read(mctx.path(Label("//:a.txt")))\n    mctx.path(Label("//sub:BUILD.bazel")).dirname.readdir()\n    mctx.read(mctx.path(Label("//:b.txt")))\n    made(name = "x")',extra_files={"a.txt":"A\n","b.txt":"B\n"}),
 case("abs_path_in_ws",'    mctx.read(mctx.path(Label("//:a.txt")).realpath)\n    made(name = "x")',extra_files={"a.txt":"A\n"}),
 case("subdir_file",'    mctx.read(mctx.path(Label("//sub:f.txt")))\n    made(name = "x")',extra_files={"sub/f.txt":"F\n"}),
 case("missing_file_exists",'    mctx.path(Label("//:none.txt")).exists\n    made(name = "x")'),
 case("getenv_unset_nodefault",'    mctx.getenv("FJFJ_UNSET_Q")\n    made(name = "x")'),
 case("getenv_value_with_space",'    mctx.getenv("FJFJ_SP")\n    made(name = "x")'),
 case("file_in_other_repo",'    mctx.read(mctx.path(Label("@bazel_tools//tools/build_defs/repo:utils.bzl")))\n    made(name = "x")'),
]
json.dump(cases,open("lock_cases3.json","w"))
