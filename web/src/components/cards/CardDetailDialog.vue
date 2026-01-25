<template>
  <el-drawer
    v-model="dialogVisible"
    title="卡片详情"
    direction="rtl"
    size="400px"
  >
    <div
      v-if="card"
      class="card-detail"
    >
      <!-- 基本信息 -->
      <div class="detail-section">
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

      <!-- 分发记录 -->
      <div
        v-if="card.metadata?.distribution"
        class="detail-section"
      >
        <div class="section-title">
          分发记录
        </div>
        <el-descriptions
          :column="1"
          size="small"
          border
        >
          <el-descriptions-item label="处理方式">
            {{ card.metadata.distribution.method }}
          </el-descriptions-item>
          <el-descriptions-item
            v-if="card.metadata.distribution.method === '代领' && card.metadata.distribution.proxy_callsign"
            label="代领人"
          >
            {{ card.metadata.distribution.proxy_callsign }}
          </el-descriptions-item>
          <el-descriptions-item label="分发时间">
            {{ formatDateTime(card.metadata.distribution.distributed_at) }}
          </el-descriptions-item>
          <el-descriptions-item
            v-if="card.metadata.distribution.address"
            label="分发地址"
          >
            {{ card.metadata.distribution.address }}
          </el-descriptions-item>
          <el-descriptions-item
            v-if="card.metadata.distribution.remarks"
            label="备注"
          >
            {{ card.metadata.distribution.remarks }}
          </el-descriptions-item>
        </el-descriptions>
      </div>

      <!-- 退回记录 -->
      <div
        v-if="card.metadata?.return"
        class="detail-section"
      >
        <div class="section-title">
          退回记录
        </div>
        <el-descriptions
          :column="1"
          size="small"
          border
        >
          <el-descriptions-item label="处理方式">
            {{ card.metadata.return.method }}
          </el-descriptions-item>
          <el-descriptions-item label="退回时间">
            {{ formatDateTime(card.metadata.return.returned_at) }}
          </el-descriptions-item>
          <el-descriptions-item
            v-if="card.metadata.return.remarks"
            label="备注"
          >
            {{ card.metadata.return.remarks }}
          </el-descriptions-item>
        </el-descriptions>
      </div>
    </div>

    <template #footer>
      <div class="drawer-footer">
        <el-button
          type="success"
          @click="handleDistribute"
        >
          <el-icon><Promotion /></el-icon>
          分发
        </el-button>
        <el-button
          type="warning"
          @click="handleReturn"
        >
          <el-icon><RefreshLeft /></el-icon>
          退回
        </el-button>
        <el-button @click="dialogVisible = false">
          关闭
        </el-button>
      </div>
    </template>
  </el-drawer>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { CardWithProject, CardStatus } from '@/types/models'
import { formatSerial } from '@/utils/format'

interface Props {
  visible: boolean
  card: CardWithProject | null
}

interface Emits {
  (e: 'update:visible', value: boolean): void
  (e: 'distribute', card: CardWithProject): void
  (e: 'return', card: CardWithProject): void
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  card: null
})

const emit = defineEmits<Emits>()

// 双向绑定 visible
const dialogVisible = computed<boolean>({
  get: (): boolean => props.visible,
  set: (val: boolean): void => emit('update:visible', val)
})

// 分发按钮点击
const handleDistribute = (): void => {
  if (props.card) {
    emit('distribute', props.card)
    dialogVisible.value = false
  }
}

// 退回按钮点击
const handleReturn = (): void => {
  if (props.card) {
    emit('return', props.card)
    dialogVisible.value = false
  }
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
</script>

<style scoped>
.card-detail {
  padding: 0 4px;
}

.detail-section {
  margin-bottom: 24px;
}

.detail-section:last-child {
  margin-bottom: 0;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 12px;
  padding-left: 8px;
  border-left: 3px solid #409eff;
}

.drawer-footer {
  display: flex;
  gap: 8px;
}
</style>
