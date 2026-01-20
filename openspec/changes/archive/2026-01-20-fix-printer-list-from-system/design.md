# 设计文档：打印机列表获取修正

## 变更概述

本次变更主要是：

1. **删除 MockBackend** - 移除不再需要的开发用虚拟打印后端
2. **保留其他后端** - CupsBackend、WindowsBackend、PdfBackend 无需修改

## 详细设计

### 1. 删除 MockBackend

**理由**：
- MockBackend 是开发阶段的临时方案，用于在没有真实打印机时测试打印流程
- 现在有了 PdfBackend 可以生成预览，MockBackend 不再需要
- MockBackend 在打印机列表中显示虚拟打印机名称（Mock Printer），造成用户困惑

**操作**：

1. 删除文件 `src/printer/backend/mock.rs`

2. 修改 `src/printer/backend/mod.rs`：
   ```rust
   // 删除以下行：
   pub mod mock;
   pub use mock::MockBackend;
   ```

3. 修改 `src/printer/manager.rs`：
   ```rust
   // 删除以下 import：
   use super::backend::MockBackend;

   // 删除 PrinterManager::new() 中的以下代码：
   backends.push(Box::new(MockBackend::new(output_dir)?));
   ```

### 2. 保留 PdfBackend

PdfBackend 继续提供 "PDF 测试打印机"，用于生成打印预览（PNG/PDF 文件）。

**现有实现**（无需修改）：
```rust
impl PrinterBackend for PdfBackend {
    fn list_printers(&self) -> Result<Vec<String>> {
        Ok(vec!["PDF 测试打印机".to_string()])
    }
}
```

### 3. CupsBackend 和 WindowsBackend 保持不变

**验证结果**：

**CupsBackend（macOS/Linux）**：
- 使用 `lpstat -p` 命令枚举系统打印机
- 解析输出格式 `printer <name> is <status>...`
- 实际测试：正确返回 `HP_LaserJet_M104w__C40012_`

**WindowsBackend（Windows）**：
- 使用 Win32 API `EnumPrintersW` 枚举打印机
- 使用 `PRINTER_ENUM_LOCAL` 获取本地打印机
- 从 `PRINTER_INFO_2W.pPrinterName` 提取打印机名称
- 代码逻辑正确，符合 Windows 打印 API 规范

## PrinterManager 变更后的结构

```rust
impl PrinterManager {
    pub fn new() -> Result<Self> {
        let mut backends: Vec<Box<dyn PrinterBackend>> = Vec::new();

        // 平台特定后端（获取真实打印机）
        #[cfg(target_os = "windows")]
        {
            backends.push(Box::new(WindowsBackend::new()));
        }

        #[cfg(target_family = "unix")]
        {
            backends.push(Box::new(CupsBackend::new()));
        }

        // PDF 后端（用于预览，显示 "PDF 测试打印机"）
        match PdfBackend::with_downloads_dir() {
            Ok(pdf_backend) => backends.push(Box::new(pdf_backend)),
            Err(e) => eprintln!("⚠️  PDF 后端初始化失败: {}", e),
        }

        // MockBackend 已删除

        Ok(Self {
            backends,
            tspl_generator: TSPLGenerator::new(),
        })
    }
}
```

## 影响分析

| 功能 | 影响 |
|------|------|
| 打印机列表显示 | 显示系统打印机 + PDF 测试打印机 |
| 打印到真实打印机 | 无影响 |
| PDF 预览功能 | 无影响 |
| Mock 打印测试 | 移除（不再支持） |

## 测试计划

1. **编译测试**
   - `cargo build` 无错误
   - `cargo clippy` 无警告

2. **打印机列表测试**
   - macOS：运行 `lpstat -p`，对比应用中的打印机列表
   - Windows：检查"设备和打印机"，对比应用中的打印机列表
   - 验证显示 "PDF 测试打印机"
   - 验证不显示 "Mock Printer"、"Mock Printer 2"

3. **无打印机环境测试**
   - 在没有安装打印机的系统上运行
   - 验证打印机列表只显示 "PDF 测试打印机"
