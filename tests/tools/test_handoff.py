import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
import zipfile

spec = importlib.util.spec_from_file_location('export_handoff', Path(__file__).resolve().parents[2] / 'tools/export_handoff.py')
handoff = importlib.util.module_from_spec(spec)
spec.loader.exec_module(handoff)


class HandoffTests(unittest.TestCase):
    def prepare(self, root):
        subprocess.run(['git', 'init', '-q', str(root)], check=True)
        (root / '.gitignore').write_text('artifacts/\n.tools/\n')
        (root / 'CONTINUE_HERE.md').write_text('Context and current work\n')
        subprocess.run(['git', '-C', str(root), 'add', '.'], check=True)
        subprocess.run(['git', '-C', str(root), '-c', 'user.name=Lab Test', '-c', 'user.email=lab@example.invalid', '-c', 'commit.gpgsign=false', 'commit', '-qm', 'fixture'], check=True)

    def test_history_restores_and_ignored_cache_is_excluded(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d) / 'source'
            root.mkdir()
            self.prepare(root)
            (root / '.tools').mkdir()
            (root / '.tools/private-cache').write_text('must not export')
            archive = handoff.export_handoff(root)
            with zipfile.ZipFile(archive) as zipped:
                self.assertFalse(any('.tools/' in name for name in zipped.namelist()))
                manifest = json.loads(zipped.read('ScrumeVPN/HANDOFF-MANIFEST.json'))
                for name, digest in manifest['files'].items():
                    self.assertEqual(hashlib.sha256(zipped.read('ScrumeVPN/' + name)).hexdigest(), digest)
                bundle = Path(d) / 'repository.bundle'
                bundle.write_bytes(zipped.read('ScrumeVPN/repository.bundle'))
            restored = Path(d) / 'restored'
            subprocess.run(['git', '-c', 'core.autocrlf=false', 'clone', '-q', str(bundle), str(restored)], check=True)
            self.assertEqual((root / 'CONTINUE_HERE.md').read_bytes(), (restored / 'CONTINUE_HERE.md').read_bytes())
            self.assertEqual(subprocess.check_output(['git', '-C', str(restored), 'rev-parse', 'HEAD']).decode().strip(), manifest['head'])

    def test_uncommitted_work_is_never_silently_lost(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            self.prepare(root)
            (root / 'new-notes.md').write_text('not committed yet')
            with self.assertRaises(ValueError):
                handoff.export_handoff(root)
