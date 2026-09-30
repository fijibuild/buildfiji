import json
def case(body, attrs="", call="", files=None, pre=""):
    bzl = pre + "def _impl(ctx):\n" + "".join("    " + l + "\n" for l in body.split("\n")) + "    ctx.file('z.marker', '')\n" + f"r = repository_rule(_impl{', attrs=' + attrs if attrs else ''})\n"
    return {"bzl": bzl, "call": call, "files": files or {}}
