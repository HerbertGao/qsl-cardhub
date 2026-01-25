# 项目上下文

## 目的

qsl-cardhub 是为业余无线电爱好者设计的 QSL 卡片管理工具（Rust 版本）。本项目是 Python 版的跨平台重写，旨在提供更好的性能、更小的可执行文件体积和更现代化的用户体验。

**核心功能：**
- **卡片管理**：卡片录入、编辑、分发、退卡、查询和筛选
- **项目管理**：按转卡项目组织卡片，支持项目统计
- **标签打印**：支持 TSPL 热敏打印机（如 Deli DL-888C），打印 QSL 卡片标签
- **顺丰快递集成**：面单打印、订单创建、确认、查询和管理
- **QRZ 集成**：支持 QRZ.cn、QRZ.com、QRZ.herbertgao.me 地址查询
- **配置管理**：打印配置（Profile）和模板配置管理，支持导入导出
- **自动更新**：应用内自动检查更新，支持手动更新
- **数据管理**：本地 SQLite 数据库，支持数据导出导入

**技术方案特点：**
- 采用 Rust + Tauri 2 构建跨平台桌面应用
- 使用 TSPL 打印指令直接发送到系统打印队列
- 前端使用 Vue 3 + TypeScript + Element Plus，提供现代化用户界面
- 编译为原生可执行文件，无需运行时依赖
- 支持 Windows（x64、ARM64）和 macOS（Intel、Apple Silicon）

## 技术栈

### 核心技术
- **Rust**：主要开发语言（后端逻辑、打印服务、配置管理）
- **Tauri**：跨平台桌面应用框架（替代 Python Eel）
- **Vue 3**：前端框架
- **Element Plus**：UI 组件库
- **Vite**：前端构建工具

### 关键库（Rust 依赖）
- **tauri** 2.x：桌面应用框架
- **serde**、**serde_json**：JSON 序列化/反序列化
- **toml**：TOML 配置文件解析
- **tokio**：异步运行时
- **rusqlite**：SQLite 数据库操作
- **rusttype**、**ab_glyph**：字体渲染和文本测量
- **barcoders**：Code128 条形码生成
- **image**、**imageproc**：图像处理和 PDF 渲染
- **uuid**：UUID 生成
- **chrono**：日期时间处理
- **reqwest**：HTTP 客户端（QRZ 集成、顺丰 API）
- **keyring**：系统钥匙串集成（凭据加密存储）
- **log**、**env_logger**：日志系统
- **windows-rs**（Windows）：Win32 API 打印支持
- **cups-sys**（macOS/Linux）：CUPS 打印支持

### 前端依赖
- **vue** ^3.4.0：前端框架
- **typescript**：类型系统
- **element-plus** ^2.5.0：UI 组件库
- **@element-plus/icons-vue** ^2.3.0：图标库
- **@tauri-apps/api** ^2.x：Tauri API 绑定
- **@vitejs/plugin-vue**：Vue 插件
- **vite**：构建工具

### 目标平台
- **Windows x64**：通过 Windows Print API（Win32）
- **Windows ARM64**：通过 Windows Print API（Win32）
- **macOS Intel (x86_64)**：通过 CUPS 打印系统
- **macOS Apple Silicon (ARM64)**：通过 CUPS 打印系统
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

- **Tauri Commands 参数命名约定：**
  - **默认行为：** Tauri 2 会自动将 Rust 后端的 `snake_case` 参数名转换为前端的 `camelCase`
  - **Rust 后端：** 使用 `snake_case` 命名参数（如 `proxy_callsign`、`project_id`）
  - **前端调用：** 使用 `camelCase` 命名参数（如 `proxyCallsign`、`projectId`）
  - **类型定义：** 在 `web/src/types/tauri.ts` 中使用 `snake_case` 定义接口（与 Rust 保持一致），但前端调用时使用 `camelCase`
  - **示例：**
    ```rust
    // Rust 后端 (src/commands/cards.rs)
    #[tauri::command]
    pub async fn distribute_card_cmd(
        id: String,
        proxy_callsign: Option<String>,  // snake_case
    ) -> Result<Card, String> { ... }
    ```
    ```typescript
    // 前端类型定义 (web/src/types/tauri.ts)
    export interface DistributeCardParams {
      id: string
      proxy_callsign?: string | null  // snake_case（与 Rust 保持一致）
    }
    ```
    ```typescript
    // 前端调用 (web/src/views/CardManagementView.vue)
    await invoke('distribute_card_cmd', {
      id: data.id,
      proxyCallsign: data.proxy_callsign || null  // camelCase（Tauri 自动转换）
    })
    ```
  - **注意事项：**
    - 无需在 Rust 代码中添加 `rename_all` 配置，使用默认行为即可
    - 确保类型定义中包含所有参数，避免类型检查遗漏
    - 如果参数名不匹配，会导致参数无法传递到后端，造成功能失效

