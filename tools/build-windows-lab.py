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
import subprocess
import tarfile
from dev import ROOT, environment, download
from windows_payload import stage, test_executable

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
    service = ROOT / 'target' / TARGET / 'release/ScrumeVPNSvc.exe'
    env['SCRUME_LAB_SERVICE_SHA256'] = hashlib.sha256(service.read_bytes()).hexdigest()
    subprocess.run(['cargo', 'build', '--locked', '--release', '-p', 'scrume-lab', '--bin', 'ScrumeVPNLab', '--target', TARGET],
                   env=env, cwd=ROOT, check=True)
    licenses = [(tool / source, name) for source, name in [
        ('LICENSE', 'ZIG-LICENSE.txt'), ('lib/libc/mingw/COPYING', 'MINGW-COPYING.txt'),
        ('lib/libunwind/LICENSE.TXT', 'LLVM-UNWIND-LICENSE.txt')]]
    out = stage(ROOT, TARGET, 'Rust 1.99.1 + Zig ' + pin['version'], service,
                test_executable(result.stdout), ROOT / 'target' / TARGET / 'release/ScrumeVPNLab.exe', licenses)
    if args.iso:
        mkisofs = '/Applications/VMware Fusion.app/Contents/Library/mkisofs'
        subprocess.run([mkisofs, '-quiet', '-J', '-R', '-V', 'SCRUMEVPN_LAB', '-o', str(ROOT / 'artifacts/ScrumeVPN-Lab-ARM64-SCM.iso'), str(out)], check=True)
    print('ARM64 lab payload built; this build has NOT been executed in Windows')


if __name__ == '__main__':
    main()
