# 提案：卡片管理模块

**状态**：📋 提案中
**最后更新**：2026-01-21

---

## 为什么

当前 qsl-cardhub 系统仅提供打印功能，缺少卡片录入、管理和分发的完整流程，导致以下问题：

1. **无法追踪卡片状态**：打印后无法记录卡片是否已分发、退卡等状态
2. **缺少项目管理**：无法按转卡项目组织卡片，不便于批量管理
3. **数据无法持久化**：每次打印都是独立操作，无法查询历史记录
4. **单用户限制**：无法支持多用户协作和多设备同步

**解决方案价值：**

- ✅ **项目化管理**：按转卡项目组织卡片，便于分类管理
- ✅ **状态追踪**：记录卡片从录入 → 分发 → 退卡的完整生命周期
- ✅ **数据持久化**：本地 SQLite 数据库存储，支持查询和统计
- ✅ **批量操作**：支持连续录入、批量筛选查询
- ✅ **多设备同步**（可选）：支持云数据库，实现多用户协作和多设备访问

## 变更内容

新增卡片管理模块，分 3 个阶段实施：

### Phase 1：卡片管理基础（必需）

**目标**：建立核心基础设施

1. **本地 SQLite 数据库**
   - 首次启动自动创建数据库
   - 自动执行初始化脚本
   - 数据库版本管理
   - 零配置，开箱即用

2. **转卡项目管理**
   - 创建项目（输入项目名称）
   - 查询项目列表（按创建时间降序）
   - 更新项目（重命名）
   - 删除项目（带确认）
   - 项目统计显示

3. **菜单导航优化**
   - 菜单分组（用横线分隔）
   - 新增"卡片管理"菜单项
   - 图标优化（Box 图标）

4. **页面框架**
   - 左右分栏布局
   - 项目列表组件
   - 占位符组件（Phase 2 替换）
   - 响应式设计

### Phase 2：卡片录入和管理（必需）

**目标**：实现完整的卡片生命周期管理

**依赖**：Phase 1 必须完成

1. **卡片数据模型**
   - cards 表结构定义
   - 外键关联 projects
   - 状态枚举：pending、distributed、returned
   - 元数据 JSON 存储

2. **卡片录入**
   - 单条录入模式
   - 连续录入模式
   - 呼号验证（3-10字符）
   - 数量验证（1-9999）
   - 快捷键支持（Enter/Esc）

3. **卡片列表**
   - 按项目筛选
   - 按呼号搜索（模糊匹配）
   - 按状态筛选
   - 分页显示（每页20条）

4. **卡片分发**
   - 处理方式选择（直接分发、邮寄、自取）
   - 分发地址录入
   - 备注信息
   - 状态更新

5. **卡片退卡**
   - 退卡原因选择
   - 备注信息
   - 状态更新

6. **卡片详情**
   - 基本信息查看
   - 分发历史（如果有）
   - 退卡历史（如果有）

7. **本地凭据加密存储**
   - 优先使用系统钥匙串（Keychain/Credential Manager/Secret Service）
   - 降级使用本地加密文件（AES-256-GCM）
   - 统一凭据存储接口
   - 支持密码、Cookie、Token 加密存储
   - 配置文件不包含敏感信息

8. **数据配置页面**
   - 新增"数据配置"菜单（卡片管理下方）
   - 左侧锚点导航
   - 右侧配置内容区域
   - 锚点自动滚动和高亮

9. **QRZ.cn 集成**
   - 数据配置页面中的登录配置区域
   - 用户名密码默认记住并加密存储
   - 自动加载已保存凭据
   - "保存并登录"一键保存并登录
   - "清除凭据"按钮（带确认）
   - 获取并保存 Cookie（CFID、CFTOKEN，仅内存）
   - 分发弹框中的地址查询
   - 优先显示缓存地址（1年有效期）
   - 支持手动刷新查询
   - HTML 解析提取中英文地址
   - 标准格式展示（呼号 (qrz.cn)、中文地址、更新时间、缓存时间）
   - 智能缓存去重（数据一致时仅更新时间）
   - 多版本地址历史（同一来源最多2条）
   - 地址复制功能

