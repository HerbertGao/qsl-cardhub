# 设计文档：Rust + Tauri 架构迁移

## 架构概览

本文档描述从 Python + Eel 到 Rust + Tauri 的架构迁移设计。

### 整体架构

```
┌────────────────────────────────────────────────────────┐
│                    前端层 (Web)                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │ Vue 3 + Element Plus + Vite                      │  │
│  │ - PrintView.vue (打印页面)                        │  │
│  │ - ConfigView.vue (配置管理)                       │  │
│  │ - AboutView.vue (关于页面)                        │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────┬───────────────────────────────────┘
                     │ Tauri IPC (JSON-RPC over WebSocket)
                     │ invoke("command_name", { args })
┌────────────────────▼───────────────────────────────────┐
│              Tauri Commands 层 (Rust)                   │
│  ┌──────────────────────────────────────────────────┐  │
│  │ Commands (暴露给前端的 API)                       │  │
│  │ - get_initial_data()                             │  │
│  │ - get_profiles(), create_profile(), ...          │  │
│  │ - get_printers(), print_qsl(), ...               │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────┬───────────────────────────────────┘
                     │ 调用业务逻辑
┌────────────────────▼───────────────────────────────────┐
│                业务逻辑层 (Rust)                        │
│  ┌──────────────┬──────────────┬───────────────────┐   │
│  │ProfileManager│PrinterManager│ TSPLGenerator     │   │
│  │- CRUD 操作   │- 枚举打印机  │ - 生成 TSPL 指令  │   │
│  │- 持久化     │- 发送打印    │ - QSL 卡片布局    │   │
│  │- 验证       │- 后端选择    │ - 校准页布局      │   │
│  └──────────────┴──────────────┴───────────────────┘   │
└────────────────────┬───────────────────────────────────┘
                     │ 平台抽象接口
┌────────────────────▼───────────────────────────────────┐
│              平台抽象层 (Rust Traits)                   │
│  ┌──────────────────────────────────────────────────┐  │
│  │ PrinterBackend trait                             │  │
│  │ - list_printers()                                │  │
│  │ - send_raw(printer_name, data)                   │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────┬───────────────────────────────────┘
                     │ 平台特定实现
┌────────────────────▼───────────────────────────────────┐
│              平台实现层 (条件编译)                       │
│  ┌────────────┬────────────┬─────────────────────┐     │
│  │  Windows   │   CUPS     │       Mock          │     │
│  │  Backend   │  Backend   │      Backend        │     │
│  │ (Win32 API)│(lp 命令)   │ (文件输出)          │     │
│  └────────────┴────────────┴─────────────────────┘     │
└────────────────────────────────────────────────────────┘
```

## 模块设计

### 1. Tauri 应用入口 (`src/main.rs`)

**职责：**
- 初始化 Tauri 应用
- 注册所有 Commands
- 管理应用状态（State）
- 处理窗口生命周期

