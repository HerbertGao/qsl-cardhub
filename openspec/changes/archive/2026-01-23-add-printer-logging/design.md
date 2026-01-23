# 设计文档：打印日志增强

## 架构概览

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri Commands                        │
│  (print_cards, print_waybill)                           │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│                  PrinterBackend trait                    │
│  send_raw() -> Result<PrintResult>                      │
└─────────────────────┬───────────────────────────────────┘
                      │
          ┌───────────┴───────────┐
          ▼                       ▼
┌─────────────────┐     ┌─────────────────┐
│   CupsBackend   │     │  WindowsBackend │
│  (macOS/Linux)  │     │    (Windows)    │
├─────────────────┤     ├─────────────────┤
│  - lp 命令       │     │  - Win32 API    │
│  - lpstat 查询   │     │  - GetLastError │
│  - 作业ID解析    │     │  - GetJob 查询   │
└─────────────────┘     └─────────────────┘
```

## 数据流

### 打印成功流程

```
1. 命令层调用 send_raw(printer, data)
2. 后端记录: "开始打印: printer=xxx, size=xxx"
3. 后端执行打印操作
4. 后端解析系统响应获取作业ID
5. 后端记录: "打印成功: job_id=xxx"
6. 返回 PrintResult { success: true, job_id: Some("xxx"), ... }
7. 命令层记录完成日志
```

### 打印失败流程

```
1. 命令层调用 send_raw(printer, data)
2. 后端记录: "开始打印: printer=xxx, size=xxx"
3. 后端执行打印操作
4. 后端捕获错误和详细信息
5. 后端记录: "打印失败: error=xxx, details=xxx"
6. 返回 Err(...) 或 PrintResult { success: false, ... }
7. 命令层处理错误并记录
```

## CUPS 实现细节

### lp 命令输出解析

成功输出格式：
```
request id is PRINTER-123 (1 file(s))
```

解析代码：
```rust
fn parse_job_id(output: &str) -> Option<String> {
    // 使用简单字符串匹配避免正则依赖
    if let Some(start) = output.find("request id is ") {
        let rest = &output[start + 14..];
        if let Some(end) = rest.find(' ') {
            return Some(rest[..end].to_string());
        }
    }
    None
}
```

### lpstat 状态查询（可选）

```rust
fn get_job_status(job_id: &str) -> Option<String> {
    let output = Command::new("lpstat")
        .arg("-l")
        .arg("-o")
        .arg(job_id)
        .output()
        .ok()?;

    String::from_utf8(output.stdout).ok()
}
```

## Windows 实现细节

### 错误码转换

```rust
fn format_win32_error(error_code: u32) -> String {
    unsafe {
        let mut buffer: [u16; 512] = [0; 512];
        let len = FormatMessageW(
            FORMAT_MESSAGE_FROM_SYSTEM,
            std::ptr::null(),
            error_code,
            0,
            buffer.as_mut_ptr(),
            buffer.len() as u32,
            std::ptr::null(),
        );
        if len > 0 {
            String::from_utf16_lossy(&buffer[..len as usize])
                .trim()
                .to_string()
        } else {
            format!("未知错误 ({})", error_code)
        }
    }
}
```

### 作业信息获取（可选）

```rust
fn get_job_info(printer_handle: HANDLE, job_id: u32) -> Option<String> {
    // 使用 GetJob API 获取作业状态
    // 这是可选的增强功能
}
```

## 日志级别策略

| 事件 | 级别 | 说明 |
|------|------|------|
| 开始打印 | INFO | 记录打印机和数据大小 |
| 系统响应（成功） | INFO | 记录原始响应 |
| 系统响应（失败） | ERROR | 记录错误详情 |
| 作业ID解析 | DEBUG | 调试时有用 |
| 队列状态 | DEBUG | 可选的额外信息 |

## 风险与缓解

### 风险1：日志过多影响性能

**缓解**：使用适当的日志级别，详细信息使用 DEBUG 级别

### 风险2：不同系统输出格式差异

**缓解**：使用容错的解析逻辑，解析失败时返回 None 而非错误

### 风险3：API 签名变更影响调用方

**缓解**：提供向后兼容的辅助方法或默认实现
