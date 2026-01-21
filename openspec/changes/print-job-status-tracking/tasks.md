# 实施任务清单

本文档列出了打印任务状态跟踪功能的详细实施任务。

---

## 阶段 1：后端基础架构

### 任务 1.1：扩展 PrinterBackend trait

**描述**：在 `PrinterBackend` trait 中添加任务状态查询方法和修改 `send_raw` 返回值。

**步骤**：
1. 打开 `src/printer/backend/mod.rs`
2. 定义 `PrintJobStatus` 枚举（Pending、Printing、Completed、Failed、Cancelled、Unknown）
3. 定义 `PrintJobInfo` 结构体（job_id、printer_name、job_name、submitted_at、status）
4. 修改 `PrinterBackend` trait：
   - 将 `send_raw` 返回值从 `Result<()>` 改为 `Result<String>`（返回 job_id）
   - 添加 `get_job_status(&self, printer_name: &str, job_id: &str) -> Result<PrintJobStatus>` 方法
   - 为 `get_job_status` 提供默认实现，返回 `PrintJobStatus::Unknown`
5. 保存文件

**验证**：
- 代码编译通过（其他文件会有编译错误，后续任务修复）
- `PrintJobStatus` 和 `PrintJobInfo` 结构体可以序列化/反序列化

**依赖**：无

**估算时间**：30 分钟

---

### 任务 1.2：实现 Windows 后端的任务跟踪

**描述**：修改 `WindowsBackend` 以返回 job_id 并实现状态查询。

**步骤**：
1. 打开 `src/printer/backend/windows.rs`
2. 修改 `send_raw` 方法：
   - 保存 `StartDocPrinterW` 返回的 `job_id`（u32 类型）
   - 在函数最后返回 `Ok(job_id.to_string())`
3. 实现 `get_job_status` 方法：
   - 调用 `GetJobW` API 获取任务信息
   - 将 Windows 状态码映射到 `PrintJobStatus`：
     - `JOB_STATUS_PAUSED` → `Pending`
     - `JOB_STATUS_PRINTING` → `Printing`
     - `JOB_STATUS_PRINTED` → `Completed`
     - `JOB_STATUS_ERROR` → `Failed { error: "打印错误" }`
     - `JOB_STATUS_DELETED` → `Cancelled`
   - 如果查询失败，返回 `PrintJobStatus::Unknown`
4. 添加必要的 Windows API 导入（`GetJobW`、`JOB_INFO_1W` 等）
5. 保存文件

**验证**：
- 代码编译通过
- Windows 平台打印返回 job_id
- 可以查询任务状态

**依赖**：任务 1.1

**估算时间**：1 小时

---

### 任务 1.3：实现 CUPS 后端的任务跟踪

**描述**：修改 `CupsBackend` 以返回 job_id 并实现状态查询。

**步骤**：
1. 打开 `src/printer/backend/cups.rs`
2. 修改 `send_raw` 方法：
   - 捕获 `lp` 命令的标准输出
   - 解析输出获取 job_id（格式："request id is Printer-123 (1 file(s))"）
   - 使用正则表达式或字符串解析提取 "Printer-123"
   - 返回 `Ok(job_id)`
3. 实现 `get_job_status` 方法：
   - 执行 `lpstat -W completed -o <job_id>` 命令
   - 解析输出获取任务状态
   - 将 CUPS 状态映射到 `PrintJobStatus`：
     - "pending" → `Pending`
     - "processing" → `Printing`
     - "completed" → `Completed`
     - "aborted" → `Failed { error: "打印中止" }`
     - "canceled" → `Cancelled`
   - 如果任务不存在或查询失败，返回 `PrintJobStatus::Unknown`
4. 保存文件

**验证**：
- 代码编译通过
- macOS/Linux 平台打印返回 job_id
- 可以查询任务状态

**依赖**：任务 1.1

**估算时间**：1 小时

---

### 任务 1.4：实现 PDF 后端的任务跟踪

**描述**：修改 `PdfBackend` 以返回模拟 job_id。

**步骤**：
1. 打开 `src/printer/backend/pdf.rs`
2. 修改 `send_raw` 方法（如果存在）：
   - 生成模拟 job_id：`format!("pdf-{}", chrono::Utc::now().timestamp())`
   - 返回 `Ok(job_id)`
