<template>
  <el-container style="height: 100vh">
    <!-- 顶部标题栏 -->
    <el-header style="background: #409EFF; padding: 0">
      <div style="display: flex; align-items: center; height: 100%; padding: 0 30px">
        <h2 style="margin: 0; flex: 1; color: white">qsl-cardhub</h2>
        <span style="font-size: 14px; opacity: 0.9; color: white">业余无线电卡片打印系统</span>
      </div>
    </el-header>

    <el-container>
      <!-- 左侧导航 -->
      <el-aside width="220px" style="background: #f5f5f5; border-right: 1px solid #e0e0e0">
        <el-menu
          :default-active="activeMenu"
          @select="handleMenuSelect"
          style="border: none; background: #f5f5f5"
        >
          <div style="padding: 20px 15px 15px; font-weight: bold; color: #666; font-size: 13px">
            功能菜单
          </div>

          <el-menu-item index="print">
            <el-icon><Printer /></el-icon>
            <span>打印</span>
          </el-menu-item>

          <el-menu-item index="config">
            <el-icon><Setting /></el-icon>
            <span>配置管理</span>
          </el-menu-item>

          <el-menu-item index="template">
            <el-icon><Edit /></el-icon>
            <span>模板设置</span>
          </el-menu-item>

          <el-menu-item index="logs">
            <el-icon><Document /></el-icon>
            <span>日志查看</span>
          </el-menu-item>

          <el-divider style="margin: 20px 0"></el-divider>

          <el-menu-item index="about">
            <el-icon><InfoFilled /></el-icon>
            <span>关于</span>
          </el-menu-item>
        </el-menu>
      </el-aside>

      <!-- 主内容区 -->
      <el-main style="background: #fff">
        <!-- 打印页面 -->
        <PrintView v-if="activeMenu === 'print'" />

        <!-- 配置管理页面 -->
        <ConfigView v-if="activeMenu === 'config'" :autoOpenNewDialog="shouldAutoOpenNewConfig" />

        <!-- 模板设置页面 -->
        <TemplateView v-if="activeMenu === 'template'" />

        <!-- 日志查看页面 -->
        <LogView v-if="activeMenu === 'logs'" />

        <!-- 关于页面 -->
        <AboutView v-if="activeMenu === 'about'" />
      </el-main>
    </el-container>
  </el-container>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import PrintView from './views/PrintView.vue'
import ConfigView from './views/ConfigView.vue'
import TemplateView from './views/TemplateView.vue'
import LogView from './views/LogView.vue'
import AboutView from './views/AboutView.vue'

const activeMenu = ref('print')
const shouldAutoOpenNewConfig = ref(false)

const handleMenuSelect = (index) => {
  activeMenu.value = index
}

// 启动时检查配置状态
onMounted(async () => {
  try {
    // 调用后端 API 获取配置列表
    const profiles = await invoke('get_profiles')
    const defaultId = await invoke('get_default_profile_id')

    // 如果没有配置或没有默认配置，跳转到配置页面并自动打开新建弹框
    if (!profiles || profiles.length === 0 || !defaultId) {
      activeMenu.value = 'config'
      shouldAutoOpenNewConfig.value = true
    }
  } catch (error) {
    console.error('获取配置失败:', error)
    // 出错时也跳转到配置页面并自动打开新建弹框
    activeMenu.value = 'config'
    shouldAutoOpenNewConfig.value = true
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
  overflow-y: auto;
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
  border-radius: 8px;
  margin: 4px 15px;
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