10. **QRZ.com 集成**
   - 数据配置页面中的登录配置区域
   - 用户名密码默认记住并加密存储
   - 自动加载已保存凭据
   - "保存并登录"一键保存并登录
   - "清除凭据"按钮（带确认）
   - 获取并保存 Cookie（xf_session，仅内存）
   - 分发弹框中的地址查询
   - 优先显示缓存地址（1年有效期）
   - 支持手动刷新查询
   - HTML 解析提取呼号和地址信息
   - 标准格式展示（呼号 (qrz.com)、姓名、地址、更新时间、缓存时间）
   - 智能缓存去重（数据一致时仅更新时间）
   - 多版本地址历史（同一来源最多2条）
   - 地址复制功能

11. **QRZ.herbertgao.me 集成**
   - 无需登录配置，公开 API
   - 分发弹框中自动查询地址
   - 与 QRZ.cn、QRZ.com 并行查询
   - JSON API 查询（仅取 isShow: true 的记录）
   - 提取呼号、姓名、邮寄方式、地址、创建时间
   - 标准格式展示（呼号 (qrz.herbertgao.me)、姓名、邮寄方式、地址、更新时间、缓存时间）
   - 智能缓存去重（数据一致时仅更新时间）
   - 多版本地址历史（同一来源最多2条）
   - 地址复制功能
   - 查询失败静默处理

### Phase 3：云数据库支持（可选）

**目标**：支持多用户协作和多设备同步

**依赖**：Phase 1 + Phase 2 必须完成

1. **数据库配置界面**
   - 存储模式选择（SQLite / 云数据库）
   - SQLite 模式：显示文件路径、状态、记录数、备份按钮
   - 云数据库模式：显示连接配置表单

2. **云数据库配置**
   - 数据库类型（PostgreSQL / MySQL）
   - 主机地址、端口、数据库名
   - 用户名、密码（加密存储）
   - SSL/TLS 连接（开关）
   - 测试连接、初始化数据库按钮

3. **用户认证**
   - 注册功能
   - 登录功能
   - JWT Token 管理
   - Token 自动刷新
   - 多用户隔离

4. **数据迁移**
   - SQLite → 云数据库
   - 云数据库 → SQLite
   - 显示迁移进度
   - 冲突解决策略

5. **数据库抽象层**
   - Repository 模式
   - 统一接口
   - SQLite 和 PostgreSQL 实现
   - 配置驱动

## 范围

### 包含的功能

**Phase 1（18-24 小时）：**
- ✅ 本地 SQLite 数据库
- ✅ 转卡项目 CRUD
- ✅ 菜单导航优化
- ✅ 页面框架搭建

**Phase 2（24-31 小时）：**
- ✅ 卡片 CRUD 操作
- ✅ 卡片录入（单条/连续）
- ✅ 卡片列表（查询/筛选/分页）
- ✅ 分发和退卡功能
- ✅ 卡片详情查看
- ✅ 本地凭据加密存储（系统钥匙串+本地加密）
- ✅ 数据配置页面（锚点导航）
- ✅ QRZ.cn 登录配置
- ✅ QRZ.com 登录配置
- ✅ QRZ.herbertgao.me 公开 API 集成
- ✅ 分发弹框中的地址查询（支持 QRZ.cn、QRZ.com、QRZ.herbertgao.me，缓存+手动刷新）
- ✅ 地址历史多版本管理（支持多数据源）

**Phase 3（16-20 小时，可选）：**
- ✅ 数据库配置界面
- ✅ PostgreSQL 连接支持
- ✅ 用户注册和登录
- ✅ JWT Token 管理
- ✅ 数据迁移功能
- ✅ Repository 抽象层

### 不包含的功能

❌ **批量导入/导出**：后续版本
❌ **打印集成**：后续版本（将卡片数据直接用于打印）
❌ **MySQL 支持**：仅 PostgreSQL（MySQL 可后续添加）
❌ **离线同步**：后续版本
❌ **权限管理**：后续版本（角色、协作者）
❌ **第三方登录**：后续版本（OAuth）
❌ **其他呼号查询网站**：仅支持 QRZ.cn、QRZ.com、QRZ.herbertgao.me（其他网站可后续添加）
❌ **自动地址填充**：查询结果仅展示，不自动填入分发地址
❌ **批量查询**：不支持批量查询多个卡片地址
❌ **定时刷新**：QRZ.herbertgao.me 在弹框打开时自动查询，但不定时检测缓存过期

