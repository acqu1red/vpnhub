// LOCAL TEST SERVER ONLY. No SMTP, production access, persistence, or real Xray API.
import { createServer } from 'node:http';
import { generateKeyPairSync, randomUUID, randomBytes, sign, createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';

export function createMockIssuer({ now = () => Math.floor(Date.now() / 1000), employees = ['worker@example.invalid'] } = {}) {
  const { publicKey, privateKey } = generateKeyPairSync('ed25519');
  const keyId = 'mock-ephemeral-only';
  const publicKeyRaw = publicKey.export({ format: 'jwk' }).x;
  const challenges = new Map();
  const lastRequests = new Map();
  const receipts = new Map();
  const enrollments = new Map();
  const allowed = new Set(employees);
  const fixedTestOtp = '000000'; // Explicit public test code; no real mail is sent.
  const prefix = Buffer.from('scrumevpn/bundle/v1\0');
  const policy = { auto_connect: true, kill_switch: false, refresh_interval_seconds: 21600 };

  function reply(res, status, data, headers = {}) {
    res.writeHead(status, { 'content-type': 'application/json', 'cache-control': 'no-store', ...headers });
    res.end(JSON.stringify(data));
  }
  const error = (res, status, code, headers) => reply(res, status, { error: { code } }, headers);
  function publish(enrollment) {
    const payload = {
      schema_version: 1, context: 'scrumevpn/bundle/v1', key_id: keyId,
      enrollment_id: enrollment.id, version: enrollment.version, issued_at: now(), expires_at: now() + 86400,
      profile: {
        server: 'vpn.example.invalid', port: 8443, uuid: enrollment.uuid,
        sni: 'reality.example.invalid', public_key: Buffer.alloc(32, 17).toString('base64url'),
        short_id: 'aabbccdd', fingerprint: 'chrome', mtu: 1400,
        dns: ['1.1.1.1'], exclude_cidrs: [], exclude_domains: [],
      }, policy: { ...policy },
    };
    const bytes = Buffer.from(JSON.stringify(payload));
    enrollment.envelope = { key_id: keyId, payload: bytes.toString('base64url'),
      signature: sign(null, Buffer.concat([prefix, bytes]), privateKey).toString('base64url') };
  }

  async function handle(req, res) {
    const time = now();
    // Receipts contain mock tokens; prune their bounded lifetime.
    for (const [id, receipt] of receipts) if (receipt.expires <= time) receipts.delete(id);
    for (const [email, challenge] of challenges) if (challenge.expires <= time) challenges.delete(email);
    for (const [email, created] of lastRequests) if (time - created >= 60) lastRequests.delete(email);
    if (req.method === 'GET' && req.url === '/vpn/v1/bundle') {
      const token = req.headers['x-install-token'];
      const enrollment = typeof token === 'string' ? enrollments.get(token) : null;
      if (!enrollment || enrollment.revoked) return error(res, 401, 'ENROLLMENT_INVALID');
      const current = JSON.parse(Buffer.from(enrollment.envelope.payload, 'base64url'));
      if (current.expires_at <= time + 60) { enrollment.version++; publish(enrollment); }
      return reply(res, 200, enrollment.envelope);
    }
    if (req.method !== 'POST' || !['/vpn/v1/auth/request', '/vpn/v1/auth/verify'].includes(req.url)) {
      return error(res, 404, 'NOT_FOUND');
    }
    if (!/^application\/json(?:;|$)/i.test(req.headers['content-type'] ?? '')) return error(res, 415, 'JSON_REQUIRED');
    const chunks = []; let length = 0;
    for await (const chunk of req) {
      length += chunk.length;
      if (length > 4096) return error(res, 413, 'REQUEST_TOO_LARGE');
      chunks.push(chunk);
    }
    let body;
    try { body = JSON.parse(Buffer.concat(chunks).toString('utf8')); }
    catch { return error(res, 400, 'INVALID_JSON'); }
    if (!body || typeof body !== 'object' || Array.isArray(body) || typeof body.email !== 'string'
        || body.email.length > 254 || !/^[^\s@]+@[^\s@]+\.invalid$/.test(body.email)) {
      return error(res, 400, 'TEST_EMAIL_REQUIRED');
    }
    // Only example.invalid-style addresses accepted; cannot send actual mail.
    const email = body.email.toLowerCase();
    if (req.url.endsWith('/request')) {
      if (Object.keys(body).some(k => k !== 'email')) return error(res, 400, 'UNKNOWN_FIELD');
      const previous = lastRequests.get(email);
      if (previous !== undefined && time - previous < 60) return error(res, 429, 'RESEND_LIMIT', { 'retry-after': '60' });
      lastRequests.set(email, time);
      challenges.set(email, { created: time, expires: time + 600, attempts: 0, eligible: allowed.has(email) });
      return reply(res, 202, { ok: true, retry_after: 60 });
    }
    if (Object.keys(body).some(k => !['email', 'code', 'enrollment_request_id'].includes(k))
        || typeof body.code !== 'string' || !/^\d{6}$/.test(body.code)
        || typeof body.enrollment_request_id !== 'string'
        || !/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(body.enrollment_request_id)) {
      return error(res, 400, 'INVALID_VERIFY_REQUEST');
    }
    const binding = createHash('sha256').update(email + '\0' + body.code).digest('hex');
    const cached = receipts.get(body.enrollment_request_id);
    if (cached) {
      if (cached.binding !== binding) return error(res, 409, 'IDEMPOTENCY_CONFLICT');
      if (enrollments.get(cached.result.install_token)?.revoked) return error(res, 401, 'ENROLLMENT_INVALID');
      return reply(res, 200, cached.result);
    }
    const challenge = challenges.get(email);
    if (!challenge) return error(res, 401, 'OTP_INVALID');
    if (challenge.attempts >= 5) return error(res, 429, 'ATTEMPTS_EXHAUSTED');
    challenge.attempts++;
    if (body.code !== fixedTestOtp || !challenge.eligible) return error(res, 401, 'OTP_INVALID');
    const enrollment = { id: randomUUID(), uuid: randomUUID(), version: 1, revoked: false };
    const token = 'mock-install-' + randomBytes(32).toString('base64url');
    publish(enrollment);
    enrollments.set(token, enrollment);
    challenges.delete(email);
    const result = { install_token: token, enrollment_id: enrollment.id };
    receipts.set(body.enrollment_request_id, { binding, result, expires: time + 300 });
    return reply(res, 200, result);
  }

  const server = createServer((req, res) => {
    handle(req, res).catch(() => { if (!res.headersSent) error(res, 500, 'MOCK_INTERNAL'); else res.end(); });
  });
  server.requestTimeout = 10000;
  server.headersTimeout = 10000;
  server.maxHeadersCount = 32;
  return {
    server,
    trustRoot: { key_id: keyId, public_key: publicKeyRaw },
    // Test harness only. There is deliberately no HTTP admin/revoke endpoint.
    revoke(id) { for (const e of enrollments.values()) if (e.id === id) e.revoked = true; },
    setAutoConnect(enabled) {
      if (typeof enabled !== 'boolean') throw new Error('boolean required');
      policy.auto_connect = enabled;
      for (const e of enrollments.values()) { e.version++; publish(e); }
    },
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const mock = createMockIssuer();
  mock.server.listen(4200, '127.0.0.1', () => {
    console.log('MOCK ONLY http://127.0.0.1:4200 ; worker@example.invalid ; public test OTP 000000');
    console.log('Ephemeral public trust root (never use dynamic trust in production):', JSON.stringify(mock.trustRoot));
  });
}
