/**
 * 可信真实客户端 IP 解析（trusted-client-ip 规范）
 *
 * 双入口架构感知：
 * - 直连 Cloudflare 源站：`CF-Connecting-IP` 即真实用户 IP（CF 边缘按 TCP 层对端写入、覆盖客户端同名头）。
 * - 经阿里云 CDN 回源：`CF-Connecting-IP` 是 CDN 回源节点 IP；真实用户 IP 由 CDN 注入受信头（`Ali-Cdn-Real-Ip`，默认携带）。
 *
 * 信任信号 = **CDN 注入的密钥回源头 `X-Origin-Auth`**（非 IP 白名单）。阿里云回源 IP 动态分配、
 * 官方不建议固定白名单（见 docs/web-query-service-deploy.md 与阿里云文档），故改用密钥头判定「确来自 CDN」：
 * 阿里云 CDN 用「修改出站请求头」以覆盖语义注入固定密钥 `X-Origin-Auth: <CDN_ORIGIN_SECRET>`，
 * 客户端伪造的同名头在回源时被覆盖、攻击者也猜不到密钥。仅当密钥校验通过才采信 `Ali-Cdn-Real-Ip`。
 *
 * 该解析产物**仅**作限流/防爬计数键，禁止用作访问控制/鉴权。
 *
 * 本模块导出命名纯函数供 index.js 与单测 import。
 */

// 阿里云 CDN 注入的密钥回源头名（与阿里云「修改出站请求头」配置一致）。
const ORIGIN_AUTH_HEADER = 'X-Origin-Auth';

// ============================================================
// IP 字面量校验（仅用于校验受信真实 IP 头的值，不参与任何 CIDR 匹配）
// ============================================================

/**
 * 将点分十进制 IPv4 字符串转为 32 位无符号整数；非法（段数≠4、非整数、>255、负、前导零、空段）→ null。
 * 仅用于 IPv4 字面量合法性校验。
 * @param {string} ip
 * @returns {number|null}
 */
function ipv4ToUint32(ip) {
  if (typeof ip !== 'string') return null;
  const parts = ip.split('.');
  if (parts.length !== 4) return null;
  let acc = 0;
  for (const part of parts) {
    // 仅允许纯十进制；禁止前导零（'01'）、空串、符号、非数字。'0' 单字符合法。
    if (!/^(0|[1-9][0-9]{0,2})$/.test(part)) return null;
    const n = Number(part);
    if (n > 255) return null;
    acc = (acc * 256 + n) >>> 0;
  }
  return acc >>> 0;
}

/**
 * 判定一个字符串是否为合法 IPv6 字面量（字面量校验）。支持完整/压缩（::）形式、IPv4-mapped 末段；不接受 zone-id（含 '%'）。
 * @param {string} value
 * @returns {boolean}
 */
function isValidIPv6(value) {
  if (typeof value !== 'string') return false;
  if (value.length === 0) return false;
  if (value.includes('%')) return false; // 拒绝 zone-id
  if (!value.includes(':')) return false; // 至少含一个冒号
  if (value.includes(':::')) return false; // 拒绝 3+ 连续冒号
  // 边界为孤立单冒号（非 '::'）一律拒：':1:2...'、'1:2...:' 等畸形串。
  if (value.startsWith(':') && !value.startsWith('::')) return false;
  if (value.endsWith(':') && !value.endsWith('::')) return false;
  const doubleColonCount = (value.match(/::/g) || []).length;
  if (doubleColonCount > 1) return false; // '::' 至多一次

  const hasDoubleColon = value.includes('::');
  let head = value;
  let embeddedIPv4 = false;
  const lastColon = value.lastIndexOf(':');
  const tail = value.slice(lastColon + 1);
  if (tail.includes('.')) {
    if (ipv4ToUint32(tail) === null) return false; // 内嵌 IPv4 末段须合法
    embeddedIPv4 = true;
    head = value.slice(0, lastColon + 1);
  }

  const groups = head.split(':');
  for (const g of groups) {
    if (g === '') continue; // 来自 '::' 或边界冒号
    if (!/^[0-9a-fA-F]{1,4}$/.test(g)) return false;
  }

  const hexGroups = groups.filter((g) => g !== '').length;
  const total = hexGroups + (embeddedIPv4 ? 2 : 0);
  if (hasDoubleColon) return total <= 7; // 压缩形式 :: 至少代表一组 0
  return total === 8; // 非压缩须恰好 8 组（embeddedIPv4 占 2）
}

