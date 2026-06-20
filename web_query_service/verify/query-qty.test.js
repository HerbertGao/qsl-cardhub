/**
 * add-query-qty-display 单测（node:test）：服务端按租户 qty_display_mode 脱敏卡片数量（公开查询面 default-deny）。
 *
 * 两层覆盖：
 *   A. 纯函数 formatQtyByMode（分桶/阈值/mode-default-deny 的最稳锚）：
 *      exact→原始整数；approximate 边界 10/11/50/51；'foo'/''/undefined/null → undefined（不展示）。
 *   B. 端点级行为（worker.fetch 集成，真 KV + 真 token/sk/签名，mock env.DB）：
 *      - exact → item.qty 为原始整数（number）
 *      - approximate → item.qty 为受限字符串 {≤10,≤50,>50} 之一，且 item 内不存在数值型 qty/原始精确数字段
 *      - 无该行 / 未知值('foo') / 空串 → item 不含 qty 键
 *      - 读设置异常（app_settings .first() 抛错）→ 仍返回卡片列表且 item 不含 qty（非 500、不丢卡片）
 *      - bare 默认租户且无 qty_display_mode 行 → item 不含 qty（公开面 deny 路径）
 *      - 多租户：/t/<slug>/ 查询时 cards 与 app_settings 读取所 bind 的 tenant_id 一致（不串租户）
 *
 * 端点级脚手架仿 tenant-path-routing.test.js 的 makeSessionDataEnv / buildPrefixedSessionReq（真 KV + 含前缀签名）。
 * mock 的 DB.prepare 按 SQL 文本区分 cards / app_settings / tenants 三类查询。
 */
import { test } from 'node:test';
import assert from 'node:assert/strict';

import worker, { formatQtyByMode } from '../src/worker/index.js';
import { buildCanonicalPayload } from '../src/worker/canonical.js';
import { hmacSha256Hex, randomHex, makeToken, uaHash } from '../src/worker/session.js';

const ctx = { waitUntil() {} };

// ============================================================
// A. 纯函数 formatQtyByMode（分桶/阈值/mode-default-deny 最稳锚）
// ============================================================

test('formatQtyByMode: exact → 返回原始整数（类型为 number、值不变）', () => {
  for (const q of [1, 7, 10, 11, 50, 51, 9999]) {
    const v = formatQtyByMode(q, 'exact');
    assert.equal(typeof v, 'number', 'exact 必须返回 number 原值');
    assert.equal(v, q);
  }
});

test('formatQtyByMode: approximate 边界 10/11/50/51 → 受限字符串分桶（严格等值）', () => {
  assert.strictEqual(formatQtyByMode(10, 'approximate'), '≤10');
  assert.strictEqual(formatQtyByMode(11, 'approximate'), '≤50');
  assert.strictEqual(formatQtyByMode(50, 'approximate'), '≤50');
  assert.strictEqual(formatQtyByMode(51, 'approximate'), '>50');
  // 下界与高值也落在受限集内
  assert.strictEqual(formatQtyByMode(1, 'approximate'), '≤10');
  assert.strictEqual(formatQtyByMode(9999, 'approximate'), '>50');
});

test('formatQtyByMode: approximate 返回值恒为受限字符串集 {≤10,≤50,>50} 之一', () => {
  const allowed = new Set(['≤10', '≤50', '>50']);
  for (let q = 1; q <= 120; q++) {
    const v = formatQtyByMode(q, 'approximate');
    assert.equal(typeof v, 'string', `approximate 必须返回字符串（qty=${q}）`);
    assert.ok(allowed.has(v), `approximate(${q})=${v} 须 ∈ {≤10,≤50,>50}`);
  }
});

test('formatQtyByMode: 未知值/空串/undefined/null → undefined（default-deny，省略 qty）', () => {
  for (const mode of ['foo', '', 'Exact', 'EXACT', 'approx', undefined, null, 0, false]) {
    assert.strictEqual(
      formatQtyByMode(7, mode),
      undefined,
      `mode=${JSON.stringify(mode)} 必须返回 undefined（不展示）`
    );
  }
});

// ============================================================
// B. 端点级行为（worker.fetch 集成，真 KV + 真签名，mock env.DB）
// ============================================================

// 真 in-memory KV（仿 fail-matrix.test.js / tenant-path-routing.test.js mockKV）。
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

/**
 * 数据端点会话集成 env：mock DB.prepare 按 SQL 文本分辨三类查询：
 *   - 'FROM tenants'     → resolveQueryTenant 活跃校验（.first()，返 {1:1} 命中 / null 未命中），记 tenantBinds
 *   - 'FROM cards'       → 读取面卡片查询（.all()，返一行 card），记 cardBinds[ [tenant_id, callsign] ]
 *   - 'app_settings'     → qty_display_mode 读取（.first()），记 settingBinds；settingMode 为返回 value（undefined→无该行）
 * @param opts.settingMode  app_settings 返回的 value：字符串=返回该 value；undefined=无该行（null）；
 * @param opts.settingThrow true 时 app_settings 的 .first() 抛错（模拟读设置异常）
 * @param opts.cardQty      cards 行的 qty（默认 5）
 */
