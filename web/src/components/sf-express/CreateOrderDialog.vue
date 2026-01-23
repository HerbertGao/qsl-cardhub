<template>
  <el-dialog
    v-model="dialogVisible"
    title="顺丰速运下单"
    width="700px"
    :close-on-click-modal="false"
    @close="handleClose"
  >
    <div class="order-content">
      <!-- 寄件人信息 -->
      <div class="section">
        <div class="section-title">
          寄件人信息
        </div>

        <div
          v-if="sender"
          class="sender-info"
        >
          <div class="info-row">
            <span class="label">姓名：</span>
            <span class="value">{{ sender.name }}</span>
          </div>
          <div class="info-row">
            <span class="label">电话：</span>
            <span class="value">{{ sender.phone }}</span>
          </div>
          <div class="info-row">
            <span class="label">地址：</span>
            <span class="value">
              {{ sender.province }}{{ sender.city }}{{ sender.district }}{{ sender.address }}
            </span>
          </div>
        </div>

        <el-empty
          v-else
          description="请先配置寄件人信息"
          :image-size="60"
        >
          <el-button
            type="primary"
            size="small"
            @click="goToConfig"
          >
            去配置
          </el-button>
        </el-empty>
      </div>

      <!-- 托寄物信息 -->
      <div class="section">
        <div class="section-title">
          托寄物信息
        </div>

        <el-form
          label-width="80px"
        >
          <el-form-item label="物品名称">
            <el-input
              v-model="cargoName"
              placeholder="请输入托寄物名称"
              clearable
            />
          </el-form-item>

          <el-form-item label="付款方式">
            <el-radio-group v-model="payMethod">
              <el-radio :value="1">
                寄方付
              </el-radio>
              <el-radio :value="2">
                收方付
              </el-radio>
              <el-radio :value="3">
                第三方付
              </el-radio>
            </el-radio-group>
          </el-form-item>
        </el-form>
      </div>

      <!-- 收件人信息 -->
      <div class="section">
        <div class="section-title">
          收件人信息
        </div>

        <el-form
          ref="formRef"
          :model="recipientForm"
          :rules="rules"
          label-width="80px"
        >
          <el-row :gutter="16">
            <el-col :span="12">
              <el-form-item
                label="姓名"
                prop="name"
              >
                <el-input
                  v-model="recipientForm.name"
                  placeholder="请输入收件人姓名"
                  clearable
                />
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item
                label="手机号"
                prop="phone"
              >
                <el-input
                  v-model="recipientForm.phone"
                  placeholder="请输入手机号码"
                  clearable
                  maxlength="11"
                />
              </el-form-item>
            </el-col>
          </el-row>

          <el-form-item
            label="所在地区"
            prop="province"
          >
            <AddressSelector
              v-model:province="recipientForm.province"
              v-model:city="recipientForm.city"
              v-model:district="recipientForm.district"
            />
          </el-form-item>

          <el-form-item
            label="详细地址"
            prop="address"
          >
            <el-input
              v-model="recipientForm.address"
              type="textarea"
              :rows="2"
              placeholder="请输入详细地址（街道、门牌号等）"
            />
          </el-form-item>
        </el-form>
      </div>

      <!-- 订单状态 -->
      <div
        v-if="orderResult"
        class="section"
      >
        <div class="section-title">
          下单结果
        </div>
        <el-result
          icon="success"
          title="下单成功"
          :sub-title="`运单号：${orderResult.waybill_no_list[0] || '待确认'}`"
        >
          <template #extra>
            <el-button
              v-if="orderResult.local_order.status === 'pending'"
              type="primary"
              @click="handleConfirmOrder"
            >
              立即确认订单
            </el-button>
            <el-button @click="dialogVisible = false">
              稍后确认
            </el-button>
          </template>
        </el-result>
      </div>
    </div>

    <template #footer>
      <el-button
        v-if="!orderResult"
        @click="dialogVisible = false"
      >
        取消
      </el-button>
      <el-button
        v-if="!orderResult"
        type="primary"
        :loading="submitting"
        :disabled="!sender"
        @click="handleSubmit"
      >
        提交订单
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import type { SenderInfo, RecipientInfo, CreateOrderResponse, SFOrder } from '@/types/models'
import AddressSelector from './AddressSelector.vue'
import { useLoading } from '@/composables/useLoading'

const { withLoading } = useLoading()

interface Props {
  visible: boolean
  cardId?: string | null
  defaultRecipient?: Partial<RecipientInfo> | null
}

interface Emits {
  (e: 'update:visible', value: boolean): void
  (e: 'success', order: SFOrder): void
  (e: 'go-config'): void
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  cardId: null,
  defaultRecipient: null
})

const emit = defineEmits<Emits>()

const formRef = ref<FormInstance | null>(null)
const submitting = ref(false)
const confirming = ref(false)

// 寄件人
const sender = ref<SenderInfo | null>(null)

// 托寄物名称（从 localStorage 读取上次使用的值，默认 QSL卡片）
const CARGO_NAME_STORAGE_KEY = 'sf_last_cargo_name'
const cargoName = ref(localStorage.getItem(CARGO_NAME_STORAGE_KEY) || 'QSL卡片')

