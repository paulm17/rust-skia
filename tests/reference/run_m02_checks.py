#!/usr/bin/env python3
"""Capture cumulative checks and isolate all executable M02 cases, without stopping on failure."""
import csv
import json
import subprocess
from pathlib import Path
out=Path('plan/acceptance/M02-evidence')
out.mkdir(parents=True,exist_ok=True)
summary={}
def run(label,cmd,timeout=120):
 try:
  r=subprocess.run(cmd,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=timeout)
  code,output=r.returncode,r.stdout
 except subprocess.TimeoutExpired as e:
  code,output=124,(e.stdout or b'').decode() if isinstance(e.stdout,bytes) else (e.stdout or '')
  output+='\nTIMEOUT\n'
 (out/(label+'.log')).write_text('COMMAND: '+repr(cmd)+'\n'+output+'\nEXIT: '+str(code)+'\n')
 print(label,'exit',code,flush=True)
 summary[label]={'exit':code}
 return code,output
for profile in ['debug','release']:
 flags=['--release']if profile=='release'else[]
 run('cargo-test-'+profile,['cargo','test']+flags+['--locked','--','--nocapture'])
 # The requested command stops at a failing integration binary. Continue so all
 # previously accepted binaries execute even when the M02 binary fails.
 run('cargo-cumulative-'+profile,['cargo','test']+flags+['--locked','--no-fail-fast','--','--nocapture'])
 code,output=run('cargo-m02-build-'+profile,['cargo','test']+flags+['--locked','--test','m02_test','--no-run','--message-format=json'])
 if code:continue
 binary=None
 for line in output.splitlines():
  try:m=json.loads(line)
  except ValueError:continue
  if m.get('reason')=='compiler-artifact' and m.get('executable') and m.get('target',{}).get('name')=='m02_test' and m.get('target',{}).get('kind')==['test'] and m.get('profile',{}).get('test'):binary=m['executable']
 if not binary:raise RuntimeError('M02 binary absent')
 code,output=run('rust-discovery-'+profile,[binary,'--list'])
 names=[line.split(': test')[0]for line in output.splitlines()if line.endswith(': test')]
 expected={r['test'] for r in csv.DictReader(open('plan/inventory/test-registrations.csv')) if r['milestone']=='2'} - {'RawPtr_DanglingPointerDetected'}
 originals={n.split('::')[-1] for n in names if not n.startswith(('support::','supplemental::'))}
 if code or originals != expected or len(names)!=122 or len(set(names))!=122:raise RuntimeError('Rust M02 discovery mismatch: '+repr((binary,originals^expected,len(names))))
 results={}
 for name in names:
  code,_=run('rust-'+profile+'-'+name.replace('::','-'),[binary,name,'--exact','--nocapture'],60)
  results[name]=code
 summary['isolated-'+profile]={'results':results}
run('api-driver',['python3','tests/reference/check_m02_api.py'])
run('m01-reference-driver',['python3','tests/reference/run_m01.py','--source','/Volumes/Data/Users/paul/development/src/github/skia','--output',str(out/'M01-reference')],120)
run('fmt-check',['cargo','fmt','--all','--','--check'])
run('clippy',['cargo','clippy','--all-targets','--locked','--','-D','warnings'])
run('validate-plan',['python3','plan/validate_plan.py','--source','/Volumes/Data/Users/paul/development/src/github/skia','--check-hashes'])
(out/'checks-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
raise SystemExit(int(any(x.get('exit',0) or any(x.get('results',{}).values())for x in summary.values())))
