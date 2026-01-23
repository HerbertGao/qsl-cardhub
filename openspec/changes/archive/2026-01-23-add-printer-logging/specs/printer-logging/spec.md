# 规范：打印日志增强

## 概述

增强打印操作的日志记录，捕获打印机和操作系统的响应信息，便于调试。

## ADDED

- `PrintResult` 结构体 - 打印操作结果，包含成功状态、作业 ID、消息和详细信息
- `PrintResult::success()` - 创建成功结果的便捷方法
- `PrintResult::success_with_job_id()` - 创建带作业 ID 的成功结果
- `PrintResult::with_details()` - 设置详细信息的链式方法
- `CupsBackend::parse_job_id()` - 解析 lp 命令输出获取作业 ID
- `WindowsBackend::format_win32_error()` - Win32 错误码转可读描述
- `WindowsBackend::log_last_error()` - 记录当前 Win32 错误

## MODIFIED

- `PrinterBackend::send_raw()` - 返回值从 `Result<()>` 改为 `Result<PrintResult>`
- `CupsBackend::send_raw()` - 增强日志记录，返回 `PrintResult`
- `WindowsBackend::send_raw()` - 增强日志记录，返回 `PrintResult`
- `PdfBackend::send_raw()` - 更新签名匹配 trait
- `print_qsl` 命令 - 处理 `PrintResult` 并记录作业 ID
- `sf_print_waybill` 命令 - 处理 `PrintResult` 并在返回消息中包含作业 ID

## 数据结构

### PrintResult

```rust
/// 打印操作结果
#[derive(Debug, Clone)]
pub struct PrintResult {
    /// 是否成功
    pub success: bool,
    /// 打印作业 ID（如果系统提供）
    pub job_id: Option<String>,
    /// 结果消息
    pub message: String,
    /// 详细信息（stdout/stderr 等）
    pub details: Option<String>,
}
```

## 接口变更

### PrinterBackend trait

```rust
pub trait PrinterBackend: Send + Sync {
    /// 后端名称
    fn name(&self) -> &str;

    /// 列出可用打印机
    fn list_printers(&self) -> Result<Vec<String>>;

    /// 发送原始数据到打印机
    ///
    /// 返回 PrintResult 包含详细的打印结果信息
    fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<PrintResult>;
}
```

## CUPS 后端实现

### 日志记录内容

1. **打印前**：
   - 打印机名称
   - 数据大小（字节）

2. **打印中**：
   - `lp` 命令的完整 stdout
   - `lp` 命令的完整 stderr

3. **打印后**：
   - 解析的作业 ID（格式：`request id is PRINTER-123`）
   - 打印队列状态（可选，通过 `lpstat` 获取）

### 作业 ID 解析

CUPS `lp` 命令成功时输出格式：
```
request id is PRINTER-123 (1 file(s))
```

使用正则表达式解析：
```rust
let re = Regex::new(r"request id is ([^\s]+)").unwrap();
```

## Windows 后端实现

### 日志记录内容

1. **打印前**：
   - 打印机名称
   - 数据大小（字节）

2. **打印中**：
   - Win32 API 调用的返回值
   - `GetLastError` 错误码和描述

3. **打印后**：
   - 文档 ID（从 `StartDocPrinter` 返回）
   - 打印机状态标志（如果可获取）

### 错误码转换

使用 `FormatMessageW` 将 Win32 错误码转换为可读描述。

## 日志格式

### 成功日志示例

```
[INFO] 🖨️ 开始打印: 打印机=TSC_TDP-225, 数据大小=1234字节
[INFO] 📤 发送打印数据...
[INFO] 📋 系统响应: request id is TSC_TDP-225-456 (1 file(s))
[INFO] ✅ 打印成功: 作业ID=TSC_TDP-225-456
```

### 失败日志示例

```
[INFO] 🖨️ 开始打印: 打印机=TSC_TDP-225, 数据大小=1234字节
[INFO] 📤 发送打印数据...
[ERROR] ❌ 打印失败: 打印机离线
[ERROR] 详细信息: lp: error - unable to access "TSC_TDP-225" - Printer not available
```

## 兼容性

- 现有调用方只需处理 `Result<PrintResult>` 而非 `Result<()>`
- 如果只关心成功/失败，可以只检查 `result.success`
- 额外的日志信息不影响现有功能

## 测试要点

1. 正常打印时能获取作业 ID
2. 打印机离线时能获取详细错误
3. 日志内容完整且可读
4. 不影响打印性能
