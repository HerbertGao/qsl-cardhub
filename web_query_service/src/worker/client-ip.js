/**
 * 可信真实客户端 IP 解析（trusted-client-ip 规范）
 *
 * 双入口架构感知：
 * - 直连 Cloudflare 源站：`CF-Connecting-IP` 即真实用户 IP（CF 边缘按 TCP 层对端写入、覆盖客户端同名头）。
 * - 经阿里云 CDN 回源：`CF-Connecting-IP` 是 CDN 回源节点 IP；真实用户 IP 由 CDN 注入受信头（推荐 `Ali-Cdn-Real-Ip`）。
 *
 * 该解析产物**仅**作限流/防爬计数键，禁止用作访问控制/鉴权。
 *
 * 本模块导出命名纯函数供 index.js 与单测 import。
 */

// ============================================================
// task 1.1 — IPv4 ip ∈ CIDR 判定（纯函数）
// ============================================================

/**
 * 将点分十进制 IPv4 字符串转为 32 位无符号整数。
 * 非法（段数≠4、非整数、>255、负、带前导零、空段）→ 返回 null。
 * @param {string} ip
 * @returns {number|null} 0..0xFFFFFFFF（无符号）或 null
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
 * 解析并校验 CIDR 字符串为 { network, prefix }（network 为无符号 32 位）。
 * 校验：必须形如 a.b.c.d/p；p 为整数、0..32、无前导零（'/08' 非法）、非负。
 * 非法 → 返回 null。
 * @param {string} cidr
 * @returns {{network: number, prefix: number}|null}
 */
function parseCidr(cidr) {
  if (typeof cidr !== 'string') return null;
  const slash = cidr.indexOf('/');
  if (slash < 0) return null;
  const ipPart = cidr.slice(0, slash);
  const prefixPart = cidr.slice(slash + 1);
  // 前缀格式：纯十进制整数、无前导零（'0' 单字符合法）、0..32。
  if (!/^(0|[1-9][0-9]?)$/.test(prefixPart)) return null;
  const prefix = Number(prefixPart);
  if (!Number.isInteger(prefix) || prefix < 0 || prefix > 32) return null;
  const network = ipv4ToUint32(ipPart);
  if (network === null) return null;
  return { network: network >>> 0, prefix };
}

/**
 * 由前缀位数生成 32 位无符号网络掩码。
 * prefix===0 → 0（特判：避免 JS `0xFFFFFFFF << 32` 的 mod-32 no-op 陷阱）。
 * prefix===32 → 0xFFFFFFFF。
 * @param {number} prefix 0..32
 * @returns {number} 无符号 32 位掩码
 */
function prefixToMask(prefix) {
  if (prefix <= 0) return 0;
  if (prefix >= 32) return 0xFFFFFFFF >>> 0;
  // 0xFFFFFFFF << (32 - prefix) 再无符号化；此处 32-prefix ∈ [1,31]，无 no-op 陷阱。
  return (0xFFFFFFFF << (32 - prefix)) >>> 0;
}

/**
 * task 1.1：判定 IPv4 `ip` 是否落在 `cidr` 段内（纯函数）。
 * 非法 IP / 非法 CIDR → 返回 false（即不命中，绝不误判）。
 * 钉死陷阱：比较前所有值 `>>> 0` 无符号化；prefix===0 特判 mask=0；prefix===32 精确。
 * @param {string} ip 点分十进制 IPv4
 * @param {string} cidr 形如 a.b.c.d/p
 * @returns {boolean}
 */
export function ipv4InCidr(ip, cidr) {
  const ipNum = ipv4ToUint32(ip);
  if (ipNum === null) return false;
  const parsed = parseCidr(cidr);
  if (parsed === null) return false;
  const mask = prefixToMask(parsed.prefix);
  return ((ipNum & mask) >>> 0) === ((parsed.network & mask) >>> 0);
}

// ============================================================
// task 1.2 / 1.3 — 白名单解析（默认拒：仅接受全局可路由公网单播 IPv4 CIDR）
// ============================================================

/**
 * 需拒绝的非公网/保留 IPv4 段（[base, prefix]）。
 * 私网 10/8、172.16/12、192.168/16；CGNAT 100.64/10；loopback 127/8；
 * link-local 169.254/16；组播 224.0.0.0/4；保留 240.0.0.0/4；0.0.0.0/8。
 */
const RESERVED_RANGES = [
  ['0.0.0.0', 8],        // 本网络 / "this host"
  ['10.0.0.0', 8],       // 私网
  ['100.64.0.0', 10],    // CGNAT
  ['127.0.0.0', 8],      // loopback
  ['169.254.0.0', 16],   // link-local
  ['172.16.0.0', 12],    // 私网
  ['192.168.0.0', 16],   // 私网
  ['224.0.0.0', 4],      // 组播 224.0.0.0 - 239.255.255.255
  ['240.0.0.0', 4],      // 保留 240.0.0.0 - 255.255.255.255（含 255.255.255.255 广播）
];

