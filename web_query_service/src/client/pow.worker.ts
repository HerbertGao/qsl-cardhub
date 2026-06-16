/**
 * PoW Web Worker：在后台线程找满足前导零难度的 nonce，不阻塞 UI（design 决策9）。
 * 用共享同步 sha256（`src/worker/sha256.js`）做紧循环（Web Crypto 异步开销无法满足 0.1–0.3s 目标）。
 * 协议同 `sha256(seed + ":" + nonce)`，与服务端 crypto.subtle 校验一致。
 */
import { sha256Hex, leadingZeroBits } from '../worker/sha256.js'

interface PowRequest {
  seed: string
  difficulty: number
}

// 安全上限：避免难度异常时无限自旋（DIFF_MAX 下手机可解；远超即放弃报错）。
const MAX_ITERATIONS = 1 << 27 // ~1.3 亿，封顶兜底

self.onmessage = (e: MessageEvent<PowRequest>) => {
  const { seed, difficulty } = e.data
  for (let i = 0; i < MAX_ITERATIONS; i++) {
    const nonce = String(i)
    if (leadingZeroBits(sha256Hex(`${seed}:${nonce}`)) >= difficulty) {
      ;(self as unknown as Worker).postMessage({ ok: true, nonce })
      return
    }
  }
  ;(self as unknown as Worker).postMessage({ ok: false, error: 'PoW 超出最大迭代' })
}
