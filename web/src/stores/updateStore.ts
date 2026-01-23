import { reactive } from 'vue'

// 更新信息接口
export interface UpdateInfo {
  version: string
  notes: string
  pubDate: string
  downloadUrl?: string
}

// 更新状态接口
export interface UpdateState {
  // 是否有可用更新
  hasUpdate: boolean
  // 更新信息
  updateInfo: UpdateInfo | null
  // 是否正在检查
  checking: boolean
  // 是否正在下载
  downloading: boolean
  // 下载进度 (0-100)
  downloadProgress: number
  // 错误信息
  error: string | null
  // 是否显示红点（用户是否已查看）
  showBadge: boolean
}

// 创建响应式状态
export const updateState = reactive<UpdateState>({
  hasUpdate: false,
  updateInfo: null,
  checking: false,
  downloading: false,
  downloadProgress: 0,
  error: null,
  showBadge: false,
})

// 设置有新更新
export function setUpdateAvailable(info: UpdateInfo): void {
  updateState.hasUpdate = true
  updateState.updateInfo = info
  updateState.showBadge = true
  updateState.error = null
}

// 清除更新状态
export function clearUpdate(): void {
  updateState.hasUpdate = false
  updateState.updateInfo = null
  updateState.showBadge = false
  updateState.error = null
}

// 标记已查看（清除红点）
export function markAsViewed(): void {
  updateState.showBadge = false
}

// 设置检查状态
export function setChecking(checking: boolean): void {
  updateState.checking = checking
  if (checking) {
    updateState.error = null
  }
}

// 设置下载状态
export function setDownloading(downloading: boolean): void {
  updateState.downloading = downloading
  if (downloading) {
    updateState.downloadProgress = 0
  }
}

// 设置下载进度
export function setDownloadProgress(progress: number): void {
  updateState.downloadProgress = Math.min(100, Math.max(0, progress))
}

// 设置错误
export function setError(error: string | null): void {
  updateState.error = error
  updateState.checking = false
  updateState.downloading = false
}
