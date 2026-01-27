import { ref, watch } from 'vue'

export type QtyDisplayMode = 'exact' | 'approximate'

const STORAGE_KEY = 'qty_display_mode'

// 全局响应式状态，所有组件共享
const qtyDisplayMode = ref<QtyDisplayMode>(
  (localStorage.getItem(STORAGE_KEY) as QtyDisplayMode) || 'exact'
)

// 持久化到 localStorage
watch(qtyDisplayMode, (val) => {
  localStorage.setItem(STORAGE_KEY, val)
})

/**
 * 根据当前模式格式化数量
 */
function formatQty(qty: number | null | undefined): string {
  if (qty === null || qty === undefined) return '-'
  if (qtyDisplayMode.value === 'exact') return String(qty)
  if (qty <= 10) return '≤10'
  if (qty <= 50) return '≤50'
  return '>50'
}

export function useQtyDisplayMode() {
  return {
    qtyDisplayMode,
    formatQty
  }
}
