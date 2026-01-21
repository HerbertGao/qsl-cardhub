# 平台支持规范（Rust 版）

## 目的

本规范定义了 qsl-cardhub Rust 版本的跨平台兼容性需求。使用 Rust 的条件编译和 Tauri 的跨平台 API 实现 Windows、macOS、Linux 三大平台的支持。本规范在 Python 版的基础上，简化依赖管理，统一平台抽象层。

## 修改需求

### 需求:跨平台兼容性

**原需求**：系统必须支持 Windows、macOS 和 Linux 三个主要操作系统平台。（Python 版）

**修改原因**：使用 Rust 的条件编译和 Tauri 的跨平台 API。

**新需求**：系统必须使用 Rust 的 `cfg` 属性实现平台特定代码的条件编译。

#### 场景:平台检测

- **当** 系统启动时
- **那么** 应使用 Rust 的编译时检查确定平台：
  - `cfg!(target_os = "windows")` → "Windows"
  - `cfg!(target_os = "macos")` → "macOS"
  - `cfg!(target_os = "linux")` → "Linux"
- **并且** 应检测 CPU 架构：
  - `cfg!(target_arch = "x86_64")` → "x86_64"
  - `cfg!(target_arch = "aarch64")` → "arm64"
- **并且** 返回 `PlatformInfo` struct

#### 场景:Windows 支持

- **当** 系统编译为 Windows 目标
- **那么** 应使用 `#[cfg(target_os = "windows")]` 编译 WindowsBackend
- **并且** 应使用 `windows-rs` crate 调用 Win32 API
- **并且** 所有功能应正常工作

#### 场景:macOS 支持

- **当** 系统编译为 macOS 目标
- **那么** 应使用 `#[cfg(target_os = "macos")]` 编译 CupsBackend
- **并且** 应使用 `lp` 命令行工具（无需额外依赖）
- **并且** 所有功能应正常工作

#### 场景:Linux 支持

- **当** 系统编译为 Linux 目标
- **那么** 应使用 `#[cfg(target_os = "linux")]` 编译 CupsBackend
- **并且** 应使用 `lp` 命令行工具（无需额外依赖）
- **并且** 所有功能应正常工作

### 需求:打印机后端抽象

**原需求**：系统必须支持多种打印机后端，包括测试和开发用途的后端。（Python 版）

**修改原因**：使用 Rust trait 和条件编译实现后端抽象，v0.1 不包含 PDF 后端。

**新需求**：系统必须使用 PrinterBackend trait 统一打印接口，支持 Windows、CUPS、Mock 三种后端。

#### 场景:后端选择

- **当** 系统初始化 PrinterManager
- **那么** 应根据编译目标选择后端：
  - Windows: `WindowsBackend`
  - macOS/Linux: `CupsBackend`
  - 所有平台: `MockBackend`（开发测试）
- **并且** 使用条件编译确保每个平台只编译必要的后端

#### 场景:后端共存

- **当** 多个后端可用（如 Windows + Mock）
- **那么** 每个后端应独立工作
- **并且** `list_printers()` 应聚合所有后端的打印机列表
- **并且** 用户可以在不同后端的打印机之间切换

### 需求:依赖管理

**原需求**：系统必须为不同平台提供相应的依赖配置。（Python 版）

**修改原因**：使用 Cargo.toml 的平台特定依赖，简化依赖管理。

**新需求**：系统必须使用 Cargo 的 `[target.'cfg(...)'.dependencies]` 语法管理平台依赖。

#### 场景:核心依赖

- **当** 编译应用时
- **那么** 应包含以下跨平台依赖：
  ```toml
  [dependencies]
  tauri = "2"
  serde = { version = "1.0", features = ["derive"] }
  serde_json = "1.0"
  anyhow = "1.0"
  uuid = { version = "1.0", features = ["v4", "serde"] }
  chrono = { version = "0.4", features = ["serde"] }
  ```

#### 场景:Windows 依赖

- **当** 编译 Windows 目标
- **那么** 应包含以下依赖：
  ```toml
  [target.'cfg(windows)'.dependencies]
  windows = { version = "0.58", features = [
      "Win32_Graphics_Printing",
      "Win32_Graphics_Gdi",
      "Win32_Foundation"
  ] }
  ```
- **并且** 仅在 Windows 平台编译

#### 场景:Unix 依赖

