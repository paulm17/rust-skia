import hashlib,json,pathlib,subprocess,sys
root=pathlib.Path.cwd(); out=root/'plan/fix-evidence/M01'; ref=root.parent/'skia'
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def manifest():
 files={str(p.relative_to(root)):digest(p) for base in ['tests','plan/acceptance','plan/handoffs','plan/inventory'] for p in (root/base).rglob('*') if p.is_file()}
 for p in (root/'src').rglob('*.rs'):
  text=p.read_bytes(); marker=b'#[cfg(test)]'
  if marker in text: files[str(p.relative_to(root))+':test-section']=hashlib.sha256(text[text.index(marker):]).hexdigest()
 files.update({'reference:'+str(p.relative_to(ref)):digest(p) for p in ref.rglob('*') if p.is_file() and '.git' not in p.parts and p.suffix in ['.h','.cpp']})
 return files
if sys.argv[1]=='baseline':
 (out/'protected.json').write_text(json.dumps(manifest(),indent=2,sort_keys=True))
 with (out/'environment.log').open('w') as f:
  for cmd in [['git','rev-parse','HEAD'],['git','status','--short'],['git','-C',str(ref),'rev-parse','HEAD'],['git','-C',str(ref),'status','--short'],['uname','-a'],['sw_vers'],['rustc','-Vv'],['cargo','-V'],['xcrun','clang++','--version'],['xcrun','--show-sdk-version']]:
   r=subprocess.run(cmd,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT);f.write(repr(cmd)+'\n'+r.stdout+'\n')
else:
 old=json.loads((out/'protected.json').read_text()); now=manifest(); differences=[p for p in old if now.get(p)!=old[p]]
 print('Protected entries:',len(old),'changed:',len(differences)); print('\n'.join(differences));sys.exit(bool(differences))