- **模块划分（Rust src/ 目录）：**
  - `main.rs`：Tauri 应用入口，应用状态初始化
  - `commands/`：Tauri 命令（前端调用的 API）
    - `projects.rs`：项目管理命令
    - `cards.rs`：卡片管理命令
    - `printer.rs`：打印相关命令
    - `profile.rs`：配置管理命令
    - `qrz_cn.rs`、`qrz_com.rs`、`qrz_herbertgao.rs`：QRZ 集成命令
    - `sf_express.rs`：顺丰快递集成命令
    - `security.rs`：凭据管理命令
    - `logger.rs`：日志管理命令
    - `platform.rs`：平台信息命令
    - `data_transfer.rs`：数据导出导入命令
    - `factory_reset.rs`：恢复出厂设置命令
  - `db/`：数据库操作
    - `sqlite.rs`：SQLite 数据库管理
    - `projects.rs`：项目 CRUD 操作
    - `cards.rs`：卡片 CRUD 操作和地址缓存
    - `models.rs`：数据模型定义
  - `printer/`：打印服务
    - `tspl.rs`：TSPL 指令生成器
    - `manager.rs`：打印机管理（枚举、发送指令）
    - `backend/`：打印后端抽象
      - `trait.rs`：PrinterBackend trait 定义
      - `pdf.rs`：PDF 虚拟打印机（预览）
      - `windows.rs`：Windows 打印后端
      - `cups.rs`：CUPS 打印后端（macOS/Linux）
      - `mock.rs`：Mock 打印后端（开发测试）
    - `font_loader.rs`：字体加载系统
    - `text_renderer.rs`：文本渲染系统
    - `barcode_renderer.rs`：条形码渲染系统
  - `config/`：配置管理
    - `profile_manager.rs`：Profile 管理
    - `template.rs`：模板配置管理
  - `qrz/`：QRZ 集成模块
    - `qrz_cn_client.rs`、`qrz_cn_parser.rs`：QRZ.cn 客户端和解析器
    - `qrz_com_client.rs`、`qrz_com_parser.rs`：QRZ.com 客户端和解析器
    - `qrz_herbertgao_client.rs`：QRZ.herbertgao.me 客户端
  - `sf_express/`：顺丰快递集成
    - `client.rs`：顺丰 API 客户端
    - `models.rs`：顺丰数据模型
  - `security/`：安全模块
    - `credentials.rs`：凭据加密存储
    - `keyring.rs`：系统钥匙串集成
    - `encryption.rs`：AES-256-GCM 加密实现
  - `logger/`：日志系统
    - `collector.rs`：日志收集器（环形缓冲区）
    - `models.rs`：日志数据模型
  - `utils/`：工具函数

- **前端模块划分（web/src/ 目录）：**
  - `App.vue`：根组件
  - `main.ts`：应用入口（TypeScript）
  - `types/`：TypeScript 类型定义
    - `models.ts`：数据模型类型
    - `components.ts`：组件类型
    - `tauri.ts`：Tauri 命令类型
  - `views/`：页面视图
    - `CardManagementView.vue`：卡片管理页面（主页面）
    - `TemplateView.vue`：模板配置页面
    - `DataTransferView.vue`：数据导出导入页面
    - `AboutView.vue`：关于页面（含更新检查）
    - `LogView.vue`：日志查看页面
  - `components/`：组件
    - `projects/`：项目管理组件
    - `cards/`：卡片管理组件
    - `sf-express/`：顺丰快递组件
    - `qrz/`：QRZ 配置组件

### 测试策略

- **单元测试：**
  - 使用 `#[cfg(test)]` 和 `cargo test` 覆盖核心逻辑
  - 测试覆盖：日志系统、文本渲染、条形码生成、模板配置等
  - 测试通过率：100%（22/22 测试通过）

