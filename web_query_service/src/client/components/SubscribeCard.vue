<script setup lang="ts">
defineProps<{
  callsign: string
}>()

const emit = defineEmits<{
  subscribe: []
}>()
</script>

<template>
  <div class="subscribe-card card">
    <div class="subscribe-header">
      <div class="subscribe-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
        </svg>
      </div>
      <div class="subscribe-content">
        <h3 class="subscribe-title">订阅收卡推送</h3>
        <p class="subscribe-desc">
          订阅后，呼号 <strong>{{ callsign }}</strong> 的 QSL 卡片分发与物流动态将自动推送至您的微信。
        </p>
      </div>
    </div>
    <button class="btn btn-subscribe" @click="emit('subscribe')">
      <svg class="wechat-icon" viewBox="0 0 24 24" fill="currentColor">
        <path d="M8.691 2.188C3.891 2.188 0 5.476 0 9.53c0 2.212 1.17 4.203 3.002 5.55a.59.59 0 01.213.665l-.39 1.48c-.019.07-.048.141-.048.213 0 .163.13.295.29.295a.326.326 0 00.167-.054l1.903-1.114a.864.864 0 01.717-.098 10.16 10.16 0 002.837.403c.276 0 .543-.027.811-.05-.857-2.578.157-4.972 1.932-6.446 1.703-1.415 3.882-1.98 5.853-1.838-.576-3.583-4.196-6.348-8.596-6.348zM5.785 5.991c.642 0 1.162.529 1.162 1.18a1.17 1.17 0 01-1.162 1.178A1.17 1.17 0 014.623 7.17c0-.651.52-1.18 1.162-1.18zm5.813 0c.642 0 1.162.529 1.162 1.18a1.17 1.17 0 01-1.162 1.178 1.17 1.17 0 01-1.162-1.178c0-.651.52-1.18 1.162-1.18zm5.34 2.867c-1.797-.052-3.746.512-5.28 1.786-1.72 1.428-2.687 3.72-1.78 6.22.942 2.453 3.666 4.229 6.884 4.229.826 0 1.622-.12 2.361-.336a.722.722 0 01.598.082l1.584.926a.272.272 0 00.139.045c.133 0 .241-.108.241-.245 0-.06-.023-.118-.038-.177l-.327-1.233a.582.582 0 01-.023-.156.49.49 0 01.201-.398C23.024 18.48 24 16.82 24 14.98c0-3.21-2.931-5.837-6.656-6.088V8.89c-.135-.01-.27-.027-.406-.032zm-2.53 3.274c.535 0 .969.44.969.982a.976.976 0 01-.969.983.976.976 0 01-.969-.983c0-.542.434-.982.97-.982zm4.844 0c.535 0 .969.44.969.982a.976.976 0 01-.969.983.976.976 0 01-.969-.983c0-.542.434-.982.969-.982z"/>
      </svg>
      订阅收卡
    </button>
  </div>
</template>

<style scoped>
.subscribe-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1.25rem;
  margin-top: 1.5rem;
  background: linear-gradient(135deg, #f0fdf4 0%, #ecfdf5 100%);
  border: 1px solid #bbf7d0;
}

.subscribe-header {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
}

.subscribe-icon {
  flex-shrink: 0;
  width: 2.75rem;
  height: 2.75rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #10b981;
  border-radius: 50%;
  color: white;
}

.subscribe-icon svg {
  width: 1.375rem;
  height: 1.375rem;
}

.subscribe-content {
  flex: 1;
  min-width: 0;
}

.subscribe-title {
  font-size: 1rem;
  font-weight: 700;
  margin-bottom: 0.375rem;
  color: var(--text);
}

.subscribe-desc {
  font-size: 0.875rem;
  color: var(--text-secondary);
  line-height: 1.5;
}

.btn-subscribe {
  width: 100%;
  min-height: 3rem;
  background: #10b981;
  color: white;
  border: none;
  font-weight: 600;
}

.btn-subscribe:active {
  background: #059669;
}

.wechat-icon {
  width: 1.125rem;
  height: 1.125rem;
}
</style>
