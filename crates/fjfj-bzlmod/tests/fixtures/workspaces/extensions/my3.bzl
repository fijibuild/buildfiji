def _r(rctx):
    rctx.file("BUILD.bazel", "filegroup(name = 'f')")

r = repository_rule(implementation = _r)

def _impl(mctx):
    r(name = "e")
    r(name = "f")

gen = module_extension(implementation = _impl)
