import json,sys,re
res=json.load(open(sys.argv[1]))
lo=int(sys.argv[2]) if len(sys.argv)>2 else 0; hi=int(sys.argv[3]) if len(sys.argv)>3 else len(res)
def norm(s):
    s=re.sub(r"/tmp/claude-1000/[^ \"']*?/p/ob_w\d+/external/\+r\+x\d+_\d+",'<repo>',s)
    s=re.sub(r"/tmp/claude-1000/[^ \"']*?/p/ob_w\d+/external",'<ext>',s)
    s=re.sub(r"/tmp/claude-1000/[^ \"']*?/p/w_repo_w\d+",'<ws>',s)
    s=re.sub(r"x\d+_\d+",'xN',s)
    return s
for i,r in enumerate(res[lo:hi],lo):
    body=r["case"]["bzl"].split("def _impl(ctx):\n")[1].split("\nr = repository_rule")[0].replace("\n    ","; ").strip()
    print(f"[{i}] rc={r['rc']} :: {body[:200]}")
    for p in r["prints"]: print("   P:",norm(p)[:200])
    for e in r["errs"]: print("   E:",norm(e)[:230])
    t={k:(v[:40] if isinstance(v,str) else v) for k,v in r["tree"].items() if k!="REPO.bazel"}
    if t: print("   T:",{norm(k):norm(v) for k,v in t.items()})
