/**
 * QSL CardHub 云端查询与同步服务
 * - GET /ping  连接测试（Bearer API_KEY）
 * - POST /sync 全量同步（Bearer API_KEY）
 * - GET /api/callsigns/:callsign 或 /api/query?callsign= 按呼号查询收卡
 * - POST /api/sf/route-push 顺丰路由推送接收（JSON）
 * - GET /query 按呼号查询页面（含订阅收卡入口）
 */

import { getClientIP, clientBindingKey } from './client-ip.js';
import {
  verifyPow,
  makeToken,
  parseToken,
  verifySessionSig,
  difficultyFor,
  unknownDifficulty,
  uaHash,
  sessionValid,
  randomHex,
} from './session.js';

const CORS_HEADERS = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type, Authorization',
  'Access-Control-Max-Age': '86400',
};

function json(body, status = 200, headers = {}) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json; charset=UTF-8', ...CORS_HEADERS, ...headers },
  });
}

function getBearerToken(request) {
  const auth = request.headers.get('Authorization');
  if (!auth || !auth.startsWith('Bearer ')) return null;
  return auth.slice(7).trim();
}

function serverTime() {
  return new Date().toISOString().replace('Z', '+00:00');
}

// ============================================================
// 限流配置
// ============================================================
const RATE_LIMIT_MAX = 20; // 每分钟最大请求数
const RATE_LIMIT_WINDOW = 60; // 窗口时间（秒）

/**
 * IP 限流检查
 * @param {string} [bucket] 计数桶前缀，区分端点；缺省与查询端点共用 `ratelimit:${ip}`，
 *   传入则用 `ratelimit:${bucket}:${ip}` 独立计数（如订阅回调用 'authcb'，不与查询挤占预算）。
 * @returns {Promise<{allowed: boolean, remaining: number, resetAt: number}>}
 */
export async function checkRateLimit(env, ip, bucket) {
  if (!env.RATE_LIMIT) {
    // KV 未配置时跳过限流（fail-open，可用性优先；KV 为部署前置）
    return { allowed: true, remaining: RATE_LIMIT_MAX, resetAt: 0 };
  }

  const key = bucket ? `ratelimit:${bucket}:${ip}` : `ratelimit:${ip}`;
  const now = Math.floor(Date.now() / 1000);
  const windowStart = now - (now % RATE_LIMIT_WINDOW);
  const resetAt = windowStart + RATE_LIMIT_WINDOW;

  try {
    const stored = await env.RATE_LIMIT.get(key, { type: 'json' });
    let count = 0;

    if (stored && stored.window === windowStart) {
      count = stored.count;
    }

    if (count >= RATE_LIMIT_MAX) {
      return { allowed: false, remaining: 0, resetAt };
    }

    // 递增计数
    count++;
    await env.RATE_LIMIT.put(key, JSON.stringify({ window: windowStart, count }), {
      expirationTtl: RATE_LIMIT_WINDOW + 10, // 多加 10 秒缓冲
    });

    return { allowed: true, remaining: RATE_LIMIT_MAX - count, resetAt };
  } catch {
    // 纯 IP 限流（查询桶 / authcb 桶）是**可用性优先的 fail-open** 闸：KV 运行时读写失败/超时时
    // 不阻断请求（返回 allowed=true），由其后的会话校验（fail-closed）主导放行决定。
    // 注：反滥用关键键（seed/session/quota/nonce/握手桶）走各自 fail-closed 路径，不经本函数。
    return { allowed: true, remaining: RATE_LIMIT_MAX, resetAt };
  }
}

// ============================================================
// 签名校验配置
// ============================================================
const SIGN_TIME_WINDOW = 5 * 60 * 1000; // 5 分钟时间窗口
const NONCE_TTL = 5 * 60; // nonce 存储 5 分钟

/**
 * SHA-256 哈希
 */
async function sha256(message) {
  const encoder = new TextEncoder();
  const data = encoder.encode(message);
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
}

// ============================================================
// 防爬会话 / PoW（query-antibot-session 规范）
// ============================================================
const SESSION_TTL = 10 * 60; // 会话 TTL（秒）
const SESSION_TTL_MS = SESSION_TTL * 1000;
const POWSEED_TTL = 2 * 60; // PoW 题目 TTL（秒，一次性）
const POWRATE_TTL = 5 * 60; // 自适应难度计数窗（秒）
const QUOTA = 50; // 常规单会话查询配额
const QUOTA_UNKNOWN = 3; // unknown 来源会话压低配额（不绑 IP、可搬移，趋零搬移价值）
const HANDSHAKE_RATE_MAX = 30; // 握手桶每分钟上限（独立于查询桶 RATE_LIMIT_MAX）

/**
 * 会话/PoW 硬部署前置：KV 已绑 + SESSION_SECRET 已配。任一缺失 → 会话功能不可用（fail-closed 503）。
 */
export function sessionSubsystemReady(env) {
  return !!(env.RATE_LIMIT && env.SESSION_SECRET);
}

/**
 * 握手桶前置限流（独立桶 `ratelimit:session:<bkey>`，**fail-closed**）。
 * KV 读写失败时抛错 → 调用方 catch → 503（**禁止**复用查询桶 checkRateLimit 的 fail-open 短路）。
 * @returns {Promise<{allowed: boolean}>}
 */
export async function handshakeRateLimit(env, bkey) {
  const key = `ratelimit:session:${bkey}`;
  const now = Math.floor(Date.now() / 1000);
  const windowStart = now - (now % RATE_LIMIT_WINDOW);
  const stored = await env.RATE_LIMIT.get(key, { type: 'json' });
  let count = stored && stored.window === windowStart ? stored.count : 0;
  if (count >= HANDSHAKE_RATE_MAX) return { allowed: false };
  count++;
  await env.RATE_LIMIT.put(key, JSON.stringify({ window: windowStart, count }), {
    expirationTtl: RATE_LIMIT_WINDOW + 10,
  });
  return { allowed: true };
}

