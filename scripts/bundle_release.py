#!/usr/bin/env python3
"""Bundle native output, original source and notices; checksum every release file."""
# SPDX-License-Identifier: GPL-3.0-only
import hashlib
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tarfile

root = Path(__file__).resolve().parents[1]
os.chdir(root)
version = next(line.split('=', 1)[1] for line in Path('gradle.properties').read_text().splitlines()
               if line.startswith('version='))
if not re.fullmatch(r'(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)', version):
    raise SystemExit('Native release packaging requires a stable SemVer version')
if subprocess.check_output(['git', 'status', '--porcelain'], text=True).strip():
    raise SystemExit('Commit all source changes before generating a release')
tag = os.environ.get('GITHUB_REF', '')
if tag.startswith('refs/tags/') and tag != f'refs/tags/v{version}':
    raise SystemExit('Release tag must match gradle.properties version')
out = root / 'build/release'
out.mkdir(parents=True, exist_ok=True)
if any(out.iterdir()):
    raise SystemExit('Release output must be empty; run a clean build first')
native = root / 'launcher/build/native'
if not native.is_dir() or not any(native.iterdir()):
    raise SystemExit('Run :launcher:packageNative first')
stem = f'aero-client-{version}-{platform.system().lower()}-{platform.machine().lower()}'
stage = root / 'build/release-stage' / stem
stage.mkdir(parents=True, exist_ok=False)
shutil.copytree(native, stage / 'application')
for name in ['LICENSE', 'README.md', 'PRIVACY.md', 'THIRD_PARTY_LICENSES.md']:
    shutil.copy2(root / name, stage / name)
shutil.copytree(root / 'third-party', stage / 'third-party')
commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
jdk = subprocess.run(['java', '-version'], capture_output=True, text=True, check=True)
(stage / 'BUILD-INFO.txt').write_text(
    f'Version: {version}\nCommit: {commit}\nHost: {platform.platform()}\n'
    f'JAVA_HOME: {os.environ.get("JAVA_HOME", "unset")}\n{jdk.stderr}{jdk.stdout}\n'
    'Runtime source: https://github.com/adoptium/temurin21-binaries/releases\n'
    'Select the exact build above and its Sources asset. Preserve runtime legal notices.\n', encoding='utf-8')
if platform.system() == 'Windows':
    shutil.make_archive(str(out / stem), 'zip', stage.parent, stage.name)
else:
    with tarfile.open(out / f'{stem}.tar.gz', 'w:gz') as archive:
        archive.add(stage, arcname=stem)
subprocess.run(['git', 'archive', '--format=zip', f'--prefix=aero-client-{version}/',
                '-o', str(out / f'aero-client-{version}-source.zip'), 'HEAD'], check=True)
for module in ['launcher', 'client']:
    for jar in (root / module / 'build/libs').glob('*.jar'):
        shutil.copy2(jar, out / jar.name)
lines = []
for file in sorted(out.iterdir()):
    if file.is_file():
        with file.open('rb') as stream:
            lines.append(f'{hashlib.file_digest(stream, "sha256").hexdigest()}  {file.name}')
(out / 'SHA256SUMS.txt').write_text('\n'.join(lines) + '\n', encoding='utf-8')
print(f'Release artifacts: {out}')
