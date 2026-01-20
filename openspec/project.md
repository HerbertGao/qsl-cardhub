# 项目上下文

## 目的

QSL-CardHub 是为业余无线电爱好者设计的 QSL 卡片分卡系统的**输入端**项目（Rust 版本）。本项目是 Python 版的跨平台重写，旨在提供更好的性能、更小的可执行文件体积和更现代化的用户体验。

**核心目标：**
- 提供极简的桌面界面，快速输入呼号、序列号、数量等信息
- 自动生成呼号的条形码（Code128）
- 通过 Deli DL-888C 标签打印机，一键打印自定义标签/面单（76mm × 130mm）
- 实现"输入 → 一键打印 → 输出稳定、布局一致、无需手工排版"的工作流
- 支持 Windows、macOS、Linux 三大平台，无需 Python 运行时

**技术方案特点：**
- 采用 Rust + Tauri 构建跨平台桌面应用
- 使用 TSPL 打印指令直接发送到系统打印队列
- 前端使用 Vue 3 + Element Plus，提供现代化用户界面
- 编译为原生可执行文件，无需运行时依赖

## 技术栈

### 核心技术
- **Rust**：主要开发语言（后端逻辑、打印服务、配置管理）
- **Tauri**：跨平台桌面应用框架（替代 Python Eel）
- **Vue 3**：前端框架
- **Element Plus**：UI 组件库
- **Vite**：前端构建工具

### 关键库（Rust 依赖）
- **tauri**：桌面应用框架
- **serde**、**serde_json**：JSON 序列化/反序列化
- **tokio**：异步运行时
- **printpdf** 或 **cups-rs**：跨平台打印支持
- **uuid**：配置文件唯一标识符生成

### 前端依赖
- **vue** ^3.4.0
- **element-plus** ^2.5.0
- **@element-plus/icons-vue** ^2.3.0
- **@vitejs/plugin-vue**
- **vite**

### 目标平台
- **Windows**：通过 Windows Print API（Win32）
- **macOS/Linux**：通过 CUPS 或 TCP/9100 直连
- **统一接口**：Rust 封装跨平台打印逻辑

### 开发工具
- RustRover / VS Code（Rust 开发）
- Git 版本控制
- OpenSpec 规范管理
- Cargo（Rust 包管理）
- npm/pnpm（前端包管理）

## 项目约定

### 代码风格

#### Rust 代码
- **遵循 Rust 官方代码风格**（使用 `rustfmt`）
- **命名约定：**
  - 类型名：PascalCase（如 `PrinterManager`）
  - 函数/变量名：snake_case（如 `send_raw_print`）
  - 常量：SCREAMING_SNAKE_CASE（如 `DEFAULT_DPI`）
- **文档注释：** 使用中文编写，采用 Rust doc comment 风格（`///` 和 `//!`）
- **注释：** 代码注释使用中文，仅在逻辑复杂处添加
- **错误处理：** 使用 `Result<T, E>` 和 `anyhow`/`thiserror` 进行错误处理

#### 前端代码
- **遵循 Vue 3 Composition API 风格**
- **命名约定：**
  - 组件名：PascalCase（如 `PrintView.vue`）
  - 变量/函数：camelCase（如 `handlePrint`）
  - 常量：UPPER_SNAKE_CASE
- **使用 ESLint + Prettier** 格式化代码
- **注释：** 使用中文，简洁明了

### 架构模式

- **分层架构：**
  - **前端层（Vue 3 + Element Plus）**：用户界面、输入验证、状态管理
  - **Tauri 桥接层**：前后端通信（Tauri Commands）
  - **业务逻辑层（Rust）**：TSPL 指令生成、字段校验、布局计算
  - **打印服务层（Rust）**：跨平台打印抽象、打印机枚举、错误处理
  - **配置管理层（Rust）**：配置文件读写、序列化/反序列化

- **模块划分（Rust src/ 目录）：**
  - `main.rs`：Tauri 应用入口
  - `commands/`：Tauri 命令（前端调用的 API）
  - `printer/`：打印服务（TSPL 生成、打印机管理）
    - `tspl.rs`：TSPL 指令生成器
    - `manager.rs`：打印机管理（枚举、发送指令）
    - `platform/`：平台特定实现（Windows、CUPS、Mock）
  - `config/`：配置管理（JSON 读写、Profile 管理）
  - `models/`：数据模型（Profile、PrintRequest 等）
  - `utils/`：工具函数（字符转义、日志记录）

- **前端模块划分（web/src/ 目录）：**
  - `App.vue`：根组件
  - `main.js`：应用入口
  - `views/`：页面视图
    - `PrintView.vue`：打印页面
    - `ConfigView.vue`：配置管理页面
    - `AboutView.vue`：关于页面
    - `LogView.vue`：日志页面

### 测试策略

- **v1 阶段：** 手动测试为主，重点验证：
  - 打印输出正确性（条码可扫描）
  - 布局稳定性（连续打印不偏移）
  - 错误处理（打印机离线、输入为空）
  - 跨平台兼容性（Windows、macOS、Linux）

- **v2 规划：**
  - 单元测试：使用 `#[cfg(test)]` 和 `cargo test` 覆盖核心逻辑
  - 集成测试：测试 Tauri Commands 和打印流程
  - 端到端测试：使用 Tauri 测试框架

### Git 工作流

- **分支策略：**
  - `master`：主分支（稳定版本）
  - `develop`：开发分支
  - `feature/*`：新功能分支
  - `fix/*`：Bug 修复分支

