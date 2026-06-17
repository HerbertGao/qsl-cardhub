<template>
  <!-- 全局 Loading -->
  <GlobalLoading />

  <!-- 互斥状态：加载中 / 加载失败 / 首启模式网关 / 主界面。网关期主界面不挂载，避免空项目对话框等副作用 -->
  <div
    v-if="!bootReady"
    class="app-boot"
  />
  <!-- D7：配置加载失败显错误 + 重试入口（不静默退化为空配置） -->
  <div
    v-else-if="syncStore.loadError.value"
    class="app-load-error"
  >
    <el-result
      icon="warning"
      title="同步配置加载失败"
      sub-title="无法读取本地云同步配置（可能为临时故障）。请点击重试；若多次失败，请重启应用。"
    >
      <template #extra>
        <el-button
          type="primary"
          @click="retryLoad"
        >
          重试
        </el-button>
      </template>
    </el-result>
  </div>
  <AuthGateView
    v-else-if="gateVisible"
    @select="onGateSelect"
  />

  <el-container
    v-else
    style="height: 100vh"
  >
    <!-- 顶部标题栏 -->
    <el-header style="background: #409EFF; padding: 0">
      <div style="display: flex; align-items: center; height: 100%; padding: 0 30px">
        <h2 style="margin: 0; flex: 1; color: white">
          QSL 分卡助手
        </h2>
        <span
          class="tenant-badge"
          :title="syncStore.mode.value === 'cloud' ? '云同步模式，点击进入「租户 & 云端同步」设置' : '纯本地模式，点击配置云同步'"
          @click="goToSyncSettings"
        >
          <el-icon><Connection /></el-icon>
          <span>{{ badgeText }}</span>
        </span>
        <span style="font-size: 14px; opacity: 0.9; color: white; margin-left: 16px">业余无线电卡片管理系统</span>
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

            <el-menu-item index="data-config-global-settings">
              <span>全局配置</span>
            </el-menu-item>

            <el-menu-item index="data-config-qrz-cn">
              <span>QRZ.cn</span>
            </el-menu-item>

            <el-menu-item index="data-config-qrz-com">
              <span>QRZ.com</span>
            </el-menu-item>

            <el-menu-item index="data-config-sf-express">
              <span>顺丰速运</span>
            </el-menu-item>

            <el-menu-item index="data-config-data-transfer">
              <span>数据管理</span>
            </el-menu-item>
          </el-sub-menu>

          <el-sub-menu index="print-config-menu">
            <template #title>
              <el-icon>
                <Setting />
              </el-icon>
              <span>打印配置</span>
            </template>

            <el-menu-item index="print-config-printer">
              <span>打印机配置</span>
            </el-menu-item>

            <el-menu-item index="print-config-template">
              <span>标签模板配置</span>
            </el-menu-item>
          </el-sub-menu>

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
        <!-- 打印机配置页面 -->
        <ConfigView
          v-if="activeMenu === 'print-config-printer'"
        />

        <!-- 标签模板配置页面 -->
        <TemplateView v-if="activeMenu === 'print-config-template'" />

        <!-- 卡片管理页面 -->
        <CardManagementView v-if="activeMenu === 'cards'" />

        <!-- 全局配置页面 -->
        <GlobalSettingsView v-if="activeMenu === 'data-config-global-settings'" />

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
import { computed, h, onMounted, onUnmounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElNotification, ElButton } from 'element-plus'
import type { SinglePrinterConfig } from '@/types/models'
import ConfigView from '@/views/ConfigView.vue'
import TemplateView from '@/views/TemplateView.vue'
import CardManagementView from '@/views/CardManagementView.vue'
import GlobalSettingsView from '@/views/GlobalSettingsView.vue'
import QRZConfigView from '@/views/QRZConfigView.vue'
import QRZComConfigView from '@/views/QRZComConfigView.vue'
import DataTransferView from '@/views/DataTransferView.vue'
import SFExpressConfigView from '@/views/SFExpressConfigView.vue'
import SFOrderListView from '@/views/SFOrderListView.vue'
import LogView from '@/views/LogView.vue'
import AboutView from '@/views/AboutView.vue'
import { updateState } from '@/stores/updateStore'
import { checkForUpdate } from '@/services/updateCheck'
import { logger } from '@/utils/logger'
import IconSfExpress from '~icons/custom/sf-express'
import GlobalLoading from '@/components/common/GlobalLoading.vue'
import { useNavigationWatcher } from '@/stores/navigationStore'
import AuthGateView from '@/views/AuthGateView.vue'
import { syncStore } from '@/stores/syncStore'
import { isModeSelected, markModeSelected } from '@/utils/onboarding'

const activeMenu = ref<string>('cards')

// 首启模式网关是否可见（响应式，onMounted 据加载结果判定；选择后置 false 揭幕）
const gateVisible = ref<boolean>(false)

// 渲染揭幕标志：仅在「加载 → 迁移 → 网关/错误决策」全部定下后置真，
// 与 gateVisible/loadError 在同一同步块内翻转，杜绝主界面抢先挂载（Codex 竞态修复）
const bootReady = ref<boolean>(false)

// 标题栏租户徽章文案：纯本地 / 租户代码 / 云同步（有云配但无租户）
const badgeText = computed<string>(() =>
  syncStore.mode.value === 'local' ? '纯本地' : syncStore.tenant.value || '云同步'
)

// 监听导航事件
useNavigationWatcher(activeMenu)

// 徽章点击：进入「租户 & 云端同步」设置页
function goToSyncSettings(): void {
  activeMenu.value = 'data-config-data-transfer'
}

// 更新检查定时器
let updateCheckTimer: ReturnType<typeof setInterval> | null = null
const UPDATE_CHECK_INTERVAL = 5 * 60 * 1000 // 5分钟

