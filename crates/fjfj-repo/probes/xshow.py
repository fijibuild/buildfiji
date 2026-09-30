import json,sys,re
res=json.load(open(sys.argv[1]))
lo=int(sys.argv[2]) if len(sys.argv)>2 else 0; hi=int(sys.argv[3]) if len(sys.argv)>3 else len(res)
for i,r in enumerate(res[lo:hi],lo):
    f=r["case"]["files"]
    mod=f["MODULE.bazel"].replace('module(name="probe", version="1.2")\n','').replace('module(name="probe")\n','').replace("\n"," ; ").strip()
    impl=f["ext.bzl"].split("def _impl(mctx):\n")[1].split("\next = module_extension")[0].replace("\n    ","; ").strip() if "def _impl(mctx):" in f["ext.bzl"] else "?"
    print(f"[{i}] rc={r['rc']} impl: {impl[:150]}\n     MODULE: {mod[:200]}")
    for p in r["prints"]: print("   P:",p[:170])
    for e in r.get("log", r["errs"]): print("   L:",e[:300])
    if r["mapping"]: print("   M:",r["mapping"][:140])
    for n,t in r["repos"].items():
        print("   R:",n,{k:v[:30] for k,v in t.items() if k not in ("REPO.bazel",)})
