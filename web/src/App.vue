<template>
  <!-- 全局 Loading -->
  <GlobalLoading />

  <el-container style="height: 100vh">
    <!-- 顶部标题栏 -->
    <el-header style="background: #409EFF; padding: 0">
      <div style="display: flex; align-items: center; height: 100%; padding: 0 30px">
        <h2 style="margin: 0; flex: 1; color: white">
          qsl-cardhub
        </h2>
        <span style="font-size: 14px; opacity: 0.9; color: white">业余无线电卡片打印系统</span>
      </div>
    </el-header>

    <el-container style="flex: 1; overflow: hidden">
      <!-- 左侧导航 -->
      <el-aside
        width="220px"
        style="background: #f5f5f5; border-right: 1px solid #e0e0e0"
      >
        <el-menu
          :default-active="activeMenu"
          style="border: none; background: #f5f5f5"
          @select="handleMenuSelect"
        >
          <el-menu-item index="cards">
            <el-icon>
              <Box />
            </el-icon>
            <span>卡片管理</span>
          </el-menu-item>

          <el-menu-item index="sf-orders">
            <el-icon>
              <IconSfExpress />
            </el-icon>
            <span>顺丰订单</span>
          </el-menu-item>

          <el-sub-menu index="data-config-menu">
            <template #title>
              <el-icon>
                <Connection />
              </el-icon>
              <span>数据配置</span>
            </template>

            <el-menu-item index="data-config-qrz-cn">
              <span>QRZ.cn</span>
            </el-menu-item>

            <el-menu-item index="data-config-qrz-com">
              <span>QRZ.com</span>
            </el-menu-item>

            <el-menu-item index="data-config-data-transfer">
              <span>数据管理</span>
            </el-menu-item>

            <el-menu-item index="data-config-sf-express">
              <span>顺丰速运</span>
            </el-menu-item>
          </el-sub-menu>

          <el-divider style="margin: 20px 0" />

          <el-menu-item index="print">
            <el-icon>
              <Printer />
            </el-icon>
            <span>打印</span>
          </el-menu-item>

          <el-menu-item index="config">
            <el-icon>
              <Setting />
            </el-icon>
            <span>打印配置</span>
          </el-menu-item>

          <el-menu-item index="template">
            <el-icon>
              <Edit />
            </el-icon>
            <span>打印模板</span>
          </el-menu-item>

          <el-divider style="margin: 20px 0" />

          <el-menu-item index="logs">
            <el-icon>
              <Document />
            </el-icon>
            <span>日志</span>
          </el-menu-item>

          <el-menu-item index="about">
            <el-icon>
              <InfoFilled />
            </el-icon>
            <span>关于</span>
            <el-tag
              v-if="updateState.showBadge"
              type="success"
              size="small"
              effect="dark"
              round
              style="margin-left: 8px"
            >
              New!
            </el-tag>
          </el-menu-item>
        </el-menu>
      </el-aside>

      <!-- 主内容区 -->
      <el-main style="background: #fff; overflow: hidden">
        <!-- 打印页面 -->
        <PrintView v-if="activeMenu === 'print'" />

        <!-- 配置管理页面 -->
        <ConfigView
          v-if="activeMenu === 'config'"
          :auto-open-new-dialog="shouldAutoOpenNewConfig"
        />

        <!-- 模板设置页面 -->
        <TemplateView v-if="activeMenu === 'template'" />

        <!-- 卡片管理页面 -->
        <CardManagementView v-if="activeMenu === 'cards'" />

        <!-- QRZ.cn 配置页面 -->
        <QRZConfigView v-if="activeMenu === 'data-config-qrz-cn'" />

        <!-- QRZ.com 配置页面 -->
        <QRZComConfigView v-if="activeMenu === 'data-config-qrz-com'" />

        <!-- 数据管理页面 -->
        <DataTransferView v-if="activeMenu === 'data-config-data-transfer'" />

        <!-- 顺丰速运配置页面 -->
        <SFExpressConfigView v-if="activeMenu === 'data-config-sf-express'" />

        <!-- 顺丰订单列表页面 -->
        <SFOrderListView v-if="activeMenu === 'sf-orders'" />

        <!-- 日志查看页面 -->
        <LogView v-if="activeMenu === 'logs'" />

        <!-- 关于页面 -->
        <AboutView v-if="activeMenu === 'about'" />
      </el-main>
    </el-container>
  </el-container>
</template>

