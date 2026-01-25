<template>
  <el-dialog
    v-model="dialogVisible"
    title="确认订单"
    width="600px"
    :close-on-click-modal="false"
  >
    <div
      v-if="orderData"
      class="confirm-content"
    >
      <!-- 订单基本信息 -->
      <el-descriptions
        title="订单信息"
        :column="2"
        border
        size="small"
      >
        <el-descriptions-item label="客户订单号">
          {{ orderData.order_id }}
        </el-descriptions-item>
        <el-descriptions-item label="运单号">
          <el-tag
            v-if="orderData.waybill_no_list[0]"
            type="success"
            size="small"
          >
            {{ orderData.waybill_no_list[0] }}
          </el-tag>
          <span
            v-else
            class="text-muted"
          >待确认后生成</span>
        </el-descriptions-item>
        <el-descriptions-item label="产品类型">
          {{ expressTypeName }}
        </el-descriptions-item>
        <el-descriptions-item label="付款方式">
          {{ payMethodName }}
        </el-descriptions-item>
      </el-descriptions>

      <!-- 寄件人信息 -->
      <el-descriptions
        title="寄件人"
        :column="1"
        border
        size="small"
        class="section-descriptions"
      >
        <el-descriptions-item label="姓名/电话">
          <strong>{{ orderData.sender_info.name }}</strong>
          <span class="contact-phone">{{ orderData.sender_info.phone }}</span>
        </el-descriptions-item>
        <el-descriptions-item label="地址">
          {{ orderData.sender_info.full_address }}
        </el-descriptions-item>
      </el-descriptions>

      <!-- 收件人信息 -->
      <el-descriptions
        title="收件人"
        :column="1"
        border
        size="small"
        class="section-descriptions"
      >
        <el-descriptions-item label="姓名/电话">
          <strong>{{ orderData.recipient_info.name }}</strong>
          <span class="contact-phone">{{ orderData.recipient_info.phone }}</span>
        </el-descriptions-item>
        <el-descriptions-item label="地址">
          {{ orderData.recipient_info.full_address }}
        </el-descriptions-item>
      </el-descriptions>

      <!-- 托寄物信息 -->
      <el-descriptions
        title="托寄物"
        :column="1"
        border
        size="small"
        class="section-descriptions"
      >
        <el-descriptions-item label="物品名称">
          {{ orderData.cargo_name }}
        </el-descriptions-item>
      </el-descriptions>

      <!-- 顺丰返回信息 -->
      <el-descriptions
        title="顺丰返回"
        :column="2"
        border
        size="small"
        class="section-descriptions"
      >
        <el-descriptions-item label="原寄地代码">
          {{ orderData.origin_code || '-' }}
        </el-descriptions-item>
        <el-descriptions-item label="目的地代码">
          {{ orderData.dest_code || '-' }}
        </el-descriptions-item>
        <el-descriptions-item
          label="筛单结果"
          :span="2"
        >
          <el-tag
            :type="filterResultType"
            size="small"
          >
            {{ filterResultText }}
          </el-tag>
        </el-descriptions-item>
      </el-descriptions>
    </div>

    <template #footer>
      <el-button @click="handleCancel">
        稍后确认
      </el-button>
      <el-button
        type="primary"
        :loading="confirming"
        @click="handleConfirm"
      >
        立即确认订单
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import type { CreateOrderResponse, ConfirmOrderResponse, SFOrder } from '@/types/models'
import { useLoading } from '@/composables/useLoading'

const { withLoading } = useLoading()

interface Props {
  visible: boolean
  orderData: CreateOrderResponse | null
}

interface Emits {
  (e: 'update:visible', value: boolean): void
  (e: 'success', order: SFOrder): void
  (e: 'cancel'): void
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  orderData: null
})

const emit = defineEmits<Emits>()

const confirming = ref(false)

// 双向绑定
const dialogVisible = computed({
  get: () => props.visible,
  set: (val) => emit('update:visible', val)
})

// 产品类型名称
const expressTypeName = computed(() => {
  if (!props.orderData) return '-'
  const types: Record<number, string> = {
    1: '顺丰次晨',
    2: '顺丰标快',
    5: '顺丰特惠',
    6: '顺丰即日'
  }
  return types[props.orderData.express_type_id] || `类型 ${props.orderData.express_type_id}`
})

// 付款方式名称
const payMethodName = computed(() => {
  if (!props.orderData) return '-'
  const methods: Record<number, string> = {
    1: '寄方付',
    2: '收方付',
    3: '第三方付'
  }
  return methods[props.orderData.pay_method] || `方式 ${props.orderData.pay_method}`
})

// 筛单结果文本
const filterResultText = computed(() => {
  if (!props.orderData?.filter_result) return '未知'
  const results: Record<number, string> = {
    1: '人工确认',
    2: '可收派',
    3: '不可收派'
  }
  return results[props.orderData.filter_result] || `结果 ${props.orderData.filter_result}`
})

// 筛单结果标签类型
const filterResultType = computed<'success' | 'warning' | 'danger' | 'info'>(() => {
  if (!props.orderData?.filter_result) return 'info'
  const types: Record<number, 'success' | 'warning' | 'danger'> = {
    1: 'warning',
    2: 'success',
    3: 'danger'
  }
  return types[props.orderData.filter_result] || 'info'
})

// 取消/稍后确认
function handleCancel(): void {
  emit('cancel')
  dialogVisible.value = false
}

// 确认订单
async function handleConfirm(): Promise<void> {
  if (!props.orderData) return

  confirming.value = true

  try {
    const result = await withLoading(async () => await invoke<ConfirmOrderResponse>('sf_confirm_order', {
        orderId: props.orderData!.order_id
      }), '正在确认订单...')

    const waybillNo = result.waybill_no_list[0]
    ElMessage.success(`订单确认成功，运单号: ${waybillNo}`)
    emit('success', result.local_order)
    dialogVisible.value = false
  } catch (error) {
    ElMessage.error(`确认订单失败: ${error}`)
  } finally {
    confirming.value = false
  }
}
</script>

<style scoped>
.confirm-content {
  max-height: 60vh;
  overflow-y: auto;
}

.section-descriptions {
  margin-top: 16px;
}

.contact-phone {
  margin-left: 12px;
  color: #606266;
}

.text-muted {
  color: #909399;
}

:deep(.el-descriptions__title) {
  font-size: 14px;
  font-weight: 600;
}
</style>
