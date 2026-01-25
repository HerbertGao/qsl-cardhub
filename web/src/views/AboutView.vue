<template>
  <div class="page-content about-page">
    <div class="about-layout">
      <!-- 左侧：应用信息 -->
      <div class="about-main">
        <h1>关于 QSL 分卡助手</h1>

        <el-card
          style="margin-top: 30px"
          shadow="hover"
        >
          <el-descriptions
            :column="1"
            border
          >
            <el-descriptions-item label="应用名称">
              QSL 分卡助手
            </el-descriptions-item>
            <el-descriptions-item label="版本">
              <div style="display: flex; align-items: center; gap: 12px">
                <span>v{{ appVersion }}</span>
                <el-button
                  type="primary"
                  size="small"
                  :loading="updateState.checking"
                  @click="handleCheckUpdate"
                >
                  {{ updateState.checking ? '检查中...' : '检查更新' }}
                </el-button>
                <el-tag
                  v-if="updateState.hasUpdate"
                  type="success"
                  size="small"
                >
                  有新版本
                </el-tag>
              </div>
            </el-descriptions-item>
            <el-descriptions-item label="描述">
              业余无线电卡片打印系统
            </el-descriptions-item>
            <el-descriptions-item label="技术栈">
              Rust + Tauri 2 + Vue 3 + Element Plus
            </el-descriptions-item>
            <el-descriptions-item label="平台支持">
              Windows / macOS
            </el-descriptions-item>
            <el-descriptions-item label="版权">
              © 2026 Herbert Software
            </el-descriptions-item>
          </el-descriptions>
        </el-card>

        <!-- 更新提示卡片 -->
        <el-card
          v-if="updateState.hasUpdate && updateState.updateInfo"
          style="margin-top: 20px"
          shadow="hover"
        >
          <template #header>
            <div style="display: flex; align-items: center; justify-content: space-between">
              <span>发现新版本</span>
              <el-tag type="success">
                v{{ updateState.updateInfo.version }}
              </el-tag>
            </div>
          </template>

          <div style="margin-bottom: 16px">
            <div style="color: #909399; font-size: 13px; margin-bottom: 8px">
              发布日期：{{ formatDate(updateState.updateInfo.pubDate) }}
            </div>
            <div
              v-if="updateState.updateInfo.notes"
              style="white-space: pre-wrap; line-height: 1.6"
            >
              {{ updateState.updateInfo.notes }}
            </div>
          </div>

          <!-- 下载进度 -->
          <div
            v-if="updateState.downloading"
            style="margin-bottom: 16px"
          >
            <el-progress
              :percentage="updateState.downloadProgress"
              :status="updateState.downloadProgress === 100 ? 'success' : undefined"
            />
            <div style="color: #909399; font-size: 12px; margin-top: 4px">
              正在下载更新...
            </div>
          </div>

          <div style="display: flex; gap: 12px">
            <el-button
              type="primary"
              :loading="updateState.downloading"
              :disabled="updateState.downloadProgress === 100"
              @click="handleDownloadUpdate"
            >
              {{ updateState.downloading ? '下载中...' : (updateState.downloadProgress === 100 ? '下载完成' : '下载更新') }}
            </el-button>
            <el-button
              v-if="updateState.downloadProgress === 100"
              type="success"
              @click="handleInstallUpdate"
            >
              立即安装
            </el-button>
            <el-button @click="handleOpenReleasePage">
              查看发布页
            </el-button>
          </div>
        </el-card>

        <!-- 错误提示 -->
        <el-alert
          v-if="updateState.error"
          :title="updateState.error"
          type="error"
          style="margin-top: 20px"
          show-icon
          closable
          @close="updateState.error = null"
        />

        <!-- 已是最新版本提示 -->
        <el-alert
          v-if="showLatestMessage"
          title="已是最新版本"
          type="success"
          style="margin-top: 20px"
          show-icon
          closable
          @close="showLatestMessage = false"
        />

        <!-- 危险操作区域 -->
        <el-card
          style="margin-top: 30px"
          shadow="hover"
        >
          <template #header>
            <span style="color: #f56c6c">危险操作</span>
          </template>
          <div style="margin-bottom: 12px; color: #909399; font-size: 13px">
            此操作将删除所有数据（项目、卡片、配置、登录凭据等），且无法恢复。默认打印模板将被保留。
          </div>
          <el-button
            type="danger"
            :loading="resetting"
            @click="handleFactoryReset"
          >
            抹掉所有内容和设置
          </el-button>
        </el-card>
      </div>

      <!-- 右侧：赞助信息 -->
      <div class="about-sidebar">
        <el-card shadow="hover">
          <template #header>
            <div class="sponsor-header">
              <el-icon><CoffeeCup /></el-icon>
              <span>赞助支持</span>
            </div>
          </template>
          <div class="sponsor-content">
            <p class="sponsor-desc">
              如果这个项目对您有帮助，欢迎请作者喝杯咖啡
            </p>
            <div class="qrcode-container">
              <div class="qrcode-item">
                <el-image
                  :src="alipayQR"
                  :preview-src-list="[alipayQR, wechatQR]"
                  :initial-index="0"
                  fit="contain"
                  class="qrcode-image"
                  preview-teleported
                />
              </div>
              <div class="qrcode-item">
                <el-image
                  :src="wechatQR"
                  :preview-src-list="[alipayQR, wechatQR]"
                  :initial-index="1"
                  fit="contain"
                  class="qrcode-image"
                  preview-teleported
                />
              </div>
            </div>
          </div>
        </el-card>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { ElMessage, ElMessageBox } from 'element-plus'