3. 实现 `get_job_status` 方法：
   - 始终返回 `PrintJobStatus::Unknown`
4. 保存文件

**验证**：
- 代码编译通过
- PDF 测试打印机返回模拟 job_id

**依赖**：任务 1.1

**估算时间**：15 分钟

---

## 阶段 2：打印任务跟踪器

### 任务 2.1：创建打印任务跟踪器模块

**描述**：创建 `JobTracker` 结构体，在内存中维护打印任务历史。

**步骤**：
1. 创建新文件 `src/printer/job_tracker.rs`
2. 定义 `JobTracker` 结构体：
   ```rust
   pub struct JobTracker {
       /// 打印任务历史（最多 100 条）
       history: VecDeque<PrintJobInfo>,
       /// 最大历史记录数
       max_history_size: usize,
   }
   ```
3. 实现 `JobTracker` 方法：
   - `new() -> Self`：创建新的跟踪器
   - `add_job(&mut self, job: PrintJobInfo)`：添加任务到历史
   - `get_history(&self, limit: Option<usize>) -> Vec<PrintJobInfo>`：获取历史记录
   - `get_job(&self, job_id: &str) -> Option<&PrintJobInfo>`：根据 job_id 查找任务
4. 在 `src/printer/mod.rs` 中添加 `pub mod job_tracker;`
5. 保存文件

**验证**：
- 代码编译通过
- 可以添加和查询任务历史
- 历史记录数量不超过 100 条

**依赖**：任务 1.1

**估算时间**：30 分钟

---

### 任务 2.2：将 JobTracker 集成到 PrinterState

**描述**：在 `PrinterState` 中添加 `JobTracker` 实例。

**步骤**：
1. 打开 `src/commands/printer.rs`
2. 在 `PrinterState` 结构体中添加字段：
   ```rust
   pub job_tracker: Arc<Mutex<JobTracker>>,
   ```
3. 在 `PrinterState::new()` 中初始化 `JobTracker`
4. 保存文件

**验证**：
- 代码编译通过
- `PrinterState` 包含 `JobTracker` 实例

**依赖**：任务 2.1

**估算时间**：15 分钟

---

## 阶段 3：Tauri Commands 修改

### 任务 3.1：修改 print_qsl 命令

**描述**：修改 `print_qsl` 命令返回 `PrintJobInfo`。

**步骤**：
1. 打开 `src/commands/printer.rs`
2. 修改 `print_qsl` 函数签名：
   - 返回值从 `Result<(), String>` 改为 `Result<PrintJobInfo, String>`
3. 修改打印逻辑：
   - 调用 `system_backend.send_raw()` 获取 `job_id`
   - 构建 `PrintJobInfo` 结构体：
     ```rust
     let job_info = PrintJobInfo {
         job_id,
         printer_name: printer_name.clone(),
         job_name: "QSL Card".to_string(),
         submitted_at: chrono::Local::now().to_rfc3339(),
         status: PrintJobStatus::Pending,
     };
     ```
   - 调用 `state.job_tracker.lock().unwrap().add_job(job_info.clone())`
   - 返回 `Ok(job_info)`
4. 保存文件

**验证**：
- 代码编译通过
- 打印成功返回 `PrintJobInfo`
- 任务被记录到历史

**依赖**：任务 2.2、任务 1.2/1.3/1.4

**估算时间**：30 分钟

---

### 任务 3.2：新增 get_print_job_status 命令

**描述**：创建查询打印任务状态的 Tauri 命令。

**步骤**：
1. 在 `src/commands/printer.rs` 中添加新函数：
   ```rust
   #[tauri::command]
   pub async fn get_print_job_status(
       printer_name: String,
       job_id: String,
       state: State<'_, PrinterState>,
   ) -> Result<PrintJobStatus, String>
   ```
2. 实现逻辑：
   - 获取 `system_backend` 锁
   - 调用 `system_backend.get_job_status(&printer_name, &job_id)`
   - 返回状态
3. 在 `src/main.rs` 中注册命令：
   ```rust
   .invoke_handler(tauri::generate_handler![
       // ... 现有命令
       get_print_job_status,
   ])
   ```
