# 打印功能规范（Rust 版）

## 目的

本规范定义了 qsl-cardhub Rust 版本的打印功能需求。使用 Rust 实现 TSPL 指令生成和跨平台打印支持，保持与 Python 版的功能一致性。本规范在 Python 版的基础上，利用 Rust 的类型系统和条件编译特性优化打印实现。

## 修改需求

### 需求： TSPL 渲染器

**原需求**：系统必须提供 TSPL 命令渲染器，生成适用于 Deli DL-888C 打印机的打印指令。（Python 版）

**修改原因**：迁移到 Rust 实现，使用 String 拼接和格式化宏。

**新需求**：系统必须使用 Rust 实现 TSPLGenerator，生成与 Python 版完全一致的 TSPL 指令。

#### 场景： 渲染 QSL 卡片

- **当** 系统调用 `TSPLGenerator::generate_qsl_card(callsign, serial, qty)`
- **那么** 应使用 Rust 的 `format!()` 宏生成 TSPL 字符串
- **并且** 应包含以下内容（按顺序）：
  1. `SIZE 76 mm, 130 mm\n`
  2. `GAP 2 mm, 0 mm\n`
  3. `DIRECTION 0\n`
  4. `CLS\n`
  5. `TEXT 304,80,"5",0,3,3,"{callsign}"\n`（呼号，大字号，居中）
  6. `BARCODE 200,300,"128",120,1,0,3,3,"{callsign}"\n`（条形码，居中，高度120 dots）
  7. `TEXT 50,520,"5",0,2,2,"SN: {:03}"\n`（序列号，使用 Rust 的 {:03} 格式化）
  8. `TEXT 50,720,"5",0,2,2,"QTY: {}"\n`（数量）
  9. `PRINT 1\n`
- **并且** 返回完整的 TSPL 命令字符串（`String` 类型）
- **并且** 字符串应使用 UTF-8 编码

#### 场景： 校准页渲染

- **当** 系统调用 `TSPLGenerator::generate_calibration_page()`
- **那么** 应生成包含以下内容的 TSPL 命令：
  - 标题文字（"CALIBRATION PAGE"）
  - 边框（顶部、底部、左侧、右侧）
  - 四个角标记
  - 尺寸信息文本
  - 中心十字线
- **并且** 与 Python 版输出完全一致

#### 场景： 字符转义

- **当** 呼号包含特殊字符（如双引号、反斜杠）
- **那么** 系统应转义特殊字符
- **并且** 使用 Rust 的 `escape_default()` 或自定义转义逻辑
- **并且** 确保 TSPL 命令不会因特殊字符而损坏

### 需求： 打印机通信

**原需求**：系统必须支持向打印机发送原始 TSPL 命令。（Python 版）

**修改原因**：使用 Rust 的 trait 和条件编译实现跨平台打印。

**新需求**：系统必须使用 PrinterBackend trait 抽象打印操作，并为不同平台提供实现。

#### 场景： 发送打印命令

- **当** 系统调用 `PrinterManager::print_qsl(printer_name, callsign, serial, qty)`
- **那么** 应使用 TSPLGenerator 生成 TSPL 命令
- **并且** 应将命令转换为 `&[u8]`（UTF-8 字节）
- **并且** 应调用对应 PrinterBackend 的 `send_raw()` 方法
- **并且** 如果打印机未找到，应返回错误"打印机未找到: {printer_name}"

#### 场景： 打印错误处理

- **当** 打印过程中出现错误
- **那么** 系统应返回 `Result<(), anyhow::Error>`
- **并且** 错误应包含详细信息（打印机名称、错误原因）
- **并且** Tauri Command 应将错误转换为中文消息返回前端

## 新增需求

### 需求： 打印机后端抽象层

系统必须定义 PrinterBackend trait 统一打印接口。

#### 场景： PrinterBackend trait 定义

- **当** 系统定义打印机后端
- **那么** 应定义以下 trait：
  ```rust
  pub trait PrinterBackend: Send + Sync {
      fn name(&self) -> &str;
      fn list_printers(&self) -> Result<Vec<String>>;
      fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<()>;
  }
  ```
- **并且** trait 应标记为 `Send + Sync`（支持多线程）
- **并且** 所有方法应返回 `Result` 类型

#### 场景： 后端实现

- **当** 系统初始化 PrinterManager
- **那么** 应创建以下后端实例：
  - Windows: `WindowsBackend`（仅在 Windows 平台编译）
  - CUPS: `CupsBackend`（仅在 Unix 平台编译）
  - Mock: `MockBackend`（所有平台）
- **并且** 使用 `Vec<Box<dyn PrinterBackend>>` 存储后端列表

### 需求： Windows 打印后端

系统必须使用 windows-rs crate 实现 Windows 平台的 RAW 打印。

#### 场景： Windows 后端初始化

