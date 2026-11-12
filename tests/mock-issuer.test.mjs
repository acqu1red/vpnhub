import { test } from 'node:test';
import assert from 'node:assert/strict';
import { randomUUID, createPublicKey, verify } from 'node:crypto';
import { createMockIssuer } from '../tools/mock-issuer.mjs';

async function lab(t) {
  let time = 1805000000;
  const mock = createMockIssuer({ now: () => time });
  await new Promise(resolve => mock.server.listen(0, '127.0.0.1', resolve));
  t.after(() => new Promise(resolve => { mock.server.closeAllConnections(); mock.server.close(resolve); }));
  const base = `http://127.0.0.1:${mock.server.address().port}`;
  async function post(path, body) {
    const r = await fetch(base + '/vpn/v1/auth/' + path, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(body) });
    return { status: r.status, headers: r.headers, body: await r.json() };
  }
  async function enroll(email = 'worker@example.invalid') {
    assert.equal((await post('request', { email })).status, 202);
    const result = await post('verify', { email, code: '000000', enrollment_request_id: randomUUID() });
    assert.equal(result.status, 200); return result.body;
  }
  async function bundle(token) {
    const r = await fetch(base + '/vpn/v1/bundle', { headers: { 'x-install-token': token } });
    return { status: r.status, body: await r.json() };
  }
  return { ...mock, post, enroll, bundle, base, advance: seconds => time += seconds };
}

test('enrollment returns no third-party API key and a verified device-bound bundle', async t => {
  const l = await lab(t); const e = await l.enroll();
  assert.deepEqual(Object.keys(e).sort(), ['enrollment_id', 'install_token']);
  assert.match(e.install_token, /^mock-install-/);
  const b = await l.bundle(e.install_token); assert.equal(b.status, 200);
  const key = createPublicKey({ key: { kty: 'OKP', crv: 'Ed25519', x: l.trustRoot.public_key }, format: 'jwk' });
  assert.ok(verify(null, Buffer.concat([Buffer.from('scrumevpn/bundle/v1\0'), Buffer.from(b.body.payload, 'base64url')]), key, Buffer.from(b.body.signature, 'base64url')));
  assert.equal(JSON.parse(Buffer.from(b.body.payload, 'base64url')).enrollment_id, e.enrollment_id);
  assert.deepEqual((await l.bundle(e.install_token)).body, b.body);
});

test('OTP resend, expiry and attempt limits are server-enforced', async t => {
  const l = await lab(t); const email = 'worker@example.invalid';
  assert.equal((await l.post('request', { email })).status, 202);
  assert.equal((await l.post('request', { email })).status, 429);
  for (let i = 0; i < 5; i++) assert.equal((await l.post('verify', { email, code: '999999', enrollment_request_id: randomUUID() })).status, 401);
  assert.equal((await l.post('verify', { email, code: '000000', enrollment_request_id: randomUUID() })).status, 429);
  l.advance(601);
  assert.equal((await l.post('verify', { email, code: '000000', enrollment_request_id: randomUUID() })).status, 401);
});

test('unknown employee does not enroll or leak existence in request response', async t => {
  const l = await lab(t);
  const known = await l.post('request', { email: 'worker@example.invalid' });
  const unknown = await l.post('request', { email: 'unknown@example.invalid' });
  assert.equal(unknown.status, known.status); assert.deepEqual(unknown.body, known.body);
  assert.equal((await l.post('verify', { email: 'unknown@example.invalid', code: '000000', enrollment_request_id: randomUUID() })).status, 401);
});

test('verify retries are idempotent and cannot switch identity', async t => {
  const l = await lab(t); const email = 'worker@example.invalid';
  await l.post('request', { email });
  const body = { email, code: '000000', enrollment_request_id: randomUUID() };
  const first = await l.post('verify', body); const retry = await l.post('verify', body);
  assert.equal(retry.status, 200); assert.deepEqual(retry.body, first.body);
  assert.equal((await l.post('verify', { ...body, email: 'another@example.invalid' })).status, 409);
});

test('two devices get distinct identities and revoke is isolated', async t => {
  const l = await lab(t); const a = await l.enroll(); l.advance(60); const b = await l.enroll();
  assert.notEqual(a.enrollment_id, b.enrollment_id);
  const pa = JSON.parse(Buffer.from((await l.bundle(a.install_token)).body.payload, 'base64url'));
  const pb = JSON.parse(Buffer.from((await l.bundle(b.install_token)).body.payload, 'base64url'));
  assert.notEqual(pa.profile.uuid, pb.profile.uuid);
  l.revoke(a.enrollment_id);
  assert.equal((await l.bundle(a.install_token)).status, 401);
  assert.equal((await l.bundle(b.install_token)).status, 200);
});

test('admin policy publishes a higher signed version with boot disabled', async t => {
  const l = await lab(t); const e = await l.enroll();
  const old = JSON.parse(Buffer.from((await l.bundle(e.install_token)).body.payload, 'base64url'));
  l.setAutoConnect(false);
  const next = JSON.parse(Buffer.from((await l.bundle(e.install_token)).body.payload, 'base64url'));
  assert.equal(next.policy.auto_connect, false); assert.ok(next.version > old.version);
});

test('unknown tokens and real-domain email are rejected', async t => {
  const l = await lab(t);
  assert.equal((await l.bundle('unknown')).status, 401);
  assert.equal((await l.post('request', { email: 'worker@example.com' })).status, 400);
  assert.equal((await l.post('request', { email: 'worker@example.invalid', exec: 'bad' })).status, 400);
});

test('successful verify does not reset resend limiter and leases renew with higher version', async t => {
  const l = await lab(t); const e = await l.enroll();
  assert.equal((await l.post('request', { email: 'worker@example.invalid' })).status, 429);
  const old = JSON.parse(Buffer.from((await l.bundle(e.install_token)).body.payload, 'base64url'));
  l.advance(86400);
  const next = JSON.parse(Buffer.from((await l.bundle(e.install_token)).body.payload, 'base64url'));
  assert.ok(next.version > old.version); assert.ok(next.expires_at > old.expires_at);
});
