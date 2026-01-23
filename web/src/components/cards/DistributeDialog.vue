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
            分发信息
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
                  value="快递"
                  border
                >
                  快递
                </el-radio>
                <el-radio
                  value="挂号信"
                  border
                >
                  挂号信
                </el-radio>
                <el-radio
                  value="平邮"
                  border
                >
                  平邮
                </el-radio>
                <el-radio
                  value="自取"
                  border
                >
                  自取
                </el-radio>
                <el-radio
                  value="代领"
                  border
                >
                  代领
                </el-radio>
                <el-radio
                  value="其它"
                  border
                >
                  其它
                </el-radio>
              </el-radio-group>
            </el-form-item>

            <!-- 顺丰下单按钮（快递方式时显示） -->
            <el-form-item v-if="form.method === '快递'">
              <el-button
                color="#141222"
                @click="handleCreateSFOrder"
              >
                <el-icon><IconSfExpress /></el-icon>
                顺丰速运下单
              </el-button>
              <div style="font-size: 12px; color: #909399; margin-top: 4px">
                点击创建顺丰订单，获取运单号后可打印面单
              </div>
            </el-form-item>

            <el-form-item
              label="备注"
              prop="remarks"
            >
              <el-input
                v-model="form.remarks"
                type="textarea"
                :rows="3"
                placeholder="可选，填写备注信息（如运单号）"
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
        <div class="section-title">
          收件地址
          <el-button
            type="primary"
            size="small"
            link
            :loading="querying"
            style="margin-left: 10px"
            @click="handleQueryAddress"
          >
            <el-icon v-if="!querying">
              <Search />
            </el-icon>
            查询地址
          </el-button>
        </div>

        <!-- 地址缓存列表 -->
        <div
          v-if="addressCache.length > 0"
          class="address-content"
        >
          <div class="address-list">
            <div
              v-for="(addr, index) in addressCache"
              :key="index"
              class="address-item"
            >
              <div class="address-header">
                <span class="address-callsign">{{ card?.callsign }}</span>
                <el-tag
                  size="small"
                  type="info"
                >
                  {{ addr.source }}
                </el-tag>
              </div>
              <!-- QRZ.cn 地址：显示中英文地址 -->
              <template v-if="addr.source === 'qrz.cn'">
                <div
                  v-if="addr.chinese_address"
                  class="address-text"
                >
                  {{ addr.chinese_address }}
                </div>
                <div
                  v-if="addr.english_address"
                  class="address-text-en"
                >
                  {{ addr.english_address }}
                </div>
              </template>
              <!-- QRZ.com 地址：只显示英文地址 -->
              <template v-else-if="addr.source === 'qrz.com'">
                <div
                  v-if="addr.english_address"
                  class="address-text"
                >
                  {{ addr.english_address }}
                </div>
              </template>
              <!-- QRZ卡片查询 地址：显示姓名、地址和邮寄方式 -->
              <template v-else-if="addr.source === 'QRZ卡片查询'">
                <div
                  v-if="addr.name"
                  class="address-text"
                  style="font-weight: 600; margin-bottom: 4px"
                >
                  姓名: {{ addr.name }}
                </div>
                <div
                  v-if="addr.english_address"
                  class="address-text"
                  style="margin-bottom: 4px"
                >
                  {{ addr.english_address }}
                </div>
                <div
                  v-if="addr.mail_method"
                  class="address-text"
                  style="color: #409eff"
                >
                  邮寄方式: {{ addr.mail_method }}
                </div>
              </template>
              <div class="address-meta">
                <span>更新: {{ formatDate(addr.updated_at) }}</span>
                <span>缓存: {{ formatDateTime(addr.cached_at) }}</span>
              </div>
              <div class="address-actions">
                <!-- QRZ.cn 复制按钮：显示中英文按钮 -->
                <template v-if="addr.source === 'qrz.cn'">
                  <el-button
                    v-if="addr.chinese_address"
                    type="primary"
                    size="small"
                    link
                    @click="handleCopyAddress(addr.chinese_address)"
                  >
                    <el-icon><CopyDocument /></el-icon>
                    复制中文地址
                  </el-button>
                  <el-button
                    v-if="addr.english_address"
                    type="primary"
                    size="small"
                    link
                    @click="handleCopyAddress(addr.english_address)"
                  >
                    <el-icon><CopyDocument /></el-icon>
                    复制英文地址
                  </el-button>
                </template>
                <!-- QRZ.com 复制按钮：只显示单个复制按钮 -->
                <template v-else-if="addr.source === 'qrz.com'">
                  <el-button
                    v-if="addr.english_address"
                    type="primary"
                    size="small"
                    link
                    @click="handleCopyAddress(addr.english_address)"
                  >
                    <el-icon><CopyDocument /></el-icon>
                    复制地址
                  </el-button>
                </template>
                <!-- QRZ卡片查询 复制按钮：复制完整地址信息 -->
                <template v-else-if="addr.source === 'QRZ卡片查询'">
                  <el-button
                    v-if="addr.english_address"
                    type="primary"
                    size="small"
                    link
                    @click="handleCopyAddress([addr.name, addr.english_address, addr.mail_method].filter(Boolean).join('\n'))"
                  >
                    <el-icon><CopyDocument /></el-icon>
                    复制地址
                  </el-button>
                </template>
              </div>
            </div>
          </div>
        </div>

        <!-- 空状态 -->
        <div
          v-else
          class="address-content"
        >
          <el-empty
            description="暂无地址缓存"
            :image-size="60"
          >
            <el-button
              type="primary"
              size="small"
              :loading="querying"
              @click="handleQueryAddress"
            >
              立即查询
            </el-button>
          </el-empty>
        </div>
      </div>
    </div>

    <template #footer>
      <el-button @click="dialogVisible = false">
        取消
      </el-button>
      <el-button
        type="success"
        :disabled="!form.remarks"
        @click="handlePrintWaybill"
      >
        <el-icon><Printer /></el-icon>
        打印面单
      </el-button>
      <el-button
        type="primary"
        :loading="submitting"
        @click="handleSubmit"
      >
        确认分发
      </el-button>
    </template>

    <!-- 运单打印弹窗 -->
    <WaybillPrintDialog
      v-model:visible="waybillPrintDialogVisible"
      :default-waybill-no="form.remarks"
    />

    <!-- 顺丰下单弹窗 -->
    <CreateOrderDialog
      v-model:visible="sfOrderDialogVisible"
      :card-id="card?.id"
      :default-recipient="defaultRecipient"
      @success="handleSFOrderSuccess"
    />
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import type { CardWithProject, CardStatus, AddressEntry, SFOrder, RecipientInfo } from '@/types/models'
import WaybillPrintDialog from '@/components/cards/WaybillPrintDialog.vue'
import CreateOrderDialog from '@/components/sf-express/CreateOrderDialog.vue'
import IconSfExpress from '~icons/custom/sf-express'
import { useLoading } from '@/composables/useLoading'

