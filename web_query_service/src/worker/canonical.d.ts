// 手写类型声明：供前端 TS（client tsconfig 无 allowJs）re-export 引用同目录 `canonical.js`。
// 与 canonical.js 的导出保持一致，是该共享模块的类型契约。

export type CanonicalParams =
  | URLSearchParams
  | Record<string, string>
  | Iterable<[string, string]>;

export interface CanonicalInput {
  /** 会话 id（非完整 token） */
  sid: string;
  /** 原始 url.pathname（保留 URL 编码形态、不解码） */
  path: string;
  /** 业务查询参数（自动排除 _sig/_ts/_nonce）；可选 */
  params?: CanonicalParams;
  /** 请求时间戳（毫秒） */
  ts: string | number;
  /** 随机 nonce */
  nonce: string;
}

export function canonicalParamString(params: CanonicalParams | null | undefined): string;
export function buildCanonicalPayload(input: CanonicalInput): string;
