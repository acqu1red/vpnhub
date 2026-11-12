"""Shared staging for unsigned native Windows lab builds (no execution)."""
import hashlib
import json
from pathlib import Path
import shutil


def stage(root, target, toolchain, service, tests, smoke, tool_licenses=()):
    out = root / 'artifacts/windows-lab'
    out.mkdir(parents=True, exist_ok=True)
    for source, name in [(service, 'ScrumeVPNSvc.exe'), (tests, 'windows-adapter-tests.exe'),
                         (smoke, 'ScrumeVPNLab.exe'),
                         (root / 'tools/windows/service-smoke.ps1', 'service-smoke.ps1'),
                         (root / 'THIRD_PARTY_NOTICES.txt', 'THIRD_PARTY_NOTICES.txt'), *tool_licenses]:
        shutil.copy2(source, out / name)
    launcher = (root / 'tools/windows/run-native-tests.cmd').read_text()
    (out / 'run-native-tests.cmd').write_bytes(launcher.replace('\n', '\r\n').encode('ascii'))
    (out / 'run-dpapi-tests.cmd').unlink(missing_ok=True)
    (out / 'READ-ME.txt').write_text('''UNSIGNED NATIVE LAB BUILD - NOT A VPN RELEASE
First run run-native-tests.cmd as the active console user without admin.
SCM smoke: in an elevated terminal in the disposable VM, run:
  D:\\ScrumeVPNLab.exe --disposable-lab
Use the actual CD drive letter. No PowerShell script policy changes are needed.
The EXE refuses an existing service/directory and a mismatched service hash.
It installs a temporary Manual LocalSystem status service in a newly protected
Program Files directory, tests status/stop, deletes the service, and verifies
SCM removal before PASS. Lab files are retained for inspection.
Only the fixed sibling ScrumeVPNSvc.exe can be installed, with its build-pinned
SHA256. No arbitrary binary paths, scripts or network settings are accepted.
No tunnel, enrollment, vault ACL ownership or boot-connect is implemented.
service-smoke.ps1 is a previous implementation for environments allowing signed
or approved scripts; do not change ExecutionPolicy, Defender or UAC for this lab.
''')
    record = {'status': 'BUILT_NOT_EXECUTED', 'target': target, 'signing': 'UNSIGNED_LAB_ONLY',
              'toolchain': toolchain,
              'files': {p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                        for p in sorted(out.iterdir()) if p.is_file() and p.name != 'build-manifest.json'}}
    (out / 'build-manifest.json').write_text(json.dumps(record, indent=2) + '\n')
    return out


def test_executable(output):
    found = []
    for line in output.splitlines():
        event = json.loads(line)
        if event.get('reason') == 'compiler-artifact' and event.get('profile', {}).get('test') and event.get('executable'):
            found.append(Path(event['executable']))
    if len(found) != 1:
        raise ValueError('Expected one native adapter test executable')
    return found[0]
