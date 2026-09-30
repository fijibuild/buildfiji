import json,sys,re
res=json.load(open(sys.argv[1]))
base=set(res[0]["cache"])
lo=int(sys.argv[2]) if len(sys.argv)>2 else 0; hi=int(sys.argv[3]) if len(sys.argv)>3 else len(res)
for i,r in enumerate(res[lo:hi],lo):
    c=r["case"]
    body=c["bzl"].split("def _impl(ctx):\n")[1].split("\n    ctx.file('z.marker'")[0].replace("\n    ","; ").strip()
    print(f"[{i}] rc={r['rc']} :: {body[:190]}" + (f"  call={c['call']}" if c.get("call") else "") + (" TWICE" if c.get("twice") else ""))
    for p in r["prints"]: print("   P:",p[:200])
    for l in r["log"]:
        if l.startswith(("Error in", "WARNING")): print("   L:",l[:230])
    t={k:(v[:40] if isinstance(v,str) else v) for k,v in r["tree"].items() if k not in ("REPO.bazel","z.marker")}
    if t: print("   T:",t)
    print("   Q:",r["requests"], "C:",[x.split("/")[2][:8] for x in r["cache"] if x not in base])