// 静默检查更新（与关于页「检查更新」同一套逻辑，保证关于页展示与下载更新行为一致）
async function silentCheckUpdate(): Promise<void> {
  if (updateState.hasUpdate) {
    logger.info('[更新检查] 已发现更新，跳过检查')
    return
  }
  logger.info('[更新检查] 开始检查更新...')
  const hadUpdate = updateState.hasUpdate
  await checkForUpdate({ silent: true })
  if (!hadUpdate && updateState.hasUpdate && updateState.updateInfo) {
    logger.info(`[更新检查] 发现新版本: v${updateState.updateInfo.version}`)
    // 主界面未挂载的任一态都不弹更新通知（加载占位 `!bootReady`〔含 retry 关门窗口〕/ 网关 / 加载失败错误分支）：
    // 此时「查看详情」跳「关于」是死按钮（About 未挂载）；进主界面后「关于」徽章仍会提示。检查本身照常跑（D6 正交）
    if (!bootReady.value || gateVisible.value || syncStore.loadError.value) {
      return
    }
    ElNotification({
      title: '发现新版本',
      message: h('div', [
        h('p', { style: 'margin: 0 0 8px 0' }, `新版本 v${updateState.updateInfo.version} 已发布`),
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
  } else if (!updateState.hasUpdate) {
    logger.info('[更新检查] 已是最新版本')
  }
}

const handleMenuSelect = (index: string): void => {
  activeMenu.value = index
}

// 打印机配置检测：无配置则落到打印机配置页。抽成函数，供「非网关分支」与「纯本地选择后」复用
async function runPrinterCheck(): Promise<void> {
  try {
    // 调用后端 API 获取打印机配置（单配置模式）
    const config = await invoke<SinglePrinterConfig>('get_printer_config')

    // 如果没有配置打印机，跳转到打印机配置页面
    if (!config || !config.printer.name) {
      activeMenu.value = 'print-config-printer'
    }
  } catch (error) {
    console.error('获取打印机配置失败:', error)
    // 出错时也跳转到打印机配置页面
    activeMenu.value = 'print-config-printer'
  }
}

// 首启网关二选一回调：置 onboarding 标志 + 揭幕；云同步落到配置页，纯本地补跑打印机检测
function onGateSelect(choice: 'cloud' | 'local'): void {
  markModeSelected()
  if (choice === 'cloud') {
    // 云同步：**先**切到「数据管理」（含租户 & 云端同步）**再**揭幕，
    // 保证主界面首帧 activeMenu 已是设置页，绝不闪过默认卡片页（零项目会弹「新建项目」）
    activeMenu.value = 'data-config-data-transfer'
    gateVisible.value = false
  } else {
    // 纯本地：揭幕后补跑打印机检测（首启被网关拦截、此前未跑）
    gateVisible.value = false
    runPrinterCheck()
  }
}

// 加载完成后的引导序：迁移 → 网关判定 →（无网关）打印机检测，**最后**才揭幕（置 bootReady）。
// loadError 时跳过迁移/网关，直接揭幕到错误分支（由模板 + retryLoad 接管，D7：显错误 + 重试入口）。
// 关键：gateVisible/loadError 与 bootReady 在本函数同一同步执行中定下，Vue 单次 flush 即读到正确分支，
// 不存在「揭幕了但 gateVisible 还没赋值」的中间帧（Codex 竞态修复）
function bootstrapAfterLoad(): void {
  if (!syncStore.loadError.value) {
    // 迁移：已配置云端的老用户视为已 onboarding，不给他们弹网关
    if (!isModeSelected() && syncStore.apiUrl.value) {
      markModeSelected()
    }

    // 首启网关判定：未配置云端 + 从未选过模式 → 弹一次
    gateVisible.value = !syncStore.apiUrl.value && !isModeSelected()

    // 非网关分支才在此跑打印机检测；网关分支待用户选「纯本地」后补跑
    if (!gateVisible.value) {
      runPrinterCheck()
    }
  }

  // 决策全部定下后揭幕（加载态 → 错误/网关/主界面）
  bootReady.value = true
}

// 错误分支「重试」：重新加载云配置；成功则继续引导序，仍失败则停留错误分支。
// **先关揭幕门**（回到加载态）再 load——否则 load 清 loadError 后、bootstrapAfterLoad 重算 gateVisible 前，
// 会出现 bootReady=true/loadError=false/gateVisible=false 的一帧主界面抢挂（与首启竞态同源，Codex 二轮 Q6）。
// 关门期间错误分支（含本按钮）卸载，天然防连点。
async function retryLoad(): Promise<void> {
  bootReady.value = false
  await syncStore.load()
  bootstrapAfterLoad()
}

// 启动时检查配置状态
onMounted(async () => {
  // 先加载云配置（决定模式 / 徽章 / 是否首启 / 是否加载失败）
  await syncStore.load()
  bootstrapAfterLoad()

  // 更新检查无条件启动（与网关/错误分支正交；网关期由 silentCheckUpdate 内部抑制弹窗）
  silentCheckUpdate()
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

/* 首启加载占位（syncStore.load 期间，通常瞬时） */
.app-boot {
  height: 100vh;
  background: #fff;
}

/* 配置加载失败错误分支（D7） */
.app-load-error {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100vh;
  background: #fff;
}

/* 标题栏租户徽章 */
.tenant-badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  height: 26px;
  padding: 0 12px;
  border-radius: 13px;
  background: rgba(255, 255, 255, 0.18);
  color: #fff;
  font-size: 13px;
  line-height: 1;
  cursor: pointer;
  transition: background 0.15s ease;
  white-space: nowrap;
}

.tenant-badge:hover {
  background: rgba(255, 255, 255, 0.3);
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
