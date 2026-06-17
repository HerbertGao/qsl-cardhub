// 首启模式 onboarding 标志
//
// device-local（localStorage、不随云端同步——每台设备独立 onboarding）；
// 语义 = 「已做过一次模式选择」（点网关任一按钮即置位），**非**「已配置云端」。
// 不存所选模式（模式由 api_url 派生）；永不主动清（单调 done 标志）。

const KEY = 'qsl:onboarding:mode-selected'

export function isModeSelected(): boolean {
  return localStorage.getItem(KEY) === '1'
}

export function markModeSelected(): void {
  localStorage.setItem(KEY, '1')
}
