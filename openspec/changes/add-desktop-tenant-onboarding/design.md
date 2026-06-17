## 上下文

4-C2 已落地桌面端租户申报 + 云同步配置：`SyncConfig.tenant`、`load/save/clear_sync_config_cmd`（返回 `{api_url, client_id, last_sync_at, has_api_key, base_version, tenant}`）、`test/execute/restore`、配置 UX（API 下拉、租户失焦格式化、复制粘贴）。

代码现状（已调研）：
- `App.vue`：自绘 `el-header`（无 Tauri 原生标题），`activeMenu` ref + 每视图 `v-if`；`onMounted` 有首启先例（`get_printer_config` 无打印机则 `activeMenu='print-config-printer'`）。
- `navigationStore.navigateTo(target)` → watcher 设 `activeMenu`。
- 持久化：`get/set_app_setting_cmd`（DB `app_settings`，**会同步到云端**）/ `localStorage`（device-local、不同步）。
- 无共享 sync store；`DataTransferView` 自己 invoke。

## 目标 / 非目标

**目标：**
- 首启让用户显式选「云同步 / 纯本地」；身份/模式常驻标题栏。
- 模式靠 `api_url` 派生、不引入模式字段；后端零改。
- 存量用户不被破坏（云用户零打扰、纯本地用户最多一次网关）。

**非目标：**
- **不开新 Tauri 窗**（网关是应用内遮罩）；**不引入路由库**（用 navigationStore）。
- 不做「切换模式」独立控件（派生副产物）。
- 不做未完成云配置的草稿态/续配引导。
- 不动后端、不动服务端、无 D1 迁移。

## 决策

### D1：网关触发 = `!api_url && !onboardingDone`；标志 localStorage、语义「已选过模式」
- 触发**只两个布尔与**，**不查本地库 count**（砍掉）：count 的唯一价值是免存量纯本地用户一次点击，不值一个后端命令 + 跨层调用（YAGNI）。
- onboarding 标志存 **localStorage**（`qsl:onboarding:mode-selected`="1"）：onboarding 是「这台设备的人是否被引导过」=纯 device-local 语义。`app_settings`(DB) 会跨设备同步→设备 B 继承「已 onboard」语义错；sync.toml 是同步配置文件、塞 UI 偏好越界。被清最多重弹一次、无害。
- **标志语义 = 「已做过一次模式选择」**（非「已配云」）：点网关任一按钮即置位。若语义是「已配云」，选纯本地的用户会每次启动满足「api_url 空+未配云」被反复弹——钉死在「已选过」上避免此坑。
- **不存选了哪个模式**（模式由 `api_url` 派生，存模式=冗余真源）。**永不主动清**（单调 done 标志、无生命周期 bug 面）。

### D1.4：存量用户处理（迁移时写标志，非运行时探测）
- 存量**云**用户：localStorage 无 key 且 `api_url` 非空 → 启动时静默置 done（零打扰）。
- 存量**纯本地**用户（无 key + api_url 空）：**允许见一次网关**，点「纯本地」即置 done、终身静默。**用户已拍板取舍**：接受一次网关换后端零改。
- **修正（review）**：触发集**无「有卡片」维**——触发恒为 `!apiUrl && !onboardingDone`，全新用户与存量纯本地用户**走同一条路径**（都弹一次网关）。design/spec 里**禁止**出现「有卡片/有本地数据」限定（与 D1「砍掉 count」矛盾、会误导实现者加被砍的 count 调用）。
- **求值顺序（钉死，S1 联动）**：onMounted 内必须是单一有序序列——`await syncStore.load()` →（迁移：`!isModeSelected() && apiUrl 非空` → `markModeSelected()`）→ 计算网关条件 `!apiUrl && !isModeSelected()` → **最后**置 `bootReady` 揭幕。迁移**必须先于**网关判定（否则存量云用户可能闪一帧网关）。揭幕标志用 App 自有的 `bootReady`（**非** store 的加载完成态）：见 D5 竞态说明。

### D2：AuthGateView 只做分诊、不重复云表单
- 网关两个大按钮 + 简短模式说明：「云同步」→ `markModeSelected()` → `navigateTo('data-config-data-transfer')` → 关网关，**且该分支直接结束、不触发后续打印机检测**（用户已显式选了落点，跑打印机检测会覆盖云配置页 → review blocker）；「纯本地」→ `markModeSelected()` → 关网关 → 关闭回调里**才**跑 `runPrinterCheck()`（见 S1）。
- **不内嵌云配置表单**：DataTransferView（4-C2 的 API 下拉/租户格式化/复制粘贴）是云配置**单一事实源**，网关再放一套=DRY 违背 + 漂移源。网关只把人送到正确页面。
- 边界：跳云后 onboarding **立即置 done**（onboarding 是「选模式」非「完成配置」）；中途放弃→下次以纯本地态静默进入（api_url 仍空）、徽章显「纯本地」、点徽章可重进设置续配。**本次会话内选云后徽章即显「纯本地」是正确的**（mode 派生自已保存 api_url，未配置=本地态；非 bug，spec 需声明此瞬时态）。**不做草稿态**（S7）。

