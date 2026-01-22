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

## Phase 2.5：凭据加密存储、数据配置页面和 QRZ.cn 集成

### 2.5.1 本地凭据加密存储

- [ ] 添加加密存储依赖到 Cargo.toml
  - [ ] `keyring` (系统钥匙串)
  - [ ] `aes-gcm` (AES-256-GCM 加密)
  - [ ] `pbkdf2` (密钥派生)
  - [ ] `rand` (随机数生成)
- [ ] 创建 `src/security/mod.rs` - 安全模块入口
- [ ] 创建 `src/security/credentials.rs` - 凭据存储抽象层
  - [ ] `CredentialStorage` trait 定义
  - [ ] 检测钥匙串可用性
  - [ ] 自动选择存储方式（钥匙串/本地加密文件）
  - [ ] 统一的存储/读取/删除接口
- [ ] 创建 `src/security/keyring.rs` - 系统钥匙串实现
  - [ ] `KeyringStorage` 实现 `CredentialStorage`
  - [ ] 跨平台钥匙串访问（macOS/Windows/Linux）
  - [ ] 错误处理和降级检测
- [ ] 创建 `src/security/encryption.rs` - 本地加密文件实现
  - [ ] `EncryptedFileStorage` 实现 `CredentialStorage`
  - [ ] 主密钥生成（PBKDF2）
  - [ ] AES-256-GCM 加密/解密
  - [ ] 加密文件读写（原子操作）
  - [ ] 文件权限设置（Unix 0600）
- [ ] 创建 `src/commands/security.rs` - 凭据管理 Tauri 命令
  - [ ] `save_credential(key, value)` 命令
  - [ ] `get_credential(key)` 命令
  - [ ] `delete_credential(key)` 命令
  - [ ] `check_keyring_available()` 命令
- [ ] 更新 `src/main.rs` - 注册安全命令

### 2.5.2 数据配置页面

- [ ] 更新 `web/src/App.vue` - 添加数据配置子菜单
  - [ ] 在"卡片管理"下方添加"数据配置"子菜单
  - [ ] 使用 Connection 图标
  - [ ] 添加子菜单项："QRZ.cn" 和 "云数据库"（待开发）
  - [ ] 添加路由到 QRZConfigView 和 CloudDatabaseConfigView
- [ ] 创建 `web/src/views/QRZConfigView.vue` - QRZ.cn 配置页面
  - [ ] 标准页面布局（page-content）
  - [ ] 页面标题
  - [ ] 卡片容器布局（el-card）
  - [ ] 用户名输入框
  - [ ] 密码输入框（遮罩）
  - [ ] "保存并登录"按钮
  - [ ] "清除凭据"按钮（带确认对话框）
  - [ ] "测试连接"按钮
  - [ ] 登录状态显示（已登录/未登录）
  - [ ] 已保存凭据提示
  - [ ] 存储方式提示（系统钥匙串/本地加密文件）
  - [ ] 自动加载已保存的用户名和密码
- [ ] 创建 `web/src/views/CloudDatabaseConfigView.vue` - 云数据库配置页面（占位符）
  - [ ] 标准页面布局（page-content）
  - [ ] 页面标题
  - [ ] 空状态占位符（el-empty）

### 2.5.2 QRZ.cn 和 QRZ.com 登录功能（集成凭据存储）

- [ ] 添加 HTTP 客户端和 HTML 解析依赖到 Cargo.toml
  - [ ] `reqwest`（HTTP 客户端）
  - [ ] `scraper`（HTML 解析）
  - [ ] `encoding_rs`（GBK 编码转换，仅 QRZ.cn 需要）
- [ ] 创建 `src/qrz/mod.rs` - QRZ 模块入口（支持多站点）
- [ ] 创建 `config/qrz.toml` 配置文件模板（支持 qrz.cn 和 qrz.com）
- [ ] 创建 `src/qrz/client.rs` - HTTP 客户端（统一接口）
  - [ ] `login_qrz_cn(username, password) -> Result<QrzCnSession>`
    - [ ] Cookie 内存管理（CFID、CFTOKEN）
    - [ ] 会话过期检测（推测 30 天）
  - [ ] `login_qrz_com(username, password) -> Result<QrzComSession>`
    - [ ] Cookie 内存管理（xf_session）
    - [ ] 会话过期检测（约 30 天）
  - [ ] `query_callsign_cn(session, callsign) -> Result<CallsignInfo>`
    - [ ] GBK 到 UTF-8 转换
  - [ ] `query_callsign_com(session, callsign) -> Result<CallsignInfo>`
