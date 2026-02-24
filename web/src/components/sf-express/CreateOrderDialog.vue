<template>
  <el-dialog
    v-model="dialogVisible"
    title="顺丰速运下单"
    width="650px"
    :close-on-click-modal="false"
    @close="handleClose"
  >
    <div class="order-content">
      <!-- API 未配置提示 -->
      <el-alert
        v-if="!apiConfigured"
        title="请先配置顺丰速运 API"
        type="warning"
        show-icon
        :closable="false"
        style="margin-bottom: 12px"
      >
        <template #default>
          <div style="display: flex; align-items: center; gap: 8px">
            <span>顺丰 API 凭据未配置或配置不完整，无法创建订单</span>
            <el-button
              type="primary"
              size="small"
              @click="goToConfig('api')"
            >
              去配置
            </el-button>
          </div>
        </template>
      </el-alert>

      <!-- 沙箱环境提示 -->
      <el-alert
        v-if="isSandbox"
        title="当前为沙箱环境"
        type="warning"
        show-icon
        :closable="false"
        style="margin-bottom: 12px"
      >
        <template #default>
          <div style="display: flex; align-items: center; gap: 4px">
            <span>订单将不会被真实派发，如需切换至生产环境</span>
            <el-link
              type="warning"
              :underline="true"
              @click="goToConfig('api')"
            >
              请点击此处
            </el-link>
          </div>
        </template>
      </el-alert>

      <!-- 寄件人信息 -->
      <div class="section">
        <div class="section-title">
          寄件人信息
        </div>

        <div
          v-if="sender"
          class="sender-info-compact"
        >
          <span class="sender-contact">
            <strong>{{ sender.name }}</strong>
            <span class="sender-phone">{{ sender.phone }}</span>
          </span>
          <span class="sender-address">
            {{ sender.province }}{{ sender.city }}{{ sender.district }}{{ sender.address }}
          </span>
        </div>

        <el-empty
          v-else
          description="请先配置寄件人信息"
          :image-size="40"
        >
          <el-button
            type="primary"
            size="small"
            @click="goToConfig('sender')"
          >
            去配置
          </el-button>
        </el-empty>
      </div>

      <!-- 托寄物和付款方式 -->
      <div class="section">
        <div class="section-title">
          托寄物信息
        </div>

        <el-form
          label-width="80px"
          class="compact-form"
        >
          <el-row :gutter="16">
            <el-col :span="12">
              <el-form-item label="物品名称">
                <el-input
                  v-model="cargoName"
                  placeholder="请输入托寄物名称"
                  clearable
                />
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item label="付款方式">
                <el-radio-group v-model="payMethod">
                  <el-radio :value="1">
                    寄方付
                  </el-radio>
                  <el-radio :value="2">
                    收方付
                  </el-radio>
                </el-radio-group>
              </el-form-item>
            </el-col>
          </el-row>
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
          class="compact-form"
        >
          <el-row :gutter="16">
            <el-col :span="12">
              <el-form-item
                label="姓名"
                prop="name"
              >
                <el-input
                  v-model="recipientForm.name"
                  placeholder="收件人姓名"
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
                  placeholder="手机号码"
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
            <div style="width: 100%">
              <el-input
                v-model="recipientForm.address"
                type="textarea"
                :rows="2"
                placeholder="粘贴完整收件信息可智能识别，如：张三 13812345678 广东省深圳市南山区科技园路1号"
              />
              <el-button
                type="primary"
                size="small"
                link
                style="margin-top: 4px"
                @click="handleSmartParse"
              >
                智能识别
              </el-button>
            </div>
          </el-form-item>
        </el-form>
      </div>
    </div>

    <template #footer>
      <el-button @click="dialogVisible = false">
        取消
      </el-button>
      <el-button
        type="primary"
        :loading="submitting"
        :disabled="!sender || !apiConfigured"
        @click="handleSubmit"
      >
        提交并确认订单
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import type { SenderInfo, RecipientInfo, CreateOrderResponse, ConfirmOrderResponse, SFOrder } from '@/types/models'
import AddressSelector from './AddressSelector.vue'
import { useLoading } from '@/composables/useLoading'
import { parseAddress } from '@/utils/addressParser'

const { withLoading } = useLoading()

interface Props {
  visible: boolean
  cardId?: string | null
  defaultRecipient?: Partial<RecipientInfo> | null
}

interface Emits {
  (e: 'update:visible', value: boolean): void
  (e: 'success', order: SFOrder): void
  (e: 'go-config', tab?: string): void
}

const props = withDefaults(defineProps<Props>(), {
  visible: false,
  cardId: null,
  defaultRecipient: null
})

const emit = defineEmits<Emits>()

const formRef = ref<FormInstance | null>(null)
const submitting = ref(false)

// 寄件人
const sender = ref<SenderInfo | null>(null)

// API 配置状态
const apiConfigured = ref(true)
const currentEnvironment = ref('')

// 托寄物名称（从 localStorage 读取上次使用的值，默认 QSL卡片）
const CARGO_NAME_STORAGE_KEY = 'sf_last_cargo_name'
const cargoName = ref(localStorage.getItem(CARGO_NAME_STORAGE_KEY) || 'QSL卡片')

// 付款方式（1=寄方付, 2=收方付, 3=第三方付）
const payMethod = ref(2)

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

// 是否为沙箱环境
const isSandbox = computed(() => apiConfigured.value && currentEnvironment.value === 'sandbox')

// 双向绑定
const dialogVisible = computed({
  get: () => props.visible,
  set: (val) => emit('update:visible', val)
})