/**
 * 判定一个字符串是否为合法 IP 字面量（IPv4 或 IPv6）。
 * @param {string} value
 * @returns {boolean}
 */
function isValidIPLiteral(value) {
  if (typeof value !== 'string') return false;
  if (ipv4ToUint32(value) !== null) return true;
  return isValidIPv6(value);
}

/**
 * 将合法 IPv6 字面量展开为 8 组各 4-hex（小写、零填充）。调用方须已确保 value 合法。
 * 处理 `::` 压缩与内嵌 IPv4 末段。失败 → null。
 * @param {string} value
 * @returns {string[]|null}
 */
function expandIPv6(value) {
  let v = value.toLowerCase();
  // 内嵌 IPv4 末段 → 两个 hextet
  const lastColon = v.lastIndexOf(':');
  const tail = v.slice(lastColon + 1);
  if (tail.includes('.')) {
    const n = ipv4ToUint32(tail);
    if (n === null) return null;
    const h1 = ((n >>> 16) & 0xffff).toString(16);
    const h2 = (n & 0xffff).toString(16);
    v = v.slice(0, lastColon + 1) + h1 + ':' + h2;
  }
  let groups;
  if (v.includes('::')) {
    const [l, r] = v.split('::');
    const lg = l === '' ? [] : l.split(':');
    const rg = r === '' ? [] : r.split(':');
    const missing = 8 - (lg.length + rg.length);
    if (missing < 0) return null;
    groups = [...lg, ...Array(missing).fill('0'), ...rg];
  } else {
    groups = v.split(':');
    if (groups.length !== 8) return null;
  }
  return groups.map((g) => (g === '' ? '0' : g).padStart(4, '0'));
}

/**
 * 单一 IP 归一键（client_binding_key）：统一全套按-IP 键（seed.challenge_ip、session.ip、
 * powrate 桶、ratelimit:session 握手桶、兑换/查询时 IP 比对）的归桶口径。
 * - IPv4 → 完整地址（/32，直通原值）
 * - IPv6 → /64 前缀（取前 64 位归一，形如 `2001:0db8:0000:0001::/64`）——使同一 /64 内隐私地址
 *   轮换归同一键（不误杀），且攻击者无法用 /64 内海量地址各建一次会话逃逸自适应难度
 * - 'unknown' / 非法 → 'unknown'（直通独立桶，不污染具体 IP 桶）
 * @param {string} ip getClientIP 的返回值
 * @returns {string}
 */
export function clientBindingKey(ip) {
  if (typeof ip !== 'string' || ip.length === 0) return 'unknown';
  if (ip === 'unknown') return 'unknown';
  if (ipv4ToUint32(ip) !== null) return ip; // IPv4 /32 直通
  if (isValidIPv6(ip)) {
    const groups = expandIPv6(ip);
    if (!groups) return 'unknown';
    return `${groups.slice(0, 4).join(':')}::/64`;
  }
  return 'unknown'; // 非法字面量兜底
}

/**
 * 受信真实 IP 头值运行时校验（纯函数）。
 * 采信前校验为**单值、合法 IP 字面量、不含逗号**；多值/含逗号/非法 → false（覆写假设失效信号，调用方落 'unknown'）。
 * **实现禁止** split(',') 取段再校验——对原始返回值整体判定（含逗号即多值即 false），
 * 否则透传 `8.8.8.8,<真实>` 的首段会通过校验（F1 复现）。
 * @param {string|null|undefined} value headers.get(headerName) 的原始返回值
 * @returns {boolean} true=单值合法 IP，可采信；false=需落 unknown
 */
export function isTrustedHeaderValueValid(value) {
  if (typeof value !== 'string') return false;
  if (value.includes(',')) return false; // 含逗号=多值 → 拒（禁 split 取段）
  const trimmed = value.trim();
  if (trimmed.length === 0) return false;
  if (/\s/.test(trimmed)) return false; // trim 后仍含内部空白 → 拒
  return isValidIPLiteral(trimmed);
}

// ============================================================
// 密钥比对（常量时间，避免计时侧信道泄漏密钥）
// ============================================================

