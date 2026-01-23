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
          <el-button
            type="primary"
            size="small"
            :loading="saving"
            @click="handleSaveConfig"
          >
            保存
          </el-button>
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
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { Refresh } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'
import type { SinglePrinterConfig, PlatformInfo } from '@/types/models'

// 配置数据
const config = ref<SinglePrinterConfig>({
  printer: {
    name: ''
  },
  platform: {
    os: '',
    arch: ''
  }
})

const availablePrinters = ref<string[]>([])
const loading = ref<boolean>(true)
const saving = ref<boolean>(false)

// 计算平台显示文本
const platformDisplay = computed(() => {
  if (config.value.platform.os && config.value.platform.arch) {
    return `${config.value.platform.os} (${config.value.platform.arch})`
  }
  return '加载中...'
})

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

// 保存配置
const handleSaveConfig = async (): Promise<void> => {
  if (!config.value.printer.name) {
    ElMessage.warning('请选择打印机')
    return
  }

  saving.value = true
  try {
    await invoke('save_printer_config', {
      config: config.value
    })
    ElMessage.success('打印机配置已保存')
  } catch (error) {
    console.error('保存打印机配置失败:', error)
    ElMessage.error('保存失败: ' + error)
  } finally {
    saving.value = false
  }
}

onMounted(async (): Promise<void> => {
  await Promise.all([
    loadConfig(),
    loadPrinters()
  ])
})
</script>
