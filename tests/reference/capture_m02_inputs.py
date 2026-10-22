import subprocess,csv,hashlib,json,re,pathlib,ast
root=pathlib.Path.cwd();out=root/'plan/acceptance/M02-evidence';source=pathlib.Path('/Volumes/Data/Users/paul/development/src/github/skia')
commands=[['sw_vers'],['uname','-a'],['rustc','-Vv'],['cargo','-V'],['xcrun','clang++','--version'],['xcrun','--show-sdk-version'],['xcrun','--show-sdk-path'],['git','rev-parse','HEAD'],['git','-C',str(source),'rev-parse','HEAD'],['git','-C',str(source),'status','--porcelain'],['date','-u','+%Y-%m-%dT%H:%M:%SZ']]
with (out/'environment.log').open('w') as f:
 for cmd in commands:
  r=subprocess.run(cmd,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);f.write('COMMAND: '+repr(cmd)+'\n'+r.stdout+'EXIT: '+str(r.returncode)+'\n')
text=(root/'tests/reference/run_m02.py').read_text();tree=ast.parse(text)
production=next(ast.literal_eval(node.value.func.value) for node in tree.body if isinstance(node,ast.Assign) and any(isinstance(t,ast.Name)and t.id=='production'for t in node.targets)).split()
tests=list(dict.fromkeys(r['source'] for r in csv.DictReader(open('plan/inventory/test-registrations.csv'))if r['milestone']=='2'))
paths={source/r['path']for r in csv.DictReader(open('plan/inventory/source-files.csv'))if r['milestone']=='2'}
for profile,flag in [('debug','-DSK_DEBUG'),('release','-DNDEBUG')]:
 cmd=['xcrun','clang++','-std=c++20','-MM','-DSK_DISABLE_TRACING',flag,'-I'+str(source),'tests/reference/m02.cpp','tests/reference/m02_contracts.cpp']+[str(source/s)for s in tests+production]
 r=subprocess.run(cmd,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
 (out/('reference-dependencies-'+profile+'.log')).write_text('COMMAND: '+repr(cmd)+'\n'+r.stdout+'EXIT: '+str(r.returncode)+'\n')
 if r.returncode:raise RuntimeError('dependency scan failed')
 for token in r.stdout.replace('\\\n',' ').split():
  p=pathlib.Path(token)
  if p.is_absolute() and p.is_relative_to(source) and p.is_file():paths.add(p)
paths.add(source/'LICENSE')
with (out/'reference.sha256').open('w') as f:
 for p in sorted(paths):f.write(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+str(p.relative_to(source))+'\n')
handoff=(root/'plan/handoffs/M02.md').read_text();verified=[]
for name,sha in re.findall(r'\| `([^`]+)` \| `([a-f0-9]{64})` \|',handoff):
 p=root/name
 if p.is_file():
  actual=hashlib.sha256(p.read_bytes()).hexdigest();verified.append({'path':name,'expected':sha,'actual':actual,'matches':actual==sha})
(out/'handoff-hashes.json').write_text(json.dumps(verified,indent=2)+'\n')
print('Reference files',len(paths),'handoff hashes',len(verified),'mismatches',[v['path']for v in verified if not v['matches']])
