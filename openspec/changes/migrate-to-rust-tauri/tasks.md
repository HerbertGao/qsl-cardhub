# 实施任务清单

本文档列出了从 Python + Eel 迁移到 Rust + Tauri 的详细实施任务。任务按照依赖关系和优先级排序，提供可验证的小步骤。

## 阶段 1：基础设施搭建（第 1-2 周）

### 任务 1.1：Tauri 项目初始化

**描述**：创建 Tauri 项目结构，配置基础依赖。

**步骤**：
1. 安装 Tauri CLI：`cargo install tauri-cli`
2. 初始化 Tauri 项目（在现有 Cargo 项目中）：`cargo tauri init`
3. 配置 `tauri.conf.json`：
   - 设置 `productName: "QSL-CardHub"`
   - 设置 `identifier: "com.herbert.qsl-cardhub"`
   - 配置窗口尺寸（1200x800）
4. 配置 `Cargo.toml` 添加核心依赖：
   ```toml
   [dependencies]
   tauri = { version = "2", features = ["shell-open"] }
   serde = { version = "1.0", features = ["derive"] }
   toml = "0.8"
   anyhow = "1.0"
   uuid = { version = "1.0", features = ["v4", "serde"] }
   chrono = { version = "0.4", features = ["serde"] }
   ```

**验证**：
- 运行 `cargo tauri dev` 能成功启动应用
- 窗口标题显示"QSL-CardHub"
- 窗口尺寸为 1200x800

**估算时间**：2-3 小时

---

### 任务 1.2：前端集成

**描述**：集成现有的 Vue 3 前端到 Tauri。

**步骤**：
1. 在 `tauri.conf.json` 中配置前端：
   - 设置 `beforeDevCommand: "cd web && npm run dev"`
   - 设置 `beforeBuildCommand: "cd web && npm run build"`
   - 设置 `devUrl: "http://localhost:5173"`
   - 设置 `frontendDist: "../web/dist"`
2. 修改 `web/vite.config.js`：
   - 设置 `base: "./"`（使用相对路径）
3. 修改 `web/index.html`：
   - 移除 Eel.js 引用

**验证**：
- 运行 `cargo tauri dev` 能显示 Vue 前端界面
- 前端热重载功能正常
- 控制台无错误

**估算时间**：1-2 小时

---

### 任务 1.3：应用状态管理

**描述**：实现 Tauri 应用状态管理（AppState）。

**步骤**：
1. 创建 `src/state.rs`：
   ```rust
   use std::sync::{Arc, Mutex};
   use crate::config::ProfileManager;
   use crate::printer::PrinterManager;

   pub struct AppState {
       pub profile_manager: Arc<Mutex<ProfileManager>>,
       pub printer_manager: Arc<Mutex<PrinterManager>>,
   }
   ```
2. 在 `src/main.rs` 中初始化状态：
   ```rust
   .setup(|app| {
       let state = AppState {
           profile_manager: Arc::new(Mutex::new(ProfileManager::new()?)),
           printer_manager: Arc::new(Mutex::new(PrinterManager::new()?)),
       };
       app.manage(state);
       Ok(())
   })
   ```

**验证**：
- 应用启动成功
- 状态初始化无错误

**估算时间**：1 小时

---

### 任务 1.4：基础 Tauri Commands

**描述**：实现第一个简单的 Tauri Command 测试前后端通信。

**步骤**：
1. 创建 `src/commands/mod.rs`
2. 实现 `get_platform_info()` command：
   ```rust
   #[tauri::command]
   fn get_platform_info() -> PlatformInfo {
       detect_platform()
   }
   ```
3. 在 `main.rs` 中注册 command
4. 在前端调用测试：
   ```javascript
   import { invoke } from '@tauri-apps/api/core';
   const platform = await invoke('get_platform_info');
   console.log(platform);
   ```

**验证**：
- 前端能成功调用 command
- 返回正确的平台信息（os 和 arch）

**估算时间**：2 小时

---

## 阶段 2：配置管理实现（第 3 周）

### 任务 2.1：数据模型定义

**描述**：定义 Profile 相关的 Rust structs。

**步骤**：
1. 创建 `src/config/models.rs`
2. 定义所有数据模型：
   - `Profile`
   - `Platform`
   - `PrinterConfig`
   - `PaperSpec`
   - `Template`
   - `AppConfig`（替代 ProfileStore）
   - `WindowState`
3. 为所有 struct 派生 `Serialize`、`Deserialize`、`Debug`、`Clone` traits

**验证**：
- 编译通过
- 能正确序列化/反序列化 TOML

**估算时间**：2 小时

---

### 任务 2.2：ProfileManager 实现

**描述**：实现配置管理器的核心逻辑。