### D3：syncStore 响应式单一事实源（4 消费者、字段完整、mode≠canSync）
- 新建 `syncStore`：封装 `load/save/clear/import`，暴露响应式**完整 `SyncConfigResponse` 字段**：`apiUrl/tenant/hasApiKey/lastSyncAt/clientId/baseVersion`（**review 修正：补 clientId/baseVersion**——DataTransferView 展示 client_id、sync/restore 后更新 base_version，漏则 UI 字段丢失）。
- **派生（computed，不存字段）**：`mode = computed(() => apiUrl.value ? 'cloud' : 'local')`（徽章/模式展示用）；**`canSync = computed(() => !!apiUrl.value && hasApiKey.value)`（review 新增：同步/恢复就绪判据——后端 `/sync` 硬要求 API Key，`mode` 不等于「可同步」，按钮禁用必须用 `canSync` 不是 `!apiUrl`）**。
- **4 个消费者（review 修正：原写 3 个漏了 CardManagementView）**：徽章(App.vue) / 网关 / DataTransferView / **CardManagementView**（其 `syncConfigured`(:156) + onMounted 自读配置 gate 卡片页同步按钮、execute_sync/restore 入口）。CardManagementView **必须**收口到 store（否则 clear 后其 `syncConfigured` 滞后、卡片页同步按钮仍可点 = 正是本提案要消灭的「写后不即时反映」换组件复现）。
- **草稿态 vs 已保存态（review 红线，M4）**：DataTransferView 有 `syncForm`(表单 v-model 草稿) + 展示态。**store 持「已保存态」，`syncForm` 持「编辑草稿」，徽章/mode 只读 store**——用户在表单改 api_url 未点保存时徽章**不变**（mode=已保存 api_url 派生）。**禁止**把 `syncForm` 接到 store（否则编辑未保存即改模式/徽章，违背「模式=已保存 api_url 派生」红线）。
- **刷新时机（S4，review 修正措辞）**：①App onMounted 首 load；②save/import 成功后用**命令返回的 `SyncConfigResponse`**就地更新 store（**非「刚提交的值」**——返回值含后端规整后的 tenant[slug 校验/trim 可能不同]）；③clear 成功后 `reset()`；④sync/restore 成功后回写（原标「可选」改必须，否则 last_sync 漂移）——**但回写字段按操作类型分**（review RC#1）：**sync** 成功（`SyncCmdResult` 有 `sync_time`）回写 `lastSyncAt`+`baseVersion`；**restore** 成功（`RestoreResult` **无 `sync_time`**、只有 `server_version`）**仅**回写 `baseVersion`，**禁止**用统一签名把 `lastSyncAt` 置空。**回写适用于所有调用点**——DataTransferView **与 CardManagementView** 的 execute_sync/restore 两处都经 store 回写、不旁路改局部；⑤paste(import) 同 save 路径。不轮询。
- **`reset()` 字段清单（review，M5）**：清空 `apiUrl/tenant/hasApiKey/lastSyncAt/baseVersion`，**保留 `clientId`**（设备标识、清云配不该丢，对齐现状 DataTransferView clear 保留 client_id）。
- store 只封装既有命令 + 4 消费者需要的字段，不预加未来方法（YAGNI）。

### D4：标题栏徽章三态（读 store 已保存态）
| 状态 | 判据（store 已保存值） | 徽章 |
|---|---|---|
| 纯本地 | `!apiUrl` | `纯本地` |
| 云·已申报 | `apiUrl && tenant` | tenant 值（如 `bh2ro`） |
| 云·未申报 | `apiUrl && !tenant` | `云同步` |
- `!apiUrl` 吸收 tenant 维度（本地态 tenant 无意义）；`reset()` 必须清 tenant 否则出现 `!apiUrl && tenant非空` 第四态（M5 已堵）。判据取 **store 已保存值**、非 syncForm 草稿（M4）。
- 位置 el-header 右侧、点击 `navigateTo('data-config-data-transfer')`。
- **未申报态不显「未申报」**：4-C1 已定未申报是软约束兼容期的合法态，徽章是身份/模式指示器、不该把合法态渲成像错误。催申报是设置页内引导的职责，非标题栏。
- 徽章纯展示、不承担归属判断（4-C1 红线）。
- **徽章三态与 `canSync` 正交（review CR🟡3）**：徽章显 tenant（如 `bh2ro`）**不蕴含** `canSync` 为真——`apiUrl && tenant && !hasApiKey`（如导入了带 tenant 但缺 Key 的配置串）徽章仍显 `bh2ro`、但同步/恢复按钮按 `canSync` 禁用。徽章=身份/模式、canSync=就绪，二者独立。

