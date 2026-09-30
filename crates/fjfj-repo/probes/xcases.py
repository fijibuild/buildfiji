import json
REPO = '''def _repo_impl(ctx):
    ctx.file("BUILD.bazel", "filegroup(name='all')")
    ctx.file("v.txt", ctx.attr.value)
repo = repository_rule(_repo_impl, attrs={"value": attr.string()})
'''
def ext(impl_body, tag_classes='{"tag": tag_class(attrs={"name": attr.string(), "value": attr.string(default="dflt")})}', extra="", pre=REPO):
    body = "".join("    " + l + "\n" for l in impl_body.split("\n"))
    return pre + extra + "def _impl(mctx):\n" + body + f"ext = module_extension(_impl, tag_classes={tag_classes})\n"
def case(ext_bzl, module='module(name="probe", version="1.2")\next = use_extension("//:ext.bzl", "ext")\next.tag(name="a", value="va")\nuse_repo(ext, "a")\n', files=None, fetch=None, flags=None):
    f = {"MODULE.bazel": module, "ext.bzl": ext_bzl}
    f.update(files or {})
    c = {"files": f}
    if fetch: c["fetch"] = fetch
    if flags: c["flags"] = flags
    return c