- [ ] 创建 `src/qrz/parser.rs` - HTML 解析器（支持多站点）
  - [ ] `parse_qrz_cn_callsign(html) -> Result<CallsignInfo>`
    - [ ] 提取呼号
    - [ ] 提取查询次数
    - [ ] 提取更新日期
    - [ ] 提取中文地址
    - [ ] 提取英文地址
  - [ ] `parse_qrz_com_callsign(html) -> Result<CallsignInfo>`
    - [ ] 提取呼号（从 `<span class="csignm hamcall">` 标签）
    - [ ] 提取姓名和地址（从 `<p class="m0">` 标签）
    - [ ] 提取更新时间（从 Detail 标签页）
- [ ] 创建 `src/commands/qrz.rs` - QRZ Tauri 命令
  - [ ] `qrz_cn_save_and_login` 命令（保存凭据并登录 QRZ.cn）
  - [ ] `qrz_cn_load_credentials` 命令（自动加载 QRZ.cn 凭据）
  - [ ] `qrz_cn_clear_credentials` 命令（清除 QRZ.cn 凭据和 Cookie）
  - [ ] `qrz_cn_query_callsign` 命令
  - [ ] `qrz_com_save_and_login` 命令（保存凭据并登录 QRZ.com）
  - [ ] `qrz_com_load_credentials` 命令（自动加载 QRZ.com 凭据）
  - [ ] `qrz_com_clear_credentials` 命令（清除 QRZ.com 凭据和 Cookie）
  - [ ] `qrz_com_query_callsign` 命令
- [ ] 更新 `src/main.rs` - 注册 QRZ 命令

### 2.5.3 地址缓存功能（智能去重，支持多数据源）

- [ ] 更新 `src/db/models.rs` - 扩展 Card metadata
  - [ ] 添加 `AddressHistoryEntry` 结构体（单条地址记录）
    - [ ] `source: String` - 数据源标识（"qrz.cn" 或 "qrz.com"）
    - [ ] `callsign: String` - 呼号
    - [ ] `name: Option<String>` - 姓名（可选，QRZ.com 使用）
    - [ ] `chinese_address: Option<String>` - 中文地址（可选，QRZ.cn 使用）
    - [ ] `english_address: Option<String>` - 英文地址（可选，QRZ.cn 使用）
    - [ ] `address: Option<String>` - 地址（QRZ.com 使用）
    - [ ] `query_count: Option<u32>` - 查询次数（可选，QRZ.cn 使用）
    - [ ] `data_updated_at: String` - 数据更新日期
    - [ ] `cached_at: String` - 本地查询时间戳
  - [ ] 实现 `AddressHistoryEntry` 数据内容比较方法（按 source 区分比较字段）
  - [ ] 添加 `AddressHistory` 结构体（地址历史数组）
  - [ ] 序列化/反序列化支持
- [ ] 更新 `src/db/cards.rs` - 添加地址历史管理函数
  - [ ] `upsert_address_history(card_id, entry) -> Result<()>` - 智能插入/更新
    - [ ] 检查最新记录数据是否一致（按 source 区分）
    - [ ] 一致时仅更新 cached_at
    - [ ] 不一致时追加新记录
  - [ ] `get_latest_address(card_id, source: &str) -> Result<Option<AddressHistoryEntry>>` - 获取指定数据源的最新版本
  - [ ] `prune_source_history(card_id, source: &str, max_count: usize) -> Result<()>` - 清理同源旧记录（保留最新 2 条）
- [ ] 创建 `src/commands/cards.rs` 中的缓存命令
  - [ ] `upsert_card_address` 命令
  - [ ] `get_card_latest_address` 命令（支持指定 source）

### 2.5.4 分发弹框中的地址查询（支持多数据源）

- [ ] 创建 `web/src/components/qrz/QrzAddressDisplay.vue` - 地址展示组件
  - [ ] 支持多数据源显示（qrz.cn 和 qrz.com）
  - [ ] QRZ.cn 格式:
    - [ ] 呼号: [呼号] (qrz.cn)
    - [ ] 中文地址: [中文地址]
    - [ ] 英文地址: [英文地址]
    - [ ] 更新时间: YYYY-MM-DD
    - [ ] 缓存时间: YYYY-MM-DD HH:mm
  - [ ] QRZ.com 格式:
    - [ ] 呼号: [呼号] (qrz.com)
    - [ ] 姓名: [姓名]
    - [ ] 地址: [地址（多行）]
    - [ ] 更新时间: YYYY-MM-DD
    - [ ] 缓存时间: YYYY-MM-DD HH:mm
  - [ ] 缓存有效性标识（365天内/已过期）
  - [ ] 复制地址按钮（根据数据源动态调整）
  - [ ] 刷新按钮
