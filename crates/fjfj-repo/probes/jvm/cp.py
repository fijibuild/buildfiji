import zipfile, struct, sys
def pool(data):
    i=10; n=struct.unpack(">H",data[8:10])[0]; cp=[None]*n; k=1
    while k<n:
        t=data[i]
        if t==1:
            l=struct.unpack(">H",data[i+1:i+3])[0]; cp[k]=("utf8",data[i+3:i+3+l].decode("utf8","replace")); i+=3+l
        elif t in (3,4): cp[k]=("int",data[i+1:i+5]); i+=5
        elif t in (5,6): cp[k]=("long",data[i+1:i+9]); i+=9; k+=1
        elif t in (7,8,16,19,20): cp[k]=({7:"class",8:"string",16:"mtype",19:"module",20:"package"}[t],struct.unpack(">H",data[i+1:i+3])[0]); i+=3
        elif t in (9,10,11,12,17,18): cp[k]=({9:"field",10:"meth",11:"imeth",12:"nt",17:"dyn",18:"indy"}[t],struct.unpack(">HH",data[i+1:i+5])); i+=5
        elif t==15: cp[k]=("mh",data[i+1:i+4]); i+=4
        k+=1
    return cp
def show(jar,name):
    z=zipfile.ZipFile(jar); data=z.read(name); cp=pool(data)
    def u(i): return cp[i][1]
    out=[]
    for k,e in enumerate(cp):
        if not e: continue
        if e[0]=="utf8": out.append(e[1])
    return out
if __name__=="__main__":
    jar=sys.argv[1]
    for name in sys.argv[2:]:
        print("=====",name)
        for s in show(jar,name): print("  ",s[:140])
