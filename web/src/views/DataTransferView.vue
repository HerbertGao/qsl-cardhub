<template>
  <div class="page-content">
    <h1>数据管理</h1>

    <!-- 数据导出 -->
    <el-card shadow="hover" style="margin-bottom: 20px">
      <template #header>
        <div class="card-header">
          <span>数据导出</span>
        </div>
      </template>
      <el-form label-width="100px">
        <el-form-item label="导出说明">
          <div class="description-text">
            将本地数据库中的所有数据（项目、卡片、寄件人、订单）导出为 JSON 格式文件，便于备份和迁移。
          </div>
        </el-form-item>
        <el-form-item>
          <el-button
            type="primary"
            :loading="exportLoading"
            @click="handleExport"
          >
            <el-icon><Download /></el-icon>
            <span style="margin-left: 4px">导出数据</span>
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- 数据导入 -->
    <el-card shadow="hover" style="margin-bottom: 20px">
      <template #header>
        <div class="card-header">
          <span>数据导入</span>
        </div>
      </template>
      <el-form label-width="100px">
        <el-form-item label="导入说明">
          <div class="description-text">
            <el-alert
              title="警告：导入将覆盖本地所有数据，此操作不可逆！建议先导出备份。"
              type="warning"
              :closable="false"
              show-icon
              style="margin-bottom: 10px"
            />
            从 QSL-CardHub 导出的 .qslhub 文件导入数据到本地数据库。
          </div>
        </el-form-item>
        <el-form-item>
          <el-button
            type="warning"
            :loading="importLoading"
            @click="handleImport"
          >
            <el-icon><Upload /></el-icon>
            <span style="margin-left: 4px">导入数据</span>
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- 云端同步 -->
    <el-card shadow="hover">
      <template #header>
        <div class="card-header">
          <span>云端同步</span>
          <el-tag v-if="syncConfig?.last_sync_at" type="success" size="small">
            上次同步: {{ formatDateTime(syncConfig.last_sync_at) }}
          </el-tag>
        </div>
      </template>
      <el-form label-width="100px" :model="syncForm">
        <el-form-item label="同步说明">
          <div class="description-text">
            将本地数据全量同步到您自建的云端 API。请参考
            <el-link type="primary" @click="showApiSpec">API 规范文档</el-link>
            部署接收服务。
          </div>
        </el-form-item>

        <el-form-item label="API 地址" required>
          <el-input
            v-model="syncForm.api_url"
            placeholder="https://your-api.example.com/api"
            style="max-width: 400px"
          />
        </el-form-item>

        <el-form-item label="API Key" required>
          <el-input
            v-model="syncForm.api_key"
            type="password"
            placeholder="输入您的 API Key"
            show-password
            style="max-width: 400px"
          />
          <div v-if="syncConfig?.has_api_key && !syncForm.api_key" class="form-hint">
            已保存 API Key（留空则保持不变）
          </div>
        </el-form-item>

        <el-form-item label="客户端 ID">
          <el-input
            v-model="syncConfig.client_id"
            disabled
            style="max-width: 400px"
          />
          <div class="form-hint">自动生成的客户端标识，用于云端识别</div>
        </el-form-item>

        <el-form-item>
          <el-button-group>
            <el-button
              type="primary"
              :loading="saveConfigLoading"
              @click="handleSaveConfig"
            >
              保存配置
            </el-button>
            <el-button
              :loading="testConnectionLoading"
              :disabled="!syncForm.api_url"
              @click="handleTestConnection"
            >
              测试连接
            </el-button>
            <el-button
              type="success"
              :loading="syncLoading"
              :disabled="!syncConfig?.has_api_key || !syncForm.api_url"
              @click="handleSync"
            >
              <el-icon><Refresh /></el-icon>
              <span style="margin-left: 4px">立即同步</span>
            </el-button>
          </el-button-group>
          <el-button
            type="danger"
            plain
            style="margin-left: 12px"
            :disabled="!syncConfig?.api_url"
            @click="handleClearConfig"
          >
            清除配置
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- 导入预览对话框 -->
    <el-dialog
      v-model="importPreviewVisible"
      title="导入预览"
      width="500px"
    >
      <div v-if="importPreview">
        <el-descriptions :column="1" border>
          <el-descriptions-item label="文件格式版本">
            {{ importPreview.version }}
          </el-descriptions-item>
          <el-descriptions-item label="数据库版本">
            {{ importPreview.db_version_display }}
          </el-descriptions-item>
          <el-descriptions-item label="应用版本">
            {{ importPreview.app_version }}
          </el-descriptions-item>
          <el-descriptions-item label="导出时间">
            {{ formatDateTime(importPreview.exported_at) }}
          </el-descriptions-item>
        </el-descriptions>

        <el-divider />

        <el-descriptions title="数据统计" :column="2" border>
          <el-descriptions-item label="项目">
            {{ importPreview.stats.projects }} 个
          </el-descriptions-item>
          <el-descriptions-item label="卡片">
            {{ importPreview.stats.cards }} 张
          </el-descriptions-item>
          <el-descriptions-item label="寄件人">
            {{ importPreview.stats.sf_senders }} 个
          </el-descriptions-item>
          <el-descriptions-item label="订单">
            {{ importPreview.stats.sf_orders }} 个
          </el-descriptions-item>
        </el-descriptions>

        <el-divider />

        <el-descriptions title="版本兼容性" :column="1" border>
          <el-descriptions-item label="文件版本">
            {{ importPreview.db_version_display }}
          </el-descriptions-item>
          <el-descriptions-item label="本地版本">
            {{ importPreview.local_db_version_display }}
          </el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag v-if="importPreview.can_import" type="success">
              可以导入
            </el-tag>
            <el-tag v-else type="danger">
              无法导入
            </el-tag>
          </el-descriptions-item>
        </el-descriptions>

        <el-alert
          v-if="!importPreview.can_import"
          :title="importPreview.error_message"
          type="error"
          :closable="false"
          show-icon
          style="margin-top: 16px"
        />

        <el-alert
          v-else
          title="导入将覆盖本地所有数据，此操作不可逆！"
          type="warning"
          :closable="false"
          show-icon
          style="margin-top: 16px"
        />
      </div>

      <template #footer>
        <el-button @click="importPreviewVisible = false">取消</el-button>
        <el-button
          type="primary"
          :disabled="!importPreview?.can_import"
          :loading="importLoading"
          @click="confirmImport"
        >
          确认导入
        </el-button>
      </template>
    </el-dialog>

    <!-- API 规范对话框 -->
    <el-dialog
      v-model="apiSpecVisible"
      title="云端同步 API 规范"
      width="700px"
    >
      <div class="api-spec-content">
        <h3>认证方式</h3>
        <p>使用 API Key (Bearer Token) 认证，在请求头中携带：</p>
        <pre>Authorization: Bearer {api_key}</pre>

        <h3>接口列表</h3>

        <h4>1. 连接测试</h4>
        <pre>
