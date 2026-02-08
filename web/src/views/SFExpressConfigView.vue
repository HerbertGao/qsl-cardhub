<template>
  <div class="sf-config-page">
    <h1>顺丰速运配置</h1>

    <el-tabs
      v-model="activeTab"
      tab-position="left"
      class="sf-config-tabs"
    >
      <!-- API 配置面板 -->
      <el-tab-pane
        label="API 凭据配置"
        name="api"
      >
        <div class="tab-content">
          <div style="margin-bottom: 20px">
            <p style="color: #909399; font-size: 14px; margin: 0">
              配置顺丰速运 API 凭据，用于打印运单面单。凭据将加密保存到本地加密文件。
            </p>
          </div>

          <el-form
            :model="form"
            label-width="120px"
            style="max-width: 600px"
          >
            <!-- 参数来源选择 -->
            <el-form-item label="参数来源">
              <el-radio-group
                v-model="configMode"
                @change="handleConfigModeChange"
              >
                <el-radio
                  value="default"
                  :disabled="!defaultApiConfig?.enabled"
                >
                  使用默认参数
                </el-radio>
                <el-radio value="custom">
                  使用自定义参数（推荐）
                </el-radio>
              </el-radio-group>
            </el-form-item>

            <!-- 默认参数模式提示 -->
            <el-form-item v-if="configMode === 'default'">
              <el-alert
                type="info"
                :closable="false"
                show-icon
              >
                <template #title>
                  默认参数为公共资源，可能随时调整。如需更稳定的服务，建议申请专属凭据。
                </template>
              </el-alert>
            </el-form-item>

            <!-- 自定义参数模式提示 -->
            <el-form-item v-if="configMode === 'custom'">
              <el-alert
                type="info"
                :closable="false"
              >
                <template #title>
                  <span>前往 </span>
                  <a
                    href="https://open.sf-express.com/"
                    target="_blank"
                    style="color: #409eff"
                  >顺丰开放平台</a>
                  <span> 申请您自己的 API 凭据</span>
                </template>
              </el-alert>
            </el-form-item>

            <el-form-item label="环境">
              <el-radio-group v-model="form.environment">
                <el-radio value="sandbox">
                  沙箱环境
                </el-radio>
                <el-radio value="production">
                  生产环境
                </el-radio>
              </el-radio-group>
            </el-form-item>

            <el-form-item label="顾客编码">
              <el-input
                v-model="form.partnerId"
                :placeholder="configMode === 'default' ? '使用默认参数' : '请输入顺丰顾客编码（partnerID）'"
                :disabled="configMode === 'default'"
                clearable
              />
            </el-form-item>

            <el-form-item label="模板编码">
              <el-input
                v-model="form.templateCode"
                disabled
              />
              <div style="font-size: 12px; color: #909399; margin-top: 4px">
                固定使用 76mm × 130mm 标准模板
              </div>
            </el-form-item>

            <el-divider v-if="configMode === 'custom'" />

            <!-- 沙箱环境校验码（仅自定义模式显示） -->
            <el-form-item
              v-if="configMode === 'custom' && form.environment === 'sandbox'"
              label="沙箱校验码"
            >
              <el-input
                v-model="form.checkwordSandbox"
                type="password"
                placeholder="请输入沙箱环境校验码"
                show-password
                clearable
              />
              <div
                v-if="hasSandboxCheckword"
                style="font-size: 12px; color: #67c23a; margin-top: 4px"
              >
                <el-icon><CircleCheckFilled /></el-icon>
                已保存
              </div>
            </el-form-item>

            <!-- 生产环境校验码（仅自定义模式显示） -->
            <el-form-item
              v-if="configMode === 'custom' && form.environment === 'production'"
              label="生产校验码"
            >
              <el-input
                v-model="form.checkwordProd"
                type="password"
                placeholder="请输入生产环境校验码"
                show-password
                clearable
              />
              <div
                v-if="hasProdCheckword"
                style="font-size: 12px; color: #67c23a; margin-top: 4px"
              >
                <el-icon><CircleCheckFilled /></el-icon>
                已保存
              </div>
            </el-form-item>

            <!-- 默认模式下显示校验码状态 -->
            <el-form-item
              v-if="configMode === 'default'"
              label="校验码状态"
            >
              <el-tag
                v-if="form.environment === 'sandbox' && defaultApiConfig?.has_sandbox_checkword"
                type="success"
              >
                沙箱校验码已配置
              </el-tag>
              <el-tag
                v-else-if="form.environment === 'production' && defaultApiConfig?.has_prod_checkword"
                type="success"
              >
                生产校验码已配置
              </el-tag>
              <el-tag
                v-else
                type="danger"
              >
                当前环境校验码未配置
              </el-tag>
            </el-form-item>

            <el-form-item>
              <el-alert
                v-if="storageInfo"
                type="info"
                :closable="false"
                style="margin-bottom: 15px"
              >
                <template #title>
                  <div style="font-size: 13px">
                    存储方式：{{ storageInfo }}
                  </div>
                </template>
              </el-alert>
            </el-form-item>

            <el-form-item>
              <el-button
                type="primary"
                :loading="saving"
                :disabled="configMode === 'custom' && !form.partnerId"
                @click="handleSave"
              >
                <el-icon v-if="!saving">
                  <Check />
                </el-icon>
                保存配置
              </el-button>

              <el-button
                type="danger"
                plain
                :disabled="!hasConfig"
                @click="handleClear"
              >
                <el-icon>
                  <Delete />
                </el-icon>
                清除凭据
              </el-button>
            </el-form-item>

            <el-form-item label="当前状态">
              <el-tag :type="currentStatusType">
                {{ currentStatusText }}
              </el-tag>
            </el-form-item>
          </el-form>
        </div>
      </el-tab-pane>

      <!-- 寄件人配置面板 -->
      <el-tab-pane
        label="寄件人信息"
        name="sender"
      >
        <div class="tab-content">
          <div style="margin-bottom: 20px">
            <p style="color: #909399; font-size: 14px; margin: 0">
              配置默认寄件人信息，创建顺丰订单时将使用此信息。
            </p>
          </div>

          <el-form
            ref="senderFormRef"
            :model="senderForm"
            :rules="senderRules"
            label-width="120px"
            style="max-width: 600px"
          >
            <el-form-item
              label="寄件人姓名"
              prop="name"
            >
              <el-input
                v-model="senderForm.name"
                placeholder="请输入寄件人姓名"
                clearable
              />
            </el-form-item>

            <el-form-item
              label="手机号"
              prop="phone"
            >
              <el-input
                v-model="senderForm.phone"
                placeholder="请输入手机号码"
                clearable
                maxlength="11"
              />
            </el-form-item>

            <el-form-item
              label="固定电话"
              prop="mobile"
            >
              <el-input
                v-model="senderForm.mobile"
                placeholder="选填，格式如 010-12345678"
                clearable
              />
            </el-form-item>

            <el-form-item
              label="所在地区"
              prop="province"
            >
              <AddressSelector
                v-model:province="senderForm.province"
                v-model:city="senderForm.city"
                v-model:district="senderForm.district"
              />
            </el-form-item>

            <el-form-item
              label="详细地址"
              prop="address"
            >
              <el-input
                v-model="senderForm.address"
                type="textarea"
                :rows="2"
                placeholder="请输入详细地址（街道、门牌号等）"
              />
            </el-form-item>

            <el-form-item>
              <el-button
                type="primary"
                :loading="savingSender"
                @click="handleSaveSender"
              >
                <el-icon v-if="!savingSender">
                  <Check />
                </el-icon>
                保存寄件人
              </el-button>
            </el-form-item>
          </el-form>
        </div>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { FormInstance, FormRules } from 'element-plus'
