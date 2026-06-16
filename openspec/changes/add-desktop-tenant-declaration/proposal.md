## 为什么

阶段 4-C1 已上线服务端交叉校验：worker 读请求头 `X-Tenant-Id`，与写凭据 `resolveTenant(key)` 解析出的租户比对，不一致返 `403 tenant_mismatch`，**缺头/空头向后兼容放行**。但桌面端目前**从不发送** `X-Tenant-Id`，也无处配置「租户代码」——服务端这道校验对官方客户端等于空转。本变更补齐桌面端的客户端另一半：让用户申报所属租户、随同步请求带上申报头、并把服务端的 403 转成用户可分辨的引导。

红线沿用 4-C1：归属真源**永远**是 `key→tenant`（写凭据决定租户）；`X-Tenant-Id` 只参与「申报 + 交叉校验 + 回显」，**绝不**当写入/读取目标（不复活信任客户端自报归属）。

## 变更内容

- **新增** `SyncConfig.tenant: Option<String>`（`#[serde(default)]`，兼容旧 `sync.toml`，与 `base_version` 同模式），落盘 `sync.toml`；写凭据仍存 OS 凭据库不变。
- **新增** 客户端在 `/sync`、`/pull`、`/ping` 三端发送 `X-Tenant-Id`——**仅当 `tenant` 为 `Some` 且 trim 后非空时才发头**；`None`/空时不发，行为与今天逐字一致（软约束，不强制必填）。
- **新增** 第四态：`/sync` 的 `403 tenant_mismatch` 解析为 `SyncCmdResult::TenantMismatch`（结构化、可分辨）；`/pull`、`/ping` 的 403 走 `Err(可识别文案)`（须显式探测 `code:'tenant_mismatch'`，禁吞成泛化错）。
- **新增** 配置表单的「租户代码」输入框 + 客户端 slug 校验 `^[a-z0-9-]{1,32}$`（落命令层，逐字对齐服务端 schema CHECK）；大写**拒绝并报错**，不静默转小写。
- **新增** `/ping` 测试连接回显「已认证租户」：扩 `PingResponse` 加 `tenant`/`fallback`（`#[serde(default)]` 兼容旧服务端），测连成功展示认证租户；`fallback=true` 时提示「凭据命中默认租户兜底（请确认 Key 归属）」（信息提示、**非 mismatch**——真正的不匹配由 /ping 的 403 捕获）。
- **改造** 把 sync 相关类型纳入 ts-rs 自动生成（含 `SyncStats`——`SyncResponse.stats` 编译期级联强制），**删除前端两个消费者**（`DataTransferView.vue` 与 `CardManagementView.vue`，均 invoke `execute_sync_cmd`/`restore_from_cloud`）手写的 `SyncCmdResult`/`RestoreResult` 类型定义，改 import 生成文件（消灭手写/生成双份漂移源）。两处 `switch` 均改为穷尽检查（`assertNever`）、新增 `tenant_mismatch` case 显式处理。

本批次追加（PR review 期间用户要求的同步配置 UX 改进，均挂在同一「云端数据同步」能力下）：
- **新增** 「API 地址」改为可手动输入的下拉框（预设空值 + 官方云 `https://qsl.herbertgao.me`）；选中官方云预设时租户代码与 API Key **必填**（UI 层必填，不改后端软约束）。
- **新增** 「配置迁移」一键复制/一键粘贴：导出/导入 API 地址·租户代码·API Key 的 Base64 字符串（**含明文 Key**，仅供本机设备间迁移）；新增后端命令 `export_sync_config_string_cmd`/`import_sync_config_string_cmd`（base64 + 凭据读写在 Rust，导入复用 `save_sync_config_cmd`，**不含 client_id**）。
- **改造** 租户代码输入完毕（失焦）**自动格式化**为合法 slug（小写/去非法字符/截断 32）——把原「拒绝不转换」改为「可见格式化」，省用户手动修改（用户要求）。
- **改造** 「测试连接」改测**表单当前填写的值**（无需先保存）；API Key 留空回落已保存凭据（`test_sync_connection_cmd` 加入参）。
- **修复** 客户端 ID 改用只读 `<div>` 展示（规避 WKWebView 对禁用 `<input>` 在 re-mount 时文字下缘裁切的渲染 bug）；API Key 已保存时占位符提示「已保存…」（空框自解释、非丢失）。

不破坏：纯本地模式（`api_url` 为空）完全不触发任何租户逻辑；存量 bh2ro 用户升级后、填 tenant 前同步行为逐字不变。无 D1 迁移、无服务端改动。

## 功能 (Capabilities)

### 新增功能
<!-- 无新增能力：客户端申报是对既有「云端数据同步」能力的扩展 -->

### 修改功能
- `cloud-database-support`: 「云端数据同步」需求新增租户申报维度——配置租户代码、同步/恢复/测连携带 `X-Tenant-Id`、`403 tenant_mismatch` 可分辨结果、测连回显认证租户、客户端 slug 校验；并扩充配置 UX——API 地址下拉预设 + 官方云必填、租户代码失焦自动格式化、测试连接测表单值、配置导出/导入（一键复制/粘贴）。

## 影响

- **Rust**：`src/sync/config.rs`（`tenant` 字段 + serde default + 兼容测试）、`src/sync/client.rs`（三端发头、`PingResponse` 扩字段、`pull_data`/`test_connection` 加 `tenant: Option<&str>` 参、403 显式分支、`SyncResponse`/`SyncStats` derive TS）、`src/commands/sync.rs`（`SyncCmdResult` 加 `TenantMismatch`、`save_sync_config_cmd` 加 tenant 入参、slug 校验、相关类型 derive TS）、`src/db/export.rs`（`ExportStats` derive TS，被 `SyncCmdResult` 级联强制）。
- **ts-rs**：`tests/export_bindings.rs` 加 import + `export_all`；遵循既有 `ts-rs-codegen` 能力（`i64` 字段须 `#[ts(type="number")]`、tagged enum 渲染须逐字 diff 验收）。无新需求，属应用既有能力。
- **前端**：`web/src/views/DataTransferView.vue`（删手写类型改 import、加租户输入框 + 校验提示、四态穷尽 switch、测连展示租户/fallback）；`web/src/views/CardManagementView.vue`（同为 `execute_sync_cmd`/`restore_from_cloud` 消费者——删手写类型改 import、四态穷尽 switch + 补 `tenant_mismatch` case、清掉现有 default 兜底）。
- **追加 Rust**：`src/commands/sync.rs` 加 `export_sync_config_string_cmd`/`import_sync_config_string_cmd`（`ExportedSyncConfig` 内部 DTO + `base64` 编解码）、`test_sync_connection_cmd` 改为接 `(api_url, api_key, tenant)` 测表单值；`src/main.rs` 注册两命令；依赖 `base64`（已在 Cargo.toml）。
- **追加前端**：`DataTransferView.vue` API 地址 `el-select`(可手填) + 官方云必填联动、配置迁移复制/粘贴按钮、租户代码 `@blur` 自动格式化、客户端 ID 改只读 `div`（`.readonly-field` 样式）、API Key 动态占位符、`form-hint` 宽度修正使提示恒在字段下方。
- **破坏性签名**：`save_sync_config_cmd`、`pull_data`、`test_connection`/`test_sync_connection_cmd` 入参变更，需同步改前端 invoke 调用点。
- **兼容期遗留**：本变更把「云同步必填 tenant」降级为软约束，靠服务端缺头放行续命；兼容期终止点（缺头→403 + 客户端硬必填）记入 design ADR，留待后续变更收紧。
