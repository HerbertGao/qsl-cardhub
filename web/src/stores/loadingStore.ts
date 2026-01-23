import { reactive, computed } from 'vue'

// Loading 状态接口
interface LoadingState {
  visible: boolean  // 是否显示
  text: string      // 显示文本
  count: number     // 引用计数（支持嵌套调用）
}

// 响应式状态
const state = reactive<LoadingState>({
  visible: false,
  text: '加载中...',
  count: 0
})

// 导出只读状态
export const loadingState = computed(() => ({
  visible: state.visible,
  text: state.text
}))

// 显示 loading
export function showLoading(text?: string): void {
  state.count++
  if (text) {
    state.text = text
  }
  state.visible = state.count > 0
}

// 隐藏 loading
export function hideLoading(): void {
  if (state.count > 0) {
    state.count--
  }
  state.visible = state.count > 0
  // 当完全关闭时重置文本
  if (state.count === 0) {
    state.text = '加载中...'
  }
}

// 设置 loading 文本
export function setLoadingText(text: string): void {
  state.text = text
}

// 重置状态（用于异常情况）
export function resetLoading(): void {
  state.count = 0
  state.visible = false
  state.text = '加载中...'
}
