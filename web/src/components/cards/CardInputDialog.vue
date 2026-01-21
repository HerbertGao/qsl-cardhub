<template>
  <el-dialog
      v-model="dialogVisible"
      title="录入卡片"
      width="450px"
      :close-on-click-modal="false"
      @close="handleClose"
      @keydown.enter="handleSubmit"
      @keydown.esc="dialogVisible = false"
  >
    <el-form
        ref="formRef"
        :model="form"
        :rules="rules"
        label-width="80px"
    >
      <el-form-item label="转卡" prop="projectId">
        <el-select
            v-model="form.projectId"
            placeholder="请选择转卡"
            style="width: 100%"
            :disabled="!!preselectedProjectId"
        >
          <el-option
              v-for="project in projects"
              :key="project.id"
              :label="project.name"
              :value="project.id"
          />
        </el-select>
      </el-form-item>

      <el-form-item label="呼号" prop="callsign">
        <el-input
            ref="callsignInputRef"
            v-model="form.callsign"
            placeholder="请输入呼号（3-10 字符）"
            maxlength="10"
            show-word-limit
            @input="handleCallsignInput"
        />
      </el-form-item>

      <el-form-item label="数量" prop="qty">
        <el-input-number
            v-model="form.qty"
            :min="1"
            :max="9999"
            controls-position="right"
            style="width: 100%"
        />
      </el-form-item>

      <el-form-item>
        <el-checkbox v-model="continuousMode">连续录入模式</el-checkbox>
        <el-tooltip content="提交后保持弹窗打开，继续录入下一条" placement="top">
          <el-icon style="margin-left: 4px; color: #909399">
            <QuestionFilled/>
          </el-icon>
        </el-tooltip>
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="dialogVisible = false">取消 (Esc)</el-button>
      <el-button type="primary" @click="handleSubmit" :loading="submitting">
        录入 (Enter)
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup>
import {computed, nextTick, ref, watch} from 'vue'

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  projects: {
    type: Array,
    default: () => []
  },
  preselectedProjectId: {
    type: String,
    default: null
  }
})

const emit = defineEmits(['update:visible', 'confirm'])

// 表单引用
const formRef = ref(null)
const callsignInputRef = ref(null)

// 表单数据
const form = ref({
  projectId: '',
  callsign: '',
  qty: 1
})

// 连续录入模式
const continuousMode = ref(false)

// 提交状态
const submitting = ref(false)

// 呼号验证正则
const callsignPattern = /^[A-Za-z0-9/]{3,10}$/

// 表单验证规则
const rules = {
  projectId: [
    {required: true, message: '请选择转卡', trigger: 'change'}
  ],
  callsign: [
    {required: true, message: '请输入呼号', trigger: 'blur'},
    {
      validator: (rule, value, callback) => {
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
    {required: true, message: '请输入数量', trigger: 'blur'},
    {type: 'number', min: 1, max: 9999, message: '数量必须在 1-9999 之间', trigger: 'blur'}
  ]
}

// 双向绑定 visible
const dialogVisible = computed({
  get: () => props.visible,
  set: (val) => emit('update:visible', val)
})

// 监听弹窗打开
watch(() => props.visible, (newVal) => {
  if (newVal) {
    // 重置表单
    form.value = {
      projectId: props.preselectedProjectId || '',
      callsign: '',
      qty: 1
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
const handleCallsignInput = () => {
  form.value.callsign = form.value.callsign.toUpperCase()
}

// 关闭弹窗
const handleClose = () => {
  submitting.value = false
}

// 提交表单
const handleSubmit = async () => {
  if (submitting.value) return

  try {
    await formRef.value.validate()
    submitting.value = true

    emit('confirm', {
      projectId: form.value.projectId,
      callsign: form.value.callsign.trim().toUpperCase(),
      qty: form.value.qty,
      continuousMode: continuousMode.value
    })
  } catch (error) {
    // 验证失败
  } finally {
    submitting.value = false
  }
}

// 重置表单（连续录入模式使用）
const resetForContinuous = () => {
  form.value.callsign = ''
  form.value.qty = 1
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
