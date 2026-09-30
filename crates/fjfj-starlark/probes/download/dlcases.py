def case(body, serve=None, call="", attrs="", files=None, flags=None, twice=False, pre=""):
    bzl = pre + "def _impl(ctx):\n" + "".join("    " + l + "\n" for l in body.split("\n")) + "    ctx.file('z.marker', '')\n" + f"r = repository_rule(_impl{', attrs=' + attrs if attrs else ''})\n"
    c = {"bzl": bzl, "call": call, "files": files or {}, "serve": serve or {}}
    if flags: c["flags"] = flags
    if twice: c["twice"] = True
    return c
T = {"text": "hello"}
