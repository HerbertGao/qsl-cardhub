## 为什么

公开「按呼号查询收卡信息页」（web_query_service）目前不展示每张卡的「数量(qty)」——`cloud-backend-api` 规范当前**明确禁止**查询接口返回 `qty`（数据最小化约束）。但桌面端已有「数量显示模式」`qty_display_mode`（`exact` 精确 / `approximate` 大致），且该设置已随全量同步进入云端 `app_settings` 表（按租户隔离）。用户希望查询页展示数量，且**精度跟随桌面端设置**：桌面选「大致」则查询页也大致，选「精确」则精确。同时公开页**禁止泄露精确收卡数**——大致模式下脱敏必须在服务端完成。

## 变更内容

- 按呼号查询接口（`GET /api/query`）的响应**按租户 `qty_display_mode` 决定是否及如何返回 `qty`**（服务端脱敏）：`exact` → 原始整数；`approximate` → `≤10` / `≤50` / `>50` 字符串（分桶阈值与桌面端 `formatQty` 一致）；**未配置 / 未知值 / 读取异常 → 不返回 `qty`**（default-deny，公开面不暴露未授权数量）。
- 解除 `cloud-backend-api` 规范对查询响应返回 `qty` 的禁令，改为「按租户脱敏后**有条件**返回（仅显式配置过精度的租户）」。
- 查询结果页（`ResultList.vue`）**当接口返回 `qty` 时**每张卡始终展示数量（不受 `status`/分发详情条件限制，区别于仅特定状态才显示的分发/退卡详情块）；接口未返回 `qty`（租户未配置精度）时不展示数量。
- 桌面端、`/sync` 导入端点、D1 schema、`app_settings` 同步链路**零改动**——`qty_display_mode` 早已随同步上云并按租户存储。

## 功能 (Capabilities)

### 新增功能

（无）

### 修改功能

- `cloud-backend-api`: 「按呼号查询收卡信息接口」需求——查询成功响应**有条件**包含脱敏后的 `qty`（仅租户配置过精度时）；解除原「禁止返回 qty」约束；新增「服务端按租户 `qty_display_mode` 脱敏（未配置/未知/读取异常则不返回）」与「结果页在返回 qty 时始终展示」场景。

## 影响

- **代码**：`web_query_service/src/worker/index.js`（查询端点结果映射新增 `qty` + 新增脱敏函数 + 读取租户 `qty_display_mode`）；`web_query_service/src/client/components/ResultList.vue`（`CardItem` 接口加 `qty` + 始终渲染）。
- **隐私**：`approximate` 模式脱敏在服务端完成，网络响应体不含精确数字，扒包亦不可还原。
- **默认行为（default-deny）**：从未在桌面端配置过 `qty_display_mode` 的租户，公开查询页**不展示**任何数量；只有显式在桌面端选定精度（精确 / 大致）并同步一次后，查询页才展示（精确数 / 分桶）。逆转既有「禁止返回 qty」的数据最小化约束**仅对显式配置过的租户**生效。
- **聚合推断暴露面**：对配置过精度的租户，公开逐卡数量（即便分桶）结合呼号枚举可被聚合推断收卡量画像；详见 design 风险节，立场为接受（精确暴露是该租户主动选择，default-deny 限定暴露范围）。
- **无变更**：D1 schema 与迁移、`/sync` 与 `/pull` 端点、桌面端 Rust/前端同步代码、`app_settings` 表结构。
