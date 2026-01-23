<template>
  <el-dialog
    v-model="dialogVisible"
    title="录入卡片"
    width="500px"
    :close-on-click-modal="false"
    @close="handleClose"
    @keydown.enter="handleSubmit"
    @keydown.esc="dialogVisible = false"
  >
    <el-form
      ref="formRef"
      :model="form"
      :rules="rules"
      label-width="100px"
    >
      <el-form-item
        label="转卡"
        prop="projectId"
      >
        <el-select
          v-model="form.projectId"
          placeholder="请选择转卡"
          style="width: 100%"
          :disabled="!!preselectedProjectId"
          @change="handleProjectChange"
        >
          <el-option
            v-for="project in projects"
            :key="project.id"
            :label="project.name"
            :value="project.id"
          />
        </el-select>
      </el-form-item>

      <el-form-item
        label="呼号"
        prop="callsign"
      >
        <el-input
          ref="callsignInputRef"
          v-model="form.callsign"
          placeholder="请输入呼号（3-10 字符）"
          maxlength="10"
          show-word-limit
          @input="handleCallsignInput"
        />
      </el-form-item>

      <el-form-item
        label="数量"
        prop="qty"
      >
        <el-input-number
          v-model="form.qty"
          :min="1"
          :max="9999"
        />
      </el-form-item>

      <!-- 序列号区域 -->
      <el-divider content-position="left">
        序列号
      </el-divider>

      <el-form-item label="序列号">
        <div style="display: flex; gap: 10px; align-items: center">
          <el-input-number
            v-model="serialNumber"
            :min="1"
            :max="999"
            @change="handleSerialChange"
          />
          <el-button
            size="small"
            @click="handleResetSerial"
          >
            重置
          </el-button>
          <span style="color: #909399; font-size: 12px">
            预览: {{ serialPreview }}
          </span>
        </div>
      </el-form-item>

      <el-form-item label="">
        <el-checkbox v-model="skipFour">
          跳过包含数字 4 的序列号
        </el-checkbox>
      </el-form-item>

      <!-- 打印设置区域 -->
      <el-divider content-position="left">
        打印设置
      </el-divider>

      <el-form-item label="录入后打印">
        <el-checkbox v-model="printAfterSave">
          打印标签
        </el-checkbox>
        <el-tooltip
          content="录入成功后自动打印卡片标签"
          placement="top"
        >
          <el-icon style="margin-left: 4px; color: #909399">
            <QuestionFilled />
          </el-icon>
        </el-tooltip>
      </el-form-item>

      <el-divider />

      <el-form-item label="">
        <el-checkbox v-model="continuousMode">
          连续录入模式
        </el-checkbox>
        <el-tooltip
          content="提交后保持弹窗打开，继续录入下一条，序列号自动递增"
          placement="top"
        >
          <el-icon style="margin-left: 4px; color: #909399">
            <QuestionFilled />
          </el-icon>
        </el-tooltip>
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="dialogVisible = false">
        取消 (Esc)
      </el-button>
      <el-button
        type="primary"
        :loading="submitting"
        @click="handleSubmit"
      >
        {{ printAfterSave ? '录入并打印 (Enter)' : '录入 (Enter)' }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import type { ProjectWithStats, SinglePrinterConfig } from '@/types/models'
import { formatSerial } from '@/utils/format'

interface Props {
  visible: boolean
  projects: ProjectWithStats[]
  preselectedProjectId: string | null
}

interface CardInputFormData {
  projectId: string
  callsign: string
  qty: number
}

interface ConfirmData {
  projectId: string
  callsign: string
  qty: number
  serial: number | null
  continuousMode: boolean
  printAfterSave: boolean
  printerName: string | null
}

interface Emits {
  (e: 'update:visible', value: boolean): void
  (e: 'confirm', data: ConfirmData): void
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  projects: () => [],
  preselectedProjectId: null
})

const emit = defineEmits<Emits>()

// 表单引用
const formRef = ref<FormInstance | null>(null)
const callsignInputRef = ref<HTMLInputElement | null>(null)

// 表单数据
const form = ref<CardInputFormData>({
  projectId: '',
  callsign: '',
  qty: 1
})

// 序列号（单独管理，使用数字类型）
const serialNumber = ref<number>(1)

// 上一个序列号值（用于判断增减方向）
const previousSerial = ref<number>(1)

// 连续录入模式
const continuousMode = ref<boolean>(false)

// 跳过包含数字4的序列号（默认勾选）
const skipFour = ref<boolean>(true)

// 打印设置（默认勾选）
const printAfterSave = ref<boolean>(true)

// 下一个序列号（自动计算的基础值）
const nextSerial = ref<number | null>(null)

// 打印机名称（从配置获取）
const printerName = ref<string | null>(null)

// 提交状态
const submitting = ref<boolean>(false)

// 呼号验证正则
const callsignPattern: RegExp = /^[A-Za-z0-9/]{3,10}$/

// 表单验证规则
const rules: FormRules<CardInputFormData> = {
  projectId: [
    { required: true, message: '请选择转卡', trigger: 'change' }
  ],
  callsign: [
    { required: true, message: '请输入呼号', trigger: 'blur' },
    {
      validator: (_rule, value: string, callback: (error?: Error) => void): void => {
        if (!value) {
          callback()
        } else if (!callsignPattern.test(value)) {
          callback(new Error('呼号格式无效：3-10 个字符，仅包含字母、数字、斜杠'))
        } else {
          callback()
        }
      },
      trigger: 'blur'
    }
  ],
  qty: [
    { required: true, message: '请输入数量', trigger: 'blur' },
    { type: 'number', min: 1, max: 9999, message: '数量必须在 1-9999 之间', trigger: 'blur' }
  ]
}

