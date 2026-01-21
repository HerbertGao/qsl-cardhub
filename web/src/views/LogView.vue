<template>
  <div class="page-content">
    <div class="header-section">
      <h1>日志查看</h1>
      <div class="header-actions">
        <el-button @click="refreshLogs" :loading="loading">
          <el-icon>
            <Refresh/>
          </el-icon>
          刷新
        </el-button>
        <el-button @click="clearLogs" type="warning">
          <el-icon>
            <Delete/>
          </el-icon>
          清空
        </el-button>
        <el-button @click="exportLogs" type="primary">
          <el-icon>
            <Download/>
          </el-icon>
          导出
        </el-button>
      </div>
    </div>

    <!-- 过滤选项 -->
    <el-card style="margin-top: 20px">
      <div class="filter-row">
        <div class="filter-item">
          <label>日志级别：</label>
          <el-radio-group v-model="selectedLevel" @change="refreshLogs">
            <el-radio-button label="">全部</el-radio-button>
            <el-radio-button label="debug">DEBUG</el-radio-button>
            <el-radio-button label="info">INFO</el-radio-button>
            <el-radio-button label="warning">WARNING</el-radio-button>
            <el-radio-button label="error">ERROR</el-radio-button>
          </el-radio-group>
        </div>

        <div class="filter-item">
          <label>显示数量：</label>
          <el-select v-model="logLimit" @change="refreshLogs" style="width: 120px">
            <el-option label="50 条" :value="50"/>
            <el-option label="100 条" :value="100"/>
            <el-option label="200 条" :value="200"/>
            <el-option label="500 条" :value="500"/>
            <el-option label="全部" :value="null"/>
          </el-select>
        </div>

        <div class="filter-item">
          <el-switch
              v-model="autoRefresh"
              active-text="自动刷新"
              @change="toggleAutoRefresh"
          />
        </div>
      </div>

      <div class="log-file-info" v-if="logFilePath">
        <el-text size="small" type="info">
          <el-icon>
            <Document/>
          </el-icon>
          日志文件：{{ logFilePath }}
        </el-text>
      </div>
    </el-card>

    <!-- 日志表格 -->
    <el-card style="margin-top: 20px">
      <el-table
          :data="logs"
          stripe
          style="width: 100%"
          :max-height="600"
          v-loading="loading"
      >
        <el-table-column prop="timestamp" label="时间" width="180">
          <template #default="{ row }">
            {{ formatTimestamp(row.timestamp) }}
          </template>
        </el-table-column>

        <el-table-column prop="level" label="级别" width="100">
          <template #default="{ row }">
            <el-tag :type="getLevelTagType(row.level)" size="small">
              {{ row.level.toUpperCase() }}
            </el-tag>
          </template>
        </el-table-column>

        <el-table-column prop="source" label="来源" width="150"/>

        <el-table-column prop="message" label="消息" show-overflow-tooltip/>
      </el-table>

      <el-empty
          v-if="!loading && logs.length === 0"
          description="暂无日志"
          :image-size="100"
      />

      <div class="log-stats" v-if="logs.length > 0">
        <el-text size="small" type="info">
          共 {{ logs.length }} 条日志
        </el-text>
      </div>
    </el-card>
  </div>
</template>

<script setup>
import {onMounted, onUnmounted, ref} from 'vue'
import {ElMessage, ElMessageBox} from 'element-plus'
import {Delete, Document, Download, Refresh} from '@element-plus/icons-vue'
import {invoke} from '@tauri-apps/api/core'
import {save} from '@tauri-apps/plugin-dialog'

// 响应式状态
const logs = ref([])
const loading = ref(false)
const selectedLevel = ref('')
const logLimit = ref(100)
const autoRefresh = ref(false)
const logFilePath = ref('')
let refreshTimer = null

// 获取日志
const refreshLogs = async () => {
  loading.value = true
  try {
    const level = selectedLevel.value || null
    const limit = logLimit.value

    logs.value = await invoke('get_logs', {level, limit})
  } catch (error) {
    ElMessage.error(`获取日志失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 清空日志
const clearLogs = async () => {
  try {
    await ElMessageBox.confirm(
        '确定要清空内存日志吗？日志文件不会被删除。',
        '确认清空',
        {
          confirmButtonText: '确定',
          cancelButtonText: '取消',
          type: 'warning'
        }
    )

    loading.value = true
    await invoke('clear_logs')

    // 清空前端日志列表
    logs.value = []

    ElMessage.success('日志已清空')
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(`清空日志失败: ${error}`)
    }
  } finally {
    loading.value = false
  }
}

// 导出日志
const exportLogs = async () => {
  try {
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19)
    const defaultFilename = `qsl-logs-${timestamp}.txt`

    const exportPath = await save({
      defaultPath: defaultFilename,
      filters: [
        {
          name: '文本文件',
          extensions: ['txt']
        }
      ]
    })

    if (!exportPath) {
      return // 用户取消
    }

    await invoke('export_logs', {exportPath})
    ElMessage.success(`日志已导出到: ${exportPath}`)
  } catch (error) {
    ElMessage.error(`导出日志失败: ${error}`)
  }
}

// 获取日志文件路径
const fetchLogFilePath = async () => {
  try {
    const path = await invoke('get_log_file_path')
    logFilePath.value = path || ''
  } catch (error) {
    console.error('获取日志文件路径失败:', error)
  }
}

// 切换自动刷新
const toggleAutoRefresh = () => {
  if (autoRefresh.value) {
    // 启动自动刷新（每 5 秒）
    refreshTimer = setInterval(refreshLogs, 5000)
    ElMessage.info('已启动自动刷新（5 秒间隔）')
  } else {
    // 停止自动刷新
    if (refreshTimer) {
      clearInterval(refreshTimer)
      refreshTimer = null
    }
    ElMessage.info('已停止自动刷新')
  }
}

// 格式化时间戳
const formatTimestamp = (timestamp) => {
  // timestamp 格式: "2024-01-20T10:30:45.123+08:00"
  const date = new Date(timestamp)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

// 获取日志级别标签类型
const getLevelTagType = (level) => {
  const levelMap = {
    debug: 'info',
    info: 'success',
    warning: 'warning',
    error: 'danger'
  }
  return levelMap[level.toLowerCase()] || ''
}

// 生命周期
onMounted(async () => {
  await refreshLogs()
  await fetchLogFilePath()
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
})
</script>

<style scoped>
.page-content {
  padding: 20px;
}

.header-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-section h1 {
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.filter-row {
  display: flex;
  gap: 30px;
  align-items: center;
  flex-wrap: wrap;
}

.filter-item {
  display: flex;
  align-items: center;
  gap: 10px;
}

.filter-item label {
  font-size: 14px;
  color: #606266;
  white-space: nowrap;
}

.log-file-info {
  margin-top: 15px;
  padding-top: 15px;
  border-top: 1px solid #ebeef5;
}

.log-stats {
  margin-top: 15px;
  text-align: right;
}
</style>