const { withLoading } = useLoading()

interface Props {
  visible: boolean
  card: CardWithProject | null
}

interface DistributeFormData {
  method: string
  remarks: string
}

interface ConfirmData {
  id: string
  method: string
  address: string | null
  remarks: string | null
}

interface Emits {
  (e: 'update:visible', value: boolean): void
  (e: 'confirm', data: ConfirmData): void
  (e: 'refresh'): void
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  card: null
})

const emit = defineEmits<Emits>()

// 表单引用
const formRef = ref<FormInstance | null>(null)

// 表单数据
const form = ref<DistributeFormData>({
  method: '快递',
  remarks: ''
})

// 提交状态
const submitting = ref<boolean>(false)

// 运单打印弹窗
const waybillPrintDialogVisible = ref<boolean>(false)

// 顺丰下单弹窗
const sfOrderDialogVisible = ref<boolean>(false)
const defaultRecipient = ref<Partial<RecipientInfo> | null>(null)

// 地址查询状态
const querying = ref<boolean>(false)

// 地址缓存列表
const addressCache = ref<AddressEntry[]>([])

// 收件地址（用于提交）- 使用最新的地址
const recipientAddress = computed<string>(() => {
  return addressCache.value.length > 0 ? (addressCache.value[0].chinese_address || '') : ''
})

// 表单验证规则
const rules: FormRules<DistributeFormData> = {
  method: [
    { required: true, message: '请选择处理方式', trigger: 'change' }
  ]
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

// 格式化日期（仅日期）
const formatDate = (datetime: string | null | undefined): string => {
  if (!datetime) return '-'
  const date = new Date(datetime)
  return date.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit'
  })
}

