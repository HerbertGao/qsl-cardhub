# 实现任务

## 1. syncStore 共享响应式单一事实源（D3）

- [x] 1.1 新建 `web/src/stores/syncStore.ts`：响应式**完整字段** `apiUrl/tenant/hasApiKey/lastSyncAt/clientId/baseVersion`（补 clientId/baseVersion，否则丢设置页展示/同步后更新）+ 派生 `mode = computed(() => apiUrl.value ? 'cloud' : 'local')` + `canSync = computed(() => !!apiUrl.value && hasApiKey.value)`；`loadError` ref（**渲染揭幕门改用 App 自有 `bootReady`，不放 store**——见 3.1 竞态修复）
- [x] 1.2 `load()`：调 `load_sync_config_cmd` 填充；**reject 分支**——try/catch 吞异常（catch 置 `loadError` 真、**不重抛**，故 `await load()` 必 resolve、调用方引导序总能揭幕）、**禁止**按空 apiUrl 判首启/置 onboarding 标志
- [x] 1.3 `applyConfig(resp)`：用 `save_sync_config_cmd`/`import_sync_config_string_cmd` **返回的 `SyncConfigResponse`** 就地更新（含后端规整后 tenant，非「刚提交的值」）；回写**按操作类型分**——`applySyncSuccess(r)`：sync 成功回写 `lastSyncAt`+`baseVersion`（`SyncCmdResult` 有 `sync_time`）；`applyRestoreSuccess(r)`：restore 成功**仅**回写 `baseVersion`（`RestoreResult` 无 `sync_time`，**禁止**置空 `lastSyncAt`）；`reset()`：清 `apiUrl/tenant/hasApiKey/lastSyncAt/baseVersion`、**保留 `clientId`**
- [x] 1.4 **草稿/已保存分离**：store 持已保存态，DataTransferView 的 `syncForm` 持编辑草稿；徽章/mode/canSync 只读 store，**禁止**把 syncForm 接 store（编辑未保存不改模式，红线）
- [x] 1.5 只封装既有命令 + 4 消费者所需字段，不预加未来方法（YAGNI）

## 2. AuthGateView 首启网关（D1/D2/S5）

- [x] 2.1 新建 `web/src/views/AuthGateView.vue`：应用内首屏遮罩（全屏覆盖、阻断式、**无关闭按钮**），两个大按钮「云同步」「纯本地」+ 简短模式说明
- [x] 2.2 「云同步」handler：`markModeSelected()` + **先同步设 `activeMenu='data-config-data-transfer'`（不只靠非 immediate 的 navigateTo watcher）再揭幕主视图** + 关网关，**该分支不触发 `runPrinterCheck()`**（已选落点，打印机检测会覆盖设置页）——避免揭幕帧 `activeMenu` 仍是默认 `cards` → 卡片页空项目弹「新建项目」（review RC#2）；「纯本地」handler：`markModeSelected()` + 关网关 → 关闭回调里**才**跑 `runPrinterCheck()`。实现：网关 `emit('select', choice)`，App.vue `onGateSelect` 落地置标志/切菜单/揭幕/补检测
- [x] 2.3 onboarding 标志读写工具（`web/src/utils/onboarding.ts`）：`isModeSelected()`/`markModeSelected()`，key `qsl:onboarding:mode-selected`="1"，localStorage；**不存所选模式、永不主动清**

## 3. App.vue 三态渲染 + 徽章 + onMounted 重构（D4/D5/D6/D7/S6）

- [x] 3.1 **四态互斥渲染（D5）**：顶层 `!bootReady`→loading / `loadError`→错误分支(D7) / `gateVisible`→仅 `AuthGateView` / 否则→主界面。**主内容视图（含卡片页）在 loading/网关期禁止挂载**。**揭幕门用 App 自有 `bootReady`（非 store 加载态）**——修 Codex 竞态：store 加载态在 `load()` 内部早于 App 网关赋值翻转 → 会闪一帧 `gateVisible=false` 的主界面抢先挂载；`bootReady` 仅在 `bootstrapAfterLoad` 把 gateVisible/loadError 全定下后同一同步块置真
- [x] 3.2 el-header 右侧租户徽章：三态文案（`纯本地` / tenant值 / `云同步`）取自 **syncStore 已保存态**，点击进入 `data-config-data-transfer`；D5 互斥渲染保证 `bootReady` 前不渲染
- [x] 3.3 **抽 `runPrinterCheck()`（D6）**：把既有 `get_printer_config` 无打印机→`activeMenu='print-config-printer'` 抽成独立函数，供**无网关分支** + **网关「纯本地」回调**两处调；「云同步」分支不调
- [x] 3.4 **更新检查无条件（D6）**：`silentCheckUpdate()` + `setInterval` 与网关正交、无条件注册，**未**被网关 return 吞掉；`silentCheckUpdate` 内 `gateVisible` 为真时抑制更新通知弹窗
- [x] 3.5 **onMounted 执行序（D1.4/D7）**：`await syncStore.load()` → `bootstrapAfterLoad()`〔`loadError` 则跳过迁移/网关、揭幕到错误分支；否则 存量迁移（`!isModeSelected() && apiUrl 非空` → `markModeSelected()`，**先于**网关判定）→ 计算网关条件 →（无网关）`runPrinterCheck()` → **最后**置 `bootReady` 揭幕〕→ 无条件更新检查 + `setInterval`。`retryLoad()` 复用 `bootstrapAfterLoad`

