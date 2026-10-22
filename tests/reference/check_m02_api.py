#!/usr/bin/env python3
"""Record API failures independently; never turn expected compiler errors into PASS."""
import json, subprocess
from pathlib import Path
out=Path('plan/acceptance/M02-evidence')
summary={}
for profile in ['debug','release']:
 args=['cargo','build','--locked','--message-format=json']+(['--release'] if profile=='release' else [])
 r=subprocess.run(args,text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 rlib=None
 for line in r.stdout.splitlines():
  m=json.loads(line)
  if m.get('reason')=='compiler-artifact' and m['target']['name']=='rust_skia':
   rlib=next((x for x in m['filenames'] if x.endswith('.rlib')),rlib)
 if not rlib:raise RuntimeError(r.stderr)
 deps=Path(rlib).parent
 for path in sorted(Path('tests/m02-api').glob('*.rs')):
  cmd=['rustc','--edition=2024','--test',str(path),'--extern','rust_skia='+rlib,'-L','dependency='+str(deps/'deps'),'-L','dependency='+str(deps),'-o',str(out/('api-'+profile+'-'+path.stem))]+(['-C','opt-level=3','-C','debug-assertions=no']if profile=='release' else [])
  r=subprocess.run(cmd,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
  (out/('api-'+profile+'-'+path.stem+'.log')).write_text('COMMAND: '+repr(cmd)+'\n'+r.stdout+'\nEXIT: '+str(r.returncode)+'\n')
  summary[profile+'-'+path.stem]={'compile_exit':r.returncode}
  if path.stem in {'erasure_type_safety', 'erasure_sized_type_safety'}:
   summary[profile+'-'+path.stem]['expected_compile_rejection'] = True
   summary[profile+'-'+path.stem]['contract_failed'] = r.returncode == 0
   continue
  if r.returncode==0:
   cmd=[str(out/('api-'+profile+'-'+path.stem)),'--nocapture']
   r=subprocess.run(cmd,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
   (out/('api-run-'+profile+'-'+path.stem+'.log')).write_text(r.stdout+'\nEXIT: '+str(r.returncode)+'\n')
   summary[profile+'-'+path.stem]['run_exit']=r.returncode
(out/'api-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
raise SystemExit(int(any(x['contract_failed'] if x.get('expected_compile_rejection') else x['compile_exit'] or x.get('run_exit',0) for x in summary.values())))
