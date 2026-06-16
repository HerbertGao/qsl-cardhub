/**
 * tenant-path-routing 单测（node:test）：`/t/<slug>/` 前缀文法、端点三分类、默认租户配置化、
 * 会话签名双 path 不变量（D4）、每租户 /api/config 形状、微信订阅 state 解析。
 *
 * 这些用例即 tenant-path-routing 各场景的可执行不变量检查。被测纯函数导出自 worker：
 * parseTenantPrefix / resolveQueryTenant / defaultTenant（src/worker/index.js）。
 * 配置端点 / 微信 callback 走 worker `fetch` 集成（mock env），仿 cross-check.test.js 的 mock env.DB 模式。
 */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';

import worker, {
  defaultTenant,
  parseTenantPrefix,
  resolveQueryTenant,
  isNonQuerySurface,
} from '../src/worker/index.js';
import { buildCanonicalPayload } from '../src/worker/canonical.js';
import { hmacSha256Hex, verifySessionSig, randomHex, makeToken, uaHash } from '../src/worker/session.js';

// ============================================================
// 8.1 前缀文法解析（parseTenantPrefix）
// ============================================================

test('8.1 parseTenantPrefix: 命中——/t/bh2ro/api/query → slug=bh2ro, routePath=/api/query', () => {
  assert.deepEqual(parseTenantPrefix('/t/bh2ro/api/query'), { slug: 'bh2ro', routePath: '/api/query' });
});

test('8.1 parseTenantPrefix: /t/bh2ro（无尾斜杠）→ slug=bh2ro, routePath=/（缺省）', () => {
  assert.deepEqual(parseTenantPrefix('/t/bh2ro'), { slug: 'bh2ro', routePath: '/' });
});

test('8.1 parseTenantPrefix: /t/bh2ro/（裸外壳）→ slug=bh2ro, routePath=/', () => {
  assert.deepEqual(parseTenantPrefix('/t/bh2ro/'), { slug: 'bh2ro', routePath: '/' });
});

test('8.1 parseTenantPrefix: /t → namespaceInvalid（404，禁 fall-through）', () => {
  assert.deepEqual(parseTenantPrefix('/t'), { namespaceInvalid: true });
});

test('8.1 parseTenantPrefix: /t/ → namespaceInvalid', () => {
  assert.deepEqual(parseTenantPrefix('/t/'), { namespaceInvalid: true });
});

test('8.1 parseTenantPrefix: /t//（空 slug 段）→ namespaceInvalid', () => {
  assert.deepEqual(parseTenantPrefix('/t//'), { namespaceInvalid: true });
  assert.deepEqual(parseTenantPrefix('/t//api/query'), { namespaceInvalid: true });
});

test('8.1 parseTenantPrefix: /tfoo（不命中 ^/t(/|$) 命名空间）→ bare（slug:null, routePath 原样）', () => {
  assert.deepEqual(parseTenantPrefix('/tfoo'), { slug: null, routePath: '/tfoo' });
  assert.deepEqual(parseTenantPrefix('/tfoo/api/query'), { slug: null, routePath: '/tfoo/api/query' });
});

test('8.1 parseTenantPrefix: 大写 slug /t/ABC/ → namespaceInvalid（文法仅 [a-z0-9-]）', () => {
  assert.deepEqual(parseTenantPrefix('/t/ABC/'), { namespaceInvalid: true });
  assert.deepEqual(parseTenantPrefix('/t/Bh2Ro/api/query'), { namespaceInvalid: true });
});

test('8.1 parseTenantPrefix: 非法字符 slug → namespaceInvalid', () => {
  assert.deepEqual(parseTenantPrefix('/t/a_b/api/query'), { namespaceInvalid: true }); // 下划线非法
  assert.deepEqual(parseTenantPrefix('/t/a.b'), { namespaceInvalid: true }); // 点非法
});

test('8.1 parseTenantPrefix: 超 32 长 slug → namespaceInvalid；恰 32 长 → 命中', () => {
  const slug33 = 'a'.repeat(33);
  assert.deepEqual(parseTenantPrefix(`/t/${slug33}/api/query`), { namespaceInvalid: true });
  const slug32 = 'a'.repeat(32);
  assert.deepEqual(parseTenantPrefix(`/t/${slug32}/api/query`), { slug: slug32, routePath: '/api/query' });
});

test('8.1 parseTenantPrefix: bare /api/query（不命中命名空间）→ slug:null, routePath 原样', () => {
  assert.deepEqual(parseTenantPrefix('/api/query'), { slug: null, routePath: '/api/query' });
  assert.deepEqual(parseTenantPrefix('/'), { slug: null, routePath: '/' });
  assert.deepEqual(parseTenantPrefix('/api/config'), { slug: null, routePath: '/api/config' });
});

test('8.1 parseTenantPrefix: 含连字符/数字 slug 命中', () => {
  assert.deepEqual(parseTenantPrefix('/t/bh-2-ro/api/callsigns/BV2ABC'), {
    slug: 'bh-2-ro',
    routePath: '/api/callsigns/BV2ABC',
  });
});

