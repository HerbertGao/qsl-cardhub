// 云同步配置共享响应式单一事实源
//
// 徽章(App.vue) / 网关 / DataTransferView / CardManagementView 四处共享。
// store 持「已保存态」；表单草稿（DataTransferView 的 syncForm）不入 store。
// mode/canSync 派生、不存字段：mode = api_url 空否；canSync = api_url && has_api_key
// （后端 /sync 硬要求 API Key——模式≠可同步就绪）。

import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { SyncConfigResponse } from '@/types/models'

// 已保存态字段（完整 SyncConfigResponse）
const apiUrl = ref<string>('')
const tenant = ref<string | null>(null)
const hasApiKey = ref<boolean>(false)
const lastSyncAt = ref<string | null>(null)
const clientId = ref<string>('')
const baseVersion = ref<number | null>(null)

// 加载错误态（reject 时为真，驱动 App 错误分支 + 重试）。
// 注意：渲染揭幕标志不放这里——见 App.vue 的 bootReady（须在网关/错误决策定下后才揭幕，避免抢先挂载主界面）
const loadError = ref<boolean>(false)

// 派生（computed，不存字段）
const mode = computed<'cloud' | 'local'>(() => (apiUrl.value ? 'cloud' : 'local'))
const canSync = computed<boolean>(() => !!apiUrl.value && hasApiKey.value)

// 用命令返回的 SyncConfigResponse 就地更新（含后端规整后的 tenant）
function applyConfig(c: SyncConfigResponse): void {
  apiUrl.value = c.api_url
  tenant.value = c.tenant
  hasApiKey.value = c.has_api_key
  lastSyncAt.value = c.last_sync_at
  clientId.value = c.client_id
  baseVersion.value = c.base_version
}

// 首启加载；reject 时置 loadError、不按空 apiUrl 判首启/不置 onboarding 标志（调用方据 loadError 处理）。
// 内部吞异常（catch 不重抛），故 await load() 必 resolve，调用方的引导序总能跑到揭幕，不卡加载态
async function load(): Promise<void> {
  try {
    const c = await invoke<SyncConfigResponse | null>('load_sync_config_cmd')
    if (c) applyConfig(c)
    loadError.value = false
  } catch {
    loadError.value = true
  }
}

// clear 后重置：清云字段、**保留 clientId**（设备标识不随清云配丢失）
function reset(): void {
  apiUrl.value = ''
  tenant.value = null
  hasApiKey.value = false
  lastSyncAt.value = null
  baseVersion.value = null
  // clientId 保留
}

// sync 成功回写 lastSyncAt + baseVersion（SyncCmdResult 有 sync_time）
function applySyncSuccess(serverVersion: number | null, syncTime: string): void {
  lastSyncAt.value = syncTime
  baseVersion.value = serverVersion
}

// restore 成功仅回写 baseVersion（RestoreResult 无 sync_time，禁止置空 lastSyncAt）
function applyRestoreSuccess(serverVersion: number | null): void {
  baseVersion.value = serverVersion
}

export const syncStore = {
  apiUrl,
  tenant,
  hasApiKey,
  lastSyncAt,
  clientId,
  baseVersion,
  loadError,
  mode,
  canSync,
  load,
  applyConfig,
  reset,
  applySyncSuccess,
  applyRestoreSuccess,
}
