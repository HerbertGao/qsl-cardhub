## 1. Worker 查询端点：按租户有条件返回脱敏 qty

- [x] 1.1 在 `web_query_service/src/worker/index.js` 顶部工具区新增 `formatQtyByMode(qty, mode)`：`mode === 'exact'` → 原始整数；`mode === 'approximate'` → `qty<=10`→`'≤10'`、`<=50`→`'≤50'`、否则 `'>50'`（分桶阈值对齐 `web/src/composables/useQtyDisplayMode.ts:65-67`，approximate 返回值恒为 `{≤10,≤50,>50}` 之一）；对 `exact`/`approximate` **以外**的 mode（缺失/未知）返回「不展示」哨兵（如 `undefined`），供 1.3 省略字段。
- [x] 1.2 读取 `qty_display_mode`：**复用**查询端点已解析、与 `cards` 查询同一个 `tenant_id`（`resolveQueryTenant` 结果，约 774 行），`SELECT value FROM app_settings WHERE tenant_id=? AND key='qty_display_mode'`，与卡片查询同批 `DB.batch`（或紧邻一次 `.first()`）；**禁止**二次解析或直读 `env.DEFAULT_TENANT`。读不到该行 / 读取异常 → 一律按「未配置」处理（不返回 qty，禁止 fail-open 退回精确值）。
- [x] 1.3 结果映射（806-824 行）：**仅当** mode 为 `exact`/`approximate` 时在返回对象加入 `qty`（= `formatQtyByMode` 结果）；未配置/未知/异常 → 返回对象**不含** `qty` 字段。确认 `approximate` 下不附带原始整数、不与脱敏值并发下发。

## 2. 查询页前端：有条件展示数量

- [x] 2.1 在 `web_query_service/src/client/components/ResultList.vue` 的 `CardItem` 接口加**可选**字段 `qty?: number | string`。
- [x] 2.2 在 card-header（status 各态均可见、非条件 `card-body`）以 `v-if="item.qty != null"` 渲染 `{{ item.qty }}`（如「数量：N」）；接口未返回 qty 则不展示；前端不对 qty 做任何换算。用 `!= null`（而非 `v-if="item.qty"`）以明确「字段存在即展示」、依赖 schema `qty>0`（`schema.sql:67`，故无需担心 exact 的 `0`），避免日后误用 falsy 判断。

## 3. 验证

- [x] 3.1 新增测试文件 `web_query_service/verify/query-qty.test.js`（相对 `web_query_service` 即 `verify/query-qty.test.js`，须匹配 `verify/*.test.js` glob 才被 `node --test` 收集），覆盖：
  - `qty_display_mode='exact'` → `qty` 为原始整数；
  - `='approximate'` 边界：qty=10→`≤10`、11→`≤50`、50→`≤50`、51→`>50`；断言 `qty` 字段为**字符串**且严格等于预期分桶串，且 item 内**不存在**数值型 `qty` 或其它携带原始精确数的字段（**注意**：`≤10`/`≤50` 串本身含 `10`/`50` 文本，断言须针对 `qty` 字段的**类型/值**，而非对响应全文做「不含数字」子串搜索——后者不可执行会误报）；
  - 无该行 / 未知值（如 `'foo'`）/ 空串 → 响应 item **不含** `qty` 字段；
  - **读设置异常**（mock `app_settings` 读抛错）→ 响应**仍返回卡片列表**且 item **不含** `qty`（非 500、不丢卡片）；
  - **bare 默认租户路径**且该租户无 `qty_display_mode` 行 → 响应 item **不含** `qty`（公开面最常命中、最易被未来 `?? 'exact'` 回退误伤的 deny 路径）；
  - **多租户**：同一呼号在 `/t/<slug>/` 查询，`cards` 与 `qty_display_mode` 分属不同租户且模式相反，断言脱敏读取所 bind 的 `tenant_id` 与 `cards` 查询一致（不串租户）。
- [x] 3.2 `cd web_query_service && pnpm run test:unit` 跑通。
- [x] 3.3 `cd web_query_service && pnpm run build` 通过；本地 `wrangler dev` 手测：未配置租户查呼号→不显示数量；桌面端选 exact/approximate 同步后→查询页相应显示精确数 / 分桶。

## 4. 收尾

- [x] 4.1 实现时在 `qty_display_mode` **读取处局部捕获**异常，与「无该行」走同一分支：**省略 qty 但照常返回卡片列表**（fail-safe，不退回精确值、不使整次查询 500/丢卡片）；顶层脱敏 catch 仅兜底防泄露。
- [ ] 4.2 `openspec-cn validate add-query-qty-display --strict` 通过后，运行 `/opsx:apply` 实现并归档。
