# 提案：打印任务状态跟踪

**状态**：📋 提案中
**最后更新**：2026-01-21

---

## 为什么

当前应用在打印 QSL 卡片时存在以下问题：

1. **缺乏打印反馈**：调用 `print_qsl` 后仅返回成功/失败，无法知道打印任务的实际执行状态
2. **用户体验不佳**：用户不知道任务是否在队列中、是否正在打印、是否已完成
3. **错误排查困难**：当打印失败时，无法获取详细的错误信息（打印机离线、缺纸、卡纸等）
4. **无法追踪历史**：无法查询过去的打印任务状态

**当前实现**：

**Windows**（`src/printer/backend/windows.rs:94-154`）：
```rust
fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<()> {
    // 使用 Win32 API 发送打印数据
    // 返回 Ok(()) 或 Err，无法获取任务 ID 或状态
}
```

**CUPS**（`src/printer/backend/cups.rs:83-116`）：
```rust
fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<()> {
    // 使用 lp 命令发送数据
    // 标准输出包含 job ID，但当前未解析
    // "request id is Printer-123 (1 file(s))"
}
```

**前端调用**（`web/src/views/PrintView.vue`）：
```javascript
await invoke('print_qsl', { printerName, request })
// 仅知道成功或失败，无法获取任务状态
```

**用户期望**：
- 知道打印任务是否已提交到队列
- 查看任务当前状态（排队中、打印中、已完成、失败）
- 获取详细的错误信息（如打印机离线、缺纸等）
- 查看打印历史记录

## 变更内容

添加打印任务状态跟踪功能，允许应用查询和显示打印任务的执行状态。

### 具体变更

1. **修改 `print_qsl` 命令返回值**
   - 从 `Result<(), String>` 改为 `Result<PrintJobInfo, String>`
   - 返回打印任务信息（job_id、提交时间、打印机名称等）

2. **新增打印任务状态查询 API**
   - `get_print_job_status(job_id: String) -> Result<PrintJobStatus, String>`
   - 返回任务当前状态（pending、printing、completed、failed、cancelled）

3. **新增打印历史查询 API**
   - `get_print_job_history(limit: Option<usize>) -> Result<Vec<PrintJobInfo>, String>`
   - 返回最近的打印任务列表

4. **扩展打印机后端接口**
   - 在 `PrinterBackend` trait 添加 `get_job_status()` 方法
   - Windows: 使用 `GetJobW` API 查询任务状态
   - CUPS: 使用 `lpstat` 命令查询任务状态

5. **前端显示打印状态**
   - 在打印页面显示当前打印任务状态
   - 添加打印历史查看功能（可选）

### 技术方案

#### 方案 A：查询系统打印队列（推荐）

**优点**：
- ✅ 利用操作系统的打印队列机制
- ✅ 可以获取详细的任务状态
- ✅ 实现相对简单
- ✅ 支持所有打印机

**缺点**：
- ⚠️ 状态更新依赖轮询
- ⚠️ 无法获取打印机物理状态（卡纸、缺纸等）
- ⚠️ 任务完成后可能很快从队列中清除

**实现细节**：

**Windows**：
```rust
use windows::Win32::Graphics::Printing::{GetJobW, JOB_INFO_1W};

pub fn get_job_status(&self, printer_name: &str, job_id: u32) -> Result<JobStatus> {
    // 使用 GetJobW API 查询任务状态
    // 状态码：JOB_STATUS_PAUSED, JOB_STATUS_PRINTING, JOB_STATUS_PRINTED 等
}
```

**CUPS**：
```rust
pub fn get_job_status(&self, job_id: &str) -> Result<JobStatus> {
    // lpstat -W completed -o <job_id>
    // 解析输出获取状态：pending, processing, completed, canceled, aborted
}
```

#### 方案 B：打印机双向通信（高级方案，暂不实施）

**优点**：
- ✅ 可以获取打印机物理状态
- ✅ 状态更新实时

**缺点**：
- ❌ 需要打印机支持双向通信
- ❌ 实现复杂（需要直接访问端口）
- ❌ 不是所有打印机都支持
- ❌ Windows 下访问原始端口需要管理员权限

**暂不实施**，留待 v2.0 考虑。

### 数据结构

```rust
/// 打印任务信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJobInfo {
    /// 任务 ID（平台相关格式）
    pub job_id: String,
    /// 打印机名称
    pub printer_name: String,
    /// 任务名称
    pub job_name: String,
    /// 提交时间
    pub submitted_at: String,
    /// 当前状态
    pub status: PrintJobStatus,
}

/// 打印任务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrintJobStatus {
    /// 排队中
    Pending,
    /// 打印中
    Printing,
    /// 已完成
    Completed,
    /// 失败
    Failed { error: String },
    /// 已取消
    Cancelled,
    /// 未知（无法查询到）
    Unknown,
}
```

## 范围

### 包含的功能

1. ✅ **修改 `print_qsl` 返回值**
   - 返回打印任务信息（job_id、打印机名称、提交时间）
   - 解析平台返回的 job ID

2. ✅ **新增打印任务状态查询 API**
   - Windows: 使用 `GetJobW` 查询任务状态
   - CUPS: 使用 `lpstat` 查询任务状态
   - 返回标准化的任务状态枚举

3. ✅ **新增打印历史查询 API**
   - 在内存中缓存最近的打印任务（最多 100 条）
   - 支持查询最近 N 条打印记录

4. ✅ **前端状态显示**
   - 打印后显示任务状态
   - 提供"查看状态"按钮手动刷新

### 不包含的功能