/**
 * 计算 challenge 难度。powrate 读写失败 **fail-secure → DIFF_MAX**（禁跌回 baseMin，防攻击者诱发读失败降难度）。
 * unknown 来源恒取最高档（≡ DIFF_MAX）。
 * @returns {Promise<number>}
 */
export async function computeDifficulty(env, bkey) {
  if (bkey === 'unknown') return unknownDifficulty();
  try {
    const now = Math.floor(Date.now() / 1000);
    const windowStart = now - (now % POWRATE_TTL);
    const pr = await env.RATE_LIMIT.get(`powrate:${bkey}`, { type: 'json' });
    const rate = pr && pr.window === windowStart ? pr.count : 0;
    await env.RATE_LIMIT.put(`powrate:${bkey}`, JSON.stringify({ window: windowStart, count: rate + 1 }), {
      expirationTtl: POWRATE_TTL + 10,
    });
    return difficultyFor(rate);
  } catch {
    return unknownDifficulty(); // powrate 读写失败 fail-secure → DIFF_MAX
  }
}

/**
 * 查询端点会话校验（fail-closed）：token HMAC ∧ KV 命中 session ∧ 绑定项（exp/ip/ua）∧ 会话签名 ∧
 * _ts 时窗 ∧ _nonce 防重放 ∧ 会话配额。任一不过返回相应拒绝码。
 * KV 运行时失败/超时 → fail-closed（503），**禁止** fail-open 放行。
 * @returns {Promise<{ok: true} | {ok: false, status: number, message: string}>}
 */
export async function validateQuerySession(env, request, url, bkey) {
  if (!sessionSubsystemReady(env)) return { ok: false, status: 503, message: '会话功能不可用' };
  const token = url.searchParams.get('token');
  const ts = url.searchParams.get('_ts');
  const nonce = url.searchParams.get('_nonce');
  const sig = url.searchParams.get('_sig');
  if (!token || !ts || !nonce || !sig) return { ok: false, status: 401, message: '缺少会话凭据' };
  // _nonce 字符集/长度约束（hex/base64url，8–128）——天然杜绝 `nonce:<nonce>` KV 键的分隔符注入/超长
  if (!/^[A-Za-z0-9_-]{8,128}$/.test(nonce)) return { ok: false, status: 401, message: '会话凭据格式无效' };
  // _ts 时窗（与会话失效区分：时钟漂移/过期签名 → 403；会话无效 → 401）
  const tsNum = parseInt(ts, 10);
  if (Number.isNaN(tsNum) || Math.abs(Date.now() - tsNum) > SIGN_TIME_WINDOW) {
    return { ok: false, status: 403, message: '签名已过期' };
  }
  try {
    const sid = await parseToken(token, env.SESSION_SECRET);
    if (!sid) return { ok: false, status: 401, message: '会话无效' };
    const raw = await env.RATE_LIMIT.get(`session:${sid}`);
    if (!raw) return { ok: false, status: 401, message: '会话无效或已过期' };
    let sess;
    try { sess = JSON.parse(raw); } catch { return { ok: false, status: 401, message: '会话无效' }; }
    const uah = await uaHash(request.headers.get('User-Agent'));
    if (!sessionValid(sess, bkey, uah, Date.now())) return { ok: false, status: 401, message: '会话无效或已过期' };
    // 会话签名（sk）—— canonicalPayload 由共享模块构造（单一事实源）
    const sigOk = await verifySessionSig({ sid, path: url.pathname, params: url.searchParams, ts, nonce }, sess.sk, sig);
    if (!sigOk) return { ok: false, status: 401, message: '签名无效' };
    // _nonce 防重放（fail-closed）
    const nonceKey = `nonce:${nonce}`;
    if (await env.RATE_LIMIT.get(nonceKey)) return { ok: false, status: 401, message: '请求已处理' };
    await env.RATE_LIMIT.put(nonceKey, '1', { expirationTtl: NONCE_TTL });
    // 会话配额（unknown 会话取压低的 QUOTA_UNKNOWN）
    const quotaMax = sess.binding_mode === 'none' ? QUOTA_UNKNOWN : QUOTA;
    const qKey = `sessionq:${sid}`;
    const qStored = await env.RATE_LIMIT.get(qKey, { type: 'json' });
    const used = qStored && typeof qStored.count === 'number' ? qStored.count : 0;
    if (used >= quotaMax) return { ok: false, status: 429, message: '会话配额已用尽，请重新获取会话' };
    await env.RATE_LIMIT.put(qKey, JSON.stringify({ count: used + 1 }), { expirationTtl: SESSION_TTL });
    return { ok: true };
  } catch {
    // KV 运行时失败/超时 → fail-closed（禁冒泡到顶层 catch 变 500）
    return { ok: false, status: 503, message: '会话功能暂不可用' };
  }
}

/**
 * 由写入 Key 解析租户（表驱动为主 + env.API_KEY 直比兜底，见 tenant-isolation 规范）。
 * @returns {Promise<string|null>} 命中返回 tenant_id；不命中/env 空返回 null（调用方应 401）。
 * @throws 兜底计数器写失败（表缺失 no such table 等）时抛错，使 /sync 返非 200、不静默吞。
 */
