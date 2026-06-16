<template>
  <div class="page-content">
    <h1>数据管理</h1>

    <!-- 数据导出 -->
    <el-card
      shadow="hover"
      style="margin-bottom: 20px"
    >
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
    <el-card
      shadow="hover"
      style="margin-bottom: 20px"
    >
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
          <el-tag
            v-if="syncConfig?.last_sync_at"
            type="success"
            size="small"
          >
            上次同步: {{ formatDateTime(syncConfig.last_sync_at) }}
          </el-tag>
        </div>
      </template>
      <el-form
        label-width="100px"
        :model="syncForm"
      >
        <el-form-item label="同步说明">
          <div class="description-text">
            将本地数据全量同步到您自建的云端 API。请参考
            <el-link
              type="primary"
              @click="showApiSpec"
            >
              API 规范文档
            </el-link>
            部署接收服务。
          </div>
        </el-form-item>

        <el-form-item
          label="API 地址"
          required
        >
          <el-select
            v-model="syncForm.api_url"
            filterable
            allow-create
            default-first-option
            clearable
            placeholder="选择官方云或手动输入 API 地址"
            style="max-width: 400px"
          >
            <el-option
              label="（空）未配置"
              value=""
            />
            <el-option
              label="https://qsl.herbertgao.me（官方云）"
              :value="OFFICIAL_CLOUD_URL"
            />
          </el-select>
          <div
            v-if="isOfficialCloud"
            class="form-hint"
          >
            官方云：租户代码与 API Key 必填
          </div>
        </el-form-item>

        <el-form-item
          label="API Key"
          :required="isOfficialCloud"
        >
          <el-input
            v-model="syncForm.api_key"
            type="password"
            :placeholder="syncConfig?.has_api_key ? '已保存（留空保持不变，输入新值可更新）' : '输入您的 API Key'"
            show-password
            style="max-width: 400px"
          />
          <div
            v-if="syncConfig?.has_api_key && !syncForm.api_key"
            class="form-hint"
          >
            ✓ 已保存 API Key（出于安全不回显，留空则保持不变）
          </div>
        </el-form-item>

        <el-form-item
          label="租户代码"
          :required="isOfficialCloud"
        >
          <el-input
            v-model="syncForm.tenant"
            placeholder="请输入租户代码"
            style="max-width: 400px"
            @blur="formatTenant"
          />
          <div class="form-hint">
            申报所属租户（小写字母 / 数字 / 连字符，1-32 位，<strong>输入完毕自动格式化</strong>）；归属以 API Key 为准，此处仅申报 + 交叉校验。留空则不申报、按兼容模式同步。
          </div>
        </el-form-item>

        <el-form-item label="客户端 ID">
          <!-- 只读展示，故用 div 而非 disabled el-input：规避 WKWebView 对禁用输入框
               re-mount 时文字下缘裁切的渲染 bug，且本就是不可编辑的自动生成值 -->
          <div class="readonly-field">
            {{ syncConfig.client_id || '—' }}
          </div>
          <div class="form-hint">
            自动生成的客户端标识，用于云端识别
          </div>
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
              @click="handleSync(false)"
            >
              <el-icon><Refresh /></el-icon>
              <span style="margin-left: 4px">立即同步</span>
            </el-button>
            <el-button
              type="warning"
              :loading="restoreLoading"
              :disabled="!syncConfig?.has_api_key || !syncForm.api_url"
              @click="handleRestoreFromCloud"
            >
              <el-icon><Download /></el-icon>
              <span style="margin-left: 4px">从云端恢复</span>
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

        <el-form-item label="配置迁移">
          <el-button-group>
            <el-button
              :loading="copyConfigLoading"
              :disabled="!syncConfig?.api_url"
              @click="handleCopyConfig"
            >
              一键复制
            </el-button>
            <el-button
              :loading="pasteConfigLoading"
              @click="handlePasteConfig"
            >
              一键粘贴
            </el-button>
          </el-button-group>
          <div class="form-hint">
            复制：把当前 API 地址 / 租户代码 / API Key 导出为字符串到剪贴板（<strong>含明文 Key</strong>，仅用于本机设备间迁移，请妥善保管）。粘贴：从剪贴板字符串导入并保存。
          </div>
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
        <el-descriptions
          :column="1"
          border
        >
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

        <el-descriptions
          title="数据统计"
          :column="2"
          border
        >
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

        <el-descriptions
          title="版本兼容性"
          :column="1"
          border
        >
          <el-descriptions-item label="文件版本">
            {{ importPreview.db_version_display }}
          </el-descriptions-item>
          <el-descriptions-item label="本地版本">
            {{ importPreview.local_db_version_display }}
          </el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag
              v-if="importPreview.can_import"
              type="success"
            >
              可以导入
            </el-tag>
            <el-tag
              v-else
              type="danger"
            >
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
        <el-button @click="importPreviewVisible = false">
          取消
        </el-button>
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
  "base_version": 41,        // 可选，整数，本地基线版本
  "force": false,            // 可选，布尔，强制覆盖云端
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
  "server_version": 42,
  "stats": {
    "projects": 10,
    "cards": 500,
    "sf_senders": 5,
    "sf_orders": 100
  }
}

