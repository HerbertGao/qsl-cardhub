<template>
  <div class="page-content">
    <h1>顺丰速运配置</h1>

    <el-card shadow="hover">
      <div style="margin-bottom: 20px">
        <p style="color: #909399; font-size: 14px; margin: 0">
          配置顺丰速运 API 凭据，用于打印运单面单。凭据将加密保存到系统钥匙串或本地加密文件。
        </p>
      </div>

      <el-form
        :model="form"
        label-width="120px"
        style="max-width: 600px"
      >
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
            placeholder="请输入顺丰顾客编码（partnerID）"
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

        <el-divider />

        <!-- 沙箱环境校验码 -->
        <el-form-item
          v-if="form.environment === 'sandbox'"
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

        <!-- 生产环境校验码 -->
        <el-form-item
          v-if="form.environment === 'production'"
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
            :disabled="!form.partnerId"
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
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'

interface SFConfigResponse {
  environment: string
  partner_id: string
  template_code: string
  has_prod_checkword: boolean
  has_sandbox_checkword: boolean
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

// 是否已配置
const hasConfig = computed<boolean>(() => {
  return form.partnerId !== '' || hasProdCheckword.value || hasSandboxCheckword.value
})

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
const loadConfig = async (): Promise<void> => {
  try {
    // 检查钥匙串是否可用
    const keyringAvailable = await invoke<boolean>('check_keyring_available')
    storageInfo.value = keyringAvailable ? '系统钥匙串' : '本地加密文件'

    // 加载配置
    const config = await invoke<SFConfigResponse>('sf_load_config')

    console.log('加载顺丰配置:', config)

    form.environment = config.environment
    form.partnerId = config.partner_id
    form.templateCode = config.template_code
    hasProdCheckword.value = config.has_prod_checkword
    hasSandboxCheckword.value = config.has_sandbox_checkword
  } catch (error) {
    console.error('加载配置失败:', error)
    ElMessage.error(`加载配置失败: ${error}`)
  }
}

// 保存配置
const handleSave = async (): Promise<void> => {
  if (!form.partnerId) {
    ElMessage.warning('请输入顾客编码')
    return
  }

  saving.value = true

  try {
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

    ElMessage.success('配置已保存')

    // 重新加载配置以验证保存成功
    await loadConfig()
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

onMounted(() => {
  loadConfig()
})
</script>

<style scoped>
.page-content h1 {
  margin-bottom: 20px;
}
</style>
