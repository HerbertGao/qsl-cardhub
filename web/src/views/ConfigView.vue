<template>
  <div class="page-content">
    <h1>打印机配置</h1>

    <el-card
      style="max-width: 600px; margin-top: 30px"
      shadow="hover"
    >
      <template #header>
        <div style="display: flex; justify-content: space-between; align-items: center">
          <span style="font-weight: bold">打印机设置</span>
          <el-tag
            v-if="saveStatus"
            :type="saveStatus.type === 'success' ? 'success' : 'danger'"
            size="small"
          >
            {{ saveStatus.message }}
          </el-tag>
        </div>
      </template>

      <el-form
        v-loading="loading"
        :model="config"
        label-width="120px"
      >
        <el-form-item label="操作系统">
          <el-input
            :value="platformDisplay"
            disabled
          />
        </el-form-item>

        <el-form-item
          label="打印机名称"
          required
        >
          <div style="display: flex; gap: 10px; width: 100%">
            <el-select
              v-model="config.printer.name"
              placeholder="请选择打印机"
              style="flex: 1; min-width: 0"
              :fit-input-width="true"
              :popper-options="{ strategy: 'fixed' }"
            >
              <el-option
                v-for="printer in availablePrinters"
                :key="printer"
                :label="printer"
                :value="printer"
              />
            </el-select>
            <el-button @click="refreshPrinters">
              <el-icon>
                <Refresh />
              </el-icon>
            </el-button>
          </div>
        </el-form-item>

        <el-form-item label="GAP (mm)">
          <div style="display: flex; gap: 8px; width: 100%">
            <el-input-number
              v-model="config.tspl.gap_mm"
              :min="0"
              :max="10"
              :step="0.5"
              :precision="1"
              style="width: 50%"
            />
            <el-input-number
              v-model="config.tspl.gap_offset_mm"
              :min="0"
              :max="10"
              :step="0.5"
              :precision="1"
              style="width: 50%"
            />
          </div>
        </el-form-item>

        <el-form-item label="DIRECTION">
          <el-select
            v-model="config.tspl.direction"
            placeholder="请选择 DIRECTION"
            style="width: 100%"
          >
            <el-option label="1,0 (推荐)" value="1,0" />
            <el-option label="1" value="1" />
            <el-option label="0,0" value="0,0" />
            <el-option label="0" value="0" />
            <el-option label="2,0" value="2,0" />
            <el-option label="3,0" value="3,0" />
          </el-select>
        </el-form-item>

        <el-divider />

        <el-alert
          type="info"
          :closable="false"
          show-icon
        >
          <template #title>
            所有打印功能（QSL标签、地址标签、顺丰面单）都将使用此打印机
          </template>
        </el-alert>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { Refresh } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import type { SinglePrinterConfig, PlatformInfo } from '@/types/models'

interface SaveStatus {
  type: 'success' | 'error'
  message: string
}

// 配置数据
const config = ref<SinglePrinterConfig>({
  printer: {
    name: ''
  },
  platform: {
    os: '',
    arch: ''
  },
  tspl: {
    gap_mm: 2,
    gap_offset_mm: 0,
    direction: '1,0'
  },
})

const availablePrinters = ref<string[]>([])
const loading = ref<boolean>(true)
const saveStatus = ref<SaveStatus | null>(null)

// 是否已完成初始加载（防止加载时触发保存）
const initialized = ref<boolean>(false)

// 计算平台显示文本
const platformDisplay = computed(() => {
  if (config.value.platform.os && config.value.platform.arch) {
    return `${config.value.platform.os} (${config.value.platform.arch})`
  }
  return '加载中...'
})

// 防抖保存
let saveTimeout: ReturnType<typeof setTimeout> | null = null
const debouncedSave = (): void => {
  // 未初始化完成时不保存
  if (!initialized.value) return

  // 清除保存状态
  saveStatus.value = null

  // 清除之前的定时器
  if (saveTimeout) {
    clearTimeout(saveTimeout)
  }

  // 设置新的定时器
  saveTimeout = setTimeout(async () => {
    // 未选择打印机时不保存
    if (!config.value.printer.name) {
      return
    }

    try {
      await invoke('save_printer_config', {
        config: config.value
      })
      saveStatus.value = { type: 'success', message: '✓ 配置已自动保存' }

      // 3秒后清除成功提示
      setTimeout(() => {
        if (saveStatus.value?.type === 'success') {
          saveStatus.value = null
        }
      }, 3000)
    } catch (error) {
      console.error('保存失败:', error)
      saveStatus.value = { type: 'error', message: `保存失败: ${error}` }
    }
  }, 500) // 500ms 防抖
}

// 监听配置变化，自动保存
watch(
  () => [config.value.printer.name, config.value.tspl.gap_mm, config.value.tspl.gap_offset_mm, config.value.tspl.direction],
  (): void => {
    debouncedSave()
  }
)

// 加载打印机配置
const loadConfig = async (): Promise<void> => {
  loading.value = true
  try {
    const savedConfig = await invoke<SinglePrinterConfig>('get_printer_config')
    config.value = savedConfig
  } catch (error) {
    console.error('加载打印机配置失败:', error)
    // 如果加载失败，获取平台信息作为基础配置
    try {
      const platform = await invoke<PlatformInfo>('get_platform_info')
      config.value.platform = platform
    } catch (platformError) {
      console.error('获取平台信息失败:', platformError)
    }
  } finally {
    loading.value = false
    // 标记初始化完成 - 使用 nextTick 确保在 watcher 执行后才设置，防止初始加载时触发保存
    nextTick(() => {
      initialized.value = true
    })
  }
}

// 加载打印机列表
const loadPrinters = async (): Promise<void> => {
  try {
    availablePrinters.value = await invoke<string[]>('get_printers')
  } catch (error) {
    console.error('获取打印机列表失败:', error)
    availablePrinters.value = []
  }
}

// 刷新打印机列表
const refreshPrinters = async (): Promise<void> => {
  await loadPrinters()
  ElMessage.success('打印机列表已刷新')
}

onMounted(async (): Promise<void> => {
  await Promise.all([
    loadConfig(),
    loadPrinters()
  ])
})
</script>
