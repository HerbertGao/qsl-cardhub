import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
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

/**
 * 获取当前平台对应的 GitHub Release 资产文件名匹配关键词。
 * 返回 { platform, extension } 或 null（无法识别平台时）。
 */
async function getPlatformAssetKeyword(): Promise<{ platform: string; extension: string } | null> {
  try {
    const info = await invoke<{ os: string; arch: string }>('get_platform_info')
    const os = info.os.toLowerCase()
    const arch = info.arch.toLowerCase()

    if (os === 'macos' || os === 'darwin') {
      if (arch === 'arm64' || arch === 'aarch64') return { platform: 'macos-arm64', extension: '.dmg' }
      if (arch === 'x86_64') return { platform: 'macos-x64', extension: '.dmg' }
    } else if (os === 'windows') {
      if (arch === 'x86_64') return { platform: 'windows-x64', extension: '-setup.exe' }
      if (arch === 'arm64' || arch === 'aarch64') return { platform: 'windows-arm64', extension: '-setup.exe' }
    }
    return null
  } catch {
    return null
  }
}

/**
 * 检查 GitHub Release 的 assets 中是否包含当前平台的安装包。
 */
function hasAssetForPlatform(
  assets: Array<{ name: string }>,
  platformKeyword: string,
  extensionKeyword: string
): boolean {
  return assets.some(
    (asset) => asset.name.includes(platformKeyword) && asset.name.endsWith(extensionKeyword)
  )
}

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
      // 验证当前平台的安装包是否已上传（避免 CI/CD 构建期间误报）
      const keyword = await getPlatformAssetKeyword()
      const assets = release.assets || []
      if (keyword && !hasAssetForPlatform(assets, keyword.platform, keyword.extension)) {
        console.warn(`新版本 ${latestVersion} 已发布，但当前平台 (${keyword.platform}) 的安装包尚未就绪`)
        return
      }
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
        // 验证当前平台的安装包是否已上传（避免 CI/CD 构建期间误报）
        const keyword = await getPlatformAssetKeyword()
        const assets = release.assets || []
        if (keyword && !hasAssetForPlatform(assets, keyword.platform, keyword.extension)) {
          console.warn(`新版本 ${latestVersion} 已发布，但当前平台 (${keyword.platform}) 的安装包尚未就绪`)
          return
        }
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
