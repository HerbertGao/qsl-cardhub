import { getVersion } from '@tauri-apps/api/app'
import { check } from '@tauri-apps/plugin-updater'
import { ElMessage } from 'element-plus'
import {
  setUpdateAvailable,
  setChecking,
  setError,
  setPendingTauriUpdate,
  type UpdateInfo
} from '@/stores/updateStore'

const GITHUB_OWNER = 'HerbertGao'
const GITHUB_REPO = 'QSL-CardHub'

function compareVersions(v1: string, v2: string): number {
  const parts1 = v1.replace(/^v/, '').split('.').map(Number)
  const parts2 = v2.replace(/^v/, '').split('.').map(Number)
  for (let i = 0; i < Math.max(parts1.length, parts2.length); i++) {
    const num1 = parts1[i] || 0
    const num2 = parts2[i] || 0
    if (num1 > num2) return 1
    if (num1 < num2) return -1
  }
  return 0
}

/**
 * 检查更新（与关于页「检查更新」按钮逻辑一致：先 Tauri Updater，失败再 GitHub API）。
 * 自动检查与手动检查共用此逻辑，保证关于页展示与「下载更新」行为一致。
 */
export async function checkForUpdate(options: { silent: boolean }): Promise<void> {
  const { silent } = options
  if (!silent) {
    setChecking(true)
  }

  try {
    const update = await check()
    if (update) {
      setPendingTauriUpdate(update)
      const updateInfo: UpdateInfo = {
        version: update.version,
        notes: update.body || '无更新说明',
        pubDate: update.date || new Date().toISOString(),
        downloadUrl: `https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/releases/tag/v${update.version}`
      }
      setUpdateAvailable(updateInfo)
      if (!silent) {
        ElMessage.success('发现新版本！')
      }
      return
    }

    setPendingTauriUpdate(null)
    // 无 Tauri 更新，走 GitHub API 判断是否已是最新
    const currentVersion = await getVersion()
    const response = await fetch(
      `https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPO}/releases/latest`,
      {
        headers: { Accept: 'application/vnd.github.v3+json' }
      }
    )
    if (!response.ok) {
      if (!silent) {
        setError(`请求失败: ${response.status}`)
        ElMessage.error('检查更新失败，请稍后重试')
      }
      return
    }
    const release = await response.json()
    const latestVersion = release.tag_name.replace(/^v/, '')
    if (compareVersions(latestVersion, currentVersion) > 0) {
      setPendingTauriUpdate(null)
      const updateInfo: UpdateInfo = {
        version: latestVersion,
        notes: release.body || '无更新说明',
        pubDate: release.published_at,
        downloadUrl: release.html_url
      }
      setUpdateAvailable(updateInfo)
      if (!silent) {
        ElMessage.success('发现新版本！')
      }
    }
    // 已是最新：不设置 hasUpdate，不弹窗（静默）；手动检查时由 AboutView 显示「已是最新版本」
  } catch (err) {
    console.error('Tauri Updater 检查失败，尝试 GitHub API:', err)
    setPendingTauriUpdate(null)
    try {
      const currentVersion = await getVersion()
      const response = await fetch(
        `https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPO}/releases/latest`,
        { headers: { Accept: 'application/vnd.github.v3+json' } }
      )
      if (!response.ok) {
        if (response.status === 404) {
          throw new Error('未找到发布版本')
        }
        throw new Error(`请求失败: ${response.status}`)
      }
      const release = await response.json()
      const latestVersion = release.tag_name.replace(/^v/, '')
      if (compareVersions(latestVersion, currentVersion) > 0) {
        const updateInfo: UpdateInfo = {
          version: latestVersion,
          notes: release.body || '无更新说明',
          pubDate: release.published_at,
          downloadUrl: release.html_url
        }
        setUpdateAvailable(updateInfo)
        if (!silent) {
          ElMessage.success('发现新版本！')
        }
      }
    } catch (fallbackErr) {
      console.error('GitHub API 检查也失败:', fallbackErr)
      const msg = fallbackErr instanceof Error ? fallbackErr.message : '未知错误'
      setError(`检查更新失败: ${msg}`)
      if (!silent) {
        ElMessage.error('检查更新失败，请稍后重试')
      }
    }
  } finally {
    if (!silent) {
      setChecking(false)
    }
  }
}
