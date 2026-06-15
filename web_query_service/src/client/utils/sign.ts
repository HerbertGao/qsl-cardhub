/**
 * 查询会话签名（会话专属 sk）。
 *
 * canonicalPayload **复用** worker 与 client 共享的单一模块 `src/worker/canonical.js`（单一事实源，
 * 经同名 `canonical.d.ts` 提供 TS 类型；禁止在前端另写一份易漂移实现）。
 * 签名为 `HMAC-SHA256(sk, canonicalPayload)`（与服务端 session.js verifySessionSig 同算法）。
 */
import { buildCanonicalPayload } from '../../worker/canonical.js'
import type { SessionSnapshot } from '../../worker/session-client'

async function hmacSha256Hex(keyStr: string, message: string): Promise<string> {
  const encoder = new TextEncoder()
  const key = await crypto.subtle.importKey(
    'raw',
    encoder.encode(keyStr),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign']
  )
  const sig = await crypto.subtle.sign('HMAC', key, encoder.encode(message))
  return Array.from(new Uint8Array(sig)).map((b) => b.toString(16).padStart(2, '0')).join('')
}

/** 高熵 hex nonce（_nonce 字符集 hex，天然无分隔符注入面）。 */
function hexNonce(bytes = 16): string {
  const arr = new Uint8Array(bytes)
  crypto.getRandomValues(arr)
  return Array.from(arr).map((b) => b.toString(16).padStart(2, '0')).join('')
}

/**
 * 构造带会话 token + 会话签名的查询 URL（供 session-client 注入为 signQuery 依赖）。
 * canonicalPayload 字段③自动排除 token/_sig/_ts/_nonce；token 经字段①(sid) 绑定。
 */
export async function signQuery(
  path: string,
  params: Record<string, string>,
  snapshot: SessionSnapshot
): Promise<string> {
  const url = new URL(path, window.location.origin)
  for (const [k, v] of Object.entries(params)) url.searchParams.set(k, v)
  const ts = Date.now().toString()
  const nonce = hexNonce()
  url.searchParams.set('token', snapshot.token)
  // 此刻 searchParams 含业务参数 + token（canonical 排除 token/_ts/_nonce/_sig）
  const payload = buildCanonicalPayload({ sid: snapshot.sid, path: url.pathname, params: url.searchParams, ts, nonce })
  const sig = await hmacSha256Hex(snapshot.sk, payload)
  url.searchParams.set('_ts', ts)
  url.searchParams.set('_nonce', nonce)
  url.searchParams.set('_sig', sig)
  return url.toString()
}
