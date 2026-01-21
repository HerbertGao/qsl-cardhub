<template>
  <el-dialog
      v-model="dialogVisible"
      title="分发卡片"
      width="700px"
      :close-on-click-modal="false"
      @close="handleClose"
  >
    <div class="distribute-content">
      <!-- 左侧：卡片信息和表单 -->
      <div class="left-panel">
        <div class="detail-section" v-if="card">
          <div class="section-title">基本信息</div>
          <el-descriptions :column="1" size="small" border>
            <el-descriptions-item label="转卡项目">{{ card.project_name }}</el-descriptions-item>
            <el-descriptions-item label="呼号">{{ card.callsign }}</el-descriptions-item>
            <el-descriptions-item label="数量">{{ card.qty }}</el-descriptions-item>
            <el-descriptions-item label="状态">
              <el-tag :type="getStatusType(card.status)" size="small">
                {{ getStatusLabel(card.status) }}
              </el-tag>
            </el-descriptions-item>
            <el-descriptions-item label="录入时间">
              {{ formatDateTime(card.created_at) }}
            </el-descriptions-item>
          </el-descriptions>
        </div>

        <div class="detail-section">
          <div class="section-title">分发信息</div>
          <el-form
              ref="formRef"
              :model="form"
              :rules="rules"
              label-width="80px"
          >
            <el-form-item label="处理方式" prop="method">
              <el-radio-group v-model="form.method" class="radio-group-vertical">
                <el-radio value="快递" border>快递</el-radio>
                <el-radio value="挂号信" border>挂号信</el-radio>
                <el-radio value="平邮" border>平邮</el-radio>
                <el-radio value="自取" border>自取</el-radio>
                <el-radio value="代领" border>代领</el-radio>
                <el-radio value="其它" border>其它</el-radio>
              </el-radio-group>
            </el-form-item>

            <el-form-item label="备注" prop="remarks">
              <el-input
                  v-model="form.remarks"
                  type="textarea"
                  :rows="3"
                  placeholder="可选，填写备注信息"
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
      </div>

      <!-- 右侧：收件地址 -->
      <div class="right-panel">
        <div class="section-title">收件地址</div>
        <div class="address-content">
          <template v-if="recipientAddress">
            <div class="address-text">{{ recipientAddress }}</div>
          </template>
          <template v-else>
            <el-empty description="暂无收件地址" :image-size="60" />
          </template>
        </div>
      </div>
    </div>

    <template #footer>
      <el-button @click="dialogVisible = false">取消</el-button>
      <el-button type="primary" @click="handleSubmit" :loading="submitting">
        确认分发
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
  card: {
    type: Object,
    default: null
  }
})

const emit = defineEmits(['update:visible', 'confirm'])

// 表单引用
const formRef = ref(null)

// 表单数据
const form = ref({
  method: '快递',
  remarks: ''
})

// 提交状态
const submitting = ref(false)

// 收件地址（只读，后续从卡片关联数据获取）
// TODO: 后续实现从呼号关联的联系人信息中获取收件地址
const recipientAddress = computed(() => {
  // 暂时返回空，后续从 props.card 关联的数据中获取
  return ''
})

// 表单验证规则
const rules = {
  method: [
    {required: true, message: '请选择处理方式', trigger: 'change'}
  ]
}

// 获取状态标签类型
const getStatusType = (status) => {
  const types = {
    pending: 'info',
    distributed: 'success',
    returned: 'warning'
  }
  return types[status] || 'info'
}

// 获取状态标签文本
const getStatusLabel = (status) => {
  const labels = {
    pending: '待分发',
    distributed: '已分发',
    returned: '已退回'
  }
  return labels[status] || status
}

// 格式化时间
const formatDateTime = (datetime) => {
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
      method: '快递',
      remarks: ''
    }

    // 清除验证状态
    nextTick(() => {
      formRef.value?.clearValidate()
    })
  }
})

// 关闭弹窗
const handleClose = () => {
  submitting.value = false
}

// 复制备注内容到剪贴板
const handleCopy = async () => {
  try {
    if (form.value.remarks) {
      await navigator.clipboard.writeText(form.value.remarks)
    }
  } catch (error) {
    console.error('复制到剪贴板失败:', error)
  }
}

// 粘贴剪贴板内容
const handlePaste = async () => {
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
const handleSubmit = async () => {
  if (submitting.value) return

  try {
    await formRef.value.validate()
    submitting.value = true

    emit('confirm', {
      id: props.card.id,
      method: form.value.method,
      address: recipientAddress.value || null,
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
.distribute-content {
  display: flex;
  gap: 24px;
}

.left-panel {
  flex: 1;
  min-width: 0;
}

.right-panel {
  width: 240px;
  flex-shrink: 0;
  border-left: 1px solid #ebeef5;
  padding-left: 24px;
}

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

.address-content {
  min-height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.address-text {
  padding: 12px;
  background-color: #f5f7fa;
  border-radius: 4px;
  line-height: 1.6;
  width: 100%;
}

.radio-group-vertical {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px 16px;
}

.radio-group-vertical .el-radio {
  margin-right: 0;
}
</style>