### D5：四态互斥渲染（loading / 加载失败 / 网关 / 主界面）——网关是阻断首屏、非叠加遮罩（review B2）
- App.vue 顶层**互斥 v-if 链**：`!bootReady` → loading（留白/极短）；`bootReady && loadError` → 错误分支（D7）；`bootReady && 网关条件成立` → 仅渲染 AuthGateView；否则 → 渲染主界面。**主内容视图（含卡片页）在 loading/网关期间禁止挂载**。
- **理由**：若按「主视图照常挂载 + 遮罩叠上去」，卡片页 onMounted 会先跑（空项目时立即弹「新建项目」对话框盖在网关下/旁），网关不是干净的阻断首屏。互斥渲染保证网关期主界面零副作用。
- **揭幕标志为何放 App 自有 `bootReady`、不放 store（Codex 竞态修复）**：若把揭幕门放在 store（如随 `load()` 完成翻转的加载态 ref），它会**早于** App 的迁移/网关赋值翻转——`load()` 内对它的同步置真触发 Vue flush 调度、排在 `await load()` 续体之前，于是出现「揭幕了但 `gateVisible` 还是默认 false」的一帧 → 主界面（含卡片页）抢先挂载。改用 `bootReady`：它**仅在** `bootstrapAfterLoad` 把 `gateVisible`/`loadError` 全部定下后、同一同步块内置真，Vue 单次 flush 即读到正确分支，无中间帧。**retry 同理**：`retryLoad` 必须先 `bootReady=false` 关门再 `load()`，否则 load 清 `loadError` 后揭幕门仍开、`gateVisible` 未重算，复现同一抢挂（二轮 Codex Q6）。
- 与 S6（首帧不渲染防闪烁）合并：`bootReady` 既是防闪烁门、也是互斥渲染门。

### D6：onMounted 串行化重构——抽 `runPrinterCheck()`、更新检查无条件（review B1）
- **抽函数**：把既有打印机检测（App.vue:228-233 `get_printer_config` 无打印机→`activeMenu='print-config-printer'`）抽成独立 `runPrinterCheck()`，供**两处**调用：①无网关分支（onMounted 内）；②网关「纯本地」关闭回调。**「云同步」分支不调**（已 navigate 到设置页，跑打印机检测会覆盖落点 → blocker）。
  - 仅 return 无法实现「关闭后再跑」：onMounted 是单次 async、return 即结束，网关关闭是之后的异步回调——必须抽函数由回调复用。
- **更新检查无条件**：`silentCheckUpdate()` + `setInterval` 定时器**与网关正交、无条件注册**（它们不依赖 activeMenu）。**禁止**被「网关成立则 return」吞掉（否则首启走网关的会话永久无更新检查）。
- **更新通知不绕过阻断首屏（review F4 + 二轮 RC-A）**：更新检查的 `ElNotification`「查看详情」会切 `activeMenu='about'`。**任一阻断首屏期间（网关 `gateVisible` 或加载失败错误分支 `loadError`，主界面均未挂载）抑制该通知**——否则「查看详情」是死按钮（目标视图未挂载）/ 误导。实现：`silentCheckUpdate` 在弹通知前 `if (gateVisible.value || syncStore.loadError.value) return`。检查本身照常跑（D6 正交），徽章「New!」进主界面后仍提示。
- **执行序（与 D1.4 一致）**：`await load()` → `bootstrapAfterLoad()`（迁移置标志 → 判网关 → (无网关)`runPrinterCheck()` → **最后**置 `bootReady` 揭幕）→ 无条件更新检查 + `setInterval`。迁移/网关/揭幕收口在 `bootstrapAfterLoad` 单函数内同步完成，retryLoad 复用之。