GET /ping
Authorization: Bearer {api_key}

响应（200）：
{
  "success": true,
  "message": "pong",
  "server_time": "2026-01-23T14:30:00+08:00"
}</pre>

        <h4>2. 数据同步</h4>
        <pre>
POST /sync
Authorization: Bearer {api_key}
Content-Type: application/json

请求体：
{
  "client_id": "uuid",
  "sync_time": "2026-01-23T14:30:00+08:00",
  "data": {
    "projects": [...],
    "cards": [...],
    "sf_senders": [...],
    "sf_orders": [...]
  }
}

响应（200）：
{
  "success": true,
  "message": "同步成功",
  "received_at": "2026-01-23T14:30:01+08:00",
  "stats": {
    "projects": 10,
    "cards": 500,
    "sf_senders": 5,
    "sf_orders": 100
  }
}</pre>

        <h3>错误响应格式</h3>
        <pre>
{
  "success": false,
  "message": "错误描述"
}</pre>

        <h3>实现建议</h3>
        <ul>
          <li>按 client_id 隔离不同客户端数据</li>
          <li>实现请求频率限制</li>
          <li>使用 HTTPS</li>
          <li>验证 API Key 有效性</li>
        </ul>
      </div>

      <template #footer>
        <el-button @click="apiSpecVisible = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save, open } from '@tauri-apps/plugin-dialog'
