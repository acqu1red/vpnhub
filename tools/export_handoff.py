"""Private local source/history export. Never uploads or copies caches/credentials."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import zipfile


def git(root, *args):
    return subprocess.check_output(['git', '-C', str(root), *args], stderr=subprocess.PIPE)


def export_handoff(root):
    root = Path(root).resolve()
    if git(root, 'status', '--porcelain', '--untracked-files=normal').strip():
        raise ValueError('Commit all source/document changes before export; nothing is silently omitted')
    revision = git(root, 'rev-parse', 'HEAD').decode().strip()
    files = git(root, 'ls-files', '-z').decode('utf-8').strip('\0').split('\0')
    excluded = {'.git', '.tools', '.cache', 'target', 'artifacts', 'node_modules'}
    for name in files:
        path = Path(name)
        if (path.is_absolute() or '..' in path.parts or set(path.parts) & excluded
                or path.suffix.lower() in {'.pfx', '.p12', '.pem', '.key'}
                or path.name == '.env' or path.name.startswith('.env.')
                or (root / path).is_symlink() or not (root / path).is_file()):
            raise ValueError('Export refused: forbidden path, secret file, submodule or symlink: ' + name)
    destination = root / 'artifacts' / 'handoff'
    destination.mkdir(parents=True, exist_ok=True)
    archive = destination / ('ScrumeVPN-handoff-' + revision[:12] + '.zip')
    if archive.exists():
        raise ValueError('This commit already has an export; verify the existing archive instead of overwriting it')
    with tempfile.TemporaryDirectory(prefix='scrume-handoff-') as temporary:
        bundle = Path(temporary) / 'repository.bundle'
        git(root, 'bundle', 'create', str(bundle), '--all')
        git(root, 'bundle', 'verify', str(bundle))
        hashes = {name: hashlib.sha256((root / name).read_bytes()).hexdigest() for name in sorted(files)}
        hashes['repository.bundle'] = hashlib.sha256(bundle.read_bytes()).hexdigest()
        manifest = {'format_version': 1, 'visibility': 'PRIVATE_LOCAL_TRANSFER_ONLY',
                    'head': revision, 'branch': git(root, 'branch', '--show-current').decode().strip(),
                    'scope': 'Committed source/docs/evidence and Git history; no tools, VM, tokens or caches',
                    'files': hashes}
        temporary_zip = Path(temporary) / 'handoff.zip'
        with zipfile.ZipFile(temporary_zip, 'w', zipfile.ZIP_DEFLATED) as out:
            for name in sorted(files):
                out.write(root / name, 'ScrumeVPN/' + name)
            out.write(bundle, 'ScrumeVPN/repository.bundle')
            out.writestr('ScrumeVPN/HANDOFF-MANIFEST.json', json.dumps(manifest, indent=2) + '\n')
        with zipfile.ZipFile(temporary_zip) as check:
            if check.testzip() is not None:
                raise ValueError('Export ZIP checksum failed')
            for name, expected in hashes.items():
                if hashlib.sha256(check.read('ScrumeVPN/' + name)).hexdigest() != expected:
                    raise ValueError('Export content mismatch')
        # create_new prevents replacing an existing archive even in a race.
        with archive.open('xb') as output:
            output.write(temporary_zip.read_bytes())
    checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
    archive.with_suffix('.zip.sha256').write_text(checksum + '  ' + archive.name + '\n')
    print('Private handoff archive:', archive)
    print('HEAD:', revision, 'files:', len(files), 'SHA256:', checksum)
    print('Verified source ZIP and Git bundle; clean Windows build/restore is a separate check.')
    return archive