async function resolveTenant(env, key) {
  const trimmedKey = (key || '').trim();
  // 空/缺失 Bearer 永不鉴权：即便 sha256('') 被误 seed 成 active 凭据，也直接拒绝（defense-in-depth）
  if (trimmedKey === '') return null;
  // env.API_KEY「空」统一判据（纯空白经 trim 成空串也算空）
  const apiKey = (env.API_KEY || '').trim();

  // 表驱动为主：sha256(trim(key)) 查 tenant_credentials（status='active'）
  const keyHash = await sha256(trimmedKey);
  const cred = await env.DB.prepare(
    "SELECT tenant_id FROM tenant_credentials WHERE key_hash = ? AND status = 'active' LIMIT 1"
  )
    .bind(keyHash)
    .first();
  if (cred && cred.tenant_id) {
    return cred.tenant_id;
  }

  // env.API_KEY 直比兜底（过渡期）：未命中且 trim(key)===trim(env.API_KEY) 且 env.API_KEY 非空
  if (apiKey !== '' && trimmedKey === apiKey) {
    // 递增 D1 计数行（禁用 KV，KV fail-open 会吞递增致假绿）；
    // 此 UPDATE 是独立 .run()、不进 /sync 数据 batch。
    const result = await env.DB.prepare(
      "UPDATE service_counters SET count = count + 1 WHERE name = 'auth_fallback'"
    ).run();
    // 写失败判据：result.meta.changes === 0（行缺失，漏 seed）或上面 .run() 抛错（表缺失）→ 视为写失败
    if (!result || !result.meta || result.meta.changes === 0) {
      throw new Error('auth_fallback counter row missing');
    }
    return 'bh2ro';
  }

  // env.API_KEY 未配置或仍不命中 → 不命中（调用方 401）；
  // 禁止沿用「env.API_KEY 空即放行」。
  return null;
}