// ============================================================
// 8.2 / 8.3 resolveQueryTenant + defaultTenant + 非查询面判定
// ============================================================

// fake env.DB：tenants 活跃 slug 集合驱动 SELECT；记录 prepare 是否被调用（断言 bare 不读表）。
function makeEnv({ activeTenants = [], names = {}, env: vars = {} } = {}) {
  let prepareCalls = 0;
  const e = {
    ...vars,
    _prepareCalls: () => prepareCalls,
    DB: {
      prepare(sql) {
        prepareCalls += 1;
        const q = { args: [] };
        q.bind = (...a) => { q.args = a; return q; };
        q.first = async () => {
          const slug = q.args[0];
          if (sql.includes('SELECT name FROM tenants')) {
            if (activeTenants.includes(slug)) return { name: names[slug] ?? null };
            return null;
          }
          if (sql.includes('FROM tenants')) {
            return activeTenants.includes(slug) ? { 1: 1 } : null;
          }
          return null;
        };
        q.run = async () => ({ meta: { changes: 1 } });
        q.all = async () => ({ results: [] });
        return q;
      },
    },
  };
  return e;
}

test('8.2/8.3 resolveQueryTenant: 显式活跃 slug → 返 slug', async () => {
  const env = makeEnv({ activeTenants: ['bh2ro'] });
  assert.equal(await resolveQueryTenant(env, 'bh2ro'), 'bh2ro');
});

test('8.2 resolveQueryTenant: 未知 slug → null（调用方 404）', async () => {
  const env = makeEnv({ activeTenants: ['bh2ro'] });
  assert.equal(await resolveQueryTenant(env, 'ghost'), null);
});

test('8.2 resolveQueryTenant: 停用 slug（不在活跃集）→ null', async () => {
  const env = makeEnv({ activeTenants: ['bh2ro'] }); // 'paused' 模拟非活跃：不在集合
  assert.equal(await resolveQueryTenant(env, 'paused'), null);
});

test('8.2/8.3 resolveQueryTenant: bare（slug=null）→ defaultTenant 且不读 tenants 表（DB.prepare 未被调用）', async () => {
  const env = makeEnv({ activeTenants: ['bh2ro'], env: { DEFAULT_TENANT: 'bh2ro' } });
  assert.equal(await resolveQueryTenant(env, null), 'bh2ro');
  assert.equal(env._prepareCalls(), 0, 'bare 路径禁为解析默认租户读 tenants 表');
});

test('8.2/8.3 resolveQueryTenant: bare 默认租户取 DEFAULT_TENANT 配置值（非硬编码）', async () => {
  const env = makeEnv({ env: { DEFAULT_TENANT: 'otherco' } });
  assert.equal(await resolveQueryTenant(env, null), 'otherco');
  assert.equal(env._prepareCalls(), 0);
});

test('8.2/8.3 resolveQueryTenant: 显式 slug 命中时确实读了一次表', async () => {
  const env = makeEnv({ activeTenants: ['bh2ro'] });
  await resolveQueryTenant(env, 'bh2ro');
  assert.equal(env._prepareCalls(), 1);
});

test('8.3 defaultTenant: DEFAULT_TENANT 已配置 → 透传', () => {
  assert.equal(defaultTenant({ DEFAULT_TENANT: 'club-x' }), 'club-x');
});

test('8.3 defaultTenant: DEFAULT_TENANT 未配置（undefined）→ 缺省 bh2ro', () => {
  assert.equal(defaultTenant({}), 'bh2ro');
});

test('8.3 defaultTenant: DEFAULT_TENANT=\'\'（空串）→ 经 || 缺省 bh2ro（断言空串不透传）', () => {
  // 钉死用 || 而非 ??：空串落入缺省，绝不透传空串（否则全面静默空结果）
  assert.equal(defaultTenant({ DEFAULT_TENANT: '' }), 'bh2ro');
  assert.notEqual(defaultTenant({ DEFAULT_TENANT: '' }), '');
});

// 非查询面集合判定（isNonQuerySurface 未导出 → 测内复算等价集合，仿 worker NON_QUERY_SURFACE + 前缀匹配）。
// 见 issues：建议 export isNonQuerySurface 以直接锚定实现。
const NON_QUERY_SURFACE = new Set(['/sync', '/pull', '/ping']);
function isNonQuerySurfaceLocal(routePath) {
  return (
    NON_QUERY_SURFACE.has(routePath) ||
    routePath.startsWith('/api/sf/') ||
    routePath.startsWith('/api/wechat/')
  );
}

test('8.2 非查询面判定：/sync /pull /ping /api/sf/* /api/wechat/* 经 parseTenantPrefix 剥离后 routePath ∈ 非查询面集合', () => {
  for (const ep of ['/sync', '/pull', '/ping', '/api/sf/route-push', '/api/wechat/auth-callback']) {
    const parsed = parseTenantPrefix(`/t/x${ep}`);
    assert.equal(parsed.slug, 'x', `前缀应解析出 slug：/t/x${ep}`);
    // routePath 去尾斜杠口径与 worker 一致（worker: prefix.routePath.replace(/\/$/,'')||'/'）
    const routePath = parsed.routePath.replace(/\/$/, '') || '/';
    assert.equal(isNonQuerySurfaceLocal(routePath), true, `${ep} 应判为非查询面（gate 命中 → 404）`);
  }
});

