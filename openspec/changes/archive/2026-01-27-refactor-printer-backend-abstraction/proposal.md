# 提案：重构打印机后端抽象层

## 为什么

当前打印机后端实现存在以下问题：
1. 各打印命令（如 `print_qsl`、`print_address`、`sf_print_waybill`）中使用魔法字符串 `"PDF 测试打印机"` 来判断打印机类型
2. 不同打印命令重复实现相同的打印机类型判断逻辑
3. `PrinterBackend` trait 缺少统一的图像打印接口，导致调用方需要了解底层实现细节
4. PDF 测试打印机的名称硬编码在多处，不利于维护

## 变更内容

### 1. 扩展 PrinterBackend trait
- 添加 `ImagePrintConfig` 结构体，定义打印配置（纸张尺寸、DPI）
- 添加 `owns_printer()` 方法，让后端声明它拥有哪些打印机
- 添加 `print_image()` 方法，提供统一的图像打印接口

### 2. 定义打印机名称常量
- 在 `PdfBackend` 中定义 `PDF_TEST_PRINTER_NAME` 常量
- 从 `backend/mod.rs` 重新导出该常量

### 3. 各后端实现新接口
- **PdfBackend**: `print_image()` 保存图像为 PNG 文件到下载目录
- **CupsBackend**: `print_image()` 调用 TSPL 生成器并通过 `send_raw()` 发送
- **WindowsBackend**: `print_image()` 调用 TSPL 生成器并通过 `send_raw()` 发送

### 4. 扩展 TSPLGenerator
- 添加 `generate_from_image()` 方法，直接从 `GrayImage` 生成 TSPL 指令

### 5. 添加统一打印接口
- 在 `PrinterState` 中添加 `print_image_to_printer()` 方法
- 该方法根据打印机名称自动路由到正确的后端

### 6. 更新调用方
- `sf_print_waybill` 使用新的统一接口
- `print_qsl` 和 `print_address` 使用常量替代魔法字符串

## 影响

### 受影响文件
- `src/printer/backend/mod.rs` - 添加 trait 方法和配置结构
- `src/printer/backend/pdf.rs` - 实现新接口，定义常量
- `src/printer/backend/cups.rs` - 实现新接口
- `src/printer/backend/windows.rs` - 实现新接口
- `src/printer/tspl.rs` - 添加 `generate_from_image()` 方法
- `src/commands/printer.rs` - 添加 `print_image_to_printer()` 方法，使用常量
- `src/commands/sf_express.rs` - 使用统一接口

### 兼容性
- 向后兼容：现有 API 保持不变
- 新增 API 供内部使用

## 验收标准

1. 所有打印命令不再使用魔法字符串判断打印机类型
2. PDF 测试打印机名称通过常量定义，只在一处维护
3. 新增打印机后端时，只需实现 `PrinterBackend` trait 的方法
4. 调用方只需调用 `print_image_to_printer()`，无需了解后端细节
5. 现有功能（QSL 打印、地址打印、顺丰面单打印）正常工作
