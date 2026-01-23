# 卡片管理模块

## 概述

为 qsl-cardhub 添加完整的卡片管理功能，包括转卡项目管理、卡片录入、卡片分发/退卡，以及可选的云数据库支持。

## 分阶段实施

本提案分为 3 个阶段渐进式实施：

### Phase 1：卡片管理基础（必需，18-24 小时）

**目标**：建立核心基础设施

- ✅ 本地 SQLite 数据库（自动创建，零配置）
- ✅ 转卡项目 CRUD 操作
- ✅ 菜单导航优化（分组显示）
- ✅ 页面框架搭建（左右分栏）

**交付成果**：
- 用户可以创建和管理转卡项目
- 菜单显示"卡片管理"入口
- 页面框架就绪，等待 Phase 2 填充功能

### Phase 2：卡片录入和管理（必需，20-26 小时）

**目标**：实现完整的卡片生命周期管理

**依赖**：Phase 1 必须完成

- ✅ 卡片录入（单条/连续模式）
- ✅ 卡片列表（查询/筛选/分页）
- ✅ 卡片分发功能
- ✅ 卡片退卡功能
- ✅ 卡片详情查看

**交付成果**：
- 用户可以录入、管理、分发和退卡
- 支持按项目、呼号、状态筛选
- 完整的状态追踪（录入 → 分发 → 退卡）

### Phase 3：云数据库支持（可选，16-20 小时）

**目标**：支持多用户协作和多设备同步

**依赖**：Phase 1 + Phase 2 必须完成

- ✅ 存储模式选择（SQLite / 云数据库）
- ✅ 云数据库连接配置（PostgreSQL）
- ✅ 用户认证（注册/登录/JWT）
- ✅ 数据迁移（SQLite ↔ 云数据库）
- ✅ Repository 抽象层（统一接口）

**交付成果**：
- 用户可以选择使用本地或云端存储
- 支持多用户协作，数据隔离
- 支持多设备同步

## 快速开始

### 查看提案

```bash
cd /Users/herbertgao/RustroverProjects/qsl-cardhub
openspec-cn show add-card-management
```

### 验证提案

```bash
openspec-cn validate add-card-management --strict
```

### 应用变更（批准后）

```bash
# 先实施 Phase 1
openspec-cn apply add-card-management

# Phase 1 完成后再实施 Phase 2
# Phase 2 完成后可选实施 Phase 3
```

## 文档结构

```
openspec/changes/add-card-management/
├── README.md                                   # 本文件
├── proposal.md                                 # 完整提案文档
└── specs/                                      # 规范增量文件
    ├── card-management-core/                   # Phase 1 规范
    │   └── spec.md
    ├── card-entry-and-management/              # Phase 2 规范
    │   └── spec.md
    └── cloud-database-support/                 # Phase 3 规范
        └── spec.md
```

## 规范说明

本提案包含 **3 个规范增量文件**，对应 3 个实施阶段：

1. **card-management-core**（Phase 1）：6 个需求，21 个场景
   - 本地 SQLite 数据库自动初始化
   - 转卡项目管理
   - 菜单导航优化
   - 卡片管理页面框架
   - 错误处理
   - 数据模型定义

2. **card-entry-and-management**（Phase 2）：6 个需求，18 个场景
   - 卡片数据模型
   - 卡片录入功能
   - 卡片列表展示
   - 卡片分发功能
   - 卡片退卡功能
   - 卡片详情查看

3. **cloud-database-support**（Phase 3）：7 个需求，21 个场景
   - 数据存储模式选择
   - 云数据库连接配置
   - 用户认证
   - 数据迁移
   - Repository 抽象层
   - 多用户数据隔离
   - 错误处理和提示

## 时间估算

- **Phase 1**：18-24 小时（约 3 工作日）
- **Phase 2**：20-26 小时（约 3-4 工作日）
- **Phase 3（可选）**：16-20 小时（约 2-3 工作日）

**总计**：54-70 小时（约 7-9 工作日）

## 依赖关系

**实施顺序：**
1. ✅ Phase 1 必须首先完成（基础设施）
2. ✅ Phase 2 依赖 Phase 1（项目表已存在）
3. ⏳ Phase 3 依赖 Phase 1 + Phase 2（可选功能）

**外部依赖：**
- Tauri 项目已配置完成 ✅
- 前端框架（Vue 3 + Element Plus）已就绪 ✅
- 后端命令系统已建立 ✅

**Phase 3 额外依赖：**
- 需要用户自行部署 PostgreSQL 数据库 ⚠️

## 破坏性变更

- ✅ 无破坏性变更，全部为新增功能
- ✅ 不影响现有打印功能

## 注意事项

- **渐进式实施**：Phase 1 + 2 为核心功能，Phase 3 为可选扩展
- **单用户场景**：仅需 Phase 1 + 2，可跳过 Phase 3
- **多用户场景**：需要完整实施 Phase 1 + 2 + 3
- **数据库部署**：Phase 3 需要用户自行部署和维护云数据库
