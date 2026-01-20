# QSL-CardHub 架构文档

## 概述

本文档描述 QSL-CardHub Rust + Tauri 版本的架构设计和模块组织。

## 架构图

```
┌─────────────────────────────────────────────────────────┐
│                   前端层 (Vue 3)                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │ PrintView │ ConfigView │ AboutView                │  │
│  └───────────────────────────────────────────────────┘  │
└──────────────────────┬──────────────────────────────────┘
                       │ Tauri Invoke API
                       │ (JSON-RPC over IPC)
┌──────────────────────▼──────────────────────────────────┐
│              Tauri Commands 层 (Rust)                    │
│  ┌───────────────────────────────────────────────────┐  │
│  │ get_platform_info()                               │  │
│  │ get_profiles(), create_profile(), ...             │  │
│  │ get_printers(), print_qsl(), ...                  │  │
│  └───────────────────────────────────────────────────┘  │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│               业务逻辑层 (Rust)                          │
│  ┌──────────────┬──────────────┬──────────────────┐    │
│  │ProfileManager│PrinterManager│ TSPLGenerator    │    │
│  │- CRUD 操作   │- 枚举打印机  │ - 生成 TSPL      │    │
│  │- 持久化      │- 路由打印    │ - 布局计算       │    │
│  └──────────────┴──────────────┴──────────────────┘    │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│              平台抽象层 (Rust Traits)                    │
│  ┌───────────────────────────────────────────────────┐  │
│  │ PrinterBackend trait                              │  │
│  │ - list_printers()                                 │  │
│  │ - send_raw()                                      │  │
│  └───────────────────────────────────────────────────┘  │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│              平台实现层 (条件编译)                        │
│  ┌────────────┬────────────┬──────────────────────┐    │
│  │  Windows   │   CUPS     │       Mock           │    │
│  │  Backend   │  Backend   │      Backend         │    │
│  │ (Win32 API)│(lp 命令)   │ (文件输出)           │    │
│  └────────────┴────────────┴──────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

## 模块说明

### 1. Commands 模块 (`src/commands/`)

**职责**：定义前端可调用的 Tauri API

**子模块**：
- `platform.rs` - 平台信息相关 API
- `profile.rs` - 配置管理相关 API
- `printer.rs` - 打印功能相关 API

**关键特性**：
- 统一的错误处理（`Result<T, String>`）
- 异步 API（`async fn`）
- 状态管理（`State<'_, T>`）

### 2. Config 模块 (`src/config/`)

**职责**：配置文件管理和持久化

**子模块**：
- `models.rs` - 数据模型定义
- `profile_manager.rs` - Profile CRUD 操作

**数据模型**：
- `Profile` - 打印配置
- `AppConfig` - 全局配置
- `Platform` - 平台信息
- `PrinterConfig` - 打印机配置
- `PaperSpec` - 纸张规格
- `Template` - 模板配置

**存储格式**：TOML

### 3. Printer 模块 (`src/printer/`)

**职责**：打印功能实现

**子模块**：
- `tspl.rs` - TSPL 指令生成器
- `manager.rs` - 打印机管理器
- `backend/` - 打印后端实现
  - `mod.rs` - PrinterBackend trait 定义
  - `windows.rs` - Windows 平台实现
  - `cups.rs` - CUPS 平台实现
  - `mock.rs` - Mock 实现（开发测试）

**关键特性**：
- 跨平台抽象（trait）
- 条件编译（`#[cfg(...)]`）
- 后端自动选择

### 4. Utils 模块 (`src/utils/`)

**职责**：通用工具函数

**子模块**：
- `platform.rs` - 平台检测

### 5. Error 模块 (`src/error/`)

**职责**：统一的错误类型定义

**错误类型**：
- `AppError::Config` - 配置错误
- `AppError::Print` - 打印错误
- `AppError::Io` - IO 错误
- `AppError::Serde` - 序列化错误

## 数据流

### 配置管理流程

```
前端点击"创建配置"
    ↓
Commands::create_profile()
    ↓
ProfileManager::create()
    ↓
Profile::new() + 生成 UUID
    ↓
保存到 config/profiles/{uuid}.toml
    ↓
返回新创建的 Profile
```

