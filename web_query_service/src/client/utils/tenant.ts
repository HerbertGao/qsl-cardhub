/**
 * 租户路径前缀派生（单一事实源，供 fetch/requestQuery 入参处单点前置）。
 *
 * 文法与 worker parser 同构：段边界正则 `^/t/[a-z0-9-]{1,32}(?:/|$)`——必须有尾部边界
 * `(?:/|$)`，禁用无边界 `^/t/[a-z0-9-]{1,32}`（否则超 32 长 slug 被截断出错误前缀）。
 */

/** 段边界匹配 `/t/<slug>`，捕获 slug；尾部须为 `/` 或字符串结束。 */
const TENANT_PREFIX_RE = /^\/t\/([a-z0-9-]{1,32})(?:\/|$)/

/**
 * 读 `location.pathname` 派生租户前缀（**含** `/t/`、**无**尾斜杠，如 `/t/bh2ro`）。
 * 不命中（含 bare 路径、超长 slug、非法字符）→ `''`。
 * 仅在调用 API 的入参处前置**一次**：`fetch(`${tenantBase()}/api/...`)`。
 */
export function tenantBase(): string {
  const m = TENANT_PREFIX_RE.exec(window.location.pathname)
  return m ? `/t/${m[1]}` : ''
}

/**
 * 派生纯租户 slug（如 `'bh2ro'`），bare 路径 → `''`。
 * 供微信订阅 OAuth `state` 取路由租户用（避免重复正则）。
 */
export function tenantSlug(): string {
  const m = TENANT_PREFIX_RE.exec(window.location.pathname)
  return m ? m[1] : ''
}
