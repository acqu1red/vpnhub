#!/usr/bin/env python3
"""Local automation. No production endpoints, credentials, or host network changes."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
import urllib.request
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def environment():
    env = os.environ.copy()
    local = ROOT / '.tools' / 'cargo'
    if (local / 'bin' / 'rustup').is_file():
        env['CARGO_HOME'] = str(local)
        env['RUSTUP_HOME'] = str(ROOT / '.tools' / 'rustup')
        env['PATH'] = str(local / 'bin') + os.pathsep + env.get('PATH', '')
    return env


def run(*args, capture=False):
    print('+', ' '.join(args), flush=True)
    result = subprocess.run(args, cwd=ROOT, env=environment(), check=True,
                            stdout=subprocess.PIPE if capture else None, text=True)
    return result.stdout


def doctor():
    env = environment()
    print('Host:', platform.system(), platform.machine())
    print('Windows SCM/TUN/E2E:', 'REQUIRES_GUEST_ADMIN_AND_STAGING')
    print('GitHub remote:', 'not configured by this command')
    for name in ['git', 'node', 'cargo', 'rustc', 'rustup']:
        print(name + ':', shutil.which(name, path=env.get('PATH')) or 'MISSING')
    if shutil.which('cargo', path=env.get('PATH')):
        run('cargo', '--version')
    print('No credentials or environment variable values are printed.')


def tests():
    run('cargo', 'fmt', '--all', '--', '--check')
    run('cargo', 'clippy', '--workspace', '--all-targets', '--locked', '--', '-D', 'warnings')
    run('cargo', 'test', '--workspace', '--locked')
    run('node', 'tools/signing-vectors.mjs', 'verify')
    run('node', '--test', 'tests/mock-issuer.test.mjs')
    run('cargo', 'run', '--locked', '--quiet', '-p', 'scrume-lab', '--', 'verify-fixture')
    run('cargo', 'run', '--locked', '--quiet', '-p', 'scrume-service', '--', '--capabilities')
    run(sys.executable, '-m', 'unittest', 'discover', '-s', 'tests/tools', '-v')


def download(url, expected, max_size=64 * 1024 * 1024):
    if not url.startswith('https://') or len(expected) != 64:
        raise ValueError('HTTPS URL and pinned SHA256 required')
    req = urllib.request.Request(url, headers={'User-Agent': 'ScrumeVPN-dev/0.1'})
    with urllib.request.urlopen(req, timeout=120) as response:
        if not response.geturl().startswith('https://'):
            raise ValueError('HTTPS downgrade rejected')
        data = response.read(max_size + 1)
    if len(data) > max_size:
        raise ValueError('archive size exceeds limit')
    if hashlib.sha256(data).hexdigest() != expected:
        raise ValueError('archive SHA256 mismatch')
    return data


def fetch_runtime():
    manifest = json.loads((ROOT / 'upstreams.lock.json').read_text())
    target = ROOT / '.cache' / 'runtime' / 'windows-x64'
    target.mkdir(parents=True, exist_ok=True)
    report = {'status': 'DOWNLOADED_NOT_EXECUTED', 'components': []}
    for component in manifest['runtime']:
        print('Fetching official pinned component:', component['name'], component['version'], flush=True)
        cache = ROOT / '.cache' / (component['sha256'] + '.zip')
        if cache.exists():
            data = cache.read_bytes()
            if hashlib.sha256(data).hexdigest() != component['sha256']:
                raise ValueError('cached archive SHA256 mismatch')
        else:
            data = download(component['url'], component['sha256'])
            cache.write_bytes(data)
        archive = zipfile.ZipFile(io.BytesIO(data))
        files = []
        # Explicit archive paths only; never extractall an untrusted archive.
        for source, dest in component['files'].items():
            destination = target / dest
            if Path(dest).name != dest:
                raise ValueError('manifest destination must be a basename')
            info = archive.getinfo(source)
            if info.file_size > 64 * 1024 * 1024:
                raise ValueError('member size exceeds limit')
            content = archive.read(info)
            destination.write_bytes(content)
            files.append({'file': dest, 'sha256': hashlib.sha256(content).hexdigest()})
        report['components'].append({'name': component['name'], 'version': component['version'],
                                     'archive_sha256': component['sha256'], 'files': files})
    (target / 'download-manifest.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Verified runtime saved in .cache/runtime/windows-x64; no binaries executed')


def licenses():
    metadata = json.loads(run('cargo', 'metadata', '--locked', '--format-version', '1', capture=True))
    rows = []
    notices = ['ScrumeVPN development dependency notices. Not a final MSI distribution manifest.\n']
    for package in metadata['packages']:
        if package['source'] is None:
            continue
        license_paths = []
        package_root = Path(package['manifest_path']).parent
        for path in sorted(package_root.iterdir()):
            if path.is_file() and any(word in path.name.upper() for word in ('LICENSE', 'COPYING', 'NOTICE')):
                license_paths.append({'path': path.name, 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
                notices.append('\n' + '=' * 72 + '\n' + package['name'] + ' ' + package['version']
                               + ' / ' + path.name + '\n' + path.read_text(errors='replace'))
        rows.append({'name': package['name'], 'version': package['version'], 'license': package['license'],
                     'license_file': package.get('license_file'), 'repository': package['repository'],
                     'source': package['source'], 'license_files': license_paths})
    out = ROOT / 'artifacts'
    out.mkdir(exist_ok=True)
    (out / 'dependencies.json').write_text(json.dumps(rows, indent=2) + '\n')
    (ROOT / 'THIRD_PARTY_NOTICES.txt').write_text('\n'.join(notices), encoding='utf-8')
    refs = {p['id']: 'pkg:cargo/' + p['name'] + '@' + p['version'] for p in metadata['packages']}
    components = [{'type': 'library', 'bom-ref': refs[p['id']], 'name': p['name'], 'version': p['version'],
                   'purl': 'pkg:cargo/' + p['name'] + '@' + p['version'],
                   **({'licenses': [{'expression': p['license'].replace('MIT/Apache-2.0', 'MIT OR Apache-2.0')}]} if p['license'] else {})}
                  for p in metadata['packages'] if p['source'] is not None]
    bom = {'bomFormat': 'CycloneDX', 'specVersion': '1.5', 'version': 1,
           'metadata': {'properties': [{'name': 'scrumevpn:scope', 'value': 'development Cargo graph, all targets; not final MSI'},
                                      {'name': 'scrumevpn:cargo-lock-sha256', 'value': hashlib.sha256((ROOT / 'Cargo.lock').read_bytes()).hexdigest()}]},
           'components': components,
           'dependencies': [{'ref': refs[node['id']], 'dependsOn': [refs[d] for d in node['dependencies']]} for node in metadata['resolve']['nodes']]}
    # Include workspace components because graph edges reference their IDs too.
    for p in metadata['packages']:
        if p['source'] is None:
            bom['components'].append({'type': 'application', 'bom-ref': refs[p['id']], 'name': p['name'], 'version': p['version']})
    (out / 'sbom.cdx.json').write_text(json.dumps(bom, indent=2) + '\n')
    unknown = [r['name'] for r in rows if not r['license'] and not r['license_file']]
    print('Dependency inventory:', len(rows), 'packages; unknown:', ', '.join(unknown) or 'none')
    print('Inventory is evidence, not full license approval or a release SBOM.')
    if unknown:
        raise ValueError('unresolved dependency license')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['doctor', 'test', 'fetch-runtime', 'licenses', 'check-windows'])
    args = parser.parse_args()
    commands = {'doctor': doctor, 'test': tests, 'fetch-runtime': fetch_runtime, 'licenses': licenses,
                'check-windows': lambda: run('cargo', 'check', '--locked', '--workspace', '--target', 'x86_64-pc-windows-msvc')}
    commands[args.command]()


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        # Only developer tooling errors; product never logs raw responses.
        print('DEV_CHECK_FAILED:', str(error), file=sys.stderr)
        sys.exit(1)
