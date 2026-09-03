import os,re,tomllib
names=set()
lock=open('Cargo.lock',encoding='utf-8').read()
for m in re.finditer(r'\[\[package\]\]\s+name = "([^"]+)"\s+version = "[^"]+"\s+source = "git\+https://github.com/servo/servo[^"]*',lock): names.add(m.group(1))
paths={}
for root,dirs,files in os.walk('vendor/servo'):
    if 'Cargo.toml' in files:
        try: t=tomllib.load(open(os.path.join(root,'Cargo.toml'),'rb'))
        except Exception: continue
        n=t.get('package',{}).get('name')
        if n in names: paths[n]=root.replace(os.sep,'/')
missing=names-set(paths)
print('lock crates',len(names),'mapped',len(paths),'missing',missing)
s=open('Cargo.toml',encoding='utf-8').read()
assert '[patch."https://github.com/servo/servo"]' not in s
s+='\n[patch."https://github.com/servo/servo"]\n'+''.join('%s = { path = "%s" }\n'%(k,paths[k]) for k in sorted(paths))
open('Cargo.toml','w',encoding='utf-8').write(s)
