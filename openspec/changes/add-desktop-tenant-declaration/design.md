## 上下文

阶段 4-C1 已上线服务端 `crossCheckTenant(env, key, declared)`（worker `index.js`）：读 `X-Tenant-Id`（收到后 `.trim()`），与 `resolveTenant(key)` 解析出的租户比对——`declared` 空（缺头/空串）→ 向后兼容放行；非空且 `!== 解析租户` → `403 {code:'tenant_mismatch'}`。已接入 `/sync`、`/pull`、`/ping`（`/ping` 还回显 `{tenant, fallback}`）。**红线**：归属真源恒为 `key→tenant`，`X-Tenant-Id` 只申报/校验/回显，绝不当写入目标。

桌面端现状（核对代码）：`SyncConfig{api_url, client_id, last_sync_at, base_version}`（`src/sync/config.rs`），无 tenant 字段；`src/sync/client.rs` 的 `sync_data(config, api_key, force)`、`pull_data(api_url, api_key)`、`test_connection(api_url, api_key)` **只发 Authorization**；`SyncOutcome`（client）/`SyncCmdResult`（command，`#[serde(tag="status", rename_all="snake_case")]`）均为三态 Success/AuthFailed/Conflict；`PingResponse` 无 tenant/fallback 字段。sync 类型**不走 ts-rs**，前端 `DataTransferView.vue:444-458` 手写 `SyncCmdResult`/`RestoreResult`，`:653-674` switch 带 default 兜底。两模式靠 `api_url` 空否派生（非空=云同步、空=纯本地）。

## 目标 / 非目标

**目标：**
- 桌面端申报租户、三端（/sync·/pull·/ping）携带 `X-Tenant-Id`，对接 4-C1 服务端校验。
- `/sync` 的 403 升级为可分辨的第四态 `TenantMismatch`；/pull·/ping 的 403 给可识别 `Err` 文案。
- sync 类型纳入 ts-rs，消灭前端手写/生成双份漂移源。
- 客户端 slug 校验逐字对齐服务端；测连回显认证租户。
- 存量 bh2ro 用户与纯本地用户行为零回归。

**非目标：**
- **不做**命令层硬必填（软约束，详见决策 D2）。
- **不改**服务端、**不做** D1 迁移。
- **不动**登录窗/标题栏徽章（属 4-C3）、CLI mint/自托管文档（属 4-C4）、路径路由（属 4-B、已闭环）。
- **不改**清除配置删整文件的既有语义（阶段1遗留，独立清理项）。

## 决策

### D1：第四态仅在 `/sync` 建枚举，/pull·/ping 走可识别 Err
`SyncOutcome`（client）与 `SyncCmdResult`（command）各加 `TenantMismatch` 单元变体，仅 `execute_sync_cmd` 路径产出。`/sync` 是高频常态操作，403 值得结构化让前端精准引导。`/pull`（`restore_from_cloud`）、`/ping`（`test_sync_connection_cmd`）是低频一次性操作，多建枚举态属投机对称性（YAGNI）。
- **代价（非零工作量）**：`pull_data`/`test_connection` 当前把所有非 2xx 统一映射 `Err`，403 会被吞成泛化「请求失败」。本次**必须**在这两个函数显式探测 `status==403 && body.code=='tenant_mismatch'`，返回可读文案。
- 替代方案：三端都建枚举态——否决，前端分支翻倍、收益仅对低频路径。

### D2：软约束，不做硬必填（修正初始倾向）
「云同步必填 tenant」与「存量用户填前逐字一致、不被挡死」**不可兼得**。本次选后者：空 tenant = 不发头 = 服务端兼容放行 = 旧行为逐字一致。命令层**不**加「api_url 非空但 tenant 空 → Err」拦截。输入框是「可填」非「必填才能用」。
- 硬必填 + 缺头收紧推迟到 **D7 兼容期终止点**（见 ADR）。本次加硬必填会亲手制造红线禁止的「存量 bh2ro 用户升级即被挡死」。
- 替代方案：命令层硬必填——否决，与零回归约束直接冲突。

### D3：仅 `Some(非空 trim)` 才发头；签名最小变更
发头条件统一：`tenant.as_deref().map(str::trim).filter(|s| !s.is_empty())`，`None`/空白一律不发。
- `sync_data` 已持 `&config` → 内部取 `config.tenant` 加头，**不改签名**。
- `pull_data`/`test_connection` 当前只收 `(api_url, api_key)` → 各加一个参 `tenant: Option<&str>`（不把整个 config 耦合进 client 纯函数）。这是破坏性签名变更，牵动 `commands/sync.rs` 调用点。
- 替代方案：发空头 `X-Tenant-Id:`（服务端 trim 后也放行）——否决，平白增加可观测差异；不发更干净、对齐「逐字一致」。

