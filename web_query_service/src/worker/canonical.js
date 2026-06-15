/**
 * 查询会话签名的 canonicalPayload 构造（单一事实源）。
 *
 * worker（`src/worker/index.js`）与前端（`src/client/utils/sign.ts`）**共用本模块**——
 * worker `import`、前端 `utils/sign.ts` re-export 引用；同名 `canonical.d.ts` 供 TS 端类型解析
 * （client tsconfig 无 `allowJs`，靠 sibling `.d.ts` 解析裸 `.js` 导入，避免 TS7016）。
 * 禁止在前端另写一份易漂移的 canonical 拼装逻辑。
 *
 * canonicalPayload **按固定顺序**用 `\n` 连接五项：
 *   ① sid（会话 id，非完整 token，避免拼装漂移）
 *   ② path（原始 `url.pathname`，保留 URL 编码形态、不解码）
 *   ③ 业务查询参数串（按 key 字母序；**排除** `_sig`/`_ts`/`_nonce`；每个 key 与 value 各自
 *      `encodeURIComponent` 后以 `=` 拼对、`&` 连接——保证 `&`/`=`/`\n`/`%` 均被百分号编码，
 *      使分隔符注入不可能；无业务参数时为空串）
 *   ④ _ts
 *   ⑤ _nonce
 *
 * `_ts`/`_nonce` 经④⑤纳入 HMAC 输入（绑定新鲜度/防重放、禁换 ts/nonce 复用旧 _sig），
 * 但**仅**由④⑤承载、不出现在③。
 *
 * 纯字符串运算、无 crypto/IO —— 浏览器与 Worker 运行时均可用。
 */

/**
 * 取出业务查询参数（排除 _sig/_ts/_nonce），返回 [key, value] 数组。
 * @param {URLSearchParams|Record<string,string>|Iterable<[string,string]>} params
 * @returns {Array<[string,string]>}
 */
function businessEntries(params) {
  let entries;
  if (params == null) {
    entries = [];
  } else if (typeof params.entries === 'function') {
    // URLSearchParams 或 Map
    entries = [...params.entries()];
  } else {
    entries = Object.entries(params);
  }
  // 排除签名相关参数（_sig 输出本身、_ts/_nonce 由字段④⑤承载）与会话 token（sid 已在字段①承载）。
  return entries.filter(([k]) => k !== '_sig' && k !== '_ts' && k !== '_nonce' && k !== 'token');
}

/**
 * 构造业务查询参数串（字段③）：按 key 字母序，每个 key/value 各自 encodeURIComponent，
 * `key=value` 以 `&` 连接。无业务参数时为空串。
 * @param {URLSearchParams|Record<string,string>|Iterable<[string,string]>} params
 * @returns {string}
 */
export function canonicalParamString(params) {
  const pairs = businessEntries(params).map(([k, v]) => [String(k), String(v)]);
  // 稳定排序：先按 key，再按 value（同名重复参数确定性）。
  pairs.sort((a, b) => (a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : a[1] < b[1] ? -1 : a[1] > b[1] ? 1 : 0));
  return pairs.map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`).join('&');
}

/**
 * 构造 canonicalPayload（五项固定顺序、`\n` 连接）。
 * @param {{ sid: string, path: string, params?: URLSearchParams|Record<string,string>|Iterable<[string,string]>, ts: string|number, nonce: string }} input
 * @returns {string}
 */
export function buildCanonicalPayload(input) {
  const { sid, path, params, ts, nonce } = input;
  const paramStr = canonicalParamString(params);
  return [String(sid), String(path), paramStr, String(ts), String(nonce)].join('\n');
}