// 加载地址缓存
const loadAddressCache = (): void => {
  addressCache.value = []

  if (props.card && props.card.metadata && props.card.metadata.address_cache) {
    addressCache.value = props.card.metadata.address_cache
  }
}

// 查询地址
const handleQueryAddress = async (isAutoQuery: boolean = false): Promise<void> => {
  if (!props.card || !props.card.callsign) {
    if (!isAutoQuery) {
      ElMessage.warning('无效的卡片信息')
    }
    return
  }

  querying.value = true

  // 保存卡片引用，避免闭包中的 null 检查问题
  const card = props.card

  // 手动查询时显示全局 loading
  const executeQuery = async () => {
    try {
    // 并行检查两个数据源的登录状态
    const [cnLoggedIn, comLoggedIn] = await Promise.all([
      invoke<boolean>('qrz_check_login_status').catch(() => false),
      invoke<boolean>('qrz_com_check_login_status').catch(() => false)
    ])

    // 给出信息提示（仅手动查询时显示）
    // 注意：QRZ.herbertgao.me 无需登录，始终可用
    if (!isAutoQuery) {
      if (!cnLoggedIn && !comLoggedIn) {
        ElMessage.info('QRZ.cn 和 QRZ.com 未登录，将只查询 QRZ.herbertgao.me')
      } else if (!cnLoggedIn) {
        ElMessage.info('QRZ.cn 未登录，将查询 QRZ.com 和 QRZ.herbertgao.me')
      } else if (!comLoggedIn) {
        ElMessage.info('QRZ.com 未登录，将查询 QRZ.cn 和 QRZ.herbertgao.me')
      }
    }

    // 并行查询三个数据源
    const queryPromises: Promise<{
      source: string
      callsign: string
      chinese_address?: string | null
      english_address?: string | null
      name?: string | null
      address?: string | null
      mail_method?: string | null
      updated_at?: string | null
      created_at?: string | null
    } | null>[] = []

    // QRZ.cn 查询（需要登录）
    if (cnLoggedIn) {
      queryPromises.push(
        invoke<{
          source: string
          callsign: string
          chinese_address: string | null
          english_address: string | null
          updated_at: string | null
        } | null>('qrz_query_callsign', {
          callsign: card.callsign
        }).catch(error => {
          console.error('QRZ.cn 查询失败:', error)
          return null
        })
      )
    }

    // QRZ.com 查询（需要登录）
    if (comLoggedIn) {
      queryPromises.push(
        invoke<{
          source: string
          callsign: string
          name: string | null
          address: string | null
          updated_at: string | null
        } | null>('qrz_com_query_callsign', {
          callsign: card.callsign
        }).catch(error => {
          console.error('QRZ.com 查询失败:', error)
          return null
        })
      )
    }

    // QRZ.herbertgao.me 查询（无需登录，始终查询）
    queryPromises.push(
      invoke<{
        source: string
        callsign: string
        name: string
        address: string
        mail_method: string
        created_at: string
      } | null>('qrz_herbertgao_query_callsign', {
        callsign: card.callsign
      }).catch(error => {
        // 静默处理错误，仅记录到控制台
        console.error('QRZ.herbertgao.me 查询失败:', error)
        return null
      })
    )

    const results = await Promise.all(queryPromises)
    const validResults = results.filter(r => r !== null)

    if (validResults.length === 0) {
      // 自动查询时不显示警告
      if (!isAutoQuery) {
        ElMessage.warning('未找到该呼号的地址信息')
      }
      return
    }

    // 保存所有查询到的地址
    for (const result of validResults) {
      if (result) {
        // 根据数据源格式化地址
        let chineseAddress = null
        let englishAddress = null
        let name = null
        let mailMethod = null
        let updatedAt = null

        if (result.source === 'qrz.cn') {
          chineseAddress = result.chinese_address || null
          englishAddress = result.english_address || null
          updatedAt = result.updated_at || null
        } else if (result.source === 'qrz.com') {
          // QRZ.com 的地址放到英文地址字段
          if (result.name) {
            englishAddress = result.name
            if (result.address) {
              englishAddress += '\n' + result.address
            }
          } else if (result.address) {
            englishAddress = result.address
          }
          updatedAt = result.updated_at || null
        } else if (result.source === 'QRZ卡片查询') {
          // QRZ卡片查询 使用独立的字段
          name = result.name || null
          mailMethod = result.mail_method || null
          englishAddress = result.address || null
          updatedAt = result.created_at || null
        }

        await invoke<CardWithProject>('save_card_address_cmd', {
          cardId: card.id,
          source: result.source,
          chineseAddress,
          englishAddress,
          name,
          mailMethod,
          updatedAt
        })
      }
    }

    // 重新加载卡片以获取最新的地址缓存
    const updatedCard = await invoke<CardWithProject>('get_card_cmd', {
      id: card.id
    })

    // 更新本地状态
    card.metadata = updatedCard.metadata

    // 重新加载地址缓存
    loadAddressCache()

    // 仅手动查询时显示成功提示
    if (!isAutoQuery) {
      const sourceNames = validResults.map(r => r!.source).join(' 和 ')
      ElMessage.success(`从 ${sourceNames} 查询到地址信息并已保存`)
    }

    // 通知父组件刷新
    emit('refresh')
  } catch (error) {
    // 仅手动查询时显示错误提示
    if (!isAutoQuery) {
      ElMessage.error(`查询地址失败: ${error}`)
    } else {
      console.error('自动查询地址失败:', error)
    }
  } finally {
    querying.value = false
  }
  }

  // 手动查询时使用全局 loading，自动查询时静默执行
  if (isAutoQuery) {
    await executeQuery()
  } else {
    await withLoading(executeQuery, '正在查询地址...')
  }
}

