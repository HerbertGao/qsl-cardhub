# 提案：修正打印机列表获取，确保从系统获取真实打印机

## 摘要

当前配置管理界面中的打印机列表包含了不需要的虚拟打印机（Mock Printer）。本提案旨在：

1. **删除 MockBackend** - 移除开发用的虚拟打印后端
2. **保留 PdfBackend** - PDF 预览打印机继续显示，用于生成打印预览
3. **确认系统打印机获取** - 验证 CupsBackend 和 WindowsBackend 正确获取系统打印机

## 问题背景

### 当前行为

`PrinterManager::list_printers()` 聚合了所有打印后端的打印机列表：

| 后端 | 平台 | 获取方式 | 返回内容 | 状态 |
|------|------|----------|----------|------|
| CupsBackend | Unix/macOS | `lpstat -p` 命令 | 系统打印机列表 | ✅ 正确 |
| WindowsBackend | Windows | Win32 API `EnumPrintersW` | 系统打印机列表 | ✅ 正确 |
| PdfBackend | 全平台 | 硬编码 | `["PDF 测试打印机"]` | ✅ 保留 |
| MockBackend | 全平台 | 硬编码 | `["Mock Printer", "Mock Printer 2"]` | ❌ 需删除 |

**验证结果**（macOS）：
- `lpstat -p` 正确返回系统打印机：`HP_LaserJet_M104w__C40012_`
- CupsBackend 实现正确，能解析 lpstat 输出

### 期望行为

打印机列表应该显示：
- **系统真实打印机**（通过 CupsBackend/WindowsBackend 获取）
- **PDF 测试打印机**（用于预览，保留）
- ~~Mock Printer~~（删除）

## 影响范围

### 需要修改的文件

| 文件 | 操作 |
|------|------|
| `src/printer/backend/mock.rs` | **删除** |
| `src/printer/backend/mod.rs` | 移除 MockBackend 导出 |
| `src/printer/manager.rs` | 移除 MockBackend 相关代码 |

### 无需修改的文件

| 文件 | 原因 |
|------|------|
| `src/printer/backend/cups.rs` | 已正确实现系统打印机获取 |
| `src/printer/backend/windows.rs` | 已正确实现系统打印机获取 |
| `src/printer/backend/pdf.rs` | 保留现有实现 |
| `src/commands/printer.rs` | 接口不变 |
| `web/src/views/ConfigView.vue` | 自动显示正确列表 |

## 相关代码分析

### CupsBackend（已验证正确）

```rust
// src/printer/backend/cups.rs:34-59
fn list_printers(&self) -> Result<Vec<String>> {
    let output = Command::new("lpstat")
        .arg("-p")
        .output()
        .context("无法执行 lpstat 命令")?;
    // ... 解析 "printer PrinterName is idle..." 格式
}
```

**测试结果**：
```bash
$ lpstat -p
打印机HP_LaserJet_M104w__C40012_闲置，启用时间始于Sun Jan  4 07:34:49 2026
```

### WindowsBackend（代码分析正确）

```rust
// src/printer/backend/windows.rs:40-89
fn list_printers(&self) -> Result<Vec<String>> {
    // 使用 EnumPrintersW 枚举 PRINTER_ENUM_LOCAL 打印机
    // 返回 PRINTER_INFO_2W 中的 pPrinterName
}
```

使用标准 Win32 API，实现正确。

### PdfBackend（保留）

```rust
// src/printer/backend/pdf.rs:388-391
fn list_printers(&self) -> Result<Vec<String>> {
    Ok(vec!["PDF 测试打印机".to_string()])
}
```

PDF 后端提供预览功能，保留显示。
