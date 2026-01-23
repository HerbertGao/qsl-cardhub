# 提案：卡片管理模块

**状态**：📋 提案中
**最后更新**：2026-01-21

---

## 为什么

当前 qsl-cardhub 系统仅提供打印功能，缺少卡片录入、管理和分发的完整流程，导致以下问题：

1. **无法追踪卡片状态**：打印后无法记录卡片是否已分发、退卡等状态
2. **缺少项目管理**：无法按转卡项目组织卡片，不便于批量管理
3. **数据无法持久化**：每次打印都是独立操作，无法查询过往记录
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
   - 分发记录（如果有）
   - 退卡记录（如果有）

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
   - 单版本地址缓存（每个来源只保存1条最新记录）
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
   - 单版本地址缓存（每个来源只保存1条最新记录）
   - 地址复制功能

11. **QRZ.herbertgao.me 集成**
   - 无需登录配置，公开 API
   - 分发弹框中自动查询地址
   - 与 QRZ.cn、QRZ.com 并行查询
   - JSON API 查询（仅取 isShow: true 的记录）
   - 提取呼号、姓名、邮寄方式、地址、创建时间
   - 标准格式展示（呼号 (qrz.herbertgao.me)、姓名、邮寄方式、地址、更新时间、缓存时间）
   - 智能缓存去重（数据一致时仅更新时间）
   - 单版本地址缓存（每个来源只保存1条最新记录）
   - 地址复制功能
   - 查询失败静默处理

### Phase 3：数据导出导入与云端同步（可选）

> **重大变更**：原计划的云数据库（PostgreSQL）支持已取消，改为数据导出导入和云端 API 同步功能。

**目标**：支持数据备份、迁移和云端查询

**依赖**：Phase 1 + Phase 2 必须完成

1. **数据导出功能**
   - 导出为 JSON 文件（.qslhub 扩展名）
   - 包含版本信息（数据库版本、应用版本）
   - 包含所有表数据（projects、cards、sf_senders、sf_orders）
   - 显示导出统计

2. **数据导入功能**
   - 从 JSON 文件导入
   - 版本兼容性检查（高版本拒绝导入）
   - 导入预览（显示版本、数据统计）
   - 覆盖模式（清空后导入，事务保证原子性）
   - 错误时回滚

3. **云端数据同步**
   - 配置云端 API 地址
   - API Key 认证（加密存储）
   - 测试连接功能
   - 全量同步（本地 → 云端）
   - 同步状态显示

4. **云端 API 规范**
   - 提供接口规范文档
   - 认证方式：API Key（Bearer Token）
   - 接口：`GET /ping`（连接测试）、`POST /sync`（全量同步）
   - 数据结构定义
   - 云端实现建议

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
- ✅ 地址缓存单版本管理（每个来源只保存1条最新记录）

**Phase 3（18-25 小时，可选）：**
- ✅ 数据导出功能（JSON 格式，.qslhub 文件）
- ✅ 数据导入功能（版本检查、覆盖模式）
- ✅ 云端 API 同步（API Key 认证、全量同步）
- ✅ API 规范文档

### 不包含的功能

❌ **云数据库支持**：不再支持 PostgreSQL/MySQL 直连（改用 API 同步）
❌ **用户认证系统**：不需要注册/登录/JWT（使用 API Key）
❌ **增量同步**：仅支持全量同步
❌ **双向同步**：仅支持本地 → 云端
❌ **合并导入**：仅支持覆盖导入（后续版本可添加）
❌ **打印集成**：后续版本（将卡片数据直接用于打印）
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
- `src/db/cards.rs` - 卡片 CRUD 操作和地址缓存管理
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
- `web/src/views/CloudDatabaseConfigView.vue` - 云数据库配置页面（占位符，Phase 3 替换）

**Phase 3（Rust 后端）：**
- `src/db/export.rs` - 数据导出模块
- `src/db/import.rs` - 数据导入模块
- `src/sync/mod.rs` - 同步模块入口
- `src/sync/client.rs` - 同步 HTTP 客户端
- `src/commands/data_transfer.rs` - 数据传输命令
- `src/commands/sync.rs` - 同步命令

**Phase 3（前端）：**
- `web/src/views/DataTransferView.vue` - 数据管理页面（替换 CloudDatabaseConfigView）

**Phase 3（文档）：**
- `docs/cloud-sync-api-spec.md` - 云端同步 API 规范文档

### 修改文件

**Phase 1：**
- `web/src/App.vue` - 添加卡片管理菜单项和路由
- `src/main.rs` - 注册项目管理命令

**Phase 2：**
- `web/src/App.vue` - 添加"数据配置"菜单项和路由
- `web/src/views/CardManagementView.vue` - 替换占位符为CardList
- `web/src/components/cards/DistributeDialog.vue` - 集成地址查询功能（支持 QRZ.cn 和 QRZ.com，缓存+刷新）
- `src/main.rs` - 注册卡片管理命令和 QRZ 命令
- `src/db/models.rs` - 添加 Card 数据模型和地址缓存结构体
- `Cargo.toml` - 添加依赖（HTTP 客户端、HTML 解析、加密库、keyring）
- `config/qrz.toml` - QRZ 配置文件（不含密码，支持 qrz.cn 和 qrz.com）

**Phase 3：**
- `src/db/mod.rs` - 导出 export/import 模块
- `src/commands/mod.rs` - 导出 data_transfer/sync 命令模块
- `src/lib.rs` - 导出 sync 模块
- `src/main.rs` - 注册数据传输和同步命令
- `web/src/App.vue` - 更新菜单（"云数据库"改为"数据管理"）

**Phase 3 删除文件：**
- `web/src/views/CloudDatabaseConfigView.vue` - 移除占位页面

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
- [ ] 数据变化时直接覆盖原有记录（单版本）

### Phase 3 验收（可选）

**数据导出：**
- [ ] 点击"导出数据"弹出保存对话框
- [ ] 默认文件名格式：`qslhub_backup_YYYYMMDD_HHmmss.qslhub`
- [ ] 导出文件为有效 JSON
- [ ] 文件包含正确的版本信息
- [ ] 文件包含所有表数据
- [ ] 导出成功显示统计

**数据导入：**
- [ ] 点击"导入数据"弹出选择对话框
- [ ] 过滤 `.qslhub` 文件
- [ ] 选择后显示预览对话框
- [ ] 高版本文件显示错误，禁止导入
- [ ] 兼容版本显示警告确认
- [ ] 确认后执行导入
- [ ] 导入成功刷新数据
- [ ] 导入失败回滚，显示错误

**云端同步：**
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

## 时间估算

- **Phase 1**：18-24 小时（约 3 工作日）
- **Phase 2**：24-31 小时（约 3-4 工作日，包含加密存储、数据配置页面和 QRZ.cn 集成）
- **Phase 3（可选）**：18-25 小时（约 2-3 工作日）

**总计**：60-80 小时（约 8-10 工作日）

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
- 云端同步需要用户自行实现接收 API（提供规范文档）

## 注意事项

- **渐进式实施**：Phase 1 + 2 为核心功能，Phase 3 为可选扩展
- **单用户场景**：仅需 Phase 1 + 2，可跳过 Phase 3
- **数据备份**：Phase 3 的导出功能可用于数据备份
- **云端查询**：Phase 3 的同步功能可用于将数据发送到云端供其他系统查询
- **向后兼容**：Phase 3 不影响 Phase 1 + 2 的功能
- **无需部署数据库**：Phase 3 不再需要用户部署 PostgreSQL/MySQL
