import { ref, watch } from 'vue'

// 导航目标
export const navigationTarget = ref<string | null>(null)

// 导航到指定页面
export function navigateTo(target: string): void {
  navigationTarget.value = target
}

// 清除导航目标（导航完成后调用）
export function clearNavigationTarget(): void {
  navigationTarget.value = null
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
