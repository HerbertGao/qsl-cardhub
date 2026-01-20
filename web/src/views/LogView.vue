<template>
  <div class="page-content">
    <div class="page-header">
      <h2>日志查看</h2>
      <div>
        <el-button @click="refreshLogs" :loading="loading" :icon="Refresh">刷新</el-button>
        <el-button @click="clearLogs" :icon="Delete">清空</el-button>
        <el-button @click="exportLogs" :icon="Download">导出</el-button>
      </div>
    </div>

    <!-- 过滤和设置 -->
    <el-card style="margin-bottom: 20px">
      <el-form :inline="true" :model="filterForm">
        <el-form-item label="日志级别">
          <el-select v-model="filterForm.level" placeholder="全部级别" clearable style="width: 150px">
            <el-option label="全部" value="" />
            <el-option label="DEBUG" value="DEBUG" />
            <el-option label="INFO" value="INFO" />
            <el-option label="WARNING" value="WARNING" />
            <el-option label="ERROR" value="ERROR" />
            <el-option label="CRITICAL" value="CRITICAL" />
          </el-select>
        </el-form-item>
        <el-form-item label="显示条数">
          <el-input-number v-model="filterForm.limit" :min="100" :max="2000" :step="100" style="width: 120px" />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="applyFilter">应用过滤</el-button>
        </el-form-item>
        <el-form-item v-if="logFilePath">
          <el-text type="info" size="small">日志文件: {{ logFilePath }}</el-text>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- 日志列表 -->
    <el-card>
      <template #header>
        <div style="display: flex; justify-content: space-between; align-items: center">
          <span>日志记录 (共 {{ logs.length }} 条)</span>
          <el-switch v-model="autoRefresh" active-text="自动刷新" @change="handleAutoRefreshChange" />
        </div>
      </template>

      <div class="log-container" ref="logContainer">
        <div
          v-for="(log, index) in logs"
          :key="index"
          :class="['log-item', `log-${log.level.toLowerCase()}`]"
        >
          <div class="log-header">
            <el-tag :type="getLogTagType(log.level)" size="small">{{ log.level }}</el-tag>
            <span class="log-timestamp">{{ log.timestamp }}</span>
            <span class="log-name">{{ log.name }}</span>
          </div>
          <div class="log-message">{{ log.message }}</div>
        </div>
        <div v-if="logs.length === 0" class="empty-log">
          <el-empty description="暂无日志" />
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Refresh, Delete, Download } from '@element-plus/icons-vue'

const logs = ref([])
const loading = ref(false)
const autoRefresh = ref(false)
const refreshTimer = ref(null)
const logContainer = ref(null)
const logFilePath = ref('')
const filterForm = ref({
  level: '',
  limit: 500
})

const getLogs = async () => {
  loading.value = true
  try {
    const result = await window.eel.get_logs(filterForm.value.level || null, filterForm.value.limit)()
    if (result.success) {
      logs.value = result.logs || []
      logFilePath.value = result.log_file_path || ''
      // 滚动到底部
      await nextTick()
      if (logContainer.value) {
        logContainer.value.scrollTop = logContainer.value.scrollHeight
      }
    } else {
      ElMessage.error('获取日志失败: ' + (result.error || '未知错误'))
    }
  } catch (error) {
    console.error('获取日志失败:', error)
    ElMessage.error('获取日志失败: ' + error.message)
  } finally {
    loading.value = false
  }
}

const refreshLogs = () => {
  getLogs()
}

const clearLogs = async () => {
  try {
    await ElMessageBox.confirm('确定要清空内存中的日志吗？', '确认清空', {
      type: 'warning'
    })
    const result = await window.eel.clear_logs()()
    if (result.success) {
      ElMessage.success('日志已清空')
      logs.value = []
    } else {
      ElMessage.error('清空日志失败: ' + (result.error || '未知错误'))
    }
  } catch (error) {
    if (error !== 'cancel') {
      console.error('清空日志失败:', error)
      ElMessage.error('清空日志失败: ' + error.message)
    }
  }
}

const exportLogs = () => {
  if (logs.value.length === 0) {
    ElMessage.warning('没有日志可导出')
    return
  }
  
  const content = logs.value.map(log => `${log.timestamp} [${log.level}] ${log.name}: ${log.message}`).join('\n')
  const blob = new Blob([content], { type: 'text/plain;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = `qsl-cardhub-logs-${new Date().toISOString().slice(0, 10)}.txt`
  link.click()
  URL.revokeObjectURL(url)
  ElMessage.success('日志已导出')
}

const applyFilter = () => {
  getLogs()
}

const getLogTagType = (level) => {
  const typeMap = {
    'DEBUG': 'info',
    'INFO': '',
    'WARNING': 'warning',
    'ERROR': 'danger',
    'CRITICAL': 'danger'
  }
  return typeMap[level] || ''
}

const handleAutoRefreshChange = (value) => {
  if (value) {
    refreshTimer.value = setInterval(() => {
      getLogs()
    }, 2000) // 每2秒刷新一次
  } else {
    if (refreshTimer.value) {
      clearInterval(refreshTimer.value)
      refreshTimer.value = null
    }
  }
}

onMounted(() => {
  getLogs()
})

onUnmounted(() => {
  if (refreshTimer.value) {
    clearInterval(refreshTimer.value)
  }
})
</script>

<style scoped>
.log-container {
  max-height: 600px;
  overflow-y: auto;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', 'Consolas', 'source-code-pro', monospace;
  font-size: 13px;
  line-height: 1.6;
}

.log-item {
  padding: 10px;
  margin-bottom: 8px;
  border-left: 3px solid #e0e0e0;
  background: #fafafa;
  border-radius: 4px;
  transition: all 0.2s;
}

.log-item:hover {
  background: #f0f0f0;
}

.log-debug {
  border-left-color: #909399;
  background: #f4f4f5;
}

.log-info {
  border-left-color: #409eff;
  background: #ecf5ff;
}

.log-warning {
  border-left-color: #e6a23c;
  background: #fdf6ec;
}

.log-error,
.log-critical {
  border-left-color: #f56c6c;
  background: #fef0f0;
}

.log-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 6px;
}

.log-timestamp {
  color: #909399;
  font-size: 12px;
}

.log-name {
  color: #606266;
  font-size: 12px;
  font-weight: 500;
}

.log-message {
  color: #303133;
  word-break: break-word;
  white-space: pre-wrap;
}

.empty-log {
  padding: 40px;
  text-align: center;
}
</style>
