## 上下文

当前应用有两类用户偏好设置分散存储在非数据库位置：

1. **卡片张数显示模式** (`qty_display_mode`)：存储在前端 `localStorage`（键 `qty_display_mode`，值 `exact` 或 `approximate`），由 `web/src/composables/useQtyDisplayMode.ts` 管理。
2. **标签标题固定文字**：硬编码在 `config/templates/callsign.toml` 的 `title` 元素 `value` 字段中（当前值为 "中国无线电协会业余分会-2区卡片局"），由模板引擎 `resolve_fixed()` 读取。

这两项设置不在数据库中，因此导出/导入流程无法自动同步。

数据库迁移系统使用版本化 SQL 脚本（`migrations/` 目录），文件名格式 `YYYY.M.DD.NNN_name.sql`，当前最新版本为 `2026.1.24.003`。

## 目标 / 非目标

**目标：**
- 新建 `app_settings` 数据库表，提供通用键值对存储
- 将 `qty_display_mode` 从 `localStorage` 迁移到数据库
- 将标签标题固定文字从 `callsign.toml` 迁移到数据库
- 导出/导入自动包含 `app_settings` 表数据
- 云端同步包含 `app_settings` 表数据
- 提供 Tauri Command 接口供前端读写设置

**非目标：**
- 不迁移其他配置（如打印机配置、同步配置、模板布局参数等）
- 不修改模板配置文件格式或模板引擎的 `source` 类型系统
- 不创建独立的"设置"页面（设置项在各自功能区域内编辑）

## 决策

### 决策 1：使用简单键值对表而非结构化表

**选择**：创建 `app_settings(key TEXT PRIMARY KEY, value TEXT NOT NULL)` 表。

**替代方案**：
- 方案 B：为每个设置项创建单独的列 — 每新增设置就要加迁移，不灵活
- 方案 C：使用 JSON 单行存储所有设置 — 并发修改不安全，更新单个值需要读写整个 JSON

**理由**：键值对表最灵活，新增设置项无需修改 schema。`value` 统一为 `TEXT` 类型，复杂值可序列化为 JSON 字符串。SQLite 对小表的 key lookup 性能极高。

### 决策 2：打印时从数据库覆盖模板 `title` 的 `value`

**选择**：在打印命令中，加载模板配置后、调用模板引擎之前，从 `app_settings` 表读取 `label_title` 设置值。如果存在，将其注入到模板引擎的运行时数据中（`data` HashMap），并将 `title` 元素的 `source` 从 `fixed` 改为 `input`（`key = "label_title"`）。

**替代方案**：
- 方案 B：直接修改 `callsign.toml` 文件中的 `value` — 侵入模板文件，模板应保持为"默认值模板"
- 方案 C：在模板引擎 `resolve_fixed()` 中加数据库查询 — 模板引擎不应有数据库依赖

**理由**：在命令层注入数据是最干净的方式：模板引擎保持纯粹（不依赖数据库），模板文件保持默认值不变，逻辑集中在打印命令中处理。如果数据库中没有配置，则继续使用模板文件中的默认 `fixed` 值。

### 决策 3：导出文件中 `app_settings` 作为可选字段

**选择**：在 `ExportTables` 中新增 `app_settings: Option<Vec<AppSetting>>`（带 `#[serde(default, skip_serializing_if = "Option::is_none")]`）。

**理由**：向后兼容。旧版导出文件不含此字段，导入时 `serde(default)` 自动处理为 `None`，跳过恢复。旧版客户端打开新版导出文件时，未知字段被忽略（serde 默认行为）。

### 决策 4：云端同步包含 `app_settings`

**选择**：在 `SyncData` 结构体中新增 `app_settings: Vec<AppSetting>` 字段（非 Option，始终包含）。云端 `/sync` 端点在 DELETE 阶段额外清除 `app_settings` 表，INSERT 阶段写入新数据。

**替代方案**：
- 方案 B：不同步 `app_settings` — 多设备间偏好设置不一致，用户需要在每台设备上手动配置

**理由**：`app_settings` 与其他业务表的同步逻辑一致（全量推送），实现成本极低。云端 D1 数据库需要新建 `app_settings` 表（与本地结构相同）。

### 决策 5：前端 `useQtyDisplayMode` 改为通过 Tauri Command 读写

**选择**：保留 `useQtyDisplayMode.ts` composable 的接口不变，但内部改为调用 `get_app_setting` / `set_app_setting` Tauri Command。应用启动时从后端加载初始值，变更时调用后端保存。

**理由**：保持前端 API 兼容性，所有使用 `useQtyDisplayMode()` 的组件无需修改。仅改变数据持久化层。

## 风险 / 权衡

- **[风险] 首次迁移时 localStorage 旧值丢失** → 缓解：前端首次加载时检查 localStorage 是否有旧值，如果数据库中尚无记录则将旧值写入数据库，然后清除 localStorage。
- **[风险] 打印命令增加一次数据库查询** → 缓解：查询极轻量（单行 key lookup），对打印性能影响可忽略。
- **[权衡] `value` 统一为 TEXT 类型** → 布尔值、数字等需要在应用层做类型转换。但当前仅两个设置项，类型转换代价极低。
