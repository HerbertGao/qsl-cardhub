<template>
  <div class="page-content">
    <div class="page-header">
      <h1 style="margin: 0">QSL 卡片打印</h1>
      <div style="display: flex; gap: 10px; align-items: center">
        <el-select
          v-model="currentProfile"
          placeholder="选择配置"
          size="default"
          style="width: 200px"
        >
          <template #prefix>
            <span style="color: #999; font-size: 12px">配置:</span>
          </template>
          <el-option
            v-for="profile in profiles"
            :key="profile.id"
            :label="profile.name"
            :value="profile.id"
          />
        </el-select>
        <el-button
          size="default"
          :loading="loadingProfiles"
          :icon="Refresh"
          @click="handleRefreshProfiles"
          title="刷新配置列表"
        />
      </div>
    </div>

    <!-- 任务名称显示 -->
    <div v-if="currentProfileInfo" style="margin-top: 15px; padding-left: 0">
      <el-text type="info" size="small">任务名称：</el-text>
      <el-text size="default">{{ currentProfileInfo.task_name || '未设置' }}</el-text>
    </div>

    <el-card shadow="hover" style="margin-top: 30px">
      <template #header>
        <div style="display: flex; align-items: center">
          <el-icon style="margin-right: 8px"><Edit /></el-icon>
          <span style="font-weight: bold">输入信息</span>
        </div>
      </template>

      <el-form :model="printForm" label-width="160px" style="max-width: 600px">
        <el-form-item label="呼号 / CALLSIGN" required>
          <el-input
            v-model="printForm.callsign"
            placeholder="请输入呼号"
            clearable
          />
        </el-form-item>

        <el-form-item label="数量 / QTY" required>
          <el-input-number
            v-model="printForm.qty"
            :min="1"
            :max="100"
            style="width: 100%"
          />
        </el-form-item>

        <el-form-item label="序列号 / SERIAL">
          <div style="display: flex; gap: 10px; align-items: center">
            <el-input-number
              v-model="printForm.serial"
              :min="1"
              :max="999"
              style="flex: 1"
              @change="handleSerialChange"
            />
            <el-tag type="info" size="large">预览: {{ formattedSerial }}</el-tag>
            <el-button @click="resetSerial">重置</el-button>
          </div>
        </el-form-item>

        <el-form-item label=" ">
          <el-checkbox v-model="skip4Enabled">
            跳过包含数字 4 的序列号
          </el-checkbox>
          <el-tooltip
            content="勾选后，序列号会自动跳过所有包含数字 4 的数字（如 4, 14, 40-49, 140-149 等）"
            placement="right"
          >
            <el-icon style="margin-left: 5px; cursor: help">
              <QuestionFilled />
            </el-icon>
          </el-tooltip>
        </el-form-item>

        <el-form-item style="text-align: center">
          <el-button-group>
            <el-button
              type="primary"
              size="default"
              :loading="printing"
              @click="handlePrint"
              style="width: 150px"
            >
              <el-icon style="margin-right: 8px"><Printer /></el-icon>
              {{ printing ? '打印中...' : '打印' }}
            </el-button>
            <el-dropdown trigger="click" @command="handleDropdownCommand">
              <el-button type="primary" size="default">
                <el-icon><ArrowDown /></el-icon>
              </el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="calibration">打印校准页</el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </el-button-group>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- 打印机信息 -->
    <el-alert
      v-if="currentProfileInfo"
      :title="`打印机: ${currentProfileInfo.printer.name} | 纸张: ${currentProfileInfo.paper.width}×${currentProfileInfo.paper.height}mm`"
      type="info"
      :closable="false"
      style="margin-top: 20px"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { Refresh, QuestionFilled } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api/core'

const profiles = ref([])
const currentProfile = ref('')
const defaultProfileId = ref('')
const printing = ref(false)
const loadingProfiles = ref(false)
const skip4Enabled = ref(true)  // 默认勾选
const previousSerial = ref(1)

const printForm = ref({
  callsign: '',
  serial: 1,
  qty: 1
})

const currentProfileInfo = computed(() => {
  if (!currentProfile.value || profiles.value.length === 0) {
    return null
  }
  const profile = profiles.value.find(p => p.id === currentProfile.value)
  if (!profile) return null

  return profile
})