import type { SenderInfo } from '@/types/models'
import AddressSelector from '@/components/sf-express/AddressSelector.vue'
import { useLoading } from '@/composables/useLoading'
import { consumeNavigationParams } from '@/stores/navigationStore'

const { withLoading } = useLoading()

// Tab 激活项
const activeTab = ref('api')

interface SFConfigResponse {
  environment: string
  partner_id: string
  template_code: string
  has_prod_checkword: boolean
  has_sandbox_checkword: boolean
  use_default: boolean
}

interface FormData {
  environment: string
  partnerId: string
  templateCode: string
  checkwordProd: string
  checkwordSandbox: string
}

const form = reactive<FormData>({
  environment: 'sandbox',
  partnerId: '',
  templateCode: 'fm_76130_standard_HBTRJT0FNP6E',
  checkwordProd: '',
  checkwordSandbox: ''
})

const saving = ref<boolean>(false)
const storageInfo = ref<string>('')
const hasProdCheckword = ref<boolean>(false)
const hasSandboxCheckword = ref<boolean>(false)

// 默认参数配置
interface DefaultApiConfig {
  enabled: boolean
  partner_id_masked: string
  has_sandbox_checkword: boolean
  has_prod_checkword: boolean
}

const defaultApiConfig = ref<DefaultApiConfig | null>(null)
const configMode = ref<'default' | 'custom'>('custom')

