<template>
  <div class="page-content">
    <h1>配置文件管理</h1>

    <div style="display: flex; gap: 20px; margin-top: 30px">
      <!-- 左侧列表 -->
      <el-card style="width: 300px; flex-shrink: 0" shadow="hover">
        <template #header>
          <div style="display: flex; justify-content: space-between; align-items: center">
            <span style="font-weight: bold">配置列表</span>
            <el-button type="primary" size="small" @click="handleNewConfig">
              <el-icon>
                <Plus/>
              </el-icon>
              新建
            </el-button>
          </div>
        </template>

        <el-menu :default-active="selectedConfigId" @select="handleConfigSelect">
          <el-menu-item v-for="profile in profiles" :key="profile.id" :index="profile.id">
            <span>{{ profile.name }}</span>
            <el-tag
                v-if="profile.id === defaultProfileId"
                size="small"
                type="success"
                style="margin-left: 10px"
            >
              默认
            </el-tag>
          </el-menu-item>
        </el-menu>

        <div style="margin-top: 15px; display: flex; gap: 10px">
          <el-button
              size="small"
              type="danger"
              @click="handleDeleteConfig"
              :disabled="!selectedConfigId"
              style="flex: 1"
          >
            删除
          </el-button>
          <el-button
              size="small"
              type="success"
              @click="handleSetDefault"
              :disabled="!selectedConfigId"
              style="flex: 1"
          >
            设为默认
          </el-button>
        </div>
      </el-card>

      <!-- 右侧详情 -->
      <el-card style="flex: 1" shadow="hover">
        <template #header>
          <div style="display: flex; justify-content: space-between; align-items: center">
            <span style="font-weight: bold">配置详情</span>
            <div v-if="selectedConfig">
              <el-button type="primary" size="small" @click="handleSaveConfig">保存修改</el-button>
            </div>
          </div>
        </template>

        <!-- 空状态提示 -->
        <el-empty
            v-if="!selectedConfig"
            description="请从左侧选择一个配置或新建配置"
            :image-size="120"
        />

        <!-- 配置表单 -->
        <el-form v-else-if="selectedConfig" :model="selectedConfig" label-width="120px" style="max-width: 800px">
          <el-form-item label="配置名称">
            <el-input v-model="selectedConfig.name"/>
          </el-form-item>

          <el-form-item label="任务名称">
            <el-input
                v-model="selectedConfig.task_name"
                placeholder="请输入任务名称（可选）"
                maxlength="50"
                show-word-limit
                clearable
            />
          </el-form-item>

          <el-form-item label="操作系统">
            <el-input
                :value="`${selectedConfig.platform.os} (${selectedConfig.platform.arch})`"
                disabled
            />
          </el-form-item>

          <el-form-item label="打印机名称">
            <div style="display: flex; gap: 10px; width: 100%">
              <el-select
                  v-model="selectedConfig.printer.name"
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
                  <Refresh/>
                </el-icon>
              </el-button>
            </div>
          </el-form-item>

          <el-form-item label="打印模板">
            <el-input
                :value="selectedConfig.template_display_name || selectedConfig.template.path"
                disabled
            />
          </el-form-item>
        </el-form>
      </el-card>
    </div>
  </div>

  <!-- 新建配置对话框 -->
  <el-dialog v-model="newConfigDialogVisible" title="新建配置文件" width="500px">
    <el-form :model="newConfigForm" label-width="120px">
      <el-form-item label="配置名称" required>
        <el-input v-model="newConfigForm.name" placeholder="请输入配置名称"/>
      </el-form-item>

      <el-form-item label="任务名称">
        <el-input
            v-model="newConfigForm.taskName"
            placeholder="请输入任务名称（可选）"
            maxlength="50"
            show-word-limit
            clearable
        />
      </el-form-item>

      <el-form-item label="操作系统">
        <el-input :value="platformInfo" disabled/>
      </el-form-item>

      <el-form-item label="打印机名称" required>
        <el-select v-model="newConfigForm.printerName" placeholder="请选择打印机" style="width: 100%">
          <el-option
              v-for="printer in availablePrinters"
              :key="printer"
              :label="printer"
              :value="printer"
          />
        </el-select>
      </el-form-item>

      <el-form-item label="打印模板">
        <el-input :value="defaultTemplateName" disabled/>
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="newConfigDialogVisible = false">取消</el-button>
      <el-button type="primary" @click="handleCreateConfig">创建</el-button>
    </template>
  </el-dialog>
</template>

<script setup>
import {nextTick, onMounted, ref} from 'vue'
import {ElMessage, ElMessageBox} from 'element-plus'
import {Plus, Refresh} from '@element-plus/icons-vue'
import {invoke} from '@tauri-apps/api/core'

// Props
const props = defineProps({
  autoOpenNewDialog: {
    type: Boolean,
    default: false
  }
})

// 生成默认配置名称（格式：配置YYYYMMDD）
const getDefaultConfigName = () => {
  const now = new Date()
  const year = now.getFullYear()
  const month = String(now.getMonth() + 1).padStart(2, '0')
  const day = String(now.getDate()).padStart(2, '0')
  return `配置${year}${month}${day}`
}

const profiles = ref([])
const selectedConfigId = ref('')
const selectedConfig = ref(null)
const availablePrinters = ref([])
const defaultProfileId = ref('')
const newConfigDialogVisible = ref(false)
const platformInfo = ref('')
const defaultTemplateName = ref('加载中...')

const newConfigForm = ref({
  name: '',
  taskName: '',
  printerName: ''
})

