<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  success: [token: string, answer: number]
  cancel: []
}>()

const loading = ref(false)
const error = ref('')
const question = ref('')
const token = ref('')
const answer = ref('')
const canvasRef = ref<HTMLCanvasElement | null>(null)

async function loadCaptcha() {
  loading.value = true
  error.value = ''

  try {
    const response = await fetch('/api/captcha')
    const data = await response.json()

    if (!data.success) {
      error.value = data.message || '获取验证码失败'
      return
    }

    question.value = data.question
    token.value = data.token
    answer.value = ''

    // 在 Canvas 上渲染验证码
    renderCaptcha(data.question)
  } catch (e) {
    error.value = e instanceof Error ? e.message : '网络请求失败'
  } finally {
    loading.value = false
  }
}

function renderCaptcha(text: string) {
  const canvas = canvasRef.value
  if (!canvas) return

  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const width = canvas.width
  const height = canvas.height

  // 清空画布
  ctx.clearRect(0, 0, width, height)

  // 背景
  ctx.fillStyle = '#f5f5f5'
  ctx.fillRect(0, 0, width, height)

  // 干扰线
  for (let i = 0; i < 4; i++) {
    ctx.strokeStyle = `hsl(${Math.random() * 360}, 50%, 70%)`
    ctx.lineWidth = 1
    ctx.beginPath()
    ctx.moveTo(Math.random() * width, Math.random() * height)
    ctx.lineTo(Math.random() * width, Math.random() * height)
    ctx.stroke()
  }

  // 干扰点
  for (let i = 0; i < 30; i++) {
    ctx.fillStyle = `hsl(${Math.random() * 360}, 50%, 60%)`
    ctx.beginPath()
    ctx.arc(Math.random() * width, Math.random() * height, 1, 0, Math.PI * 2)
    ctx.fill()
  }

  // 文字
  ctx.font = 'bold 24px Arial, sans-serif'
  ctx.fillStyle = '#333'
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'

  // 轻微旋转
  ctx.save()
  ctx.translate(width / 2, height / 2)
  ctx.rotate((Math.random() - 0.5) * 0.1)
  ctx.fillText(text, 0, 0)
  ctx.restore()
}

function handleSubmit() {
  const numAnswer = parseInt(answer.value, 10)
  if (isNaN(numAnswer)) {
    error.value = '请输入数字答案'
    return
  }

  emit('success', token.value, numAnswer)
}

function handleCancel() {
  emit('cancel')
}

function handleRefresh() {
  loadCaptcha()
}

// 当组件可见时加载验证码
watch(() => props.visible, (visible) => {
  if (visible) {
    loadCaptcha()
  }
})

onMounted(() => {
  if (props.visible) {
    loadCaptcha()
  }
})
</script>

<template>
  <div v-if="visible" class="captcha-overlay" @click.self="handleCancel">
    <div class="captcha-modal">
      <div class="captcha-header">
        <h3 class="captcha-title">安全验证</h3>
        <button class="captcha-close" @click="handleCancel" aria-label="关闭">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <div class="captcha-body">
        <p class="captcha-hint">请计算下图中的算式结果</p>

        <div class="captcha-canvas-wrapper">
          <canvas
            ref="canvasRef"
            width="200"
            height="60"
            class="captcha-canvas"
          />
          <button
            class="captcha-refresh"
            @click="handleRefresh"
            :disabled="loading"
            aria-label="刷新验证码"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" :class="{ spinning: loading }">
              <path stroke-linecap="round" stroke-linejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </button>
        </div>

        <div v-if="error" class="captcha-error">{{ error }}</div>

        <input
          v-model="answer"
          type="number"
          inputmode="numeric"
          class="captcha-input"
          placeholder="请输入计算结果"
          @keyup.enter="handleSubmit"
          :disabled="loading"
        />
      </div>

      <div class="captcha-footer">
        <button class="btn btn-secondary" @click="handleCancel">取消</button>
        <button class="btn btn-primary" @click="handleSubmit" :disabled="loading || !answer">确认</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.captcha-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 1rem;
}

.captcha-modal {
  background: white;
  border-radius: 12px;
  width: 100%;
  max-width: 320px;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
}

.captcha-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1rem 0.5rem;
  border-bottom: 1px solid #e5e7eb;
}

.captcha-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: #111827;
  margin: 0;
}

.captcha-close {
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: #6b7280;
  cursor: pointer;
  border-radius: 6px;
}

.captcha-close:hover {
  background: #f3f4f6;
  color: #111827;
}

.captcha-close svg {
  width: 1.25rem;
  height: 1.25rem;
}

.captcha-body {
  padding: 1rem;
}

.captcha-hint {
  font-size: 0.875rem;
  color: #6b7280;
  margin-bottom: 0.75rem;
  text-align: center;
}

.captcha-canvas-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.captcha-canvas {
  border: 1px solid #e5e7eb;
  border-radius: 8px;
}

.captcha-refresh {
  width: 2.5rem;
  height: 2.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f3f4f6;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  color: #6b7280;
  cursor: pointer;
}

.captcha-refresh:hover:not(:disabled) {
  background: #e5e7eb;
  color: #111827;
}

.captcha-refresh:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.captcha-refresh svg {
  width: 1.25rem;
  height: 1.25rem;
}

.captcha-refresh svg.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.captcha-error {
  font-size: 0.875rem;
  color: #dc2626;
  text-align: center;
  margin-bottom: 0.75rem;
}

.captcha-input {
  width: 100%;
  padding: 0.75rem 1rem;
  font-size: 1.125rem;
  text-align: center;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  outline: none;
  -webkit-appearance: none;
  appearance: none;
}

.captcha-input:focus {
  border-color: #2563eb;
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
}

.captcha-footer {
  display: flex;
  gap: 0.75rem;
  padding: 1rem;
  border-top: 1px solid #e5e7eb;
}

.captcha-footer .btn {
  flex: 1;
  padding: 0.75rem 1rem;
  font-size: 0.9375rem;
  font-weight: 500;
  border-radius: 8px;
  cursor: pointer;
}

.btn-secondary {
  background: #f3f4f6;
  border: 1px solid #d1d5db;
  color: #374151;
}

.btn-secondary:hover {
  background: #e5e7eb;
}

.btn-primary {
  background: #2563eb;
  border: 1px solid #2563eb;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #1d4ed8;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
