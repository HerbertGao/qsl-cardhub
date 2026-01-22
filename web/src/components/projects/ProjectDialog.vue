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
      <el-form-item
        label="转卡项目"
        prop="name"
      >
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
      <el-button @click="dialogVisible = false">
        取消
      </el-button>
      <el-button
        type="primary"
        :loading="submitting"
        @click="handleSubmit"
      >
        {{ mode === 'create' ? '创建' : '保存' }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'
import type { ProjectWithStats } from '@/types/models'

interface Props {
  visible: boolean
  mode: 'create' | 'edit'
  project: ProjectWithStats | null
}

interface ProjectFormData {
  name: string
}

interface ConfirmData {
  name: string
}

interface Emits {
  (e: 'update:visible', value: boolean): void
  (e: 'confirm', data: ConfirmData): void
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  mode: 'create',
  project: null
})

const emit = defineEmits<Emits>()

// 表单引用
const formRef = ref<FormInstance | null>(null)
const nameInputRef = ref<HTMLInputElement | null>(null)

// 表单数据
const form = ref<ProjectFormData>({
  name: ''
})

// 提交状态
const submitting = ref<boolean>(false)

// 表单验证规则
const rules: FormRules<ProjectFormData> = {
  name: [
    { required: true, message: '转卡项目不能为空', trigger: 'blur' },
    { min: 1, max: 50, message: '转卡项目长度不能超过 50 个字符', trigger: 'blur' }
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
const handleClose = (): void => {
  submitting.value = false
}

// 提交表单
const handleSubmit = async (): Promise<void> => {
  if (submitting.value) return

  try {
    await formRef.value!.validate()
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