<script setup lang="ts">
import { h, onMounted, onUnmounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import { ElNotification, ElButton } from 'element-plus'
import type { Profile } from '@/types/models'
import PrintView from '@/views/PrintView.vue'
import ConfigView from '@/views/ConfigView.vue'
import TemplateView from '@/views/TemplateView.vue'
import CardManagementView from '@/views/CardManagementView.vue'
import QRZConfigView from '@/views/QRZConfigView.vue'
import QRZComConfigView from '@/views/QRZComConfigView.vue'
import DataTransferView from '@/views/DataTransferView.vue'
import SFExpressConfigView from '@/views/SFExpressConfigView.vue'
import SFOrderListView from '@/views/SFOrderListView.vue'
import LogView from '@/views/LogView.vue'
import AboutView from '@/views/AboutView.vue'
import {
  updateState,
  setUpdateAvailable,
  type UpdateInfo
} from '@/stores/updateStore'
import { logger } from '@/utils/logger'
import IconSfExpress from '~icons/custom/sf-express'
import GlobalLoading from '@/components/common/GlobalLoading.vue'

const activeMenu = ref<string>('cards')
const shouldAutoOpenNewConfig = ref<boolean>(false)

// 更新检查定时器
let updateCheckTimer: ReturnType<typeof setInterval> | null = null
const UPDATE_CHECK_INTERVAL = 5 * 60 * 1000 // 5分钟

// GitHub 仓库信息
const GITHUB_OWNER = 'HerbertGao'
const GITHUB_REPO = 'QSL-CardHub'

// 比较版本号
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

// 静默检查更新
async function silentCheckUpdate(): Promise<void> {
  // 如果已经发现更新，不再重复检查
  if (updateState.hasUpdate) {
    logger.info('[更新检查] 已发现更新，跳过检查')
    return
  }

  logger.info('[更新检查] 开始检查更新...')

  try {
    const currentVersion = await getVersion()
    logger.info(`[更新检查] 当前版本: v${currentVersion}`)

    const response = await fetch(
      `https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPO}/releases/latest`,
      {
        headers: {
          'Accept': 'application/vnd.github.v3+json'
        }
      }
    )

    if (!response.ok) {
      return
    }

    const release = await response.json()
    const latestVersion = release.tag_name.replace(/^v/, '')
    logger.info(`[更新检查] 最新版本: v${latestVersion}`)

    if (compareVersions(latestVersion, currentVersion) > 0) {
      const updateInfo: UpdateInfo = {
        version: latestVersion,
        notes: release.body || '无更新说明',
        pubDate: release.published_at,
        downloadUrl: release.html_url
      }
      setUpdateAvailable(updateInfo)
      logger.info(`[更新检查] 发现新版本: v${latestVersion}`)

      // 发送更新通知
      ElNotification({
        title: '发现新版本',
        message: h('div', [
          h('p', { style: 'margin: 0 0 8px 0' }, `新版本 v${latestVersion} 已发布`),
          h(ElButton, {
            type: 'primary',
            size: 'small',
            onClick: () => {
              activeMenu.value = 'about'
            }
          }, () => '查看详情')
        ]),
        type: 'success',
        duration: 8000,
        position: 'bottom-right'
      })
    } else {
      logger.info('[更新检查] 已是最新版本')
    }
  } catch (error) {
    // 静默忽略错误
    logger.error(`[更新检查] 检查失败: ${error}`)
  }
}

const handleMenuSelect = (index: string): void => {
  activeMenu.value = index
}

// 监听菜单切换，重置自动打开新建配置的标志
watch(activeMenu, (newMenu: string, oldMenu: string) => {
  // 当离开配置页面时，重置标志，避免下次进入时重复打开
  if (oldMenu === 'config' && newMenu !== 'config') {
    shouldAutoOpenNewConfig.value = false
  }
})

// 启动时检查配置状态
onMounted(async () => {
  try {
    // 调用后端 API 获取配置列表
    const profiles = await invoke<Profile[]>('get_profiles')

    // 如果没有任何配置，跳转到配置页面并自动打开新建弹框
    if (!profiles || profiles.length === 0) {
      activeMenu.value = 'config'
      shouldAutoOpenNewConfig.value = true
    }
  } catch (error) {
    console.error('获取配置失败:', error)
    // 出错时也跳转到配置页面并自动打开新建弹框
    activeMenu.value = 'config'
    shouldAutoOpenNewConfig.value = true
  }

  // 启动时静默检查更新（不阻塞启动流程）
  silentCheckUpdate()

  // 启动定时检查更新（每5分钟）
  updateCheckTimer = setInterval(() => {
    silentCheckUpdate()
  }, UPDATE_CHECK_INTERVAL)
})

// 组件卸载时清理定时器
onUnmounted(() => {
  if (updateCheckTimer) {
    clearInterval(updateCheckTimer)
    updateCheckTimer = null
  }
})
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'PingFang SC', 'Hiragino Sans GB',
  'Microsoft YaHei', 'Helvetica Neue', Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #303133;
  background: #fff;
}

#app {
  height: 100vh;
  overflow: hidden;
}

.page-content {
  padding: 30px;
  height: 100%;
  max-height: 100%;
  overflow-y: auto;
  box-sizing: border-box;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

/* Element Plus 样式覆盖 */
.el-menu {
  background: #f5f5f5 !important;
}

.el-menu-item {
  border-radius: 0 8px 8px 0;
  margin: 4px 0;
}

.el-menu-item.is-active {
  background: #ecf5ff !important;
  color: #409eff !important;
}

.el-menu-item:hover {
  background: #e6f7ff !important;
}

.el-card {
  border-radius: 12px;
  border: 1px solid #e0e0e0;
}

.el-card__header {
  background: #fafafa;
  border-bottom: 1px solid #e0e0e0;
}

.el-form-item__label {
  font-weight: 500;
}

.el-button {
  border-radius: 6px;
}

/* 滚动条样式 */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-thumb {
  background: #d0d0d0;
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: #b0b0b0;
}

::-webkit-scrollbar-track {
  background: transparent;
}
</style>