/**
 * 判定候选 CIDR 块 [base, broadcast] 是否与任一保留段区间相交。
 * 只查网络基址会漏掉过宽聚合段（如 8.0.0.0/6 含 10/8、96.0.0.0/4 含 CGNAT），
 * 故须按整块相交判定：start <= rend && rstart <= end（无符号比较）。
 * 单 IP /32 仍正确（块退化为一点）。
 * @param {number} network 无符号 32 位网络地址
 * @param {number} prefix 0..32
 * @returns {boolean} true=触及保留段，应拒绝
 */
function cidrTouchesReserved(network, prefix) {
  const mask = prefixToMask(prefix);
  const start = (network & mask) >>> 0;
  const end = (network | (~mask >>> 0)) >>> 0; // 块最高地址（无符号化）
  for (const [base, rp] of RESERVED_RANGES) {
    const rbNum = ipv4ToUint32(base);
    if (rbNum === null) continue;
    const rmask = prefixToMask(rp);
    const rstart = (rbNum & rmask) >>> 0;
    const rend = (rbNum | (~rmask >>> 0)) >>> 0;
    if (start <= rend && rstart <= end) return true; // 区间相交
  }
  return false;
}

/**
 * task 1.2/1.3：把 env.CDN_ORIGIN_CIDRS（逗号/空白分隔）解析为合法公网单播 IPv4 CIDR 数组。
 * 默认拒口径：仅接受全局可路由公网单播 IPv4 CIDR，其余一律丢弃该条目。
 * - 拒绝：私网、CGNAT、loopback、link-local、组播、保留、/0、非法前缀（非整数/>32/负/前导零）、非法 IP。
 * - IPv6 条目仅丢弃该条（含 ':' 即非 IPv4 CIDR，跳过），IPv4 条目仍正常保留（task 1.3）。
 * - 未配置 / 空串 / 全被拒 → 返回空数组。
 * @param {string|undefined|null} envValue
 * @returns {string[]} 规范化保留的合法 CIDR 字符串（原样保留通过的条目）
 */
export function parseTrustedCidrs(envValue) {
  if (typeof envValue !== 'string') return [];
  // 逗号或任意空白分隔
  const tokens = envValue.split(/[\s,]+/).map((t) => t.trim()).filter((t) => t.length > 0);
  const out = [];
  for (const token of tokens) {
    // IPv6 条目（含冒号）一律丢弃该条，不影响其余 IPv4 条目（task 1.3）。
    if (token.includes(':')) continue;
    const parsed = parseCidr(token);
    if (parsed === null) continue;        // 非法 IP / 非法前缀（含前导零、>32、负）
    if (parsed.prefix === 0) continue;    // /0 直接拒（决策 2）
    if (cidrTouchesReserved(parsed.network, parsed.prefix)) continue; // 整块触及保留段即拒（含过宽聚合段）
    out.push(token);
  }
  return out;
}

// ============================================================
// task 1.5 — 受信真实 IP 头值运行时校验（纯函数）
// ============================================================

/**
 * 判定一个字符串是否为合法 IPv6 字面量（字面量校验，不参与 CIDR 匹配）。
 * 支持完整/压缩（::）形式、IPv4-mapped 末段。不接受 zone-id（含 '%'）。
 * @param {string} value
 * @returns {boolean}
 */
