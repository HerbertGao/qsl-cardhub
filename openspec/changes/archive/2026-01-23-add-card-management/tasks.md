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
  - [x] 分发记录（条件显示）
  - [x] 退卡记录（条件显示）
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

## Phase 2.5：凭据加密存储、数据配置页面和 QRZ.cn 集成 ✅

### 2.5.1 本地凭据加密存储

- [x] 添加加密存储依赖到 Cargo.toml
  - [x] `keyring` (系统钥匙串)
  - [x] `aes-gcm` (AES-256-GCM 加密)
  - [x] `pbkdf2` (密钥派生)
  - [x] `rand` (随机数生成)
- [x] 创建 `src/security/mod.rs` - 安全模块入口
- [x] 创建 `src/security/credentials.rs` - 凭据存储抽象层
  - [x] `CredentialStorage` trait 定义
  - [x] 检测钥匙串可用性
  - [x] 自动选择存储方式（钥匙串/本地加密文件）
  - [x] 统一的存储/读取/删除接口
- [x] 创建 `src/security/keyring.rs` - 系统钥匙串实现
  - [x] `KeyringStorage` 实现 `CredentialStorage`
  - [x] 跨平台钥匙串访问（macOS/Windows/Linux）
  - [x] 错误处理和降级检测
- [x] 创建 `src/security/encryption.rs` - 本地加密文件实现
  - [x] `EncryptedFileStorage` 实现 `CredentialStorage`
  - [x] 主密钥生成（PBKDF2）
  - [x] AES-256-GCM 加密/解密
  - [x] 加密文件读写（原子操作）
  - [x] 文件权限设置（Unix 0600）
- [x] 创建 `src/commands/security.rs` - 凭据管理 Tauri 命令
  - [x] `save_credential(key, value)` 命令
  - [x] `get_credential(key)` 命令
  - [x] `delete_credential(key)` 命令
  - [x] `check_keyring_available()` 命令
- [x] 更新 `src/main.rs` - 注册安全命令

### 2.5.2 数据配置页面

- [x] 更新 `web/src/App.vue` - 添加数据配置子菜单
  - [x] 在"卡片管理"下方添加"数据配置"子菜单
  - [x] 使用 Connection 图标
  - [x] 添加子菜单项："QRZ.cn" 和 "云数据库"（待开发）
  - [x] 添加路由到 QRZConfigView 和 CloudDatabaseConfigView
- [x] 创建 `web/src/views/QRZConfigView.vue` - QRZ.cn 配置页面
  - [x] 标准页面布局（page-content）
  - [x] 页面标题
  - [x] 卡片容器布局（el-card）
  - [x] 用户名输入框
  - [x] 密码输入框（遮罩）
  - [x] "保存并登录"按钮
  - [x] "清除凭据"按钮（带确认对话框）
  - [x] "测试连接"按钮
  - [x] 登录状态显示（已登录/未登录）
  - [x] 已保存凭据提示
  - [x] 存储方式提示（系统钥匙串/本地加密文件）
  - [x] 自动加载已保存的用户名和密码
- [x] 创建 `web/src/views/CloudDatabaseConfigView.vue` - 云数据库配置页面（占位符）
  - [x] 标准页面布局（page-content）
  - [x] 页面标题
  - [x] 空状态占位符（el-empty）

### 2.5.2 QRZ.cn 和 QRZ.com 登录功能（集成凭据存储）

- [x] 添加 HTTP 客户端和 HTML 解析依赖到 Cargo.toml
  - [x] `reqwest`（HTTP 客户端）
  - [x] `scraper`（HTML 解析）
  - [x] `encoding_rs`（GBK 编码转换，仅 QRZ.cn 需要）
- [x] 创建 `src/qrz/mod.rs` - QRZ 模块入口（支持多站点）
- [x] 创建 `config/qrz.toml` 配置文件模板（支持 qrz.cn 和 qrz.com）
- [x] 创建 `src/qrz/client.rs` - HTTP 客户端（统一接口）
  - [x] `login_qrz_cn(username, password) -> Result<QrzCnSession>`
    - [x] Cookie 内存管理（CFID、CFTOKEN）
    - [x] 会话过期检测（推测 30 天）
  - [x] `login_qrz_com(username, password) -> Result<QrzComSession>`
    - [x] Cookie 内存管理（xf_session）
    - [x] 会话过期检测（约 30 天）
  - [x] `query_callsign_cn(session, callsign) -> Result<CallsignInfo>`
    - [x] GBK 到 UTF-8 转换
  - [x] `query_callsign_com(session, callsign) -> Result<CallsignInfo>`