响应（409，基线陈旧时返回）：
{
  "success": false,
  "server_version": 42       // 云端当前版本
}</pre>

        <h4>3. 拉取全量快照</h4>
        <pre>
GET /pull
Authorization: Bearer {api_key}

按写入 Key 拉回全量快照 + server_version，用于换机/恢复。

响应（200）：
{
  "success": true,
  "server_version": 42,
  "data": {
    "projects": [...],
    "cards": [...],
    "sf_senders": [...],
    "sf_orders": [...]
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
          <li>按写入 Key 解析租户归属，client_id 仅作设备溯源、不决定归属</li>
          <li>实现请求频率限制</li>
          <li>使用 HTTPS</li>
          <li>验证 API Key 有效性</li>
        </ul>
      </div>

      <template #footer>
        <el-button @click="apiSpecVisible = false">
          关闭
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { save, open } from '@tauri-apps/plugin-dialog'
import { ElMessage, ElMessageBox } from 'element-plus'
import { logger } from '@/utils/logger'
import type {
  ExportStats,
  PingResponse,
  RestoreResult,
  SyncCmdResult,
  SyncConfigResponse,
} from '@/types/models'

// 类型定义
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
  has_api_key: false,
  base_version: null,
  tenant: null
})

const restoreLoading = ref(false)
const copyConfigLoading = ref(false)
const pasteConfigLoading = ref(false)

const syncForm = reactive({
  api_url: '',
  api_key: '',
  tenant: ''
})

// 官方云预设地址：选中它时租户代码与 API Key 必填
const OFFICIAL_CLOUD_URL = 'https://qsl.herbertgao.me'
const isOfficialCloud = computed(() => syncForm.api_url === OFFICIAL_CLOUD_URL)

// 租户代码输入完毕（blur）自动格式化为合法 slug：小写 + 去非法字符 + 截断 32 位，避免用户手动修改
function formatTenant() {
  syncForm.tenant = syncForm.tenant
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9-]/g, '')
    .slice(0, 32)
}

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
      syncForm.tenant = config.tenant ?? ''
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

  // 即时 slug 校验（与服务端 schema CHECK 对齐），拒大写/非法字符；服务端为权威校验
  const tenant = syncForm.tenant.trim()
  if (tenant && !/^[a-z0-9-]{1,32}$/.test(tenant)) {
    ElMessage.warning('租户代码只能是小写字母、数字、连字符，长度 1-32 位')
    return
  }

  // 官方云预设：租户代码 + API Key 必填（API Key 已保存则可不重填）
  if (isOfficialCloud.value) {
    if (!tenant) {
      ElMessage.warning('选择官方云时，租户代码为必填')
      return
    }
    if (!syncForm.api_key && !syncConfig.value.has_api_key) {
      ElMessage.warning('选择官方云时，API Key 为必填')
      return
    }
  }

  try {
    saveConfigLoading.value = true
    const config = await invoke<SyncConfigResponse>('save_sync_config_cmd', {
      apiUrl: syncForm.api_url,
      apiKey: syncForm.api_key || null,
      tenant: tenant || null
    })

    syncConfig.value = config
    syncForm.api_key = ''
    syncForm.tenant = config.tenant ?? ''
    ElMessage.success('配置已保存')
    logger.info('[同步配置] 保存成功')
  } catch (error) {
    ElMessage.error(`保存失败：${error}`)
    logger.error(`[同步配置] 保存失败: ${error}`)
  } finally {
    saveConfigLoading.value = false
  }
}