test('8.2 非查询面判定：查询面端点 routePath ∉ 非查询面集合（不被 gate 误伤）', () => {
  for (const ep of ['/api/query', '/api/config', '/api/session', '/api/session/challenge', '/api/callsigns/BV2ABC', '/']) {
    assert.equal(isNonQuerySurfaceLocal(ep), false, `${ep} 不应判为非查询面`);
  }
});

// ============================================================
// 8.4（关键 D4）会话签名双 path 不变量
// ============================================================

test('8.4 会话签名：带前缀路径签名 + 服务端按同一带前缀 pathname 校验 → 通过', async () => {
  const sk = randomHex(32);
  const sid = 'SID-abc';
  const prefixedPath = '/t/bh2ro/api/callsigns/BV2ABC'; // 客户端实际请求的【含前缀】pathname
  const ts = '1700000000000';
  const nonce = 'NONCE-1';
  // 客户端对【含前缀】pathname 签名（前端 signQuery 的 url.pathname 已从带前缀入参派生）
  const sig = await hmacSha256Hex(sk, buildCanonicalPayload({ sid, path: prefixedPath, params: {}, ts, nonce }));
  // 服务端 verifySessionSig 实参 = 原始 url.pathname（含同一前缀，未被剥离改写）
  const ok = await verifySessionSig({ sid, path: prefixedPath, params: {}, ts, nonce }, sk, sig);
  assert.equal(ok, true, '带前缀签名 + 带前缀校验应通过');
});

test('8.4 会话签名：对剥离路径签名、服务端按带前缀 pathname 校验 → 失败（证剥离/双前缀未误入校验）', async () => {
  const sk = randomHex(32);
  const sid = 'SID-abc';
  const prefixedPath = '/t/bh2ro/api/callsigns/BV2ABC';
  const strippedPath = '/api/callsigns/BV2ABC'; // 剥离前缀后的 routePath（仅供分发，禁入签名）
  const ts = '1700000000000';
  const nonce = 'NONCE-1';
  // 误把剥离路径喂给签名（反模式：routePath 误入签名校验）
  const sigOnStripped = await hmacSha256Hex(sk, buildCanonicalPayload({ sid, path: strippedPath, params: {}, ts, nonce }));
  // 服务端仍按带前缀 pathname 校验 → 必须失败（否则前端签带前缀、服务端按剥离校验会致全部前缀查询 401）
  const ok = await verifySessionSig({ sid, path: prefixedPath, params: {}, ts, nonce }, sk, sigOnStripped);
  assert.equal(ok, false, '剥离路径签名按带前缀 pathname 校验应失败');
});

test('8.4 会话签名：双前缀路径签名按单前缀 pathname 校验 → 失败（防前端二次拼接 /t/x/t/x）', async () => {
  const sk = randomHex(32);
  const sid = 'SID-abc';
  const singlePrefixed = '/t/bh2ro/api/callsigns/BV2ABC';
  const doublePrefixed = '/t/bh2ro/t/bh2ro/api/callsigns/BV2ABC';
  const ts = '1700000000000';
  const nonce = 'NONCE-1';
  const sigOnDouble = await hmacSha256Hex(sk, buildCanonicalPayload({ sid, path: doublePrefixed, params: {}, ts, nonce }));
  const ok = await verifySessionSig({ sid, path: singlePrefixed, params: {}, ts, nonce }, sk, sigOnDouble);
  assert.equal(ok, false, '双前缀签名按单前缀校验应失败（signQuery 内禁二次拼前缀）');
});

// ============================================================
// 8.5 读取面 / config（worker fetch 集成，mock env）
// ============================================================

// 最小 env：覆盖 /api/config 所需的全局开关 + DB；ASSETS/RATE_LIMIT 缺省（fail-open）。
function makeConfigEnv({ activeTenants = [], names = {}, vars = {} } = {}) {
  return makeEnv({ activeTenants, names, env: vars });
}

function req(path, init) {
  return new Request(`https://qsl.example${path}`, init);
}
const ctx = { waitUntil() {} };

test('8.5 /api/config: 显式活跃 slug → 含嵌套 tenant:{id,name}（取自 tenants 表）', async () => {
  const env = makeConfigEnv({ activeTenants: ['bh2ro'], names: { bh2ro: 'BH2RO 俱乐部' } });
  const res = await worker.fetch(req('/t/bh2ro/api/config'), env, ctx);
  assert.equal(res.status, 200);
  const body = await res.json();
  assert.deepEqual(body.tenant, { id: 'bh2ro', name: 'BH2RO 俱乐部' });
});