// 双向绑定 visible
const dialogVisible = computed<boolean>({
  get: (): boolean => props.visible,
  set: (val: boolean): void => emit('update:visible', val)
})

// 计算下一个序列号（跳过包含4的数字）
const calculateNextSerial = (current: number | null, skip4: boolean): number => {
  let next = (current || 0) + 1
  // 超过999时重置为1
  if (next > 999) {
    next = 1
  }
  if (skip4) {
    // 跳过包含数字4的序列号
    while (String(next).includes('4')) {
      next++
      // 超过999时重置为1
      if (next > 999) {
        next = 1
      }
    }
  }
  return next
}

// 跳过包含数字 4 的序列号（向上）
const skip4Up = (number: number): number => {
  while (String(number).includes('4')) {
    number++
    // 超过999时重置为1
    if (number > 999) {
      number = 1
    }
  }
  return number
}

// 跳过包含数字 4 的序列号（向下）
const skip4Down = (number: number): number => {
  while (String(number).includes('4') && number > 1) {
    number--
  }
  return number
}

// 处理序列号变化（用户手动修改时应用跳过4逻辑）
const handleSerialChange = (value: number | null): void => {
  if (!value) return

  if (skipFour.value) {
    let skipped = value

    // 判断是增加还是减少
    if (value > previousSerial.value) {
      // 向上增加，使用正向跳过
      skipped = skip4Up(value)
    } else if (value < previousSerial.value) {
      // 向下减少，使用反向跳过
      skipped = skip4Down(value)
    }

    if (skipped !== value) {
      serialNumber.value = skipped
    }

    // 更新上一个值
    previousSerial.value = serialNumber.value
  } else {
    // 未勾选跳过4时，也要更新上一个值
    previousSerial.value = value
  }
}

// 序列号预览
const serialPreview = computed(() => {
  if (serialNumber.value > 0) {
    return formatSerial(serialNumber.value)
  }
  return '---'
})

// 加载打印机配置
const loadPrinterConfig = async (): Promise<void> => {
  try {
    const config = await invoke<SinglePrinterConfig>('get_printer_config')
    printerName.value = config.printer.name || null
  } catch (error) {
    console.error('加载打印机配置失败:', error)
    printerName.value = null
  }
}

// 加载项目的最大序列号
const loadMaxSerial = async (projectId: string): Promise<void> => {
  if (!projectId) {
    nextSerial.value = null
    serialNumber.value = 1
    previousSerial.value = 1
    return
  }

  try {
    const maxSerial = await invoke<number | null>('get_max_serial_cmd', { projectId })
    nextSerial.value = calculateNextSerial(maxSerial, skipFour.value)
    serialNumber.value = nextSerial.value
    previousSerial.value = nextSerial.value
  } catch (error) {
    console.error('加载最大序列号失败:', error)
    nextSerial.value = calculateNextSerial(null, skipFour.value)
    serialNumber.value = nextSerial.value
    previousSerial.value = nextSerial.value
  }
}

// 重置序列号到 1
const handleResetSerial = (): void => {
  serialNumber.value = 1
  previousSerial.value = 1
  ElMessage.success('序列号已重置')
}

// 项目选择变化时加载序列号
const handleProjectChange = (projectId: string): void => {
  loadMaxSerial(projectId)
}

// 监听跳过4选项变化，重新计算序列号
watch(skipFour, (): void => {
  if (form.value.projectId) {
    // 重新加载序列号
    loadMaxSerial(form.value.projectId)
  }
})

// 监听弹窗打开
watch(() => props.visible, (newVal: boolean): void => {
  if (newVal) {
    // 重置表单
    form.value = {
      projectId: props.preselectedProjectId || '',
      callsign: '',
      qty: 1
    }
    serialNumber.value = 1
    previousSerial.value = 1

    // 加载打印机配置
    loadPrinterConfig()

    // 如果有预选项目，加载序列号
    if (props.preselectedProjectId) {
      loadMaxSerial(props.preselectedProjectId)
    }

    // 清除验证状态
    nextTick(() => {
      formRef.value?.clearValidate()
      // 聚焦呼号输入框
      callsignInputRef.value?.focus()
    })
  }
})

// 呼号输入处理（自动大写）
const handleCallsignInput = (): void => {
  form.value.callsign = form.value.callsign.toUpperCase()
}

// 关闭弹窗
const handleClose = (): void => {
  submitting.value = false
}

// 提交表单
const handleSubmit = async (): Promise<void> => {
  if (submitting.value) return

  try {
    await formRef.value!.validate()

    // 如果需要打印但未配置打印机，提示用户
    if (printAfterSave.value && !printerName.value) {
      ElMessage.warning('请先在「打印配置」中配置打印机')
      return
    }

    submitting.value = true

    emit('confirm', {
      projectId: form.value.projectId,
      callsign: form.value.callsign.trim().toUpperCase(),
      qty: form.value.qty,
      serial: serialNumber.value > 0 ? serialNumber.value : null,
      continuousMode: continuousMode.value,
      printAfterSave: printAfterSave.value,
      printerName: printerName.value
    })
  } catch (error) {
    // 验证失败
  } finally {
    submitting.value = false
  }
}

// 重置表单（连续录入模式使用）
const resetForContinuous = (): void => {
  form.value.callsign = ''
  form.value.qty = 1

  // 序列号自动递增（无论是否打印）
  if (form.value.projectId) {
    loadMaxSerial(form.value.projectId)
  }

  nextTick(() => {
    formRef.value?.clearValidate(['callsign', 'qty'])
    callsignInputRef.value?.focus()
  })
}

// 暴露方法供父组件调用
defineExpose({
  resetForContinuous
})
</script>