// 一键复制：导出当前同步配置为字符串到剪贴板（含明文 API Key）
async function handleCopyConfig() {
  try {
    copyConfigLoading.value = true
    const str = await invoke<string>('export_sync_config_string_cmd')
    await navigator.clipboard.writeText(str)
    ElMessage.success('配置已复制到剪贴板（含明文 API Key，请妥善保管）')
    logger.info('[同步配置] 已导出配置字符串')
  } catch (error) {
    ElMessage.error(`复制失败：${error}`)
    logger.error(`[同步配置] 导出失败: ${error}`)
  } finally {
    copyConfigLoading.value = false
  }
}

// 一键粘贴：从剪贴板字符串导入并保存同步配置
async function handlePasteConfig() {
  try {
    pasteConfigLoading.value = true
    const str = await navigator.clipboard.readText()
    if (!str || !str.trim()) {
      ElMessage.warning('剪贴板为空')
      return
    }
    const config = await invoke<SyncConfigResponse>('import_sync_config_string_cmd', {
      data: str.trim()
    })
    syncConfig.value = config
    syncForm.api_url = config.api_url
    syncForm.tenant = config.tenant ?? ''
    syncForm.api_key = ''
    ElMessage.success('配置已从剪贴板导入并保存')
    logger.info('[同步配置] 从字符串导入成功')
  } catch (error) {
    ElMessage.error(`粘贴导入失败：${error}`)
    logger.error(`[同步配置] 导入失败: ${error}`)
  } finally {
    pasteConfigLoading.value = false
  }
}

// 测试连接
async function handleTestConnection() {
  try {
    testConnectionLoading.value = true
    // 测试表单当前填写的值（无需先保存）；API Key 留空则后端回落到已保存凭据
    const ping = await invoke<PingResponse>('test_sync_connection_cmd', {
      apiUrl: syncForm.api_url,
      apiKey: syncForm.api_key || null,
      tenant: syncForm.tenant.trim() || null
    })
    ElMessage.success(ping.tenant ? `连接成功，已认证租户：${ping.tenant}` : '连接成功')
    // fallback：凭据兜底命中默认租户（信息提示、非 mismatch；真正的不匹配由 403 捕获）
    if (ping.fallback) {
      ElMessage.warning('凭据命中默认租户兜底，请确认 API Key 归属')
    }
    logger.info('[同步配置] 连接测试成功')
  } catch (error) {
    ElMessage.error(`连接失败：${error}`)
    logger.error(`[同步配置] 连接测试失败: ${error}`)
  } finally {
    testConnectionLoading.value = false
  }
}

// 穷尽检查辅助：所有 union 分支处理后，残余类型应为 never；漏 case 时 TS 编译报错
function assertNever(x: never): never {
  throw new Error(`未处理的同步结果状态: ${JSON.stringify(x)}`)
}

// 编译期钉住 SyncCmdResult 的 discriminant 形状（status + snake_case 单元变体）：
// ts-rs 渲染若漂移（如 status 改名 / auth_failed 变非对象）此处即编译报错
void ({ status: 'auth_failed' } satisfies SyncCmdResult)

// 执行同步（force 为 true 时走强制覆盖逃生门）
async function handleSync(force = false) {
  try {
    syncLoading.value = true
    const result = await invoke<SyncCmdResult>('execute_sync_cmd', { force })

    switch (result.status) {
      case 'success': {
        syncConfig.value.last_sync_at = result.sync_time
        syncConfig.value.base_version = result.server_version
        ElMessage.success(
          `同步成功：${result.stats.projects} 个项目，${result.stats.cards} 张卡片，${result.stats.sf_senders} 个寄件人，${result.stats.sf_orders} 个订单`
        )
        logger.info('[同步] 同步完成')
        break
      }
      case 'auth_failed': {
        ElMessage.error('认证失败，请检查 API Key')
        logger.error('[同步] 认证失败')
        break
      }
      case 'conflict': {
        logger.warn(`[同步] 版本冲突，云端版本: ${result.server_version}`)
        await handleSyncConflict(result.server_version)
        break
      }
      case 'tenant_mismatch': {
        ElMessage.error('租户代码与 API Key 归属的租户不一致，请检查租户代码')
        logger.error('[同步] 租户不匹配 (403 tenant_mismatch)')
        break
      }
      default: {
        // 穷尽检查（D11）：四态全覆盖后此分支不可达；新增状态未处理时 TS 在此编译报错
        assertNever(result)
      }
    }
  } catch (error) {
    ElMessage.error(`同步失败：${error}`)
    logger.error(`[同步] 失败: ${error}`)
  } finally {
    syncLoading.value = false
  }
}