import { ElMessage, ElMessageBox } from 'element-plus'
import { logger } from '@/utils/logger'

// 类型定义
interface ExportStats {
  projects: number
  cards: number
  sf_senders: number
  sf_orders: number
}

interface ImportPreview {
  version: string
  db_version: number
  db_version_display: string
  app_version: string
  exported_at: string
  stats: ExportStats
  can_import: boolean
  error_message: string | null
  local_db_version: number
  local_db_version_display: string
}

interface SyncConfigResponse {
  api_url: string
  client_id: string
  last_sync_at: string | null
  has_api_key: boolean
}

// 状态
const exportLoading = ref(false)
const importLoading = ref(false)
const saveConfigLoading = ref(false)
const testConnectionLoading = ref(false)
const syncLoading = ref(false)

const importPreviewVisible = ref(false)
const importPreview = ref<ImportPreview | null>(null)
const importFilePath = ref<string>('')

const apiSpecVisible = ref(false)

const syncConfig = ref<SyncConfigResponse>({
  api_url: '',
  client_id: '',
  last_sync_at: null,
  has_api_key: false
})

const syncForm = reactive({
  api_url: '',
  api_key: ''
})

// 格式化时间
function formatDateTime(dateStr: string): string {
  if (!dateStr) return ''
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

// 导出数据
async function handleExport() {
  try {
    const now = new Date()
    const timestamp = now.toISOString().slice(0, 19).replace(/[-:T]/g, '').replace(/(\d{8})(\d{6})/, '$1_$2')
    const defaultName = `qslhub_backup_${timestamp}.qslhub`

    const filePath = await save({
      defaultPath: defaultName,
      filters: [{ name: 'QSL-CardHub 备份文件', extensions: ['qslhub'] }]
    })

    if (!filePath) return

    exportLoading.value = true
    const stats = await invoke<ExportStats>('export_data', { filePath })

    ElMessage.success(
      `导出成功：${stats.projects} 个项目，${stats.cards} 张卡片，${stats.sf_senders} 个寄件人，${stats.sf_orders} 个订单`
    )
    logger.info(`[数据导出] 导出完成: ${filePath}`)
  } catch (error) {
    ElMessage.error(`导出失败：${error}`)
    logger.error(`[数据导出] 失败: ${error}`)
  } finally {
    exportLoading.value = false
  }
}

// 导入数据
async function handleImport() {
  try {
    const filePath = await open({
      filters: [{ name: 'QSL-CardHub 备份文件', extensions: ['qslhub'] }]
    })

    if (!filePath) return

    importLoading.value = true
    importFilePath.value = filePath as string

    const preview = await invoke<ImportPreview>('preview_import_data', {
      filePath: importFilePath.value
    })

    importPreview.value = preview
    importPreviewVisible.value = true
    logger.info(`[数据导入] 预览文件: ${filePath}`)
  } catch (error) {
    ElMessage.error(`预览失败：${error}`)
    logger.error(`[数据导入] 预览失败: ${error}`)
  } finally {
    importLoading.value = false
  }
}

// 确认导入
async function confirmImport() {
  try {
    await ElMessageBox.confirm(
      '确定要导入数据吗？此操作将覆盖本地所有数据，不可逆！',
      '确认导入',
      {
        confirmButtonText: '确认导入',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    importLoading.value = true
    const stats = await invoke<ExportStats>('import_data', {
      filePath: importFilePath.value
    })

    importPreviewVisible.value = false
    ElMessage.success(
      `导入成功：${stats.projects} 个项目，${stats.cards} 张卡片，${stats.sf_senders} 个寄件人，${stats.sf_orders} 个订单`
    )
    logger.info(`[数据导入] 导入完成`)
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(`导入失败：${error}`)
      logger.error(`[数据导入] 失败: ${error}`)
    }
  } finally {
    importLoading.value = false
  }
}

// 加载同步配置
async function loadSyncConfig() {
  try {
    const config = await invoke<SyncConfigResponse | null>('load_sync_config_cmd')
    if (config) {
      syncConfig.value = config
      syncForm.api_url = config.api_url
    }
  } catch (error) {
    logger.error(`[同步配置] 加载失败: ${error}`)
  }
}

// 保存同步配置
async function handleSaveConfig() {
  if (!syncForm.api_url) {
    ElMessage.warning('请输入 API 地址')
    return
  }

  try {
    saveConfigLoading.value = true
    const config = await invoke<SyncConfigResponse>('save_sync_config_cmd', {
      apiUrl: syncForm.api_url,
      apiKey: syncForm.api_key || null
    })

    syncConfig.value = config
    syncForm.api_key = ''
    ElMessage.success('配置已保存')
    logger.info('[同步配置] 保存成功')
  } catch (error) {
    ElMessage.error(`保存失败：${error}`)
    logger.error(`[同步配置] 保存失败: ${error}`)
  } finally {
    saveConfigLoading.value = false
  }
}

// 测试连接
async function handleTestConnection() {
  try {
    testConnectionLoading.value = true
    await invoke('test_sync_connection_cmd')
    ElMessage.success('连接成功')
    logger.info('[同步配置] 连接测试成功')
  } catch (error) {
    ElMessage.error(`连接失败：${error}`)
    logger.error(`[同步配置] 连接测试失败: ${error}`)
  } finally {
    testConnectionLoading.value = false
  }
}

// 执行同步
async function handleSync() {
  try {
    syncLoading.value = true
    const result = await invoke<{
      response: { success: boolean; message: string }
      stats: ExportStats
      sync_time: string
    }>('execute_sync_cmd')

    syncConfig.value.last_sync_at = result.sync_time
    ElMessage.success(
      `同步成功：${result.stats.projects} 个项目，${result.stats.cards} 张卡片，${result.stats.sf_senders} 个寄件人，${result.stats.sf_orders} 个订单`
    )
    logger.info('[同步] 同步完成')
  } catch (error) {
    ElMessage.error(`同步失败：${error}`)
    logger.error(`[同步] 失败: ${error}`)
  } finally {
    syncLoading.value = false
  }
}

// 清除配置
async function handleClearConfig() {
  try {
    await ElMessageBox.confirm(
      '确定要清除同步配置吗？这将删除 API 地址和 API Key。',
      '确认清除',
      {
        confirmButtonText: '确认清除',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    await invoke('clear_sync_config_cmd')

    syncConfig.value = {
      api_url: '',
      client_id: syncConfig.value.client_id,
      last_sync_at: null,
      has_api_key: false
    }
    syncForm.api_url = ''
    syncForm.api_key = ''

    ElMessage.success('配置已清除')
    logger.info('[同步配置] 配置已清除')
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(`清除失败：${error}`)
      logger.error(`[同步配置] 清除失败: ${error}`)
    }
  }
}

// 显示 API 规范
function showApiSpec() {
  apiSpecVisible.value = true
}

// 初始化
onMounted(() => {
  loadSyncConfig()
})
</script>

<style scoped>
.page-content h1 {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.description-text {
  color: #606266;
  font-size: 14px;
  line-height: 1.6;
}

.form-hint {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

.api-spec-content {
  max-height: 500px;
  overflow-y: auto;
}

.api-spec-content h3 {
  margin-top: 20px;
  margin-bottom: 10px;
  color: #303133;
}

.api-spec-content h3:first-child {
  margin-top: 0;
}

.api-spec-content h4 {
  margin-top: 16px;
  margin-bottom: 8px;
  color: #606266;
}

.api-spec-content p {
  margin-bottom: 8px;
  color: #606266;
}

.api-spec-content pre {
  background: #f5f7fa;
  padding: 12px;
  border-radius: 4px;
  overflow-x: auto;
  font-size: 13px;
  line-height: 1.5;
}

.api-spec-content ul {
  padding-left: 20px;
  color: #606266;
}

.api-spec-content li {
  margin-bottom: 4px;
}
</style>
