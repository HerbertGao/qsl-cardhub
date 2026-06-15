/**
 * 防爬 fail 策略矩阵强锚（worker 集成层，mock KV）：
 * - 反滥用关键键（seed/session/quota/nonce）KV 未绑 / 运行时读写失败 → fail-closed（503/401），禁 fail-open 放行
 * - 握手桶 ratelimit:session 运行时失败 → 抛错（调用方 503），禁继承查询桶 fail-open
 * - powrate 读写失败 → difficulty 取 DIFF_MAX（fail-secure），禁跌回 baseMin
 * - 完整有效会话查询 → ok；配额用尽 → 429
 */
import { test } from 'node:test';
import assert from 'node:assert/strict';

import {
  validateQuerySession,
  computeDifficulty,
  handshakeRateLimit,
  checkRateLimit,
} from '../src/worker/index.js';
import { makeToken, hmacSha256Hex, uaHash, DIFFICULTY_DEFAULTS } from '../src/worker/session.js';
import { buildCanonicalPayload } from '../src/worker/canonical.js';

const UA = 'Mozilla/5.0 (test)';

function mockKV() {
  const m = new Map();
  return {
    store: m,
    async get(k, opts) {
      const v = m.get(k);
      if (v === undefined) return null;
      return opts && opts.type === 'json' ? JSON.parse(v) : v;
    },
    async put(k, v) { m.set(k, typeof v === 'string' ? v : JSON.stringify(v)); },
    async delete(k) { m.delete(k); },
  };
}
const throwingKV = {
  async get() { throw new Error('kv down'); },
  async put() { throw new Error('kv down'); },
  async delete() { throw new Error('kv down'); },
};

function mockRequest(ua = UA) {
  return { headers: { get: (h) => (String(h).toLowerCase() === 'user-agent' ? ua : null) } };
}

// 构造一个完整有效的查询 URL（模拟客户端签名）
async function buildSignedUrl({ kv, secret, sid, sk, bkey, callsign = 'BG7XYZ', ts, nonce = 'noncexyz' }) {
  const now = ts ?? Date.now();
  const token = await makeToken(sid, secret);
  // 写会话
  await kv.put(`session:${sid}`, JSON.stringify({
    binding_mode: 'ip', ip: bkey, ua_hash: await uaHash(UA), exp: now + 60000, sk,
  }));
  const url = new URL(`https://x/api/query?callsign=${callsign}&token=${encodeURIComponent(token)}&_ts=${now}&_nonce=${nonce}`);
  const payload = buildCanonicalPayload({ sid, path: url.pathname, params: url.searchParams, ts: String(now), nonce });
  const sig = await hmacSha256Hex(sk, payload);
  url.searchParams.set('_sig', sig);
  return url;
}

// ---------- fail-closed：KV 未绑 / SESSION_SECRET 缺 ----------
test('validateQuerySession: KV 未绑 → 503', async () => {
  const r = await validateQuerySession({ SESSION_SECRET: 's' }, mockRequest(), new URL('https://x/api/query?callsign=A'), '1.2.3.4');
  assert.equal(r.ok, false);
  assert.equal(r.status, 503);
});
test('validateQuerySession: SESSION_SECRET 缺 → 503', async () => {
  const r = await validateQuerySession({ RATE_LIMIT: mockKV() }, mockRequest(), new URL('https://x/api/query?callsign=A'), '1.2.3.4');
  assert.equal(r.status, 503);
});

// ---------- 缺凭据 / 过期 ----------
test('validateQuerySession: 缺会话凭据 → 401', async () => {
  const env = { RATE_LIMIT: mockKV(), SESSION_SECRET: 's' };
  const r = await validateQuerySession(env, mockRequest(), new URL('https://x/api/query?callsign=A'), '1.2.3.4');
  assert.equal(r.status, 401);
});
test('validateQuerySession: _ts 超时窗 → 403', async () => {
  const env = { RATE_LIMIT: mockKV(), SESSION_SECRET: 's' };
  const stale = Date.now() - 10 * 60 * 1000;
  const url = new URL(`https://x/api/query?callsign=A&token=a.b&_ts=${stale}&_nonce=noncevalid1&_sig=x`);
  const r = await validateQuerySession(env, mockRequest(), url, '1.2.3.4');
  assert.equal(r.status, 403);
});

// ---------- 运行时 KV 失败 → fail-closed 503（禁 fail-open / 禁冒泡 500）----------
test('validateQuerySession: KV 运行时抛错 → fail-closed 503（非放行、非 500 冒泡）', async () => {
  // 用合法 token 通过 parseToken（不碰 KV），随后 session KV 读抛错 → catch → 503
  const secret = 'session-secret';
  const token = await makeToken('abcdef0123', secret);
  const env = { RATE_LIMIT: throwingKV, SESSION_SECRET: secret };
  const url = new URL(`https://x/api/query?callsign=A&token=${encodeURIComponent(token)}&_ts=${Date.now()}&_nonce=noncevalid1&_sig=x`);
  const r = await validateQuerySession(env, mockRequest(), url, '1.2.3.4');
  assert.equal(r.ok, false);
  assert.equal(r.status, 503); // 非 200/非放行；catch 内部，不冒泡成顶层 500
});

