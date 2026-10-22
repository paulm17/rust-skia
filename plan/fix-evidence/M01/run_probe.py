import pathlib,subprocess,json
p=pathlib.Path('plan/fix-evidence/M01');ref=pathlib.Path('../skia').resolve(); results={}
def run(name,cmd):
 r=subprocess.run(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE);(p/(name+'.out')).write_bytes(r.stdout);(p/(name+'.log')).write_text(repr(cmd)+'\n'+r.stderr.decode()+'\nEXIT: '+str(r.returncode));results[name]=r.returncode
 if r.returncode: raise RuntimeError(name)
for profile,cf,rf in [('debug',['-O0','-g','-DSK_DEBUG'],[]),('release',['-O3','-DNDEBUG'],['-O'])]:
 run('probe-cpp-build-'+profile,['xcrun','clang++','-std=c++20','-ffp-contract=off','-Wl,-dead_strip','-DSK_DISABLE_TRACING','-I'+str(ref),*cf,str(p/'boundary.cpp'),*[str(ref/s) for s in ['src/core/SkPoint.cpp','src/core/SkScalar.cpp','src/core/SkFloatingPoint.cpp','src/core/SkDebug.cpp','src/core/SkLog.cpp','src/ports/SkLog_stdio.cpp','src/utils/SkLogHandler.cpp','src/core/SkMalloc.cpp','src/ports/SkMemory_malloc.cpp']],'-o',str(p/('cpp-'+profile))])
 run('probe-rust-build-'+profile,['rustc','--edition=2024',*rf,str(p/'boundary.rs'),'--extern','rust_skia='+str(pathlib.Path('target')/profile/'deps'/next((pathlib.Path('target')/profile/'deps').glob('librust_skia-*.rlib')).name),'-L','dependency='+str(pathlib.Path('target')/profile/'deps'),'-o',str(p/('rust-'+profile))])
 for lang in ['cpp','rust']:run('probe-'+lang+'-'+profile,[str(p/(lang+'-'+profile))])
 run('adaptations-build-'+profile,['rustc','--edition=2024',*rf,str(p/'adaptations.rs'),'--extern','rust_skia='+str(next((pathlib.Path('target')/profile/'deps').glob('librust_skia-*.rlib'))),'-L','dependency='+str(pathlib.Path('target')/profile/'deps'),'-o',str(p/('adaptations-'+profile))])
 run('adaptations-'+profile,[str(p/('adaptations-'+profile))])
 a=(p/('probe-cpp-'+profile+'.out')).read_text().split();b=(p/('probe-rust-'+profile+'.out')).read_text().split();diff=[(i,x,y) for i,(x,y) in enumerate(zip(a,b)) if x!=y];results['compare-'+profile]={'cpp_tokens':len(a),'rust_tokens':len(b),'differences':len(diff),'first_differences':diff[:30]}
(p/'probe-results.json').write_text(json.dumps(results,indent=2));print(json.dumps(results,indent=2))

assert all(v['differences']==0 and v['cpp_tokens']==v['rust_tokens'] for k,v in results.items() if k.startswith('compare-'))