import { CoffeeCup } from '@element-plus/icons-vue'
import alipayQR from '@/assets/alipay.jpg'
import wechatQR from '@/assets/wechat.jpg'
import {
  updateState,
  markAsViewed,
  setChecking,
  setUpdateAvailable,
  setError,
  setDownloading,
  setDownloadProgress,
  type UpdateInfo
} from '@/stores/updateStore'

// 保存更新对象引用
let pendingUpdate: Awaited<ReturnType<typeof check>> | null = null

// 应用版本号
const appVersion = ref('0.0.0')

// 显示"已是最新版本"提示
const showLatestMessage = ref(false)

// 恢复出厂设置状态
const resetting = ref(false)

// GitHub 仓库信息
const GITHUB_OWNER = 'HerbertGao'
const GITHUB_REPO = 'QSL-CardHub'

// 获取应用版本号
onMounted(async () => {
  try {
    appVersion.value = await getVersion()
  } catch (error) {
    console.error('获取版本号失败:', error)
    appVersion.value = '未知'
  }

  // 标记已查看，清除菜单红点
  markAsViewed()
})

// 比较版本号，返回 1 表示 v1 > v2，-1 表示 v1 < v2，0 表示相等
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

// 格式化日期
function formatDate(dateStr: string): string {
  try {
    const date = new Date(dateStr)
    return date.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    })
  } catch {
    return dateStr
  }
}

// 检查更新
async function handleCheckUpdate(): Promise<void> {
  setChecking(true)
  showLatestMessage.value = false

  try {
    // 首先尝试使用 Tauri Updater 插件检查
    const update = await check()

    if (update) {
      pendingUpdate = update
      const updateInfo: UpdateInfo = {
        version: update.version,
        notes: update.body || '无更新说明',
        pubDate: update.date || new Date().toISOString(),
        downloadUrl: `https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/releases/tag/v${update.version}`
      }
      setUpdateAvailable(updateInfo)
      ElMessage.success('发现新版本！')
    } else {
      showLatestMessage.value = true
      ElMessage.info('已是最新版本')
    }
  } catch (error) {
    console.error('Tauri Updater 检查失败，尝试 GitHub API:', error)

    // 回退到 GitHub API
    try {
      const response = await fetch(
        `https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPO}/releases/latest`,
        {
          headers: {
            'Accept': 'application/vnd.github.v3+json'
          }
        }
      )

      if (!response.ok) {
        if (response.status === 404) {
          throw new Error('未找到发布版本')
        }
        throw new Error(`请求失败: ${response.status}`)
      }

      const release = await response.json()
      const latestVersion = release.tag_name.replace(/^v/, '')
      const currentVersion = appVersion.value

      if (compareVersions(latestVersion, currentVersion) > 0) {
        const updateInfo: UpdateInfo = {
          version: latestVersion,
          notes: release.body || '无更新说明',
          pubDate: release.published_at,
          downloadUrl: release.html_url
        }
        setUpdateAvailable(updateInfo)
        ElMessage.success('发现新版本！')
      } else {
        showLatestMessage.value = true
        ElMessage.info('已是最新版本')
      }
    } catch (fallbackError) {
      console.error('GitHub API 检查也失败:', fallbackError)
      setError(`检查更新失败: ${fallbackError instanceof Error ? fallbackError.message : '未知错误'}`)
      ElMessage.error('检查更新失败，请稍后重试')
    }
  } finally {
    setChecking(false)
  }
}

