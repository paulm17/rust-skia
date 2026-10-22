#!/usr/bin/env python3
"""Compile unchanged pinned test closure; discover then isolate each registration."""
import argparse
import csv
import json
import re
import subprocess
from pathlib import Path
p = argparse.ArgumentParser()
p.add_argument('--source', required=True)
p.add_argument('--output', default='plan/acceptance/M02-evidence')
a = p.parse_args()
source = Path(a.source).resolve()
out = Path(a.output).resolve()
out.mkdir(parents=True, exist_ok=True)
rows = [r for r in csv.DictReader(open('plan/inventory/test-registrations.csv')) if r['milestone'] == '2']
tests = list(dict.fromkeys(r['source'] for r in rows))
production = '''tests/Test.cpp tools/flags/CommandLineFlags.cpp tests/CtsEnforcement.cpp
src/core/SkPoint.cpp src/core/SkScalar.cpp src/core/SkFloatingPoint.cpp src/core/SkRect.cpp
src/core/SkString.cpp src/core/SkStringUtils.cpp src/core/SkMalloc.cpp src/core/SkDebug.cpp
src/ports/SkMemory_malloc.cpp src/core/SkLog.cpp src/ports/SkLog_stdio.cpp
src/core/SkSafeMath.cpp src/utils/SkLogHandler.cpp src/core/SkArenaAlloc.cpp
src/core/SkData.cpp src/core/SkDataTable.cpp src/core/SkStream.cpp src/core/SkDeque.cpp
src/core/SkContainers.cpp src/core/SkMD5.cpp src/core/SkUTF.cpp src/core/SkBlockAllocator.cpp
src/core/SkTDArray.cpp src/core/SkSemaphore.cpp src/core/SkSharedMutex.cpp src/core/SkSpinlock.cpp
src/core/SkExecutor.cpp src/core/SkExecutors.cpp src/core/SkTaskGroup.cpp src/core/SkTime.cpp
src/ports/SkOSFile_posix.cpp src/ports/SkOSFile_stdio.cpp src/utils/SkOSPath.cpp
src/core/SkBuffer.cpp tools/SkMetaData.cpp src/core/SkThreadID.cpp src/core/SkChecksum.cpp'''.split()
def run(label, command, timeout=300):
    try:
        r = subprocess.run(command, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout)
        code, output = r.returncode, r.stdout
    except subprocess.TimeoutExpired as e:
        code, output = 124, (e.stdout or b'').decode() if isinstance(e.stdout, bytes) else (e.stdout or '')
        output += '\nTIMEOUT\n'
    (out / (label+'.log')).write_text('COMMAND: '+repr(command)+'\n'+output+'\nEXIT: '+str(code)+'\n')
    print(label, 'exit', code, flush=True)
    return code, output
summary = {}
for profile, flags in [('debug', ['-O0', '-g', '-DSK_DEBUG']), ('release', ['-O3', '-DNDEBUG'])]:
    binary = str(out / ('reference-'+profile))
    command = ['xcrun', 'clang++', '-std=c++20', '-Wl,-dead_strip', '-DSK_DISABLE_TRACING', '-I'+str(source)] + flags + ['tests/reference/m02.cpp','tests/reference/m02_contracts.cpp'] + [str(source / s) for s in tests+production] + ['-o', binary]
    code, _ = run('reference-build-'+profile, command)
    if code:
        summary[profile] = {'build_exit': code}
        continue
    contract_exit, _ = run('reference-contracts-'+profile, [binary,'--contracts'])
    code, output = run('reference-discovery-'+profile, [binary])
    discovered = re.findall(r'^REGISTERED (\w+) (CPU|CPUSerial)$', output, re.M)
    names = [x[0] for x in discovered]
    expected = {r['test'] for r in rows}
    results = {}
    for name in names:
        result, _ = run('reference-'+profile+'-'+name, [binary, name], 120)
        results[name] = result
    summary[profile] = {'contracts_exit': contract_exit, 'discovery_exit': code, 'registry_matches': len(names) == 110 and len(set(names)) == 110 and expected-set(names) == {'RawPtr_DanglingPointerDetected'} and not set(names)-expected, 'discovered': discovered, 'missing': sorted(expected-set(names)), 'extra': sorted(set(names)-expected), 'results': results}
(out/'reference-summary.json').write_text(json.dumps(summary, indent=2)+'\n')
raise SystemExit(int(any('build_exit' in s or s.get('contracts_exit') or s.get('discovery_exit') or not s.get('registry_matches', False) or s.get('extra') or any(s.get('results', {}).values()) for s in summary.values())))
