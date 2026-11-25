#!/usr/bin/env python3
"""Audit exact locked Maven dependencies with OSV; fail on errors or known findings."""
# SPDX-License-Identifier: GPL-3.0-only
import json
from pathlib import Path
import urllib.request

root = Path(__file__).resolve().parents[1]
packages = set()
for lock in root.glob('*/gradle.lockfile'):
    for line in lock.read_text(encoding='utf-8').splitlines():
        if line and not line.startswith(('#', 'empty=')):
            group, name, version = line.split('=', 1)[0].split(':')
            packages.add((f'{group}:{name}', version))
if not packages:
    raise SystemExit('No locked dependencies found; refusing an empty audit.')
ordered = sorted(packages)
payload = {'queries': [{'package': {'name': name, 'ecosystem': 'Maven'}, 'version': version}
                       for name, version in ordered]}
request = urllib.request.Request('https://api.osv.dev/v1/querybatch',
    data=json.dumps(payload).encode(), headers={'Content-Type': 'application/json'})
with urllib.request.urlopen(request, timeout=60) as response:
    results = json.load(response)['results']
if len(results) != len(ordered):
    raise SystemExit('Incomplete audit response')
findings = []
for package, result in zip(ordered, results):
    for vulnerability in result.get('vulns', []):
        findings.append(f'{package[0]}:{package[1]}: {vulnerability["id"]}')
if findings:
    raise SystemExit('\n'.join(findings))
print(f'No known OSV findings in {len(ordered)} locked Maven packages.')
