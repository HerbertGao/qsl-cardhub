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
    // seed-not-found（KV 写后立即读最终一致致 challenge 写入的 seed 暂不可见）→ 短退避 + 重取
    // challenge 一次（design 决策2 / task 3.2）；其它失败（PoW 不足/上下文不一致）不重试。
    for (let attempt = 0; attempt < 2; attempt++) {
      const { seed, difficulty } = await deps.getChallenge();
      const nonce = await deps.solvePow(seed, difficulty);
      try {
        const res = await deps.postSession(seed, nonce);
        return Object.freeze({ token: res.token, sk: res.sk, sid: parseSid(res.token), exp: res.exp, quota: res.quota });
      } catch (e) {
        if (e && e.seedNotFound && attempt === 0) {
          if (typeof deps.backoff === 'function') await deps.backoff();
          continue;
        }
        throw e;
      }
    }
    throw new Error('握手失败');
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
   * 带会话的查询：无会话先握手（不裸发）；遇「会话失效/配额用尽」仅使匹配快照失效、重走握手后**至多重试一次**。
   * 区分两类 429：**配额用尽**（无 `retry_after`）→ 重握手有益（新会话=新配额）；
   * **Layer0 IP 限流**（带 `retry_after`）→ 重握手无益（IP 仍受限）→ 不重握手、退避交由调用方提示，避免白算 PoW。
   * @returns {Promise<{status: number, data: any, retried: boolean}>}
   */
  async function requestQuery(path, params) {
    let last = null;
    for (let attempt = 0; attempt < 2; attempt++) {
      const snap = await getSession(); // 不可变快照
      const url = await deps.signQuery(path, params || {}, snap);
      const res = await deps.doFetch(url);
      let data = null;
      if (res && typeof res.json === 'function') {
        try { data = await res.json(); } catch { data = null; }
      }
      last = { status: res.status, data, retried: attempt > 0 };
      if (attempt === 0) {
        const isRateLimited = res.status === 429 && data && data.retry_after != null; // 限流 429 带 retry_after
        if (res.status === 401 || (res.status === 429 && !isRateLimited)) {
          invalidate(snap); // 会话失效/配额用尽 → 仅使匹配旧会话失效、重走握手重试一次
          continue;
        }
        if (res.status === 403) {
          // _ts 时窗/签名过期 → 同会话**重签**（下轮 signQuery 生成新 _ts/_nonce）重试一次，**不**失效会话/不重握手
          continue;
        }
      }
      return last;
    }
    return last;
  }

  return { getSession, invalidate, requestQuery, _peek: () => session };
}
