# 任务清单：打印日志增强

## Phase 1: 增强 PrinterBackend trait

- [x] 在 `src/printer/backend/mod.rs` 中定义 `PrintResult` 结构体
- [x] 更新 `PrinterBackend` trait 的 `send_raw` 方法签名
- [x] 确保 trait 变更不破坏现有编译

## Phase 2: CUPS 后端增强

- [x] 解析 `lp` 命令输出获取作业 ID
- [x] 增强日志记录，包含完整的 stdout/stderr
- [x] 返回 `PrintResult` 替代简单的 `Result<()>`

## Phase 3: Windows 后端增强

- [x] 捕获详细的 Win32 错误码和描述
- [x] 增强日志记录
- [x] 返回 `PrintResult` 替代简单的 `Result<()>`

## Phase 4: 更新打印命令

- [x] 更新 `src/commands/printer.rs` 中的 `print_qsl` 命令
- [x] 更新 `src/commands/sf_express.rs` 中的 `sf_print_waybill` 命令
- [x] 在日志中记录 `PrintResult` 的详细信息
- [x] 更新 PDF 后端的 trait 实现以匹配新签名

## 测试验证

- [ ] 测试 macOS 上的卡片打印日志
- [ ] 测试 macOS 上的面单打印日志
- [ ] 测试 Windows 上的打印日志（如果有环境）
- [ ] 验证错误情况下的日志输出
