# 任务列表：修正打印机列表获取

## 任务清单

### 1. 删除 MockBackend 文件
- **文件**：`src/printer/backend/mock.rs`
- **操作**：删除文件
- **验证**：文件不存在
- **状态**：[x] 已完成

### 2. 从 mod.rs 移除 MockBackend 导出
- **文件**：`src/printer/backend/mod.rs`
- **操作**：
  - 删除 `pub mod mock;`
  - 删除 `pub use mock::MockBackend;`
- **验证**：编译通过
- **状态**：[x] 已完成

### 3. 从 PrinterManager 移除 MockBackend
- **文件**：`src/printer/manager.rs`
- **操作**：
  - 删除 `use super::backend::MockBackend;` 导入
  - 删除 `backends.push(Box::new(MockBackend::new(output_dir)?));` 代码
  - 移除不再需要的 `output_dir` 参数
- **验证**：编译通过
- **状态**：[x] 已完成

### 4. 更新调用 PrinterManager 的代码
- **文件**：`src/main.rs`
- **操作**：移除 `output_dir` 参数
- **状态**：[x] 已完成

### 5. 构建验证
- **命令**：`cargo build`
- **验证**：编译成功，无错误
- **状态**：[x] 已完成

### 6. 运行 Clippy 检查
- **命令**：`cargo clippy`
- **验证**：无新增警告
- **状态**：[x] 已完成

### 7. 修复 CupsBackend 中文本地化问题
- **问题**：`lpstat -p` 在中文系统上输出格式为 `打印机PrinterName闲置...`，原代码只解析英文格式
- **文件**：`src/printer/backend/cups.rs`
- **操作**：修改 `list_printers()` 方法，支持中文和英文两种输出格式
- **状态**：[x] 已完成

### 8. 功能测试（手动）
- **测试点**：
  - [x] 启动应用，进入配置管理页面
  - [x] 验证打印机下拉列表显示系统打印机
  - [x] 验证显示 "PDF 测试打印机"
  - [x] 验证不显示 "Mock Printer"、"Mock Printer 2"
  - [x] 验证刷新按钮正常工作
  - [x] 验证选择打印机后可正常保存配置
- **状态**：[x] 已完成

## 变更摘要

| 文件 | 操作 |
|------|------|
| `src/printer/backend/mock.rs` | 已删除 |
| `src/printer/backend/mod.rs` | 移除 MockBackend 导出 |
| `src/printer/manager.rs` | 移除 MockBackend 相关代码，简化 `new()` 签名 |
| `src/main.rs` | 更新 `PrinterManager::new()` 调用 |
| `src/printer/backend/cups.rs` | 修复 `list_printers()` 支持中文本地化输出 |

## 完成状态

- [x] 代码变更已完成
- [x] 构建通过
- [x] Clippy 检查通过
- [x] 中文本地化问题已修复
- [x] 手动功能测试通过