function isValidIPv6(value) {
  if (typeof value !== 'string') return false;
  if (value.length === 0) return false;
  if (value.includes('%')) return false; // 拒绝 zone-id
  // 至少含一个冒号才可能是 IPv6
  if (!value.includes(':')) return false;
  // 拒绝 3+ 连续冒号（如 ':::'）
  if (value.includes(':::')) return false;
  // 边界为孤立单冒号（非 '::'）一律拒：':1:2...'、'1:2...:' 等畸形串。
  if (value.startsWith(':') && !value.startsWith('::')) return false;
  if (value.endsWith(':') && !value.endsWith('::')) return false;
  // '::' 至多出现一次
  const doubleColonCount = (value.match(/::/g) || []).length;
  if (doubleColonCount > 1) return false;

  const hasDoubleColon = value.includes('::');
  // 末段可为内嵌 IPv4（如 ::ffff:1.2.3.4）
  let head = value;
  let embeddedIPv4 = false;
  const lastColon = value.lastIndexOf(':');
  const tail = value.slice(lastColon + 1);
  if (tail.includes('.')) {
    if (ipv4ToUint32(tail) === null) return false;
    embeddedIPv4 = true;
    head = value.slice(0, lastColon + 1); // 保留末尾冒号便于分组计数
  }

  // 拆分分组（按单冒号）。'::' 会产生空分组。
  const groups = head.split(':');
  // 校验各非空分组为 1..4 位十六进制
  for (const g of groups) {
    if (g === '') continue; // 来自 '::' 或首/末冒号
    if (!/^[0-9a-fA-F]{1,4}$/.test(g)) return false;
  }

  // 统计有效 16 位分组数（embeddedIPv4 占 2 个分组）。
  const hexGroups = groups.filter((g) => g !== '').length;
  const total = hexGroups + (embeddedIPv4 ? 2 : 0);

  if (hasDoubleColon) {
    // 压缩形式：总分组数须 < 8（:: 至少代表一组 0）。
    return total <= 7;
  }
  // 非压缩形式：须恰好 8 个 16 位分组（embeddedIPv4 算 2）。
  return total === 8;
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
 * task 1.5：受信真实 IP 头值运行时校验（纯函数）。
 * 采信前校验为**单值、合法 IP 字面量、不含逗号**。
 * 多值/含逗号/非法 → false（覆写假设失效信号，调用方需落 'unknown'）。
 *
 * **实现禁止** split(',') 取段再校验——对原始返回值整体判定（含逗号即多值即 false），
 * 否则透传 `8.8.8.8,<真实>` 的首段会通过校验（F1 复现）。
 *
 * @param {string|null|undefined} value headers.get(headerName) 的原始返回值
 * @returns {boolean} true=单值合法 IP，可采信；false=需落 unknown
 */
export function isTrustedHeaderValueValid(value) {
  if (typeof value !== 'string') return false;
  // 含逗号即视为多值 → 直接拒（禁止 split 取段）。
  if (value.includes(',')) return false;
  const trimmed = value.trim();
  if (trimmed.length === 0) return false;
  // trim 后仍不得含内部空白（多值/异常信号）。
  if (/\s/.test(trimmed)) return false;
  return isValidIPLiteral(trimmed);
}

// ============================================================
// task 2.1 — 五分支核心解析（纯函数）
// ============================================================

/**
 * task 2.1：可信真实客户端 IP 解析核心（纯函数）。
 *
 * 五分支（严格按 design 决策 1 伪码）：
 * 1. cf = headers.get('CF-Connecting-IP')；cf 为空 → 'unknown'
 * 2. cf 命中白名单（ipv4InCidr 对任一 cidr 为真）：
 *    a. realIpHeaderName 为空（未配）→ 返回 cf（fail-safe，未证实覆写=安全态）
 *    b. 否则 real = headers.get(realIpHeaderName)；
 *       - isTrustedHeaderValueValid(real) → 返回 real.trim()
 *       - 否则 → 'unknown'（缺失/多值/非法，不退回 cf 节点桶）
 * 3. 否则（未命中）→ 返回 cf（忽略一切注入头）
 *
 * **绝不读 X-Forwarded-For。** IPv6 的 cf 不会命中任何 IPv4 CIDR（安全降级→不命中→分支 3）。
 *
 * @param {Headers} headers Fetch Headers 对象（headers.get 大小写不敏感）
 * @param {string[]} cidrs parseTrustedCidrs 的结果
 * @param {string|null|undefined} realIpHeaderName 受信头名；空=未配
 * @returns {string} IP 字符串或 'unknown'
 */
export function resolveClientIP(headers, cidrs, realIpHeaderName) {
  const cf = headers.get('CF-Connecting-IP');
  if (!cf) return 'unknown';

  const list = Array.isArray(cidrs) ? cidrs : [];
  let hit = false;
  for (const cidr of list) {
    if (ipv4InCidr(cf, cidr)) {
      hit = true;
      break;
    }
  }

  if (hit) {
    // 命中白名单：这一跳确来自受信 CDN 回源。
    const headerName = typeof realIpHeaderName === 'string' ? realIpHeaderName.trim() : '';
    if (headerName.length === 0) {
      // 头名未配（未证实覆写）→ fail-safe 到 CF-IP，不采信任何头。
      return cf;
    }
    const real = headers.get(headerName);
    if (isTrustedHeaderValueValid(real)) {
      return real.trim();
    }
    // 头名已配但本请求缺失/为空/多值/非法 → 惩罚桶，不退回 cf（节点桶）。
    return 'unknown';
  }

  // 未命中白名单（直连 CF 或来源不可信）→ 只信 CF-Connecting-IP，忽略一切注入头。
  return cf;
}

// ============================================================
// 包装：供 index.js 调用
// ============================================================

/**
 * 可信真实客户端 IP 解析（请求级包装）。
 * 注意 env 可能无相关字段（undefined 视为未配）。
 * @param {Request} request Fetch Request
 * @param {object} [env] Worker env（读 CDN_ORIGIN_CIDRS 与 CDN_REAL_IP_HEADER）
 * @returns {string} IP 字符串或 'unknown'
 */
export function getClientIP(request, env) {
  const e = env || {};
  return resolveClientIP(
    request.headers,
    parseTrustedCidrs(e.CDN_ORIGIN_CIDRS),
    e.CDN_REAL_IP_HEADER
  );
}
