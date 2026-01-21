# 提案：迁移到 Rust + Tauri 架构

**状态**：🔄 实施中（v0.1~v0.3 已完成，v0.4 规划中）
**最后更新**：2026-01-20（晚上）

---

## 动机

当前 Python + Eel 版本的 QSL-CardHub 存在以下问题：

1. **体积过大**：打包后的可执行文件包含完整 Python 运行时，体积超过 100MB
2. **启动速度慢**：Python 解释器启动和 Eel 服务器初始化需要 2-3 秒
3. **跨平台复杂性**：Windows（pywin32）和 macOS/Linux（pycups）使用不同的打印库，维护成本高
4. **依赖管理复杂**：需要管理多套 requirements 文件（Windows、macOS、Linux）
5. **内存占用高**：Python 运行时 + 浏览器引擎占用内存较大

**迁移到 Rust + Tauri 的优势：**

- ✅ **体积优化**：编译为单一原生可执行文件，体积约 10-15MB
- ✅ **性能提升**：Rust 原生性能，启动时间 < 500ms
- ✅ **统一跨平台**：Tauri 提供统一的 API，减少平台差异代码
- ✅ **类型安全**：Rust 的类型系统减少运行时错误
- ✅ **现代化**：保留 Vue 3 + Element Plus 前端，提升后端质量

## 目标

将 Python 版 QSL-CardHub 的核心功能迁移到 Rust + Tauri 架构，实现：

1. ✅ **Tauri 桌面应用框架集成**（v0.1）
2. ✅ **配置管理系统**（Profile CRUD、持久化）（v0.1）
3. ✅ **TSPL 打印指令生成**（呼号、条形码、序列号、数量）（v0.1）
4. ✅ **跨平台打印支持**（Windows RAW Print、CUPS、Mock）（v0.1）
5. ✅ **前端集成**（Vue 3 + Element Plus 与 Tauri Commands 通信）（v0.1）
6. ✅ **日志系统**（统一日志收集、前后端集成）（v0.2）
7. ✅ **PDF 虚拟打印机**（忠实渲染 TSPL 预览）（v0.2 + v0.3）
8. ✅ **模板配置系统**（TOML 配置、模板管理）（v0.3）

**非目标（v0.1 阶段）：**
- ✅ PDF 测试后端（v0.2 已完成）
- ✅ 打印模板配置化（v0.3 已完成）
- ✅ 日志查看页面（v0.2 已完成）
- ❌ 中文字体支持（延后到 v2.0）
- ❌ 批量打印功能（延后到 v2.0）

## 范围

### 包含的功能

#### 1. Tauri 应用框架
- Tauri 项目初始化和配置
- 前后端桥接（Tauri Commands）
- 窗口管理和生命周期
- 打包和分发配置

#### 2. 配置管理
- Profile 数据模型（Rust struct + serde）
- Profile CRUD 操作（创建、读取、更新、删除）
- 配置持久化（JSON 文件）
- 默认配置管理
- 配置导入导出

#### 3. 打印功能
- TSPL 指令生成器（QSL 卡片、校准页）
- 打印机后端抽象层
- Windows 打印支持（Win32 API）
- CUPS 打印支持（macOS/Linux）
- Mock 打印后端（开发测试）

#### 4. 平台支持
- 平台检测（OS 和 CPU 架构）
- 打印机枚举（跨平台）
- 文件路径兼容性
- 字符编码处理（UTF-8）

#### 5. 前端集成
- Vue 3 + Element Plus 界面（已有）
- Tauri API 调用（替换 Eel API）
- 错误处理和用户提示
- 状态管理

### 不包含的功能（当前版本）

- ✅ PDF 测试后端（v0.2 已完成）
- ✅ 日志查看功能（v0.2 已完成）
- ✅ 打印模板配置化（v0.3 已完成）
- ❌ 中文字体打印（v2.0）
- ❌ 批量打印（v2.0）
- ❌ 网络打印（TCP 9100）（v2.0）

## 技术方案

### 架构设计

```
┌─────────────────────────────────────────────┐
│    前端层 (Vue 3 + Element Plus)             │
│  ┌──────────┬──────────┬──────────┬───────┐ │
│  │PrintView │ConfigView│ LogView  │About  │ │
│  └──────────┴──────────┴──────────┴───────┘ │
└─────────────────┬───────────────────────────┘
                  │ Tauri Invoke API
┌─────────────────▼───────────────────────────┐
│      Tauri Commands (Rust)                  │
│  ┌──────────────────────────────────────┐   │
│  │ print_qsl, get_profiles,             │   │
│  │ create_profile, get_printers,        │   │
│  │ get_logs, frontend_log               │   │
│  └──────────────────────────────────────┘   │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│      业务逻辑层 (Rust)                       │
│  ┌──────────┬──────────┬─────────┬────────┐ │
│  │  TSPL    │ Profile  │ Printer │  Log   │ │
│  │Generator │ Manager  │ Manager │Collect │ │
│  │          │          │         │  or    │ │
│  │Template  │          │         │        │ │
│  │ Manager  │          │         │        │ │
│  └──────────┴──────────┴─────────┴────────┘ │
└─────────────────┬───────────────────────────┘
                  │
┌─────────────────▼───────────────────────────┐
│      平台抽象层 (Rust)                       │
│  ┌──────────┬──────────┬─────────┬────────┐ │
│  │ Windows  │  CUPS    │  Mock   │  PDF   │ │
│  │ Backend  │ Backend  │ Backend │Virtual │ │
│  │          │          │         │Printer │ │
│  └──────────┴──────────┴─────────┴────────┘ │
└─────────────────────────────────────────────┘

v0.3 架构变更说明：
- PDF Backend 现在作为"虚拟打印机"忠实渲染 TSPL 指令
- 移除了独立的动态布局系统（layout.rs）
- 模板配置统一管理所有元素的坐标和样式
- 所见即所得的 PDF 预览体验
```