**步骤**：
1. 创建 `src/config/profile_manager.rs`
2. 实现 `ProfileManager::new()` - 加载 config.toml 和扫描 profiles/ 目录
3. 实现 `ProfileManager::get_all()` - 读取所有 .toml 文件并解析
4. 实现 `ProfileManager::get_by_id()` - 读取单个配置文件
5. 实现 `ProfileManager::create()` - 创建新的 profiles/{uuid}.toml
6. 实现 `ProfileManager::update()` - 更新 profiles/{id}.toml
7. 实现 `ProfileManager::delete()` - 删除 profiles/{id}.toml
8. 实现 `ProfileManager::set_default()` - 更新 config.toml
9. 添加配置文件注释生成逻辑

**验证**：
- 单元测试通过
- 能正确读写 TOML 文件
- 配置文件结构正确

**估算时间**：8-10 小时

---

### 任务 2.3：配置管理 Commands

**描述**：实现配置管理相关的 Tauri Commands。

**步骤**：
1. 在 `src/commands/profile.rs` 中实现：
   - `get_profiles()`
   - `get_profile(id)`
   - `create_profile(name, printer_name)`
   - `update_profile(id, profile)`
   - `delete_profile(id)`
   - `set_default_profile(id)`
   - `export_profile(id)`
   - `import_profile(json)`
2. 注册所有 commands 到 Tauri

**验证**：
- 前端能成功调用所有 commands
- 配置 CRUD 操作正常

**估算时间**：4-5 小时

---

### 任务 2.4：前端配置管理 UI 集成

**描述**：修改前端代码，将 Eel API 调用替换为 Tauri Commands。

**步骤**：
1. 安装 `@tauri-apps/api`：`npm install @tauri-apps/api`
2. 在 `ConfigView.vue` 中替换所有 `window.eel.xxx()` 调用为 `invoke('xxx')`
3. 更新错误处理逻辑（Tauri 返回的错误是 String）
4. 测试所有配置管理功能

**验证**：
- 能创建、编辑、删除配置
- 能导入导出配置
- 能设置默认配置

**估算时间**：3-4 小时

---

## 阶段 3：打印功能实现（第 4-5 周）

### 任务 3.1：TSPL 生成器

**描述**：实现 TSPL 指令生成逻辑。

**步骤**：
1. 创建 `src/printer/tspl.rs`
2. 实现 `TSPLGenerator::new()`
3. 实现 `TSPLGenerator::generate_qsl_card(callsign, serial, qty)`
4. 实现 `TSPLGenerator::generate_calibration_page()`
5. 添加单元测试验证生成的 TSPL 指令

**验证**：
- 生成的 TSPL 指令与 Python 版完全一致
- 单元测试通过

**估算时间**：4-5 小时

---

### 任务 3.2：打印机后端抽象

**描述**：定义 PrinterBackend trait 和 Mock 实现。

**步骤**：
1. 创建 `src/printer/backend/mod.rs`
2. 定义 `PrinterBackend` trait
3. 创建 `src/printer/backend/mock.rs`
4. 实现 `MockBackend`：
   - `list_printers()` 返回 `["Mock Printer"]`
   - `send_raw()` 保存到 `output/print_YYYYMMDD_HHMMSS.tspl`

**验证**：
- Mock 后端能正确保存 TSPL 文件
- 文件内容正确

**估算时间**：3 小时

---

### 任务 3.3：Windows 打印后端

**描述**：实现 Windows 平台的 RAW 打印支持。

**依赖**：任务 3.2

**步骤**：
1. 在 `Cargo.toml` 添加 Windows 依赖：
   ```toml
   [target.'cfg(windows)'.dependencies]
   windows = { version = "0.58", features = [
       "Win32_Graphics_Printing",
       "Win32_Graphics_Gdi",
       "Win32_Foundation"
   ] }
   ```
2. 创建 `src/printer/backend/windows.rs`
3. 实现 `WindowsBackend::list_printers()` 使用 `EnumPrintersW()`
4. 实现 `WindowsBackend::send_raw()` 使用 Win32 打印 API

**验证**：
- 在 Windows 平台能枚举打印机
- 能成功发送 RAW 打印数据到实际打印机

**估算时间**：8-10 小时

---

### 任务 3.4：CUPS 打印后端

**描述**：实现 macOS/Linux 平台的 CUPS 打印支持。

**依赖**：任务 3.2

**步骤**：
1. 创建 `src/printer/backend/cups.rs`
2. 实现 `CupsBackend::list_printers()` 使用 `lpstat -p`
3. 实现 `CupsBackend::send_raw()` 使用 `lp -d <printer> -o raw`
4. 添加错误处理（解析命令输出）

**验证**：
- 在 macOS/Linux 平台能枚举打印机
- 能成功发送打印数据到实际打印机

