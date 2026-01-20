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
              <el-icon><Plus /></el-icon>
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
            <div v-show="selectedConfig">
              <el-button type="primary" size="small" @click="handleSaveConfig">保存修改</el-button>
              <el-button size="small" @click="handleExportConfig">导出</el-button>
              <el-button size="small" @click="handleImportConfig">导入</el-button>
            </div>
          </div>
        </template>

        <!-- 空状态提示 -->
        <el-empty
          v-show="!selectedConfig"
          description="请从左侧选择一个配置或新建配置"
          :image-size="120"
        />

        <!-- 配置表单 -->
        <el-form v-show="selectedConfig" :model="selectedConfig" label-width="120px" style="max-width: 800px">
          <el-form-item label="配置名称">
            <el-input v-model="selectedConfig.name" />
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

          <el-form-item label="打印机型号">
            <el-input :value="selectedConfig.printer.model" disabled />
          </el-form-item>

          <el-form-item label="打印机名称">
            <div style="display: flex; gap: 10px; width: 100%">
              <el-select
                v-model="selectedConfig.printer.name"
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
                <el-icon><Refresh /></el-icon>
              </el-button>
            </div>
          </el-form-item>

          <el-form-item label="纸张规格">
            <el-input
              :value="`${selectedConfig.paper.width}mm × ${selectedConfig.paper.height}mm`"
              disabled
            />
          </el-form-item>

          <el-form-item label="打印模板">
            <el-input
              :value="`${selectedConfig.template.name} (${selectedConfig.template.version})`"
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
        <el-input v-model="newConfigForm.name" placeholder="请输入配置名称" />
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
        <el-input :value="platformInfo" disabled />
      </el-form-item>

      <el-form-item label="打印机型号">
        <el-input value="Deli DL-888C" disabled />
      </el-form-item>

      <el-form-item label="打印机名称" required>
        <el-select v-model="newConfigForm.printerName" style="width: 100%">
          <el-option
            v-for="printer in availablePrinters"
            :key="printer"
            :label="printer"
            :value="printer"
          />
        </el-select>
      </el-form-item>

      <el-form-item label="纸张规格">
        <el-input value="76mm × 130mm" disabled />
      </el-form-item>

      <el-form-item label="打印模板">
        <el-input value="标准模板 (v1)" disabled />
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="newConfigDialogVisible = false">取消</el-button>
      <el-button type="primary" @click="handleCreateConfig">创建</el-button>
    </template>
  </el-dialog>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'

const profiles = ref([])
const selectedConfigId = ref('')
const selectedConfig = ref(null)
const availablePrinters = ref([])
const defaultProfileId = ref('')
const newConfigDialogVisible = ref(false)
const platformInfo = ref('')

const newConfigForm = ref({
  name: '',
  taskName: '',
  printerName: ''
})

const loadProfiles = async () => {
  try {
    const data = await window.eel.get_profiles()()
    profiles.value = data.profiles || []
    defaultProfileId.value = data.default_id || ''

    // 自动选中配置（优先选中默认配置，否则选中第一个）
    if (profiles.value.length > 0 && !selectedConfigId.value) {
      const autoSelectId = defaultProfileId.value || profiles.value[0].id
      handleConfigSelect(autoSelectId)
    }
  } catch (error) {
    console.error('加载配置失败:', error)
    ElMessage.error('加载配置失败')
  }
}

const loadPlatformInfo = async () => {
  try {
    platformInfo.value = await window.eel.get_platform_info()()
  } catch (error) {
    console.error('获取平台信息失败:', error)
  }
}

const loadPrinters = async () => {
  try {
    availablePrinters.value = await window.eel.get_printers()()
  } catch (error) {
    console.error('获取打印机列表失败:', error)
    availablePrinters.value = []
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
      printer: { ...config.printer },
      platform: { ...config.platform },
      paper: { ...config.paper },
      template: { ...config.template }
    }
  }
}

const handleNewConfig = () => {
  newConfigForm.value = {
    name: '',
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
    const result = await window.eel.create_profile(
      newConfigForm.value.name,
      newConfigForm.value.printerName,
      newConfigForm.value.taskName || ''
    )()

    if (result.success) {
      ElMessage.success('配置创建成功')
      newConfigDialogVisible.value = false
      await loadProfiles()
    } else {
      ElMessage.error(result.error || '创建失败')
    }
  } catch (error) {
    console.error('创建配置失败:', error)
    ElMessage.error('创建失败: ' + error.message)
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
    const result = await window.eel.update_profile(selectedConfig.value)()

    if (result.success) {
      ElMessage.success('配置已保存')
      await loadProfiles()
    } else {
      ElMessage.error(result.error || '保存失败')
    }
  } catch (error) {
    console.error('保存配置失败:', error)
    ElMessage.error('保存失败: ' + error.message)
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

    const result = await window.eel.delete_profile(selectedConfigId.value)()

    if (result.success) {
      ElMessage.success('配置已删除')
      selectedConfigId.value = ''
      selectedConfig.value = null
      await loadProfiles()
    } else {
      ElMessage.error(result.error || '删除失败')
    }
  } catch (error) {
    if (error !== 'cancel') {
      console.error('删除配置失败:', error)
      ElMessage.error('删除失败: ' + error.message)
    }
  }
}

const handleSetDefault = async () => {
  if (!selectedConfigId.value) return

  try {
    const result = await window.eel.set_default_profile(selectedConfigId.value)()

    if (result.success) {
      ElMessage.success('已设为默认配置')
      await loadProfiles()
    } else {
      ElMessage.error(result.error || '设置失败')
    }
  } catch (error) {
    console.error('设置默认配置失败:', error)
    ElMessage.error('设置失败: ' + error.message)
  }
}

const handleExportConfig = async () => {
  if (!selectedConfig.value) return

  try {
    const result = await window.eel.export_profile(selectedConfigId.value)()

    if (result.success) {
      const blob = new Blob([result.data], { type: 'application/json' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `${selectedConfig.value.name}.json`
      a.click()
      URL.revokeObjectURL(url)

      ElMessage.success('配置已导出')
    } else {
      ElMessage.error(result.error || '导出失败')
    }
  } catch (error) {
    console.error('导出配置失败:', error)
    ElMessage.error('导出失败: ' + error.message)
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
      const result = await window.eel.import_profile(text)()

      if (result.success) {
        ElMessage.success('配置已导入')
        await loadProfiles()
      } else {
        ElMessage.error(result.error || '导入失败')
      }
    } catch (error) {
      console.error('导入配置失败:', error)
      ElMessage.error('导入失败: ' + error.message)
    }
  }

  input.click()
}

onMounted(async () => {
  await loadProfiles()
  await loadPlatformInfo()
  await loadPrinters()
})
</script>
