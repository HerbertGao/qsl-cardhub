## 为什么

4-C2 落地了桌面端租户申报与云同步配置，但缺少**首启引导**与**身份常驻展示**：新用户首次打开不知道有「云同步 / 纯本地」两种用法；用了云同步的用户在主界面看不到自己当前所属租户/模式，要进设置页才知道。4-C3 补齐桌面端的模式 onboarding 网关 + 标题栏租户徽章 + 设置页重组，让两种模式的入口与身份显式化。

红线沿用 4-C1/4-C2：归属真源永远是写凭据（`key→tenant`），标题栏徽章**只展示**当前模式/租户，不作归属判定。

## 变更内容

- **新增** 首启模式 onboarding 网关 `AuthGateView.vue`（应用内首屏遮罩，**非新窗口**）：仅当 `api_url` 空 **且** onboarding 标志未设时显示；两个大按钮——「云同步」→ 导航到设置页配置 + 置 onboarding done；「纯本地」→ 置 onboarding done。网关**不可空手关闭**（必二选一，纯本地即低摩擦出口）。
- **新增** onboarding「已选过模式」标志，存 **localStorage**（key `qsl:onboarding:mode-selected`="1"，device-local 语义、不同步、永不主动清）。存量**云**用户（无该 key 且 `api_url` 非空）首启时静默置 done（零打扰）；存量**纯本地**用户首升级允许见一次网关、点「纯本地」即终身静默（不引入后端 count 查询）。
- **新增** 标题栏租户徽章（`App.vue` el-header 右侧）：三态——`纯本地`（`!api_url`）/ 租户值如 `bh2ro`（`api_url && tenant`）/ `云同步`（`api_url && !tenant`，软约束未申报态**不显「未申报」**）；点击 `navigateTo('data-config-data-transfer')` 进设置。
- **新增** `syncStore`（前端响应式单一事实源）：封装 `load/save/clear/import`，暴露完整 `apiUrl/tenant/hasApiKey/lastSyncAt/clientId/baseVersion` + 派生 `mode`（computed `apiUrl?'cloud':'local'`，徽章/展示用）+ `canSync`（computed `apiUrl && hasApiKey`，同步/恢复就绪用——后端硬要求 Key、模式≠可同步）。**4 个消费者**收口：徽章(App.vue) / 网关 / `DataTransferView` / **`CardManagementView`**（卡片页同步按钮 gate，原自读一份配置——漏收口则 clear 后滞后）；save/import 用**命令返回值**就地更新、clear 用 `reset()`（保留 client_id）、sync/restore 回写 lastSyncAt/baseVersion → 徽章/按钮响应式。**store 持已保存态、表单 `syncForm` 持草稿态、徽章只读 store**（编辑未保存不改模式，红线）。
- **改造** `DataTransferView.vue`：云端同步 card **移至顶部 + 改名「租户 & 云端同步」**（区域用途不变、装饰性改名）；数据导出/导入在下；云配置收口到 `syncStore`（同步/恢复按钮判据改 `canSync`，非 `!apiUrl`）。
- **改造** `App.vue`：**互斥渲染**（加载/加载失败/网关/主界面，主视图在加载/网关期**不挂载**，避免卡片页空项目弹「新建项目」盖网关下）；el-header 徽章读 store；onMounted **重构**——把打印机检测抽 `runPrinterCheck()`（无网关分支 + 网关选「纯本地」关闭后调，「云同步」分支不调以免覆盖已导航的设置页）；**更新检查/定时器无条件运行**（不被网关 return 吞掉）+ 阻断首屏（网关/加载失败）期抑制可绕过它的更新通知；执行序 `await load()`→`bootstrapAfterLoad`（存量迁移置标志→判网关→无网关跑打印机→**最后**置 App 自有 `bootReady` 揭幕，与 gateVisible 同一同步块、retry 先关门，杜绝竞态）；`load()` 失败不按空 apiUrl 判首启/不置 onboarding 标志、走错误分支显错误 + 重试。

不破坏：模式**靠 `api_url` 派生、不单独存字段**（与既定决策一致）；无「切换模式」按钮（填 api_url 保存=转云 / `clear_sync_config` 清空=回纯本地，派生副产物）；存量本地用户数据零影响。**后端零改动、无路由库、无 D1 迁移、无服务端改动。**

## 功能 (Capabilities)

### 新增功能
<!-- 无新增能力：桌面端同步模式/首启引导/徽章是对既有「云端数据同步」能力的扩展 -->

### 修改功能
- `cloud-database-support`: 新增「桌面端同步模式与首启引导」需求——首启模式网关、模式（云同步/纯本地）派生、标题栏租户徽章、模式切换为配置副产物。

## 影响

- **前端（全部改动集中于此）**：新建 `web/src/views/AuthGateView.vue`、`web/src/stores/syncStore.ts`、`web/src/utils/onboarding.ts`（localStorage 标志读写）；改 `web/src/App.vue`（互斥渲染：加载/加载失败/网关/主界面 + el-header 徽章 + onMounted 抽 `runPrinterCheck()` + 无条件更新检查 + App 自有 `bootReady` 揭幕门 + 加载失败错误分支重试）、`web/src/views/DataTransferView.vue`（云 card 置顶改名 + 收口 syncStore + 同步/恢复按钮改 canSync）、`web/src/views/CardManagementView.vue`（同步状态收口 syncStore，弃自读配置）。
- **复用**：`navigationStore.navigateTo`（徽章/网关跳转）、4-C2 的 `load/save/clear_sync_config_cmd`（经 syncStore 调用，签名不变）、云同步按钮既有 `:disabled`。
- **持久化**：onboarding 标志走 localStorage（device-local）；模式不落字段（派生）。
- **零后端**：不新增/改任何 Rust 命令；存量纯本地用户首升级接受一次网关（D1.4 取舍：换后端零改）。