**估算时间**：6-8 小时

---

### 任务 3.5：PrinterManager 实现

**描述**：实现打印机管理器，聚合所有后端。

**依赖**：任务 3.2、3.3、3.4

**步骤**：
1. 创建 `src/printer/manager.rs`
2. 实现 `PrinterManager::new()` - 初始化所有后端
3. 实现 `PrinterManager::list_printers()` - 聚合所有后端的打印机
4. 实现 `PrinterManager::print_qsl()` - 路由打印任务到正确的后端
5. 实现 `PrinterManager::print_calibration()`

**验证**：
- 能正确聚合所有打印机
- 打印任务能正确路由

**估算时间**：4 小时

---

### 任务 3.6：打印 Commands

**描述**：实现打印相关的 Tauri Commands。

**依赖**：任务 3.5

**步骤**：
1. 在 `src/commands/printer.rs` 中实现：
   - `get_printers()`
   - `print_qsl(profile_id, callsign, serial, qty)`
   - `print_calibration(profile_id)`
2. 注册 commands 到 Tauri
3. 添加错误处理和日志

**验证**：
- 前端能成功调用打印 commands
- 打印功能正常

**估算时间**：3 小时

---

### 任务 3.7：前端打印 UI 集成

**描述**：修改前端打印页面，替换 Eel API 为 Tauri Commands。

**依赖**：任务 3.6

**步骤**：
1. 在 `PrintView.vue` 中替换 API 调用
2. 更新打印机列表获取逻辑
3. 更新打印按钮点击处理
4. 测试打印功能

**验证**：
- 能显示打印机列表
- 能成功打印 QSL 卡片
- 能打印校准页

**估算时间**：2-3 小时

---

## 阶段 4：集成测试和优化（第 6 周）

### 任务 4.1：功能测试

**描述**：全面测试所有功能。

**步骤**：
1. 测试配置管理（创建、编辑、删除、导入导出）
2. 测试打印功能（QSL 卡片、校准页）
3. 测试序列号自动增长
4. 测试错误处理（打印机离线、输入为空等）

**验证**：
- 所有功能正常工作
- 无崩溃或严重 bug

**估算时间**：4-6 小时

---

### 任务 4.2：跨平台测试

**描述**：在 Windows、macOS、Linux 三个平台测试。

**步骤**：
1. 在 Windows 平台构建并测试
2. 在 macOS 平台构建并测试
3. 在 Linux 平台构建并测试（Ubuntu/Debian）
4. 记录平台特定问题并修复

**验证**：
- 所有平台启动正常
- 打印功能在所有平台正常

**估算时间**：6-8 小时

---

### 任务 4.3：打包测试

**描述**：测试应用打包和分发。

**步骤**：
1. 运行 `cargo tauri build` 构建生产版本
2. 验证可执行文件体积 < 20MB
3. 测试 Windows `.msi` 安装包
4. 测试 macOS `.dmg` 镜像
5. 测试 Linux `.AppImage`

**验证**：
- 所有平台打包成功
- 安装包能正常安装和卸载
- 体积明显小于 Python 版

**估算时间**：4-5 小时

---

### 任务 4.4：性能优化

**描述**：优化启动时间和运行性能。

**步骤**：
1. 测量启动时间（目标 < 500ms）
2. 优化 ProfileManager 加载逻辑（异步加载）
3. 优化 PrinterManager 初始化（延迟初始化后端）
4. 启用 Cargo release 优化（`lto = true`）

**验证**：
- 启动时间 < 500ms
- 打印响应时间 < 1s

**估算时间**：3-4 小时

---

### 任务 4.5：文档完善

**描述**：编写用户文档和开发文档。

**步骤**：
1. 更新 README.md（Rust 版使用说明）
2. 编写构建指南（Windows、macOS、Linux）
3. 编写迁移指南（从 Python 版迁移）
4. 添加代码注释（关键模块）

**验证**：
- 文档完整准确
- 新用户能根据文档使用应用

**估算时间**：4-5 小时

---

## 总计

**总估算时间**：约 80-100 小时（10-12 工作日）

**关键里程碑**：
1. 第 1 周结束：基础设施搭建完成，前后端通信正常
2. 第 3 周结束：配置管理功能完成
3. 第 5 周结束：打印功能完成
4. 第 6 周结束：测试、优化、打包完成

**风险和缓解**：
- **Windows 打印 API 复杂**：预留额外时间调试，先使用 Mock 后端测试
- **CUPS 依赖问题**：使用命令行调用避免编译依赖
- **Tauri 学习曲线**：先实现简单功能，逐步学习

**并行任务**：
- 任务 3.3 和 3.4 可以并行（不同平台）
- 任务 4.1、4.2、4.3 可以部分并行（不同测试类型）
