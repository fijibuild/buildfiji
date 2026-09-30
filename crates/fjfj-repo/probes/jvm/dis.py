import zipfile, struct, sys
from cp import pool
OPS={0x01:"aconst_null",0x02:"iconst_m1",0x03:"iconst_0",0x04:"iconst_1",0x05:"iconst_2",0x06:"iconst_3",0x07:"iconst_4",0x08:"iconst_5",0x2a:"aload_0",0x2b:"aload_1",0x2c:"aload_2",0x2d:"aload_3",0x1a:"iload_0",0x1b:"iload_1",0x4b:"astore_0",0x4c:"astore_1",0x4d:"astore_2",0x4e:"astore_3",0xb0:"areturn",0xb1:"return",0xac:"ireturn",0x59:"dup",0x57:"pop",0xbb:"new",0xb4:"getfield",0xb5:"putfield",0xb2:"getstatic",0xb6:"invokevirtual",0xb7:"invokespecial",0xb8:"invokestatic",0xb9:"invokeinterface",0xba:"invokedynamic",0xc0:"checkcast",0xc1:"instanceof",0x12:"ldc",0x13:"ldc_w",0x99:"ifeq",0x9a:"ifne",0xa7:"goto",0xc6:"ifnull",0xc7:"ifnonnull",0x10:"bipush",0x19:"aload",0x3a:"astore",0x15:"iload",0x36:"istore",0x84:"iinc",0x32:"aaload",0x53:"aastore",0xbd:"anewarray",0xbe:"arraylength",0x5f:"swap",0x1c:"iload_2",0x1d:"iload_3",0x3c:"istore_1",0x3d:"istore_2",0x3e:"istore_3",0x9f:"if_icmpeq",0xa0:"if_icmpne",0xa2:"if_icmpge",0xa4:"if_icmple",0xa5:"if_acmpeq",0xa6:"if_acmpne",0x9b:"iflt",0x9c:"ifge",0x9d:"ifgt",0x9e:"ifle",0xa1:"if_icmplt",0xa3:"if_icmpgt",0x11:"sipush",0x60:"iadd",0x64:"isub",0xbf:"athrow",0xc2:"monitorenter",0xc3:"monitorexit"}
def dump(jar,cname,want=None):
    z=zipfile.ZipFile(jar); d=z.read(cname); cp=pool(d)
    def u(i): return cp[i][1]
    def cls(i): return u(cp[i][1])
    def nt(i): a,b=cp[i][1]; return u(a)+":"+u(b)
    def ref(i):
        kind,(c,n)=cp[i]; return cls(c)+"."+nt(n)
    # skip to methods
    i=10
    n=struct.unpack(">H",d[8:10])[0]
    k=1
    while k<n:
        t=d[i]
        if t==1: i+=3+struct.unpack(">H",d[i+1:i+3])[0]
        elif t in (3,4,9,10,11,12,17,18): i+=5
        elif t in (5,6): i+=9; k+=1
        elif t in (7,8,16,19,20): i+=3
        elif t==15: i+=4
        k+=1
    i+=6
    ic=struct.unpack(">H",d[i:i+2])[0]; i+=2+2*ic
    def attrs(i):
        ac=struct.unpack(">H",d[i:i+2])[0]; i+=2; res=[]
        for _ in range(ac):
            nm=u(struct.unpack(">H",d[i:i+2])[0]); ln=struct.unpack(">I",d[i+2:i+6])[0]; res.append((nm,d[i+6:i+6+ln])); i+=6+ln
        return res,i
    fc=struct.unpack(">H",d[i:i+2])[0]; i+=2
    for _ in range(fc):
        _,i=attrs(i+6)
    mc=struct.unpack(">H",d[i:i+2])[0]; i+=2
    for _ in range(mc):
        name=u(struct.unpack(">H",d[i+2:i+4])[0]); desc=u(struct.unpack(">H",d[i+4:i+6])[0])
        at,i=attrs(i+6)
        if want and name not in want: continue
        print("##",name,desc)
        for nm,data in at:
            if nm!="Code": continue
            ln=struct.unpack(">I",data[4:8])[0]; code=data[8:8+ln]; p=0
            while p<ln:
                op=code[p]; s=OPS.get(op,"op_%02x"%op); arg=""
                if op in (0xb2,0xb4,0xb5,0xb6,0xb7,0xb8,0xb9,0xba):
                    idx=struct.unpack(">H",code[p+1:p+3])[0]
                    try: arg=ref(idx) if op!=0xba else "indy"
                    except Exception: arg="?"
                    p+=5 if op in (0xb9,0xba) else 3
                elif op in (0xbb,0xc0,0xc1,0xbd): arg=cls(struct.unpack(">H",code[p+1:p+3])[0]); p+=3
                elif op==0x12:
                    e=cp[code[p+1]]; arg=repr(u(e[1])) if e[0]=="string" else str(e); p+=2
                elif op==0x13:
                    e=cp[struct.unpack(">H",code[p+1:p+3])[0]]; arg=repr(u(e[1])) if e[0]=="string" else str(e); p+=3
                elif op in (0x99,0x9a,0xa7,0xc6,0xc7,0x9f,0xa0,0xa5,0xa6,0x9b,0x9c,0x9d,0x9e,0xa1,0xa2,0xa3,0xa4): arg=str(struct.unpack(">h",code[p+1:p+3])[0]); p+=3
                elif op in (0x10,0x19,0x3a,0x15,0x36): arg=str(code[p+1]); p+=2
                elif op==0x11: p+=3
                elif op==0x84: p+=3
                else: p+=1
                print("   ",s,arg)
if __name__=="__main__":
    dump(sys.argv[1],sys.argv[2],sys.argv[3:] or None)