// 下载并安装更新
async function handleDownloadUpdate(): Promise<void> {
  if (!pendingUpdate) {
    // 如果没有 Tauri Updater 的更新对象，打开发布页面
    await handleOpenReleasePage()
    ElMessage.info('请下载对应平台的安装包进行安装')
    return
  }

  setDownloading(true)

  try {
    let downloaded = 0
    let contentLength = 0

    await pendingUpdate.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          contentLength = event.data.contentLength || 0
          console.log(`开始下载，总大小: ${contentLength} bytes`)
          break
        case 'Progress':
          downloaded += event.data.chunkLength
          if (contentLength > 0) {
            const progress = Math.round((downloaded / contentLength) * 100)
            setDownloadProgress(progress)
          }
          break
        case 'Finished':
          setDownloadProgress(100)
          console.log('下载完成')
          break
      }
    })

    ElMessage.success('下载完成，即将重启安装...')

    // 短暂延迟后重启
    setTimeout(async () => {
      await relaunch()
    }, 1500)
  } catch (error) {
    console.error('下载更新失败:', error)
    setError(`下载失败: ${error instanceof Error ? error.message : '未知错误'}`)
    ElMessage.error('下载失败，请稍后重试或手动下载')
  } finally {
    setDownloading(false)
  }
}

// 安装更新
async function handleInstallUpdate(): Promise<void> {
  if (pendingUpdate) {
    // 如果有 Tauri Updater 的更新，重启应用完成安装
    try {
      await relaunch()
    } catch (error) {
      console.error('重启失败:', error)
      ElMessage.error('重启失败，请手动重启应用')
    }
  } else {
    // 否则打开发布页面
    await handleOpenReleasePage()
    ElMessage.info('请下载对应平台的安装包进行安装')
  }
}

// 打开发布页面
async function handleOpenReleasePage(): Promise<void> {
  try {
    const url = updateState.updateInfo?.downloadUrl ||
      `https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/releases/latest`
    await open(url)
  } catch (error) {
    console.error('打开链接失败:', error)
    ElMessage.error('打开链接失败')
  }
}

// 抹掉所有内容和设置
async function handleFactoryReset(): Promise<void> {
  try {
    await ElMessageBox.confirm(
      '此操作将删除以下数据，且无法恢复：<br><br>' +
      '• 所有项目和卡片数据<br>' +
      '• 打印机配置<br>' +
      '• QRZ.cn / QRZ.com 登录凭据<br>' +
      '• 顺丰速运配置<br>' +
      '• 云同步配置<br><br>' +
      '默认打印模板将被保留。<br><br>' +
      '确定要继续吗？',
      '抹掉所有内容和设置',
      {
        confirmButtonText: '确认重置',
        cancelButtonText: '取消',
        type: 'warning',
        dangerouslyUseHTMLString: true,
      }
    )

    resetting.value = true
    await invoke('factory_reset')
    ElMessage.success('重置完成，即将重启应用...')

    setTimeout(async () => {
      await relaunch()
    }, 1500)
  } catch (error) {
    if (error !== 'cancel') {
      console.error('重置失败:', error)
      ElMessage.error(`重置失败: ${error}`)
    }
  } finally {
    resetting.value = false
  }
}
</script>

<style scoped>
.about-page {
  max-width: 1200px;
}

.about-layout {
  display: flex;
  gap: 30px;
  align-items: flex-start;
}

.about-main {
  flex: 1;
  min-width: 0;
}

.about-main h1 {
  margin-top: 0;
}

.about-sidebar {
  width: 280px;
  flex-shrink: 0;
  position: sticky;
  top: 20px;
}

.sponsor-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
}

.sponsor-content {
  text-align: center;
}

.sponsor-desc {
  color: #606266;
  font-size: 14px;
  margin: 0 0 16px 0;
  line-height: 1.5;
}

.qrcode-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.qrcode-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.qrcode-image {
  width: 200px;
  height: auto;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

/* 响应式布局 */
@media (max-width: 900px) {
  .about-layout {
    flex-direction: column;
  }

  .about-sidebar {
    width: 100%;
    max-width: 400px;
    position: static;
  }

  .qrcode-container {
    flex-direction: row;
    justify-content: center;
  }

  .qrcode-image {
    width: 150px;
  }
}
</style>