1. ❌ **打印机物理状态查询**：不查询缺纸、卡纸、墨水等硬件状态
2. ❌ **打印任务取消功能**：v1 不实现任务取消
3. ❌ **打印队列管理**：不实现查看所有队列任务、暂停/恢复任务
4. ❌ **持久化存储**：打印历史仅在内存中，应用重启后清空
5. ❌ **实时推送**：不实现 WebSocket 推送，依赖手动刷新或轮询
6. ❌ **打印统计**：不统计打印数量、成功率等

## 影响

### 修改文件

**后端**：
- `src/printer/backend/mod.rs` - 修改 `PrinterBackend` trait，添加 `get_job_status()` 方法
- `src/printer/backend/windows.rs` - 实现 Windows 任务状态查询
- `src/printer/backend/cups.rs` - 实现 CUPS 任务状态查询，解析 `lp` 命令返回的 job ID
- `src/commands/printer.rs` - 修改 `print_qsl` 返回值，新增查询 API
- `src/printer/job_tracker.rs`（新建）- 打印任务跟踪器，管理打印历史

**前端**：
- `web/src/views/PrintView.vue` - 显示打印任务状态，添加刷新按钮

### 新增规范

- `openspec/changes/print-job-status-tracking/specs/print-job-tracking/spec.md`

### 受影响规范

- 无（新增功能，不修改现有规范）

### 用户体验影响

- ✅ **改进反馈**：用户可以看到打印任务的执行状态
- ✅ **减少焦虑**：知道任务是否已提交、是否正在处理
- ✅ **错误诊断**：获取详细的错误信息
- ⚠️ **学习成本**：需要理解打印任务状态的含义

## 验收标准

### 功能验收

- [ ] 调用 `print_qsl` 返回打印任务信息（job_id、打印机名称、提交时间）
- [ ] Windows 平台可以查询打印任务状态（pending、printing、completed 等）
- [ ] macOS/Linux 平台可以查询打印任务状态
- [ ] 前端可以显示当前打印任务的状态
- [ ] 可以查询最近的打印历史（最多 100 条）

### API 验收

- [ ] `print_qsl` 返回 `Result<PrintJobInfo, String>`
- [ ] `get_print_job_status(job_id)` 返回任务状态
- [ ] `get_print_job_history(limit)` 返回打印历史列表
- [ ] 错误情况正确处理（打印机不存在、job_id 无效等）

### 跨平台验收

- [ ] Windows 上打印任务状态查询正常工作
- [ ] macOS 上打印任务状态查询正常工作
- [ ] PDF 测试打印机支持状态查询（返回 Unknown）

### 用户界面验收

- [ ] 打印成功后显示任务信息
- [ ] 可以手动刷新任务状态
- [ ] 任务失败时显示错误信息

## 风险和缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 任务完成后很快从队列中清除 | 中 | 在内存中缓存打印历史，保存最后已知状态 |
| 不同平台的 job ID 格式不一致 | 低 | 使用统一的字符串格式，平台特定解析 |
| 查询 API 性能开销 | 低 | 前端避免频繁轮询，使用手动刷新 |
| Windows API 调用失败 | 中 | 返回 Unknown 状态，不影响打印功能 |
| CUPS 解析输出格式变化 | 中 | 支持多种格式，失败时返回 Unknown |

## 实施计划

### 阶段 1：后端实现（核心）

1. **修改 `PrinterBackend` trait**（30 分钟）
   - 添加 `get_job_status()` 方法（可选实现）
   - 定义返回类型 `PrintJobStatus`

2. **修改 `send_raw` 返回值**（30 分钟）
   - Windows: 从 `StartDocPrinterW` 获取 job_id
   - CUPS: 解析 `lp` 命令输出获取 job_id
   - 返回 job_id 而不是 `()`

3. **实现 Windows 状态查询**（1 小时）
   - 使用 `GetJobW` API
   - 映射 Windows 状态码到 `PrintJobStatus`

4. **实现 CUPS 状态查询**（1 小时）
   - 使用 `lpstat -W completed -o <job_id>`
   - 解析输出获取状态

5. **添加打印任务跟踪器**（1 小时）
   - 在内存中维护打印历史
   - 提供查询接口

### 阶段 2：Tauri Commands（30 分钟）

1. **修改 `print_qsl` 命令**
   - 返回 `PrintJobInfo`
   - 记录到任务跟踪器

2. **新增查询命令**
   - `get_print_job_status`
   - `get_print_job_history`

### 阶段 3：前端实现（1 小时）

1. **修改打印页面**
   - 显示打印任务状态
   - 添加刷新按钮

2. **测试和调试**
   - 测试各种状态
   - 测试错误处理

**总计**：约 5-6 小时

## 依赖关系

- **前置依赖**：无（独立功能）
- **并行任务**：可独立实施
- **后续任务**：
  - v2.0: 打印任务取消功能
  - v2.0: 打印机物理状态查询（双向通信）
  - v2.0: 打印历史持久化存储

## 备选方案

### 方案 A：查询系统打印队列（推荐）✅

如提案所述，使用操作系统的打印队列 API。

**优点**：实现简单，支持所有打印机
**缺点**：无法获取物理状态

### 方案 B：仅记录提交状态（最简方案）

```rust
// 仅记录任务已提交，不查询实际状态
pub struct PrintJobInfo {
    pub submitted: bool,
    pub submitted_at: String,
}
```

**优点**：实现极其简单
**缺点**：用户体验提升有限，无法知道实际执行状态

### 方案 C：打印机双向通信（高级方案）

使用 TSPL 查询命令 `~!S` 查询打印机状态。

**优点**：可以获取物理状态
**缺点**：实现复杂，不是所有打印机都支持

**推荐**：方案 A（查询系统打印队列）
