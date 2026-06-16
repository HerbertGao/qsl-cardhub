/**
 * 防爬会话 / PoW 纯函数（query-antibot-session 规范）。
 *
 * 抽离为命名纯函数供 `index.js` 与单测 import；核心安全逻辑（PoW 校验、token HMAC 签发/校验、
 * 会话签名校验、自适应难度、会话绑定）集中于此，便于 node:test 覆盖边界——这些单测即本能力
 * 各类别的**强锚**（可执行不变量检查）。
 *
 * 运行时：依赖全局 `crypto.subtle`（Cloudflare Workers 与 Node ≥16 均提供）。
 */

import { buildCanonicalPayload } from './canonical.js';

const enc = new TextEncoder();

// ============================================================
// 哈希 / HMAC
// ============================================================

/** SHA-256 → 小写 hex。 */
export async function sha256Hex(message) {
  const buf = await crypto.subtle.digest('SHA-256', enc.encode(String(message)));
  return [...new Uint8Array(buf)].map((b) => b.toString(16).padStart(2, '0')).join('');
}

/** HMAC-SHA256(keyStr, message) → 小写 hex（真 HMAC，禁用 sha256(secret+msg)）。 */
export async function hmacSha256Hex(keyStr, message) {
  const key = await crypto.subtle.importKey(
    'raw',
    enc.encode(String(keyStr)),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  );
  const sig = await crypto.subtle.sign('HMAC', key, enc.encode(String(message)));
  return [...new Uint8Array(sig)].map((b) => b.toString(16).padStart(2, '0')).join('');
}

/** 常量时间字符串相等（长度不同直接 false）。 */
export function constantTimeEqual(a, b) {
  if (typeof a !== 'string' || typeof b !== 'string') return false;
  if (a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i++) diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  return diff === 0;
}

/** 高熵随机 hex（默认 16 字节=128bit）。 */
export function randomHex(bytes = 16) {
  const arr = new Uint8Array(bytes);
  crypto.getRandomValues(arr);
  return [...arr].map((b) => b.toString(16).padStart(2, '0')).join('');
}

// ============================================================
// PoW（hashcash 前导零位）
// ============================================================

/**
 * 统计 hex 哈希串的二进制前导零位数。
 * @param {string} hashHex 小写/大写 hex
 * @returns {number}
 */
export function powLeadingZeroBits(hashHex) {
  if (typeof hashHex !== 'string' || hashHex.length === 0) return 0;
  let bits = 0;
  for (let i = 0; i < hashHex.length; i++) {
    const nibble = parseInt(hashHex[i], 16);
    if (Number.isNaN(nibble)) return bits; // 非法字符即止
    if (nibble === 0) {
      bits += 4;
      continue;
    }
    // 该 nibble 非零：计其二进制内的前导零位（nibble∈[1,15]）
    if (nibble < 0b0010) bits += 3;
    else if (nibble < 0b0100) bits += 2;
    else if (nibble < 0b1000) bits += 1;
    break;
  }
  return bits;
}

/**
 * 校验 PoW：sha256(seed + ":" + nonce) 的二进制前导零位数 ≥ difficulty。
 * 守卫：difficulty 须为非负整数；nonce 须为非空字符串；seed 须为非空字符串。
 * @returns {Promise<boolean>}
 */
export async function verifyPow(seed, nonce, difficulty) {
  if (typeof seed !== 'string' || seed.length === 0) return false;
  if (typeof nonce !== 'string' || nonce.length === 0) return false;
  if (!Number.isInteger(difficulty) || difficulty < 0) return false;
  const hash = await sha256Hex(`${seed}:${nonce}`);
  return powLeadingZeroBits(hash) >= difficulty;
}

// ============================================================
// 会话 token = sid.HMAC(SESSION_SECRET, sid)
// ============================================================

// sid/nonce 字符集：base64url（含 hex），不含 '.'，使 lastIndexOf('.') 切分无歧义、KV 键无分隔注入。
const SID_RE = /^[A-Za-z0-9_-]+$/;

/** 签发会话 token：`sid.HMAC`（真 HMAC-SHA256）。 */
export async function makeToken(sid, secret) {
  const mac = await hmacSha256Hex(secret, sid);
  return `${sid}.${mac}`;
}

/**
 * 解析并校验会话 token → sid（合法）或 null。
 * 顺序（硬约束）：lastIndexOf('.') 切分 → 校验 sid 字符集 ∈ base64url/hex（不过即 null）→ 常量时间验 HMAC（不过即 null）。
 * @returns {Promise<string|null>}
 */
