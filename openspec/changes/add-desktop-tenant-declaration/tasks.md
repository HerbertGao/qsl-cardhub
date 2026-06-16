# 实现任务

## 1. SyncConfig 加 tenant 字段（D2/D9/D10/D13）

- [x] 1.1 `src/sync/config.rs`：`SyncConfig` 加 `tenant: Option<String>` + `#[serde(default)]`，`Default::default()` 为 `None`
- [x] 1.2 在 `tenant` 字段处加文档注释，显式区分 `client_id`=设备身份（OCC 用）/ `tenant`=申报归属（仅 `X-Tenant-Id` 用、**非写入目标**）（D13 红线）
- [x] 1.3 单测：旧 `sync.toml`（无 `tenant` 字段）解析后 `tenant == None`（复刻既有 `test_config_without_base_version_field_parses` 模式）

## 2. client.rs 发头 + 403 探测 + PingResponse（D1/D3/D6）

- [x] 2.1 `src/sync/client.rs`：抽**纯函数** `tenant_header_value(tenant: Option<&str>) -> Option<String>`（`.map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned)`），三端发头统一调它（**可单测、无需 HTTP mock**——这是 2.6 强锚的可测接缝）；`sync_data` 内部对 `config.tenant.as_deref()` 调它、`Some` 时加 `X-Tenant-Id` 头（**不改签名**，已持 `&config`）
- [x] 2.2 `pull_data`/`test_connection` 各加 `tenant: Option<&str>` 入参，经 `tenant_header_value` 同一条件加 `X-Tenant-Id` 头
- [x] 2.3 `SyncOutcome` 加 `TenantMismatch` 变体；抽**纯函数** `is_tenant_mismatch_body(body: &str) -> bool`（解析 JSON 取 `code=='tenant_mismatch'`、非 JSON/缺 code→`false`，可单测）；`sync_data` 在 `status==403` 时用它判定 → `TenantMismatch`（与 401/409/Err 区分）
- [x] 2.4 `pull_data`/`test_connection` 在 `status==403` 时用 `is_tenant_mismatch_body` 判定 → 返回可识别 `Err` 文案（禁吞成泛化「请求/拉取失败」）；body 非 JSON/缺 code → 该纯函数返 `false` → 退化为泛化 `Err`、**禁 panic**（仿 409 `ConflictBody` 的 `.ok().and_then` 容错先例）
- [x] 2.5 `PingResponse` 加 `#[serde(default)] tenant: Option<String>` + `#[serde(default)] fallback: Option<bool>`（serde default 兼容旧服务端）
- [x] 2.6 单测（**纯函数、无需 HTTP mock，作 cargo-test 强锚**）：`tenant_header_value(None)==None`、`Some("  ")==None`、`Some("bh2ro")==Some("bh2ro")`（向后兼容核心断言：None→不发头）；`is_tenant_mismatch_body` 对 `{"code":"tenant_mismatch"}`→true、`{}`/非 JSON/`{"code":"auth_failed"}`→false（403 status→variant 路由在 `sync_data` 用 `response.status()==403` + 此纯函数判定，逻辑由该纯函数单测兜底）；`SyncResponse`/`PingResponse` 200 路径反序列化形态。**注**：「三端实际出站头有无」属端到端行为，无 HTTP-mock dev-dep 故**不在本期 cargo-test 单测范围**，由 6.5 手工集成验证覆盖

## 3. commands/sync.rs 命令层（D1/D2/D5/D8）

- [x] 3.1 `SyncCmdResult` 加 `TenantMismatch` 单元变体；`execute_sync_cmd` 映射 `SyncOutcome::TenantMismatch → SyncCmdResult::TenantMismatch`
- [x] 3.2 `save_sync_config_cmd` 加 `tenant: Option<String>` 入参，存盘（空字符串→存 `None`，D9）；`SyncConfigResponse` 加 `tenant` 字段；`load_sync_config_cmd` 回显 `tenant`
- [x] 3.3 `save_sync_config_cmd` 加 slug 校验 `^[a-z0-9-]{1,32}$`（锚点 + 连字符末尾）；含大写/非法字符/超长→返回明确 `Err`，**禁**静默转小写、**禁**保存（D5）
- [x] 3.4 `test_sync_connection_cmd`/`restore_from_cloud` 从 config 取 tenant 传给 `test_connection`/`pull_data`；**不**加命令层硬必填（D2 软约束）
- [x] 3.5 单测：slug 校验拒（`BH2RO`/`a b`/`x!`/33 位）与通过（`bh2ro`/`tenant-1`）用例

