# 任务清单：卡片管理模块

**状态说明**：
- `[ ]` 待完成
- `[x]` 已完成

---

## Phase 1：卡片管理基础 ✅

### 1.1 数据库基础设施

- [x] 添加 `rusqlite` 和 `uuid` 依赖到 Cargo.toml
- [x] 创建 `src/db/mod.rs` - 数据库模块入口
- [x] 创建 `src/db/sqlite.rs` - SQLite 连接管理
  - [x] 实现数据库文件路径获取（按平台）
  - [x] 实现数据库自动创建
  - [x] 实现版本检查和迁移执行
- [x] 创建 `src/db/models.rs` - 数据模型定义
  - [x] 定义 `Project` 结构体
  - [x] 实现 `Serialize`/`Deserialize`
- [x] 创建 `migrations/001_init.sql` - 初始化脚本
  - [x] 创建 `projects` 表

### 1.2 项目管理后端

- [x] 创建 `src/db/projects.rs` - 项目 CRUD 操作
  - [x] `create_project(name: String) -> Result<Project>`
  - [x] `list_projects() -> Result<Vec<ProjectWithStats>>`
  - [x] `get_project(id: &str) -> Result<Option<Project>>`
  - [x] `update_project(id: &str, name: String) -> Result<Project>`
  - [x] `delete_project(id: &str) -> Result<()>`
- [x] 创建 `src/commands/projects.rs` - Tauri 命令
  - [x] `create_project` 命令
  - [x] `list_projects` 命令
  - [x] `get_project` 命令
  - [x] `update_project` 命令
  - [x] `delete_project` 命令
- [x] 更新 `src/main.rs` - 注册项目命令
- [x] 更新 `src/lib.rs` - 导出 db 模块

### 1.3 前端菜单导航

- [x] 更新 `web/src/App.vue` - 添加菜单分组
  - [x] 添加第一条分隔线（打印功能后）
  - [x] 添加"卡片管理"菜单项（Box 图标）
  - [x] 添加第二条分隔线（卡片管理后）
  - [x] 调整菜单项顺序

### 1.4 前端页面框架

- [x] 创建 `web/src/views/CardManagementView.vue` - 卡片管理页面
  - [x] 左右分栏布局（左侧 240px）
  - [x] 集成 ProjectList 组件
  - [x] 集成占位符组件
- [x] 创建 `web/src/components/projects/ProjectList.vue` - 项目列表
  - [x] 新建项目按钮
  - [x] 项目列表显示（名称 + 卡片数）
  - [x] 选中项目高亮
  - [x] 右键菜单（重命名、删除）
  - [x] 项目总数统计
- [x] 创建 `web/src/components/projects/ProjectDialog.vue` - 项目弹窗
  - [x] 新建模式
  - [x] 编辑模式
  - [x] 表单验证
- [x] 创建 `web/src/components/cards/CardListPlaceholder.vue` - 占位符
  - [x] 提示文本
- [x] 更新 `web/src/App.vue` - 添加 CardManagementView 路由

### 1.5 Phase 1 验收测试

- [x] 首次启动自动创建数据库文件
- [x] 数据库初始化脚本自动执行
- [x] 可以创建转卡项目
- [x] 可以查询项目列表（按创建时间降序）
- [x] 可以重命名项目
- [x] 可以删除项目（带确认）
- [x] 菜单显示分组横线
- [x] "卡片管理"菜单项正常工作
- [x] 左右分栏布局正确显示
- [x] 项目列表联动显示（右侧显示占位符）

---

## Phase 2：卡片录入和管理 ✅

### 2.1 数据库扩展

- [x] 创建 `migrations/002_add_cards.sql` - 卡片表
  - [x] 创建 `cards` 表
  - [x] 创建索引
  - [x] 外键级联删除
- [x] 更新 `src/db/models.rs` - 添加 Card 模型
  - [x] 定义 `Card` 结构体
  - [x] 定义 `CardStatus` 枚举
  - [x] 定义 `CardMetadata` 结构体
- [x] 更新 `src/db/sqlite.rs` - 版本迁移到 v2

### 2.2 卡片管理后端

