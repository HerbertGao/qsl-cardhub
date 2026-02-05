<template>
  <div class="page-content">
    <div class="page-header">
      <h1>顺丰订单列表</h1>
      <el-button
        type="primary"
        @click="handleRefresh"
      >
        <el-icon><Refresh /></el-icon>
        刷新
      </el-button>
    </div>

    <!-- 筛选条件 -->
    <el-card
      shadow="hover"
      style="margin-bottom: 20px"
    >
      <el-row :gutter="16">
        <el-col :span="6">
          <el-select
            v-model="filter.status"
            placeholder="订单状态"
            clearable
            style="width: 100%"
            @change="handleFilter"
          >
            <el-option
              label="待确认"
              value="pending"
            />
            <el-option
              label="已确认"
              value="confirmed"
            />
            <el-option
              label="已取消"
              value="cancelled"
            />
            <el-option
              label="已打印"
              value="printed"
            />
          </el-select>
        </el-col>
        <el-col :span="6">
          <el-input
            v-model="filter.orderId"
            placeholder="搜索订单号"
            clearable
            @keyup.enter="handleFilter"
          />
        </el-col>
        <el-col :span="6">
          <el-input
            v-model="filter.waybillNo"
            placeholder="搜索运单号"
            clearable
            @keyup.enter="handleFilter"
          />
        </el-col>
        <el-col :span="6">
          <el-button
            type="primary"
            @click="handleFilter"
          >
            <el-icon><Search /></el-icon>
            查询
          </el-button>
          <el-button @click="handleResetFilter">
            重置
          </el-button>
        </el-col>
      </el-row>
    </el-card>

    <!-- 订单列表 -->
    <el-card shadow="hover">
      <el-table
        v-loading="loading"
        :data="orders"
        style="width: 100%"
        empty-text="暂无订单"
      >
        <el-table-column
          prop="order_id"
          label="订单号"
          width="200"
        />
        <el-table-column
          prop="waybill_no"
          label="运单号"
          width="150"
        >
          <template #default="{ row }">
            <el-button
              v-if="row.waybill_no"
              type="primary"
              link
              @click="handleViewDetail(row)"
            >
              {{ row.waybill_no }}
            </el-button>
            <el-button
              v-else
              type="info"
              link
              @click="handleViewDetail(row)"
            >
              查看详情
            </el-button>
          </template>
        </el-table-column>
        <el-table-column
          label="状态"
          width="100"
        >
          <template #default="{ row }">
            <el-tag :type="getStatusType(row.status)">
              {{ getStatusLabel(row.status) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column
          label="呼号"
          width="120"
        >
          <template #default="{ row }">
            <span v-if="row.callsign">{{ row.callsign }}</span>
            <span
              v-else
              style="color: #909399"
            >-</span>
          </template>
        </el-table-column>
        <el-table-column
          label="收件人"
          min-width="150"
        >
          <template #default="{ row }">
            <div>{{ row.recipient_info.name }}</div>
            <div style="font-size: 12px; color: #909399">
              {{ row.recipient_info.phone }}
            </div>
          </template>
        </el-table-column>
        <el-table-column
          label="收件地址"
          min-width="200"
          show-overflow-tooltip
        >
          <template #default="{ row }">
            {{ row.recipient_info.province }}{{ row.recipient_info.city }}{{ row.recipient_info.district }}{{ row.recipient_info.address }}
          </template>
        </el-table-column>
        <el-table-column
          label="创建时间"
          width="160"
        >
          <template #default="{ row }">
            {{ formatDateTime(row.created_at) }}
          </template>
        </el-table-column>
        <el-table-column
          label="操作"
          width="120"
          fixed="right"
          align="center"
        >
          <template #default="{ row }">
            <div class="action-buttons">
              <el-button
                type="primary"
                link
                size="small"
                :disabled="!row.waybill_no"
                @click="handlePrintWaybill(row)"
              >
                打印面单
              </el-button>
              <el-dropdown
                trigger="click"
                @command="(cmd: string) => handleRowCommand(cmd, row)"
              >
                <el-button
                  type="info"
                  link
                  size="small"
                >
                  更多
                  <el-icon class="el-icon--right">
                    <ArrowDown />
                  </el-icon>
                </el-button>
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item
                      v-if="row.status === 'pending'"
                      command="confirm"
                    >
                      <el-icon><Select /></el-icon>
                      确认订单
                    </el-dropdown-item>
                    <el-dropdown-item
                      v-if="row.status === 'pending' || row.status === 'confirmed'"
                      command="cancel"
                    >
                      <el-icon><CloseBold /></el-icon>
                      取消订单
                    </el-dropdown-item>
                    <el-dropdown-item
                      command="delete"
                      divided
                    >
                      <el-icon><Delete /></el-icon>
                      删除
                    </el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
            </div>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div
        v-if="total > 0"
        style="margin-top: 20px; display: flex; justify-content: flex-end"
      >
        <el-pagination
          v-model:current-page="pagination.page"
          v-model:page-size="pagination.pageSize"
          :total="total"
          :page-sizes="[10, 20, 50, 100]"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="handleSizeChange"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>

    <!-- 订单详情对话框 -->
    <el-dialog
      v-model="detailDialogVisible"
      title="订单详情"
      width="750px"
    >
      <template v-if="selectedOrder">
        <el-descriptions
          :column="2"
          border
        >
          <el-descriptions-item label="订单号">
            {{ selectedOrder.order_id }}
          </el-descriptions-item>
          <el-descriptions-item label="运单号">
            {{ selectedOrder.waybill_no || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag :type="getStatusType(selectedOrder.status)">
              {{ getStatusLabel(selectedOrder.status) }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="付款方式">
            {{ getPayMethodLabel(selectedOrder.pay_method) }}
          </el-descriptions-item>
          <el-descriptions-item label="托寄物">
            {{ selectedOrder.cargo_name || 'QSL卡片' }}
          </el-descriptions-item>
        </el-descriptions>

        <!-- 顺丰查询结果 -->
        <el-divider>顺丰查询结果</el-divider>
        <el-descriptions
          v-if="queryResult"
          :column="3"
          border
        >
          <el-descriptions-item label="运单号">
            {{ queryResult.waybill_no_list?.join(', ') || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="筛单结果">
            {{ getFilterResultLabel(queryResult.filter_result) }}
          </el-descriptions-item>
          <el-descriptions-item label="查询状态">
            <el-tag
              type="success"
              size="small"
            >
              已查询
            </el-tag>
          </el-descriptions-item>
        </el-descriptions>
        <div
          v-else-if="queryLoading"
          style="text-align: center; padding: 10px; color: #909399"
        >
          <el-icon class="is-loading">
            <Loading />
          </el-icon>
          正在查询...
        </div>
        <div
          v-else-if="queryError"
          style="text-align: center; padding: 10px; color: #f56c6c"
        >
          查询失败: {{ queryError }}
        </div>
        <div
          v-else
          style="text-align: center; padding: 10px; color: #909399"
        >
          -
        </div>

        <!-- 关联卡片信息 -->
        <template v-if="selectedOrder.card_id">
          <el-divider>关联卡片</el-divider>
          <el-descriptions
            :column="3"
            border
          >
            <el-descriptions-item label="转卡项目">
              {{ selectedOrder.project_name || '-' }}
            </el-descriptions-item>
            <el-descriptions-item label="呼号">
              {{ selectedOrder.callsign || '-' }}
            </el-descriptions-item>
            <el-descriptions-item label="数量">
              {{ formatQty(selectedOrder.qty) }}
            </el-descriptions-item>
          </el-descriptions>
        </template>

        <el-divider>寄件人信息</el-divider>
        <el-descriptions
          :column="2"
          border
        >
          <el-descriptions-item label="姓名">
            {{ selectedOrder.sender_info.name }}
          </el-descriptions-item>
          <el-descriptions-item label="电话">
            {{ selectedOrder.sender_info.phone }}
          </el-descriptions-item>
          <el-descriptions-item
            label="地址"
            :span="2"
          >
            {{ selectedOrder.sender_info.province }}{{ selectedOrder.sender_info.city }}{{ selectedOrder.sender_info.district }}{{ selectedOrder.sender_info.address }}
          </el-descriptions-item>
        </el-descriptions>

        <el-divider>收件人信息</el-divider>
        <el-descriptions
          :column="2"
          border
        >
          <el-descriptions-item label="姓名">
            {{ selectedOrder.recipient_info.name }}
          </el-descriptions-item>
          <el-descriptions-item label="电话">
            {{ selectedOrder.recipient_info.phone }}
          </el-descriptions-item>
          <el-descriptions-item
            label="地址"
            :span="2"
          >
            {{ selectedOrder.recipient_info.province }}{{ selectedOrder.recipient_info.city }}{{ selectedOrder.recipient_info.district }}{{ selectedOrder.recipient_info.address }}
          </el-descriptions-item>
        </el-descriptions>

        <el-divider>时间信息</el-divider>
        <el-descriptions
          :column="2"
          border
        >
          <el-descriptions-item label="创建时间">
            {{ formatDateTime(selectedOrder.created_at) }}
          </el-descriptions-item>
          <el-descriptions-item label="更新时间">
            {{ formatDateTime(selectedOrder.updated_at) }}
          </el-descriptions-item>
        </el-descriptions>
      </template>

      <template #footer>
        <div style="display: flex; justify-content: space-between">
          <div>
            <el-button
              v-if="selectedOrder?.status === 'pending' || selectedOrder?.status === 'confirmed'"
              type="danger"
              @click="handleCancelOrderFromDetail"
            >
              取消订单
            </el-button>
          </div>
          <div>
            <el-button @click="detailDialogVisible = false">
              关闭
            </el-button>
            <el-button
              v-if="selectedOrder?.status === 'pending'"
              type="primary"
              @click="handleConfirmOrderFromDetail"
            >
              确认订单
            </el-button>
            <el-button
              v-if="selectedOrder?.waybill_no"
              type="success"
              @click="handlePrintWaybillFromDetail"
            >
              打印面单
            </el-button>
          </div>
        </div>
      </template>
    </el-dialog>

    <!-- 运单打印对话框 -->
    <WaybillPrintDialog
      v-model:visible="waybillPrintDialogVisible"
      :default-waybill-no="selectedWaybillNo"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { SFOrderWithCard, SFOrderStatus, ListOrdersResponse } from '@/types/models'
import WaybillPrintDialog from '@/components/cards/WaybillPrintDialog.vue'
import { useLoading } from '@/composables/useLoading'
import { useQtyDisplayMode } from '@/composables/useQtyDisplayMode'

const { withLoading } = useLoading()
const { formatQty } = useQtyDisplayMode()

// 状态
const loading = ref(false)
const orders = ref<SFOrderWithCard[]>([])
const total = ref(0)

// 筛选条件
const filter = reactive({
  status: '' as SFOrderStatus | '',
  orderId: '',
  waybillNo: ''
})

// 分页
const pagination = reactive({
  page: 1,
  pageSize: 10
})

// 详情对话框
const detailDialogVisible = ref(false)
const selectedOrder = ref<SFOrderWithCard | null>(null)

// 查询结果（集成到详情对话框中）
const queryResult = ref<{ order_id: string; waybill_no_list: string[]; filter_result?: string | null } | null>(null)
const queryLoading = ref(false)
const queryError = ref<string | null>(null)

// 打印对话框
const waybillPrintDialogVisible = ref(false)
const selectedWaybillNo = ref('')

// 获取状态标签类型
function getStatusType(status: string): 'info' | 'warning' | 'success' | 'danger' {
  const types: Record<SFOrderStatus, 'info' | 'warning' | 'success' | 'danger'> = {
    pending: 'warning',
    confirmed: 'success',
    cancelled: 'info',
    printed: 'success'
  }
  return types[status as SFOrderStatus] || 'info'
}

// 获取状态标签文本
function getStatusLabel(status: string): string {
  const labels: Record<SFOrderStatus, string> = {
    pending: '待确认',
    confirmed: '已确认',
    cancelled: '已取消',
    printed: '已打印'
  }
  return labels[status as SFOrderStatus] || status
}

// 获取筛单结果标签
function getFilterResultLabel(filterResult?: string | null): string {
  if (!filterResult) return '-'
  const labels: Record<string, string> = {
    '1': '人工确认',
    '2': '可收派',
    '3': '不可收派'
  }
  return labels[filterResult] || filterResult
}

// 获取付款方式标签
function getPayMethodLabel(payMethod?: number | null): string {
  if (!payMethod) return '寄方付'
  const labels: Record<number, string> = {
    1: '寄方付',
    2: '收方付',
    3: '第三方付'
  }
  return labels[payMethod] || `未知(${payMethod})`
}

// 格式化时间
function formatDateTime(datetime: string): string {
  if (!datetime) return '-'
  const date = new Date(datetime)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

// 解析订单中的 JSON 字符串字段
function parseOrderInfo(order: SFOrderWithCard): SFOrderWithCard {
  return {
    ...order,
    sender_info: typeof order.sender_info === 'string'
      ? JSON.parse(order.sender_info)
      : order.sender_info,
    recipient_info: typeof order.recipient_info === 'string'
      ? JSON.parse(order.recipient_info)
      : order.recipient_info
  } as SFOrderWithCard
}

// 加载订单列表
async function loadOrders(): Promise<void> {
  loading.value = true
  try {
    const result = await invoke<ListOrdersResponse>('sf_list_orders', {
      params: {
        status: filter.status || null,
        page: pagination.page,
        page_size: pagination.pageSize
      }
    })
    // 解析 JSON 字符串字段
    orders.value = result.items.map(parseOrderInfo)
    total.value = result.total
  } catch (error) {
    console.error('加载订单列表失败:', error)
    ElMessage.error(`加载订单列表失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 刷新
function handleRefresh(): void {
  loadOrders()
}

// 筛选
function handleFilter(): void {
  pagination.page = 1
  loadOrders()
}

// 重置筛选
function handleResetFilter(): void {
  filter.status = ''
  filter.orderId = ''
  filter.waybillNo = ''
  pagination.page = 1
  loadOrders()
}

// 分页变化
function handlePageChange(): void {
  loadOrders()
}

function handleSizeChange(): void {
  pagination.page = 1
  loadOrders()
}

// 确认订单
async function handleConfirmOrder(order: SFOrderWithCard): Promise<void> {
  try {
    await ElMessageBox.confirm(
      `确定要确认订单「${order.order_id}」吗？`,
      '确认订单',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'info'
      }
    )

    const result = await withLoading(async () => await invoke<{ order: SFOrderWithCard; api_response: { success: boolean; waybill_no?: string; error_msg?: string } }>('sf_confirm_order', {
        orderId: order.order_id
      }), '正在确认订单...')

    if (result.api_response.success) {
      ElMessage.success(`订单确认成功，运单号: ${result.api_response.waybill_no}`)
      loadOrders()
    } else {
      ElMessage.error(`确认失败: ${result.api_response.error_msg}`)
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(`确认订单失败: ${error}`)
    }
  }
}

// 取消订单
async function handleCancelOrder(order: SFOrderWithCard): Promise<void> {
  try {
    await ElMessageBox.confirm(
      `确定要取消订单「${order.order_id}」吗？`,
      '取消订单',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    await withLoading(async () => {
      await invoke('sf_cancel_order', { orderId: order.order_id })
    }, '正在取消订单...')
    ElMessage.success('订单已取消')
    loadOrders()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(`取消订单失败: ${error}`)
    }
  }
}

// 从详情对话框确认订单
async function handleConfirmOrderFromDetail(): Promise<void> {
  if (!selectedOrder.value) return
  await handleConfirmOrder(selectedOrder.value)
  detailDialogVisible.value = false
}

// 从详情对话框取消订单
async function handleCancelOrderFromDetail(): Promise<void> {
  if (!selectedOrder.value) return
  await handleCancelOrder(selectedOrder.value)
  detailDialogVisible.value = false
}

// 从详情对话框打印面单
function handlePrintWaybillFromDetail(): void {
  if (!selectedOrder.value) return
  handlePrintWaybill(selectedOrder.value)
}

// 查看详情
async function handleViewDetail(order: SFOrderWithCard): Promise<void> {
  selectedOrder.value = order
  queryResult.value = null
  queryError.value = null
  detailDialogVisible.value = true

  // 查询顺丰订单状态
  queryLoading.value = true
  try {
    queryResult.value = await invoke<{ order_id: string; waybill_no_list: string[]; filter_result?: string | null }>('sf_search_order', {
      orderId: order.order_id,
      waybillNo: order.waybill_no || null
    })
  } catch (error) {
    queryError.value = String(error)
  } finally {
    queryLoading.value = false
  }
}

// 删除订单
async function handleDeleteOrder(order: SFOrderWithCard): Promise<void> {
  try {
    await ElMessageBox.confirm(
      `确定要删除订单「${order.order_id}」吗？此操作不可恢复。`,
      '删除订单',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    await invoke('sf_delete_order', { id: order.id })
    ElMessage.success('订单已删除')
    loadOrders()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(`删除订单失败: ${error}`)
    }
  }
}

// 打印面单
function handlePrintWaybill(order: SFOrderWithCard): void {
  if (!order.waybill_no) {
    ElMessage.warning('该订单暂无运单号')
    return
  }
  selectedWaybillNo.value = order.waybill_no
  waybillPrintDialogVisible.value = true
}

// 行操作命令处理
function handleRowCommand(command: string, row: SFOrderWithCard): void {
  switch (command) {
    case 'confirm':
      handleConfirmOrder(row)
      break
    case 'cancel':
      handleCancelOrder(row)
      break
    case 'print':
      handlePrintWaybill(row)
      break
    case 'delete':
      handleDeleteOrder(row)
      break
  }
}

onMounted(() => {
  loadOrders()
})
</script>

<style scoped>
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.page-header h1 {
  margin: 0;
}

.action-buttons {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
}
</style>
