/**
 * QSL CardHub 云端查询与同步服务
 * - GET /ping  连接测试（Bearer API_KEY）
 * - POST /sync 全量同步（Bearer API_KEY）
 * - GET /api/callsigns/:callsign 或 /api/query?callsign= 按呼号查询收卡
 * - POST /api/sf/route-push 顺丰路由推送接收（JSON）
 * - GET /query 按呼号查询页面（含订阅收卡入口）
 */

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
 * 获取客户端 IP
 */
function getClientIP(request) {
  return request.headers.get('CF-Connecting-IP') ||
    request.headers.get('X-Forwarded-For')?.split(',')[0]?.trim() ||
    'unknown';
}

/**
 * IP 限流检查
 * @returns {Promise<{allowed: boolean, remaining: number, resetAt: number}>}
 */
async function checkRateLimit(env, ip) {
  if (!env.RATE_LIMIT) {
    // KV 未配置时跳过限流
    return { allowed: true, remaining: RATE_LIMIT_MAX, resetAt: 0 };
  }

  const key = `ratelimit:${ip}`;
  const now = Math.floor(Date.now() / 1000);
  const windowStart = now - (now % RATE_LIMIT_WINDOW);
  const resetAt = windowStart + RATE_LIMIT_WINDOW;

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

/**
 * 验证请求签名
 * @returns {Promise<{valid: boolean, error?: string}>}
 */
async function verifySignature(env, request, url) {
  if (!env.CLIENT_SIGN_KEY) {
    // 未配置签名密钥时跳过验证
    return { valid: true };
  }

  const ts = url.searchParams.get('_ts');
  const nonce = url.searchParams.get('_nonce');
  const sig = url.searchParams.get('_sig');

  if (!ts || !nonce || !sig) {
    return { valid: false, error: '缺少签名参数' };
  }

  // 1. 时间窗口检查
  const timestamp = parseInt(ts, 10);
  const now = Date.now();
  if (isNaN(timestamp) || Math.abs(now - timestamp) > SIGN_TIME_WINDOW) {
    return { valid: false, error: '签名已过期' };
  }

  // 2. nonce 唯一性检查（防重放）
  if (env.RATE_LIMIT) {
    const nonceKey = `nonce:${nonce}`;
    const existing = await env.RATE_LIMIT.get(nonceKey);
    if (existing) {
      return { valid: false, error: '请求已处理' };
    }
    await env.RATE_LIMIT.put(nonceKey, '1', { expirationTtl: NONCE_TTL });
  }

  // 3. 签名正确性校验
  // 获取主要参数（排除签名相关参数）
  const params = new URLSearchParams(url.searchParams);
  params.delete('_ts');
  params.delete('_nonce');
  params.delete('_sig');
  params.sort(); // 按字母排序确保一致性

  const path = url.pathname;
  const paramStr = params.toString();
  const payload = `${path}:${paramStr}:${ts}:${nonce}`;
  const expectedSig = await sha256(payload + env.CLIENT_SIGN_KEY);

  if (sig !== expectedSig) {
    return { valid: false, error: '签名无效' };
  }

  return { valid: true };
}

// ============================================================
// 验证码配置
// ============================================================
const CAPTCHA_TTL = 5 * 60 * 1000; // 验证码有效期 5 分钟

/**
 * 生成 HMAC-SHA256 签名
 */
async function hmacSha256(message, secret) {
  const encoder = new TextEncoder();
  const keyData = encoder.encode(secret);
  const key = await crypto.subtle.importKey(
    'raw',
    keyData,
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  );
  const signature = await crypto.subtle.sign('HMAC', key, encoder.encode(message));
  const hashArray = Array.from(new Uint8Array(signature));
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
}

/**
 * 生成算术验证码
 * @returns {{question: string, answer: number, token: string, expires: number}}
 */
async function generateCaptcha(env) {
  // 生成随机算术题
  const operators = ['+', '-'];
  const operator = operators[Math.floor(Math.random() * operators.length)];
  let num1, num2, answer;

  if (operator === '+') {
    num1 = Math.floor(Math.random() * 50) + 1; // 1-50
    num2 = Math.floor(Math.random() * 50) + 1; // 1-50
    answer = num1 + num2;
  } else {
    num1 = Math.floor(Math.random() * 50) + 20; // 20-70
    num2 = Math.floor(Math.random() * 20) + 1; // 1-20
    answer = num1 - num2;
  }

  const question = `${num1} ${operator} ${num2} = ?`;
  const expires = Date.now() + CAPTCHA_TTL;

  // 生成加密 token
  const payload = `${answer}:${expires}`;
  const signature = await hmacSha256(payload, env.CAPTCHA_SECRET || 'default-secret');
  const token = btoa(JSON.stringify({ a: answer, e: expires, s: signature }));

  return { question, answer, token, expires };
}

/**
 * 校验验证码
 * @returns {{valid: boolean, error?: string}}
 */
async function verifyCaptcha(env, token, userAnswer) {
  if (!env.CAPTCHA_SECRET) {
    // 未配置验证码密钥时跳过验证
    return { valid: true };
  }

  if (!token || userAnswer === undefined || userAnswer === null) {
    return { valid: false, error: '缺少验证码参数' };
  }

  try {
    const decoded = JSON.parse(atob(token));
    const { a: answer, e: expires, s: signature } = decoded;

    // 1. 检查是否过期
    if (Date.now() > expires) {
      return { valid: false, error: '验证码已过期' };
    }

    // 2. 验证签名
    const payload = `${answer}:${expires}`;
    const expectedSig = await hmacSha256(payload, env.CAPTCHA_SECRET);
    if (signature !== expectedSig) {
      return { valid: false, error: '验证码无效' };
    }

    // 3. 验证答案
    if (parseInt(userAnswer, 10) !== answer) {
      return { valid: false, error: '验证码答案错误' };
    }

    return { valid: true };
  } catch {
    return { valid: false, error: '验证码格式错误' };
  }
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
        if (env.API_KEY && token !== env.API_KEY) {
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
        if (env.API_KEY && token !== env.API_KEY) {
          return json({ success: false, message: '认证失败，请检查 API Key' }, 401);
        }
        let body;
        try {
          body = await request.json();
        } catch {
          return json({ success: false, message: '请求体不是有效 JSON' }, 400);
        }
        const client_id = body.client_id;
        const sync_time = body.sync_time;
        const data = body.data;
        if (!client_id || !data) {
          return json({ success: false, message: '缺少 client_id 或 data' }, 400);
        }

        const DB = env.DB;
        const received_at = serverTime();

        await DB.batch([
          DB.prepare('DELETE FROM projects WHERE client_id = ?').bind(client_id),
          DB.prepare('DELETE FROM cards WHERE client_id = ?').bind(client_id),
          DB.prepare('DELETE FROM sf_senders WHERE client_id = ?').bind(client_id),
          DB.prepare('DELETE FROM sf_orders WHERE client_id = ?').bind(client_id),
        ]);

        const projects = data.projects || [];
        const cards = data.cards || [];
        const sf_senders = data.sf_senders || [];
        const sf_orders = data.sf_orders || [];

        for (const p of projects) {
          await DB.prepare(
            'INSERT INTO projects (client_id, id, name, created_at, updated_at) VALUES (?,?,?,?,?)'
          )
            .bind(client_id, p.id, p.name, p.created_at || received_at, p.updated_at || received_at)
            .run();
        }
        for (const c of cards) {
          const meta = c.metadata ? JSON.stringify(c.metadata) : null;
          const status = typeof c.status === 'string' ? c.status : (c.status && c.status.value) || 'pending';
          await DB.prepare(
            'INSERT INTO cards (client_id, id, project_id, creator_id, callsign, qty, serial, status, metadata, created_at, updated_at) VALUES (?,?,?,?,?,?,?,?,?,?,?)'
          )
            .bind(
              client_id,
              c.id,
              c.project_id,
              c.creator_id ?? null,
              c.callsign,
              c.qty,
              c.serial ?? null,
              status,
              meta,
              c.created_at || received_at,
              c.updated_at || received_at
            )
            .run();
        }
        for (const s of sf_senders) {
          await DB.prepare(
            'INSERT INTO sf_senders (client_id, id, name, phone, mobile, province, city, district, address, is_default, created_at, updated_at) VALUES (?,?,?,?,?,?,?,?,?,?,?,?)'
          )
            .bind(
              client_id,
              s.id,
              s.name,
              s.phone,
              s.mobile ?? null,
              s.province,
              s.city,
              s.district,
              s.address,
              s.is_default ? 1 : 0,
              s.created_at || received_at,
              s.updated_at || received_at
            )
            .run();
        }
        for (const o of sf_orders) {
          await DB.prepare(
            'INSERT INTO sf_orders (client_id, id, order_id, waybill_no, card_id, status, pay_method, cargo_name, sender_info, recipient_info, created_at, updated_at) VALUES (?,?,?,?,?,?,?,?,?,?,?,?)'
          )
            .bind(
              client_id,
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
              o.updated_at || received_at
            )
            .run();
        }

        await DB.prepare(
          'INSERT OR REPLACE INTO sync_meta (client_id, sync_time, received_at) VALUES (?,?,?)'
        )
          .bind(client_id, sync_time || received_at, received_at)
          .run();

        return json({
          success: true,
          message: '同步成功',
          received_at,
          stats: {
            projects: projects.length,
            cards: cards.length,
            sf_senders: sf_senders.length,
            sf_orders: sf_orders.length,
          },
        });
      }

      // GET /api/captcha 生成算术验证码
      if (path === '/api/captcha' && method === 'GET') {
        // 应用 Layer 0: IP 限流
        const clientIP = getClientIP(request);
        const rateLimit = await checkRateLimit(env, clientIP);
        if (!rateLimit.allowed) {
          return json({
            success: false,
            message: '请求过于频繁，请稍后再试',
            retry_after: rateLimit.resetAt - Math.floor(Date.now() / 1000),
          }, 429);
        }

        if (!env.CAPTCHA_SECRET) {
          return json({ success: false, message: '验证码功能未启用' }, 503);
        }
        const captcha = await generateCaptcha(env);
        return json({
          success: true,
          question: captcha.question,
          token: captcha.token,
          expires: captcha.expires,
        });
      }

      // GET /api/callsigns/:callsign 或 /api/query?callsign=
      const callsignMatch = path.match(/^\/api\/callsigns\/([^/]+)$/);
      const callsignFromPath = callsignMatch ? decodeURIComponent(callsignMatch[1]) : null;
      const callsignFromQuery = url.searchParams.get('callsign');
      const callsign = callsignFromPath || callsignFromQuery;

      if (callsign && method === 'GET' && (path.startsWith('/api/callsigns/') || path === '/api/query')) {
        // 应用 Layer 0: IP 限流
        const clientIP = getClientIP(request);
        const rateLimit = await checkRateLimit(env, clientIP);
        if (!rateLimit.allowed) {
          return json({
            success: false,
            message: '请求过于频繁，请稍后再试',
            retry_after: rateLimit.resetAt - Math.floor(Date.now() / 1000),
          }, 429);
        }

        // 应用 Layer 1: 签名校验
        const sigResult = await verifySignature(env, request, url);
        if (!sigResult.valid) {
          return json({ success: false, message: sigResult.error || '签名验证失败' }, 403);
        }

        const DB = env.DB;
        const rows = await DB.prepare(
          `SELECT c.id, c.project_id, c.callsign, c.qty, c.serial, c.status, c.metadata, c.created_at, c.updated_at, p.name AS project_name
           FROM cards c
           LEFT JOIN projects p ON p.client_id = c.client_id AND p.id = c.project_id
           WHERE c.callsign = ? COLLATE NOCASE
           ORDER BY c.created_at DESC`
        )
          .bind(callsign.toUpperCase())
          .all();

        const items = (rows.results || []).map((r) => {
          const metadata = r.metadata ? (typeof r.metadata === 'string' ? JSON.parse(r.metadata) : r.metadata) : null;
          const dist = metadata?.distribution;
          const ret = metadata?.return_info;
          return {
            id: r.id,
            project_name: r.project_name || null,
            status: r.status,
            distribution: dist ? {
              method: dist.method || null,
              proxy_callsign: dist.proxy_callsign || null,
              remarks: dist.remarks || null,
            } : null,
            return_info: ret ? {
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
                if (orderid) {
                  const row = await DB.prepare(
                    'SELECT c.callsign FROM sf_orders o JOIN cards c ON o.client_id = c.client_id AND o.card_id = c.id WHERE o.order_id = ? LIMIT 1'
                  )
                    .bind(orderid)
                    .first();
                  if (row) callsign = row.callsign;
                }
                if (!callsign && mailno) {
                  const row = await DB.prepare(
                    'SELECT c.callsign FROM sf_orders o JOIN cards c ON o.client_id = c.client_id AND o.card_id = c.id WHERE o.waybill_no = ? LIMIT 1'
                  )
                    .bind(mailno)
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
          return new Response('微信授权失败：' + (tokenData.errmsg || JSON.stringify(tokenData)), {
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

      // GET /api/config 前端配置（包含功能开关、签名密钥和 WECHAT_APPID）
      if (path === '/api/config' && method === 'GET') {
        const wechatSubscribeEnabled = !!(env.WECHAT_APPID && env.WECHAT_SECRET);
        const wechatPushEnabled = !!(env.WECHAT_APPID && env.WECHAT_SECRET && env.WECHAT_TEMPLATE_ID);
        const captchaEnabled = !!(env.CLIENT_SIGN_KEY && env.CAPTCHA_SECRET);
        let filing = null;
        if (env.SITE_FILING) {
          try { filing = JSON.parse(env.SITE_FILING); } catch { /* ignore invalid JSON */ }
        }
        return json({
          features: {
            wechat_subscribe: wechatSubscribeEnabled,
            wechat_push: wechatPushEnabled,
            captcha: captchaEnabled,
          },
          wechat_appid: wechatSubscribeEnabled ? env.WECHAT_APPID : null,
          sign_key: env.CLIENT_SIGN_KEY || null,
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
      console.error(e);
      return json({ success: false, message: e.message || '服务器错误' }, 500);
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
