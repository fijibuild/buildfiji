def _r(rctx):
    rctx.file("BUILD.bazel", "")

rr = repository_rule(implementation = _r)
