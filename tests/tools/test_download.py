import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch, MagicMock

spec = importlib.util.spec_from_file_location('dev', Path(__file__).resolve().parents[2] / 'tools' / 'dev.py')
dev = importlib.util.module_from_spec(spec)
spec.loader.exec_module(dev)


class DownloadTests(unittest.TestCase):
    def test_rejects_http_before_network(self):
        with patch.object(dev.urllib.request, 'urlopen') as network:
            with self.assertRaises(ValueError):
                dev.download('http://example.invalid/archive', '0' * 64)
            network.assert_not_called()

    def test_rejects_corrupted_binary(self):
        response = MagicMock()
        response.__enter__.return_value = response
        response.geturl.return_value = 'https://example.invalid/archive'
        response.read.return_value = b'corrupted executable'
        with patch.object(dev.urllib.request, 'urlopen', return_value=response):
            with self.assertRaisesRegex(ValueError, 'SHA256'):
                dev.download('https://example.invalid/archive', '0' * 64)

    def test_rejects_https_downgrade(self):
        response = MagicMock()
        response.__enter__.return_value = response
        response.geturl.return_value = 'http://example.invalid/archive'
        with patch.object(dev.urllib.request, 'urlopen', return_value=response):
            with self.assertRaisesRegex(ValueError, 'downgrade'):
                dev.download('https://example.invalid/archive', '0' * 64)

    def test_rejects_oversized_archive(self):
        response = MagicMock()
        response.__enter__.return_value = response
        response.geturl.return_value = 'https://example.invalid/archive'
        response.read.return_value = b'12345'
        with patch.object(dev.urllib.request, 'urlopen', return_value=response):
            with self.assertRaisesRegex(ValueError, 'size'):
                dev.download('https://example.invalid/archive', '0' * 64, max_size=4)