## 影响

### 新增文件

**Phase 1（Rust 后端）：**
- `src/db/sqlite.rs` - SQLite 数据库管理
- `src/db/projects.rs` - 项目 CRUD 操作
- `src/db/models.rs` - 数据模型定义
- `src/commands/projects.rs` - 项目管理 Tauri 命令
- `migrations/001_init.sql` - 数据库初始化脚本

**Phase 1（前端）：**
- `web/src/views/CardManagementView.vue` - 卡片管理页面（框架）
- `web/src/components/projects/ProjectList.vue` - 项目列表
- `web/src/components/projects/ProjectDialog.vue` - 新建/编辑项目弹窗
- `web/src/components/cards/CardListPlaceholder.vue` - 占位符（Phase 2 替换）

**Phase 2（Rust 后端）：**
- `src/db/cards.rs` - 卡片 CRUD 操作和地址历史管理
- `src/commands/cards.rs` - 卡片管理 Tauri 命令
- `src/security/credentials.rs` - 凭据加密存储（系统钥匙串+本地加密）
- `src/security/keyring.rs` - 系统钥匙串集成
- `src/security/encryption.rs` - AES-256-GCM 加密实现
- `src/qrz/mod.rs` - QRZ 集成模块（支持 qrz.cn、qrz.com、qrz.herbertgao.me）
- `src/qrz/qrz_cn_client.rs` - QRZ.cn HTTP 客户端
- `src/qrz/qrz_cn_parser.rs` - QRZ.cn HTML 解析器
- `src/qrz/qrz_com_client.rs` - QRZ.com HTTP 客户端
- `src/qrz/qrz_com_parser.rs` - QRZ.com HTML 解析器
- `src/qrz/qrz_herbertgao_client.rs` - QRZ.herbertgao.me JSON API 客户端
- `src/commands/qrz_cn.rs` - QRZ.cn 相关 Tauri 命令
- `src/commands/qrz_com.rs` - QRZ.com 相关 Tauri 命令
- `src/commands/qrz_herbertgao.rs` - QRZ.herbertgao.me 相关 Tauri 命令
- `migrations/002_add_cards.sql` - cards 表创建脚本

**Phase 2（前端）：**
- `web/src/components/cards/CardList.vue` - 卡片列表（替换占位符）
- `web/src/components/cards/CardInputDialog.vue` - 录入弹窗
- `web/src/components/cards/DistributeDialog.vue` - 分发弹窗
- `web/src/components/cards/ReturnDialog.vue` - 退卡弹窗
- `web/src/components/cards/CardDetailDialog.vue` - 详情弹窗
- `web/src/views/QRZConfigView.vue` - QRZ 配置页面（支持 QRZ.cn 和 QRZ.com）
- `web/src/views/CloudDatabaseConfigView.vue` - 云数据库配置页面（占位符）

**Phase 3（Rust 后端）：**
- `src/db/repository.rs` - Repository trait 定义
- `src/db/postgres/` - PostgreSQL 实现
- `src/auth/` - 认证模块（JWT、密码加密）
- `src/commands/auth.rs` - 认证 API
- `src/commands/database.rs` - 数据库配置 API
- `src/config/database.rs` - 数据库配置管理

**Phase 3（前端）：**
- `web/src/views/DatabaseConfigView.vue` - 数据库配置页面
- `web/src/components/auth/` - 登录/注册组件
- `web/src/stores/auth.js` - 用户状态管理（Pinia）

### 修改文件

**Phase 1：**
- `web/src/App.vue` - 添加卡片管理菜单项和路由
- `src/main.rs` - 注册项目管理命令

