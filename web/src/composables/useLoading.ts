import { computed } from 'vue'
import {
  loadingState,
  showLoading as storeShowLoading,
  hideLoading as storeHideLoading
} from '@/stores/loadingStore'

/**
 * Loading 状态管理 composable
 * 提供全局 loading 的显示、隐藏和包装函数
 */
export function useLoading() {
  // 计算属性：当前是否正在 loading
  const isLoading = computed(() => loadingState.value.visible)

  // 显示 loading
  function showLoading(text?: string): void {
    storeShowLoading(text)
  }

  // 隐藏 loading
  function hideLoading(): void {
    storeHideLoading()
  }

  /**
   * 包装异步函数，自动管理 loading 状态
   * @param fn 异步函数
   * @param text 可选的 loading 文本
   * @returns 异步函数的返回值
   */
  async function withLoading<T>(
    fn: () => Promise<T>,
    text?: string
  ): Promise<T> {
    showLoading(text)
    try {
      return await fn()
    } finally {
      hideLoading()
    }
  }

  return {
    isLoading,
    showLoading,
    hideLoading,
    withLoading
  }
}