test('8.5 /api/config: 不下发任何查询签名密钥 / captcha', async () => {
  const env = makeConfigEnv({ activeTenants: ['bh2ro'], names: { bh2ro: 'X' } });
  const res = await worker.fetch(req('/t/bh2ro/api/config'), env, ctx);
  const body = await res.json();
  const keys = Object.keys(body);
  assert.ok(!('sign_key' in body) && !('client_sign_key' in body) && !('CLIENT_SIGN_KEY' in body), 'config 禁含签名密钥');
  assert.ok(!('captcha' in body), 'config 禁含 captcha 配置');
  // 防整串里夹带（如嵌套对象误带）：序列化后不含 sign_key/captcha 字样
  const flat = JSON.stringify(body).toLowerCase();
  assert.ok(!flat.includes('sign_key') && !flat.includes('captcha'), `config 序列化不应含密钥/验证码字样：${keys}`);
});

test('8.5 /api/config: 未知/停用 slug → 404（驱动前端「租户不存在」）', async () => {
  const env = makeConfigEnv({ activeTenants: ['bh2ro'] });
  const res = await worker.fetch(req('/t/ghost/api/config'), env, ctx);
  assert.equal(res.status, 404);
});

test('8.5 /api/config: bare → tenant.id=DEFAULT_TENANT、tenant.name 为 null 且不读 tenants 表', async () => {
  const env = makeConfigEnv({ activeTenants: ['bh2ro'], names: { bh2ro: '不应被取' }, vars: { DEFAULT_TENANT: 'bh2ro' } });
  const res = await worker.fetch(req('/api/config'), env, ctx);
  assert.equal(res.status, 200);
  const body = await res.json();
  assert.equal(body.tenant.id, 'bh2ro');
  assert.equal(body.tenant.name, null, 'bare config 的 name 恒为 null（不读 tenants 表取展示名）');
  assert.equal(env._prepareCalls(), 0, 'bare /api/config 不读 tenants 表');
});

test('8.5 /api/config: bare DEFAULT_TENANT 配置值透传到 tenant.id', async () => {
  const env = makeConfigEnv({ vars: { DEFAULT_TENANT: 'clubx' } });
  const res = await worker.fetch(req('/api/config'), env, ctx);
  const body = await res.json();
  assert.equal(body.tenant.id, 'clubx');
  assert.equal(env._prepareCalls(), 0);
});

test('8.5 读取面 tenant_id 来自路由解析、不取前端参数：bare /api/query 用 DEFAULT_TENANT（忽略 ?tenant= 噪声）', async () => {
  // 读取面 SQL 的 tenant_id 由 resolveQueryTenant 注入；前端传的 ?tenant= 不参与 WHERE。
  // 此处验证：bare 查询不被租户 404 拒（DEFAULT_TENANT 解析成功），而是进入会话管线被 fail-closed 拒（非 200 泄漏）。
  // 本最小 env 未配 SESSION_SECRET/RATE_LIMIT → sessionSubsystemReady=false → 503（会话功能不可用）；
  // 关键是【非 404、非 200】：证 tenant_id 取自解析值、且前端 ?tenant= 噪声未把请求旁路到他租户数据。
  const env = makeConfigEnv({ activeTenants: ['bh2ro'], vars: { DEFAULT_TENANT: 'bh2ro' } });
  const res = await worker.fetch(req('/api/query?callsign=BV2ABC&tenant=evil'), env, ctx);
  assert.notEqual(res.status, 404, 'bare 查询不应租户 404（DEFAULT_TENANT 解析成功）');
  assert.notEqual(res.status, 200, '无会话凭据绝不放行返 200（fail-closed），不得因 ?tenant= 旁路出数据');
  assert.equal(res.status, 503, '会话子系统未就绪 → 503（已越过租户解析、进入会话管线），证未走前端参数旁路');
});

test('8.5 读取面：显式未知 slug 查询 → 404（早于会话闸门，租户解析失败即拒）', async () => {
  const env = makeConfigEnv({ activeTenants: ['bh2ro'] });
  const res = await worker.fetch(req('/t/ghost/api/query?callsign=BV2ABC'), env, ctx);
  assert.equal(res.status, 404, '未知 slug 数据端点 → 404');
});

// ============================================================
// 8.6 微信订阅 state / callback（worker fetch 集成）
// ============================================================

test('8.6 callback: state 按【首个】冒号拆分——bh2ro:BH2RO/P → tenant=bh2ro, callsign=BH2RO/P（含 /）', async () => {
  // 未配置 WECHAT_APPID/SECRET → 活跃校验通过后在「未配置微信服务号」503 处停下，
  // 证明 state 已成功拆分 + 租户活跃校验通过（503 而非 400「无效租户」/「无效呼号」）。
  const env = makeConfigEnv({ activeTenants: ['bh2ro'] });
  const res = await worker.fetch(req('/api/wechat/auth-callback?code=CODE123&state=bh2ro%3ABH2RO%2FP'), env, ctx);
  assert.equal(res.status, 503, '首冒号拆分得 tenant=bh2ro(活跃)+callsign=BH2RO/P(合法) → 落到未配置微信 503');
  assert.equal(await res.text(), '未配置微信服务号');
});