**Phase 2：**
- `web/src/App.vue` - 添加"数据配置"菜单项和路由
- `web/src/views/CardManagementView.vue` - 替换占位符为CardList
- `web/src/components/cards/DistributeDialog.vue` - 集成地址查询功能（支持 QRZ.cn 和 QRZ.com，缓存+刷新）
- `src/main.rs` - 注册卡片管理命令和 QRZ 命令
- `src/db/models.rs` - 添加 Card 数据模型和地址历史结构体
- `Cargo.toml` - 添加依赖（HTTP 客户端、HTML 解析、加密库、keyring）
- `config/qrz.toml` - QRZ 配置文件（不含密码，支持 qrz.cn 和 qrz.com）

**Phase 3：**
- `Cargo.toml` - 添加 PostgreSQL、JWT、加密依赖
- `src/db/sqlite.rs` - 适配 Repository trait
- `src/db/projects.rs` - 适配 Repository trait
- `src/db/cards.rs` - 适配 Repository trait

## 验收标准

### Phase 1 验收

- [ ] 首次启动自动创建数据库文件
- [ ] 数据库初始化脚本自动执行
- [ ] 可以创建转卡项目
- [ ] 可以查询项目列表（按创建时间降序）
- [ ] 可以重命名项目
- [ ] 可以删除项目（带确认）
- [ ] 菜单显示分组横线
- [ ] "卡片管理"菜单项正常工作
- [ ] 左右分栏布局正确显示
- [ ] 项目列表联动显示（右侧显示占位符）

### Phase 2 验收

- [ ] 可以录入卡片，选择项目、输入呼号和数量
- [ ] 呼号格式验证正确（3-10字符）
- [ ] 数量范围验证正确（1-9999）
- [ ] 连续录入模式正常工作
- [ ] 快捷键（Enter/Esc）生效
- [ ] 显示选中项目的所有卡片
- [ ] 按呼号搜索正常（模糊匹配）
- [ ] 按状态筛选正常
- [ ] 分页功能正常
- [ ] 只能分发"已录入"状态的卡片
- [ ] 邮寄方式时地址必填
- [ ] 分发成功后状态更新为"已分发"
- [ ] 可以退卡"已录入"和"已分发"状态的卡片
- [ ] 退卡成功后状态更新为"已退卡"
- [ ] 显示完整的卡片详情
- [ ] 可以访问"数据配置"菜单
- [ ] 数据配置页面显示锚点导航
- [ ] 点击锚点自动滚动到对应区域
- [ ] 滚动时锚点自动高亮
- [ ] 可以在数据配置页面配置 QRZ.cn 登录凭据
- [ ] 可以测试 QRZ.cn 连接并获取 Cookie
- [ ] 打开分发弹框时自动显示缓存地址（如果有）
- [ ] 缓存显示查询时间和有效性标识
- [ ] 可以手动刷新查询最新地址
- [ ] 查询结果正确显示中英文地址
- [ ] 可以复制地址信息
- [ ] 数据一致时仅更新缓存时间（智能去重）
- [ ] 数据变化时追加新记录到地址历史
- [ ] 同一来源超过 2 条时自动删除最旧记录

### Phase 3 验收（可选）

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

## 时间估算

- **Phase 1**：18-24 小时（约 3 工作日）
- **Phase 2**：24-31 小时（约 3-4 工作日，包含加密存储、数据配置页面和 QRZ.cn 集成）
- **Phase 3（可选）**：16-20 小时（约 2-3 工作日）

**总计**：58-75 小时（约 7-10 工作日）

## 依赖关系

**实施顺序：**
1. Phase 1 必须首先完成（基础设施）
2. Phase 2 依赖 Phase 1 完成（项目表已存在）
3. Phase 3 依赖 Phase 1 + Phase 2 完成（可选功能）

**外部依赖：**
- Tauri 项目已配置完成
- 前端框架（Vue 3 + Element Plus）已就绪
- 后端命令系统已建立

**Phase 3 额外依赖：**
- 需要用户自行部署 PostgreSQL 数据库

## 注意事项

- **渐进式实施**：Phase 1 + 2 为核心功能，Phase 3 为可选扩展
- **单用户场景**：仅需 Phase 1 + 2，可跳过 Phase 3
- **多用户场景**：需要完整实施 Phase 1 + 2 + 3
- **数据库部署**：Phase 3 需要用户自行部署和维护云数据库
- **向后兼容**：Phase 3 的 Repository 模式不影响 Phase 1 + 2 的功能