export async function parseToken(token, secret) {
  if (typeof token !== 'string') return null;
  const dot = token.lastIndexOf('.');
  if (dot <= 0 || dot === token.length - 1) return null;
  const sid = token.slice(0, dot);
  const mac = token.slice(dot + 1);
  if (!SID_RE.test(sid)) return null; // 先验字符集（非法即 null，HMAC 之前）
  const expected = await hmacSha256Hex(secret, sid);
  if (!constantTimeEqual(mac, expected)) return null;
  return sid;
}

// ============================================================
// 查询会话签名（HMAC-SHA256(sk, canonicalPayload)）
// ============================================================

/**
 * 校验查询会话签名。canonicalPayload 由共享 canonical 模块构造（单一事实源）。
 * @param {{ sid: string, path: string, params?: any, ts: string|number, nonce: string }} input
 * @param {string} sk 会话专属签名密钥
 * @param {string} sigHex 客户端提交的 _sig
 * @returns {Promise<boolean>}
 */
export async function verifySessionSig(input, sk, sigHex) {
  if (typeof sk !== 'string' || sk.length === 0) return false;
  if (typeof sigHex !== 'string' || sigHex.length === 0) return false;
  const payload = buildCanonicalPayload(input);
  const expected = await hmacSha256Hex(sk, payload);
  return constantTimeEqual(sigHex, expected);
}

// ============================================================
// 自适应难度
// ============================================================

/**
 * 默认难度参数（design 决策3 给出的区间；apply 实测后可调，但必须落在约束内：
 * baseMin>0、diffMax 手机可解、单调递增封顶）。
 */
export const DIFFICULTY_DEFAULTS = Object.freeze({
  base: 18, // 正常用户基线（手机 ~0.1–0.3s）
  baseMin: 12, // 正下限（>0，恒有真实 PoW）
  diffMax: 22, // 上限封顶（封顶仍手机可解，避免共享出口 IP 正常用户 DoS）
  step: 3, // 每超一个阈值 +step bit
  threshold: 5, // 窗口内每 threshold 次建会话提升一档
});

/**
 * 按近期建会话频率 rate 计算难度：clamp(base + tier, baseMin, diffMax)，tier = floor(rate/threshold)*step。
 * @param {number} rate 近期建会话计数（窗口内）
 * @param {Partial<typeof DIFFICULTY_DEFAULTS>} [opts]
 * @returns {number}
 */
export function difficultyFor(rate, opts = {}) {
  const { base, baseMin, diffMax, step, threshold } = { ...DIFFICULTY_DEFAULTS, ...opts };
  const r = Number.isFinite(rate) && rate > 0 ? Math.floor(rate) : 0;
  const tier = Math.floor(r / threshold) * step;
  let d = base + tier;
  if (d < baseMin) d = baseMin; // 下限（>0）
  if (d > diffMax) d = diffMax; // 上限封顶
  return d;
}

/** unknown 来源难度档 ≡ diffMax（最高档，取封顶值）。 */
export function unknownDifficulty(opts = {}) {
  const { diffMax } = { ...DIFFICULTY_DEFAULTS, ...opts };
  return diffMax;
}

// ============================================================
// UA 指纹 / 会话绑定校验
// ============================================================

/** UA 指纹：sha256(UA||'') 全长 hex（与 spec「sha256(User-Agent)」一致）。UA 缺失头归一为空串。 */
export async function uaHash(ua) {
  const s = typeof ua === 'string' ? ua : '';
  return sha256Hex(s);
}

/**
 * 会话绑定校验（纯函数；调用方传入已算好的 bindingKey 与 uaHashValue）。
 * - exp 未过期（now < session.exp）
 * - binding_mode='ip' → bindingKey === session.ip（归一键）；binding_mode='none' → 跳过 IP 比对
 * - ua_hash 始终校验（两种 mode 均绑）
 * 注：token HMAC + KV 命中由调用方在此之前完成（授权三要素与），本函数只判绑定项。
 * @returns {boolean}
 */
export function sessionValid(session, bindingKey, uaHashValue, now) {
  if (!session || typeof session !== 'object') return false;
  if (typeof session.exp !== 'number' || !(now < session.exp)) return false;
  if (session.ua_hash !== uaHashValue) return false;
  if (session.binding_mode === 'none') return true; // unknown 会话跳过 IP 比对
  // 默认（含缺省）按 ip 模式：必须归一键相等
  return session.ip != null && session.ip === bindingKey;
}