## 4. DataTransferView 重组 + 收口 syncStore（D2/D3）

- [x] 4.1 租户 & 云端同步 card **移至顶部** + 标题改名「租户 & 云端同步」；数据导出/导入 card 在下（在 DOM 里实际前置，保持 `.page-content` 普通块级流——勿用 flex 容器，会逼卡片压缩出逐卡滚动条）
- [x] 4.2 云配置**展示态**收口到 `syncStore`（删自身 `syncConfig` ref + 不再自取 `load_sync_config_cmd`，改 `hydrateForm` 回填草稿）；保留 `syncForm` 作表单草稿；save/import-paste 走 `applyConfig`、clear 走 `reset`、sync 走 `applySyncSuccess`、restore 走 `applyRestoreSuccess` → 徽章响应式
- [x] 4.3 同步/恢复按钮判据改 `canSync`（`apiUrl && hasApiKey`），非仅 `!apiUrl`；官方云预设已有「租户代码与 API Key 必填」引导，纯本地态按钮禁用而非堵死

## 4b. CardManagementView 同步状态收口（D3 第 4 消费者，B3）

- [x] 4b.1 **删除**卡片页 `syncConfigured` ref + 其 onMounted `load_sync_config_cmd` 读取；CardList 的 `:sync-configured` prop **改绑 store `canSync`**（语义本就是 `api_url && has_api_key`）——clear 后随共享态即时失效（**替换非叠加**）
- [x] 4b.2 卡片页 sync/restore 就绪判据用 store `canSync`；execute_sync 成功后 `applySyncSuccess`、restore 成功后 `applyRestoreSuccess` 回写（不旁路改局部）

## 5. 验证（强锚）

- [x] 5.1 前端 `vue-tsc --noEmit` + `eslint .` + `vite build` 全绿（`npm run build` 一把过）
- [~] 5.2 **跳过（无前端测试基建）**：项目无 vitest/test-utils/任何 runner，构建链无 test 步。为一段 3 行 computed + 直白状态变更引入 vitest+jsdom+tauri mock 属未请求的脚手架（ponytail：无框架则不加、YAGNI 同适用于测试）。该逻辑已被 ① `vue-tsc` 类型校验 ② 5.4 grep 锚验证接线 ③ 6.2 真机验收覆盖 `reset` 保留 clientId / restore 不动 lastSyncAt 的语义。若后续 `add-desktop-test-harness` 提案落地测试基建，再补 onboarding/derive/reset 单测
- [ ] 5.3 手动核对（用现有 `npm run dev` + 截图，**不依赖 Playwright/dev:mock**——该基建归 `add-desktop-test-harness` 提案、本期不做）：①首启空配置→网关显示且不可空手关、主界面未挂载 ②选纯本地→网关关、徽章「纯本地」、（无打印机则）跳打印机页 ③选云同步→落设置页、不被打印机跳转覆盖 ④存量云（mock api_url 非空）→无网关、徽章 tenant ⑤更新检查在网关期不绕过网关。**注**：原生壳专属行为真机验收见 6.x ← 用户自跑
- [x] 5.4 调用图审查 + grep 锚：模式无独立存储字段（恒派生 api_url）、徽章/canSync 只读 store 已保存态不读 syncForm 草稿（红线）、后端零改（无新增/改 Rust 命令）、**onboarding 标志走 localStorage 未误用 `set_app_setting_cmd`**（grep 确认：`load_sync_config_cmd` 仅存于 `syncStore.load()` 单点；onboarding 仅 App.vue 经 localStorage）

## 6. 发布与验收（部分用户自跑）

- [ ] 6.1 `cargo tauri build` 出包 ← 用户自跑
- [ ] 6.2 真机验收：全新安装→网关二选一；存量云用户升级→无网关、徽章显租户；存量纯本地升级→一次网关→选纯本地→此后静默；填 api_url 保存→徽章转租户、clear→回「纯本地」 ← 用户自跑
- [ ] 6.3 `openspec-cn archive add-desktop-tenant-onboarding`（增量并入 `cloud-database-support` 主规范）；**归档时核对**主规范「配置云端 API」场景的区域措辞与改名后「租户 & 云端同步」无新旧标题并存（D8 review RC#4）