- [x] 创建 `src/qrz/parser.rs` - HTML 解析器（支持多站点）
  - [x] `parse_qrz_cn_callsign(html) -> Result<CallsignInfo>`
    - [x] 提取呼号
    - [x] 提取查询次数
    - [x] 提取更新日期
    - [x] 提取中文地址
    - [x] 提取英文地址
  - [x] `parse_qrz_com_callsign(html) -> Result<CallsignInfo>`
    - [x] 提取呼号（从 `<span class="csignm hamcall">` 标签）
    - [x] 提取姓名和地址（从 `<p class="m0">` 标签）
    - [x] 提取更新时间（从 Detail 标签页）
- [x] 创建 `src/commands/qrz.rs` - QRZ Tauri 命令
  - [x] `qrz_cn_save_and_login` 命令（保存凭据并登录 QRZ.cn）
  - [x] `qrz_cn_load_credentials` 命令（自动加载 QRZ.cn 凭据）
  - [x] `qrz_cn_clear_credentials` 命令（清除 QRZ.cn 凭据和 Cookie）
  - [x] `qrz_cn_query_callsign` 命令
  - [x] `qrz_com_save_and_login` 命令（保存凭据并登录 QRZ.com）
  - [x] `qrz_com_load_credentials` 命令（自动加载 QRZ.com 凭据）
  - [x] `qrz_com_clear_credentials` 命令（清除 QRZ.com 凭据和 Cookie）
  - [x] `qrz_com_query_callsign` 命令
- [x] 更新 `src/main.rs` - 注册 QRZ 命令

### 2.5.3 地址缓存功能（智能去重，支持多数据源）

- [x] 更新 `src/db/models.rs` - 扩展 Card metadata
  - [x] 添加 `AddressEntry` 结构体（单条地址记录）
    - [x] `source: String` - 数据源标识（"qrz.cn" 或 "qrz.com"）
    - [x] `callsign: String` - 呼号
    - [x] `name: Option<String>` - 姓名（可选，QRZ.com 使用）
    - [x] `chinese_address: Option<String>` - 中文地址（可选，QRZ.cn 使用）
    - [x] `english_address: Option<String>` - 英文地址（可选，QRZ.cn 使用）
    - [x] `address: Option<String>` - 地址（QRZ.com 使用）
    - [x] `query_count: Option<u32>` - 查询次数（可选，QRZ.cn 使用）
    - [x] `data_updated_at: String` - 数据更新日期
    - [x] `cached_at: String` - 本地查询时间戳
  - [x] 实现 `AddressEntry` 数据内容比较方法（按 source 区分比较字段）
  - [x] 添加 `AddressCache` 结构体（地址缓存映射，每个来源一条）
  - [x] 序列化/反序列化支持
- [x] 更新 `src/db/cards.rs` - 添加地址缓存管理函数
  - [x] `upsert_address_cache(card_id, entry) -> Result<()>` - 智能插入/更新
    - [x] 检查指定来源是否已有缓存（按 source 区分）
    - [x] 一致时仅更新 cached_at
    - [x] 不一致时直接覆盖原有记录（单版本）
  - [x] `get_address_cache(card_id, source: &str) -> Result<Option<AddressEntry>>` - 获取指定数据源的缓存
- [x] 创建 `src/commands/cards.rs` 中的缓存命令
  - [x] `upsert_card_address` 命令
  - [x] `get_card_latest_address` 命令（支持指定 source）

### 2.5.4 分发弹框中的地址查询（支持多数据源）

