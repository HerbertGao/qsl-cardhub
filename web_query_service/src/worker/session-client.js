/**
 * 客户端会话状态机核心（依赖注入、可单测）。
 *
 * query-antibot-session「客户端会话状态机」需求的可测落点：singleflight 握手、不可变凭据快照、
 * 401/429 仅使匹配旧会话失效后至多重试一次、无会话先握手不裸发。
 * 浏览器侧 `src/client/utils/session.ts` 注入真实 fetch + PoW Web Worker + 签名实现；
 * verify/session-client.test.js 注入 mock 验证并发/重试不变量（强锚）。
 *
 * 共享 `.js` + 同名 `.d.ts`（client tsconfig 无 allowJs，靠 sibling .d.ts 解析）。
 */

/** 从 token（`sid.HMAC`）取 sid。 */
export function parseSid(token) {
  const dot = typeof token === 'string' ? token.lastIndexOf('.') : -1;
  return dot > 0 ? token.slice(0, dot) : '';
}

/**
 * @param {{
 *   getChallenge: () => Promise<{seed: string, difficulty: number}>,
 *   solvePow: (seed: string, difficulty: number) => Promise<string>,
 *   postSession: (seed: string, nonce: string) => Promise<{token: string, sk: string, exp: number, quota: number}>,
 *   signQuery: (path: string, params: Record<string,string>, snapshot: object) => Promise<string>,
 *   doFetch: (url: string) => Promise<{status: number, json?: () => any}>,
 *   now: () => number,
 *   skewMs?: number,
 * }} deps
 */
export function createSessionManager(deps) {
  const skew = typeof deps.skewMs === 'number' ? deps.skewMs : 5000;
  let session = null; // 不可变快照 {token, sk, sid, exp, quota}
  let inflight = null; // singleflight 握手 promise

  async function handshake() {
    const { seed, difficulty } = await deps.getChallenge();
    const nonce = await deps.solvePow(seed, difficulty);
    const res = await deps.postSession(seed, nonce);
    return Object.freeze({ token: res.token, sk: res.sk, sid: parseSid(res.token), exp: res.exp, quota: res.quota });
  }

  /** 取有效会话快照；无效则 singleflight 握手（并发查询共享同一 promise，禁重复 PoW/覆盖凭据）。 */
  async function getSession() {
    const now = deps.now();
    if (session && session.exp - skew > now) return session;
    if (!inflight) {
      inflight = handshake().then(
        (s) => { session = s; inflight = null; return s; },
        (e) => { inflight = null; throw e; }
      );
    }
    return inflight;
  }

  /** 仅使匹配快照对应的会话失效（不误伤其它流程刚建立的新会话）。 */
  function invalidate(snapshot) {
    if (session && snapshot && session.token === snapshot.token) session = null;
  }

  /**
   * 带会话的查询：无会话先握手（不裸发）；遇 401/429 仅使匹配快照失效、重走握手后**至多重试一次**。
   * @returns {Promise<{status: number, json?: () => any, retried: boolean}>}
   */
  async function requestQuery(path, params) {
    let last = null;
    for (let attempt = 0; attempt < 2; attempt++) {
      const snap = await getSession(); // 不可变快照
      const url = await deps.signQuery(path, params || {}, snap);
      const res = await deps.doFetch(url);
      last = res;
      if ((res.status === 401 || res.status === 429) && attempt === 0) {
        invalidate(snap); // 仅使匹配旧会话失效
        continue; // 重走握手重试一次
      }
      return Object.assign(res, { retried: attempt > 0 });
    }
    return Object.assign(last, { retried: true });
  }

  return { getSession, invalidate, requestQuery, _peek: () => session };
}