- [x] 创建 `src/db/cards.rs` - 卡片 CRUD 操作
  - [x] `create_card(project_id, callsign, qty) -> Result<Card>`
  - [x] `list_cards(project_id, filters, pagination) -> Result<PagedCards>`
  - [x] `get_card(id) -> Result<Option<Card>>`
  - [x] `distribute_card(id, method, address, remarks) -> Result<Card>`
  - [x] `return_card(id, reason, remarks) -> Result<Card>`
  - [x] `delete_card(id) -> Result<()>`
- [x] 创建 `src/commands/cards.rs` - Tauri 命令
  - [x] `create_card` 命令
  - [x] `list_cards` 命令
  - [x] `get_card` 命令
  - [x] `distribute_card` 命令
  - [x] `return_card` 命令
  - [x] `delete_card` 命令
- [x] 更新 `src/main.rs` - 注册卡片命令

### 2.3 前端卡片组件

- [x] 创建 `web/src/components/cards/CardList.vue` - 卡片列表
  - [x] 工具栏（录入按钮、搜索框、状态筛选）
  - [x] 表格显示（序号、呼号、数量、状态、操作）
  - [x] 分页控件
  - [x] 操作按钮（查看、分发、退卡、删除）
- [x] 创建 `web/src/components/cards/CardInputDialog.vue` - 录入弹窗
  - [x] 项目选择（预选当前项目）
  - [x] 呼号输入（自动大写、格式验证）
  - [x] 数量输入（范围验证）
  - [x] 连续录入模式
  - [x] 快捷键支持（Enter/Esc）
- [x] 创建 `web/src/components/cards/DistributeDialog.vue` - 分发弹窗
  - [x] 处理方式选择（直接分发、邮寄、自取）
  - [x] 地址输入（邮寄时必填）
  - [x] 备注输入
- [x] 创建 `web/src/components/cards/ReturnDialog.vue` - 退卡弹窗
  - [x] 退卡原因选择
  - [x] 备注输入
- [x] 创建 `web/src/components/cards/CardDetailDialog.vue` - 详情弹窗
  - [x] 基本信息显示
  - [x] 分发历史（条件显示）
  - [x] 退卡历史（条件显示）
- [x] 更新 `web/src/views/CardManagementView.vue` - 替换占位符
  - [x] 集成 CardList 组件
  - [x] 项目选择联动

### 2.4 Phase 2 验收测试

- [x] 可以录入卡片，选择项目、输入呼号和数量
- [x] 呼号格式验证正确（3-10字符）
- [x] 数量范围验证正确（1-9999）
- [x] 连续录入模式正常工作
- [x] 快捷键（Enter/Esc）生效
- [x] 显示选中项目的所有卡片
- [x] 按呼号搜索正常（模糊匹配）
- [x] 按状态筛选正常
- [x] 分页功能正常
- [x] 只能分发"已录入"状态的卡片
- [x] 邮寄方式时地址必填
- [x] 分发成功后状态更新为"已分发"
- [x] 可以退卡"已录入"和"已分发"状态的卡片
- [x] 退卡成功后状态更新为"已退卡"
- [x] 显示完整的卡片详情
- [x] 可以删除卡片记录

---

## Phase 3：云数据库支持（可选）

> Phase 3 为可选功能，在 Phase 1 + Phase 2 完成后实施。

### 3.1 数据库配置

- [ ] 添加 PostgreSQL、JWT、keyring 依赖到 Cargo.toml
- [ ] 创建 `src/config/database.rs` - 数据库配置管理
- [ ] 创建 `src/db/repository.rs` - Repository trait 定义
- [ ] 创建 `src/db/postgres/` - PostgreSQL 实现
- [ ] 适配 `src/db/sqlite.rs` 实现 Repository trait
- [ ] 适配 `src/db/projects.rs` 实现 Repository trait
- [ ] 适配 `src/db/cards.rs` 实现 Repository trait

### 3.2 用户认证

- [ ] 创建 `src/auth/mod.rs` - 认证模块
- [ ] 创建 `src/auth/jwt.rs` - JWT Token 管理
- [ ] 创建 `src/auth/password.rs` - 密码加密
- [ ] 创建 `src/commands/auth.rs` - 认证 API
  - [ ] `register` 命令
  - [ ] `login` 命令
  - [ ] `refresh_token` 命令
  - [ ] `logout` 命令

### 3.3 数据迁移

