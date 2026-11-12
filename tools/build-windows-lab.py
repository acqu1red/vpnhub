#!/usr/bin/env python3
"""Build an unsigned native ARM64 lab EXE and DPAPI tests on Apple Silicon.

Does not run Windows code, change VM configuration, or replace release MSVC CI.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tarfile
from dev import ROOT, environment, download

TARGET = 'aarch64-pc-windows-gnullvm'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--iso', action='store_true', help='Create a read-only payload ISO using Fusion mkisofs')
    args = parser.parse_args()
    if (platform.system(), platform.machine()) != ('Darwin', 'arm64'):
        raise RuntimeError('This optional lab builder currently supports macOS arm64 only')
    pin = json.loads((ROOT / 'upstreams.lock.json').read_text())['build_tools']['zig_macos_arm64']
    tool = ROOT / '.tools' / ('zig-aarch64-macos-' + pin['version'])
    archive = ROOT / '.tools' / 'zig.tar.xz'
    if archive.exists():
        if hashlib.sha256(archive.read_bytes()).hexdigest() != pin['sha256']:
            raise RuntimeError('Zig archive checksum mismatch')
    else:
        archive.parent.mkdir(exist_ok=True)
        archive.write_bytes(download(pin['url'], pin['sha256']))
    if not (tool / 'zig').exists():
        with tarfile.open(archive) as tar:
            for member in tar.getmembers():
                destination = (ROOT / '.tools' / member.name).resolve()
                if not destination.is_relative_to(ROOT / '.tools') or member.issym() or member.islnk():
                    raise RuntimeError('Unsafe archive member')
            tar.extractall(ROOT / '.tools')
    env = environment()
    subprocess.run(['rustup', 'target', 'add', TARGET], env=env, cwd=ROOT, check=True)
    linker = ROOT / '.tools' / 'zig-arm-linker.py'
    # Rust gnullvm is explicitly UCRT-based. Its libmsvcrt.a name is an LLVM-
    # mingw alias for UCRT. Zig 0.15 provides that runtime via -lc. This mapping
    # must NEVER be used for the older MSVCRT-based windows-gnu target.
    linker.write_text('''#!/usr/bin/env python3
import os, sys
from pathlib import Path
r = Path(__file__).resolve().parents[1]
os.environ['ZIG_GLOBAL_CACHE_DIR'] = str(r / '.tools/zig-cache')
args = [('-lc' if a == '-lmsvcrt' else a) for a in sys.argv[1:]
        if a not in ['-Wl,-Bdynamic', '-Wl,-Bstatic', '-Wl,-O1']]
os.execv(str(r / '.tools/zig-aarch64-macos-0.15.2/zig'), ['zig', 'cc', '-target', 'aarch64-windows-gnu'] + args)
''')
    linker.chmod(0o700)
    env['CARGO_TARGET_AARCH64_PC_WINDOWS_GNULLVM_LINKER'] = str(linker)
    subprocess.run(['cargo', 'build', '--locked', '--release', '-p', 'scrume-service', '--target', TARGET],
                   env=env, cwd=ROOT, check=True)
    result = subprocess.run(['cargo', 'test', '--locked', '--release', '--no-run', '-p', 'scrume-windows',
                             '--target', TARGET, '--message-format=json'],
                            env=env, cwd=ROOT, check=True, stdout=subprocess.PIPE, text=True)
    tests = []
    for line in result.stdout.splitlines():
        event = json.loads(line)
        if event.get('reason') == 'compiler-artifact' and event.get('profile', {}).get('test') and event.get('executable'):
            tests.append(Path(event['executable']))
    if len(tests) != 1:
        raise RuntimeError('Expected one native Windows adapter test executable')
    out = ROOT / 'artifacts' / 'windows-lab'
    out.mkdir(parents=True, exist_ok=True)
    shutil.copy2(ROOT / 'target' / TARGET / 'release/ScrumeVPNSvc.exe', out / 'ScrumeVPNSvc.exe')
    shutil.copy2(tests[0], out / 'windows-adapter-tests.exe')
    shutil.copy2(ROOT / 'tools/windows/service-smoke.ps1', out / 'service-smoke.ps1')
    launcher = (ROOT / 'tools/windows/run-native-tests.cmd').read_text()
    (out / 'run-native-tests.cmd').write_bytes(launcher.replace('\n', '\r\n').encode('ascii'))
    # Remove only the superseded launcher produced by this builder.
    (out / 'run-dpapi-tests.cmd').unlink(missing_ok=True)
    shutil.copy2(ROOT / 'THIRD_PARTY_NOTICES.txt', out / 'THIRD_PARTY_NOTICES.txt')
    for source, name in [('LICENSE', 'ZIG-LICENSE.txt'), ('lib/libc/mingw/COPYING', 'MINGW-COPYING.txt'), ('lib/libunwind/LICENSE.TXT', 'LLVM-UNWIND-LICENSE.txt')]:
        shutil.copy2(tool / source, out / name)
    (out / 'READ-ME.txt').write_text('''UNSIGNED ARM64 LAB BUILD - NOT A VPN RELEASE
Target: aarch64-pc-windows-gnullvm (UCRT). Does not contain Xray or Wintun.
First run run-native-tests.cmd as the active Windows console user, without admin.
It runs 3 DPAPI tests and 1 console-only IPC test, and saves a log in guest TEMP.
The service must not be running during the console IPC test.
For the SCM/status smoke test, use elevated PowerShell in the disposable VM:
powershell -NoProfile -File .\\service-smoke.ps1 -Binary .\\ScrumeVPNSvc.exe -DisposableLab
Do not lower ExecutionPolicy; review or sign the local script if your policy requires it.
The smoke script installs a temporary SYSTEM service, reads status, stops/removes
its own service, and keeps lab binaries under Program Files for inspection.
It does not implement a tunnel, enrollment, vault ACLs, or boot-connect.
''')
    record = {'status': 'BUILT_NOT_EXECUTED', 'target': TARGET, 'signing': 'UNSIGNED_LAB_ONLY',
              'toolchain': 'Rust 1.99.1 + Zig ' + pin['version'],
              'zig_archive_sha256': pin['sha256'],
              'files': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.iterdir()) if p.is_file() and p.name != 'build-manifest.json'}}
    (out / 'build-manifest.json').write_text(json.dumps(record, indent=2) + '\n')
    if args.iso:
        mkisofs = '/Applications/VMware Fusion.app/Contents/Library/mkisofs'
        subprocess.run([mkisofs, '-quiet', '-J', '-R', '-V', 'SCRUMEVPN_LAB', '-o', str(ROOT / 'artifacts/ScrumeVPN-Lab-ARM64.iso'), str(out)], check=True)
    print('ARM64 lab payload built; native Windows tests have NOT been executed')


if __name__ == '__main__':
    main()
