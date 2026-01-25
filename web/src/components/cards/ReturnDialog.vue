<template>
  <el-dialog
    v-model="dialogVisible"
    title="退回卡片"
    width="500px"
    :close-on-click-modal="false"
    @close="handleClose"
  >
    <div
      v-if="card"
      class="detail-section"
    >
      <div class="section-title">
        基本信息
      </div>
      <el-descriptions
        :column="1"
        size="small"
        border
      >
        <el-descriptions-item label="转卡项目">
          {{ card.project_name }}
        </el-descriptions-item>
        <el-descriptions-item label="呼号">
          {{ card.callsign }}
        </el-descriptions-item>
        <el-descriptions-item label="数量">
          {{ card.qty }}
        </el-descriptions-item>
        <el-descriptions-item label="序列号">
          <span :style="{ color: card.serial ? undefined : '#909399' }">{{ formatSerial(card.serial) }}</span>
        </el-descriptions-item>
        <el-descriptions-item label="状态">
          <el-tag
            :type="getStatusType(card.status)"
            size="small"
          >
            {{ getStatusLabel(card.status) }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="录入时间">
          {{ formatDateTime(card.created_at) }}
        </el-descriptions-item>
      </el-descriptions>
    </div>

    <div class="detail-section">
      <div class="section-title">
        退回信息
      </div>
      <el-form
        ref="formRef"
        :model="form"
        :rules="rules"
        label-width="80px"
      >
        <el-form-item
          label="处理方式"
          prop="method"
        >
          <el-radio-group
            v-model="form.method"
            class="radio-group-vertical"
          >
            <el-radio
              value="NOT FOUND"
              border
            >
              NOT FOUND
            </el-radio>
            <el-radio
              value="CALLSIGN INVALID"
              border
            >
              CALLSIGN INVALID
            </el-radio>
            <el-radio
              value="REFUSED"
              border
            >
              REFUSED
            </el-radio>
            <el-radio
              value="OTHER"
              border
            >
              OTHER
            </el-radio>
          </el-radio-group>
        </el-form-item>

        <el-form-item
          label="备注"
          prop="remarks"
        >
          <el-input
            v-model="form.remarks"
            type="textarea"
            :rows="2"
            placeholder="可选，填写退回备注"
          />
          <div style="margin-top: 4px">
            <el-button
              type="primary"
              link
              size="small"
              @click="handleCopy"
            >
              <el-icon><CopyDocument /></el-icon>
              复制
            </el-button>
            <el-button
              type="primary"
              link
              size="small"
              @click="handlePaste"
            >
              <el-icon><DocumentCopy /></el-icon>
              粘贴
            </el-button>
          </div>
        </el-form-item>
      </el-form>
    </div>

    <template #footer>
      <el-button @click="dialogVisible = false">
        取消
      </el-button>
      <el-button
        type="warning"
        :loading="submitting"
        @click="handleSubmit"
      >
        确认退回
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'
import type { CardWithProject, CardStatus } from '@/types/models'
import { formatSerial } from '@/utils/format'

interface Props {
  visible: boolean
  card: CardWithProject | null
}

interface ReturnFormData {
  method: string
  remarks: string
}

interface ConfirmData {
  id: string
  method: string
  remarks: string | null
}

interface Emits {
  (e: 'update:visible', value: boolean): void
  (e: 'confirm', data: ConfirmData): void
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  card: null
})

const emit = defineEmits<Emits>()

// 表单引用
const formRef = ref<FormInstance | null>(null)

// 表单数据
const form = ref<ReturnFormData>({
  method: 'NOT FOUND',
  remarks: ''
})

// 提交状态
const submitting = ref<boolean>(false)

// 表单验证规则
const rules: FormRules<ReturnFormData> = {
  method: [
    { required: true, message: '请选择处理方式', trigger: 'change' }
  ]
}

// 双向绑定 visible
const dialogVisible = computed<boolean>({
  get: (): boolean => props.visible,
  set: (val: boolean): void => emit('update:visible', val)
})

// 监听弹窗打开
watch(() => props.visible, (newVal: boolean): void => {
  if (newVal) {
    // 检查是否有历史退回信息
    const returnInfo = props.card?.metadata?.return
    if (returnInfo && props.card?.status === 'returned') {
      // 用历史数据填充表单
      form.value = {
        method: returnInfo.method || 'NOT FOUND',
        remarks: returnInfo.remarks || ''
      }
    } else {
      // 重置表单为默认值
      form.value = {
        method: 'NOT FOUND',
        remarks: ''
      }
    }

    // 清除验证状态
    nextTick(() => {
      formRef.value?.clearValidate()
    })
  }
})

// 关闭弹窗
const handleClose = (): void => {
  submitting.value = false
}

// 获取状态标签类型
const getStatusType = (status: CardStatus): 'info' | 'success' | 'warning' => {
  const types: Record<CardStatus, 'info' | 'success' | 'warning'> = {
    pending: 'info',
    distributed: 'success',
    returned: 'warning'
  }
  return types[status] || 'info'
}

// 获取状态标签文本
const getStatusLabel = (status: CardStatus): string => {
  const labels: Record<CardStatus, string> = {
    pending: '待分发',
    distributed: '已分发',
    returned: '已退回'
  }
  return labels[status] || status
}

// 格式化时间
const formatDateTime = (datetime: string | null | undefined): string => {
  if (!datetime) return '-'
  const date = new Date(datetime)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  })
}

// 复制备注内容到剪贴板
const handleCopy = async (): Promise<void> => {
  try {
    if (form.value.remarks) {
      await navigator.clipboard.writeText(form.value.remarks)
    }
  } catch (error) {
    console.error('复制到剪贴板失败:', error)
  }
}

// 粘贴剪贴板内容
const handlePaste = async (): Promise<void> => {
  try {
    const text = await navigator.clipboard.readText()
    if (text) {
      form.value.remarks = form.value.remarks ? form.value.remarks + text : text
    }
  } catch (error) {
    console.error('读取剪贴板失败:', error)
  }
}

// 提交表单
const handleSubmit = async (): Promise<void> => {
  if (submitting.value) return

  try {
    await formRef.value!.validate()
    submitting.value = true

    emit('confirm', {
      id: props.card!.id,
      method: form.value.method,
      remarks: form.value.remarks.trim() || null
    })
  } catch (error) {
    // 验证失败
  } finally {
    submitting.value = false
  }
}
</script>

<style scoped>
.detail-section {
  margin-bottom: 20px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 12px;
  padding-left: 8px;
  border-left: 3px solid #409eff;
}

.radio-group-vertical {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px 16px;
}

.radio-group-vertical .el-radio {
  margin-right: 0;
}
</style>
