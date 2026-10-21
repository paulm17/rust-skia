#!/usr/bin/env python3
"""Validate plan accounting without modifying the plan or either repository."""
import argparse
import csv
import hashlib
import json
from pathlib import Path
import re
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--source', type=Path, required=True, help='Pinned C++ Skia checkout')
parser.add_argument('--check-hashes', action='store_true')
args = parser.parse_args()
root = Path(__file__).resolve().parent

def read(name):
    with (root / 'inventory' / name).open(newline='') as stream:
        return list(csv.DictReader(stream))

rows = read('source-files.csv')
cases = read('test-registrations.csv')
backends = read('backend-validation.csv')
hashes = read('source-hashes.csv')
status = json.loads((root / 'status.json').read_text())
tracked = set(subprocess.check_output(['git', 'ls-files'], cwd=args.source, text=True).splitlines())
current = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=args.source, text=True).strip()
assert current == status['upstream_commit'], 'Reference commit has changed; amend the plan explicitly'
indexed = {r['path']: r for r in rows}
assert len(indexed) == len(rows), 'Duplicate source ownership'
assert tracked == set(indexed), f'Unaccounted paths: {tracked ^ set(indexed)}'
assert set(indexed) == {h['path'] for h in hashes}, 'Source hash coverage differs'
documents = sorted((root / 'milestones').glob('*.md'))
assert len(documents) == len(status['milestones']) == 47
assert {int(p.name[:2]) for p in documents} == set(range(1,48))
for row in rows:
    assert 0 <= int(row['milestone']) <= 47
    assert (row['milestone'] == '0') == (row['disposition'] == 'excluded-platform')
for case in cases:
    assert case['source'] in indexed
    line = (args.source / case['source']).read_text(errors='replace').splitlines()[int(case['line'])-1]
    assert case['macro'] in line, f'Bad source location: {case}'
    assert 0 <= int(case['milestone']) <= 47
for row in backends:
    assert row['source'] in indexed
    assert 32 <= int(row['milestone']) <= 37
for doc in documents:
    text = doc.read_text()
    for heading in ('Port these production files (Deepseek)', 'Port these test/runner/support files (Codex)', 'Commands, required evidence and stop condition'):
        assert heading in text, f'Missing ownership/gate heading in {doc.name}'
for doc in root.rglob('*.md'):
    for target in re.findall(r'\]\(([^)]+)\)', doc.read_text()):
        if '://' not in target and not target.startswith('#'):
            assert (doc.parent / target.split('#')[0]).exists(), f'Broken link in {doc}: {target}'
if args.check_hashes:
    for item in hashes:
        actual = hashlib.sha256((args.source / item['path']).read_bytes()).hexdigest()
        assert actual == item['sha256'], f'Reference file changed: {item["path"]}'
print(f'PASS: {len(rows)} source paths, {len(documents)} milestones, {len(cases)} registration sites, {len(backends)} backend obligations; links and source locations valid.')
print('This checks accounting, not runtime dependency completeness or implementation correctness.')