**核心代码结构：**
```rust
use tauri::{Manager, State};

// 应用状态
struct AppState {
    profile_manager: Arc<Mutex<ProfileManager>>,
    printer_manager: Arc<Mutex<PrinterManager>>,
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // 初始化应用状态
            let profile_manager = ProfileManager::new()?;
            let printer_manager = PrinterManager::new()?;

            app.manage(AppState {
                profile_manager: Arc::new(Mutex::new(profile_manager)),
                printer_manager: Arc::new(Mutex::new(printer_manager)),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_initial_data,
            get_profiles,
            create_profile,
            // ... 其他 commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 2. Tauri Commands (`src/commands/mod.rs`)

**职责：**
- 定义前端可调用的 API
- 参数验证和错误处理
- 将前端请求转发到业务逻辑层

**API 设计原则：**
- 返回统一的 `Result<T, String>` 格式
- 错误信息使用中文，用户友好
- 参数使用强类型（serde 自动反序列化）

**关键 Commands：**

```rust
// 配置管理 Commands
#[tauri::command]
async fn get_profiles(state: State<'_, AppState>) -> Result<Vec<Profile>, String>;

#[tauri::command]
async fn create_profile(
    name: String,
    printer_name: String,
    state: State<'_, AppState>
) -> Result<Profile, String>;

#[tauri::command]
async fn update_profile(
    id: String,
    profile: Profile,
    state: State<'_, AppState>
) -> Result<(), String>;

#[tauri::command]
async fn delete_profile(
    id: String,
    state: State<'_, AppState>
) -> Result<(), String>;

// 打印 Commands
#[tauri::command]
async fn get_printers(state: State<'_, AppState>) -> Result<Vec<String>, String>;

#[tauri::command]
async fn print_qsl(
    profile_id: String,
    callsign: String,
    serial: u32,
    qty: u32,
    state: State<'_, AppState>
) -> Result<(), String>;

#[tauri::command]
async fn print_calibration(
    profile_id: String,
    state: State<'_, AppState>
) -> Result<(), String>;

// 平台信息 Commands
#[tauri::command]
fn get_platform_info() -> PlatformInfo;
```

### 3. 配置管理模块 (`src/config/`)

#### 3.1 数据模型 (`src/config/models.rs`)

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,          // UUID
    pub name: String,
    pub platform: Platform,
    pub printer: PrinterConfig,
    pub paper: PaperSpec,
    pub template: Template,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Platform {
    pub os: String,      // "Windows" | "macOS" | "Linux"
    pub arch: String,    // "x86_64" | "arm64"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterConfig {
    pub model: String,   // "Deli DL-888C"
    pub name: String,    // 系统中的打印机名称
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperSpec {
    pub width: u32,      // mm
    pub height: u32,     // mm
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,    // "QSL Card v1"
    pub version: String, // "1.0"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileStore {
    pub profiles: Vec<Profile>,
    pub default_profile_id: Option<String>,
}
```

#### 3.2 配置管理器 (`src/config/profile_manager.rs`)

```rust
use anyhow::Result;
use std::path::PathBuf;
use std::fs;

pub struct ProfileManager {
    config_dir: PathBuf,      // ~/.config/qsl-cardhub/
    profiles_dir: PathBuf,    // ~/.config/qsl-cardhub/profiles/
    app_config: AppConfig,    // 全局配置
}

impl ProfileManager {
    pub fn new() -> Result<Self> {
        // 获取配置目录
        // 加载 config.toml
        // 扫描 profiles/ 目录
    }

    pub fn get_all(&self) -> Result<Vec<Profile>> {
        // 扫描 profiles/*.toml 文件
        // 解析每个文件为 Profile
    }

    pub fn get_by_id(&self, id: &str) -> Result<Option<Profile>> {
        // 读取 profiles/{id}.toml
        // 使用 toml::from_str() 解析
    }

    pub fn create(&mut self, name: String, printer_name: String) -> Result<Profile> {
        // 创建新配置
        // 保存到 profiles/{uuid}.toml
    }

    pub fn update(&mut self, id: &str, profile: Profile) -> Result<()> {
        // 更新配置
        // 使用 toml::to_string_pretty() 序列化
        // 写入 profiles/{id}.toml
    }

    pub fn delete(&mut self, id: &str) -> Result<()> {
        // 删除 profiles/{id}.toml 文件
    }

    pub fn set_default(&mut self, id: &str) -> Result<()> {
        // 更新 app_config.default_profile_id
        // 保存到 config.toml
    }

    fn save_app_config(&self) -> Result<()> {
        // 持久化到 config.toml
    }
}
```

### 4. 打印模块 (`src/printer/`)

#### 4.1 TSPL 生成器 (`src/printer/tspl.rs`)

```rust
pub struct TSPLGenerator {
    dpi: u32,  // 默认 203
    // v0.5: 添加 template_config: TemplateConfig
}

impl TSPLGenerator {
    pub fn generate_qsl_card(
        &self,
        callsign: &str,
        serial: u32,
        qty: u32,
    ) -> String {
        let mut tspl = String::new();

        // v0.1: 硬编码布局参数
        // SIZE 76mm x 130mm
        tspl.push_str("SIZE 76 mm, 130 mm\n");
        tspl.push_str("GAP 2 mm, 0 mm\n");
        tspl.push_str("DIRECTION 0\n");
        tspl.push_str("CLS\n");

        // 呼号（大字号，居中）
        tspl.push_str(&format!(
            "TEXT 304,80,\"5\",0,3,3,\"{}\"\n",
            callsign
        ));

        // 条形码（居中）
        tspl.push_str(&format!(
            "BARCODE 200,300,\"128\",120,1,0,3,3,\"{}\"\n",
            callsign
        ));

        // 序列号
        tspl.push_str(&format!(
            "TEXT 50,520,\"5\",0,2,2,\"SN: {:03}\"\n",
            serial
        ));

        // 数量
        tspl.push_str(&format!(
            "TEXT 50,720,\"5\",0,2,2,\"QTY: {}\"\n",
            qty
        ));

        tspl.push_str("PRINT 1\n");

        tspl
    }

    pub fn generate_calibration_page(&self) -> String {
        // 生成校准页 TSPL
    }

    // v0.5: 添加基于模板生成的方法
    // pub fn generate_from_template(
    //     &self,
    //     template: &TemplateConfig,
    //     callsign: &str,
    //     serial: u32,
    //     qty: u32,
    // ) -> String
}
```

#### 4.2 打印机后端抽象 (`src/printer/backend/mod.rs`)

```rust
use anyhow::Result;

pub trait PrinterBackend: Send + Sync {
    fn name(&self) -> &str;
    fn list_printers(&self) -> Result<Vec<String>>;
    fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<()>;
}
```

#### 4.3 Windows 后端 (`src/printer/backend/windows.rs`)

```rust
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Printing::*;

pub struct WindowsBackend;

impl PrinterBackend for WindowsBackend {
    fn name(&self) -> &str {
        "Windows"
    }

    fn list_printers(&self) -> Result<Vec<String>> {
        // 使用 EnumPrintersW 枚举打印机
    }

    fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<()> {
        // 使用 OpenPrinterW / StartDocPrinterW / WritePrinter
    }
}
```

#### 4.4 CUPS 后端 (`src/printer/backend/cups.rs`)

```rust
#[cfg(target_family = "unix")]
use std::process::Command;

pub struct CupsBackend;

impl PrinterBackend for CupsBackend {
    fn name(&self) -> &str {
        "CUPS"
    }

    fn list_printers(&self) -> Result<Vec<String>> {
        // 执行 lpstat -p 并解析输出
    }

    fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<()> {
        // 执行 lp -d <printer> -o raw
    }
}
```

#### 4.5 Mock 后端 (`src/printer/backend/mock.rs`)

```rust
use std::fs;
use std::path::PathBuf;

pub struct MockBackend {
    output_dir: PathBuf,
}

impl PrinterBackend for MockBackend {
    fn name(&self) -> &str {
        "Mock"
    }

    fn list_printers(&self) -> Result<Vec<String>> {
        Ok(vec!["Mock Printer".to_string()])
    }

    fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<()> {
        // 保存到 output/print_YYYYMMDD_HHMMSS.tspl
    }
}
```

#### 4.6 打印机管理器 (`src/printer/manager.rs`)

```rust
pub struct PrinterManager {
    backends: Vec<Box<dyn PrinterBackend>>,
    tspl_generator: TSPLGenerator,
}

impl PrinterManager {
    pub fn new() -> Result<Self> {
        let mut backends: Vec<Box<dyn PrinterBackend>> = vec![];

        #[cfg(target_os = "windows")]
        backends.push(Box::new(WindowsBackend));

        #[cfg(target_family = "unix")]
        backends.push(Box::new(CupsBackend));

        backends.push(Box::new(MockBackend::new()?));

        Ok(Self {
            backends,
            tspl_generator: TSPLGenerator::new(),
        })
    }

    pub fn list_printers(&self) -> Result<Vec<String>> {
        // 聚合所有后端的打印机列表
    }

    pub fn print_qsl(
        &self,
        printer_name: &str,
        callsign: &str,
        serial: u32,
        qty: u32,
    ) -> Result<()> {
        let tspl = self.tspl_generator.generate_qsl_card(callsign, serial, qty);
        self.send_raw(printer_name, tspl.as_bytes())
    }

    fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<()> {
        // 遍历后端，找到支持该打印机的后端
        for backend in &self.backends {
            if backend.list_printers()?.contains(&printer_name.to_string()) {
                return backend.send_raw(printer_name, data);
            }
        }
        Err(anyhow::anyhow!("打印机未找到: {}", printer_name))
    }
}
```

### 5. 平台检测模块 (`src/utils/platform.rs`)

```rust
#[derive(Debug, Serialize)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
}

pub fn detect_platform() -> PlatformInfo {
    let os = if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "Unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        std::env::consts::ARCH
    };

    PlatformInfo {
        os: os.to_string(),
        arch: arch.to_string(),
    }
}
```

## 前端集成

### API 调用适配

**Python + Eel 版本：**
```javascript
const result = await window.eel.get_profiles()();
```

**Rust + Tauri 版本：**
```javascript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('get_profiles');
```

### 错误处理

Tauri 会自动将 Rust 的 `Result<T, String>` 转换为 Promise：

```javascript
try {
    const profiles = await invoke('get_profiles');
    // 成功
} catch (error) {
    // error 是 String 类型（Rust 返回的错误消息）
    ElMessage.error(error);
}
```

## 数据持久化

### 配置文件位置

- **开发环境**：项目根目录 `config/`
- **生产环境**：
  - Windows: `%APPDATA%/qsl-cardhub/`
  - macOS: `~/Library/Application Support/qsl-cardhub/`
  - Linux: `~/.config/qsl-cardhub/`

使用 Tauri 的 `app_data_dir()` API 获取路径。

### 配置文件结构

```
~/.config/qsl-cardhub/
├── config.toml           # 全局配置
└── profiles/
    ├── uuid-1.toml       # 配置 1
    ├── uuid-2.toml       # 配置 2
    └── uuid-3.toml       # 配置 3
```

### TOML 格式

#### 全局配置 (config.toml)

```toml
# qsl-cardhub 全局配置

default_profile_id = "550e8400-e29b-41d4-a716-446655440000"

[window_state]
width = 1200
height = 800
x = 100
y = 100
```

#### 配置文件 (profiles/uuid.toml)

```toml
# qsl-cardhub 打印配置

id = "550e8400-e29b-41d4-a716-446655440000"
name = "默认配置"
created_at = "2026-01-20T10:00:00Z"
updated_at = "2026-01-20T10:00:00Z"

[platform]
os = "Windows"
arch = "x86_64"

[printer]
model = "Deli DL-888C"
name = "Deli DL-888C"

[paper]
width = 76
height = 130

[template]
name = "QSL Card v1"
version = "1.0"
```

## 构建和打包

### 开发模式

```bash
# 启动 Tauri 开发服务器
cargo tauri dev
```

### 生产构建

```bash
# 构建 Web 前端
cd web && npm run build

# 构建 Tauri 应用
cargo tauri build
```

### 打包产物

- **Windows**: `.exe` 和 `.msi` 安装包
- **macOS**: `.app` 和 `.dmg` 镜像
- **Linux**: `.AppImage` 和 `.deb` 包

## 兼容性考虑

### Python 版本配置不兼容

**重要提示**：Rust 版使用 TOML 格式存储配置，与 Python 版的 JSON 格式不兼容。用户需要在 Rust 版中重新创建配置，不支持从 Python 版自动迁移。

**理由**：
- TOML 格式更易读，支持注释
- 每个配置独立存储，互不干扰
- 简化配置管理逻辑

### API 接口一致性

Tauri Commands 的命名和参数尽量与 Python 版的 Eel API 保持一致，减少前端代码变更。

## 性能优化

1. **异步 I/O**：所有文件操作使用 `tokio::fs`
2. **状态共享**：使用 `Arc<Mutex<T>>` 避免频繁克隆
3. **条件编译**：平台特定代码使用 `#[cfg(target_os = "...")]`
4. **懒加载**：打印机列表仅在需要时枚举

## 测试策略

1. **单元测试**：业务逻辑（TSPL 生成、配置管理）
2. **集成测试**：Tauri Commands（模拟前端调用）
3. **手动测试**：跨平台打印功能
4. **性能测试**：启动时间、打印速度

## 模板配置化设计（v0.5）

### 模板配置文件结构

```toml
# templates/qsl-card-v1.toml

[paper]
width_mm = 76
height_mm = 130
gap_mm = 2
direction = 0

[callsign]
x = 304
y = 80
font = "5"
rotation = 0
x_scale = 3
y_scale = 3
align = "center"

[barcode]
x = 200
y = 300
type = "128"
height = 120
human_readable = 1
rotation = 0
narrow_bar = 3
wide_bar = 3

[serial]
x = 50
y = 520
font = "5"
rotation = 0
x_scale = 2
y_scale = 2
prefix = "SN: "
format = "{:03}"

[quantity]
x = 50
y = 720
font = "5"
rotation = 0
x_scale = 2
y_scale = 2
prefix = "QTY: "
```

### 模板加载器

```rust
pub struct TemplateConfig {
    pub paper: PaperConfig,
    pub callsign: TextConfig,
    pub barcode: BarcodeConfig,
    pub serial: TextConfig,
    pub quantity: TextConfig,
}

impl TemplateConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
    }

    pub fn default_qsl_card() -> Self {
        // 内置默认值
    }
}
```

## 待解决问题

1. **CUPS 依赖**：是否使用 `lp` 命令还是 `cups-rs` crate？
   - **建议**：使用 `lp` 命令，避免编译依赖

2. **错误消息国际化**：是否需要支持英文错误消息？
   - **建议**：v0.1 仅支持中文，v2.0 添加国际化

3. **模板配置化优先级**：v0.1 使用硬编码，v0.5 实现配置化
   - **理由**：简化初始迁移，保证功能一致性

## 参考资料

- Tauri 官方文档：https://v2.tauri.app/
- windows-rs 文档：https://microsoft.github.io/windows-docs-rs/
- serde_json 文档：https://docs.rs/serde_json/
- chrono 文档：https://docs.rs/chrono/