// 寄件人表单
const senderFormRef = ref<FormInstance | null>(null)
const savingSender = ref(false)
const currentSenderId = ref<string | null>(null)

const senderForm = reactive({
  name: '',
  phone: '',
  mobile: '',
  province: '',
  city: '',
  district: '',
  address: ''
})

// 验证规则
const senderRules: FormRules = {
  name: [
    { required: true, message: '请输入寄件人姓名', trigger: 'blur' },
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

// 是否已配置
const hasConfig = computed<boolean>(() => form.partnerId !== '' || hasProdCheckword.value || hasSandboxCheckword.value)

// 当前状态类型
const currentStatusType = computed<'success' | 'warning' | 'info'>(() => {
  if (!form.partnerId) return 'info'

  if (form.environment === 'production') {
    return hasProdCheckword.value ? 'success' : 'warning'
  } else {
    return hasSandboxCheckword.value ? 'success' : 'warning'
  }
})

// 当前状态文本
const currentStatusText = computed<string>(() => {
  if (!form.partnerId) return '未配置'

  if (form.environment === 'production') {
    return hasProdCheckword.value ? '生产环境已就绪' : '缺少生产校验码'
  } else {
    return hasSandboxCheckword.value ? '沙箱环境已就绪' : '缺少沙箱校验码'
  }
})

// 加载配置
const loadConfig = async (setDefaultTab = false): Promise<void> => {
  try {
    // 存储方式固定为本地加密文件
    storageInfo.value = '本地加密文件'

    // 加载配置
    const config = await invoke<SFConfigResponse>('sf_load_config')

    console.log('加载顺丰配置:', config)

    form.environment = config.environment
    form.templateCode = config.template_code
    hasProdCheckword.value = config.has_prod_checkword
    hasSandboxCheckword.value = config.has_sandbox_checkword

    // 根据保存的配置模式恢复状态
    if (config.use_default && defaultApiConfig.value?.enabled) {
      // 已保存为使用默认参数
      configMode.value = 'default'
      form.partnerId = defaultApiConfig.value.partner_id_masked
    } else if (config.partner_id) {
      // 已保存自定义参数
      configMode.value = 'custom'
      form.partnerId = config.partner_id
    } else if (defaultApiConfig.value?.enabled) {
      // 新用户且默认参数可用，默认选中"使用默认参数"
      configMode.value = 'default'
      form.partnerId = defaultApiConfig.value.partner_id_masked
    } else {
      // 新用户且无默认参数
      configMode.value = 'custom'
      form.partnerId = ''
    }

    // 仅在初始加载时根据 API 配置状态设置默认选中的 Tab
    if (setDefaultTab) {
      if (!config.partner_id) {
        activeTab.value = 'api'
      } else {
        activeTab.value = 'sender'
      }
    }
  } catch (error) {
    console.error('加载配置失败:', error)
    ElMessage.error(`加载配置失败: ${error}`)
  }
}

// 加载默认 API 配置
const loadDefaultApiConfig = async (): Promise<void> => {
  try {
    defaultApiConfig.value = await invoke<DefaultApiConfig>('sf_get_default_api_config')
    console.log('默认 API 配置:', defaultApiConfig.value)
  } catch (error) {
    console.error('加载默认 API 配置失败:', error)
    defaultApiConfig.value = null
  }
}

// 切换配置模式
const handleConfigModeChange = async (mode: 'default' | 'custom'): Promise<void> => {
  configMode.value = mode

  if (mode === 'default' && defaultApiConfig.value?.enabled) {
    // 使用默认参数时，显示脱敏的顾客编码
    form.partnerId = defaultApiConfig.value.partner_id_masked
  } else if (mode === 'custom') {
    // 切换到自定义模式时，如果当前显示的是脱敏的编码，清空
    if (form.partnerId.includes('***')) {
      form.partnerId = ''
    }
  }
}

// 保存配置
const handleSave = async (): Promise<void> => {
  saving.value = true

  try {
    await withLoading(async () => {
      if (configMode.value === 'default') {
        // 使用默认参数
        console.log('应用默认 API 配置')
        await invoke('sf_apply_default_api_config', {
          environment: form.environment
        })
      } else {
        // 使用自定义参数
        if (!form.partnerId) {
          ElMessage.warning('请输入顾客编码')
          return
        }

        console.log('保存顺丰配置:', {
          environment: form.environment,
          partnerId: form.partnerId,
          hasCheckwordProd: !!form.checkwordProd,
          hasCheckwordSandbox: !!form.checkwordSandbox
        })

        await invoke('sf_save_config', {
          environment: form.environment,
          partnerId: form.partnerId,
          checkwordProd: form.checkwordProd || null,
          checkwordSandbox: form.checkwordSandbox || null
        })

        // 清空校验码输入框
        if (form.checkwordProd) {
          form.checkwordProd = ''
        }
        if (form.checkwordSandbox) {
          form.checkwordSandbox = ''
        }
      }

      ElMessage.success('配置已保存')

      // 重新加载配置以验证保存成功
      await loadConfig()
    }, '正在保存配置...')
  } catch (error) {
    ElMessage.error(`保存配置失败: ${error}`)
  } finally {
    saving.value = false
  }
}

// 清除配置
const handleClear = async (): Promise<void> => {
  try {
    await ElMessageBox.confirm(
      '确定要清除所有顺丰速运凭据吗？清除后需要重新配置。',
      '确认清除',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    await invoke('sf_clear_config')

    // 重置表单
    form.environment = 'sandbox'
    form.partnerId = ''
    form.checkwordProd = ''
    form.checkwordSandbox = ''
    hasProdCheckword.value = false
    hasSandboxCheckword.value = false

    ElMessage.success('凭据已清除')
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(`清除凭据失败: ${error}`)
    }
  }
}

// 加载默认寄件人
const loadDefaultSender = async (): Promise<void> => {
  try {
    const sender = await invoke<SenderInfo | null>('sf_get_default_sender')
    if (sender) {
      currentSenderId.value = sender.id
      senderForm.name = sender.name
      senderForm.phone = sender.phone
      senderForm.mobile = sender.mobile || ''
      senderForm.province = sender.province
      senderForm.city = sender.city
      senderForm.district = sender.district
      senderForm.address = sender.address
    }
  } catch (error) {
    console.error('加载寄件人失败:', error)
  }
}

// 保存寄件人
const handleSaveSender = async (): Promise<void> => {
  if (!senderFormRef.value) return

  try {
    await senderFormRef.value.validate()
  } catch {
    return
  }

  // 验证地区选择
  if (!senderForm.city) {
    ElMessage.warning('请选择城市')
    return
  }
  if (!senderForm.district) {
    ElMessage.warning('请选择或输入区县')
    return
  }

  savingSender.value = true

  try {
    await withLoading(async () => {
      const senderData = {
        name: senderForm.name.trim(),
        phone: senderForm.phone.trim(),
        mobile: senderForm.mobile?.trim() || null,
        province: senderForm.province,
        city: senderForm.city,
        district: senderForm.district,
        address: senderForm.address.trim(),
        isDefault: true
      }

      if (currentSenderId.value) {
        // 更新现有寄件人
        await invoke<SenderInfo>('sf_update_sender', {
          id: currentSenderId.value,
          ...senderData
        })
      } else {
        // 创建新寄件人
        const result = await invoke<SenderInfo>('sf_create_sender', senderData)
        currentSenderId.value = result.id
      }

      ElMessage.success('寄件人信息已保存')
    }, '正在保存...')
  } catch (error) {
    ElMessage.error(`保存失败: ${error}`)
  } finally {
    savingSender.value = false
  }
}

onMounted(async () => {
  const navParams = consumeNavigationParams()
  await loadDefaultApiConfig()
  await loadConfig(!navParams.tab)
  // 导航参数指定了目标 tab 时，优先使用
  if (navParams.tab) {
    activeTab.value = navParams.tab
  }
  loadDefaultSender()
})
</script>

<style scoped>
.sf-config-page {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.sf-config-page h1 {
  margin-bottom: 20px;
  flex-shrink: 0;
}

.sf-config-tabs {
  flex: 1;
  min-height: 0;
}

.tab-content {
  padding: 0 20px;
}

:deep(.el-tabs__header.is-left) {
  margin-right: 20px;
}

:deep(.el-tabs__item) {
  height: 50px;
  line-height: 50px;
  font-size: 14px;
}

:deep(.el-tabs__nav-wrap.is-left::after) {
  width: 1px;
}
</style>
