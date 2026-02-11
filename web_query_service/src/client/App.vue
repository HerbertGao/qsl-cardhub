<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import SearchBox from './components/SearchBox.vue'
import ResultList from './components/ResultList.vue'
import SubscribeCard from './components/SubscribeCard.vue'
import MathCaptcha from './components/MathCaptcha.vue'
import { buildSignedUrl } from './utils/sign'

interface CardItem {
  id: string
  project_name: string | null
  status: string
  distribution: {
    method?: string
    proxy_callsign?: string
    remarks?: string
  } | null
}

interface QueryResponse {
  success: boolean
  callsign?: string
  items?: CardItem[]
  message?: string
}

const callsign = ref('')
const loading = ref(false)
const error = ref('')
const result = ref<QueryResponse | null>(null)
const wechatAppId = ref('')
const wechatSubscribeEnabled = ref(false)
const signKey = ref<string | null>(null)
const captchaEnabled = ref(false)
const showCaptcha = ref(false)
const filing = ref<{ domain?: string; icp?: string; police?: string; police_code?: string } | null>(null)

const showFiling = computed(() => filing.value?.domain && window.location.hostname === filing.value.domain)

const hasResults = computed(() => result.value?.success && (result.value.items?.length ?? 0) > 0)

onMounted(async () => {
  try {
    const response = await fetch('/api/config')
    const data = await response.json()
    wechatSubscribeEnabled.value = data.features?.wechat_subscribe ?? false
    captchaEnabled.value = data.features?.captcha ?? false
    wechatAppId.value = data.wechat_appid || ''
    signKey.value = data.sign_key || null
    filing.value = data.filing || null
  } catch {
    // 配置加载失败不影响查询功能
  }
})

async function handleSearch(query: string) {
  callsign.value = query.toUpperCase()
  if (!callsign.value) return

  loading.value = true
  error.value = ''
  result.value = null

  try {
    // 构建带签名的请求 URL
    const url = await buildSignedUrl(
      `/api/callsigns/${encodeURIComponent(callsign.value)}`,
      {},
      signKey.value
    )
    const response = await fetch(url)
    const data: QueryResponse = await response.json()

    if (!data.success) {
      error.value = data.message || '查询失败'
      return
    }

    result.value = data
  } catch (e) {
    error.value = e instanceof Error ? e.message : '网络请求失败'
  } finally {
    loading.value = false
  }
}

function handleSubscribe() {
  if (!wechatAppId.value) {
    alert('订阅收卡需要配置微信服务号，请联系管理员。')
    return
  }

  // 如果启用了验证码，先显示验证码弹窗
  if (captchaEnabled.value) {
    showCaptcha.value = true
    return
  }

  // 否则直接跳转微信授权
  redirectToWechatAuth()
}

function handleCaptchaSuccess(_token: string, _answer: number) {
  // 验证码验证成功，关闭弹窗并跳转微信授权
  // 注：当前实现中验证码仅在前端验证，后续可扩展为后端验证
  showCaptcha.value = false
  redirectToWechatAuth()
}

function handleCaptchaCancel() {
  showCaptcha.value = false
}

function redirectToWechatAuth() {
  const redirectUri = `${window.location.origin}/api/wechat/auth-callback`
  const authUrl = `https://open.weixin.qq.com/connect/oauth2/authorize?appid=${wechatAppId.value}&redirect_uri=${encodeURIComponent(redirectUri)}&response_type=code&scope=snsapi_userinfo&state=${encodeURIComponent(callsign.value)}#wechat_redirect`
  window.location.href = authUrl
}
</script>

<template>
  <div class="page">
    <header class="header">
      <div class="header-content">
        <h1 class="title">QSL 收卡查询</h1>
        <p class="subtitle">业余无线电卡片管理系统</p>
      </div>
    </header>

    <main class="main">
      <div class="container">
        <SearchBox
          :loading="loading"
          @search="handleSearch"
        />

        <div v-if="error" class="error-message">
          <svg class="error-icon" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
          </svg>
          <span>{{ error }}</span>
        </div>

        <ResultList
          v-if="result"
          :callsign="callsign"
          :items="result.items || []"
        />

        <SubscribeCard
          v-if="hasResults && wechatSubscribeEnabled"
          :callsign="callsign"
          @subscribe="handleSubscribe"
        />
      </div>
    </main>

    <!-- 验证码弹窗 -->
    <MathCaptcha
      :visible="showCaptcha"
      @success="handleCaptchaSuccess"
      @cancel="handleCaptchaCancel"
    />

    <footer class="footer">
      <div v-if="showFiling" class="filing">
        <a
          v-if="filing?.icp"
          href="https://beian.miit.gov.cn/"
          target="_blank"
          rel="noopener"
          class="filing-link"
        >{{ filing.icp }}</a>
        <a
          v-if="filing?.police && filing?.police_code"
          :href="`https://beian.mps.gov.cn/#/query/webSearch?code=${filing.police_code}`"
          target="_blank"
          rel="noopener"
          class="filing-link"
        >{{ filing.police }}</a>
      </div>
      <p>&copy; {{ new Date().getFullYear() }} Herbert Software</p>
      <a href="https://github.com/HerbertGao/qsl-cardhub" target="_blank" rel="noopener" class="footer-link">GitHub</a>
    </footer>
  </div>
</template>

<style scoped>
.page {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.header {
  background: linear-gradient(135deg, var(--primary) 0%, #1d4ed8 100%);
  color: white;
  padding: 2.5rem 1rem;
  text-align: center;
}

.header-content {
  max-width: 480px;
  margin: 0 auto;
}

.title {
  font-size: 1.75rem;
  font-weight: 700;
  margin-bottom: 0.375rem;
  letter-spacing: -0.02em;
}

.subtitle {
  font-size: 1rem;
  opacity: 0.9;
}

.main {
  flex: 1;
  padding: 1.25rem 0 2rem;
}

.error-message {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  padding: 1rem;
  background: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: var(--radius);
  color: #dc2626;
  font-size: 0.9375rem;
  margin-top: 1rem;
}

.error-icon {
  width: 1.25rem;
  height: 1.25rem;
  flex-shrink: 0;
  margin-top: 0.125rem;
}

.footer {
  background: var(--card-bg);
  border-top: 1px solid var(--border);
  padding: 0.75rem 1rem;
  padding-bottom: calc(0.75rem + env(safe-area-inset-bottom, 0px));
  text-align: center;
  font-size: 0.875rem;
  color: var(--text-secondary);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
}

.footer p {
  margin: 0;
}

.footer-link {
  color: var(--primary);
  text-decoration: none;
  font-size: 0.8125rem;
}

.footer-link:hover {
  text-decoration: underline;
}

.filing {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 0.5rem 1rem;
}

.filing-link {
  color: var(--text-secondary);
  text-decoration: none;
  font-size: 0.8125rem;
}

.filing-link:hover {
  text-decoration: underline;
}
</style>
