<template>
  <div class="page-content">
    <h1>QRZ.cn 配置</h1>

    <el-card shadow="hover">
      <div style="margin-bottom: 20px">
        <p style="color: #909399; font-size: 14px; margin: 0">
          配置 QRZ.cn 登录凭据，用于查询呼号地址信息。凭据将加密保存到系统钥匙串或本地加密文件。
        </p>
      </div>

      <el-form
        :model="form"
        label-width="100px"
        style="max-width: 600px"
      >
        <el-form-item label="呼号">
          <el-input
            v-model="form.username"
            placeholder="请输入呼号"
            clearable
          />
        </el-form-item>

        <el-form-item label="密码">
          <el-input
            v-model="form.password"
            type="password"
            placeholder="请输入密码"
            show-password
            clearable
          />
        </el-form-item>

        <el-form-item>
          <el-alert
            v-if="hasSavedCredentials && !loginStatus.message"
            type="success"
            :closable="false"
            style="margin-bottom: 15px"
          >
            <template #title>
              <div style="display: flex; align-items: center; gap: 8px">
                <el-icon>
                  <CircleCheckFilled />
                </el-icon>
                <span>已保存凭据</span>
              </div>
            </template>
          </el-alert>

          <el-alert
            v-if="loginStatus.message"
            :type="loginStatus.type"
            :closable="false"
            style="margin-bottom: 15px"
          >
            {{ loginStatus.message }}
          </el-alert>

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
            :loading="loading"
            :disabled="!form.username || !form.password"
            @click="handleSaveAndLogin"
          >
            <el-icon v-if="!loading">
              <Check />
            </el-icon>
            保存并登录
          </el-button>

          <el-button
            type="danger"
            plain
            :disabled="!hasSavedCredentials"
            @click="handleClearCredentials"
          >
            <el-icon>
              <Delete />
            </el-icon>
            清除凭据
          </el-button>

          <el-button
            plain
            :loading="testing"
            :disabled="!isLoggedIn"
            @click="handleTestConnection"
          >
            <el-icon v-if="!testing">
              <Connection />
            </el-icon>
            测试连接
          </el-button>
        </el-form-item>

        <el-form-item label="登录状态">
          <el-tag :type="isLoggedIn ? 'success' : 'info'">
            {{ isLoggedIn ? '已登录' : '未登录' }}
          </el-tag>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useLoading } from '@/composables/useLoading'

const { withLoading } = useLoading()

interface LoginFormData {
  username: string
  password: string
}

interface LoginStatus {
  message: string
  type: 'success' | 'warning' | 'error' | 'info'
}

interface QrzCredentials {
  username: string | null
  password: string | null
}

const form = reactive<LoginFormData>({
  username: '',
  password: ''
})

const loading = ref<boolean>(false)
const testing = ref<boolean>(false)
const hasSavedCredentials = ref<boolean>(false)
const isLoggedIn = ref<boolean>(false)
const storageInfo = ref<string>('')
const loginStatus = reactive<LoginStatus>({
  message: '',
  type: 'info'
})

// 加载已保存的凭据
const loadCredentials = async (): Promise<void> => {
  try {
    // 检查钥匙串是否可用
    const keyringAvailable = await invoke<boolean>('check_keyring_available')
    storageInfo.value = keyringAvailable ? '系统钥匙串' : '本地加密文件'

    // 尝试加载凭据
    const credentials = await invoke<QrzCredentials>('qrz_load_credentials')
    if (credentials.username) {
      form.username = credentials.username
    }
    if (credentials.password) {
      form.password = credentials.password
      hasSavedCredentials.value = true
    }

    // 检查登录状态
    isLoggedIn.value = await invoke<boolean>('qrz_check_login_status')
  } catch (error) {
    console.error('加载凭据失败:', error)
  }
}

// 保存并登录
const handleSaveAndLogin = async (): Promise<void> => {
  if (!form.username || !form.password) {
    ElMessage.warning('请输入用户名和密码')
    return
  }

  loading.value = true
  loginStatus.message = ''

  try {
    const result = await withLoading(async () => await invoke<string>('qrz_save_and_login', {
        username: form.username,
        password: form.password
      }), '正在登录...')

    hasSavedCredentials.value = true
    isLoggedIn.value = true
    loginStatus.message = result
    loginStatus.type = 'success'

    ElMessage.success('登录成功，凭据已保存')

    // TODO: 保存用户名到配置文件 qrz.toml
    // 这需要后端提供配置文件读写接口
  } catch (error) {
    loginStatus.message = `登录失败: ${error}`
    loginStatus.type = 'error'
    isLoggedIn.value = false
    ElMessage.error(`登录失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 清除凭据
const handleClearCredentials = async (): Promise<void> => {
  try {
    await ElMessageBox.confirm(
        '确定要清除已保存的 QRZ.cn 凭据吗？清除后需要重新登录。',
        '确认清除',
        {
          confirmButtonText: '确定',
          cancelButtonText: '取消',
          type: 'warning'
        }
    )

    await invoke('qrz_clear_credentials')

    form.username = ''
    form.password = ''
    hasSavedCredentials.value = false
    isLoggedIn.value = false
    loginStatus.message = ''

    ElMessage.success('凭据已清除')

    // TODO: 清除配置文件中的用户名
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(`清除凭据失败: ${error}`)
    }
  }
}

// 测试连接
const handleTestConnection = async (): Promise<void> => {
  testing.value = true

  try {
    // 使用固定测试呼号 BY1CRA
    const result = await withLoading(async () => await invoke<unknown>('qrz_query_callsign', {
        callsign: 'BY1CRA'
      }), '正在测试连接...')

    if (result) {
      ElMessage.success('连接测试成功，可以正常查询地址信息')
    } else {
      ElMessage.warning('连接正常，但未找到测试呼号 BY1CRA 的地址信息')
    }
  } catch (error) {
    ElMessage.error(`连接测试失败: ${error}`)
  } finally {
    testing.value = false
  }
}

onMounted(() => {
  loadCredentials()
})
</script>

<style scoped>
.page-content h1 {
  margin-bottom: 20px;
}
</style>