const formattedSerial = computed(() => {
  return String(printForm.value.serial).padStart(3, '0')
})

// 跳过包含数字 4 的序列号（向上）
const skip4 = (number) => {
  while (String(number).includes('4')) {
    number++
  }
  return number
}

// 跳过包含数字 4 的序列号（向下）
const skip4Down = (number) => {
  while (String(number).includes('4') && number > 1) {
    number--
  }
  return number
}

const loadProfiles = async () => {
  loadingProfiles.value = true
  try {
    // 获取所有配置
    const allProfiles = await invoke('get_profiles')
    profiles.value = allProfiles || []

    // 获取默认配置 ID
    const defaultId = await invoke('get_default_profile_id')
    defaultProfileId.value = defaultId || ''

    if (defaultProfileId.value && !currentProfile.value) {
      currentProfile.value = defaultProfileId.value
    }
  } catch (error) {
    console.error('加载配置失败:', error)
    ElMessage.error('加载配置失败: ' + error)
  } finally {
    loadingProfiles.value = false
  }
}

const handleRefreshProfiles = async () => {
  await loadProfiles()
  ElMessage.success('配置列表已刷新')
}

const handlePrint = async () => {
  if (!printForm.value.callsign.trim()) {
    ElMessage.warning('请输入呼号')
    return
  }
  if (!printForm.value.qty || printForm.value.qty < 1) {
    ElMessage.warning('请输入数量')
    return
  }
  if (!currentProfile.value) {
    ElMessage.warning('请选择配置')
    return
  }

  // 获取当前 Profile 的打印机名称
  const profile = currentProfileInfo.value
  if (!profile) {
    ElMessage.warning('配置信息不完整')
    return
  }

  printing.value = true
  try {
    // 应用跳过 4 逻辑
    let currentSerial = printForm.value.serial
    if (skip4Enabled.value) {
      currentSerial = skip4(currentSerial)
    }

    // 调用 Tauri API
    await invoke('print_qsl', {
      printerName: profile.printer.name,
      callsign: printForm.value.callsign,
      serial: currentSerial,
      qty: printForm.value.qty
    })

    ElMessage.success('打印成功')
    // 清空呼号，序列号自动增长
    printForm.value.callsign = ''
    currentSerial = currentSerial + 1

    // 应用跳过 4 逻辑到递增后的序列号
    if (skip4Enabled.value) {
      currentSerial = skip4(currentSerial)
    }

    printForm.value.serial = currentSerial

    // 序列号超过999则重置为1
    if (printForm.value.serial > 999) {
      printForm.value.serial = 1
    }

    // 更新上一个值
    previousSerial.value = printForm.value.serial
  } catch (error) {
    console.error('打印失败:', error)
    ElMessage.error('打印失败: ' + error)
  } finally {
    printing.value = false
  }
}

const resetSerial = () => {
  printForm.value.serial = 1
  previousSerial.value = 1
  ElMessage.success('序列号已重置')
}

const handleSerialChange = (value) => {
  // 如果勾选了"跳过4"，自动跳过包含4的数字
  if (skip4Enabled.value && value) {
    let skipped = value

    // 判断是增加还是减少
    if (value > previousSerial.value) {
      // 向上增加，使用正向跳过
      skipped = skip4(value)
    } else if (value < previousSerial.value) {
      // 向下减少，使用反向跳过
      skipped = skip4Down(value)
    }

    if (skipped !== value) {
      printForm.value.serial = skipped
    }

    // 更新上一个值
    previousSerial.value = printForm.value.serial
  } else {
    // 未勾选跳过4时，也要更新上一个值
    previousSerial.value = value
  }
}

const handlePrintCalibration = async () => {
  if (!currentProfile.value) {
    ElMessage.warning('请选择配置')
    return
  }

  const profile = currentProfileInfo.value
  if (!profile) {
    ElMessage.warning('配置信息不完整')
    return
  }

  try {
    await invoke('print_calibration', {
      printerName: profile.printer.name
    })
    ElMessage.success('校准页打印成功')
  } catch (error) {
    console.error('打印校准页失败:', error)
    ElMessage.error('打印失败: ' + error)
  }
}

const handleDropdownCommand = (command) => {
  if (command === 'calibration') {
    handlePrintCalibration()
  }
}

onMounted(() => {
  loadProfiles()
})
</script>
