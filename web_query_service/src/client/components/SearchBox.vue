<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  loading: boolean
}>()

const emit = defineEmits<{
  search: [query: string]
}>()

const query = ref('')

function handleSubmit() {
  const value = query.value.trim()
  if (value) {
    emit('search', value)
  }
}
</script>

<template>
  <div class="search-box card">
    <h2 class="search-title">输入呼号查询</h2>
    <p class="search-desc">查询您的 QSL 卡片收取状态与物流信息</p>
    <form @submit.prevent="handleSubmit" class="search-form">
      <div class="input-wrapper">
        <svg class="search-icon" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" clip-rule="evenodd" />
        </svg>
        <input
          v-model="query"
          type="text"
          class="search-input"
          placeholder="请输入呼号，如 BG7XYZ"
          :disabled="loading"
          autocomplete="off"
          autocapitalize="characters"
          enterkeyhint="search"
        />
      </div>
      <button
        type="submit"
        class="btn btn-primary search-btn"
        :disabled="loading || !query.trim()"
      >
        <svg v-if="loading" class="spinner" viewBox="0 0 24 24">
          <circle class="spinner-track" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" fill="none" />
          <path class="spinner-path" d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" fill="none" stroke-linecap="round" />
        </svg>
        <span v-else>查询</span>
      </button>
    </form>
  </div>
</template>

<style scoped>
.search-box {
  padding: 1.25rem;
}

.search-title {
  font-size: 1.125rem;
  font-weight: 700;
  margin-bottom: 0.375rem;
  color: var(--text);
}

.search-desc {
  font-size: 0.875rem;
  color: var(--text-secondary);
  margin-bottom: 1rem;
  line-height: 1.5;
}

.search-form {
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
}

.input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 1rem;
  width: 1.25rem;
  height: 1.25rem;
  color: var(--text-secondary);
  pointer-events: none;
}

.search-input {
  width: 100%;
  min-height: 3rem;
  padding: 0.875rem 1rem 0.875rem 2.75rem;
  font-size: 1rem;
  border: 2px solid var(--border);
  border-radius: var(--radius);
  outline: none;
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
  -webkit-appearance: none;
  appearance: none;
}

.search-input:focus {
  border-color: var(--primary);
  box-shadow: 0 0 0 4px rgba(59, 130, 246, 0.15);
}

.search-input::placeholder {
  color: var(--text-secondary);
}

.search-input:disabled {
  background: var(--bg);
}

.search-btn {
  width: 100%;
  min-height: 3rem;
}

.spinner {
  width: 1.25rem;
  height: 1.25rem;
  animation: spin 1s linear infinite;
}

.spinner-track {
  opacity: 0.25;
}

.spinner-path {
  opacity: 0.75;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