// 复制地址到剪贴板
const handleCopyAddress = async (address: string): Promise<void> => {
  try {
    await navigator.clipboard.writeText(address)
    ElMessage.success('地址已复制到剪贴板')
  } catch (error) {
    ElMessage.error('复制失败')
    console.error('复制到剪贴板失败:', error)
  }
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
    form.value = {
      method: '快递',
      remarks: ''
    }

    // 加载地址缓存（优先展示缓存）
    loadAddressCache()

    // 清除验证状态
    nextTick(() => {
      formRef.value?.clearValidate()
    })

    // 自动查询地址（在后台更新，不显示提示信息）
    nextTick(() => {
      handleQueryAddress(true)
    })
  }
})

// 关闭弹窗
const handleClose = (): void => {
  submitting.value = false
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

// 打印面单
const handlePrintWaybill = (): void => {
  waybillPrintDialogVisible.value = true
}

// 创建顺丰订单
const handleCreateSFOrder = (): void => {
  // 尝试从地址缓存中获取收件人信息
  if (addressCache.value.length > 0) {
    const addr = addressCache.value[0]
    // 尝试解析地址信息
    defaultRecipient.value = {
      name: addr.name || '',
      address: addr.chinese_address || addr.english_address || ''
    }
  } else {
    defaultRecipient.value = null
  }
  sfOrderDialogVisible.value = true
}

// 顺丰订单创建成功
const handleSFOrderSuccess = (order: SFOrder): void => {
  // 将运单号填入备注
  if (order.waybill_no) {
    form.value.remarks = order.waybill_no
    ElMessage.success(`运单号已填入备注：${order.waybill_no}`)
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
  align-items: flex-start;
  justify-content: center;
  max-height: 400px;
  overflow-y: auto;
}

.address-list {
  width: 100%;
}

.address-item {
  padding: 12px;
  margin-bottom: 12px;
  background-color: #f5f7fa;
  border: 1px solid #ebeef5;
  border-radius: 6px;
  transition: all 0.3s;
}

.address-item:hover {
  background-color: #ecf5ff;
  border-color: #c6e2ff;
}

.address-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.address-callsign {
  font-weight: 600;
  font-size: 14px;
  color: #303133;
}

.address-text {
  font-size: 13px;
  line-height: 1.6;
  color: #606266;
  margin-bottom: 8px;
  white-space: pre-wrap;
}

.address-text-en {
  font-size: 12px;
  line-height: 1.6;
  color: #909399;
  margin-bottom: 8px;
  font-family: monospace;
  white-space: pre-wrap;
}

.address-meta {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: #909399;
}

.address-actions {
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid #ebeef5;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  flex-wrap: wrap;
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