const loadProfiles = async () => {
  try {
    const allProfiles = await invoke('get_profiles')
    profiles.value = allProfiles || []

    const defaultId = await invoke('get_default_profile_id')
    defaultProfileId.value = defaultId || ''

    // 自动选中配置（优先选中默认配置，否则选中第一个）
    if (profiles.value.length > 0 && !selectedConfigId.value) {
      const autoSelectId = defaultProfileId.value || profiles.value[0].id
      handleConfigSelect(autoSelectId)
    }
  } catch (error) {
    console.error('加载配置失败:', error)
    ElMessage.error('加载配置失败: ' + error)
  }
}

const loadPlatformInfo = async () => {
  try {
    const info = await invoke('get_platform_info')
    platformInfo.value = `${info.os} (${info.arch})`
  } catch (error) {
    console.error('获取平台信息失败:', error)
  }
}

const loadPrinters = async () => {
  try {
    availablePrinters.value = await invoke('get_printers')
  } catch (error) {
    console.error('获取打印机列表失败:', error)
    availablePrinters.value = []
  }
}

const loadDefaultTemplate = async () => {
  try {
    defaultTemplateName.value = await invoke('get_default_template_name')
  } catch (error) {
    console.error('获取默认模板失败:', error)
    defaultTemplateName.value = '默认模板'
  }
}

const refreshPrinters = async () => {
  await loadPrinters()
  ElMessage.success('打印机列表已刷新')
}

const handleConfigSelect = (configId) => {
  selectedConfigId.value = configId
  const config = profiles.value.find(p => p.id === configId)
  if (config) {
    // 使用响应式拷贝，确保深层属性也是响应式的
    selectedConfig.value = {
      ...config,
      task_name: config.task_name || '',
      printer: {...config.printer},
      platform: {...config.platform},
      template: {...config.template},
      template_display_name: config.template_display_name
    }
  }
}

const handleNewConfig = () => {
  newConfigForm.value = {
    name: getDefaultConfigName(),
    taskName: '',
    printerName: availablePrinters.value[0] || ''
  }
  newConfigDialogVisible.value = true
}

const handleCreateConfig = async () => {
  if (!newConfigForm.value.name.trim()) {
    ElMessage.warning('请输入配置名称')
    return
  }
  if (!newConfigForm.value.printerName) {
    ElMessage.warning('请选择打印机')
    return
  }
  if (newConfigForm.value.taskName && newConfigForm.value.taskName.length > 50) {
    ElMessage.warning('任务名称最长 50 字符')
    return
  }

  try {
    // 获取平台信息
    const platform = await invoke('get_platform_info')

    // 创建配置
    await invoke('create_profile', {
      name: newConfigForm.value.name,
      taskName: newConfigForm.value.taskName?.trim() || null,
      printerName: newConfigForm.value.printerName,
      platform: platform
    })

    ElMessage.success('配置创建成功')
    newConfigDialogVisible.value = false
    await loadProfiles()
  } catch (error) {
    console.error('创建配置失败:', error)
    ElMessage.error('创建失败: ' + error)
  }
}

const handleSaveConfig = async () => {
  if (!selectedConfig.value) return

  // 验证任务名称长度
  if (selectedConfig.value.task_name && selectedConfig.value.task_name.length > 50) {
    ElMessage.warning('任务名称最长 50 字符')
    return
  }

  try {
    await invoke('update_profile', {
      id: selectedConfig.value.id,
      profile: selectedConfig.value
    })

    ElMessage.success('配置已保存')
    await loadProfiles()
  } catch (error) {
    console.error('保存配置失败:', error)
    ElMessage.error('保存失败: ' + error)
  }
}

const handleDeleteConfig = async () => {
  if (!selectedConfigId.value) return

  try {
    await ElMessageBox.confirm('确定要删除此配置吗？', '删除确认', {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning'
    })

    await invoke('delete_profile', {
      id: selectedConfigId.value
    })

    ElMessage.success('配置已删除')
    selectedConfigId.value = ''
    selectedConfig.value = null
    await loadProfiles()
  } catch (error) {
    if (error !== 'cancel') {
      console.error('删除配置失败:', error)
      ElMessage.error('删除失败: ' + error)
    }
  }
}

const handleSetDefault = async () => {
  if (!selectedConfigId.value) return

  try {
    await invoke('set_default_profile', {
      id: selectedConfigId.value
    })

    ElMessage.success('已设为默认配置')
    await loadProfiles()
  } catch (error) {
    console.error('设置默认配置失败:', error)
    ElMessage.error('设置失败: ' + error)
  }
}

const handleExportConfig = async () => {
  if (!selectedConfig.value) return

  try {
    const jsonData = await invoke('export_profile', {
      id: selectedConfigId.value
    })

    const blob = new Blob([jsonData], {type: 'application/json'})
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${selectedConfig.value.name}.json`
    a.click()
    URL.revokeObjectURL(url)

    ElMessage.success('配置已导出')
  } catch (error) {
    console.error('导出配置失败:', error)
    ElMessage.error('导出失败: ' + error)
  }
}

const handleImportConfig = () => {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.json'

  input.onchange = async (e) => {
    const file = e.target.files[0]
    if (!file) return

    try {
      const text = await file.text()
      await invoke('import_profile', {
        json: text
      })

      ElMessage.success('配置已导入')
      await loadProfiles()
    } catch (error) {
      console.error('导入配置失败:', error)
      ElMessage.error('导入失败: ' + error)
    }
  }

  input.click()
}

onMounted(async () => {
  await loadProfiles()
  await loadPlatformInfo()
  await loadPrinters()
  await loadDefaultTemplate()

  // 如果需要自动打开新建配置对话框
  if (props.autoOpenNewDialog) {
    await nextTick()
    handleNewConfig()
  }
})
</script>
