# 规范：打印任务跟踪

## 新增需求

### 需求：打印任务必须返回任务标识符

打印 QSL 卡片时，系统必须返回打印任务的唯一标识符，以便后续查询任务状态。

#### 场景：Windows 平台打印返回任务 ID
- **当** 用户在 Windows 平台上打印 QSL 卡片
- **那么** 系统必须调用 `StartDocPrinterW` 并返回 job_id
- **并且** job_id 必须为非零正整数
- **并且** 返回的 `PrintJobInfo` 必须包含字符串格式的 job_id

#### 场景：CUPS 平台打印返回任务 ID
- **当** 用户在 macOS 或 Linux 平台上打印 QSL 卡片
- **那么** 系统必须调用 `lp` 命令发送打印数据
- **并且** 系统必须解析 `lp` 命令的标准输出
- **并且** 系统必须从输出中提取 job_id（格式："request id is Printer-123"）
- **并且** 返回的 `PrintJobInfo` 必须包含提取的 job_id 字符串

#### 场景：PDF 测试打印机返回模拟任务 ID
- **当** 用户使用 PDF 测试打印机打印
- **那么** 系统必须返回模拟的任务 ID（如 "pdf-<timestamp>"）
- **并且** 返回的 `PrintJobInfo` 必须包含模拟的 job_id

#### 场景：打印失败时不返回任务 ID
- **当** 打印命令执行失败（打印机离线、驱动错误等）
- **那么** 系统必须返回错误信息
- **并且** 不得返回 `PrintJobInfo` 结构

---

### 需求：系统必须支持查询打印任务状态

系统必须提供 API 查询指定打印任务的当前执行状态。

#### 场景：Windows 平台查询任务状态
- **当** 用户查询 Windows 平台的打印任务状态
- **那么** 系统必须使用 `GetJobW` API 查询任务信息
- **并且** 系统必须将 Windows 状态码映射到标准化的 `PrintJobStatus` 枚举
- **并且** 状态必须包括：Pending、Printing、Completed、Failed、Cancelled 之一

#### 场景：CUPS 平台查询任务状态
- **当** 用户查询 macOS 或 Linux 平台的打印任务状态
- **那么** 系统必须使用 `lpstat -W completed -o <job_id>` 命令查询
- **并且** 系统必须解析 `lpstat` 的输出
- **并且** 系统必须将 CUPS 状态映射到标准化的 `PrintJobStatus` 枚举
- **并且** 状态必须包括：Pending、Printing、Completed、Failed、Cancelled 之一

#### 场景：任务不存在时返回 Unknown 状态
- **当** 用户查询的任务 ID 不存在或已从队列中清除
- **那么** 系统必须返回 `PrintJobStatus::Unknown`
- **并且** 不得返回错误

#### 场景：查询 API 调用失败时的处理
- **当** 查询打印任务状态的系统调用失败（权限不足、服务不可用等）
- **那么** 系统必须记录错误日志
- **并且** 系统必须返回 `PrintJobStatus::Unknown`
- **并且** 不得中断应用运行

---

### 需求：系统必须缓存打印任务历史

系统必须在内存中维护最近的打印任务历史，以便用户查询。

#### 场景：记录打印任务到历史
- **当** 用户成功提交打印任务
- **那么** 系统必须将 `PrintJobInfo` 添加到历史记录中
- **并且** 历史记录必须按提交时间倒序排列（最新的在前）
- **并且** 如果历史记录超过 100 条，必须自动删除最旧的记录

#### 场景：查询打印历史
- **当** 用户查询打印历史
- **那么** 系统必须返回内存中缓存的打印任务列表
- **并且** 列表必须按提交时间倒序排列
- **并且** 如果指定了 limit 参数，必须只返回最近的 N 条记录

#### 场景：应用重启后历史清空
- **当** 应用重启
- **那么** 打印历史记录必须被清空
- **并且** 系统不得从文件或数据库加载历史记录

---

### 需求：打印任务状态枚举必须标准化

系统必须定义标准化的打印任务状态枚举，屏蔽平台差异。

#### 场景：状态枚举定义
- **当** 系统定义打印任务状态
- **那么** 必须包括以下状态：
  - `Pending`：任务在队列中等待
  - `Printing`：任务正在打印
  - `Completed`：任务已成功完成
  - `Failed { error: String }`：任务失败，包含错误信息
  - `Cancelled`：任务被取消
  - `Unknown`：无法查询到任务状态

#### 场景：Windows 状态码映射
- **当** 系统查询 Windows 打印任务状态
- **那么** 必须将 Windows 状态码映射如下：
  - `JOB_STATUS_PAUSED` → `Pending`
  - `JOB_STATUS_PRINTING` → `Printing`
  - `JOB_STATUS_PRINTED` → `Completed`
  - `JOB_STATUS_ERROR` → `Failed`
  - `JOB_STATUS_DELETED` → `Cancelled`
  - 其他 → `Unknown`