- [ ] 更新 `web/src/components/cards/DistributeDialog.vue`
  - [ ] 在收件地址区域上方显示地址查询区域
  - [ ] 添加数据源选择器（QRZ.cn / QRZ.com）
  - [ ] 弹框打开时自动加载最新缓存地址（如果有，优先显示 QRZ.cn）
  - [ ] 未登录时禁用对应数据源的查询并提示
  - [ ] 集成 QrzAddressDisplay 组件
  - [ ] 实现刷新查询逻辑（支持切换数据源）
  - [ ] 显示查询加载状态
  - [ ] 查询成功后自动保存到地址历史
  - [ ] 地址信息只读展示，不自动填充

### 2.5.5 QRZ.herbertgao.me 集成（公开 API，无需登录）

- [ ] 添加 JSON 序列化依赖到 Cargo.toml（如果尚未添加）
  - [ ] `serde_json`（JSON 解析）
- [ ] 创建 `src/qrz/qrz_herbertgao_client.rs` - QRZ.herbertgao.me JSON API 客户端
  - [ ] `query_callsign(callsign) -> Result<Option<HerbertgaoAddressInfo>>`
    - [ ] 无需 Cookie 或 Token
    - [ ] 筛选 `isShow: true` 的记录
    - [ ] 提取第一条有效记录的字段
  - [ ] `HerbertgaoAddressInfo` 结构体
    - [ ] `call_sign: String`（呼号）
    - [ ] `name: String`（姓名）
    - [ ] `mail_address: String`（邮寄地址）
    - [ ] `mail_method: String`（邮寄方式）
    - [ ] `create_time: String`（创建时间）
- [ ] 创建 `src/commands/qrz_herbertgao.rs` - QRZ.herbertgao.me Tauri 命令
  - [ ] `qrz_herbertgao_query_callsign` 命令
  - [ ] 静默错误处理（查询失败不显示用户提示）
  - [ ] 仅记录错误到控制台日志
- [ ] 更新 `src/qrz/mod.rs` - 导出 QRZ.herbertgao.me 模块
- [ ] 更新 `src/commands/mod.rs` - 导出 qrz_herbertgao 命令模块
- [ ] 更新 `src/main.rs` - 注册 QRZ.herbertgao.me 命令

### 2.5.6 更新地址缓存支持 QRZ.herbertgao.me

- [ ] 更新 `src/db/models.rs` - 扩展 `AddressHistoryEntry`
  - [ ] 添加 `mail_method: Option<String>` 字段（邮寄方式，QRZ.herbertgao.me 使用）
  - [ ] 更新数据比较方法，支持 `source: "qrz.herbertgao.me"` 的字段比较
- [ ] 更新 `web/src/types/models.ts` - TypeScript 类型定义
  - [ ] 添加 `mail_method?: string` 到 AddressHistory 接口

### 2.5.7 更新分发弹框支持三数据源并行查询

- [ ] 更新 `web/src/components/cards/DistributeDialog.vue`
  - [ ] 在现有 QRZ.cn 和 QRZ.com 并行查询基础上增加 QRZ.herbertgao.me
  - [ ] QRZ.herbertgao.me 始终自动查询（无需登录检查）
  - [ ] 三个数据源并行查询，互不影响
  - [ ] QRZ.herbertgao.me 查询失败静默处理（仅控制台日志）
  - [ ] 显示 QRZ.herbertgao.me 查询结果：
    - [ ] 呼号: [呼号] (qrz.herbertgao.me)
    - [ ] 姓名: [姓名]
    - [ ] 邮寄方式: [邮寄方式]
    - [ ] 地址: [地址]
    - [ ] 更新时间: YYYY-MM-DD
    - [ ] 缓存时间: YYYY-MM-DD HH:mm
  - [ ] 提供"复制地址"按钮（单个按钮）
  - [ ] 自动保存到 address_history（智能去重）

### 2.5.8 Phase 2.5 验收测试