test('8.6 callback: 无冒号 state → 兜底租户取 DEFAULT_TENANT（活跃则过校验）', async () => {
  const env = makeConfigEnv({ activeTenants: ['clubx'], vars: { DEFAULT_TENANT: 'clubx' } });
  // 纯 callsign（无冒号）→ tenant=DEFAULT_TENANT=clubx（活跃）→ 503（未配置微信）
  const res = await worker.fetch(req('/api/wechat/auth-callback?code=CODE&state=BV2ABC'), env, ctx);
  assert.equal(res.status, 503, '无冒号兜底 DEFAULT_TENANT(clubx 活跃) → 503');
});

test('8.6 callback: 无冒号 state 兜底租户未 seed → 400「无效租户」（DEFAULT_TENANT 误配安全拒绝）', async () => {
  const env = makeConfigEnv({ activeTenants: ['bh2ro'], vars: { DEFAULT_TENANT: 'ghost' } });
  const res = await worker.fetch(req('/api/wechat/auth-callback?code=CODE&state=BV2ABC'), env, ctx);
  assert.equal(res.status, 400);
  assert.equal(await res.text(), '无效租户');
});

test('8.6 callback: :callsign（空租户段）→ 400「无效租户」（安全拒绝，不落垃圾绑定）', async () => {
  const env = makeConfigEnv({ activeTenants: ['bh2ro'] });
  // 首冒号在位置 0 → tenant_id='' 空串 → 活跃校验失败
  const res = await worker.fetch(req('/api/wechat/auth-callback?code=CODE&state=%3ABV2ABC'), env, ctx);
  assert.equal(res.status, 400);
  assert.equal(await res.text(), '无效租户');
});

test('8.6 callback: <未seed>:callsign → 400「无效租户」', async () => {
  const env = makeConfigEnv({ activeTenants: ['bh2ro'] });
  const res = await worker.fetch(req('/api/wechat/auth-callback?code=CODE&state=ghost%3ABV2ABC'), env, ctx);
  assert.equal(res.status, 400);
  assert.equal(await res.text(), '无效租户');
});

test('8.6 callback: /t/ 前缀 callback 一律 404（非查询面 gate，租户不取自路径）', async () => {
  const env = makeConfigEnv({ activeTenants: ['bh2ro'] });
  const res = await worker.fetch(req('/t/bh2ro/api/wechat/auth-callback?code=CODE&state=bh2ro%3ABV2ABC'), env, ctx);
  assert.equal(res.status, 404, '/api/wechat/* 带前缀经前缀存在 gate → 404（早于 handler）');
});

test('8.6 callback: 首冒号拆分纯逻辑——多冒号只拆首个（tenant 段不含冒号、callsign 余段含冒号亦归 callsign）', () => {
  // 复算 worker line 970-978 的首冒号拆分逻辑（state 解析纯逻辑锚）
  const split = (decoded, deflt) => {
    const i = decoded.indexOf(':');
    if (i >= 0) return { tenant: decoded.slice(0, i), callsign: decoded.slice(i + 1).toUpperCase() };
    return { tenant: deflt, callsign: decoded.toUpperCase() };
  };
  assert.deepEqual(split('bh2ro:BH2RO/P', 'D'), { tenant: 'bh2ro', callsign: 'BH2RO/P' });
  assert.deepEqual(split('bh2ro:a:b', 'D'), { tenant: 'bh2ro', callsign: 'A:B' }); // 仅拆首冒号
  assert.deepEqual(split('BV2ABC', 'deflt'), { tenant: 'deflt', callsign: 'BV2ABC' }); // 无冒号兜底
  assert.deepEqual(split(':BV2ABC', 'D'), { tenant: '', callsign: 'BV2ABC' }); // 空租户段
});

// ============================================================
// 8.2 非查询面前缀存在 gate（worker fetch 集成，证 404 早于 handler）
// ============================================================

test('8.2 gate: /t/x/sync (POST) → 404（早于 sync handler，不落入认证流）', async () => {
  const env = makeConfigEnv({ activeTenants: ['x'] });
  const res = await worker.fetch(req('/t/x/sync', { method: 'POST' }), env, ctx);
  assert.equal(res.status, 404, '/t/x/sync routePath=/sync 应被前缀存在 gate 拦为 404');
});

test('8.2 gate: /t/x/ping (GET) → 404', async () => {
  const env = makeConfigEnv({ activeTenants: ['x'] });
  const res = await worker.fetch(req('/t/x/ping'), env, ctx);
  assert.equal(res.status, 404);
});

test('8.2 gate: /t/x/api/sf/route-push (POST) → 404', async () => {
  const env = makeConfigEnv({ activeTenants: ['x'] });
  const res = await worker.fetch(req('/t/x/api/sf/route-push', { method: 'POST' }), env, ctx);
  assert.equal(res.status, 404);
});