- **集成测试：**
  - 测试 Tauri Commands 和数据库操作
  - 测试打印流程（Mock 后端）
  - 测试模板配置加载和生成

- **手动测试重点：**
  - 打印输出正确性（条码可扫描、文字居中）
  - 布局稳定性（连续打印不偏移）
  - 错误处理（打印机离线、输入为空、网络错误）
  - 跨平台兼容性（Windows x64/ARM64、macOS Intel/ARM64）
  - 数据库迁移功能
  - 数据导出导入功能

- **CI/CD 测试：**
  - GitHub Actions 自动构建和测试
  - 多平台构建验证（Windows、macOS）
  - 发布流程自动化

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
- **分卡系统：** 批量处理和分发 QSL 卡片的系统，本项目为完整的卡片管理工具

### 数据库结构
- **projects 表**：转卡项目
  - `id`：项目唯一标识（UUID）
  - `name`：项目名称（唯一）
  - `created_at`、`updated_at`：时间戳
- **cards 表**：卡片记录
  - `id`：卡片唯一标识（UUID）
  - `project_id`：所属项目（外键）
  - `callsign`：呼号
  - `qty`：数量
  - `serial`：序列号（可选）
  - `status`：状态（pending、distributed、returned）
  - `metadata`：元数据 JSON（分发信息、地址缓存等）
  - `created_at`、`updated_at`：时间戳
- **address_cache 表**：地址缓存
  - `card_id`：卡片 ID（外键）
  - `source`：数据来源（qrz.cn、qrz.com、qrz.herbertgao.me）
  - `callsign`：呼号
  - `chinese_address`、`english_address`：地址信息
  - `name`、`mail_method`：姓名和邮寄方式
  - `created_at`、`updated_at`：时间戳
- **sf_senders 表**：顺丰寄件人信息
- **sf_orders 表**：顺丰订单信息

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
- **跨平台：** 必须同时支持 Windows（x64、ARM64）和 macOS（Intel、Apple Silicon）
- **RAW 打印模式：** 必须绕过图形驱动，直接发送 TSPL 指令
- **字符安全：** TSPL 字符串需转义双引号、换行符等特殊字符
- **中文支持：** 已支持中文字体渲染（Source Han Sans SC Bold）
- **无运行时依赖：** 编译为单一可执行文件，无需 Python/Node.js 运行时
- **数据库迁移：** 必须支持数据库版本管理和自动迁移

### 业务约束
- **模板配置：** 支持 TOML 格式的模板配置文件，不支持可视化拖拽编辑
- **静默打印：** 不弹出系统打印对话框，直接发送到打印队列
- **输入校验：** 呼号（3-10字符）、数量（1-9999）为必填项，序列号为可选
- **数据持久化：** 所有数据存储在本地 SQLite 数据库，支持导出导入