## 4. ts-rs 纳入 sync 类型（D4）

- [x] 4.1 给 `SyncCmdResult`/`SyncResponse`/`SyncStats`/`RestoreResult`/`SyncConfigResponse`/`PingResponse`（commands+client）与 `ExportStats`（`src/db/export.rs`）加 `#[cfg_attr(feature="ts-rs", derive(TS))]` + `ts(export)`（**必须含 `SyncStats`**：`SyncResponse.stats: Option<SyncStats>` 编译期强制，漏则 `export_bindings` 编译失败）；逐字段给所有 `Option<i64>` 加 `#[ts(type="number")]`：`SyncResponse.server_version`、`SyncCmdResult::Success.server_version`、`SyncCmdResult::Conflict.server_version`、`RestoreResult.server_version`、`SyncConfigResponse.base_version`
- [x] 4.2 `tests/export_bindings.rs` 加上述类型的 `use` import + `Type::export_all(&config)` 调用
- [x] 4.3 **前置**：`cargo test export_bindings --features ts-rs` 须**编译并运行成功**（漏给任一级联类型 derive(TS) 会编译失败、根本不产出 `.ts`——编译成功是 diff 有对象的硬前提，不可跳过）；再逐字 diff 生成的 `SyncCmdResult.ts` 形状（discriminant=`status`、值 snake_case、单元变体不带多余字段、含 `tenant_mismatch`、i64→number），确认与原手写 union 等价（D4 验收闸）

## 5. 前端对接（D4/D5/D6/D11）

- [x] 5.1 删**两个消费者**的手写类型改 `import` 生成文件：`web/src/views/DataTransferView.vue` 与 `web/src/views/CardManagementView.vue` 都手写了 `SyncCmdResult`/`RestoreResult`，两处都删改 import（漏一个=双份漂移源 + 第四态静默误处理）
- [x] 5.2 配置表单（DataTransferView）加「租户代码」输入框 + 即时 slug 校验提示；`save_sync_config_cmd` invoke 调用传 `tenant`
- [x] 5.3 **两个文件**的四态 `switch` 都加 `tenant_mismatch` case（引导文案）+ `assertNever(result)` 穷尽检查（替换各自的运行时 default 兜底为编译期穷尽，D11）；CardManagementView 现有 default 兜底须一并清掉
- [x] 5.4 测连成功展示「已认证租户：xxx」；`fallback=true` 时提示「凭据命中默认租户兜底（请确认 Key 归属）」（信息提示、非 mismatch，D6）
- [x] 5.5 前端加编译期断言（把 `{status:'auth_failed'}` 字面量赋给生成类型）钉住 union 形状

## 6. 验证（强锚）

- [x] 6.1 `cargo test` 全绿（强锚 = 纯函数 + 序列化单测：1.3 serde default、2.6 `tenant_header_value`/`is_tenant_mismatch_body`/403 分流、3.5 slug 校验——**均不依赖 HTTP mock**）
- [x] 6.2 `cargo test export_bindings --features ts-rs` 生成无意外漂移（仅新增/预期内变更）
- [x] 6.3 前端 `pnpm run type-check` + `pnpm run lint` + 构建全绿
- [x] 6.4 纯本地模式回归：以**调用图审查**确认 tenant 校验/发头逻辑只在云同步路径（`api_url` 非空、经 client.rs）触发、导出导入路径不经 client（D12，此为代码不变量、非可执行单测，避免对不可达路径写恒真断言）；可执行侧由 2.6 `tenant_header_value(None)==None` 兜底覆盖「无 tenant→无头」
- [ ] 6.5 集成验证三端发头条件（None 不发 / Some 发）+ `/sync` 403 四态分流 + /pull·/ping 403 文案（对 4-C1 已上线 worker 实测：填错 tenant→403 tenant_mismatch）。**前置**：实测须用表驱动凭据 Key 而非 env.API_KEY 兜底 Key——兜底解析为默认租户，declared 须 ≠ 默认租户才能触发 403

## 7. 发布与验收（部分用户自跑）

- [ ] 7.1 `cargo tauri build` 出包
- [ ] 7.2 真机验收：存量 bh2ro 用户升级后填 `bh2ro` 同步 200；不填也能同步（软约束）；填错租户→可分辨 403 引导
- [ ] 7.3 `openspec-cn archive add-desktop-tenant-declaration`（增量并入 `cloud-database-support` 主规范）
