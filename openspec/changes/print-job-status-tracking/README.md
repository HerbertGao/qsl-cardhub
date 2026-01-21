# 打印任务状态跟踪

## 概述

添加打印任务状态跟踪功能，允许用户查询打印任务的执行状态（排队中、打印中、已完成、失败等），提供更好的用户反馈体验。

## 变更内容

**新增功能**：
- ✅ 打印任务返回任务标识符（job_id）
- ✅ 查询打印任务当前状态（Pending、Printing、Completed、Failed、Cancelled、Unknown）
- ✅ 查询最近的打印历史记录（内存缓存，最多 100 条）
- ✅ 前端显示打印任务状态和历史

**修改文件**：
- `src/printer/backend/mod.rs` - 扩展 `PrinterBackend` trait
- `src/printer/backend/windows.rs` - 实现 Windows 任务跟踪
- `src/printer/backend/cups.rs` - 实现 CUPS 任务跟踪
- `src/printer/backend/pdf.rs` - 实现 PDF 模拟跟踪
- `src/printer/job_tracker.rs`（新建）- 打印任务跟踪器
- `src/commands/printer.rs` - 修改命令返回值和新增查询 API
- `web/src/views/PrintView.vue` - 前端状态显示

## 技术方案

### 方案选择：查询系统打印队列

- **Windows**：使用 `GetJobW` API 查询任务状态
- **CUPS**：使用 `lpstat` 命令查询任务状态
- **PDF**：返回模拟 job_id，状态查询返回 Unknown

### 数据结构

```rust
pub enum PrintJobStatus {
    Pending,      // 排队中
    Printing,     // 打印中
    Completed,    // 已完成
    Failed { error: String },  // 失败
    Cancelled,    // 已取消
    Unknown,      // 未知
}

pub struct PrintJobInfo {
    pub job_id: String,
    pub printer_name: String,
    pub job_name: String,
    pub submitted_at: String,
    pub status: PrintJobStatus,
}
```

## 实施时间

约 6-7 小时

## 文档

- [提案](./proposal.md) - 详细的变更说明和技术方案
- [任务清单](./tasks.md) - 具体的实施步骤（5 个阶段，18 个任务）
- [规范](./specs/print-job-tracking/spec.md) - 打印任务跟踪需求规范（6 个需求）

## 验证

```bash
openspec-cn validate print-job-status-tracking --strict
```

## 依赖关系

- **前置依赖**：无
- **后续任务**：
  - v2.0: 打印任务取消功能
  - v2.0: 打印机物理状态查询（双向通信）
  - v2.0: 打印历史持久化存储

## 破坏性变更

⚠️ **警告**：此变更包含 API 破坏性修改

- `print_qsl` 命令返回值从 `Result<(), String>` 改为 `Result<PrintJobInfo, String>`
- 前端调用代码需要相应更新
- 建议在 v0.2 版本中实施