### D4：ts-rs 闭包（含级联强制类型）+ 删两个前端消费者的手写 union
纳入集 = `SyncCmdResult` + 其字段级联强制的 `SyncResponse` + `SyncStats` + `ExportStats`（`src/db/export.rs`）+ `RestoreResult` + `SyncConfigResponse` + `PingResponse`（D6 要扩它）。
- **必须含 `SyncStats`**（修正初稿「不含」的错误）：`SyncCmdResult::Success.response: SyncResponse`，而 `SyncResponse.stats: Option<SyncStats>`（`client.rs:70`）——ts-rs 编译期要求 derive(TS) 类型的**所有字段类型**也 derive(TS)，`Option<SyncStats>` 强制 `SyncStats: TS`。漏掉 `SyncStats` 则 `cargo test export_bindings` 直接编译失败。`SyncStats` 是 4×u32、与 `ExportStats` 同形，加 derive 最省（替代：`ts(skip)` `SyncResponse.stats`——否决，仓内无 `ts(skip)` 先例且会让生成类型偏离 serde 形态）。
- `export_all` **会**递归导出依赖类型；需手动维护的是 `tests/export_bindings.rs` 的**根类型调用列表**（每个要单独出 `.ts` 文件的类型须显式 `export_all`）。
- 字段级 number 标注（i64 默认渲染 bigint、与 serde_json number 不兼容，遵循 `ts-rs-codegen`）：`SyncResponse.server_version`、`SyncCmdResult::Success.server_version`、`SyncCmdResult::Conflict.server_version`、`RestoreResult.server_version`、`SyncConfigResponse.base_version` 等所有 `Option<i64>` 字段**必须** `#[ts(type="number")]`。
- **必删两处手写**：前端有**两个** `execute_sync_cmd`/`SyncCmdResult` 消费者——`DataTransferView.vue`（手写 union + switch）与 `CardManagementView.vue`（同样手写 union + switch + `default` 兜底）。两者都须删手写改 `import` 生成文件、补 `tenant_mismatch` case + `assertNever`，否则漏改的那个会把第四态落进 default 静默误处理。

### D5：slug 校验落命令层，大写拒绝不转换
正则 `^[a-z0-9-]{1,32}$`（锚点必须，否则 `bh2ro!` 部分匹配漏过；连字符在字符类放末尾），落 `save_sync_config_cmd` 入口（唯一写入口，错误直接回前端）。**不**放 `config.rs`（纯持久化层，ts-rs 导出的 SyncConfig 不该耦合校验）。前端可加同款正则做即时反馈，但权威校验在命令层。
- 大写**拒绝 + 报错**，不静默转小写：避免「填 BH2RO 存盘变 bh2ro、回显与输入不一致」的认知错位；服务端比小写解析值，本地就该挡下。

### D6：扩 `PingResponse` 回显认证租户
给 `PingResponse` 加 `#[serde(default)] tenant: Option<String>` + `#[serde(default)] fallback: Option<bool>`（serde default 保旧服务端/缺字段降级为 None）。测连成功展示「已认证租户：xxx」。
- mismatch 的发现途径是 **/ping 自身的 403**：/ping 已走 `crossCheckTenant`（readonly），declared 非空且 ≠ 解析租户时直接返 403 `tenant_mismatch`（测连按 D1 走可识别 Err 文案）。D2 软约束下用户填错 tenant 不被保存层挡住，这条 403 测连路径是发现 mismatch 的主要途径。
- `fallback` 仅作**信息提示、不表示 mismatch**：/ping 返 200 时若 declared 非空则必 == 解析租户（否则早已 403），故 `fallback=true`（凭据经 env.API_KEY 兜底命中默认租户、非表驱动）只说明「凭据命中默认租户兜底」，提示用户确认 Key 归属即可——**禁**表述成「与申报不一致」（那是 403 的职责）。

### D8：tenant 并入 `save_sync_config_cmd`
`save_sync_config_cmd(api_url, api_key)` → 加 `tenant: Option<String>` 入参（与 api_url 同生命周期、同表单提交），不新建命令。`SyncConfigResponse` 加 `tenant` 字段回显。破坏性签名变更，需同步改前端 invoke。

### D9：清空 tenant = 降级回兼容模式，静默允许
用户把已填 tenant 清空保存时静默允许（兼容期内合法）。落盘层规整：收到空字符串存为 `None` 而非 `Some("")`；D3 的 `.filter(非空)` 是最后防线（已覆盖）。