- [x] 创建 `web/src/components/qrz/QrzAddressDisplay.vue` - 地址展示组件
  - [x] 支持多数据源显示（qrz.cn 和 qrz.com）
  - [x] QRZ.cn 格式:
    - [x] 呼号: [呼号] (qrz.cn)
    - [x] 中文地址: [中文地址]
    - [x] 英文地址: [英文地址]
    - [x] 更新时间: YYYY-MM-DD
    - [x] 缓存时间: YYYY-MM-DD HH:mm
  - [x] QRZ.com 格式:
    - [x] 呼号: [呼号] (qrz.com)
    - [x] 姓名: [姓名]
    - [x] 地址: [地址（多行）]
    - [x] 更新时间: YYYY-MM-DD
    - [x] 缓存时间: YYYY-MM-DD HH:mm
  - [x] 缓存有效性标识（365天内/已过期）
  - [x] 复制地址按钮（根据数据源动态调整）
  - [x] 刷新按钮
- [x] 更新 `web/src/components/cards/DistributeDialog.vue`
  - [x] 在收件地址区域上方显示地址查询区域
  - [x] 添加数据源选择器（QRZ.cn / QRZ.com）
  - [x] 弹框打开时自动加载最新缓存地址（如果有，优先显示 QRZ.cn）
  - [x] 未登录时禁用对应数据源的查询并提示
  - [x] 集成 QrzAddressDisplay 组件
  - [x] 实现刷新查询逻辑（支持切换数据源）
  - [x] 显示查询加载状态
  - [x] 查询成功后自动保存到地址缓存
  - [x] 地址信息只读展示，不自动填充

### 2.5.5 QRZ.herbertgao.me 集成（公开 API，无需登录）

- [x] 添加 JSON 序列化依赖到 Cargo.toml（如果尚未添加）
  - [x] `serde_json`（JSON 解析）
- [x] 创建 `src/qrz/qrz_herbertgao_client.rs` - QRZ.herbertgao.me JSON API 客户端
  - [x] `query_callsign(callsign) -> Result<Option<HerbertgaoAddressInfo>>`
    - [x] 无需 Cookie 或 Token
    - [x] 筛选 `isShow: true` 的记录
    - [x] 提取第一条有效记录的字段
  - [x] `HerbertgaoAddressInfo` 结构体
    - [x] `call_sign: String`（呼号）
    - [x] `name: String`（姓名）
    - [x] `mail_address: String`（邮寄地址）
    - [x] `mail_method: String`（邮寄方式）
    - [x] `create_time: String`（创建时间）
- [x] 创建 `src/commands/qrz_herbertgao.rs` - QRZ.herbertgao.me Tauri 命令
  - [x] `qrz_herbertgao_query_callsign` 命令
  - [x] 静默错误处理（查询失败不显示用户提示）
  - [x] 仅记录错误到控制台日志
- [x] 更新 `src/qrz/mod.rs` - 导出 QRZ.herbertgao.me 模块
- [x] 更新 `src/commands/mod.rs` - 导出 qrz_herbertgao 命令模块
- [x] 更新 `src/main.rs` - 注册 QRZ.herbertgao.me 命令

### 2.5.6 更新地址缓存支持 QRZ.herbertgao.me

- [x] 更新 `src/db/models.rs` - 扩展 `AddressEntry`
  - [x] 添加 `mail_method: Option<String>` 字段（邮寄方式，QRZ.herbertgao.me 使用）
  - [x] 更新数据比较方法，支持 `source: "qrz.herbertgao.me"` 的字段比较
- [x] 更新 `web/src/types/models.ts` - TypeScript 类型定义
  - [x] 添加 `mail_method?: string` 到 AddressCache 接口

### 2.5.7 更新分发弹框支持三数据源并行查询

- [x] 更新 `web/src/components/cards/DistributeDialog.vue`
  - [x] 在现有 QRZ.cn 和 QRZ.com 并行查询基础上增加 QRZ.herbertgao.me
  - [x] QRZ.herbertgao.me 始终自动查询（无需登录检查）
  - [x] 三个数据源并行查询，互不影响
  - [x] QRZ.herbertgao.me 查询失败静默处理（仅控制台日志）
  - [x] 显示 QRZ.herbertgao.me 查询结果：
    - [x] 呼号: [呼号] (qrz.herbertgao.me)
    - [x] 姓名: [姓名]
    - [x] 邮寄方式: [邮寄方式]
    - [x] 地址: [地址]
    - [x] 更新时间: YYYY-MM-DD
    - [x] 缓存时间: YYYY-MM-DD HH:mm
  - [x] 提供"复制地址"按钮（单个按钮）
  - [x] 自动保存到 address_cache（智能去重）

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
- [ ] 查询结果智能保存到 metadata.address_cache（按 source 区分）
- [ ] 数据完全一致时仅更新 cached_at，不创建新记录
- [ ] 数据有差异时直接覆盖原有记录（单版本）
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