export default {
  async fetch(request, env, ctx) {
    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: CORS_HEADERS });
    }

    const url = new URL(request.url);
    const path = url.pathname.replace(/\/$/, '') || '/';
    const method = request.method;

    try {
      // GET /ping
      if (path === '/ping' && method === 'GET') {
        const token = getBearerToken(request);
        // 仅在 env.API_KEY 侧补 trim（token 侧 getBearerToken 已 trim）；
        // env.API_KEY 空（含纯空白经 trim 后为空）时返回 401，与 /sync 一致，删除原 fail-open。
        const apiKey = (env.API_KEY || '').trim();
        if (apiKey === '' || token !== apiKey) {
          return json({ success: false, message: 'API Key 无效' }, 401);
        }
        return json({
          success: true,
          message: 'pong',
          server_time: serverTime(),
        });
      }

      // POST /sync
      if (path === '/sync' && method === 'POST') {
        const token = getBearerToken(request);
        // 鉴权统一由 resolveTenant 命中决定（删除原 token!==env.API_KEY 前置门，
        // 否则多 Key→同租户的表驱动凭据会被旧门 401 架空）。
        const tenant_id = await resolveTenant(env, token);
        if (!tenant_id) {
          return json({ success: false, message: '认证失败，请检查 API Key' }, 401);
        }
        let body;
        try {
          body = await request.json();
        } catch {
          return json({ success: false, message: '请求体不是有效 JSON' }, 400);
        }
        if (!body || typeof body !== 'object' || Array.isArray(body)) return json({ success: false, message: '请求体格式不正确' }, 400);
        const sync_time = body.sync_time;
        const data = body.data;
        // 保留 client_id 存在性校验作请求形态契约（缺失 400）；它只写入 sync_meta.last_client_id
        // 溯源、不参与数据归属（归属仅由 Key 解析）。
        if (!body.client_id || !data) {
          return json({ success: false, message: '缺少 client_id 或 data' }, 400);
        }
        // client_id 是客户端可控字段，落 last_client_id 前长度归一 ≤128（超长截断，不拒绝），
        // 防超长串污染溯源列。
        const last_client_id = String(body.client_id).slice(0, 128);

        // base_version 严格判整数（禁 parseInt：防 "5abc" 部分解析、"5"/5.7/true 歧义）；
        // 非整数视为「未携带」→ 降级走无条件路径。force 仅布尔 true 生效。
        const base_version = Number.isInteger(body.base_version) ? body.base_version : null;
        const force = body.force === true;
        // 守卫路径 = 携带有效 base_version 且非 force；否则走无条件路径（force 或缺 base_version）。
        const guarded = base_version !== null && !force;

        const DB = env.DB;
        const received_at = serverTime();

        const projects = data.projects || [];
        const cards = data.cards || [];
        const sf_senders = data.sf_senders || [];
        const sf_orders = data.sf_orders || [];
        const app_settings = data.app_settings || [];

        // 每行 INSERT 的列值数组（不含 tenant_id 占位顺序差异，按路径分别拼 SQL）。
        // tenant_id 入库注入服务端解析值；JSON/布尔形态归一与原逻辑一致。
        const rowProjects = projects.map((p) => [tenant_id, p.id, p.name, p.created_at || received_at, p.updated_at || received_at]);
        const rowCards = cards.map((c) => {
          const meta = c.metadata ? JSON.stringify(c.metadata) : null;
          const status = typeof c.status === 'string' ? c.status : (c.status && c.status.value) || 'pending';
          return [tenant_id, c.id, c.project_id, c.creator_id ?? null, c.callsign, c.qty, c.serial ?? null, status, meta, c.created_at || received_at, c.updated_at || received_at];
        });
        const rowSenders = sf_senders.map((s) => [tenant_id, s.id, s.name, s.phone, s.mobile ?? null, s.province, s.city, s.district, s.address, s.is_default ? 1 : 0, s.created_at || received_at, s.updated_at || received_at]);
        const rowOrders = sf_orders.map((o) => [
          tenant_id,
          o.id,
          o.order_id,
          o.waybill_no ?? null,
          o.card_id ?? null,
          o.status || 'pending',
          o.pay_method ?? 1,
          o.cargo_name ?? 'QSL卡片',
          typeof o.sender_info === 'object' ? JSON.stringify(o.sender_info) : (o.sender_info || '{}'),
          typeof o.recipient_info === 'object' ? JSON.stringify(o.recipient_info) : (o.recipient_info || '{}'),
          o.created_at || received_at,
          o.updated_at || received_at,
        ]);
        const rowSettings = app_settings.map((setting) => [tenant_id, setting.key, setting.value ?? '']);

        // 各业务表「列名清单 / 占位符模板」——占位符一律位置匿名 ?，.bind() 顺序绑定。
        const TABLE_INSERTS = [
          { table: 'projects', cols: 'tenant_id, id, name, created_at, updated_at', ph: '?,?,?,?,?', rows: rowProjects },
          { table: 'cards', cols: 'tenant_id, id, project_id, creator_id, callsign, qty, serial, status, metadata, created_at, updated_at', ph: '?,?,?,?,?,?,?,?,?,?,?', rows: rowCards },
          { table: 'sf_senders', cols: 'tenant_id, id, name, phone, mobile, province, city, district, address, is_default, created_at, updated_at', ph: '?,?,?,?,?,?,?,?,?,?,?,?', rows: rowSenders },
          { table: 'sf_orders', cols: 'tenant_id, id, order_id, waybill_no, card_id, status, pay_method, cargo_name, sender_info, recipient_info, created_at, updated_at', ph: '?,?,?,?,?,?,?,?,?,?,?,?', rows: rowOrders },
          { table: 'app_settings', cols: 'tenant_id, key, value', ph: '?,?,?', rows: rowSettings },
        ];
        const TABLES = ['projects', 'cards', 'sf_senders', 'sf_orders', 'app_settings'];

        const stats = {
          projects: projects.length,
          cards: cards.length,
          sf_senders: sf_senders.length,
          sf_orders: sf_orders.length,
        };

        if (guarded) {
          // ── 守卫路径（OCC compare-and-swap）──────────────────────────────
          // 单 DB.batch：DELETE×5（带版本守卫）→ INSERT×N（INSERT…SELECT…WHERE 守卫）→ 末条 CAS。
          // 占位符一律位置匿名 ? + 顺序 .bind()，禁混用 ?1/?2。
          // 语句条数 = 6 + N（5 DELETE + N INSERT + 1 CAS）；409 分支再 +1 次 batch 后 SELECT；
          // + resolveTenant 1~2 次查询，远低于 D1 Paid 1000 上限（见 design 约束 5）。
          const stmts = [];
          for (const t of TABLES) {
            stmts.push(
              DB.prepare(
                `DELETE FROM ${t} WHERE tenant_id = ? AND (SELECT server_version FROM sync_meta WHERE tenant_id = ?) = ?`
              ).bind(tenant_id, tenant_id, base_version)
            );
          }
          for (const ti of TABLE_INSERTS) {
            for (const r of ti.rows) {
              stmts.push(
                DB.prepare(
                  `INSERT INTO ${ti.table} (${ti.cols}) SELECT ${ti.ph} WHERE (SELECT server_version FROM sync_meta WHERE tenant_id = ?) = ?`
                ).bind(...r, tenant_id, base_version)
              );
            }
          }
          // 末条：CAS 递增版本（放 batch 末尾，使前面所有守卫都看到原始 base_version）。
          const casIndex = stmts.length;
          stmts.push(
            DB.prepare(
              'UPDATE sync_meta SET server_version = server_version + 1, last_client_id = ?, sync_time = ?, received_at = ? WHERE tenant_id = ? AND server_version = ?'
            ).bind(last_client_id, sync_time || received_at, received_at, tenant_id, base_version)
          );

          const results = await DB.batch(stmts);
          // 按记录的 casIndex 取 CAS 那条结果（禁 results[results.length-1] 共用 helper）。
          const casChanges = results[casIndex]?.meta?.changes;
          if (casChanges === 1) {
            // 命中：版本确定为 base_version + 1，无需读回。
            return json({
              success: true,
              message: '同步成功',
              received_at,
              server_version: base_version + 1,
              stats,
            });
          }
          // 409：守卫保证零改动（DELETE/INSERT 守卫全落空、CAS changes==0）。
          // CAS 只给 changes、不给当前版本值，故补一次读取云端当前 server_version。
          // 行不存在时 .first() 返 null → row?.server_version ?? null（禁 row.server_version 抛错）。
          const cur = await DB.prepare('SELECT server_version FROM sync_meta WHERE tenant_id = ?').bind(tenant_id).first();
          return json({
            success: false,
            message: '云端数据已更新，本地基线已陈旧',
            server_version: cur?.server_version ?? null,
          }, 409);
        }

        // ── 无条件路径（force=true 或未携带 base_version）──────────────────
        // 单 DB.batch：DELETE×5（仅 WHERE tenant_id=?）→ INSERT×N（无守卫）→ upsert →
        // 末条读回 SELECT。语句条数 = 7 + N；永远成功、不判 409。
        const stmts = [];
        for (const t of TABLES) {
          stmts.push(DB.prepare(`DELETE FROM ${t} WHERE tenant_id = ?`).bind(tenant_id));
        }
        for (const ti of TABLE_INSERTS) {
          for (const r of ti.rows) {
            stmts.push(DB.prepare(`INSERT INTO ${ti.table} (${ti.cols}) VALUES (${ti.ph})`).bind(...r));
          }
        }
        // upsert：server_version 单调 +1（行不存在则建行置 1）。非数组末元素。
        stmts.push(
          DB.prepare(
            'INSERT INTO sync_meta (tenant_id, server_version, last_client_id, sync_time, received_at) VALUES (?,1,?,?,?) ' +
            'ON CONFLICT(tenant_id) DO UPDATE SET server_version = sync_meta.server_version + 1, ' +
            'last_client_id = excluded.last_client_id, sync_time = excluded.sync_time, received_at = excluded.received_at'
          ).bind(tenant_id, last_client_id, sync_time || received_at, received_at)
        );
        // 末条读回 SELECT：同事务 read-your-writes 取确定新版本，消除 batch 外再读的竞态。
        const readbackIndex = stmts.length;
        stmts.push(DB.prepare('SELECT server_version FROM sync_meta WHERE tenant_id = ?').bind(tenant_id));

        const results = await DB.batch(stmts);
        // 按记录的 readbackIndex 取读回 SELECT 结果（禁共用「读末元素」helper）。
        const newVersion = results[readbackIndex]?.results?.[0]?.server_version ?? null;

        return json({
          success: true,
          message: '同步成功',
          received_at,
          server_version: newVersion,
          stats,
        });
      }

      // GET /pull 按写入 Key 拉回该租户全量快照 + 当前 server_version
      if (path === '/pull' && method === 'GET') {
        const token = getBearerToken(request);
        const tenant_id = await resolveTenant(env, token);
        if (!tenant_id) {
          return json({ success: false, message: '认证失败，请检查 API Key' }, 401);
        }

        const DB = env.DB;
        // 6 条 SELECT（5 业务表显式业务列、排除 tenant_id；1 条 sync_meta），各计 1 次查询；
        // tenant_id 一律注入服务端解析值（WHERE tenant_id = ?），无字段取自请求参数。
        const [projRes, cardRes, senderRes, orderRes, settingRes, metaRow] = await DB.batch([
          DB.prepare('SELECT id, name, created_at, updated_at FROM projects WHERE tenant_id = ?').bind(tenant_id),
          DB.prepare('SELECT id, project_id, creator_id, callsign, qty, serial, status, metadata, created_at, updated_at FROM cards WHERE tenant_id = ?').bind(tenant_id),
          DB.prepare('SELECT id, name, phone, mobile, province, city, district, address, is_default, created_at, updated_at FROM sf_senders WHERE tenant_id = ?').bind(tenant_id),
          DB.prepare('SELECT id, order_id, waybill_no, card_id, status, pay_method, cargo_name, sender_info, recipient_info, created_at, updated_at FROM sf_orders WHERE tenant_id = ?').bind(tenant_id),
          DB.prepare('SELECT key, value FROM app_settings WHERE tenant_id = ?').bind(tenant_id),
          DB.prepare('SELECT server_version, last_client_id, sync_time FROM sync_meta WHERE tenant_id = ?').bind(tenant_id),
        ]);

        // 形态还原：metadata/sender_info/recipient_info 入库为 JSON 字符串、is_default 为整数；
        // 读出后逐字段还原为对象/布尔以匹配桌面端 export_database()。任一行 JSON.parse 抛错 →
        // 落顶层 catch → fail-closed 500（脱敏），禁静默跳过/返回半快照。
        const projectsOut = (projRes.results || []).map((r) => ({
          id: r.id, name: r.name, created_at: r.created_at, updated_at: r.updated_at,
        }));
        const cardsOut = (cardRes.results || []).map((r) => ({
          id: r.id,
          project_id: r.project_id,
          creator_id: r.creator_id ?? null,
          callsign: r.callsign,
          qty: r.qty,
          serial: r.serial ?? null,
          status: r.status,
          metadata: (r.metadata == null || r.metadata === '') ? null : JSON.parse(r.metadata),
          created_at: r.created_at,
          updated_at: r.updated_at,
        }));
        const sendersOut = (senderRes.results || []).map((r) => ({
          id: r.id,
          name: r.name,
          phone: r.phone,
          mobile: r.mobile ?? null,
          province: r.province,
          city: r.city,
          district: r.district,
          address: r.address,
          is_default: !!r.is_default,
          created_at: r.created_at,
          updated_at: r.updated_at,
        }));
        const ordersOut = (orderRes.results || []).map((r) => ({
          id: r.id,
          order_id: r.order_id,
          waybill_no: r.waybill_no ?? null,
          card_id: r.card_id ?? null,
          status: r.status,
          pay_method: r.pay_method,
          cargo_name: r.cargo_name,
          sender_info: r.sender_info ? JSON.parse(r.sender_info) : {},
          recipient_info: r.recipient_info ? JSON.parse(r.recipient_info) : {},
          created_at: r.created_at,
          updated_at: r.updated_at,
        }));
        const settingsOut = (settingRes.results || []).map((r) => ({ key: r.key, value: r.value }));

        return json({
          success: true,
          server_version: metaRow?.results?.[0]?.server_version ?? null,
          data: {
            projects: projectsOut,
            cards: cardsOut,
            sf_senders: sendersOut,
            sf_orders: ordersOut,
            app_settings: settingsOut,
          },
          last_client_id: metaRow?.results?.[0]?.last_client_id ?? null,
          sync_time: metaRow?.results?.[0]?.sync_time ?? null,
        });
      }

      // GET /api/session/challenge 下发 PoW 题（query-antibot-session）
      if (path === '/api/session/challenge' && method === 'GET') {
        if (!sessionSubsystemReady(env)) {
          return json({ success: false, message: '会话功能不可用' }, 503);
        }
        const bkey = clientBindingKey(getClientIP(request, env));
        try {
          const hs = await handshakeRateLimit(env, bkey); // 握手桶（fail-closed）
          if (!hs.allowed) {
            return json({ success: false, message: '请求过于频繁，请稍后再试' }, 429);
          }
          const difficulty = await computeDifficulty(env, bkey); // powrate 失败 fail-secure→DIFF_MAX
          const seed = randomHex(16);
          const record = {
            difficulty,
            challenge_ip: bkey, // 归一键（与兑换比对同口径）
            challenge_ua_hash: await uaHash(request.headers.get('User-Agent')),
            exp: Date.now() + POWSEED_TTL * 1000,
          };
          // 写 powseed 失败 → fail-closed（禁返回不可兑换的 seed）
          await env.RATE_LIMIT.put(`powseed:${seed}`, JSON.stringify(record), { expirationTtl: POWSEED_TTL });
          return json({ success: true, seed, difficulty });
        } catch {
          return json({ success: false, message: '会话功能暂不可用' }, 503);
        }
      }

      // POST /api/session 验 PoW → 签发短时会话（query-antibot-session）
      if (path === '/api/session' && method === 'POST') {
        if (!sessionSubsystemReady(env)) {
          return json({ success: false, message: '会话功能不可用' }, 503);
        }
        const bkey = clientBindingKey(getClientIP(request, env));
        let body;
        try { body = await request.json(); } catch { return json({ success: false, message: '无效 JSON' }, 400); }
        const seed = body && typeof body.seed === 'string' ? body.seed : null;
        const nonce = body && typeof body.nonce === 'string' ? body.nonce : null;
        if (!seed || !nonce) return json({ success: false, message: '缺少 seed/nonce' }, 400);
        try {
          const hs = await handshakeRateLimit(env, bkey); // 握手桶（fail-closed）
          if (!hs.allowed) return json({ success: false, message: '请求过于频繁，请稍后再试' }, 429);
          const rawSeed = await env.RATE_LIMIT.get(`powseed:${seed}`);
          // code='seed_not_found' 供客户端区分「KV 写后读窗导致 seed 暂不可见」→ 短退避重取 challenge 一次
          if (!rawSeed) return json({ success: false, message: '题目无效或已过期', code: 'seed_not_found' }, 403);
          await env.RATE_LIMIT.delete(`powseed:${seed}`); // 读后立即删（一次性、防并发重放）
          let rec;
          try { rec = JSON.parse(rawSeed); } catch { return json({ success: false, message: '题目无效' }, 403); }
          if (!(await verifyPow(seed, nonce, rec.difficulty))) {
            return json({ success: false, message: '工作量证明不足' }, 403);
          }
          // 领题者 === 兑换者（归一键 + UA），防低难度 IP 领题→他处兑换绕开自适应难度
          if (rec.challenge_ip !== bkey || rec.challenge_ua_hash !== (await uaHash(request.headers.get('User-Agent')))) {
            return json({ success: false, message: '会话上下文不一致' }, 403);
          }
          const isUnknown = bkey === 'unknown';
          const sid = randomHex(18);
          const sk = randomHex(32);
          const exp = Date.now() + SESSION_TTL_MS;
          const record = {
            binding_mode: isUnknown ? 'none' : 'ip',
            ip: isUnknown ? null : bkey,
            ua_hash: await uaHash(request.headers.get('User-Agent')),
            exp,
            sk,
          };
          await env.RATE_LIMIT.put(`session:${sid}`, JSON.stringify(record), { expirationTtl: SESSION_TTL });
          const token = await makeToken(sid, env.SESSION_SECRET);
          return json({ success: true, token, sk, exp, quota: isUnknown ? QUOTA_UNKNOWN : QUOTA });
        } catch {
          return json({ success: false, message: '会话功能暂不可用' }, 503);
        }
      }

      // GET /api/callsigns/:callsign 或 /api/query?callsign=
      const callsignMatch = path.match(/^\/api\/callsigns\/([^/]+)$/);
      const callsignFromPath = callsignMatch ? decodeURIComponent(callsignMatch[1]) : null;
      const callsignFromQuery = url.searchParams.get('callsign');
      const callsign = callsignFromPath || callsignFromQuery;

      if (callsign && method === 'GET' && (path.startsWith('/api/callsigns/') || path === '/api/query')) {
        const bkey = clientBindingKey(getClientIP(request, env));
        // Layer 0: 纯 IP 限流（查询桶 `ratelimit:<bkey>`，fail-open）——**置于会话校验之前**。
        // 顺序不变量：限流触发先返 429；fail-open 绝不在会话校验之前短路放行，会话校验(fail-closed)仍主导。
        const rateLimit = await checkRateLimit(env, bkey);
        if (!rateLimit.allowed) {
          return json({
            success: false,
            message: '请求过于频繁，请稍后再试',
            retry_after: rateLimit.resetAt - Math.floor(Date.now() / 1000),
          }, 429);
        }

        // Layer 1: 会话校验（token + 会话签名 + 配额，fail-closed）取代静态 CLIENT_SIGN_KEY 签名
        const sessionCheck = await validateQuerySession(env, request, url, bkey);
        if (!sessionCheck.ok) {
          return json({ success: false, message: sessionCheck.message }, sessionCheck.status);
        }

        const DB = env.DB;
        // 读取侧按服务端确定的 tenant_id 过滤（本期恒为常量 'bh2ro'，host/path 路由属阶段 4）；
        // tenant_id 禁止取自前端参数。projects join 同时按 tenant_id 匹配，保留 LEFT JOIN 与 COLLATE NOCASE。
        const tenant_id = 'bh2ro';
        const rows = await DB.prepare(
          `SELECT c.id, c.project_id, c.callsign, c.qty, c.serial, c.status, c.metadata, c.created_at, c.updated_at, p.name AS project_name
           FROM cards c
           LEFT JOIN projects p ON p.tenant_id = c.tenant_id AND p.id = c.project_id
           WHERE c.tenant_id = ? AND c.callsign = ? COLLATE NOCASE
           ORDER BY c.created_at DESC`
        )
          .bind(tenant_id, callsign.toUpperCase())
          .all();

        const items = (rows.results || []).map((r) => {
          const metadata = r.metadata ? (typeof r.metadata === 'string' ? JSON.parse(r.metadata) : r.metadata) : null;
          const dist = metadata?.distribution;
          const ret = metadata?.return;
          return {
            id: r.id,
            project_name: r.project_name || null,
            status: r.status,
            distribution: dist ? {
              method: dist.method || null,
              proxy_callsign: dist.proxy_callsign || null,
              remarks: dist.remarks || null,
            } : null,
            return: ret ? {
              method: ret.method || null,
              remarks: ret.remarks || null,
            } : null,
          };
        });

        return json({ success: true, callsign: callsign.toUpperCase(), items });
      }

      // POST /api/sf/route-push 或 /api/sf/route-push/sandbox 顺丰路由推送（JSON）：先返回 0000，再异步落库与微信推送；沙箱路径在用户推送中标记「【沙箱】」
      const isSandboxRoute = path === '/api/sf/route-push/sandbox';
      const isProdRoute = path === '/api/sf/route-push';
      if ((isSandboxRoute || isProdRoute) && method === 'POST') {
        let body;
        try {
          body = await request.json();
        } catch {
          return json({ return_code: '1000', return_msg: '无效 JSON' }, 200);
        }
        const waybillRoute = body.Body?.WaybillRoute;
        if (!Array.isArray(waybillRoute) || waybillRoute.length === 0) {
          return json({ return_code: '0000', return_msg: '成功' }, 200);
        }
        const isSandbox = isSandboxRoute;
        ctx.waitUntil(
          (async () => {
            const DB = env.DB;
            // 记录新插入的路由（用于后续微信推送）
            const newRoutes = [];

            // 第一步：所有路由写入数据库（去重）
            for (const r of waybillRoute) {
              try {
                const id = r.id || `${r.mailno}-${r.opCode}-${Date.now()}`;
                const existing = await DB.prepare('SELECT 1 FROM sf_route_log WHERE id = ?').bind(id).first();
                if (!existing) {
                  await DB.prepare(
                    'INSERT INTO sf_route_log (id, mailno, orderid, op_code, accept_time, remark) VALUES (?,?,?,?,?,?)'
                  )
                    .bind(
                      id,
                      r.mailno ?? null,
                      r.orderid ?? null,
                      r.opCode ?? null,
                      r.acceptTime ?? null,
                      r.remark ?? null
                    )
                    .run();
                  // 新记录，加入待推送列表
                  newRoutes.push(r);
                }
              } catch (e) {
                console.error('SF route log insert failed', e);
              }
            }

            // 第二步：按运单号分组，每个运单只推送最新节点
            const latestByMailno = new Map();
            for (const r of newRoutes) {
              const mailno = r.mailno;
              if (!mailno) continue;
              const existing = latestByMailno.get(mailno);
              if (!existing || (r.acceptTime && (!existing.acceptTime || r.acceptTime > existing.acceptTime))) {
                latestByMailno.set(mailno, r);
              }
            }

            // 第三步：对每个运单的最新节点发送微信推送
            for (const r of latestByMailno.values()) {
              try {
                const orderid = r.orderid;
                const mailno = r.mailno;
                let callsign = null;
                // route-push 是无凭据公开端点、无租户上下文，本期注入服务端常量 bh2ro；
                // 按 order 派生确定租户属阶段 4。保留业务连接键 o.card_id = c.id（漏则退化笛卡尔积错号），
                // 仅把隔离键 o.client_id=c.client_id 换为 o.tenant_id = c.tenant_id，并加 WHERE o.tenant_id = ?。
                const tenant_id = 'bh2ro';
                if (orderid) {
                  const row = await DB.prepare(
                    'SELECT c.callsign FROM sf_orders o JOIN cards c ON o.tenant_id = c.tenant_id AND o.card_id = c.id WHERE o.tenant_id = ? AND o.order_id = ? LIMIT 1'
                  )
                    .bind(tenant_id, orderid)
                    .first();
                  if (row) callsign = row.callsign;
                }
                if (!callsign && mailno) {
                  const row = await DB.prepare(
                    'SELECT c.callsign FROM sf_orders o JOIN cards c ON o.tenant_id = c.tenant_id AND o.card_id = c.id WHERE o.tenant_id = ? AND o.waybill_no = ? LIMIT 1'
                  )
                    .bind(tenant_id, mailno)
                    .first();
                  if (row) callsign = row.callsign;
                }
                // 仅在微信推送完整配置时（APPID + SECRET + TEMPLATE_ID）才尝试发送
                if (callsign && env.WECHAT_APPID && env.WECHAT_SECRET && env.WECHAT_TEMPLATE_ID) {
                  const openids = await DB.prepare('SELECT openid FROM callsign_openid_bindings WHERE callsign = ?')
                    .bind(callsign)
                    .all();
                  for (const row of openids.results || []) {
                    try {
                      await sendWechatTemplate(env, callsign, mailno, r.remark, r.acceptTime, row.openid, isSandbox);
                    } catch (e) {
                      console.error('WeChat send failed', e);
                    }
                  }
                }
              } catch (e) {
                console.error('SF route push process failed', e);
              }
            }
          })()
        );
        return json({ return_code: '0000', return_msg: '成功' }, 200);
      }

      // GET /api/wechat/auth-callback 微信网页授权回调（订阅收卡）：code + state(callsign) -> 换 openid -> 写入绑定表
      if (path === '/api/wechat/auth-callback' && method === 'GET') {
        // IP 限流：独立计数桶 authcb（不与查询共桶、互不挤占预算），
        // 计数 IP 取自可信真实客户端 IP 解析（见 client-ip.js / trusted-client-ip 规范）。
        // checkRateLimit 在 RATE_LIMIT KV 未配置时 fail-open（可用性优先）→ KV 为部署前置。
        // 订阅回调（OAuth 被动跳转）属本阶段非目标：限流键沿用真实 IP（getClientIP），
        // 不做 clientBindingKey 的 /64 归一——避免同 /64 内不相关订阅者共享 authcb 桶（保持订阅路径原状）。
        const authcbIP = getClientIP(request, env);
        const authcbRate = await checkRateLimit(env, authcbIP, 'authcb');
        if (!authcbRate.allowed) {
          return new Response('请求过于频繁，请稍后再试', { status: 429, headers: CORS_HEADERS });
        }
        const code = url.searchParams.get('code');
        const state = url.searchParams.get('state'); // callsign
        if (!code || !state) {
          return new Response('缺少 code 或 state（呼号）', { status: 400, headers: CORS_HEADERS });
        }
        const callsign = decodeURIComponent(state).toUpperCase();
        if (!env.WECHAT_APPID || !env.WECHAT_SECRET) {
          return new Response('未配置微信服务号', { status: 503, headers: CORS_HEADERS });
        }
        const tokenUrl = `https://api.weixin.qq.com/sns/oauth2/access_token?appid=${env.WECHAT_APPID}&secret=${env.WECHAT_SECRET}&code=${code}&grant_type=authorization_code`;
        const tokenRes = await fetch(tokenUrl);
        const tokenData = await tokenRes.json();
        const openid = tokenData.openid;
        if (!openid) {
          // 脱敏：不回显上游原始 errcode/errmsg 或序列化响应体，仅落服务端日志。
          console.error('WeChat oauth2 access_token failed', tokenData);
          return new Response('微信授权失败', {
            status: 400,
            headers: CORS_HEADERS,
          });
        }
        await env.DB.prepare(
          'INSERT OR IGNORE INTO callsign_openid_bindings (callsign, openid, created_at) VALUES (?,?,?)'
        )
          .bind(callsign, openid, new Date().toISOString())
          .run();
        const html = `<!DOCTYPE html><html><head><meta charset="UTF-8"/><title>订阅成功</title></head><body><p>订阅收卡成功！呼号 ${callsign} 已与您的微信绑定，后续该呼号的卡片分发与物流动态将推送至微信。</p></body></html>`;
        return new Response(html, { headers: { 'Content-Type': 'text/html; charset=UTF-8', ...CORS_HEADERS } });
      }

      // GET /api/config 前端配置（功能开关 + WECHAT_APPID + 备案；不再下发查询签名密钥/验证码配置）
      if (path === '/api/config' && method === 'GET') {
        const wechatSubscribeEnabled = !!(env.WECHAT_APPID && env.WECHAT_SECRET);
        const wechatPushEnabled = !!(env.WECHAT_APPID && env.WECHAT_SECRET && env.WECHAT_TEMPLATE_ID);
        let filing = null;
        if (env.SITE_FILING) {
          try { filing = JSON.parse(env.SITE_FILING); } catch { /* ignore invalid JSON */ }
        }
        // 退役：禁止下发 sign_key（静态 CLIENT_SIGN_KEY 退役）与 captcha 相关配置——查询签名改用会话专属 sk。
        return json({
          features: {
            wechat_subscribe: wechatSubscribeEnabled,
            wechat_push: wechatPushEnabled,
          },
          wechat_appid: wechatSubscribeEnabled ? env.WECHAT_APPID : null,
          filing,
        });
      }

      // 其他路径交由静态资源处理（Cloudflare Assets）
      // 如果是 SPA 路由（非 API、非静态资源扩展名），返回 env.ASSETS 中的 index.html
      if (env.ASSETS) {
        // API 路径已在上方处理，其余交给 Assets
        const assetResponse = await env.ASSETS.fetch(request);
        // 如果是 404 且路径不含扩展名，返回 index.html 以支持 SPA
        if (assetResponse.status === 404 && !path.includes('.')) {
          const indexRequest = new Request(new URL('/index.html', url.origin), request);
          return env.ASSETS.fetch(indexRequest);
        }
        return assetResponse;
      }

      return json({ success: false, message: 'Not Found' }, 404);
    } catch (e) {
      // 脱敏：不回显原始异常消息/内部实现细节，详细异常仅落服务端日志。
      console.error(e);
      return json({ success: false, message: '服务器错误' }, 500);
    }
  },
};

async function sendWechatTemplate(env, callsign, waybillNo, remark, acceptTime, openid, isSandbox = false) {
  if (!env.WECHAT_APPID || !env.WECHAT_SECRET || !env.WECHAT_TEMPLATE_ID) return;
  const tokenUrl = `https://api.weixin.qq.com/cgi-bin/token?grant_type=client_credential&appid=${env.WECHAT_APPID}&secret=${env.WECHAT_SECRET}`;
  const tokenRes = await fetch(tokenUrl);
  const tokenData = await tokenRes.json();
  const accessToken = tokenData.access_token;
  if (!accessToken) return;
  const url = `https://api.weixin.qq.com/cgi-bin/message/template/send?access_token=${accessToken}`;
  const sandboxPrefix = isSandbox ? '【沙箱】' : '';
  const body = {
    touser: openid,
    template_id: env.WECHAT_TEMPLATE_ID,
    data: {
      first: { value: `${sandboxPrefix}您的 QSL 卡片物流更新（呼号 ${callsign}）` },
      keyword1: { value: waybillNo || '-' },
      keyword2: { value: remark || '-' },
      keyword3: { value: acceptTime || '-' },
      remark: { value: '来自 QSL CardHub' },
    },
  };
  await fetch(url, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
}