- [ ] 创建 `src/commands/database.rs` - 数据库配置 API
  - [ ] `get_database_config` 命令
  - [ ] `save_database_config` 命令
  - [ ] `test_connection` 命令
  - [ ] `migrate_to_cloud` 命令
  - [ ] `download_from_cloud` 命令

### 3.4 前端界面

- [ ] 创建 `web/src/views/DatabaseConfigView.vue` - 数据库配置页面
  - [ ] 存储模式切换
  - [ ] SQLite 信息显示
  - [ ] 云数据库配置表单
  - [ ] 测试连接按钮
  - [ ] 迁移按钮
- [ ] 创建 `web/src/components/auth/LoginDialog.vue` - 登录弹窗
- [ ] 创建 `web/src/components/auth/RegisterDialog.vue` - 注册弹窗
- [ ] 创建 `web/src/stores/auth.js` - Pinia 用户状态管理
- [ ] 更新 `web/src/App.vue` - 添加数据库配置菜单

### 3.5 Phase 3 验收测试

- [ ] 可以选择存储模式（SQLite/云数据库）
- [ ] 云数据库配置表单正常
- [ ] 测试连接功能正常
- [ ] 密码加密存储（keyring）
- [ ] 可以注册新用户
- [ ] 可以登录获取 Token
- [ ] Token 自动刷新
- [ ] 登录状态持久化
- [ ] 可以从 SQLite 迁移到云数据库
- [ ] 可以从云数据库下载到 SQLite
- [ ] 显示迁移进度
- [ ] 冲突解决正常
- [ ] 不同用户的项目相互隔离
- [ ] 不同用户的卡片相互隔离

---

## 新增文件清单

### Phase 1 ✅

**Rust 后端**：
- `src/db/mod.rs`
- `src/db/sqlite.rs`
- `src/db/models.rs`
- `src/db/projects.rs`
- `src/commands/projects.rs`
- `migrations/001_init.sql`

**前端**：
- `web/src/views/CardManagementView.vue`
- `web/src/components/projects/ProjectList.vue`
- `web/src/components/projects/ProjectDialog.vue`
- `web/src/components/cards/CardListPlaceholder.vue`

### Phase 2 ✅

**Rust 后端**：
- `src/db/cards.rs`
- `src/commands/cards.rs`
- `migrations/002_add_cards.sql`

**前端**：
- `web/src/components/cards/CardList.vue`
- `web/src/components/cards/CardInputDialog.vue`
- `web/src/components/cards/DistributeDialog.vue`
- `web/src/components/cards/ReturnDialog.vue`
- `web/src/components/cards/CardDetailDialog.vue`

### Phase 3

**Rust 后端**：
- `src/db/repository.rs`
- `src/db/postgres/mod.rs`
- `src/db/postgres/projects.rs`
- `src/db/postgres/cards.rs`
- `src/auth/mod.rs`
- `src/auth/jwt.rs`
- `src/auth/password.rs`
- `src/commands/auth.rs`
- `src/commands/database.rs`
- `src/config/database.rs`

**前端**：
- `web/src/views/DatabaseConfigView.vue`
- `web/src/components/auth/LoginDialog.vue`
- `web/src/components/auth/RegisterDialog.vue`
- `web/src/stores/auth.js`

---

## 修改文件清单

### Phase 1 ✅

- `Cargo.toml` - 添加 rusqlite、uuid 依赖
- `src/main.rs` - 注册项目命令、初始化数据库
- `src/lib.rs` - 导出 db 模块
- `web/src/App.vue` - 添加菜单分组和卡片管理入口
- `web/vite.config.js` - 添加 @ 路径别名

### Phase 2 ✅

- `src/db/mod.rs` - 导出 cards 模块
- `src/db/models.rs` - 添加 Card 模型
- `src/db/sqlite.rs` - 版本迁移到 v2
- `src/commands/mod.rs` - 导出 cards 模块
- `src/main.rs` - 注册卡片命令
- `web/src/views/CardManagementView.vue` - 集成 CardList 组件

### Phase 3

- `Cargo.toml` - 添加 PostgreSQL、JWT、keyring 依赖
- `src/db/sqlite.rs` - 适配 Repository trait
- `src/db/projects.rs` - 适配 Repository trait
- `src/db/cards.rs` - 适配 Repository trait
- `web/src/App.vue` - 添加数据库配置菜单
