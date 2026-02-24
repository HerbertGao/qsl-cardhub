# app-settings Specification

## Purpose
全局键值对配置存储，用于持久化应用级用户偏好设置。

## 需求

### 需求：全局配置表

系统**必须**提供基于数据库的全局键值对配置存储，用于持久化应用级用户偏好设置。

#### 场景：数据库表结构

- **当** 应用首次启动并执行数据库迁移
- **那么** 必须创建 `app_settings` 表，包含 `key`（TEXT, PRIMARY KEY）和 `value`（TEXT, NOT NULL）两列

#### 场景：写入配置项

- **当** 调用 `set_app_setting` 命令，传入 `key` 和 `value`
- **那么** 系统必须将该键值对写入 `app_settings` 表
- **并且** 如果 `key` 已存在，必须覆盖更新 `value`
- **并且** 如果 `key` 不存在，必须插入新记录

#### 场景：读取配置项

- **当** 调用 `get_app_setting` 命令，传入 `key`
- **那么** 如果 `key` 存在，必须返回对应的 `value` 字符串
- **并且** 如果 `key` 不存在，必须返回 `null`

#### 场景：读取所有配置项

- **当** 调用 `get_all_app_settings` 命令
- **那么** 必须返回 `app_settings` 表中所有键值对的列表

### 需求：卡片张数显示模式配置

系统**必须**将卡片张数显示模式（精确/大致）存储在 `app_settings` 表中，键名为 `qty_display_mode`。

#### 场景：读取显示模式

- **当** 前端初始化 `useQtyDisplayMode` composable
- **那么** 必须调用 `get_app_setting` 获取 `qty_display_mode` 的值
- **并且** 如果值为 `exact` 或 `approximate`，使用该值
- **并且** 如果值不存在（返回 null），默认使用 `exact`

#### 场景：切换显示模式

- **当** 用户切换卡片张数显示模式
- **那么** 必须调用 `set_app_setting` 将新模式值写入数据库
- **并且** 前端响应式状态必须同步更新

#### 场景：从 localStorage 迁移

- **当** 前端首次加载且 `localStorage` 中存在 `qty_display_mode` 键
- **并且** 数据库 `app_settings` 表中不存在该键
- **那么** 必须将 `localStorage` 中的值写入数据库
- **并且** 清除 `localStorage` 中的旧键

### 需求：标签标题文字配置

系统**必须**将打印标签的标题固定文字存储在 `app_settings` 表中，键名为 `label_title`。

#### 场景：打印时读取标题

- **当** 执行打印命令（`print_qsl` 或 `preview_qsl`）加载模板配置后
- **并且** `app_settings` 表中存在 `label_title` 键
- **那么** 必须使用数据库中的值覆盖模板 `title` 元素的固定文字
- **并且** 禁止使用模板文件中的硬编码默认值

#### 场景：未配置标题时使用模板默认值

- **当** 执行打印命令加载模板配置后
- **并且** `app_settings` 表中不存在 `label_title` 键
- **那么** 必须使用模板文件中 `title` 元素的原始 `value`（即 "中国无线电协会业余分会-2区卡片局"）

#### 场景：标签标题首次初始化

- **当** 数据库迁移创建 `app_settings` 表时
- **那么** 必须插入默认值 `label_title` = `中国无线电协会业余分会-2区卡片局`