- **当** 在 Windows 平台初始化 WindowsBackend
- **那么** 应使用 `#[cfg(target_os = "windows")]` 条件编译
- **并且** 应导入 `windows::Win32::Graphics::Printing` 模块

#### 场景： 枚举 Windows 打印机

- **当** 调用 `WindowsBackend::list_printers()`
- **那么** 应调用 Win32 API `EnumPrintersW()`
- **并且** 应解析 `PRINTER_INFO_2` 结构
- **并且** 应返回打印机名称列表（`Vec<String>`）

#### 场景： 发送 RAW 打印数据

- **当** 调用 `WindowsBackend::send_raw(printer_name, data)`
- **那么** 应执行以下步骤：
  1. 调用 `OpenPrinterW()` 打开打印机
  2. 配置 `DOC_INFO_1` 结构（文档名称、数据类型为 "RAW"）
  3. 调用 `StartDocPrinterW()` 开始打印任务
  4. 调用 `StartPagePrinter()` 开始页面
  5. 调用 `WritePrinter()` 写入 TSPL 数据
  6. 调用 `EndPagePrinter()` 结束页面
  7. 调用 `EndDocPrinter()` 结束打印任务
  8. 调用 `ClosePrinter()` 关闭打印机
- **并且** 如果任何步骤失败，应返回详细错误

### 需求： CUPS 打印后端

系统必须使用 `lp` 命令行工具实现 macOS/Linux 平台的打印。

#### 场景： CUPS 后端初始化

- **当** 在 Unix 平台初始化 CupsBackend
- **那么** 应使用 `#[cfg(target_family = "unix")]` 条件编译
- **并且** 不需要额外的 crate 依赖

#### 场景： 枚举 CUPS 打印机

- **当** 调用 `CupsBackend::list_printers()`
- **那么** 应执行命令 `lpstat -p`
- **并且** 应解析输出（每行格式：`printer <name> is ...`）
- **并且** 应提取打印机名称
- **并且** 返回打印机名称列表

#### 场景： 发送 CUPS 打印数据

- **当** 调用 `CupsBackend::send_raw(printer_name, data)`
- **那么** 应执行以下步骤：
  1. 创建临时文件保存 TSPL 数据
  2. 执行命令 `lp -d {printer_name} -o raw {temp_file}`
  3. 等待命令完成
  4. 删除临时文件
- **并且** 如果命令失败，应返回详细错误（包含 stderr 输出）

### 需求： Mock 打印后端

系统必须提供 Mock 后端用于开发测试。

#### 场景： Mock 后端初始化

- **当** 初始化 MockBackend
- **那么** 应创建 `output/` 目录（如果不存在）
- **并且** 应存储输出目录路径

#### 场景： Mock 打印机列表

- **当** 调用 `MockBackend::list_printers()`
- **那么** 应返回 `vec!["Mock Printer".to_string()]`

#### 场景： Mock 打印输出

- **当** 调用 `MockBackend::send_raw(printer_name, data)`
- **那么** 应生成文件名 `output/print_{YYYYMMDD}_{HHMMSS}.tspl`
- **并且** 应使用 `std::fs::write()` 保存 TSPL 数据
- **并且** 应在控制台输出"TSPL 命令已保存到: {file_path}"

### 需求： PrinterManager 实现

系统必须提供 PrinterManager 管理所有打印操作。

#### 场景： 初始化 PrinterManager

- **当** 系统启动时初始化 PrinterManager
- **那么** 应创建所有可用的 PrinterBackend 实例
- **并且** 应创建 TSPLGenerator 实例（DPI=203）
- **并且** 应存储到 `Arc<Mutex<PrinterManager>>`

#### 场景： 聚合打印机列表

- **当** 调用 `PrinterManager::list_printers()`
- **那么** 应遍历所有后端
- **并且** 应调用每个后端的 `list_printers()` 方法
- **并且** 应合并所有打印机名称
- **并且** 应去重
- **并且** 返回完整列表

#### 场景： 路由打印任务

- **当** 调用 `PrinterManager::print_qsl(printer_name, ...)`
- **那么** 应遍历所有后端
- **并且** 应找到包含该打印机的后端
- **并且** 应调用该后端的 `send_raw()` 方法
- **并且** 如果所有后端都不支持该打印机，应返回错误

### 需求： 序列号管理

系统必须提供序列号自动管理功能（前端实现）。

#### 场景： 序列号初始化

- **当** 前端打印页面加载
- **那么** 序列号应从 1 开始
- **并且** 存储在前端状态中

#### 场景： 序列号自动增长

- **当** 打印成功后
- **那么** 前端应自动增加序列号
- **并且** 如果序列号 >= 999，应重置为 1

#### 场景： 序列号格式化

- **当** 前端显示或发送序列号
- **那么** 应格式化为 3 位数字（如 "001"）
- **并且** 后端接收的是数字类型（u32）
