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
              v-if="updateState.updateInfo.notes && updateState.updateInfo.notes !== '无更新说明'"
              class="release-notes"
              @click="handleNotesLinkClick"
              v-html="renderedNotes"
            />
            <div v-else-if="updateState.updateInfo.notes">
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
import { computed, onMounted, ref } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'
import { relaunch } from '@tauri-apps/plugin-process'
import { ElMessage, ElMessageBox } from 'element-plus'
import { CoffeeCup } from '@element-plus/icons-vue'
import alipayQR from '@/assets/alipay.jpg'
import wechatQR from '@/assets/wechat.jpg'
import {
  updateState,
  markAsViewed,
  clearUpdate,
  setError,
  setDownloading,
  setDownloadProgress,
  getPendingTauriUpdate
} from '@/stores/updateStore'
import { checkForUpdate } from '@/services/updateCheck'
import { renderMarkdown } from '@/utils/markdown'

// 应用版本号
const appVersion = ref('0.0.0')

// 恢复出厂设置状态
const resetting = ref(false)

// GitHub 仓库信息
const GITHUB_OWNER = 'HerbertGao'
const GITHUB_REPO = 'QSL-CardHub'

// 渲染更新说明 Markdown
const renderedNotes = computed(() => {
  const notes = updateState.updateInfo?.notes
  if (!notes || notes === '无更新说明') return ''
  return renderMarkdown(notes)
})

// 拦截更新说明中的链接点击，使用外部浏览器打开
function handleNotesLinkClick(event: MouseEvent): void {
  const target = event.target as HTMLElement
  const anchor = target.closest('a')
  if (anchor?.href) {
    event.preventDefault()
    open(anchor.href)
  }
}

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

// 检查更新（与启动时自动检查共用同一套逻辑，保证关于页展示与下载更新行为一致）
async function handleCheckUpdate(): Promise<void> {
  await checkForUpdate({ silent: false })
  if (!updateState.hasUpdate && !updateState.error) {
    ElMessage.success('已是最新版本')
  }
}

// 下载并安装更新
async function handleDownloadUpdate(): Promise<void> {
  const pendingUpdate = getPendingTauriUpdate()
  if (!pendingUpdate) {
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
    const msg = error instanceof Error ? error.message : String(error)
    // 404 或网络超时类错误说明安装包尚未就绪（CI/CD 构建中）
    if (msg.includes('404') || msg.includes('Not Found') || msg.includes('timeout') || msg.includes('network')) {
      console.warn('更新下载失败，安装包可能尚未就绪:', msg)
      clearUpdate()
      ElMessage.warning('更新暂时不可用，请稍后重试')
    } else {
      console.error('下载更新失败:', error)
      setError(`下载失败: ${msg}`)
      ElMessage.error('下载失败，请稍后重试或手动下载')
    }
  } finally {
    setDownloading(false)
  }
}

// 安装更新
async function handleInstallUpdate(): Promise<void> {
  const pendingUpdate = getPendingTauriUpdate()
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

/* 更新说明 Markdown 渲染样式 */
.release-notes {
  line-height: 1.6;
  font-size: 14px;
  color: #303133;
}

.release-notes :deep(h1),
.release-notes :deep(h2),
.release-notes :deep(h3),
.release-notes :deep(h4) {
  margin: 16px 0 8px;
  font-weight: 600;
  line-height: 1.4;
}

.release-notes :deep(h1) { font-size: 20px; }
.release-notes :deep(h2) { font-size: 17px; }
.release-notes :deep(h3) { font-size: 15px; }
.release-notes :deep(h4) { font-size: 14px; }

.release-notes :deep(p) {
  margin: 8px 0;
}

.release-notes :deep(ul),
.release-notes :deep(ol) {
  padding-left: 24px;
  margin: 8px 0;
}

.release-notes :deep(li) {
  margin: 4px 0;
}

.release-notes :deep(a) {
  color: var(--el-color-primary);
  text-decoration: none;
  cursor: pointer;
}

.release-notes :deep(a:hover) {
  text-decoration: underline;
}

.release-notes :deep(code) {
  background: #f5f7fa;
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 13px;
  font-family: 'SF Mono', Monaco, Menlo, Consolas, monospace;
}

.release-notes :deep(pre) {
  background: #f5f7fa;
  padding: 12px 16px;
  border-radius: 6px;
  overflow-x: auto;
  margin: 8px 0;
}

.release-notes :deep(pre code) {
  background: none;
  padding: 0;
}

.release-notes :deep(blockquote) {
  border-left: 3px solid #dcdfe6;
  padding-left: 12px;
  margin: 8px 0;
  color: #909399;
}

.release-notes :deep(hr) {
  border: none;
  border-top: 1px solid #ebeef5;
  margin: 12px 0;
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