### 硬件约束
- **纸张尺寸：** 默认 76mm × 130mm，可通过模板配置调整
- **打印偏移：** 提供校准页功能，允许调整 DIRECTION、GAP、REFERENCE 参数
- **走纸稳定性：** 连续打印时需保证标签不偏移（通过正确的 GAP 设置）
- **打印机支持：** 支持 TSPL 协议的热敏打印机（如 Deli DL-888C）

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
tauri = "2.9.5"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.9"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
thiserror = "2.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = "0.4"
rusqlite = { version = "0.32", features = ["bundled"] }
rusttype = "0.9"
ab_glyph = "0.2"
barcoders = "2.0"
image = "0.25"
imageproc = "0.25"
reqwest = { version = "0.12", features = ["json", "cookies"] }
keyring = "2.0"
log = "0.4"
env_logger = "0.11"
once_cell = "1.20"
dirs = "6.0"
# 平台特定打印支持（根据平台条件编译）
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = ["Win32_Graphics_Printing"] }
[target.'cfg(unix)'.dependencies]
cups-sys = "0.4"
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
- **config/profiles.json**：存储配置列表（多个 Profile）
- **config/profiles/{uuid}.toml**：单个配置详情（打印机名称、DPI、GAP、DIRECTION 等）
- **config/templates/**：模板配置文件目录
  - `callsign.toml`：呼号标签模板
  - `address.toml`：地址标签模板
- **config/sf_express_default.toml**：顺丰速运默认配置（Git 忽略，由 CI/CD 生成）
- **cards.db**：SQLite 数据库文件（存储项目、卡片、地址缓存等）
  - 位置：`~/.config/qsl-cardhub/cards.db`（macOS/Linux）
  - 位置：`%APPDATA%\qsl-cardhub\cards.db`（Windows）

---

## 功能特性

### 已实现功能

#### 卡片管理
- ✅ 卡片录入（单条/连续录入模式）
- ✅ 卡片编辑和删除
- ✅ 卡片分发（支持代领人选择）
- ✅ 卡片退卡
- ✅ 卡片查询和筛选（按项目、呼号、状态）
- ✅ 分页显示
- ✅ 卡片详情查看

#### 项目管理
- ✅ 项目创建、编辑、删除
- ✅ 项目统计（卡片总数、各状态数量）
- ✅ 项目列表展示

#### 打印功能
- ✅ QSL 卡片标签打印（呼号、序列号、数量、条形码）
- ✅ 校准页打印
- ✅ 模板配置系统（TOML 格式）
- ✅ 支持呼号标签和地址标签两种模板
- ✅ PDF 预览功能（虚拟打印机）
- ✅ 跨平台打印支持（Windows Win32、macOS CUPS）

#### 顺丰快递集成
- ✅ 顺丰配置管理（生产/沙箱环境）
- ✅ 面单打印
- ✅ 订单创建、确认、取消、查询
- ✅ 订单列表管理
- ✅ 寄件人管理（创建、更新、删除、设置默认）
- ✅ 默认 API 配置（CI/CD 自动生成）

#### QRZ 集成
- ✅ QRZ.cn 集成（登录、地址查询、缓存）
- ✅ QRZ.com 集成（登录、地址查询、缓存）
- ✅ QRZ.herbertgao.me 集成（公开 API、地址查询、缓存）
- ✅ 地址缓存管理（1年有效期、智能去重）
- ✅ 分发时自动查询地址

#### 配置管理
- ✅ Profile 管理（创建、编辑、删除、导入导出）
- ✅ 模板配置管理（TOML 格式）
- ✅ 配置导入导出功能

#### 数据管理
- ✅ SQLite 数据库自动初始化
- ✅ 数据库版本管理和自动迁移
- ✅ 数据导出导入（JSON 格式）
- ✅ 恢复出厂设置功能

#### 安全功能
- ✅ 凭据加密存储（系统钥匙串 + AES-256-GCM）
- ✅ QRZ 登录凭据安全存储
- ✅ 顺丰 API 凭据安全存储

#### 其他功能
- ✅ 日志系统（环形缓冲区、文件持久化、日志查看）
- ✅ 自动更新检查（启动时检查、手动检查、更新下载）
- ✅ 平台信息检测
- ✅ 全局加载状态管理
- ✅ 自定义图标支持

### 版本规划

#### v1.0（当前版本 - 稳定发布）
- ✅ 核心功能完整实现
- ✅ 跨平台支持（Windows x64/ARM64、macOS Intel/ARM64）
- ✅ 自动更新功能
- ✅ 数据导出导入
- 🔄 持续优化和 Bug 修复

#### v2.0（规划中）
- 批量打印功能（序列号自增）
- 网络打印支持（TCP 9100 直连）
- 高级模板编辑器（可视化编辑）

## 构建和部署

### 开发环境
- **Rust**：1.70+（推荐使用 rustup 安装）
- **Node.js**：18+（推荐使用 nvm 管理）
- **pnpm**：9+（包管理器）
- **Tauri CLI**：通过 `cargo install tauri-cli` 安装

### 开发命令
```bash
# 安装依赖
cargo build && cd web && pnpm install && cd ..

# 启动开发服务器
cargo tauri dev

# 生产构建
cargo tauri build
```

### 构建产物
- **macOS**：`.dmg` 安装包和 `.app.tar.gz`（用于自动更新）
- **Windows**：`.exe` 安装包（NSIS）和 `.msi`（可选）
- **签名文件**：`.sig` 文件（用于自动更新验证）

### CI/CD
- **GitHub Actions**：自动构建和发布
- **工作流**：
  - `build.yml`：PR 构建和测试
  - `release.yml`：Tag 发布构建（多平台）
- **自动更新**：通过 GitHub Releases 提供 `latest.json` 清单文件