// 处理版本冲突（409）：引导用户下载云端最新或强制覆盖
async function handleSyncConflict(serverVersion: number | null) {
  const versionText =
    serverVersion !== null && serverVersion !== undefined
      ? `云端当前版本：${serverVersion}。`
      : '无法获取云端当前版本。'

  try {
    // 三个动作：下载云端最新（confirm）/ 强制覆盖（cancel）/ 关闭（不处理）
    await ElMessageBox.confirm(
      `${versionText}本地基线落后于云端（其他设备已先同步）。请选择处理方式：<br><br>· <strong>下载云端最新</strong>：用云端数据覆盖本地，丢失本地未上传改动<br>· <strong>强制覆盖</strong>：用本机数据无条件覆盖云端`,
      '版本冲突',
      {
        confirmButtonText: '下载云端最新',
        cancelButtonText: '强制覆盖',
        distinguishCancelAndClose: true,
        dangerouslyUseHTMLString: true,
        type: 'warning'
      }
    )
    // 用户选择「下载云端最新」→ 走从云端恢复
    await restoreFromCloud()
  } catch (action) {
    if (action === 'cancel') {
      // 用户选择「强制覆盖」→ force=true 重发 /sync
      await handleSync(true)
    }
    // action === 'close'（点 X 或按 ESC）→ 不做处理
  }
}

// 从云端恢复（核心逻辑，供冲突引导与「从云端恢复」按钮共用）
async function restoreFromCloud() {
  try {
    restoreLoading.value = true
    const result = await invoke<RestoreResult>('restore_from_cloud')

    syncConfig.value.base_version = result.server_version
    ElMessage.success(
      `恢复成功：${result.stats.projects} 个项目，${result.stats.cards} 张卡片，${result.stats.sf_senders} 个寄件人，${result.stats.sf_orders} 个订单`
    )
    logger.info('[从云端恢复] 恢复完成')
  } catch (error) {
    ElMessage.error(String(error))
    logger.error(`[从云端恢复] 失败: ${error}`)
  } finally {
    restoreLoading.value = false
  }
}

// 从云端恢复（按钮入口，前置二次确认）
async function handleRestoreFromCloud() {
  try {
    await ElMessageBox.confirm(
      '确定要从云端恢复吗？此操作将用云端数据覆盖本地，丢失本地未上传的改动，不可逆！',
      '确认从云端恢复',
      {
        confirmButtonText: '确认恢复',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )
  } catch {
    // 用户取消
    return
  }

  await restoreFromCloud()
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
      has_api_key: false,
      base_version: null,
      tenant: null
    }
    syncForm.api_url = ''
    syncForm.api_key = ''
    syncForm.tenant = ''

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
  width: 100%; /* 独占一行，始终位于字段下方（避免短提示在 flex-wrap 里挤到字段右侧）*/
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

/* 只读字段展示（客户端 ID 等不可编辑值）：块级 div + padding 留白 + 自然行高，
   外观近似禁用输入框，但不走 input 渲染路径，规避 WKWebView 对禁用输入框的文字裁切 bug。
   用 padding 而非固定行高保证文字上下有余量、任何引擎都不裁切 */
.readonly-field {
  width: 100%;
  max-width: 400px;
  box-sizing: border-box;
  min-height: 32px;
  padding: 5px 11px;
  line-height: 20px;
  background: var(--el-fill-color-light, #f5f7fa);
  border: 1px solid var(--el-border-color, #dcdfe6);
  border-radius: 4px;
  color: var(--el-text-color-regular, #606266);
  font-size: 14px;
  word-break: break-all;
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
