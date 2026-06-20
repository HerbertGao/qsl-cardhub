## 上下文

公开「按呼号查询页」（`web_query_service`，Cloudflare Worker + D1 + Vue 客户端）目前不展示卡片数量。链路现状：

- **桌面端**：`qty_display_mode`（`exact`/`approximate`）存本地 `app_settings` 表（`web/src/composables/useQtyDisplayMode.ts` + `src/db/app_settings.rs`）。桌面端的 `formatQty` 脱敏阈值为：`≤10` / `≤50` / `>50`。
- **同步**：`execute_sync_cmd` → `export_database()` **已**导出全部 `app_settings`（含 `qty_display_mode`），随 `POST /sync` 上云。
- **云端 `/sync`**：`web_query_service/src/worker/index.js:462/487/495` **已**无 key 白名单地按租户写入 `app_settings`。即 `qty_display_mode` 早已按租户存于云端 D1。
- **云端查询 `/api/query`**（`index.js:796-828`）：SQL 已 `SELECT c.qty`，但结果映射（806-824）**丢弃** `qty`，前端 `ResultList.vue` 无 `qty` 字段。当前行为遵守 `cloud-backend-api` 规范「禁止返回 qty」的数据最小化约束。

约束：查询页是**任何人可访问的公开页**，不能泄露精确收卡数；精度须跟随租户桌面端设置；改动尽量小。

## 目标 / 非目标

**目标：**
- 查询响应**当租户配置过精度时**为每张卡返回脱敏 `qty`（服务端脱敏：`exact` 原值 / `approximate` → `≤10`/`≤50`/`>50`）；未配置 / 未知值 / 读取异常 → 不返回 `qty`（default-deny）。
- 结果页**当接口返回 `qty` 时**展示数量（跨 status 恒显，不受分发/退卡详情条件限制）；未返回则不展示。
- 复用既有同步链路与 `app_settings` 表，桌面端/导入端/schema 零改动。

**非目标：**
- 不改桌面端 `qty_display_mode` 的存储、UI 或脱敏阈值。
- 不新增 `/sync`/`/pull` 字段、不改 D1 schema、不加迁移。
- 不为查询页单独提供 `qty_display_mode` 配置入口（精度恒跟随桌面端同步值）。

## 决策

**决策 1：approximate 脱敏在服务端（worker）完成，禁止下发原始值由前端折算；exact 按租户显式配置返回原值。**
理由：查询页公开，approximate 若返回原始整数再由前端折算，扒网络响应即可还原精确值，违背隐私目标——故 approximate 必须服务端脱敏、响应体不含原始数字。`exact` 是租户主动选择的精确暴露，按配置返回原始整数；未配置则不返回。worker 在结果映射处把 `qty` 直接替换为脱敏后值（`approximate` 为受限字符串、`exact` 为整数），或直接不含该字段。
替代方案：①前端折算——否决（approximate 会泄露精确值）；②SQL 内 `CASE` 脱敏——否决（字符串逻辑塞进 SQL 可读性差，且模式值需先查 `app_settings`）。

**决策 2：仅显式 `exact` 返回原值，`approximate` 返回分桶，其余一律不返回 `qty`（default-deny）。**
理由：查询面是公开的；对一个曾被刻意最小化（原规范禁止返回 qty）的字段，缺省必须是「不暴露」。规则：`mode === 'exact'` → 原始整数；`mode === 'approximate'` → 分桶字符串（阈值 `≤10`/`≤50`/`>50`，**仅分桶阈值**对齐 `useQtyDisplayMode.ts:65-67`）；**`mode` 缺失 / 未知值 / 读取异常 → 不返回 `qty`**。不再声称与桌面端 `formatQty` 整体逐字一致——桌面端是私有自看、无「不展示」语义，本变更只复用其分桶阈值。