// 付款方式（1=寄方付, 2=收方付, 3=第三方付）
const payMethod = ref(1)

// 收件人表单
const recipientForm = reactive<RecipientInfo>({
  name: '',
  phone: '',
  mobile: null,
  province: '',
  city: '',
  district: '',
  address: ''
})

// 订单结果
const orderResult = ref<CreateOrderResponse | null>(null)

// 验证规则
const rules: FormRules = {
  name: [
    { required: true, message: '请输入收件人姓名', trigger: 'blur' },
    { min: 2, max: 20, message: '姓名长度在 2 到 20 个字符', trigger: 'blur' }
  ],
  phone: [
    { required: true, message: '请输入手机号码', trigger: 'blur' },
    { pattern: /^1[3-9]\d{9}$/, message: '请输入正确的手机号码', trigger: 'blur' }
  ],
  province: [
    { required: true, message: '请选择省份', trigger: 'change' }
  ],
  address: [
    { required: true, message: '请输入详细地址', trigger: 'blur' },
    { min: 5, max: 100, message: '详细地址长度在 5 到 100 个字符', trigger: 'blur' }
  ]
}

// 双向绑定
const dialogVisible = computed({
  get: () => props.visible,
  set: (val) => emit('update:visible', val)
})

// 监听对话框打开
watch(() => props.visible, async (newVal) => {
  if (newVal) {
    orderResult.value = null
    await loadSender()

    // 加载上次使用的托寄物名称
    cargoName.value = localStorage.getItem(CARGO_NAME_STORAGE_KEY) || 'QSL卡片'

    // 填充默认收件人信息
    if (props.defaultRecipient) {
      Object.assign(recipientForm, props.defaultRecipient)
    }
  }
})

// 加载寄件人
async function loadSender(): Promise<void> {
  try {
    sender.value = await invoke<SenderInfo | null>('sf_get_default_sender')
  } catch (error) {
    console.error('加载寄件人失败:', error)
    ElMessage.error(`加载寄件人失败: ${error}`)
  }
}

// 跳转到配置页面
function goToConfig(): void {
  dialogVisible.value = false
  emit('go-config')
}

// 关闭对话框
function handleClose(): void {
  // 重置表单
  Object.assign(recipientForm, {
    name: '',
    phone: '',
    mobile: null,
    province: '',
    city: '',
    district: '',
    address: ''
  })
  cargoName.value = localStorage.getItem(CARGO_NAME_STORAGE_KEY) || 'QSL卡片'
  payMethod.value = 1
  formRef.value?.clearValidate()
  orderResult.value = null
}

// 提交订单
async function handleSubmit(): Promise<void> {
  if (!formRef.value || !sender.value) return

  try {
    await formRef.value.validate()
  } catch {
    return
  }

  // 验证地区选择
  if (!recipientForm.city) {
    ElMessage.warning('请选择城市')
    return
  }
  if (!recipientForm.district) {
    ElMessage.warning('请选择或输入区县')
    return
  }

  submitting.value = true

  try {
    const result = await withLoading(async () => {
      return await invoke<CreateOrderResponse>('sf_create_order', {
        params: {
          sender_id: sender.value!.id,
          recipient: {
            name: recipientForm.name.trim(),
            phone: recipientForm.phone.trim(),
            mobile: recipientForm.mobile || null,
            province: recipientForm.province,
            city: recipientForm.city,
            district: recipientForm.district,
            address: recipientForm.address.trim()
          },
          cargo_name: cargoName.value.trim() || null,
          pay_method: payMethod.value,
          card_id: props.cardId || null
        }
      })
    }, '正在创建订单...')

    orderResult.value = result

    // 保存托寄物名称供下次使用
    const trimmedCargoName = cargoName.value.trim()
    if (trimmedCargoName) {
      localStorage.setItem(CARGO_NAME_STORAGE_KEY, trimmedCargoName)
    }

    ElMessage.success('订单创建成功')
  } catch (error) {
    ElMessage.error(`下单失败: ${error}`)
  } finally {
    submitting.value = false
  }
}

// 确认订单
async function handleConfirmOrder(): Promise<void> {
  if (!orderResult.value) return

  confirming.value = true

  try {
    const result = await withLoading(async () => {
      return await invoke<{ order_id: string; waybill_no_list: string[]; local_order: SFOrder }>('sf_confirm_order', {
        orderId: orderResult.value!.order_id
      })
    }, '正在确认订单...')

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

onMounted(() => {
  if (props.visible) {
    loadSender()
  }
})
</script>

<style scoped>
.order-content {
  max-height: 60vh;
  overflow-y: auto;
}

.section {
  margin-bottom: 24px;
  padding: 16px;
  background: #fafafa;
  border-radius: 8px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 12px;
  padding-left: 8px;
  border-left: 3px solid #409eff;
}

.sender-info {
  padding: 12px;
  background: #fff;
  border-radius: 6px;
  border: 1px solid #ebeef5;
}

.info-row {
  margin-bottom: 8px;
  font-size: 13px;
}

.info-row:last-child {
  margin-bottom: 0;
}

.info-row .label {
  color: #909399;
  width: 60px;
  display: inline-block;
}

.info-row .value {
  color: #303133;
}
</style>
