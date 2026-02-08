import { ref, watch } from 'vue'

// 导航目标
export const navigationTarget = ref<string | null>(null)

// 导航参数
export const navigationParams = ref<Record<string, string>>({})

// 导航到指定页面
export function navigateTo(target: string, params?: Record<string, string>): void {
  navigationParams.value = params ?? {}
  navigationTarget.value = target
}

// 清除导航目标（导航完成后调用，不清除 params，由目标页面通过 consumeNavigationParams 消费）
export function clearNavigationTarget(): void {
  navigationTarget.value = null
}

// 消费导航参数（读取后清除）
export function consumeNavigationParams(): Record<string, string> {
  const params = { ...navigationParams.value }
  navigationParams.value = {}
  return params
}

// 监听导航目标变化的工具函数
export function useNavigationWatcher(
  currentMenu: { value: string },
  onNavigate?: (target: string) => void
): void {
  watch(navigationTarget, (target) => {
    if (target) {
      currentMenu.value = target
      if (onNavigate) {
        onNavigate(target)
      }
      clearNavigationTarget()
    }
  })
}
