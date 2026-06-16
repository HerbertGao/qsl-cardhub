/**
 * 客户端会话状态机强锚：singleflight 握手、不可变快照、401/429 仅使匹配旧会话失效后至多重试一次、
 * 无会话先握手不裸发。注入 mock 依赖（无浏览器/网络）。
 */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createSessionManager, parseSid } from '../src/worker/session-client.js';

function mkDeps(over = {}) {
  let handshakes = 0;
  let powCalls = 0;
  let tokenSeq = 0;
  let challenges = 0;
  const deps = {
    handshakes: () => handshakes,
    powCalls: () => powCalls,
    challenges: () => challenges,
    getChallenge: async () => { challenges++; return { seed: 'seed' + challenges, difficulty: 4 }; },
    solvePow: async () => { powCalls++; return 'nonce'; },
    postSession: async () => { handshakes++; tokenSeq++; return { token: `sid${tokenSeq}.mac`, sk: 'sk' + tokenSeq, exp: 1_000_000, quota: 50 }; },
    signQuery: async (path, _params, snap) => `/q?token=${snap.token}`,
    doFetch: async () => ({ status: 200, json: async () => ({ ok: true }) }),
    now: () => 0,
    backoff: async () => {},
    ...over,
  };
  return deps;
}

test('parseSid: token=sid.mac → sid', () => {
  assert.equal(parseSid('abc123.deadbeef'), 'abc123');
  assert.equal(parseSid('nodot'), '');
});

test('singleflight: 并发 getSession 只握手一次', async () => {
  const deps = mkDeps();
  const mgr = createSessionManager(deps);
  const [a, b, c] = await Promise.all([mgr.getSession(), mgr.getSession(), mgr.getSession()]);
  assert.equal(deps.handshakes(), 1, '并发应只握手一次');
  assert.equal(deps.powCalls(), 1, '并发应只算一次 PoW');
  assert.equal(a.token, b.token);
  assert.equal(b.token, c.token);
});

test('snapshot: 会话有效时复用快照、不重复握手', async () => {
  const deps = mkDeps();
  const mgr = createSessionManager(deps);
  await mgr.getSession();
  await mgr.getSession();
  assert.equal(deps.handshakes(), 1, '有效快照应复用');
});

test('snapshot: 快照不可变（frozen）', async () => {
  const mgr = createSessionManager(mkDeps());
  const s = await mgr.getSession();
  assert.throws(() => { s.token = 'tampered'; }, TypeError);
});

test('requestQuery: 无会话先握手再发（不裸发）', async () => {
  const urls = [];
  const deps = mkDeps({ doFetch: async (url) => { urls.push(url); return { status: 200 }; } });
  const mgr = createSessionManager(deps);
  const res = await mgr.requestQuery('/api/query', { callsign: 'BG7XYZ' });
  assert.equal(res.status, 200);
  assert.equal(deps.handshakes(), 1);
  assert.ok(urls[0].includes('token='), '查询 URL 必带 token（先握手）');
});

test('requestQuery: 401 → 重走握手并重试一次后成功', async () => {
  let n = 0;
  const deps = mkDeps({
    doFetch: async () => { n++; return n === 1 ? { status: 401 } : { status: 200 }; },
  });
  const mgr = createSessionManager(deps);
  const res = await mgr.requestQuery('/api/query', {});
  assert.equal(res.status, 200);
  assert.equal(res.retried, true);
  assert.equal(deps.handshakes(), 2, '401 后应重新握手一次');
});

test('requestQuery: 持续 401 → 至多重试一次后返回 401（不无限重试）', async () => {
  const deps = mkDeps({ doFetch: async () => ({ status: 401 }) });
  const mgr = createSessionManager(deps);
  const res = await mgr.requestQuery('/api/query', {});
  assert.equal(res.status, 401);
  assert.equal(deps.handshakes(), 2, '至多两次握手（首次+重试一次）');
});

test('requestQuery: 配额 429（无 retry_after）→ 重握手重试一次', async () => {
  let n = 0;
  // 配额用尽 429 体无 retry_after → 应重握手（新会话=新配额）
  const deps = mkDeps({ doFetch: async () => { n++; return n === 1 ? { status: 429, json: async () => ({ success: false, message: '会话配额已用尽' }) } : { status: 200, json: async () => ({ ok: true }) }; } });
  const mgr = createSessionManager(deps);
  const res = await mgr.requestQuery('/api/query', {});
  assert.equal(res.status, 200);
  assert.equal(deps.handshakes(), 2, '配额 429 应重握手');
});

test('requestQuery: 限流 429（带 retry_after）→ 不重握手（退避，避免白算 PoW）', async () => {
  // Layer0 IP 限流 429 体带 retry_after → 重握手无益（IP 仍受限）→ 不重握手、直接返回
  const deps = mkDeps({ doFetch: async () => ({ status: 429, json: async () => ({ success: false, message: '请求过于频繁，请稍后再试', retry_after: 30 }) }) });
  const mgr = createSessionManager(deps);
  const res = await mgr.requestQuery('/api/query', {});
  assert.equal(res.status, 429);
  assert.equal(deps.handshakes(), 1, '限流 429 不应重握手（仅 1 次握手）');
  assert.equal(deps.powCalls(), 1, '限流 429 不应多算 PoW');
});

test('handshake: seed-not-found → 短退避重取 challenge 一次后成功', async () => {
  let n = 0;
  const deps = mkDeps({
    postSession: async () => {
      n++;
      if (n === 1) { const e = new Error('题目无效或已过期'); e.seedNotFound = true; throw e; }
      return { token: `sid${n}.mac`, sk: 'sk' + n, exp: 1_000_000, quota: 50 };
    },
  });
  const mgr = createSessionManager(deps);
  const s = await mgr.getSession();
  assert.ok(s && s.token, '重取 challenge 后应成功签发');
  assert.equal(deps.challenges(), 2, 'seed-not-found 应重取 challenge 一次（共 2 次）');
});

test('handshake: 非 seed-not-found 失败（如 PoW 不足）不重试 → 抛出', async () => {
  const deps = mkDeps({ postSession: async () => { throw new Error('工作量证明不足'); } });
  const mgr = createSessionManager(deps);
  await assert.rejects(() => mgr.getSession(), /工作量证明不足/);
  assert.equal(deps.challenges(), 1, '非 seed-not-found 不应重取 challenge');
});

test('invalidate: 仅使匹配快照失效，不误伤新会话', async () => {
  const deps = mkDeps();
  const mgr = createSessionManager(deps);
  const s1 = await mgr.getSession();
  // 用一个陈旧快照 invalidate，但当前会话已是 s1 → 匹配则失效
  mgr.invalidate(s1);
  assert.equal(mgr._peek(), null, '匹配快照应失效');
  const s2 = await mgr.getSession(); // 重新握手得新会话
  mgr.invalidate(s1); // 用旧 s1 再 invalidate，不应误伤 s2
  assert.equal(mgr._peek(), s2, '不匹配快照不应误伤新会话');
});
