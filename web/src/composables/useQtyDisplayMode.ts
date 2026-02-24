import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export type QtyDisplayMode = 'exact' | 'approximate'

const STORAGE_KEY = 'qty_display_mode'
const DB_KEY = 'qty_display_mode'

// 全局响应式状态，所有组件共享
const qtyDisplayMode = ref<QtyDisplayMode>('exact')

// 标记是否已从后端加载过初始值
let initialized = false

/**
 * 从后端加载初始值，并执行 localStorage 迁移
 */
async function initFromBackend() {
  if (initialized) return
  initialized = true

  try {
    const dbValue = await invoke<string | null>('get_app_setting_cmd', { key: DB_KEY })

    if (dbValue) {
      // 数据库有值，使用数据库值
      qtyDisplayMode.value = dbValue as QtyDisplayMode
    } else {
      // 数据库无值，检查 localStorage 是否有旧值（迁移）
      const localValue = localStorage.getItem(STORAGE_KEY)
      if (localValue === 'exact' || localValue === 'approximate') {
        qtyDisplayMode.value = localValue
        // 写入数据库
        await invoke('set_app_setting_cmd', { key: DB_KEY, value: localValue })
        // 清除 localStorage
        localStorage.removeItem(STORAGE_KEY)
      }
      // 如果 localStorage 也没有值，保持默认 'exact'
    }
  } catch (e) {
    console.warn('从后端加载 qty_display_mode 失败，使用默认值', e)
  }

  // 初始化完成后，开始监听变化并持久化到后端
  watch(qtyDisplayMode, async (val) => {
    try {
      await invoke('set_app_setting_cmd', { key: DB_KEY, value: val })
    } catch (e) {
      console.warn('保存 qty_display_mode 失败', e)
    }
  })
}

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
  // 首次调用时触发后端初始化
  initFromBackend()

  return {
    qtyDisplayMode,
    formatQty
  }
}
