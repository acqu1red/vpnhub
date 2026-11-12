#!/usr/bin/env python3
"""Install an isolated pinned Rust toolchain; never edits shell profiles."""
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
TOOLCHAIN = '1.99.1'


def main():
    env = os.environ.copy()
    env['CARGO_HOME'] = str(ROOT / '.tools/cargo')
    env['RUSTUP_HOME'] = str(ROOT / '.tools/rustup')
    exe = 'rustup.exe' if os.name == 'nt' else 'rustup'
    rustup = ROOT / '.tools/cargo/bin' / exe
    if not rustup.exists():
        # A new host needs an independently reviewed/pinned official bootstrap.
        # Only the host actually audited in this session is pre-approved here.
        if platform.system() != 'Darwin' or platform.machine() != 'arm64':
            raise RuntimeError('Use official rustup.rs for this host, or add a reviewed bootstrap SHA pin. No unknown executable will run.')
        evidence = json.loads((ROOT / 'docs/evidence/rust-bootstrap.json').read_text())
        if evidence['toolchain'] != TOOLCHAIN or not evidence['url'].startswith('https://static.rust-lang.org/'):
            raise RuntimeError('invalid bootstrap pin')
        with urllib.request.urlopen(evidence['url'], timeout=120) as response:
            content = response.read(32 * 1024 * 1024 + 1)
        if len(content) > 32 * 1024 * 1024 or hashlib.sha256(content).hexdigest() != evidence['sha256']:
            raise RuntimeError('official bootstrap checksum mismatch')
        installer = ROOT / '.tools/rustup-init'
        installer.parent.mkdir(parents=True, exist_ok=True)
        installer.write_bytes(content)
        installer.chmod(0o700)
        subprocess.run([str(installer), '-y', '--no-modify-path', '--profile', 'minimal',
                        '--default-toolchain', TOOLCHAIN], env=env, check=True, cwd=ROOT)
    subprocess.run([str(rustup), 'toolchain', 'install', TOOLCHAIN, '--profile', 'minimal',
                    '--component', 'rustfmt', '--component', 'clippy'], env=env, check=True, cwd=ROOT)
    print('Pinned Rust ready in .tools; run python3 tools/dev.py test')


if __name__ == '__main__':
    try:
        main()
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print('BOOTSTRAP_FAILED:', str(error), file=sys.stderr)
        sys.exit(1)
