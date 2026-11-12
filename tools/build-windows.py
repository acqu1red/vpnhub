#!/usr/bin/env python3
"""Build the native MSVC lab on Windows; never installs services or changes policy."""
import argparse
import hashlib
import platform
import subprocess
from dev import ROOT, environment
from windows_payload import stage, test_executable


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target', choices=['x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc'])
    args = parser.parse_args()
    if platform.system() != 'Windows':
        raise RuntimeError('Requires Windows with MSVC Build Tools and Windows SDK')
    machine = platform.machine().lower()
    target = args.target or ('aarch64-pc-windows-msvc' if machine in ('arm64', 'aarch64') else 'x86_64-pc-windows-msvc')
    env = environment()
    subprocess.run(['rustup', 'target', 'add', target], cwd=ROOT, env=env, check=True)
    subprocess.run(['cargo', 'build', '--release', '--locked', '-p', 'scrume-service', '--target', target], cwd=ROOT, env=env, check=True)
    service = ROOT / 'target' / target / 'release/ScrumeVPNSvc.exe'
    env['SCRUME_LAB_SERVICE_SHA256'] = hashlib.sha256(service.read_bytes()).hexdigest()
    subprocess.run(['cargo', 'build', '--release', '--locked', '-p', 'scrume-lab', '--bin', 'ScrumeVPNLab', '--target', target], cwd=ROOT, env=env, check=True)
    tests = subprocess.run(['cargo', 'test', '--release', '--locked', '--no-run', '-p', 'scrume-windows', '--target', target, '--message-format=json'], cwd=ROOT, env=env, check=True, stdout=subprocess.PIPE, text=True)
    stage(ROOT, target, 'Rust 1.99.1 + MSVC/Windows SDK (record actual installed versions for acceptance)',
          service, test_executable(tests.stdout), ROOT / 'target' / target / 'release/ScrumeVPNLab.exe')
    print('Native MSVC payload in artifacts/windows-lab. Runtime tests NOT executed.')


if __name__ == '__main__':
    main()
