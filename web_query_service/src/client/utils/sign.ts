/**
 * 请求签名工具
 * 用于生成 API 请求的签名参数，防止简单脚本刷接口
 */

/**
 * SHA-256 哈希
 */
async function sha256(message: string): Promise<string> {
  const encoder = new TextEncoder()
  const data = encoder.encode(message)
  const hashBuffer = await crypto.subtle.digest('SHA-256', data)
  const hashArray = Array.from(new Uint8Array(hashBuffer))
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('')
}

/**
 * 生成 UUID v4
 */
function uuid(): string {
  return crypto.randomUUID()
}

export interface SignedParams {
  _ts: string
  _nonce: string
  _sig: string
}

/**
 * 为请求生成签名参数
 * @param path 请求路径（如 /api/callsigns/BV2ABC）
 * @param params 其他查询参数（不包含签名参数）
 * @param signKey 签名密钥（从 /api/config 获取）
 * @returns 签名参数对象
 */
export async function signRequest(
  path: string,
  params: Record<string, string>,
  signKey: string
): Promise<SignedParams> {
  const ts = Date.now().toString()
  const nonce = uuid()

  // 按字母排序参数
  const sortedParams = new URLSearchParams(params)
  sortedParams.sort()
  const paramStr = sortedParams.toString()

  // 构建签名 payload
  const payload = `${path}:${paramStr}:${ts}:${nonce}`
  const sig = await sha256(payload + signKey)

  return { _ts: ts, _nonce: nonce, _sig: sig }
}

/**
 * 构建带签名的完整 URL
 * @param baseUrl 基础 URL（如 /api/callsigns/BV2ABC）
 * @param params 查询参数
 * @param signKey 签名密钥（可选，未提供时不添加签名）
 * @returns 完整 URL 字符串
 */
export async function buildSignedUrl(
  baseUrl: string,
  params: Record<string, string> = {},
  signKey?: string | null
): Promise<string> {
  const url = new URL(baseUrl, window.location.origin)

  // 添加原始参数
  for (const [key, value] of Object.entries(params)) {
    url.searchParams.set(key, value)
  }

  // 如果有签名密钥，添加签名参数
  if (signKey) {
    const signedParams = await signRequest(url.pathname, params, signKey)
    url.searchParams.set('_ts', signedParams._ts)
    url.searchParams.set('_nonce', signedParams._nonce)
    url.searchParams.set('_sig', signedParams._sig)
  }

  return url.toString()
}
