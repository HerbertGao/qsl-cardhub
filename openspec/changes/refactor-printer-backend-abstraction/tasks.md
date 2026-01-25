# 任务列表：重构打印机后端抽象层

## 1. 扩展 PrinterBackend trait

- [x] 1.1 添加 `ImagePrintConfig` 结构体
  - 字段：`width_mm`、`height_mm`、`dpi`
  - 实现 `Default` trait

- [x] 1.2 添加 `owns_printer()` 方法到 trait
  - 签名：`fn owns_printer(&self, printer_name: &str) -> bool`

- [x] 1.3 添加 `print_image()` 方法到 trait
  - 签名：`fn print_image(&self, printer_name: &str, image: &GrayImage, config: &ImagePrintConfig) -> Result<PrintResult>`

## 2. 实现 PdfBackend

- [x] 2.1 定义 `PDF_TEST_PRINTER_NAME` 常量

- [x] 2.2 实现 `owns_printer()` 方法
  - 当 `printer_name == PDF_TEST_PRINTER_NAME` 时返回 `true`

- [x] 2.3 实现 `print_image()` 方法
  - 将图像保存为 PNG 文件到下载目录
  - 返回成功消息包含文件路径

- [x] 2.4 从 `backend/mod.rs` 导出常量

## 3. 实现 CupsBackend

- [x] 3.1 实现 `owns_printer()` 方法
  - 当 `printer_name != PDF_TEST_PRINTER_NAME` 时返回 `true`

- [x] 3.2 实现 `print_image()` 方法
  - 调用 `TSPLGenerator::generate_from_image()` 生成 TSPL
  - 调用 `send_raw()` 发送到打印机

## 4. 实现 WindowsBackend

- [x] 4.1 实现 `owns_printer()` 方法
  - 当 `printer_name != PDF_TEST_PRINTER_NAME` 时返回 `true`

- [x] 4.2 实现 `print_image()` 方法
  - 调用 `TSPLGenerator::generate_from_image()` 生成 TSPL
  - 调用 `send_raw()` 发送到打印机

## 5. 扩展 TSPLGenerator

- [x] 5.1 添加 `generate_from_image()` 方法
  - 签名：`fn generate_from_image(&self, image: &GrayImage, width_mm: f32, height_mm: f32) -> Result<Vec<u8>>`
  - 生成完整的 TSPL 指令（SIZE、GAP、DIRECTION、CLS、BITMAP、PRINT）

## 6. 更新 PrinterState

- [x] 6.1 添加 `print_image_to_printer()` 方法
  - 根据 `owns_printer()` 自动路由到正确的后端
  - 返回统一的结果消息

- [x] 6.2 使用 `PDF_TEST_PRINTER_NAME` 常量替代魔法字符串

## 7. 更新调用方

- [x] 7.1 更新 `sf_print_waybill`
  - 使用 `print_image_to_printer()` 统一接口
  - 移除打印机类型判断逻辑

- [x] 7.2 更新 `print_qsl` 和 `print_address`
  - 使用 `PDF_TEST_PRINTER_NAME` 常量替代魔法字符串

## 8. 验证

- [ ] 8.1 运行 `cargo check` 确保编译通过
- [ ] 8.2 测试 PDF 测试打印机功能
- [ ] 8.3 测试真实打印机功能（如有条件）

## 完成情况

| 阶段 | 状态 |
|------|------|
| 1. 扩展 trait | ✅ 完成 |
| 2. PdfBackend | ✅ 完成 |
| 3. CupsBackend | ✅ 完成 |
| 4. WindowsBackend | ✅ 完成 |
| 5. TSPLGenerator | ✅ 完成 |
| 6. PrinterState | ✅ 完成 |
| 7. 调用方更新 | ✅ 完成 |
| 8. 验证 | 🔄 进行中 |