// ============================================================
// 8.7 数据端点 worker.fetch 闸门接线（变异/回归强锚，补 fetch 层覆盖缺口）
// ============================================================

// (D-fix 1) namespaceInvalid → 404 接线：worker.fetch 对非法 /t 前缀返 404（证
// `if(prefix.namespaceInvalid) return 404` 真接线、禁 fall-through 当 bare 服外壳）。
test('8.7 namespaceInvalid 接线: /t、/t/、/t//api/query、大写/下划线 slug 数据端点 → 404', async () => {
  const env = makeConfigEnv({ activeTenants: ['bh2ro'] });
  // 装 200-shell ASSETS：若 namespaceInvalid gate 被绕过、非法 /t 被当 bare 干净 fall-through，
  // 会落到此 200——故必须 404 才证「worker 顶层 gate 真 404、禁 fall-through」。
  // （无 ASSETS 时 worker 终态兜底对未路由路径亦返 404，无法区分 gate-404 与 fall-through-404，锚会 vacuous。）
  env.ASSETS = { fetch: async () => new Response('SHELL', { status: 200 }) };
  for (const p of ['/t', '/t/', '/t//api/query', '/t/ABC/api/query', '/t/a_b/api/query']) {
    const res = await worker.fetch(req(p), env, ctx);
    assert.equal(res.status, 404, `${p} 非法 /t 前缀应在 worker 顶层 404（禁 fall-through 服外壳）`);
  }
});

// (D-fix 2) 数据端点未知 slug → 404 不被 callsign gate 旁路（本轮修的 bug：
// 即使【无 callsign】也先做租户活跃校验，否则未知 slug 会 fall-through 当 bare 服外壳）。
test('8.7 未知 slug 数据端点【无 callsign】→ 404（不被 callsign 存在性旁路）', async () => {
  const env = makeConfigEnv({ activeTenants: ['bh2ro'] });
  // 装一个返 200 的 ASSETS 外壳：若 bug 复现（未知 slug 无 callsign fall-through 当 bare 服外壳），
  // 会落到此 200——故必须 404 才证「数据端点校验先于 callsign gate、不被旁路」（无 ASSETS 时 fall-through 亦 404，无法区分 bug）。
  env.ASSETS = { fetch: async () => new Response('SHELL', { status: 200 }) };
  // /t/<未 seed>/api/query 无 ?callsign= → 仍须 404（不 fall-through 当 bare 服外壳）。
  // 注：/api/query 是 isQueryDataEndpoint===true 的确定数据端点（与 callsign 存在性无关），是本 bug 的强锚；
  // /api/callsigns/<cs> 须带 cs 段才算数据端点（裸 /api/callsigns/ 经去尾斜杠成 /api/callsigns、非数据端点、落外壳），
  // 故此处只用 /api/query 无 callsign 这一无歧义锚（带 cs 的未知 slug 见下方 r2）。
  const r1 = await worker.fetch(req('/t/ghost/api/query'), env, ctx);
  assert.equal(r1.status, 404, '未知 slug /api/query 无 callsign 仍须 404（不落 ASSETS 外壳）');
  // /t/<未 seed>/api/callsigns/<cs>（路径式带 cs，命中数据端点）→ 仍走租户校验 → 404
  const r2 = await worker.fetch(req('/t/ghost/api/callsigns/BV2ABC'), env, ctx);
  assert.equal(r2.status, 404, '未知 slug /api/callsigns/<cs> → 404');
});

test('8.7 对照: active slug 数据端点【无 callsign】→ 非 404（证 active 不误 404）', async () => {
  const env = makeConfigEnv({ activeTenants: ['bh2ro'] });
  // active slug /api/query 无 callsign：租户校验通过 → 不 404；落静态外壳（ASSETS 缺省）→ 顶层 404? 否：
  // 无 ASSETS 时落最终 json 404。为锚「租户校验通过、未因未知 slug 被拒」，改用 active+无 callsign 时
  // worker 会跳过数据查询分支（callsign 为空），最终落 ASSETS/Not Found。此处 ASSETS 缺省 → 最终 404，
  // 无法与「未知 slug 404」区分。故改装一个返 200 的 ASSETS 桩，证 active 真的落到了外壳分支。
  env.ASSETS = { fetch: async () => new Response('SHELL', { status: 200 }) };
  const res = await worker.fetch(req('/t/bh2ro/api/query'), env, ctx);
  assert.notEqual(res.status, 404, 'active slug 无 callsign 不应被未知 slug 404 误伤（落静态外壳）');
  assert.equal(res.status, 200, 'active 无 callsign → 落 ASSETS 外壳（200），证租户校验已通过');
});

test('8.7 行为不变: bare /api/query 无 callsign → 非 404（落外壳，行为同改前）', async () => {
  const env = makeConfigEnv({ activeTenants: ['bh2ro'], vars: { DEFAULT_TENANT: 'bh2ro' } });
  env.ASSETS = { fetch: async () => new Response('SHELL', { status: 200 }) };
  const res = await worker.fetch(req('/api/query'), env, ctx);
  assert.notEqual(res.status, 404, 'bare 无 callsign → 不 404（DEFAULT_TENANT 解析成功、落外壳）');
  assert.equal(res.status, 200);
});

