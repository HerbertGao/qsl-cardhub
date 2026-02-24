<template>
  <div class="page-content">
    <h1>全局配置</h1>

    <el-card shadow="hover">
      <template #header>
        <div class="card-header">
          <span>应用设置</span>
        </div>
      </template>
      <el-form label-width="120px">
        <!-- 标题文本 -->
        <el-form-item label="标签标题文本">
          <div style="display: flex; align-items: center; gap: 12px; width: 100%">
            <el-input
              v-model="labelTitle"
              placeholder="请输入标签标题文本"
              @input="debouncedSaveLabelTitle"
              style="flex: 1"
            />
            <el-link type="primary" :underline="false" @click="goToTemplatePreview">
              前往模板配置预览 →
            </el-link>
          </div>
        </el-form-item>

        <!-- 数量显示模式 -->
        <el-form-item label="数量显示模式">
          <el-switch
            v-model="isApproximate"
            active-text="大致"
            inactive-text="精确"
            style="--el-switch-on-color: #409EFF; --el-switch-off-color: #67C23A"
          />
          <el-tooltip placement="right">
            <el-icon style="margin-left: 8px; color: #909399; cursor: help">
              <QuestionFilled />
            </el-icon>
            <template #content>
              精确模式显示具体数量；<br>
              大致模式显示数量范围（≤10、≤50、>50）
            </template>
          </el-tooltip>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { QuestionFilled } from '@element-plus/icons-vue'
import { navigateTo } from '@/stores/navigationStore'
import { useQtyDisplayMode } from '@/composables/useQtyDisplayMode'

const { qtyDisplayMode } = useQtyDisplayMode()

// 数量显示模式：boolean <-> QtyDisplayMode 转换
const isApproximate = computed({
  get: () => qtyDisplayMode.value === 'approximate',
  set: (val: boolean) => { qtyDisplayMode.value = val ? 'approximate' : 'exact' }
})

// 标题文本
const labelTitle = ref('')

let saveTimer: ReturnType<typeof setTimeout> | null = null

function debouncedSaveLabelTitle() {
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(async () => {
    try {
      await invoke('set_app_setting_cmd', { key: 'label_title', value: labelTitle.value })
    } catch (e) {
      console.warn('保存 label_title 失败', e)
    }
  }, 500)
}

function goToTemplatePreview() {
  navigateTo('print-config-template')
}

onMounted(async () => {
  try {
    const val = await invoke<string | null>('get_app_setting_cmd', { key: 'label_title' })
    if (val !== null) {
      labelTitle.value = val
    }
  } catch (e) {
    console.warn('加载 label_title 失败', e)
  }
})
</script>

<style scoped>
.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
</style>
