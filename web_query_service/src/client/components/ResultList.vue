<script setup lang="ts">
import { ref } from 'vue'

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

defineProps<{
  callsign: string
  items: CardItem[]
}>()

const copiedId = ref<string | null>(null)

function getStatusLabel(status: string): string {
  const map: Record<string, string> = {
    pending: '待处理',
    printed: '已打印',
    distributed: '已分发',
    completed: '已完成',
  }
  return map[status] || status
}

function getStatusClass(status: string): string {
  if (status === 'completed' || status === 'distributed') return 'badge-success'
  if (status === 'printed') return 'badge-warning'
  return 'badge-pending'
}

async function copyRemarks(id: string, text: string) {
  try {
    await navigator.clipboard.writeText(text)
    copiedId.value = id
    setTimeout(() => {
      copiedId.value = null
    }, 2000)
  } catch {
    // 降级方案
    const textarea = document.createElement('textarea')
    textarea.value = text
    document.body.appendChild(textarea)
    textarea.select()
    document.execCommand('copy')
    document.body.removeChild(textarea)
    copiedId.value = id
    setTimeout(() => {
      copiedId.value = null
    }, 2000)
  }
}
</script>

<template>
  <div class="result-list">
    <div class="result-header">
      <h2 class="result-title">
        <span class="callsign-badge">{{ callsign }}</span>
        <span class="title-text">的收卡记录</span>
      </h2>
      <span class="result-count">共 {{ items.length }} 条</span>
    </div>

    <div v-if="items.length === 0" class="empty-state">
      <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <path stroke-linecap="round" stroke-linejoin="round" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
      </svg>
      <p>该呼号暂无收卡记录</p>
    </div>

    <ul v-else class="card-list">
      <li v-for="item in items" :key="item.id" class="card-item card">
        <div class="card-header">
          <span class="project-name">{{ item.project_name || '未知项目' }}</span>
          <span :class="['badge', getStatusClass(item.status)]">
            {{ getStatusLabel(item.status) }}
          </span>
        </div>
        <div
          v-if="
            item.distribution?.remarks ||
            (item.status === 'distributed' && item.distribution?.method)
          "
          class="card-body"
        >
          <div
            v-if="item.status === 'distributed' && item.distribution?.method"
            class="distribution-row"
          >
            <span class="method-tag">{{ item.distribution.method }}</span>
            <div class="distribution-detail">
              <p
                v-if="
                  item.distribution?.method === '代领' &&
                  item.distribution?.proxy_callsign
                "
                class="detail-text"
              >
                代领人：{{ item.distribution.proxy_callsign }}
              </p>
              <div v-else-if="item.distribution?.remarks" class="remarks-row">
                <p class="remarks">{{ item.distribution.remarks }}</p>
                <button
                  class="copy-btn"
                  :class="{ copied: copiedId === item.id }"
                  @click="copyRemarks(item.id, item.distribution.remarks!)"
                  :title="copiedId === item.id ? '已复制' : '复制备注'"
                >
                  <svg v-if="copiedId === item.id" class="copy-icon" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
                  </svg>
                  <svg v-else class="copy-icon" viewBox="0 0 20 20" fill="currentColor">
                    <path d="M8 3a1 1 0 011-1h2a1 1 0 110 2H9a1 1 0 01-1-1z" />
                    <path d="M6 3a2 2 0 00-2 2v11a2 2 0 002 2h8a2 2 0 002-2V5a2 2 0 00-2-2 3 3 0 01-3 3H9a3 3 0 01-3-3z" />
                  </svg>
                </button>
              </div>
            </div>
          </div>
          <div
            v-else-if="item.distribution?.remarks"
            class="remarks-row"
          >
            <p class="remarks">{{ item.distribution.remarks }}</p>
            <button
              class="copy-btn"
              :class="{ copied: copiedId === item.id }"
              @click="copyRemarks(item.id, item.distribution.remarks!)"
              :title="copiedId === item.id ? '已复制' : '复制备注'"
            >
              <svg v-if="copiedId === item.id" class="copy-icon" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
              </svg>
              <svg v-else class="copy-icon" viewBox="0 0 20 20" fill="currentColor">
                <path d="M8 3a1 1 0 011-1h2a1 1 0 110 2H9a1 1 0 01-1-1z" />
                <path d="M6 3a2 2 0 00-2 2v11a2 2 0 002 2h8a2 2 0 002-2V5a2 2 0 00-2-2 3 3 0 01-3 3H9a3 3 0 01-3-3z" />
              </svg>
            </button>
          </div>
        </div>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.result-list {
  margin-top: 1.5rem;
}

.result-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem;
  gap: 0.5rem;
}

.result-title {
  font-size: 1rem;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.callsign-badge {
  background: var(--primary);
  color: white;
  padding: 0.25rem 0.625rem;
  border-radius: 6px;
  font-family: 'SF Mono', 'Menlo', monospace;
  font-size: 0.9375rem;
  font-weight: 600;
}

.title-text {
  white-space: nowrap;
}

.result-count {
  font-size: 0.875rem;
  color: var(--text-secondary);
  white-space: nowrap;
}

.empty-state {
  text-align: center;
  padding: 3rem 1rem;
  color: var(--text-secondary);
}

.empty-icon {
  width: 3.5rem;
  height: 3.5rem;
  margin-bottom: 1rem;
  opacity: 0.5;
}

.card-list {
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 0.875rem;
}

.card-item {
  padding: 1rem;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.project-name {
  font-weight: 600;
  font-size: 1rem;
  word-break: break-word;
}

.card-body {
  margin-top: 0.75rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--border);
}

.distribution-row {
  margin: 0 0 0.625rem;
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
}

.method-tag {
  display: inline-flex;
  align-items: center;
  min-height: 1.5rem;
  padding: 0.125rem 0.5rem;
  border-radius: 9999px;
  font-size: 0.8125rem;
  font-weight: 600;
  color: #0f766e;
  background: #ccfbf1;
}

.distribution-detail {
  margin-left: auto;
  display: flex;
  justify-content: flex-end;
}

.detail-text {
  margin: 0;
  text-align: right;
  font-family: 'SF Mono', 'Menlo', monospace;
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--text-secondary);
  padding: 0.125rem 0.5rem;
  border-radius: 6px;
  background: var(--bg);
}

.remarks-row {
  display: flex;
  align-items: flex-start;
  justify-content: flex-end;
  gap: 0.5rem;
  padding: 0.5rem 0.625rem;
  border-radius: var(--radius-sm);
  background: var(--bg);
  max-width: 100%;
}

.remarks {
  flex: 1;
  font-size: 0.875rem;
  color: var(--text-secondary);
  line-height: 1.5;
  margin: 0;
}

.copy-btn {
  flex-shrink: 0;
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--card-bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.15s ease;
}

.copy-btn:hover {
  background: var(--border);
  color: var(--text);
}

.copy-btn.copied {
  background: #dcfce7;
  border-color: #86efac;
  color: #16a34a;
}

.copy-icon {
  width: 1rem;
  height: 1rem;
}
</style>