### D10–D13：次要决策（补编号，消除 tasks 悬空引用）
- **D10**：`SyncConfig.tenant` 用 `Option<String>` + `#[serde(default)]`，兼容旧 `sync.toml`（与 `base_version` 同模式），`Default` 为 `None`。
- **D11**：前端四态 `switch` 用 `assertNever(result)` 编译期穷尽检查替代运行时 `default` 兜底——漏 case 时 type-check 报错而非运行时静默（两个消费者文件都适用，见 D4）。
- **D12**：纯本地模式（`api_url` 为空）回归保护——所有 tenant 校验/发头逻辑限定在 `api_url` 非空分支，留强锚断言「api_url 空时 tenant 逻辑零触发、导出导入路径逐字不变」。
- **D13**：在 `config.rs`/ADR 注释显式区分 `client_id`=设备身份（OCC 用）/ `tenant`=申报归属（仅 `X-Tenant-Id` 用、**非写入目标**），防后续维护者复活 4-C1 红线禁止的信任客户端自报归属。

### D14–D18：PR review 期间追加的同步配置 UX 决策

- **D14｜API 地址下拉预设 + 官方云必填联动**：API 地址改 `el-select`（`filterable allow-create`，预设空值 + `https://qsl.herbertgao.me`），用户可选预设或手填任意自托管地址。选中官方云预设（`api_url === OFFICIAL_CLOUD_URL`）时租户代码 + API Key 在**前端表单层**必填（保存校验）。**这是 UI 层必填，不改 D2 后端软约束**——自托管/自定义地址仍走软约束，官方云的必填只是表单 UX 引导（域名是配置常量，非内置默认行为）。
- **D15｜配置导出/导入字符串（含明文 Key）**：新增 `export_sync_config_string_cmd`（读 `sync.toml` + 凭据库 → JSON{api_url,tenant,api_key} → Base64）/ `import_sync_config_string_cmd`（Base64 解码 → 复用 `save_sync_config_cmd` 落盘，含 slug 校验）。**用户明确批准导出含明文 Key**（Base64 仅编码非加密、串等同密钥，界面提示敏感性、仅本机迁移）；base64 + 密钥处理留在 Rust（前端只搬运 Base64 串）。**导出不含 `client_id`**（设备本地自动生成，跨设备复制会撞 ID，语义错）。替代（不含 Key）被用户否决。
- **D16｜租户代码失焦自动格式化（覆盖 D5）**：原 D5 定「拒绝不转换」（怕静默改值困惑用户）。用户要求「输入完毕自动格式化、省手动改」→ 改为 `@blur` 时 `trim→小写→去非法→截断32`。**原顾虑不成立**：格式化发生在失焦、当面可见，非保存时静默。命令层 `validate_tenant_slug` 仍作权威兜底（格式化后恒过）。
- **D17｜测试连接测表单值（不需先保存）**：`test_sync_connection_cmd` 由无参改为 `(api_url, api_key, tenant)`，测**表单当前值**而非已保存配置（原行为「未配置同步服务」对填完未保存的用户反直觉）。API Key 表单为空时回落已保存凭据（支持已存 Key 改其它项时测试）。
- **D18｜客户端 ID 改只读 `<div>`（修 bug#2）**：客户端 ID 原用 `<el-input disabled>`，WKWebView（Tauri 原生壳）对禁用输入框在 v-if re-mount 时裁切文字下缘（Chromium 量指标健康、不复现，确认原生壳专属）。客户端 ID 本就只读/自动生成 → 改 `.readonly-field` div（块级 + padding 留白 + 自然行高），从根上不走 input 渲染路径。**此 bug 暴露了 Chromium-mock 测试层抓不到 WKWebView 专属渲染 bug，是测试 harness 提案要覆盖「真实壳层」的实证依据**（见 [[client-test-harness 提案]]）。

## 风险 / 权衡

