<template>
  <el-dialog
    v-model="dialogVisible"
    title="打印面单"
    width="600px"
    :close-on-click-modal="false"
    @close="handleClose"
  >
    <el-form
      ref="formRef"
      :model="form"
      :rules="rules"
      label-width="100px"
    >
      <el-form-item
        label="运单号"
        prop="waybillNo"
      >
        <el-input
          v-model="form.waybillNo"
          placeholder="请输入顺丰运单号"
          clearable
          :disabled="!!fetchedData"
        />
      </el-form-item>

      <el-form-item
        label="打印机"
        prop="printerName"
      >
        <el-select
          v-model="form.printerName"
          placeholder="请选择打印机"
          style="width: 100%"
          :loading="loadingPrinters"
        >
          <el-option
            v-for="printer in printers"
            :key="printer"
            :label="printer"
            :value="printer"
          />
        </el-select>
        <el-button
          type="primary"
          link
          size="small"
          style="margin-top: 4px"
          @click="loadPrinters"
        >
          <el-icon><Refresh /></el-icon>
          刷新打印机列表
        </el-button>
      </el-form-item>

      <!-- PDF 预览区域 -->
      <el-form-item
        v-if="fetchedData"
        label="面单预览"
      >
        <div class="preview-container">
          <img
            :src="`data:image/png;base64,${fetchedData.preview_image}`"
            alt="面单预览"
            class="preview-image"
          >
        </div>
      </el-form-item>

      <el-form-item>
        <el-alert
          v-if="status.message"
          :type="status.type"
          :closable="false"
        >
          {{ status.message }}
        </el-alert>
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="dialogVisible = false">
        取消
      </el-button>

      <!-- 步骤1：提交打印（获取PDF） -->
      <el-button
        v-if="!fetchedData"
        type="primary"
        :loading="fetching"
        :disabled="!form.waybillNo"
        @click="handleFetch"
      >
        <el-icon v-if="!fetching">
          <Download />
        </el-icon>
        提交打印
      </el-button>

      <!-- 重新获取按钮 -->
      <el-button
        v-if="fetchedData"
        @click="handleReset"
      >
        <el-icon><RefreshLeft /></el-icon>
        重新获取
      </el-button>

      <!-- 步骤2：打印（发送到打印机） -->
      <el-button
        v-if="fetchedData"
        type="success"
        :loading="printing"
        :disabled="!form.printerName"
        @click="handlePrint"
      >
        <el-icon v-if="!printing">
          <Printer />
        </el-icon>
        打印
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, nextTick, reactive, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import { useLoading } from '@/composables/useLoading'

const { withLoading } = useLoading()

interface PrinterInfo {
  name: string
  driver: string
  port: string
  status: string
  is_default: boolean
}

interface FetchWaybillResponse {
  preview_image: string
  pdf_data: string
  waybill_no: string
}

interface Props {
  visible: boolean
  defaultWaybillNo?: string
}

interface Emits {
  (e: 'update:visible', value: boolean): void
  (e: 'printed', waybillNo: string): void
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  defaultWaybillNo: ''
})

const emit = defineEmits<Emits>()

// 表单引用
const formRef = ref<FormInstance | null>(null)

// 表单数据
const form = reactive({
  waybillNo: '',
  printerName: ''
})

// 打印机列表
const printers = ref<string[]>([])
const loadingPrinters = ref<boolean>(false)

// 获取状态
const fetching = ref<boolean>(false)
const fetchedData = ref<FetchWaybillResponse | null>(null)

// 打印状态
const printing = ref<boolean>(false)

// 状态消息
const status = reactive<{
  message: string
  type: 'success' | 'warning' | 'error' | 'info'
}>({
  message: '',
  type: 'info'
})

// 表单验证规则
const rules: FormRules = {
  waybillNo: [
    { required: true, message: '请输入运单号', trigger: 'blur' }
  ]
}

// 双向绑定 visible
const dialogVisible = computed<boolean>({
  get: (): boolean => props.visible,
  set: (val: boolean): void => emit('update:visible', val)
})

// 加载打印机列表
const loadPrinters = async (): Promise<void> => {
  loadingPrinters.value = true

  try {
    const printerList = await invoke<PrinterInfo[]>('get_printers')
    printers.value = printerList.map(p => p.name)

    // 如果只有一个打印机，自动选中
    if (printers.value.length === 1 && !form.printerName) {
      form.printerName = printers.value[0]
    }

    // 尝试选择默认打印机
    const defaultPrinter = printerList.find(p => p.is_default)
    if (defaultPrinter && !form.printerName) {
      form.printerName = defaultPrinter.name
    }
  } catch (error) {
    console.error('加载打印机列表失败:', error)
    ElMessage.error(`加载打印机列表失败: ${error}`)
  } finally {
    loadingPrinters.value = false
  }
}

// 步骤1：获取面单
const handleFetch = async (): Promise<void> => {
  if (fetching.value) return

  try {
    await formRef.value?.validateField('waybillNo')
  } catch {
    return
  }

  fetching.value = true
  status.message = ''
  fetchedData.value = null

  try {
    const result = await withLoading(async () => {
      return await invoke<FetchWaybillResponse>('sf_fetch_waybill', {
        waybillNo: form.waybillNo
      })
    }, '正在获取面单...')

    fetchedData.value = result
    status.message = '面单获取成功，请确认预览后点击打印'
    status.type = 'success'
  } catch (error) {
    status.message = `获取面单失败: ${error}`
    status.type = 'error'
    ElMessage.error(`获取面单失败: ${error}`)
  } finally {
    fetching.value = false
  }
}

// 步骤2：打印面单
const handlePrint = async (): Promise<void> => {
  if (printing.value || !fetchedData.value) return

  // 检查打印机选择
  if (!form.printerName) {
    ElMessage.warning('请选择打印机')
    return
  }

  printing.value = true

  try {
    const result = await withLoading(async () => {
      return await invoke<string>('sf_print_waybill', {
        pdfData: fetchedData.value!.pdf_data,
        printerName: form.printerName
      })
    }, '正在打印面单...')

    status.message = result
    status.type = 'success'

    ElMessage.success('面单打印成功')
    emit('printed', form.waybillNo)
  } catch (error) {
    status.message = `打印失败: ${error}`
    status.type = 'error'
    ElMessage.error(`打印失败: ${error}`)
  } finally {
    printing.value = false
  }
}

// 重置获取状态
const handleReset = (): void => {
  fetchedData.value = null
  status.message = ''
}

// 关闭弹窗
const handleClose = (): void => {
  fetching.value = false
  printing.value = false
  fetchedData.value = null
  status.message = ''
}

// 监听弹窗打开
watch(() => props.visible, (newVal: boolean): void => {
  if (newVal) {
    // 重置表单
    form.waybillNo = props.defaultWaybillNo || ''
    fetchedData.value = null
    status.message = ''

    // 清除验证状态
    nextTick(() => {
      formRef.value?.clearValidate()
    })

    // 加载打印机列表
    if (printers.value.length === 0) {
      loadPrinters()
    }
  }
})
</script>

<style scoped>
.preview-container {
  width: 100%;
  display: flex;
  justify-content: center;
  align-items: flex-start;
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  background: #f5f7fa;
  padding: 12px;
}

.preview-image {
  /* 保持 76x130 的原始比例 */
  width: 228px;  /* 76mm * 3 = 228px，适合预览 */
  height: 390px; /* 130mm * 3 = 390px */
  object-fit: contain;
  background: white;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.15);
}
</style>