## Phase 3：数据导出导入与云端同步（可选）

> Phase 3 为可选功能，在 Phase 1 + Phase 2 完成后实施。
>
> **重大变更**：原计划的云数据库（PostgreSQL）支持已取消，改为数据导出导入和云端 API 同步功能。

### 3.1 数据导出功能

- [x] 创建 `src/db/export.rs` - 数据导出模块
  - [x] `ExportData` 结构体定义
    - [x] `version: String` - 导出格式版本（"1.0"）
    - [x] `db_version: i32` - 数据库版本号
    - [x] `db_version_display: String` - 可读版本号
    - [x] `app_version: String` - 应用版本号
    - [x] `exported_at: String` - 导出时间戳
    - [x] `tables: ExportTables` - 表数据
  - [x] `ExportTables` 结构体定义
    - [x] `projects: Vec<Project>`
    - [x] `cards: Vec<Card>`
    - [x] `sf_senders: Vec<SFSender>`
    - [x] `sf_orders: Vec<SFOrder>`
  - [x] `export_database() -> Result<ExportData>` - 导出所有数据
- [x] 创建 `src/commands/data_transfer.rs` - 数据传输命令
  - [x] `export_data(file_path: String) -> Result<ExportStats>` - 导出到文件
  - [x] `ExportStats` 结构体（项目数、卡片数、寄件人数、订单数）
- [x] 更新 `src/commands/mod.rs` - 导出模块
- [x] 更新 `src/main.rs` - 注册命令

### 3.2 数据导入功能

- [x] 创建 `src/db/import.rs` - 数据导入模块
  - [x] `ImportPreview` 结构体
    - [x] `version: String` - 文件格式版本
    - [x] `db_version: i32` - 数据库版本号
    - [x] `db_version_display: String` - 可读版本号
    - [x] `app_version: String` - 应用版本号
    - [x] `exported_at: String` - 导出时间
    - [x] `stats: ImportStats` - 数据统计
    - [x] `can_import: bool` - 是否可导入
    - [x] `error_message: Option<String>` - 错误信息
  - [x] `ImportStats` 结构体
    - [x] `projects: u32`
    - [x] `cards: u32`
    - [x] `sf_senders: u32`
    - [x] `sf_orders: u32`
  - [x] `preview_import(file_path: String) -> Result<ImportPreview>` - 预览导入
  - [x] `execute_import(file_path: String) -> Result<ImportStats>` - 执行导入
    - [x] 验证版本号（高版本拒绝导入）
    - [x] 清空现有数据（事务内）
    - [x] 插入新数据（事务内）
    - [x] 错误时回滚
- [x] 更新 `src/commands/data_transfer.rs`
  - [x] `preview_import(file_path: String) -> Result<ImportPreview>`
  - [x] `import_data(file_path: String) -> Result<ImportStats>`

### 3.3 云端同步功能

- [x] 创建 `src/sync/mod.rs` - 同步模块入口
- [x] 创建 `src/sync/client.rs` - 同步 HTTP 客户端
  - [x] `SyncConfig` 结构体
    - [x] `api_url: String` - API 地址
    - [x] `client_id: String` - 客户端标识（UUID）
  - [x] `SyncRequest` 结构体
  - [x] `SyncResponse` 结构体
  - [x] `test_connection(api_url: &str, api_key: &str) -> Result<bool>`
  - [x] `sync_data(config: &SyncConfig, api_key: &str) -> Result<SyncResponse>`
- [x] 创建 `src/commands/sync.rs` - 同步命令
  - [x] `save_sync_config(api_url: String) -> Result<()>`
  - [x] `load_sync_config() -> Result<Option<SyncConfig>>`
  - [x] `save_sync_api_key(api_key: String) -> Result<()>` - 加密存储
  - [x] `has_sync_api_key() -> Result<bool>`
  - [x] `clear_sync_config() -> Result<()>`
  - [x] `test_sync_connection() -> Result<bool>`
  - [x] `execute_sync() -> Result<SyncResponse>`
- [x] 更新 `src/main.rs` - 注册同步命令
- [x] 更新 `src/lib.rs` - 导出 sync 模块