**决策 3：查询时复用卡片查询已解析的同一 `tenant_id` 读 `qty_display_mode`，禁止二次解析。**
理由：`SELECT value FROM app_settings WHERE tenant_id=? AND key='qty_display_mode'` 必须 bind 与 `cards` 查询**同一个**已由 `resolveQueryTenant` 解析出的 `tenant_id`（见 `tenant-path-routing`）。当前 cards 查询是单条 `DB.prepare(...).all()`（`index.js:796-804`），故首选**在结果映射前紧邻一次 `.first()` 读该设置**（每次查询读一次、**禁止**放进每卡映射循环内 N 次读）；若把 cards 与 settings 改造进同一 `DB.batch([...])` 则可零额外往返（可选优化）。禁止另起 slug 解析或直读 `env.DEFAULT_TENANT`，否则脱敏口径与被脱敏数据可能分属不同租户。读不到该行（含读取异常）按决策 2 → 不返回 `qty`（fail-safe，不退化为精确暴露）。

**决策 4：`qty` 前端为可选字段，存在才渲染。**
理由：`approximate` 下是 `≤10` 等字符串，`exact` 下是整数，未配置租户接口不返回该字段。前端 `CardItem.qty?: number | string`，`v-if` 判存在后渲染（`{{ item.qty }}`，不再判断模式）。放在 card-header（status 各态均显示），不进条件 `card-body`，否则 pending 卡又不显示。

## 风险 / 权衡

- **缺省 default-deny**：未配置 `qty_display_mode` 的租户云端无该行 → 查询页**不展示**数量；逆转「禁止返回 qty」的最小化约束仅对显式配置过精度的租户生效。→ 这是缺省即最小暴露的安全方向（区别于早先「缺省 exact」的 fail-open 设计）。
- **聚合推断暴露面（逆转最小化的代价）**：本变更使逐卡数量进入**公开**查询面（PoW+会话+配额仅抬自动化成本、非访问控制，见 `query-antibot-session`）。(a) `exact` 模式下精确数 + 呼号枚举 + 多项目 → 可拼出某台站/租户的收卡量、各活动规模等运营画像；(b) `approximate` 分桶降低单点精度但**不阻止**跨呼号/跨时间聚合的趋势推断（如某项目大量卡进入 `>50` 桶）。→ 立场：default-deny 缺省把暴露限定在**显式选定精度的租户**（其精确暴露是该租户的主动选择）；approximate 供需要压低单点精度者使用。接受该暴露面，记账于此供后续审计。
- **设置滞后一次同步**：桌面端改了模式但未点同步，查询页用云端旧值（或未配置则不展示）。→ 可接受：精度跟随的是「已同步的」设置，与卡片数据同样依赖同步，语义自洽。
- **分桶阈值漂移**：桌面端日后改 `formatQty` 分桶阈值而 worker 未跟改 → 两端分桶口径不一致。→ 缓解：spec「服务端按租户脱敏」场景钉死阈值，design 注明须与 `useQtyDisplayMode.ts:65-67` 同步；二者均为小常量，且 tasks 3.1 的边界用例兼作回归锚点。
- **精度开关的信任边界 = 租户同步 Key**：`qty_display_mode` 经 `/sync` 由持 Key 者写入（`index.js:487/495` 无值级白名单），故「是否精确暴露」的实际信任边界是**谁持有该租户同步凭据**，而非桌面端 UI——被盗用的同步凭据可把该租户公开面从「不展示/大致」翻成 `exact` 精确暴露。→ 记账：与凭据本身的安全性同级（见 `tenant-isolation` / `local-credential-storage`），本变更不另加值级鉴权（未知值已被 default-deny 兜底）。
- **读设置异常须 fail-safe、保可用、不外泄**：读 `qty_display_mode` 发生异常时必须归入 default-deny。→ 实现须在**读取处局部捕获**并落到「省略 qty 但**照常返回卡片列表**」分支（一次设置读抖动不应使整次查询 500、丢卡片）；顶层脱敏 catch 仅作兜底防泄露（即便未局部捕获也禁止冒泡携带原始 `c.qty`/堆栈/SQL，与「服务端错误响应脱敏」需求一致），非预期路径。

## 迁移计划

无 schema/数据迁移。部署 = 发布新版 `web_query_service`（worker + 客户端构建）。回滚 = 部署上一版 worker（`qty` 不再返回，前端旧版本不渲染），无数据残留。