- **当** 编译 macOS/Linux 目标
- **那么** 不需要额外的 Rust 依赖（使用命令行工具）
- **并且** 系统应预装 CUPS（lp 命令）

### 需求:文件路径兼容性

**原需求**：系统必须处理不同平台的文件路径差异。（Python 版）

**修改原因**：使用 Tauri 的 `app_data_dir()` API 和 Rust 的 `std::path::PathBuf`。

**新需求**：系统必须使用跨平台的路径 API 处理文件操作。

#### 场景:配置文件路径

- **当** 系统访问 profiles.json
- **那么** 应使用 `tauri::api::path::app_data_dir()` 获取应用数据目录
- **并且** 路径应为：
  - Windows: `%APPDATA%/qsl-cardhub/profiles.json`
  - macOS: `~/Library/Application Support/qsl-cardhub/profiles.json`
  - Linux: `~/.config/qsl-cardhub/profiles.json`
- **并且** 使用 `PathBuf::join()` 拼接路径

#### 场景:输出文件路径

- **当** Mock 后端保存 TSPL 文件
- **那么** 应创建 `output/` 目录（相对于当前工作目录）
- **并且** 使用 `std::fs::create_dir_all()` 创建目录
- **并且** 使用 `PathBuf` 处理路径

### 需求:字符编码兼容性

**原需求**：系统必须正确处理不同平台的字符编码。（Python 版）

**修改原因**：Rust 默认使用 UTF-8，简化编码处理。

**新需求**：系统必须使用 UTF-8 编码处理所有文本数据。

#### 场景:配置文件编码

- **当** 保存配置文件
- **那么** 应使用 `std::fs::write()` 写入 UTF-8 字符串
- **并且** serde_json 默认输出 UTF-8
- **并且** 中文字符应正确保存

#### 场景:TSPL 命令编码

- **当** 发送 TSPL 命令到打印机
- **那么** 应使用 `.as_bytes()` 转换为 UTF-8 字节
- **并且** 打印机应正确识别 UTF-8 编码的呼号

### 需求:权限要求

**原需求**：系统必须明确不同平台的权限要求。（Python 版）

**新需求**：系统必须在不同平台上以普通用户权限运行，无需提升权限。

#### 场景:Windows 权限

- **当** 在 Windows 上运行
- **那么** 不需要管理员权限
- **并且** 需要访问打印机的权限（普通用户默认拥有）

#### 场景:macOS/Linux 权限

- **当** 在 macOS/Linux 上运行
- **那么** 不需要 root 权限
- **并且** 用户应能执行 `lp` 命令（默认可用）
- **并且** 如果 CUPS 未安装，应提示用户安装

## 新增需求

### 需求:条件编译

系统必须使用 Rust 的条件编译特性优化平台支持。

#### 场景:平台特定代码

- **当** 编译应用时
- **那么** 应使用 `#[cfg(target_os = "...")]` 属性标记平台代码
- **并且** 仅编译当前平台所需的代码
- **并且** 减少最终可执行文件体积

#### 场景:测试平台检测

- **当** 运行单元测试时
- **那么** 应测试平台检测逻辑
- **并且** 应模拟不同平台的行为（通过条件编译）

### 需求:Tauri 平台 API

系统必须使用 Tauri 提供的跨平台 API。

#### 场景:应用数据目录

- **当** 获取应用数据目录
- **那么** 应使用 `tauri::api::path::app_data_dir()`
- **并且** API 应自动处理平台差异
- **并且** 返回 `PathBuf` 类型

#### 场景:对话框

- **当** 显示错误对话框（如初始化失败）
- **那么** 应使用 `tauri::api::dialog::message()`
- **并且** 对话框应符合当前平台的 UI 风格

### 需求:Mock 后端跨平台

系统必须确保 Mock 后端在所有平台上正常工作。

#### 场景:Mock 后端输出

- **当** Mock 后端保存 TSPL 文件
- **那么** 应在所有平台上使用相同的逻辑
- **并且** 文件路径应使用 `PathBuf` 处理
- **并且** 输出目录应为 `./output/`

#### 场景:文件名生成

- **当** 生成 TSPL 文件名
- **那么** 应使用 `chrono::Local::now()` 获取时间
- **并且** 文件名格式应为 `print_YYYYMMDD_HHMMSS.tspl`
- **并且** 所有平台生成的文件名格式一致