// ── 完整有效会话脚手架（worker.fetch 集成，真 KV + 真 token/sk/签名）────────────
// 建一个带前缀的有效会话：CF-Connecting-IP=具体 IP → bkey=该 IP → 会话 binding_mode='ip'。
// 路径前缀【含 /t/<slug>/】，签名按【含前缀】pathname（worker verifySessionSig 喂 url.pathname，非 routePath）。

// 真 in-memory KV（仿 fail-matrix.test.js mockKV）：会话/配额/nonce 键读写。
function memKV() {
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

// 数据端点会话集成 env：tenants 活跃校验 + cards SELECT（返 1 行、捕获 bind 实参）+ 真 KV/SECRET。
// cardBinds 累积每次 cards 查询的 .bind(...) 实参（[tenant_id, callsign]），供断言读取面 tenant bind。
function makeSessionDataEnv({ activeTenants = [], vars = {} } = {}) {
  const cardBinds = [];
  const kv = memKV();
  const env = {
    ...vars,
    RATE_LIMIT: kv,
    SESSION_SECRET: vars.SESSION_SECRET || 'integration-session-secret',
    _cardBinds: () => cardBinds,
    _kv: kv,
    DB: {
      prepare(sql) {
        const q = { args: [], _sql: sql };
        q.bind = (...a) => { q.args = a; return q; };
        q.first = async () => {
          // tenants 活跃校验（resolveQueryTenant）
          if (sql.includes('FROM tenants')) {
            return activeTenants.includes(q.args[0]) ? { 1: 1 } : null;
          }
          return null;
        };
        q.all = async () => {
          // cards 读取面查询：记录 bind 实参（[tenant_id, callsignUpper]）
          if (sql.includes('FROM cards')) {
            cardBinds.push([...q.args]);
            // 返回一行可映射的 card（metadata 缺省 null）
            return { results: [{ id: 'card-1', project_id: 'p1', callsign: q.args[1], qty: 1, serial: null, status: 'received', metadata: null, project_name: '项目X' }] };
          }
          return { results: [] };
        };
        q.run = async () => ({ meta: { changes: 1 } });
        return q;
      },
    },
  };
  return env;
}

// 造一个【带前缀路径】的有效会话请求：在 KV 写 session:<sid>，按 signPath（默认=含前缀 pathname）签名。
// signPath 可显式传【剥离前缀】路径以构造错配（401）用例。
async function buildPrefixedSessionReq(env, {
  slug = 'bh2ro', callsign = 'BV2ABC', ip = '203.0.113.7', nonce = 'noncepref01', signPath,
} = {}) {
  const secret = env.SESSION_SECRET;
  const sid = randomHex(18);
  const sk = randomHex(32);
  const ts = Date.now();
  const bkey = ip; // IPv4 → clientBindingKey 直通 /32
  // 写会话（binding_mode='ip'，UA 与请求一致）
  const ua = 'Mozilla/5.0 (integration)';
  await env._kv.put(`session:${sid}`, JSON.stringify({
    binding_mode: 'ip', ip: bkey, ua_hash: await uaHash(ua), exp: ts + 60000, sk,
  }));
  const token = await makeToken(sid, secret);
  const reqPath = `/t/${slug}/api/callsigns/${callsign}`; // worker 实际请求的【含前缀】pathname
  const url = new URL(`https://qsl.example${reqPath}?token=${encodeURIComponent(token)}&_ts=${ts}&_nonce=${nonce}`);
  // 签名 path：默认 = 含前缀的 reqPath（worker 喂 url.pathname 校验）；可传剥离路径构造错配
  const pathToSign = signPath ?? reqPath;
  const sig = await hmacSha256Hex(sk, buildCanonicalPayload({ sid, path: pathToSign, params: url.searchParams, ts: String(ts), nonce }));
  url.searchParams.set('_sig', sig);
  const request = new Request(url.toString(), {
    headers: { 'User-Agent': ua, 'CF-Connecting-IP': ip },
  });
  return { request, sid, sk, bkey };
}

// (D-fix 3 正向锚 + D-fix 4) 带前缀有效会话 → 200 且 cards SQL bind 的 tenant_id===slug。
test('8.7 D4 接线: 带前缀有效会话 /t/<active>/api/callsigns/<cs> → 200 且 cards bind tenant_id=slug', async () => {
  const env = makeSessionDataEnv({ activeTenants: ['bh2ro'] });
  const { request } = await buildPrefixedSessionReq(env, { slug: 'bh2ro', callsign: 'BV2ABC' });
  const res = await worker.fetch(request, env, ctx);
  assert.equal(res.status, 200, '含前缀路径签名 + worker 按含前缀 url.pathname 校验 → 200（D4 正向锚）');
  const body = await res.json();
  assert.equal(body.success, true);
  assert.equal(body.callsign, 'BV2ABC');
  // 读取面 bind 真锚：cards SQL 的 tenant_id 来自路由解析（slug），非前端参数
  const binds = env._cardBinds();
  assert.equal(binds.length, 1, '应恰执行一次 cards 查询');
  assert.equal(binds[0][0], 'bh2ro', 'cards 查询 tenant_id bind 必须 === 解析出的活跃 slug');
  assert.equal(binds[0][1], 'BV2ABC', 'cards 查询 callsign bind 为大写呼号');
});

// (D-fix 3 反向锚) 对【剥离前缀】路径签名的同会话 → 401（证 worker 喂 url.pathname 非 routePath）。
test('8.7 D4 接线: 同会话对【剥离前缀】路径签名 → 401（worker 校验喂 url.pathname 含前缀，错配）', async () => {
  const env = makeSessionDataEnv({ activeTenants: ['bh2ro'] });
  const { request } = await buildPrefixedSessionReq(env, {
    slug: 'bh2ro', callsign: 'BV2ABC', nonce: 'noncestrip1',
    signPath: '/api/callsigns/BV2ABC', // 误对剥离前缀路径签名
  });
  const res = await worker.fetch(request, env, ctx);
  assert.equal(res.status, 401, '剥离前缀签名 vs worker 含前缀 url.pathname 校验 → 401（签名错配）');
  // 错配应在签名校验阶段被拒、不应触达 cards 查询
  assert.equal(env._cardBinds().length, 0, '签名错配不应触达 cards 查询');
});

// (D-fix 4) 读取面 bind 真锚：bare 会话查询 cards bind = DEFAULT_TENANT；?tenant= 噪声不参与。
test('8.7 读取面 bind 真锚: bare 有效会话 → cards bind tenant_id=DEFAULT_TENANT（忽略 ?tenant=evil）', async () => {
  const env = makeSessionDataEnv({ activeTenants: ['bh2ro'], vars: { DEFAULT_TENANT: 'bh2ro' } });
  // 直接造一个 bare /api/query 的有效会话（无前缀），并夹带 ?tenant=evil 噪声参数。
  const secret = env.SESSION_SECRET;
  const sid = randomHex(18);
  const sk = randomHex(32);
  const ts = Date.now();
  const ip = '203.0.113.9';
  const ua = 'Mozilla/5.0 (integration-bare)';
  await env._kv.put(`session:${sid}`, JSON.stringify({
    binding_mode: 'ip', ip, ua_hash: await uaHash(ua), exp: ts + 60000, sk,
  }));
  const token = await makeToken(sid, secret);
  const reqPath = '/api/query';
  const url = new URL(`https://qsl.example${reqPath}?callsign=BV2ABC&tenant=evil&token=${encodeURIComponent(token)}&_ts=${ts}&_nonce=noncebare01`);
  // 签名按 url.pathname（bare '/api/query'）+ 全部业务参数（含 callsign、tenant=evil；token/_*被 canonical 排除）
  const sig = await hmacSha256Hex(sk, buildCanonicalPayload({ sid, path: reqPath, params: url.searchParams, ts: String(ts), nonce: 'noncebare01' }));
  url.searchParams.set('_sig', sig);
  const request = new Request(url.toString(), { headers: { 'User-Agent': ua, 'CF-Connecting-IP': ip } });

  const res = await worker.fetch(request, env, ctx);
  assert.equal(res.status, 200, 'bare 有效会话 → 200');
  const binds = env._cardBinds();
  assert.equal(binds.length, 1);
  assert.equal(binds[0][0], 'bh2ro', 'bare cards bind = DEFAULT_TENANT（解析值）');
  assert.notEqual(binds[0][0], 'evil', 'cards bind 绝不取自 ?tenant= 噪声参数');
});

// (D-fix 5) isNonQuerySurface 直接集合锚 + /t/x/pull fetch gate（补齐唯一缺 fetch gate 的非查询面端点）。
test('8.7 isNonQuerySurface 集合锚: /sync /pull /ping /api/sf/* /api/wechat/* → true', () => {
  for (const p of ['/sync', '/pull', '/ping', '/api/sf/route-push', '/api/sf/x', '/api/wechat/auth-callback', '/api/wechat/x']) {
    assert.equal(isNonQuerySurface(p), true, `${p} 应判为非查询面`);
  }
});

test('8.7 isNonQuerySurface 集合锚: 查询面 /api/query /api/config /api/session* /api/callsigns/* / → false', () => {
  for (const p of ['/api/query', '/api/config', '/api/session', '/api/session/challenge', '/api/callsigns/BV2ABC', '/']) {
    assert.equal(isNonQuerySurface(p), false, `${p} 不应判为非查询面`);
  }
});

test('8.7 gate: /t/x/pull (GET) → 404（补齐 /pull 的前缀存在 gate fetch 锚）', async () => {
  const env = makeConfigEnv({ activeTenants: ['x'] });
  const res = await worker.fetch(req('/t/x/pull'), env, ctx);
  assert.equal(res.status, 404, '/t/x/pull routePath=/pull 应被前缀存在 gate 拦为 404');
});