### 3.4 前端界面

- [x] 更新 `web/src/views/DataTransferView.vue`（替换 CloudDatabaseConfigView）
  - [x] 页面标题："数据管理"
  - [x] 导出区域
    - [x] "导出数据"按钮
    - [x] 调用 Tauri 文件保存对话框
    - [x] 显示导出结果统计
  - [x] 导入区域
    - [x] "导入数据"按钮
    - [x] 调用 Tauri 文件选择对话框
    - [x] 导入预览对话框
    - [x] 版本检查结果显示
    - [x] 覆盖警告提示
  - [x] 云端同步区域
    - [x] API 地址输入框
    - [x] API Key 输入框（密码类型）
    - [x] "保存配置"按钮
    - [x] "测试连接"按钮
    - [x] "立即同步"按钮
    - [x] 同步状态显示
    - [x] 上次同步时间显示
- [x] 更新 `web/src/App.vue`
  - [x] 将"云数据库"菜单项改为"数据管理"
  - [x] 路由指向 DataTransferView

### 3.5 API 规范文档

- [x] 创建 `docs/cloud-sync-api-spec.md`
  - [x] API 概述
  - [x] 认证方式（API Key）
  - [x] 接口列表
    - [x] `GET /ping` - 连接测试
    - [x] `POST /sync` - 全量同步
  - [x] 请求格式
  - [x] 响应格式
  - [x] 错误码定义
  - [x] 数据结构定义
  - [x] 示例请求和响应
  - [x] 云端实现建议

### 3.6 Phase 3 验收测试

**数据导出验收：**
- [ ] 点击"导出数据"弹出保存对话框
- [ ] 默认文件名格式：`qslhub_backup_YYYYMMDD_HHmmss.qslhub`
- [ ] 导出文件为有效 JSON
- [ ] 文件包含正确的版本信息
- [ ] 文件包含所有表数据
- [ ] 导出成功显示统计

**数据导入验收：**
- [ ] 点击"导入数据"弹出选择对话框
- [ ] 过滤 `.qslhub` 文件
- [ ] 选择后显示预览对话框
- [ ] 高版本文件显示错误，禁止导入
- [ ] 兼容版本显示警告确认
- [ ] 确认后执行导入
- [ ] 导入成功刷新数据
- [ ] 导入失败回滚，显示错误

**云端同步验收：**
- [ ] 可以输入 API 地址
- [ ] 可以输入 API Key
- [ ] API Key 加密存储
- [ ] "保存配置"保存成功提示
- [ ] "测试连接"显示结果
- [ ] "立即同步"触发同步
- [ ] 同步中显示进度
- [ ] 同步成功显示统计
- [ ] 同步失败显示错误
- [ ] 上次同步时间正确记录

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
- `src/db/export.rs` - 数据导出模块
- `src/db/import.rs` - 数据导入模块
- `src/sync/mod.rs` - 同步模块入口
- `src/sync/client.rs` - 同步 HTTP 客户端
- `src/commands/data_transfer.rs` - 数据传输命令
- `src/commands/sync.rs` - 同步命令

**前端**：
- `web/src/views/DataTransferView.vue` - 数据管理页面（替换 CloudDatabaseConfigView）

**文档**：
- `docs/cloud-sync-api-spec.md` - 云端同步 API 规范文档

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
- `src/db/models.rs` - 扩展 Card metadata 支持地址缓存（每个来源一条）
- `src/db/cards.rs` - 添加地址缓存管理函数（单版本）
- `web/src/App.vue` - 添加"数据配置"菜单项和路由
- `web/src/components/cards/DistributeDialog.vue` - 集成地址查询功能（支持 QRZ.cn 和 QRZ.com，缓存+刷新）

### Phase 3

- `src/db/mod.rs` - 导出 export/import 模块
- `src/commands/mod.rs` - 导出 data_transfer/sync 命令模块
- `src/lib.rs` - 导出 sync 模块
- `src/main.rs` - 注册数据传输和同步命令
- `web/src/App.vue` - 更新菜单（"云数据库"改为"数据管理"）

## 删除文件清单

### Phase 3

- `web/src/views/CloudDatabaseConfigView.vue` - 移除占位页面（被 DataTransferView 替代）