- **提交约定：**
  - 使用中文提交信息
  - 格式：`类型: 简短描述`
  - 类型：`feat`（新功能）、`fix`（修复）、`docs`（文档）、`refactor`（重构）、`test`（测试）、`chore`（构建/工具）
  - 示例：`feat: 添加校准页打印功能`

## 领域上下文

### 业余无线电（Amateur Radio）
- **呼号（CALLSIGN）：** 每个业余无线电爱好者的唯一标识，如 `BG7XYZ`、`BD1ABC`
- **QSL 卡片：** 用于确认双向通联的明信片或标签，记录通联日期、频率、信号报告等
- **分卡系统：** 批量处理和分发 QSL 卡片的系统，本项目为其输入端

### TSPL 打印协议
- **TSPL（TSC Printer Language）：** 热敏打印机命令语言，类似 ZPL/EPL
- **坐标系统：** 基于 DPI（dots per inch），203dpi 时 1mm ≈ 8 dots
- **关键指令：**
  - `SIZE`：定义标签尺寸
  - `TEXT`：打印文本
  - `BARCODE`：打印条形码
  - `BOX`/`BAR`：绘制边框/线条
  - `PRINT`：触发打印

### 打印机特性（Deli DL-888C）
- **分辨率：** 203 dpi（通过校准页验证）
- **打印方式：** 热敏/热转印
- **纸张规格：** 76mm × 130mm 一联纸（快递面单尺寸）
- **连接方式：** USB（系统识别为打印队列）

## 重要约束

### 技术约束
- **跨平台：** 必须同时支持 Windows、macOS、Linux
- **RAW 打印模式：** 必须绕过图形驱动，直接发送 TSPL 指令
- **字符安全：** TSPL 字符串需转义双引号、换行符等特殊字符
- **ASCII 优先：** v1 优先保证英文/数字呼号正确显示，中文支持延后到 v2
- **无运行时依赖：** 编译为单一可执行文件，无需 Python/Node.js 运行时

### 业务约束
- **布局固定：** v1 不支持自定义模板或拖拽排版
- **静默打印：** 不弹出系统打印对话框，直接发送到打印队列
- **输入校验：** CALLSIGN 和 SERIAL 为必填项，QTY 为必填项

### 硬件约束
- **纸张尺寸：** 固定为 76mm × 130mm，超出范围会裁切
- **打印偏移：** 需提供校准页功能，允许调整 DIRECTION、GAP、REFERENCE 参数
- **走纸稳定性：** 连续打印时需保证标签不偏移（通过正确的 GAP 设置）

## 外部依赖

### 系统依赖

#### Windows
- **Windows Print Spooler**：系统打印队列服务
- **Win32 API**：通过 `windows-rs` crate 访问

#### macOS/Linux
- **CUPS（Common Unix Printing System）**：通过 `cups-rs` 或命令行调用
- **或 TCP/9100**：直连打印机（无需 CUPS）

### Rust Crate 依赖（Cargo.toml）
```toml
[dependencies]
tauri = "2.x"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
# 平台特定打印支持（根据平台条件编译）
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = ["Win32_Graphics_Printing"] }
[target.'cfg(unix)'.dependencies]
cups = "0.4"  # 或使用 printpdf
```

### 前端依赖（package.json）
```json
{
  "dependencies": {
    "vue": "^3.4.0",
    "element-plus": "^2.5.0",
    "@element-plus/icons-vue": "^2.3.0"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.0.0",
    "vite": "^5.0.0"
  }
}
```

### 配置文件
- **config.json**：存储配置列表（多个 Profile）
- **profiles/{uuid}.json**：单个配置详情（打印机名称、DPI、GAP、DIRECTION 等）

---

## 版本规划

### v0.1（当前目标 - 基础架构）
- ✅ 前端框架搭建（Vue 3 + Element Plus + Vite）
- 🔄 Tauri 集成（前后端桥接）
- 🔄 配置管理模块（Profile CRUD）
- 🔄 TSPL 指令生成器
- 🔄 跨平台打印抽象层

### v0.5（功能完善）
- QSL 卡片打印功能（呼号、序列号、数量、条形码）
- 校准页打印功能
- 序列号自动管理
- 基础设置：打印机选择、方向、GAP

### v1.0（稳定发布）
- 完整的错误处理和用户提示
- 跨平台测试通过（Windows、macOS、Linux）
- 打包为可执行文件（无运行时依赖）
- 用户文档和安装指南

### v2.0（后续扩展）
- 中文字体支持
- 多模板管理
- 批量打印（序列号自增）
- LAN 直连打印（TCP 9100）
- 日志查看功能

## 迁移说明

本项目是从 Python 版 QSL-CardHub 迁移而来。迁移的主要目标是：

1. **性能提升**：Rust 的原生性能，启动速度更快
2. **体积优化**：编译为单一可执行文件，无需 Python 运行时（~10MB vs ~100MB+）
3. **跨平台一致性**：Tauri 提供统一的 API，减少平台差异
4. **现代化技术栈**：保留 Vue 3 + Element Plus 前端，后端切换到 Rust

**参考 Python 版文档：**
- OpenSpec 规范：`/Users/herbertgao/PycharmProjects/QSL-CardHub/openspec/`
- API 文档：`/Users/herbertgao/PycharmProjects/QSL-CardHub/docs/API_REFERENCE.md`
- 错误代码：`/Users/herbertgao/PycharmProjects/QSL-CardHub/docs/ERROR_CODES.md`