- [ ] 可以访问"数据配置"菜单
- [ ] 数据配置页面显示左侧锚点导航
- [ ] 锚点导航包含"QRZ.cn 登录配置"和"QRZ.com 登录配置"项
- [ ] 点击锚点自动滚动到对应配置区域
- [ ] 滚动时锚点自动高亮当前区域
- [ ] 可以在数据配置页面配置 QRZ.cn 登录凭据
- [ ] 可以在数据配置页面配置 QRZ.com 登录凭据
- [ ] 系统钥匙串可用时显示提示
- [ ] 钥匙串不可用时显示降级警告
- [ ] 凭据默认记住,加密存储到钥匙串或本地加密文件
- [ ] 密码不出现在配置文件中
- [ ] 打开配置页面时自动加载已保存的用户名和密码
- [ ] 已保存凭据时显示"已保存凭据"提示
- [ ] 点击"保存并登录"自动执行登录
- [ ] 登录成功后显示"登录成功,凭据已保存"
- [ ] 显示 Cookie 有效期提示
- [ ] Cookie 仅内存存储，应用关闭后丢弃
- [ ] 点击"清除凭据"按钮显示确认对话框
- [ ] 确认清除后正确删除凭据、配置和 Cookie
- [ ] 清除后清空表单输入框
- [ ] 打开分发弹框时自动显示缓存地址（如果有）
- [ ] 地址按标准格式显示（呼号 (qrz.cn)、中文地址、更新时间、缓存时间）
- [ ] 缓存距今 365 天内数据有效
- [ ] 缓存距今超过 365 天显示"数据已过期，建议刷新"提示
- [ ] 可以点击"刷新"按钮手动查询最新地址
- [ ] 未登录时禁用刷新并提示
- [ ] 刷新期间显示加载状态
- [ ] 可以切换查询数据源（QRZ.cn / QRZ.com）
- [ ] 查询结果正确显示数据来源（qrz.cn / qrz.com）
- [ ] QRZ.cn 结果正确显示呼号、中英文地址
- [ ] QRZ.com 结果正确显示呼号、姓名、地址
- [ ] 地址保留换行和格式
- [ ] 可以复制地址（根据数据源显示对应按钮）
- [ ] 查询结果智能保存到 metadata.address_history（按 source 区分）
- [ ] 数据完全一致时仅更新 cached_at，不创建新记录
- [ ] 数据有差异时追加新记录
- [ ] 同一 source 超过 2 条时自动删除最旧记录
- [ ] 地址信息为只读展示，不自动填充到收件地址
- [ ] QRZ.cn Cookie 过期时提示重新登录
- [ ] QRZ.com Cookie 过期时提示重新登录
- [ ] 呼号不存在时显示友好提示（区分数据源）
- [ ] QRZ.herbertgao.me 自动查询（无需登录配置）
- [ ] QRZ.herbertgao.me 查询失败不显示错误提示（静默处理）
- [ ] QRZ.herbertgao.me 结果正确显示呼号、姓名、邮寄方式、地址
- [ ] 三个数据源（QRZ.cn、QRZ.com、QRZ.herbertgao.me）并行查询
- [ ] 各数据源查询互不影响（一个失败不影响其他）

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

### Phase 2.5

**Rust 后端**：
- `src/security/mod.rs`
- `src/security/credentials.rs`
- `src/security/keyring.rs`
- `src/security/encryption.rs`
- `src/commands/security.rs`
- `src/qrz/mod.rs`
- `src/qrz/client.rs`
- `src/qrz/parser.rs`
- `src/commands/qrz.rs`

**前端**：
- `web/src/views/QRZConfigView.vue`
- `web/src/views/CloudDatabaseConfigView.vue`

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

### Phase 2.5

- `Cargo.toml` - 添加依赖（reqwest、scraper、encoding_rs、keyring、aes-gcm、pbkdf2、rand）
- `src/main.rs` - 注册安全命令和 QRZ.cn 命令
- `src/lib.rs` - 导出 security 和 qrz 模块
- `config/qrz.toml` - QRZ 配置文件（不含密码，支持 qrz.cn 和 qrz.com）
- `src/db/models.rs` - 扩展 Card metadata 支持地址历史（多数据源）
- `src/db/cards.rs` - 添加地址历史管理函数（支持多数据源）
- `web/src/App.vue` - 添加"数据配置"菜单项和路由
- `web/src/components/cards/DistributeDialog.vue` - 集成地址查询功能（支持 QRZ.cn 和 QRZ.com，缓存+刷新）

### Phase 3

- `Cargo.toml` - 添加 PostgreSQL、JWT、keyring 依赖
- `src/db/sqlite.rs` - 适配 Repository trait
- `src/db/projects.rs` - 适配 Repository trait
- `src/db/cards.rs` - 适配 Repository trait
- `web/src/App.vue` - 添加数据库配置菜单
