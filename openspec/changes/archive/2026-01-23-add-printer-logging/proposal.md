# 提案：打印日志增强

## 概述

增强打印操作的日志记录功能，捕获打印机返回的响应和操作系统级别的打印日志，便于后续调试和问题排查。

## 背景

当前打印功能（包括卡片标签打印和顺丰面单打印）的日志记录非常简单：
- 仅记录"打印成功"或错误信息
- 不捕获打印机的实际响应
- 不记录操作系统级别的打印队列状态
- 难以排查打印失败但无明确错误的情况

### 当前实现分析

1. **CUPS 后端**（macOS/Linux）：
   - 使用 `lp` 命令发送原始数据
   - 捕获 stdout/stderr 但仅用于错误判断
   - 不记录打印作业 ID 或队列状态

2. **Windows 后端**：
   - 使用 Win32 API（OpenPrinter, WritePrinter 等）
   - 仅返回成功/失败，不记录详细信息

3. **打印命令层**：
   - `print_cards` 和 `print_waybill` 使用相同的 `send_raw` 路径
   - 日志仅记录"已发送到打印机"

## 目标

1. 捕获并记录打印机/打印系统的响应
2. 记录打印作业的详细状态
3. 提供足够的调试信息用于问题排查
4. 不影响现有打印功能的正常运行

## 非目标

- 实现打印状态的实时跟踪（可作为后续功能）
- 修改打印内容或格式
- 实现打印重试机制

## 技术方案

### 1. 增强 PrinterBackend trait

```rust
pub trait PrinterBackend: Send + Sync {
    fn name(&self) -> &str;
    fn list_printers(&self) -> Result<Vec<String>>;
    fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<PrintResult>;
}

pub struct PrintResult {
    pub success: bool,
    pub job_id: Option<String>,
    pub message: String,
    pub details: Option<String>,
}
```

### 2. CUPS 后端增强

- 解析 `lp` 命令输出获取作业 ID
- 调用 `lpstat` 获取打印队列状态
- 记录详细的 stdout/stderr 内容

### 3. Windows 后端增强

- 捕获 `GetLastError` 详细错误码
- 使用 `GetJob` 获取作业状态
- 记录打印机状态标志

### 4. 日志记录策略

- 打印前：记录打印机名称、数据大小
- 打印中：记录系统返回的响应
- 打印后：记录作业 ID 和最终状态

## 影响范围

### 需要修改的文件

1. `src/printer/backend/mod.rs` - 增强 trait 定义
2. `src/printer/backend/cups.rs` - CUPS 实现增强
3. `src/printer/backend/windows.rs` - Windows 实现增强
4. `src/commands/printer.rs` - 更新打印命令使用新返回值
5. `src/commands/sf_express.rs` - 更新面单打印使用新返回值

### 兼容性

- 向后兼容：现有 API 签名不变
- 日志增强不影响打印功能本身

## 实现计划

1. **Phase 1**: 增强 PrinterBackend trait 和数据结构
2. **Phase 2**: 实现 CUPS 后端日志增强
3. **Phase 3**: 实现 Windows 后端日志增强
4. **Phase 4**: 更新打印命令层

## 验收标准

1. 每次打印操作都记录详细日志
2. 日志包含打印作业 ID（如果系统提供）
3. 错误情况下能获取更多调试信息
4. 现有打印功能正常工作
