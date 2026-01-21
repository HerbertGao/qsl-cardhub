<template>
  <el-dialog
      v-model="dialogVisible"
      :title="mode === 'create' ? '新建转卡' : '重命名转卡'"
      width="400px"
      :close-on-click-modal="false"
      @close="handleClose"
  >
    <el-form
        ref="formRef"
        :model="form"
        :rules="rules"
        label-width="80px"
        @submit.prevent="handleSubmit"
    >
      <el-form-item label="转卡项目" prop="name">
        <el-input
            ref="nameInputRef"
            v-model="form.name"
            placeholder="请输入转卡项目"
            maxlength="50"
            show-word-limit
            @keyup.enter="handleSubmit"
        />
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="dialogVisible = false">取消</el-button>
      <el-button type="primary" @click="handleSubmit" :loading="submitting">
        {{ mode === 'create' ? '创建' : '保存' }}
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
  mode: {
    type: String,
    default: 'create' // 'create' | 'edit'
  },
  project: {
    type: Object,
    default: null
  }
})

const emit = defineEmits(['update:visible', 'confirm'])

// 表单引用
const formRef = ref(null)
const nameInputRef = ref(null)

// 表单数据
const form = ref({
  name: ''
})

// 提交状态
const submitting = ref(false)

// 表单验证规则
const rules = {
  name: [
    {required: true, message: '转卡项目不能为空', trigger: 'blur'},
    {min: 1, max: 50, message: '转卡项目长度不能超过 50 个字符', trigger: 'blur'}
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
    if (props.mode === 'edit' && props.project) {
      form.value.name = props.project.name
    } else {
      form.value.name = ''
    }

    // 清除验证状态
    nextTick(() => {
      formRef.value?.clearValidate()
      // 聚焦输入框
      nameInputRef.value?.focus()
    })
  }
})

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
      name: form.value.name.trim()
    })
  } catch (error) {
    // 验证失败
  } finally {
    submitting.value = false
  }
}
</script>