function makeQtyEnv({ activeTenants = [], vars = {}, settingMode, settingThrow = false, cardQty = 5 } = {}) {
  const cardBinds = [];
  const settingBinds = [];
  const tenantBinds = [];
  const kv = memKV();
  const env = {
    ...vars,
    RATE_LIMIT: kv,
    SESSION_SECRET: vars.SESSION_SECRET || 'qty-integration-secret',
    _cardBinds: () => cardBinds,
    _settingBinds: () => settingBinds,
    _tenantBinds: () => tenantBinds,
    _kv: kv,
    DB: {
      prepare(sql) {
        const q = { args: [], _sql: sql };
        q.bind = (...a) => { q.args = a; return q; };
        q.first = async () => {
          if (sql.includes('FROM tenants')) {
            tenantBinds.push([...q.args]);
            return activeTenants.includes(q.args[0]) ? { 1: 1 } : null;
          }
          if (sql.includes('app_settings')) {
            settingBinds.push([...q.args]);
            if (settingThrow) throw new Error('app_settings read jitter');
            return settingMode === undefined ? null : { value: settingMode };
          }
          return null;
        };
        q.all = async () => {
          if (sql.includes('FROM cards')) {
            cardBinds.push([...q.args]);
            return {
              results: [{
                id: 'card-1',
                project_id: 'p1',
                callsign: q.args[1],
                qty: cardQty,
                serial: null,
                status: 'pending',
                metadata: null,
                created_at: '2026-01-01T00:00:00+00:00',
                updated_at: '2026-01-01T00:00:00+00:00',
                project_name: '项目X',
              }],
            };
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

/**
 * 造一个【带前缀路径】或 bare 的有效会话查询请求（真 token/sk/签名，binding_mode='ip'）。
 * slug 传 null → bare /api/query?callsign=；否则 /t/<slug>/api/callsigns/<callsign>。
 */
async function buildSessionReq(env, { slug = 'bh2ro', callsign = 'BV2ABC', ip = '203.0.113.7', nonce } = {}) {
  const secret = env.SESSION_SECRET;
  const sid = randomHex(18);
  const sk = randomHex(32);
  const ts = Date.now();
  const ua = 'Mozilla/5.0 (qty-integration)';
  const n = nonce || `qtynonce${randomHex(4)}`;
  await env._kv.put(`session:${sid}`, JSON.stringify({
    binding_mode: 'ip', ip, ua_hash: await uaHash(ua), exp: ts + 60000, sk,
  }));
  const token = await makeToken(sid, secret);
  const reqPath = slug == null ? '/api/query' : `/t/${slug}/api/callsigns/${callsign}`;
  const qs = slug == null ? `callsign=${callsign}&` : '';
  const url = new URL(`https://qsl.example${reqPath}?${qs}token=${encodeURIComponent(token)}&_ts=${ts}&_nonce=${n}`);
  const sig = await hmacSha256Hex(sk, buildCanonicalPayload({ sid, path: reqPath, params: url.searchParams, ts: String(ts), nonce: n }));
  url.searchParams.set('_sig', sig);
  const request = new Request(url.toString(), {
    headers: { 'User-Agent': ua, 'CF-Connecting-IP': ip },
  });
  return request;
}

// 先自检脚手架：确认会话握手在本 mock env 下能真正放行（否则下方端点级断言会因 503/401 假绿）。
test('B-scaffold: 有效会话 → 200（脚手架能放行，端点级断言非 vacuous）', async () => {
  const env = makeQtyEnv({ activeTenants: ['bh2ro'], settingMode: 'exact' });
  const res = await worker.fetch(await buildSessionReq(env, { slug: 'bh2ro' }), env, ctx);
  assert.equal(res.status, 200, '会话脚手架须能放行到 200（否则端点级用例全为假绿）');
  const body = await res.json();
  assert.equal(body.success, true);
  assert.equal(env._cardBinds().length, 1, '应执行一次 cards 查询');
});

test('B1 exact → item.qty 为原始整数（number），严格等于 cards.qty', async () => {
  const env = makeQtyEnv({ activeTenants: ['bh2ro'], settingMode: 'exact', cardQty: 37 });
  const res = await worker.fetch(await buildSessionReq(env, { slug: 'bh2ro' }), env, ctx);
  assert.equal(res.status, 200);
  const body = await res.json();
  const item = body.items[0];
  assert.equal(typeof item.qty, 'number', 'exact 模式 qty 必须为 number');
  assert.strictEqual(item.qty, 37, 'exact 模式 qty 必须为原始整数');
});

// approximate 各边界：断言 qty 字段【类型为字符串 + 严格等值分桶串】，且 item 内不存在数值型 qty/原始精确数字段。
for (const [qty, expected] of [[10, '≤10'], [11, '≤50'], [50, '≤50'], [51, '>50']]) {
  test(`B2 approximate 边界 qty=${qty} → item.qty==='${expected}'（字符串），且无数值型/原始精确数字段`, async () => {
    const env = makeQtyEnv({ activeTenants: ['bh2ro'], settingMode: 'approximate', cardQty: qty });
    const res = await worker.fetch(await buildSessionReq(env, { slug: 'bh2ro' }), env, ctx);
    assert.equal(res.status, 200);
    const body = await res.json();
    const item = body.items[0];
    assert.equal(typeof item.qty, 'string', 'approximate 模式 qty 必须为字符串');
    assert.strictEqual(item.qty, expected, '须严格等于预期分桶串');
    // 针对【字段类型/值】判定（非全文「不含数字」子串搜索——后者会因 ≤10/≤50 串含 10/50 误报）：
    // 1) qty 字段不得为 number；2) item 任一字段不得 === 原始精确整数 qty（防原始数随其它键泄露）。
    assert.notEqual(typeof item.qty, 'number', 'approximate 下 qty 禁为数值型');
    for (const [k, v] of Object.entries(item)) {
      assert.notStrictEqual(v, qty, `字段 ${k} 不得携带原始精确整数 ${qty}（防原始 qty 与脱敏值并发下发）`);
    }
  });
}

// 无该行 / 未知值('foo') / 空串 → item 不含 qty 字段。
for (const [label, settingMode] of [['无该行', undefined], ["未知值'foo'", 'foo'], ['空串', '']]) {
  test(`B3 ${label} → item 不含 qty 字段（default-deny）`, async () => {
    const env = makeQtyEnv({ activeTenants: ['bh2ro'], settingMode });
    const res = await worker.fetch(await buildSessionReq(env, { slug: 'bh2ro' }), env, ctx);
    assert.equal(res.status, 200, 'default-deny 仍返回卡片列表');
    const body = await res.json();
    const item = body.items[0];
    assert.ok(!('qty' in item), `${label} 时 item 不得含 qty 键`);
  });
}

test('B4 读设置异常（app_settings .first() 抛错）→ 仍返回卡片列表且 item 不含 qty（非 500、不丢卡片）', async () => {
  const env = makeQtyEnv({ activeTenants: ['bh2ro'], settingMode: 'exact', settingThrow: true });
  const res = await worker.fetch(await buildSessionReq(env, { slug: 'bh2ro' }), env, ctx);
  assert.equal(res.status, 200, '读设置异常须 fail-safe（非 500）');
  const body = await res.json();
  assert.equal(body.success, true);
  assert.equal(body.items.length, 1, '不得因读设置失败而丢卡片');
  assert.ok(!('qty' in body.items[0]), '读设置异常 → default-deny，item 不含 qty');
  // 钉死确实尝试读了 app_settings（否则该用例 vacuous）
  assert.equal(env._settingBinds().length, 1, '应尝试读一次 app_settings（异常路径仍触达读取）');
});

test('B5 bare 默认租户且无 qty_display_mode 行 → item 不含 qty（公开面 deny 路径）', async () => {
  // bare：resolveQueryTenant 不读 tenants 表，直接取 DEFAULT_TENANT；app_settings 无该行（settingMode=undefined）。
  const env = makeQtyEnv({ vars: { DEFAULT_TENANT: 'bh2ro' }, settingMode: undefined });
  const res = await worker.fetch(await buildSessionReq(env, { slug: null }), env, ctx);
  assert.equal(res.status, 200);
  const body = await res.json();
  const item = body.items[0];
  assert.ok(!('qty' in item), 'bare 无 qty_display_mode 行 → item 不含 qty（禁 ?? "exact" 回退误伤）');
  // bare cards bind 须为 DEFAULT_TENANT，且 app_settings 读取所 bind 与 cards 一致
  assert.equal(env._cardBinds()[0][0], 'bh2ro', 'bare cards bind = DEFAULT_TENANT');
  assert.equal(env._settingBinds()[0][0], 'bh2ro', 'bare app_settings bind = 同一 DEFAULT_TENANT');
});

test('B6 多租户: /t/<slug>/ 查询时 app_settings 读取所 bind 的 tenant_id 与 cards 查询一致（不串租户）', async () => {
  // cards 属 tenantX；若脱敏读取误用其它租户的 qty_display_mode，bind 会不一致 → 钉死「同一 tenant_id」。
  const env = makeQtyEnv({ activeTenants: ['tenantx'], settingMode: 'approximate', cardQty: 5 });
  const res = await worker.fetch(await buildSessionReq(env, { slug: 'tenantx', callsign: 'BV2ABC' }), env, ctx);
  assert.equal(res.status, 200);
  const cardBinds = env._cardBinds();
  const settingBinds = env._settingBinds();
  assert.equal(cardBinds.length, 1, '应恰一次 cards 查询');
  assert.equal(settingBinds.length, 1, '应恰一次 app_settings 查询');
  assert.equal(cardBinds[0][0], 'tenantx', 'cards bind tenant_id = 解析出的活跃 slug');
  assert.equal(settingBinds[0][0], 'tenantx', 'app_settings bind tenant_id 必须 === cards 查询 tenant_id（不串租户）');
  assert.strictEqual(
    settingBinds[0][0], cardBinds[0][0],
    '脱敏读取与 cards 查询须同一 tenant_id（复用 resolveQueryTenant 结果，禁二次解析/串租户）'
  );
});