4. 保存文件

**验证**：
- 代码编译通过
- 前端可以调用命令查询状态
- 返回正确的任务状态

**依赖**：任务 3.1

**估算时间**：20 分钟

---

### 任务 3.3：新增 get_print_job_history 命令

**描述**：创建查询打印历史的 Tauri 命令。

**步骤**：
1. 在 `src/commands/printer.rs` 中添加新函数：
   ```rust
   #[tauri::command]
   pub async fn get_print_job_history(
       limit: Option<usize>,
       state: State<'_, PrinterState>,
   ) -> Result<Vec<PrintJobInfo>, String>
   ```
2. 实现逻辑：
   - 获取 `job_tracker` 锁
   - 调用 `job_tracker.get_history(limit)`
   - 返回历史记录列表
3. 在 `src/main.rs` 中注册命令
4. 保存文件

**验证**：
- 代码编译通过
- 前端可以调用命令查询历史
- 返回正确的历史记录列表

**依赖**：任务 3.1

**估算时间**：15 分钟

---

## 阶段 4：前端实现

### 任务 4.1：修改打印页面显示任务状态

**描述**：在 `PrintView.vue` 中显示打印任务状态。

**步骤**：
1. 打开 `web/src/views/PrintView.vue`
2. 添加响应式状态：
   ```javascript
   const lastPrintJob = ref(null)  // PrintJobInfo
   const jobStatus = ref(null)     // PrintJobStatus
   ```
3. 修改 `handlePrint` 函数：
   - 调用 `invoke('print_qsl', ...)` 获取 `PrintJobInfo`
   - 保存到 `lastPrintJob.value`
   - 初始化 `jobStatus.value = lastPrintJob.value.status`
4. 在模板中添加任务状态显示区域：
   ```vue
   <el-card v-if="lastPrintJob" style="margin-top: 20px">
     <template #header>
       <div style="display: flex; justify-content: space-between; align-items: center">
         <span>打印任务状态</span>
         <el-button @click="refreshJobStatus" size="small">刷新状态</el-button>
       </div>
     </template>
     <el-descriptions :column="2">
       <el-descriptions-item label="任务 ID">{{ lastPrintJob.job_id }}</el-descriptions-item>
       <el-descriptions-item label="打印机">{{ lastPrintJob.printer_name }}</el-descriptions-item>
       <el-descriptions-item label="提交时间">{{ formatTime(lastPrintJob.submitted_at) }}</el-descriptions-item>
       <el-descriptions-item label="状态">
         <el-tag :type="getStatusTagType(jobStatus)">
           {{ getStatusText(jobStatus) }}
         </el-tag>
       </el-descriptions-item>
     </el-descriptions>
   </el-card>
   ```
5. 保存文件

**验证**：
- 打印成功后显示任务信息
- 任务 ID、打印机名称、提交时间正确显示
- 初始状态为 "Pending"

**依赖**：任务 3.1

**估算时间**：30 分钟

---

### 任务 4.2：实现刷新任务状态功能

**描述**：添加手动刷新任务状态的功能。

**步骤**：
1. 在 `PrintView.vue` 中添加 `refreshJobStatus` 函数：
   ```javascript
   const refreshJobStatus = async () => {
     if (!lastPrintJob.value) return

     try {
       const status = await invoke('get_print_job_status', {
         printerName: lastPrintJob.value.printer_name,
         jobId: lastPrintJob.value.job_id
       })
       jobStatus.value = status
       ElMessage.success('状态已更新')
     } catch (error) {
       ElMessage.error(`查询状态失败: ${error}`)
     }
   }
   ```
2. 添加状态显示辅助函数：
   ```javascript
   const getStatusTagType = (status) => {
     if (!status) return ''
     const map = {
       Pending: 'warning',
       Printing: 'primary',
       Completed: 'success',
       Failed: 'danger',
       Cancelled: 'info',
       Unknown: 'info'
     }
     return map[status.type] || ''
   }

   const getStatusText = (status) => {
     if (!status) return ''
     const map = {
       Pending: '排队中',
       Printing: '打印中',
       Completed: '已完成',
       Failed: '失败',
       Cancelled: '已取消',
       Unknown: '未知'
     }
     return map[status.type] || '未知'
   }
   ```