// 监听对话框打开
watch(() => props.visible, async (newVal) => {
  if (newVal) {
    await loadSender()
    await checkApiConfig()

    // 加载上次使用的托寄物名称
    cargoName.value = localStorage.getItem(CARGO_NAME_STORAGE_KEY) || 'QSL卡片'

    // 先重置收件人表单
    Object.assign(recipientForm, {
      name: '',
      phone: '',
      mobile: null,
      province: '',
      city: '',
      district: '',
      address: ''
    })

    // 清除验证状态
    formRef.value?.clearValidate()

    // 填充默认收件人信息（如果有）
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

// 检查 API 配置状态
async function checkApiConfig(): Promise<void> {
  try {
    const config = await invoke<{
      partner_id: string
      has_prod_checkword: boolean
      has_sandbox_checkword: boolean
      environment: string
    }>('sf_load_config')

    // 检查是否已配置：需要有顾客编码，且当前环境的校验码已配置
    const hasCheckword = config.environment === 'production'
      ? config.has_prod_checkword
      : config.has_sandbox_checkword

    apiConfigured.value = config.partner_id !== '' && hasCheckword
    currentEnvironment.value = config.environment
  } catch (error) {
    console.error('检查 API 配置失败:', error)
    apiConfigured.value = false
    currentEnvironment.value = ''
  }
}

// 跳转到配置页面
function goToConfig(tab?: string): void {
  dialogVisible.value = false
  emit('go-config', tab)
}

// 智能识别地址（从详细地址框中的文本解析）
function handleSmartParse(): void {
  if (!recipientForm.address.trim()) {
    ElMessage.warning('请先在详细地址框中输入地址文本')
    return
  }

  const parsed = parseAddress(recipientForm.address)

  if (parsed.name) recipientForm.name = parsed.name
  if (parsed.phone) recipientForm.phone = parsed.phone
  if (parsed.province) recipientForm.province = parsed.province
  if (parsed.city) recipientForm.city = parsed.city
  if (parsed.district) recipientForm.district = parsed.district
  recipientForm.address = parsed.address

  formRef.value?.clearValidate()
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
  payMethod.value = 2
  formRef.value?.clearValidate()
}

// 提交并确认订单
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
    // 1. 创建订单
    const createResult = await withLoading(async () => await invoke<CreateOrderResponse>('sf_create_order', {
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
      }), '正在创建订单...')

    // 保存托寄物名称供下次使用
    const trimmedCargoName = cargoName.value.trim()
    if (trimmedCargoName) {
      localStorage.setItem(CARGO_NAME_STORAGE_KEY, trimmedCargoName)
    }

    // 2. 检查 filter_result 并确认订单
    const filterResult = createResult.filter_result
    if (filterResult === 3) {
      ElMessage.error('该地区不可收派，订单已保留为待确认状态')
      dialogVisible.value = false
      return
    }

    if (filterResult === 1) {
      try {
        await ElMessageBox.confirm(
          '该地区需要人工确认，是否继续确认订单？',
          '提示',
          { confirmButtonText: '继续确认', cancelButtonText: '稍后确认', type: 'warning' }
        )
      } catch {
        // 用户选择稍后确认 - 发出 success 事件以便保存运单号
        ElMessage.info('订单已创建，请稍后在订单列表中确认')
        emit('success', createResult.local_order)
        dialogVisible.value = false
        return
      }
    }

    const confirmResult = await withLoading(async () => await invoke<ConfirmOrderResponse>('sf_confirm_order', {
        orderId: createResult.order_id
      }), '正在确认订单...')

    const waybillNo = confirmResult.waybill_no_list[0]

    // 3. 自动打印面单
    try {
      const printerConfig = await invoke<{ printer: { name: string } }>('get_printer_config')
      const printerName = printerConfig.printer.name

      if (printerName) {
        const fetchResult = await withLoading(async () => await invoke<{ pdf_data: string; waybill_no: string }>('sf_fetch_waybill', {
            waybillNo
          }), '正在获取面单...')

        await withLoading(async () => await invoke<string>('sf_print_waybill', {
            pdfData: fetchResult.pdf_data,
            printerName
          }), '正在打印面单...')

        ElMessage.success(`订单已确认并打印，运单号: ${waybillNo}`)
      } else {
        ElMessage.success(`订单已确认，运单号: ${waybillNo}。请配置打印机后手动打印面单`)
      }
    } catch (printError) {
      console.error('自动打印失败:', printError)
      ElMessage.warning(`订单已确认（运单号: ${waybillNo}），但打印失败: ${printError}`)
    }

    emit('success', confirmResult.local_order)
    dialogVisible.value = false
  } catch (error) {
    ElMessage.error(`操作失败: ${error}`)
  } finally {
    submitting.value = false
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
  /* 移除高度限制，让内容自适应 */
}

.section {
  margin-bottom: 16px;
  padding: 12px;
  background: #fafafa;
  border-radius: 8px;
}

.section:last-child {
  margin-bottom: 0;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 10px;
  padding-left: 8px;
  border-left: 3px solid #409eff;
}

/* 寄件人紧凑展示 */
.sender-info-compact {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px 12px;
  background: #fff;
  border-radius: 6px;
  border: 1px solid #ebeef5;
  font-size: 13px;
}

.sender-contact {
  display: flex;
  align-items: center;
  gap: 12px;
}

.sender-contact strong {
  color: #303133;
}

.sender-phone {
  color: #606266;
}

.sender-address {
  color: #909399;
  font-size: 12px;
}

/* 寄件人未配置空状态紧凑 */
.section :deep(.el-empty) {
  padding: 4px 0;
}

/* 紧凑表单 */
.compact-form :deep(.el-form-item) {
  margin-bottom: 14px;
}

.compact-form :deep(.el-form-item:last-child) {
  margin-bottom: 0;
}
</style>