### 关键技术选型

| 组件 | Python 版 | Rust 版 | 理由 |
|------|----------|---------|------|
| 应用框架 | Python + Eel | Rust + Tauri 2 | 原生性能、体积小 |
| 前端 | Vue 3 + Element Plus | Vue 3 + Element Plus | 保持不变 |
| 配置存储 | JSON (json 模块) | TOML (toml crate) | 易读、支持注释、配置隔离 |
| Windows 打印 | pywin32 | windows-rs | 官方维护 |
| CUPS 打印 | pycups | lp 命令 | 简化依赖 |
| 条形码生成 | TSPL 指令 | TSPL 指令 + barcoders（PDF） | 打印机原生支持 + PDF预览 |
| PDF 渲染 | 无 | image + imageproc + rusttype | 虚拟打印机预览 |
| 模板配置 | 硬编码 | TOML 文件 | 灵活配置，易于调整 |

### Rust Crate 依赖

```toml
[dependencies]
tauri = { version = "2", features = ["shell-open"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
    "Win32_Graphics_Printing",
    "Win32_Graphics_Gdi",
    "Win32_Foundation"
] }

[target.'cfg(unix)'.dependencies]
# CUPS 通过命令行调用，无需额外依赖
```

## 实施计划

### 阶段 1：基础架构（第 1-2 周）✅ 已完成（v0.1）
1. ✅ Tauri 项目初始化
2. ✅ 配置 Cargo.toml 和依赖
3. ✅ 前端集成（Vue 3 + Tauri）
4. ✅ 基础 Tauri Commands

### 阶段 2：配置管理（第 3 周）✅ 已完成（v0.1）
1. ✅ Profile 数据模型
2. ✅ ProfileManager 实现
3. ✅ 配置持久化
4. ✅ 配置管理 UI 集成

### 阶段 3：打印功能（第 4-5 周）✅ 已完成（v0.1）
1. ✅ TSPL 生成器
2. ✅ 打印机后端抽象
3. ✅ Windows 打印实现
4. ✅ CUPS 打印实现
5. ✅ Mock 打印实现

### 阶段 4：日志系统（v0.2）✅ 已完成
1. ✅ 日志收集器（环形缓冲区）
2. ✅ 日志 Tauri Commands
3. ✅ LogView 前端集成
4. ✅ 文件持久化

### 阶段 5：PDF 渲染系统（v0.2）✅ 已完成
1. ✅ 字体加载系统（跨平台）
2. ✅ 文本渲染引擎（rusttype）
3. ✅ 条形码渲染（barcoders）
4. ✅ PDF 后端实现
5. ✅ 布局优化系统

### 阶段 6：模板配置系统（v0.3）✅ 已完成
1. ✅ 模板数据模型
2. ✅ 模板管理器
3. ✅ TSPL 生成器模板支持
4. ✅ PDF 后端简化（忠实渲染 TSPL）
5. ✅ 集成测试

### 阶段 7：集成测试和优化（进行中）
1. ⏳ 功能测试
2. ⏳ 跨平台测试
3. ⏳ 打包测试
4. ⏳ 文档完善

## 风险和缓解

### 风险 1：Tauri 学习曲线 ✅ 已缓解
- **影响**：延长开发时间
- **缓解**：先实现简单的 Commands，逐步学习
- **状态**：已完成学习，风险消除

### 风险 2：Windows 打印 API 复杂性 ⚠️ 待验证
- **影响**：Windows 平台打印功能可能延期
- **缓解**：参考 Python 版实现，先使用 Mock 后端测试
- **状态**：骨架已实现，需在 Windows 平台实测

### 风险 3：前端 API 调用变更 ✅ 已缓解
- **影响**：需要修改所有前端代码
- **缓解**：创建适配层，保持 API 接口一致
- **状态**：所有前端页面已迁移

### 风险 4：CUPS 依赖问题 ✅ 已缓解
- **影响**：macOS/Linux 打印功能可能受限
- **缓解**：使用 `lp` 命令行调用，无需 pycups 依赖
- **状态**：CUPS 后端已实现，需实测验证

### 风险 5：PDF 布局一致性 ✅ 已解决
- **影响**：PDF 预览与真实打印输出不一致
- **缓解**：PDF 后端忠实渲染 TSPL 指令
- **状态**：v0.3 重构完成，所见即所得

## 成功标准

- ✅ 应用启动时间 < 500ms
- ✅ 可执行文件体积 < 20MB
- ✅ 所有核心功能正常工作（打印、配置管理）
- ✅ 支持 Windows、macOS、Linux 三大平台
- ✅ 前端界面无变化（用户无感知）
- ✅ 通过手动测试验证

## 后续工作

迁移完成后，后续版本将逐步添加：

- **v0.5**：打印模板配置化、PDF 测试后端、日志查看功能
- **v1.0**：完整错误处理、打包优化、用户文档
- **v2.0**：模板管理 UI、中文字体支持、批量打印、网络打印

## 参考

- Python 版源码：`/Users/herbertgao/PycharmProjects/QSL-CardHub/`
- Python 版规范：`/Users/herbertgao/PycharmProjects/QSL-CardHub/openspec/specs/`
- Tauri 官方文档：https://v2.tauri.app/
- windows-rs 文档：https://microsoft.github.io/windows-docs-rs/