3. 保存文件

**验证**：
- 点击"刷新状态"按钮可以更新状态
- 状态变化时显示正确的颜色和文本
- 错误情况正确处理

**依赖**：任务 4.1、任务 3.2

**估算时间**：20 分钟

---

### 任务 4.3：添加打印历史查看（可选）

**描述**：在打印页面添加查看打印历史的功能。

**步骤**：
1. 在 `PrintView.vue` 中添加"查看历史"按钮
2. 添加历史记录弹窗：
   ```vue
   <el-dialog v-model="showHistory" title="打印历史">
     <el-table :data="printHistory">
       <el-table-column prop="job_id" label="任务 ID" />
       <el-table-column prop="printer_name" label="打印机" />
       <el-table-column prop="submitted_at" label="提交时间" />
       <el-table-column prop="status.type" label="状态" />
     </el-table>
   </el-dialog>
   ```
3. 添加加载历史的函数：
   ```javascript
   const loadHistory = async () => {
     try {
       printHistory.value = await invoke('get_print_job_history', { limit: 50 })
       showHistory.value = true
     } catch (error) {
       ElMessage.error(`加载历史失败: ${error}`)
     }
   }
   ```
4. 保存文件

**验证**：
- 可以查看最近的打印历史
- 历史记录按时间倒序排列
- 显示正确的任务信息

**依赖**：任务 4.2、任务 3.3

**估算时间**：30 分钟

**注意**：此任务为可选，可根据用户需求决定是否实施。

---

## 阶段 5：测试和验证

### 任务 5.1：Windows 平台测试

**描述**：在 Windows 平台测试所有功能。

**步骤**：
1. 编译应用：`cargo build`
2. 运行应用：`cargo run`
3. 测试打印功能：
   - 打印 QSL 卡片，验证返回 job_id
   - 查看任务状态，验证状态显示正确
   - 刷新状态，验证状态更新
4. 测试不同状态：
   - Pending：任务提交后立即查询
   - Printing：打印机正在工作时查询
   - Completed：打印完成后查询
5. 测试错误情况：
   - 打印机离线
   - 查询不存在的 job_id
6. 记录测试结果

**验证**：
- 所有功能正常工作
- 状态显示正确
- 错误情况正确处理

**依赖**：所有前序任务

**估算时间**：1 小时

---

### 任务 5.2：macOS/Linux 平台测试

**描述**：在 macOS 或 Linux 平台测试所有功能。

**步骤**：
1. 编译应用
2. 运行应用
3. 测试打印功能和状态查询（同 Windows）
4. 验证 CUPS 的 job_id 解析正确
5. 验证 `lpstat` 输出解析正确
6. 测试不同语言环境（中文/英文）
7. 记录测试结果

**验证**：
- 所有功能正常工作
- job_id 解析正确
- 状态查询正常

**依赖**：所有前序任务

**估算时间**：1 小时

---

### 任务 5.3：PDF 测试打印机测试

**描述**：测试 PDF 测试打印机的任务跟踪功能。

**步骤**：
1. 使用 PDF 测试打印机打印
2. 验证返回模拟 job_id
3. 查询状态，验证返回 Unknown
4. 确认不影响正常打印功能

**验证**：
- PDF 打印正常工作
- 返回模拟 job_id
- 状态查询返回 Unknown

**依赖**：所有前序任务

**估算时间**：15 分钟

---

## 总计

**总估算时间**：约 6-7 小时

**关键里程碑**：
1. 阶段 1-2：后端基础架构完成（3 小时）
2. 阶段 3：Tauri Commands 完成（1 小时）
3. 阶段 4：前端实现完成（1.5 小时）
4. 阶段 5：测试和验证通过（2.5 小时）

**并行任务建议**：
- 任务 1.2、1.3、1.4 可以并行实施（不同平台的后端）
- 任务 3.2、3.3 可以并行实施（独立的查询命令）
- 任务 5.1、5.2 可以在不同平台上并行测试

**风险提示**：
- Windows API 调用可能需要额外调试时间
- CUPS 输出格式解析可能因系统版本而异，需要充分测试
- 前端状态刷新逻辑需要仔细设计，避免频繁轮询