/**
 * 常量时间字符串相等（长度不同直接 false——长度泄漏对 256bit 随机密钥可接受）。
 * @param {string} a
 * @param {string} b
 * @returns {boolean}
 */
function timingSafeEqualStr(a, b) {
  if (typeof a !== 'string' || typeof b !== 'string') return false;
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i++) {
    diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  }
  return diff === 0;
}

// ============================================================
// 核心解析（纯函数）
// ============================================================

/**
 * 可信真实客户端 IP 解析核心（纯函数）。
 *
 * 分支（严格按 design 决策 1）：
 * 1. cf = headers.get('CF-Connecting-IP')；cf 为空 → 'unknown'
 * 2. 密钥未配（originSecret 空）→ 返回 cf（未启用采信注入头=安全态）
 * 3. 请求的 `X-Origin-Auth` 经常量时间比对**不等于**密钥 → 返回 cf（直连/无有效密钥：只信 CF-IP、忽略注入头）
 * 4. 密钥校验通过（确来自 CDN）：
 *    a. realIpHeaderName 未配 → 返回 cf（CDN 节点 IP；头名待证伪门确认后再配）
 *    b. real = headers.get(realIpHeaderName)；isTrustedHeaderValueValid(real) → real.trim()；否则 → 'unknown'（不退回 cf 节点桶）
 *
 * **绝不读 X-Forwarded-For。** 密钥头与真实 IP 头均由 CDN 以覆盖语义注入，客户端伪造被覆盖、攻击者猜不到密钥。
 *
 * @param {Headers} headers Fetch Headers 对象（headers.get 大小写不敏感）
 * @param {string|null|undefined} originSecret 期望的 X-Origin-Auth 密钥（env.CDN_ORIGIN_SECRET）；空=未启用
 * @param {string|null|undefined} realIpHeaderName 受信真实 IP 头名（env.CDN_REAL_IP_HEADER）；空=未配
 * @returns {string} IP 字符串或 'unknown'
 */
export function resolveClientIP(headers, originSecret, realIpHeaderName) {
  const cf = headers.get('CF-Connecting-IP');
  if (!cf) return 'unknown';

  const secret = typeof originSecret === 'string' ? originSecret : '';
  if (secret.length === 0) {
    // 密钥未配置（未启用采信注入头）→ 只信 CF-IP。
    return cf;
  }

  const provided = headers.get(ORIGIN_AUTH_HEADER);
  if (!timingSafeEqualStr(provided, secret)) {
    // 无 / 错误密钥（直连，或来源不可信）→ 只信 CF-IP，忽略一切注入头。
    return cf;
  }

  // 密钥校验通过：这一跳确来自受信阿里云 CDN 回源。
  const headerName = typeof realIpHeaderName === 'string' ? realIpHeaderName.trim() : '';
  // 头名须为合法 HTTP token（ASCII）；非法/非 ASCII（如全角 `ｘ-forwarded-for`）→ fail-safe 到 CF-IP，
  // 避免 headers.get 抛异常，并堵住绕过下面 XFF 守卫的编码花招。
  if (headerName.length === 0 ||
      headerName.toLowerCase() === 'x-forwarded-for' ||
      !/^[!#$%&'*+.^_`|~0-9A-Za-z-]+$/.test(headerName)) {
    // 头名未配（证伪门未确认）/ 误配为 X-Forwarded-For（绝不采信 XFF，兑现 spec 绝对禁令）/ 非法头名 → fail-safe 到 CF-IP。
    return cf;
  }
  const real = headers.get(headerName);
  if (isTrustedHeaderValueValid(real)) {
    return real.trim();
  }
  // 头名已配但本请求缺失/为空/多值/非法 → 惩罚桶，不退回 cf（节点桶）。
  return 'unknown';
}

// ============================================================
// 包装：供 index.js 调用
// ============================================================

/**
 * 可信真实客户端 IP 解析（请求级包装）。env 可能无相关字段（undefined 视为未配）。
 * @param {Request} request Fetch Request
 * @param {object} [env] Worker env（读 CDN_ORIGIN_SECRET 与 CDN_REAL_IP_HEADER）
 * @returns {string} IP 字符串或 'unknown'
 */
export function getClientIP(request, env) {
  const e = env || {};
  return resolveClientIP(request.headers, e.CDN_ORIGIN_SECRET, e.CDN_REAL_IP_HEADER);
}
