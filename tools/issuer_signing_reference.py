#!/usr/bin/env python3
"""Proposed Python signing reference, not a deployed issuer extension.

Generation uses an ephemeral private key that is never serialized or saved.
Requires the optional pinned signing reference environment; CI verifies the public
fixture with Rust without installing Python crypto dependencies.
"""
import argparse
import base64
import json
from pathlib import Path
from cryptography import __version__ as crypto_version
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

ROOT = Path(__file__).resolve().parents[1]
PREFIX = b'scrumevpn/bundle/v1\x00'


def encode(value):
    return base64.urlsafe_b64encode(value).rstrip(b'=').decode('ascii')


def sign_bundle(bundle, private_key, key_id):
    if bundle['key_id'] != key_id or not key_id or len(key_id.encode('utf-8')) > 64:
        raise ValueError('key binding invalid')
    # Deliberately different whitespace/key order from the Node signer.
    # The verifier checks the exact transmitted bytes, not a reserialization.
    raw = json.dumps(bundle, sort_keys=True, indent=1, ensure_ascii=False, allow_nan=False).encode('utf-8')
    if len(raw) > 32768:
        raise ValueError('payload too large')
    return {'key_id': key_id, 'payload': encode(raw), 'signature': encode(private_key.sign(PREFIX + raw))}


def generate():
    source = json.loads((ROOT / 'tests/fixtures/signing-v1.json').read_text())
    valid = next(case['envelope'] for case in source['cases'] if case['name'] == 'valid')
    raw = base64.urlsafe_b64decode(valid['payload'] + '=' * (-len(valid['payload']) % 4))
    payload = json.loads(raw)
    key_id = 'python-reference-only-1'
    payload['key_id'] = key_id
    key = Ed25519PrivateKey.generate()
    envelope = sign_bundle(payload, key, key_id)
    public = key.public_key().public_bytes(serialization.Encoding.Raw, serialization.PublicFormat.Raw)
    fixture = {'description': 'PUBLIC TEST DATA ONLY; Python/OpenSSL signature, no private key saved',
               'generator': 'cryptography==' + crypto_version, 'key_id': key_id, 'public_key': encode(public),
               'enrollment_id': source['enrollment_id'], 'now': source['now'],
               'cases': [{'name': 'valid', 'expected': 'ok', 'envelope': envelope}]}
    (ROOT / 'tests/fixtures/signing-python-v1.json').write_text(json.dumps(fixture, indent=2) + '\n')
    # Also prove this Python backend accepts the Node-generated signature.
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
    node_key = Ed25519PublicKey.from_public_bytes(base64.urlsafe_b64decode(source['public_key'] + '=' * (-len(source['public_key']) % 4)))
    node_key.verify(base64.urlsafe_b64decode(valid['signature'] + '=' * (-len(valid['signature']) % 4)), PREFIX + raw)
    print('Python generated public fixture and verified Node signature; private key never exported')


def verify_public_fixtures():
    from cryptography.exceptions import InvalidSignature
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey
    def decode(s):
        return base64.urlsafe_b64decode(s + '=' * (-len(s) % 4))
    count = 0
    for name in ['signing-v1.json', 'signing-python-v1.json']:
        fixture = json.loads((ROOT / 'tests/fixtures' / name).read_text())
        key = Ed25519PublicKey.from_public_bytes(decode(fixture['public_key']))
        for case in fixture['cases']:
            envelope = case['envelope']
            try:
                key.verify(decode(envelope['signature']), PREFIX + decode(envelope['payload']))
                valid = True
            except InvalidSignature:
                valid = False
            if valid != (case['expected'] != 'BUNDLE_SIGNATURE'):
                raise ValueError('fixture signature mismatch: ' + case['name'])
            count += 1
    print('Python cryptography==' + crypto_version + ' verified ' + str(count) + ' public signatures/negative vectors')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--generate-public-fixture', action='store_true')
    mode.add_argument('--verify-public-fixtures', action='store_true')
    args = parser.parse_args()
    if args.generate_public_fixture:
        generate()
    else:
        verify_public_fixtures()