### D7：`load()` 失败路径（review M6）
- `load_sync_config_cmd` 返回 `Result<Option<…>>`、**可 reject**（凭据/IO 异常）。reject 时**禁止**按「空 apiUrl」判首启（会误弹网关 / 误迁移）、**禁止**置 onboarding done。
- **实现**：`load()` 内部 try/catch 吞异常（catch 置 `loadError`、**不重抛**），故 `await load()` 必 resolve、`bootstrapAfterLoad` 总能跑到揭幕（置 `bootReady`），不卡死首屏。`loadError` 为真时 `bootstrapAfterLoad` 跳过迁移/网关、直接揭幕到**错误分支**（D5 四态之一）：`el-result` + 「重试」按钮 → `retryLoad()` 重跑 `load()` + `bootstrapAfterLoad`，仍失败则停留错误分支。**不静默退化为空配置主界面**（CR/RC review：否则临时故障与真本地态无从区分）。

### S5：网关不可空手关闭（定义，原 tasks 引用但 design 缺定义）
- 网关**无关闭按钮、点遮罩外不关**，**必须**二选一（云同步/纯本地）才进应用。「纯本地」即低摩擦出口，不构成强制注册。不给「跳过」第三选项（否则引入既没选云也没选本地却进应用的未定义态、onboarding 标志置不置位模糊）。

## 风险 / 权衡

- **[网关与打印机/更新检查首启逻辑打架]（S1，review 重写）** → onMounted 三段副作用（打印机跳转 / silentCheckUpdate / setInterval）若被「网关 return」一锅吞，会致打印机检测永不跑（关闭后）+ 更新检查永久失效。**缓解**：D6——打印机检测抽 `runPrinterCheck()` 两处调、「云同步」分支不调；更新检查/定时器无条件注册；网关期抑制更新通知导航。
- **[首帧闪烁]（S6）** → 徽章/网关依赖 syncStore 异步 load 的 `apiUrl`，首帧用默认值会闪。**缓解**：`bootReady`（App 自有揭幕门，迁移/网关决策定下后才置真）前不渲染徽章/网关/主界面，留白/极短 loading。
- **[localStorage 被清]** → 重弹一次网关。**缓解**：低频且无害（D1.2 已论证），不加持久化复杂度。
- **[模式切换无显式入口困惑]（S2）** → 用户找不到「切换模式」。**缓解**：clear 按钮（清空云配置）即「退回纯本地」事实入口、文案补一句；填 api_url 保存即转云。模式既是派生的，就不该有独立切换控件（否则诱发模式字段、违背既定决策）。

## 迁移计划

- 纯前端、无后端/服务端/D1 迁移。部署=桌面端发版。回滚=退版本（onboarding 标志残留 localStorage 无害）。
- 存量云用户：首启自动置 onboarding done（迁移逻辑）；存量纯本地用户：一次网关。

## 决策（spec 选型，review M7）

### D8：spec 选型——ADDED 新需求；区域改名按装饰性处理（不做全量 MODIFIED）
- onboarding 网关 / 徽章 / 模式派生 / canSync 是**新关注点**（不改既有同步场景的核心断言）→ 用 **ADDED**「桌面端同步模式与首启引导」新需求承载（与 4-C2 的 MODIFIED 选型不同，因 4-C2 是改既有同步 scenario 的请求头枚举、本期是新增正交关注点）。
- DataTransferView 云卡片**改名「云端同步」→「租户 & 云端同步」**：既有「云端数据同步」需求「配置云端 API」场景的「在『云端同步』区域输入」是对**区域用途**（云同步配置）的描述，改名后仍是**同一区域**（云同步配置区，只是加「租户 &」前缀）——属**装饰性标题变更、非行为断言矛盾**。故**不做全量 MODIFIED 复制既有需求**（不为一个装饰性改名复制 ~150 行 scenario）；改名落在 tasks UI 改动 + 本 ADDED 需求场景（「导航到『租户 & 云端同步』设置页」即指该区域）。**取舍**：4-C1/4-C2 教训针对的是**行为/枚举自相矛盾**，标题前缀化不属此类；若 review 坚持，再补 MODIFIED。
- **红线不重复**：ADDED 需求**引用**而非复制 4-C1 红线（归属真源 key→tenant、徽章只展示）——措辞「归属红线沿用既有『云端数据同步』需求」，不另立第二份权威拷贝（spec intro 已按此改）。

## 待解决问题

- 网关文案的具体措辞（两模式各一句说明）——实现期定。
- 徽章在窄窗口的省略/tooltip——次要，实现期看效果。
- 测试层：tasks 5.x 暂用现有 `pnpm run dev` + 手测/截图（Playwright/dev:mock 基建归 `add-desktop-test-harness` 提案、本期不依赖）。
