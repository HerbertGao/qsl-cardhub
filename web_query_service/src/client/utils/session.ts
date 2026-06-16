/**
 * 浏览器侧会话管理：把真实 fetch + PoW Web Worker + 会话签名注入共享状态机核心
 * （`src/worker/session-client.js`，singleflight/快照/重试一次逻辑已在该模块单测覆盖）。
 */
import { createSessionManager } from '../../worker/session-client.js'
import { sha256Hex, leadingZeroBits } from '../../worker/sha256.js'
import { signQuery } from './sign'
import { tenantBase } from './tenant'

/** 在 Web Worker 中算 PoW；Worker 不可用时降级主线程同步算（小难度可接受）。 */
function solvePow(seed: string, difficulty: number): Promise<string> {
  if (typeof Worker !== 'undefined') {
    return new Promise<string>((resolve, reject) => {
      let worker: Worker
      try {
        worker = new Worker(new URL('../pow.worker.ts', import.meta.url), { type: 'module' })
      } catch {
        resolve(solvePowMainThread(seed, difficulty))
        return
      }
      worker.onmessage = (e: MessageEvent<{ ok: boolean; nonce?: string; error?: string }>) => {
        worker.terminate()
        if (e.data.ok && e.data.nonce != null) resolve(e.data.nonce)
        else reject(new Error(e.data.error || 'PoW 失败'))
      }
      worker.onerror = () => {
        worker.terminate()
        resolve(solvePowMainThread(seed, difficulty)) // 降级主线程
      }
      worker.postMessage({ seed, difficulty })
    })
  }
  return Promise.resolve(solvePowMainThread(seed, difficulty))
}

function solvePowMainThread(seed: string, difficulty: number): string {
  for (let i = 0; i < (1 << 27); i++) {
    const nonce = String(i)
    if (leadingZeroBits(sha256Hex(`${seed}:${nonce}`)) >= difficulty) return nonce
  }
  throw new Error('PoW 超出最大迭代')
}

const manager = createSessionManager({
  getChallenge: async () => {
    const r = await fetch(`${tenantBase()}/api/session/challenge`)
    const d = await r.json()
    if (!d.success) throw new Error(d.message || '获取题目失败')
    return { seed: d.seed, difficulty: d.difficulty }
  },
  solvePow,
  postSession: async (seed: string, nonce: string) => {
    const r = await fetch(`${tenantBase()}/api/session`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ seed, nonce }),
    })
    const d = await r.json()
    if (!d.success) {
      const err = new Error(d.message || '建立会话失败') as Error & { seedNotFound?: boolean }
      // 服务端 code='seed_not_found'（KV 写后读窗）→ 握手层短退避重取 challenge 一次
      if (d.code === 'seed_not_found') err.seedNotFound = true
      throw err
    }
    return { token: d.token, sk: d.sk, exp: d.exp, quota: d.quota }
  },
  signQuery,
  doFetch: (url: string) => fetch(url),
  now: () => Date.now(),
  backoff: () => new Promise<void>((r) => setTimeout(r, 150)),
})

export interface QueryResult {
  status: number
  data: unknown
}

/** 带会话查询某呼号；状态机内部自动握手 + 401/配额429 重试一次（限流429 不重握手）。 */
export async function queryCallsign(callsign: string): Promise<QueryResult> {
  const res = await manager.requestQuery(`${tenantBase()}/api/callsigns/${encodeURIComponent(callsign)}`, {})
  return { status: res.status, data: res.data }
}
