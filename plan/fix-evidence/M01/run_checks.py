import subprocess,pathlib
out=pathlib.Path('plan/fix-evidence/M01')
commands=[('discovery-debug-before',['cargo','test','--locked','--','--list']),('discovery-release-before',['cargo','test','--release','--locked','--','--list']),('test-debug',['cargo','test','--locked']),('test-release',['cargo','test','--release','--locked']),('fmt',['cargo','fmt','--all','--','--check']),('clippy',['cargo','clippy','--all-targets','--locked','--','-D','warnings']),('validate-plan',['python3','plan/validate_plan.py','--source','/Volumes/Data/Users/paul/development/src/github/skia','--check-hashes']),('reference',['python3','tests/reference/run_m01.py','--source','/Volumes/Data/Users/paul/development/src/github/skia','--output',str(out/'reference')]),('discovery-debug-after',['cargo','test','--locked','--','--list']),('discovery-release-after',['cargo','test','--release','--locked','--','--list']),('integrity',['python3',str(out/'verify.py'),'check'])]
results={}
for name,cmd in commands:
 with (out/(name+'.log')).open('w') as f:
  f.write('COMMAND: '+repr(cmd)+'\n');f.flush();r=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT);f.write('\nEXIT: '+str(r.returncode)+'\n')
 results[name]=r.returncode; print(name,r.returncode,flush=True)
(out/'results.json').write_text(__import__('json').dumps(results,indent=2))