### 打印流程

```
前端点击"打印"
    ↓
Commands::print_qsl()
    ↓
PrinterManager::print_qsl()
    ↓
TSPLGenerator::generate_qsl_card()
    ↓
PrinterManager::send_raw()
    ↓
选择合适的 PrinterBackend
    ↓
Backend::send_raw()
    ↓
Windows: Win32 API / CUPS: lp 命令 / Mock: 保存文件
```

## 状态管理

### 应用状态

```rust
ProfileState {
    manager: Arc<Mutex<ProfileManager>>
}

PrinterState {
    manager: Arc<Mutex<PrinterManager>>
}
```

### 状态共享

- 使用 `Arc<Mutex<T>>` 实现线程安全的状态共享
- Tauri 通过 `app.manage()` 管理全局状态
- Commands 通过 `State<'_, T>` 访问状态

## 配置文件结构

```
config/
├── config.toml              # 全局配置
│   ├── default_profile_id
│   └── window_state
├── profiles/                # Profile 配置
│   ├── {uuid-1}.toml
│   ├── {uuid-2}.toml
│   └── example.toml
└── templates/               # 打印模板（v0.5）
    └── qsl-card-v1.toml
```

## 跨平台策略

### 条件编译

```rust
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Printing::*;

#[cfg(target_family = "unix")]
use std::process::Command;
```

### 后端选择

```rust
#[cfg(target_os = "windows")]
backends.push(Box::new(WindowsBackend::new()));

#[cfg(target_family = "unix")]
backends.push(Box::new(CupsBackend::new()));

// Mock 后端总是可用
backends.push(Box::new(MockBackend::new(output_dir)?));
```

## 扩展点（v0.5+）

### 模板配置化

- 当前：硬编码布局参数
- v0.5：从 TOML 文件加载模板配置
- 扩展点：`TSPLGenerator::generate_from_template()`

### PDF 测试后端

- v0.5：添加 PDF 渲染后端
- 扩展点：实现 `PrinterBackend` trait

### 日志系统

- v0.5：添加结构化日志
- 工具：`tracing` crate

## 性能优化

### 编译优化

```toml
[profile.release]
opt-level = "z"     # 优化体积
lto = true          # 链接时优化
codegen-units = 1   # 单代码生成单元
strip = true        # 移除符号
```

### 运行时优化

- 异步 I/O（tokio）
- 延迟初始化（后端）
- 配置缓存（内存）

## 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tspl_generator() {
        let generator = TSPLGenerator::new();
        let tspl = generator.generate_qsl_card("BG7XXX", 1, 10);
        assert!(tspl.contains("BG7XXX"));
    }
}
```

### 集成测试

位于 `tests/integration/`

### 手动测试

- 跨平台测试（Windows, macOS, Linux）
- 实际打印机测试
- UI 功能测试

## 依赖管理

### 核心依赖

- `tauri = "2"` - 桌面应用框架
- `serde = "1.0"` - 序列化
- `toml = "0.8"` - TOML 解析
- `tokio = "1.0"` - 异步运行时
- `anyhow = "1.0"` - 错误处理
- `thiserror = "1.0"` - 错误定义
- `uuid = "1.0"` - UUID 生成
- `chrono = "0.4"` - 日期时间
- `dirs = "5.0"` - 系统目录

### 平台依赖

- Windows: `windows = "0.58"`
- Unix: 无（使用系统命令）

## 安全考虑

### 文件系统安全

- 配置文件使用用户目录
- 限制文件访问权限
- 验证文件路径

### 打印安全

- 验证打印机名称
- 限制打印数据大小
- 错误处理和恢复

## 参考文档

- [提案文档](openspec/changes/migrate-to-rust-tauri/proposal.md)
- [设计文档](openspec/changes/migrate-to-rust-tauri/design.md)
- [任务清单](openspec/changes/migrate-to-rust-tauri/tasks.md)
- [Tauri 官方文档](https://v2.tauri.app/)
