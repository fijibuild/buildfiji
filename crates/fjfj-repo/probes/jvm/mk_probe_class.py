import struct
class CP:
    def __init__(s): s.items=[]; s.map={}
    def add(s,key,raw):
        if key in s.map: return s.map[key]
        s.items.append(raw); s.map[key]=len(s.items); return len(s.items)
    def utf8(s,t): b=t.encode(); return s.add(("u",t),b"\x01"+struct.pack(">H",len(b))+b)
    def cls(s,n): i=s.utf8(n); return s.add(("c",n),b"\x07"+struct.pack(">H",i))
    def nt(s,n,d): a=s.utf8(n); b=s.utf8(d); return s.add(("nt",n,d),b"\x0c"+struct.pack(">HH",a,b))
    def field(s,c,n,d): return s.add(("f",c,n,d),b"\x09"+struct.pack(">HH",s.cls(c),s.nt(n,d)))
    def meth(s,c,n,d): return s.add(("m",c,n,d),b"\x0a"+struct.pack(">HH",s.cls(c),s.nt(n,d)))
cp=CP()
this=cp.cls("com/google/devtools/build/lib/bazel/bzlmod/Probe"); sup=cp.cls("java/lang/Object")
G="com/google/devtools/build/lib/bazel/bzlmod/GsonTypeAdapterUtil"
S="com/google/devtools/build/lib/bazel/bzlmod/SingleExtensionUsagesValue"
gson_f=cp.field(G,"SINGLE_EXTENSION_USAGES_VALUE_GSON","Lcom/google/gson/Gson;")
from_json=cp.meth("com/google/gson/Gson","fromJson","(Ljava/lang/String;Ljava/lang/Class;)Ljava/lang/Object;")
sc=cp.cls(S)
trim=cp.meth(S,"trimForEvaluation","()L%s;"%S)
to_json=cp.meth("com/google/gson/Gson","toJson","(Ljava/lang/Object;)Ljava/lang/String;")
out=cp.field("java/lang/System","out","Ljava/io/PrintStream;")
println=cp.meth("java/io/PrintStream","println","(Ljava/lang/String;)V")
hash_=cp.meth(S,"hashForEvaluation","(Lcom/google/gson/Gson;L%s;)[B"%S)
b64=cp.meth("java/util/Base64","getEncoder","()Ljava/util/Base64$Encoder;")
enc=cp.meth("java/util/Base64$Encoder","encodeToString","([B)Ljava/lang/String;")
code_name=cp.utf8("Code"); main=cp.utf8("main"); md=cp.utf8("([Ljava/lang/String;)V")
u2=lambda v: struct.pack(">H",v)
code=bytearray()
def op(*b): code.extend(b)
def idx(o,i): code.extend([o]+list(u2(i)))
idx(0xb2,gson_f); op(0x2a,0x03,0x32)       # getstatic gson; aload_0; iconst_0; aaload
code.extend([0x13]+list(u2(sc)))           # ldc_w class
idx(0xb6,from_json); idx(0xc0,sc); op(0x4c) # checkcast; astore_1
idx(0xb2,out); idx(0xb2,gson_f); op(0x2b); idx(0xb6,trim); idx(0xb6,to_json); idx(0xb6,println)
idx(0xb2,out); idx(0xb8,b64); idx(0xb2,gson_f); op(0x2b); idx(0xb8,hash_); idx(0xb6,enc); idx(0xb6,println)
op(0xb1)
attr=u2(4)+u2(2)+struct.pack(">I",len(code))+bytes(code)+u2(0)+u2(0)
method=u2(0x0009)+u2(main)+u2(md)+u2(1)+u2(code_name)+struct.pack(">I",len(attr))+attr
cls=b"\xca\xfe\xba\xbe"+u2(0)+u2(49)+u2(len(cp.items)+1)+b"".join(cp.items)+u2(0x0021)+u2(this)+u2(sup)+u2(0)+u2(0)+u2(1)+method+u2(0)
import os; os.makedirs("com/google/devtools/build/lib/bazel/bzlmod",exist_ok=True); open("com/google/devtools/build/lib/bazel/bzlmod/Probe.class","wb").write(cls)
print("ok",len(cls))