// ---------- 完整有效会话 → ok；配额用尽 → 429 ----------
test('validateQuerySession: 完整有效会话查询 → ok', async () => {
  const kv = mockKV();
  const secret = 'session-secret';
  const sid = 'abcdef0123';
  const sk = 'a'.repeat(64);
  const bkey = '1.2.3.4';
  const url = await buildSignedUrl({ kv, secret, sid, sk, bkey });
  const r = await validateQuerySession({ RATE_LIMIT: kv, SESSION_SECRET: secret }, mockRequest(), url, bkey);
  assert.equal(r.ok, true);
});
test('validateQuerySession: 伪造签名（错 sk）→ 401', async () => {
  const kv = mockKV();
  const secret = 'session-secret';
  const sid = 'abcdef0123';
  const url = await buildSignedUrl({ kv, secret, sid, sk: 'a'.repeat(64), bkey: '1.2.3.4' });
  url.searchParams.set('_sig', 'deadbeef'); // 篡改签名
  const r = await validateQuerySession({ RATE_LIMIT: kv, SESSION_SECRET: secret }, mockRequest(), url, '1.2.3.4');
  assert.equal(r.status, 401);
});
test('validateQuerySession: 配额用尽 → 429', async () => {
  const kv = mockKV();
  const secret = 'session-secret';
  const sid = 'quota0123';
  const sk = 'b'.repeat(64);
  const bkey = '5.6.7.8';
  // 预置配额已满
  await kv.put(`sessionq:${sid}`, JSON.stringify({ count: 50 }));
  const url = await buildSignedUrl({ kv, secret, sid, sk, bkey, callsign: 'BD1XYZ' });
  // 需用新 nonce 避免与上一个测试冲突（独立 kv，无冲突）
  const r = await validateQuerySession({ RATE_LIMIT: kv, SESSION_SECRET: secret }, mockRequest(), url, bkey);
  assert.equal(r.status, 429);
});

// ---------- 握手桶 fail-closed（运行时抛错 → 抛，调用方 503）----------
test('handshakeRateLimit: KV 运行时抛错 → 抛错（调用方 fail-closed 503，不继承查询桶 fail-open）', async () => {
  await assert.rejects(() => handshakeRateLimit({ RATE_LIMIT: throwingKV }, '1.2.3.4'));
});
test('handshakeRateLimit: 正常计数、超限 allowed=false', async () => {
  const kv = mockKV();
  const env = { RATE_LIMIT: kv };
  let last;
  for (let i = 0; i < 35; i++) last = await handshakeRateLimit(env, '9.9.9.9');
  assert.equal(last.allowed, false); // 超 HANDSHAKE_RATE_MAX(30)
});

// ---------- 查询桶 checkRateLimit fail-open（可用性优先）----------
test('checkRateLimit: KV 运行时抛错 → fail-open（查询桶可用性优先，不阻断、由会话校验主导）', async () => {
  const r = await checkRateLimit({ RATE_LIMIT: throwingKV }, '1.2.3.4');
  assert.equal(r.allowed, true);
});
test('checkRateLimit: KV 未绑 → fail-open', async () => {
  const r = await checkRateLimit({}, '1.2.3.4');
  assert.equal(r.allowed, true);
});
test('checkRateLimit: 正常计数、超 RATE_LIMIT_MAX → allowed=false', async () => {
  const kv = mockKV();
  let last;
  for (let i = 0; i < 25; i++) last = await checkRateLimit({ RATE_LIMIT: kv }, '2.2.2.2');
  assert.equal(last.allowed, false); // 超 20/min
});

// ---------- _nonce 字符集 ----------
test('validateQuerySession: _nonce 非法字符集 → 401（防 nonce KV 键注入/超长）', async () => {
  const secret = 'session-secret';
  const token = await makeToken('abcdef0123', secret);
  const env = { RATE_LIMIT: mockKV(), SESSION_SECRET: secret };
  const url = new URL(`https://x/api/query?callsign=A&token=${encodeURIComponent(token)}&_ts=${Date.now()}&_nonce=bad!nonce&_sig=x`);
  const r = await validateQuerySession(env, mockRequest(), url, '1.2.3.4');
  assert.equal(r.status, 401);
  // 钉死是「字符集闸」触发（而非下游 session-miss 也返 401）——否则删掉字符集闸该测仍假绿
  assert.equal(r.message, '会话凭据格式无效');
});

// ---------- powrate fail-secure → DIFF_MAX ----------
test('computeDifficulty: powrate 读写失败 → DIFF_MAX（fail-secure，禁跌回 baseMin）', async () => {
  const d = await computeDifficulty({ RATE_LIMIT: throwingKV }, '1.2.3.4');
  assert.equal(d, DIFFICULTY_DEFAULTS.diffMax);
  assert.ok(d > DIFFICULTY_DEFAULTS.baseMin);
});
test('computeDifficulty: unknown → DIFF_MAX', async () => {
  const d = await computeDifficulty({ RATE_LIMIT: mockKV() }, 'unknown');
  assert.equal(d, DIFFICULTY_DEFAULTS.diffMax);
});
test('computeDifficulty: 正常低频 → base', async () => {
  const d = await computeDifficulty({ RATE_LIMIT: mockKV() }, '1.2.3.4');
  assert.equal(d, DIFFICULTY_DEFAULTS.base);
});
