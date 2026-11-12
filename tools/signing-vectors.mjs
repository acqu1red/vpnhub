// Independent signer using Node's OpenSSL-backed Ed25519, not the Rust verifier.
// Private keys are generated in memory and never saved/exported to disk.
import { generateKeyPairSync, sign, verify, createPublicKey } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const path = fileURLToPath(new URL('../tests/fixtures/signing-v1.json', import.meta.url));
const prefix = Buffer.from('scrumevpn/bundle/v1\0');
const encode = bytes => Buffer.from(bytes).toString('base64url');
const mode = process.argv[2];
if (mode === 'generate') {
  const { publicKey, privateKey } = generateKeyPairSync('ed25519');
  const jwk = publicKey.export({ format: 'jwk' });
  const now = 1805000000;
  const enrollment_id = '4c5acb88-49fc-4c98-9c56-22818442bfbb';
  const key_id = 'fixture-only-1';
  const payload = {
    schema_version: 1, context: 'scrumevpn/bundle/v1', key_id, enrollment_id,
    version: 7, issued_at: now - 60, expires_at: now + 3600,
    profile: {
      server: 'vpn.example.invalid', port: 8443,
      uuid: '836de22e-adbc-4f56-83c1-e98a81649345',
      sni: 'reality.example.invalid', public_key: encode(Buffer.alloc(32, 17)),
      short_id: 'aabbccdd', fingerprint: 'chrome', mtu: 1400,
      dns: ['1.1.1.1', '2606:4700:4700::1111'], exclude_cidrs: ['192.0.2.0/24'], exclude_domains: [],
    },
    policy: { auto_connect: true, kill_switch: false, refresh_interval_seconds: 21600 },
  };
  function envelope(raw) {
    const bytes = Buffer.from(raw);
    return { key_id, payload: encode(bytes), signature: encode(sign(null, Buffer.concat([prefix, bytes]), privateKey)) };
  }
  const cases = [];
  function add(name, mutate = () => {}, expected = 'ok') {
    const p = structuredClone(payload); mutate(p);
    cases.push({ name, expected, envelope: envelope(JSON.stringify(p)) });
  }
  add('valid');
  add('auto_connect_disabled', p => p.policy.auto_connect = false);
  add('same_version_different_valid', p => p.profile.port = 9443);
  add('wrong_context', p => p.context = 'another-product', 'BUNDLE_BINDING');
  add('wrong_device', p => p.enrollment_id = '7308298b-e3be-42db-829a-a50f20b38d52', 'BUNDLE_BINDING');
  add('wrong_key_binding', p => p.key_id = 'other', 'BUNDLE_BINDING');
  add('wrong_schema_version', p => p.schema_version = 2, 'BUNDLE_BINDING');
  add('expired', p => p.expires_at = now, 'BUNDLE_EXPIRED_OR_INVALID_LEASE');
  add('future_issue', p => p.issued_at = now + 121, 'BUNDLE_CLOCK');
  add('excessive_lease', p => p.expires_at = p.issued_at + 86401, 'BUNDLE_EXPIRED_OR_INVALID_LEASE');
  add('backwards_lease', p => p.expires_at = p.issued_at, 'BUNDLE_EXPIRED_OR_INVALID_LEASE');
  add('zero_version', p => p.version = 0, 'BUNDLE_ROLLBACK');
  add('zero_port', p => p.profile.port = 0, 'PROFILE_INVALID');
  add('nil_uuid', p => p.profile.uuid = '00000000-0000-0000-0000-000000000000', 'PROFILE_INVALID');
  add('invalid_server', p => p.profile.server = 'evil.invalid/--exec', 'PROFILE_INVALID');
  add('invalid_sni', p => p.profile.sni = 'x\n--config', 'PROFILE_INVALID');
  add('loopback_server', p => p.profile.server = '127.0.0.1', 'PROFILE_INVALID');
  add('invalid_mtu', p => p.profile.mtu = 9000, 'PROFILE_INVALID');
  add('no_dns', p => p.profile.dns = [], 'PROFILE_INVALID');
  add('loopback_dns', p => p.profile.dns = ['127.0.0.1'], 'PROFILE_INVALID');
  add('invalid_public_key', p => p.profile.public_key = 'bad', 'PROFILE_INVALID');
  add('invalid_short_id', p => p.profile.short_id = 'xyz', 'PROFILE_INVALID');
  add('unsupported_domains', p => p.profile.exclude_domains = ['example.invalid'], 'PROFILE_INVALID');
  add('unsupported_kill_switch', p => p.policy.kill_switch = true, 'POLICY_UNSUPPORTED');
  add('invalid_refresh', p => p.policy.refresh_interval_seconds = 0, 'PROFILE_INVALID');
  add('unknown_field', p => p.command = 'arbitrary', 'BUNDLE_SCHEMA');
  add('unknown_profile_field', p => p.profile.path = '/tmp/untrusted', 'BUNDLE_SCHEMA');
  add('unknown_policy_field', p => p.policy.exec = 'no', 'BUNDLE_SCHEMA');
  add('fractional_timestamp', p => p.issued_at = now - 0.5, 'BUNDLE_SCHEMA');
  add('unsupported_fingerprint', p => p.profile.fingerprint = 'untrusted', 'BUNDLE_SCHEMA');
  const raw = JSON.stringify(payload);
  cases.push({ name: 'duplicate_field', expected: 'BUNDLE_SCHEMA', envelope: envelope(raw.replace('"version":7', '"version":7,"version":8')) });
  const tampered = envelope(raw);
  tampered.payload = encode(Buffer.from(raw.replace('"port":8443', '"port":8444')));
  cases.push({ name: 'tampered_port', expected: 'BUNDLE_SIGNATURE', envelope: tampered });
  const expiry = envelope(raw);
  expiry.payload = encode(Buffer.from(raw.replace(`"expires_at":${now + 3600}`, `"expires_at":${now + 7200}`)));
  cases.push({ name: 'tampered_expiry', expected: 'BUNDLE_SIGNATURE', envelope: expiry });
  const wrongSigner = generateKeyPairSync('ed25519').privateKey;
  const wrong = envelope(raw);
  wrong.signature = encode(sign(null, Buffer.concat([prefix, Buffer.from(raw)]), wrongSigner));
  cases.push({ name: 'wrong_signing_key', expected: 'BUNDLE_SIGNATURE', envelope: wrong });
  writeFileSync(path, JSON.stringify({ description: 'PUBLIC TEST DATA ONLY; no private key or production credentials', key_id, public_key: jwk.x, enrollment_id, now, cases }, null, 2) + '\n');
  console.log(`Generated ${cases.length} public test vectors; private key discarded`);
} else if (mode === 'verify') {
  const fixture = JSON.parse(readFileSync(path, 'utf8'));
  const key = createPublicKey({ key: { kty: 'OKP', crv: 'Ed25519', x: fixture.public_key }, format: 'jwk' });
  for (const c of fixture.cases) {
    const valid = verify(null, Buffer.concat([prefix, Buffer.from(c.envelope.payload, 'base64url')]), key, Buffer.from(c.envelope.signature, 'base64url'));
    if (valid !== (c.expected !== 'BUNDLE_SIGNATURE')) throw new Error(`signature mismatch ${c.name}`);
  }
  console.log(`Node verified signatures of ${fixture.cases.length} public vectors`);
} else {
  throw new Error('usage: node tools/signing-vectors.mjs generate|verify');
}