- **[ts-rs tagged enum 渲染漂移]** → ts-rs 12 渲染 `#[serde(tag="status")]` 单元变体（`AuthFailed→{status:"auth_failed"}`）历史上有版本差异（可能退化成裸字符串）。**缓解**：纳入后逐字 diff 生成的 `.ts` 与现有手写 union 形状（discriminant=`status`、值 snake_case、单元变体不带多余字段、新增 `tenant_mismatch`），此 diff 是 D4 验收闸；前端加编译期断言（把 `{status:'auth_failed'}` 字面量赋给生成类型）钉住形状。
- **[兼容期变永久后门]** → 缺头放行若无终止条件，多租户隔离形同虚设。**缓解**：D7 ADR 明确终止点（见迁移计划）。
- **[纯本地模式被波及]** → 加校验/发头时手滑触碰纯本地路径。**缓解**：所有 tenant 逻辑限定在 `api_url` 非空分支；强锚测试断言「api_url 空时 tenant 校验/发头完全不触发」。
- **[存量 sync.toml 解析崩溃]** → `tenant` 字段漏 `#[serde(default)]` 会让旧 toml 解析 panic。**缓解**：强锚测试「旧 toml（无 tenant）解析后 tenant==None」，复刻既有 `test_config_without_base_version_field_parses`。
- **[403 被吞成泛化错]** → /pull·/ping 现有逻辑统一映射 Err。**缓解**：显式探测 `code:'tenant_mismatch'` 分支（D1）。
- **[client_id 与 tenant 混淆]** → 后续维护者把 tenant 当写入路由依据，复活 4-C1 红线禁止的信任客户端自报。**缓解（D13）**：在 `config.rs`/ADR 注释显式区分 `client_id`=设备身份（OCC 用）、`tenant`=申报归属（仅 X-Tenant-Id 用、非写入目标）。

## 强锚测试点（实现必须留可执行验证）

1. 可执行兜底 `tenant_header_value(None)==None`（无 tenant→不发头，钉死存量逐字行为，向后兼容核心断言）；「三端实际出站头无 `X-Tenant-Id`」属端到端、由 6.5 手工集成验证。
2. 可执行兜底 `tenant_header_value(Some("bh2ro"))==Some("bh2ro")`；「三端实际出站头有 `X-Tenant-Id: bh2ro`」由 6.5 手工集成验证。
3. 旧 `sync.toml`（无 tenant 字段）解析后 `tenant==None`。
4. slug 校验：`BH2RO`/`a b`/`x!`/33 位 → 拒绝；`bh2ro`/`tenant-1` → 通过。
5. `/sync` 403 `tenant_mismatch` → `SyncCmdResult::TenantMismatch`（不混入 401/409/Err）。
6. `/pull`·`/ping` 403 `tenant_mismatch` → Err 文案含可识别字样。
7. 前端四态 switch 穷尽（`assertNever`），default 分支命中数为 0。
8. 生成的 `SyncCmdResult.ts` 与预期 union 形状逐字一致（含 i64→number）。

## 迁移计划（部署 / 回滚）

- 纯客户端变更，**无 D1 迁移、无服务端改动**。服务端 4-C1 已部署且缺头兼容放行，4-C2 可独立上线、不破存量。
- 部署 = 桌面端发版（Tauri build）。回滚 = 退桌面端版本（旧版不发头，服务端照常放行）。
- 存量 bh2ro 用户升级后：填 `bh2ro` 一次（key 不变，阶段1 已 seed `bh2ro-key`=sha256(trim(API_KEY))）；不填也能继续同步（软约束）。

### ADR — D7：兼容期终止点（本次只记录、不实施）
本变更把「云同步必填 tenant」从硬约束降级为软约束，靠服务端「缺头放行」续命。**兼容期终止条件**：待全量桌面端用户升级并确认（预计在 4-C4 收尾后），由**后续变更**将服务端缺头从「放行」改为「`403 tenant_required`」，同步触发客户端命令层硬必填。在此之前，缺头放行是**临时**兼容措施，不得视为永久行为。此 ADR 防止缺头放行无声固化为多租户隔离的后门。

## 待解决问题

- 清除配置删整 `sync.toml`（连带 client_id）与「保留 client_id」措辞的既有矛盾——阶段1遗留，本次不扩范围，记为独立清理项。
- 既有主规范「云端 API 规范」需求（`cloud-database-support` 的「按 client_id 隔离不同客户端数据」表述）是 multi-tenant 前的遗留，与 key→tenant 真源不一致；权威服务端契约 `cloud-backend-api` 已正确区分 `client_id`=设备 / `tenant`=组织（D13）。本次 delta **不触及**该需求（无归档矛盾），其修订归 **4-C4 自托管 API 规范文档**变更，非本期。
- `REQUIRE_TENANT_HEADER` / 服务端缺头收紧的具体开关形态——留 D7 兼容期终止变更定。
- `SyncResult` 结构（`commands/sync.rs`）是 `SyncCmdResult::Success` 字段的死副本、全仓零引用——已**正确排除**出 ts-rs 纳入集（不 derive(TS)）；本期不删（属既有死代码、非本变更引入），记为独立清理项，提醒后续维护者勿误加 derive(TS)。