#### 场景：CUPS 状态字符串映射
- **当** 系统查询 CUPS 打印任务状态
- **那么** 必须将 CUPS 状态字符串映射如下：
  - "pending" → `Pending`
  - "processing" → `Printing`
  - "completed" → `Completed`
  - "aborted" → `Failed`
  - "canceled" → `Cancelled`
  - 其他 → `Unknown`

---

### 需求：前端必须显示打印任务状态

打印页面必须显示当前打印任务的执行状态，提供实时反馈。

#### 场景：打印成功后显示任务信息
- **当** 用户成功提交打印任务
- **那么** 前端必须显示任务信息：
  - 任务 ID
  - 打印机名称
  - 提交时间
  - 当前状态
- **并且** 必须提供"刷新状态"按钮

#### 场景：手动刷新任务状态
- **当** 用户点击"刷新状态"按钮
- **那么** 前端必须调用 `get_print_job_status` API
- **并且** 必须更新显示的任务状态
- **并且** 如果状态为 Failed，必须显示错误信息

#### 场景：任务状态图标显示
- **当** 前端显示任务状态
- **那么** 必须根据状态显示不同的图标和颜色：
  - `Pending`：黄色，时钟图标
  - `Printing`：蓝色，打印机图标
  - `Completed`：绿色，勾选图标
  - `Failed`：红色，错误图标
  - `Cancelled`：灰色，取消图标
  - `Unknown`：灰色，问号图标

---

### 需求：打印机后端接口必须支持状态查询

`PrinterBackend` trait 必须提供查询打印任务状态的方法。

#### 场景：后端实现状态查询方法
- **当** 打印机后端实现 `PrinterBackend` trait
- **那么** 必须实现 `get_job_status(&self, printer_name: &str, job_id: &str) -> Result<PrintJobStatus>` 方法
- **并且** 如果平台不支持状态查询，必须返回 `PrintJobStatus::Unknown`
- **并且** 方法不得因查询失败而 panic

#### 场景：后端返回任务 ID
- **当** 打印机后端的 `send_raw` 方法被调用
- **那么** 必须返回 `Result<String, Error>`，其中 String 为 job_id
- **并且** job_id 必须为平台相关的唯一标识符
- **并且** 如果无法获取 job_id，必须返回错误

---

## 相关规范

- **printing**（待创建）：打印功能的核心规范
- **configuration-management**：配置管理功能，打印机选择依赖此规范

## 技术约束

- **状态查询依赖操作系统**：Windows 需要 `GetJobW` API，CUPS 需要 `lpstat` 命令
- **任务 ID 格式平台相关**：Windows 为整数，CUPS 为字符串（如 "Printer-123"）
- **状态更新非实时**：依赖手动刷新或轮询，不使用 WebSocket 推送
- **历史记录非持久化**：仅在内存中缓存，应用重启后清空

## 向后兼容性

- ⚠️ **破坏性变更**：`print_qsl` 命令的返回值从 `Result<(), String>` 改为 `Result<PrintJobInfo, String>`
- ✅ **前端需要更新**：需要修改前端调用代码以处理新的返回值
- ✅ **API 版本**：建议在 v0.2 中实施此变更

## 实现细节

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
    /// 提交时间（ISO 8601 格式）
    pub submitted_at: String,
    /// 当前状态
    pub status: PrintJobStatus,
}

/// 打印任务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum PrintJobStatus {
    Pending,
    Printing,
    Completed,
    Failed { error: String },
    Cancelled,
    Unknown,
}
```

### PrinterBackend Trait 扩展

```rust
pub trait PrinterBackend: Send + Sync {
    fn name(&self) -> &str;
    fn list_printers(&self) -> Result<Vec<String>>;

    /// 发送原始数据到打印机，返回任务 ID
    fn send_raw(&self, printer_name: &str, data: &[u8]) -> Result<String>;

    /// 查询打印任务状态
    fn get_job_status(&self, printer_name: &str, job_id: &str) -> Result<PrintJobStatus> {
        // 默认实现返回 Unknown
        Ok(PrintJobStatus::Unknown)
    }
}
```

### Tauri Commands

```rust
#[tauri::command]
pub async fn print_qsl(
    printer_name: String,
    request: PrintRequest,
    state: State<'_, PrinterState>,
) -> Result<PrintJobInfo, String>

#[tauri::command]
pub async fn get_print_job_status(
    job_id: String,
    state: State<'_, PrinterState>,
) -> Result<PrintJobStatus, String>

#[tauri::command]
pub async fn get_print_job_history(
    limit: Option<usize>,
    state: State<'_, PrinterState>,
) -> Result<Vec<PrintJobInfo>, String>
```
