<template>
  <div
    class="page-content"
    style="height: calc(100vh - 60px); display: flex; flex-direction: column"
  >
    <h1 style="margin-bottom: 20px">
      标签模板配置
    </h1>

    <!-- 模板类型选择 -->
    <div style="margin-bottom: 20px">
      <el-radio-group
        v-model="templateType"
        @change="handleTemplateTypeChange"
      >
        <el-radio-button value="callsign">
          呼号模板
        </el-radio-button>
        <el-radio-button value="address">
          地址模板
        </el-radio-button>
      </el-radio-group>
    </div>

    <el-row
      :gutter="20"
      style="flex: 1; min-height: 0; margin-bottom: 20px"
    >
      <!-- 左侧表单 -->
      <el-col
        :span="12"
        style="height: 100%"
      >
        <el-card
          v-loading="loading"
          shadow="hover"
          style="height: 100%; display: flex; flex-direction: column"
        >
          <template #header>
            <span style="font-weight: bold">模板参数配置</span>
          </template>

          <el-form
            v-if="templateConfig"
            :model="templateConfig"
            label-width="150px"
            style="max-width: 100%; overflow-y: auto; flex: 1; padding-right: 10px"
          >
            <!-- 页面配置 -->
            <el-collapse
              v-model="activeCollapse"
              style="border: none"
            >
              <el-collapse-item
                name="page"
                title="页面配置"
              >
                <el-form-item label="DPI">
                  <el-input-number
                    :model-value="templateConfig.page.dpi"
                    disabled
                    :controls="false"
                  />
                  <span style="margin-left: 10px; color: #909399">dots/inch</span>
                </el-form-item>

                <el-form-item label="纸张宽度">
                  <el-input-number
                    :model-value="templateConfig.page.width_mm"
                    disabled
                    :controls="false"
                  />
                  <span style="margin-left: 10px; color: #909399">mm</span>
                </el-form-item>

                <el-form-item label="纸张高度">
                  <el-input-number
                    :model-value="templateConfig.page.height_mm"
                    disabled
                    :controls="false"
                  />
                  <span style="margin-left: 10px; color: #909399">mm</span>
                </el-form-item>

                <el-divider />

                <el-form-item label="左边距">
                  <el-input-number
                    v-model="templateConfig.page.margin_left_mm"
                    :min="0"
                    :step="0.5"
                  />
                  <span style="margin-left: 10px; color: #909399">mm</span>
                </el-form-item>

                <el-form-item label="右边距">
                  <el-input-number
                    v-model="templateConfig.page.margin_right_mm"
                    :min="0"
                    :step="0.5"
                  />
                  <span style="margin-left: 10px; color: #909399">mm</span>
                </el-form-item>

                <el-form-item label="上边距">
                  <el-input-number
                    v-model="templateConfig.page.margin_top_mm"
                    :min="0"
                    :step="0.5"
                  />
                  <span style="margin-left: 10px; color: #909399">mm</span>
                </el-form-item>

                <el-form-item label="下边距">
                  <el-input-number
                    v-model="templateConfig.page.margin_bottom_mm"
                    :min="0"
                    :step="0.5"
                  />
                  <span style="margin-left: 10px; color: #909399">mm</span>
                </el-form-item>

                <el-divider />

                <el-form-item label="显示边框">
                  <el-switch v-model="templateConfig.page.border" />
                </el-form-item>

                <el-form-item
                  v-if="templateConfig.page.border"
                  label="边框粗细"
                >
                  <el-input-number
                    v-model="templateConfig.page.border_thickness_mm"
                    :min="0.1"
                    :step="0.1"
                  />
                  <span style="margin-left: 10px; color: #909399">mm</span>
                </el-form-item>

                <!-- 双份打印（仅地址模板显示） -->
                <template v-if="templateType === 'address'">
                  <el-divider />
                  <el-form-item label="双份打印">
                    <el-switch v-model="templateConfig.page.duplicate_print" />
                    <span style="margin-left: 10px; color: #909399">上下各打印一份</span>
                  </el-form-item>
                </template>
              </el-collapse-item>

              <!-- 布局配置 -->
              <el-collapse-item
                name="layout"
                title="布局配置"
              >
                <el-form-item label="水平对齐">
                  <el-select v-model="templateConfig.layout.align_h">
                    <el-option
                      label="居中"
                      value="center"
                    />
                    <el-option
                      label="左对齐"
                      value="left"
                    />
                    <el-option
                      label="右对齐"
                      value="right"
                    />
                  </el-select>
                </el-form-item>

                <el-form-item label="垂直对齐">
                  <el-select v-model="templateConfig.layout.align_v">
                    <el-option
                      label="居中"
                      value="center"
                    />
                    <el-option
                      label="顶部对齐"
                      value="top"
                    />
                    <el-option
                      label="底部对齐"
                      value="bottom"
                    />
                  </el-select>
                </el-form-item>

                <el-form-item label="元素间距">
                  <el-input-number
                    v-model="templateConfig.layout.gap_mm"
                    :min="0"
                    :step="0.5"
                  />
                  <span style="margin-left: 10px; color: #909399">mm</span>
                </el-form-item>

                <el-form-item label="行间距">
                  <el-input-number
                    v-model="templateConfig.layout.line_gap_mm"
                    :min="0"
                    :step="0.5"
                  />
                  <span style="margin-left: 10px; color: #909399">mm</span>
                </el-form-item>
              </el-collapse-item>

              <!-- 元素配置 -->
              <el-collapse-item
                name="elements"
                title="元素配置"
              >
                <div
                  v-for="(element, index) in templateConfig.elements"
                  :key="index"
                  style="margin-bottom: 20px; padding: 15px; background: #fafafa; border-radius: 8px"
                >
                  <div style="font-weight: bold; margin-bottom: 10px; color: #409eff">
                    元素 {{ index + 1 }}: {{ element.id }}
                  </div>

                  <el-form-item label="类型">
                    <el-input
                      :model-value="element.type"
                      disabled
                    />
                  </el-form-item>

                  <el-form-item label="来源">
                    <el-input
                      :model-value="element.source"
                      disabled
                    />
                  </el-form-item>

                  <!-- Text 元素 -->
                  <template v-if="element.type === 'text'">
                    <el-form-item
                      v-if="element.value !== undefined"
                      label="文本内容"
                    >
                      <el-input
                        :model-value="element.value"
                        disabled
                      />
                    </el-form-item>

                    <el-form-item
                      v-if="element.key !== undefined"
                      label="数据键"
                    >
                      <el-input
                        :model-value="element.key"
                        disabled
                      />
                    </el-form-item>

                    <el-form-item
                      v-if="element.format !== undefined"
                      label="格式化"
                    >
                      <el-input
                        :model-value="element.format"
                        disabled
                      />
                    </el-form-item>

                    <el-form-item label="最大高度">
                      <el-input-number
                        v-model="element.max_height_mm"
                        :min="1"
                        :step="0.5"
                      />
                      <span style="margin-left: 10px; color: #909399">mm</span>
                    </el-form-item>
                  </template>

                  <!-- Barcode 元素 -->
                  <template v-if="element.type === 'barcode'">
                    <el-form-item label="数据键">
                      <el-input
                        :model-value="element.key"
                        disabled
                      />
                    </el-form-item>

                    <el-form-item label="条码高度">
                      <el-input-number
                        :model-value="element.height_mm"
                        disabled
                        :controls="false"
                      />
                      <span style="margin-left: 10px; color: #909399">mm</span>
                    </el-form-item>

                    <el-form-item label="静区">
                      <el-input-number
                        :model-value="element.quiet_zone_mm"
                        disabled
                        :controls="false"
                      />
                      <span style="margin-left: 10px; color: #909399">mm</span>
                    </el-form-item>

                    <el-form-item label="人类可读">
                      <el-switch
                        :model-value="element.human_readable"
                        disabled
                      />
                    </el-form-item>
                  </template>
                </div>
              </el-collapse-item>

              <!-- 元数据 -->
              <el-collapse-item
                name="metadata"
                title="元数据（只读）"
              >
                <el-form-item label="模板名称">
                  <el-input
                    :model-value="templateConfig.metadata.name"
                    disabled
                  />
                </el-form-item>

                <el-form-item label="版本">
                  <el-input
                    :model-value="templateConfig.metadata.version"
                    disabled
                  />
                </el-form-item>

                <el-form-item label="描述">
                  <el-input
                    :model-value="templateConfig.metadata.description"
                    disabled
                    type="textarea"
                  />
                </el-form-item>
              </el-collapse-item>

              <!-- 字体配置 -->
              <el-collapse-item
                name="fonts"
                title="字体配置（只读）"
              >
                <el-form-item label="英文字体">
                  <el-input
                    :model-value="templateConfig.fonts.english"
                    disabled
                  />
                </el-form-item>

                <el-form-item label="中文字体">
                  <el-input
                    :model-value="templateConfig.fonts.chinese"
                    disabled
                  />
                </el-form-item>
              </el-collapse-item>

              <!-- 输出配置 -->
              <el-collapse-item
                name="output"
                title="输出配置（只读）"
              >
                <el-form-item label="渲染模式">
                  <el-input
                    :model-value="templateConfig.output.mode"
                    disabled
                  />
                </el-form-item>

                <el-form-item label="二值化阈值">
                  <el-input-number
                    :model-value="templateConfig.output.threshold"
                    disabled
                    :controls="false"
                  />
                </el-form-item>
              </el-collapse-item>
            </el-collapse>
          </el-form>

          <!-- 保存状态提示（放在卡片内底部） -->
          <div
            v-if="saveStatus"
            style="padding: 10px; border-top: 1px solid #e0e0e0; margin-top: auto"
          >
            <el-tag
              :type="saveStatus.type === 'success' ? 'success' : 'danger'"
              size="small"
            >
              {{ saveStatus.message }}
            </el-tag>
          </div>
        </el-card>
      </el-col>

      <!-- 右侧预览 -->
      <el-col
        :span="12"
        style="height: 100%"
      >
        <el-card
          shadow="hover"
          style="height: 100%; display: flex; flex-direction: column"
        >
          <template #header>
            <div style="display: flex; justify-content: space-between; align-items: center">
              <span style="font-weight: bold">预览</span>
              <el-button
                type="primary"
                size="small"
                :loading="previewLoading"
                @click="handleRefreshPreview"
              >
                <el-icon v-if="!previewLoading">
                  <Refresh />
                </el-icon>
                刷新预览
              </el-button>
            </div>
          </template>

          <div
            v-loading="previewLoading"
            style="flex: 1; padding: 10px; display: flex; align-items: center; justify-content: center; overflow: hidden"
          >
            <el-empty
              v-if="!previewImageUrl"
              description="加载中..."
              :image-size="80"
            />
            <img
              v-else
              :src="`data:image/png;base64,${previewImageUrl}`"
              style="max-width: 100%; max-height: 100%; object-fit: contain; border-radius: 8px; border: 1px solid #e0e0e0"
              alt="模板预览"
            >
          </div>

          <!-- 预览提示（固定在底部） -->
          <div style="padding: 10px; border-top: 1px solid #e0e0e0; margin-top: auto; background: #f5f7fa">
            <div style="display: flex; align-items: center; color: #909399; font-size: 13px">
              <el-icon style="margin-right: 5px">
                <InfoFilled />
              </el-icon>
              <span>预览仅供参考，实际打印可能有细微差异</span>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import type { TemplateConfig } from '@/types/models'

interface SaveStatus {
  type: 'success' | 'error'
  message: string
}

interface PreviewRequest {
  template_path: string | null
  data: {
    project_name: string
    callsign: string
    sn: string
    qty: string
  }
  output_config: {
    mode: string
    threshold: number
  }
}

interface PreviewResponse {
  base64_data: string
}

// 响应式数据
const templateType = ref<'callsign' | 'address'>('callsign')
const templateConfig = ref<TemplateConfig | null>(null)
const loading = ref<boolean>(false)
const previewLoading = ref<boolean>(false)
const previewImageUrl = ref<string>('')
const saveStatus = ref<SaveStatus | null>(null)
const activeCollapse = ref<string[]>(['page', 'layout']) // 默认展开的折叠面板

// 防抖保存
let saveTimeout: ReturnType<typeof setTimeout> | null = null
const debouncedSave = (): void => {
  // 清除保存状态
  saveStatus.value = null

  // 清除之前的定时器
  if (saveTimeout) {
    clearTimeout(saveTimeout)
  }

  // 设置新的定时器
  saveTimeout = setTimeout(async () => {
    try {
      // 根据模板类型调用不同的保存命令
      const saveCommand = templateType.value === 'address'
        ? 'save_address_template_config'
        : 'save_template_config'
      await invoke(saveCommand, { config: templateConfig.value })
      saveStatus.value = { type: 'success', message: '✓ 配置已自动保存' }

      // 3秒后清除成功提示
      setTimeout(() => {
        if (saveStatus.value?.type === 'success') {
          saveStatus.value = null
        }
      }, 3000)
    } catch (error) {
      console.error('保存失败:', error)
      saveStatus.value = { type: 'error', message: `保存失败: ${error}` }
    }
  }, 500) // 500ms 防抖
}

// 监听配置变化，自动保存
watch(
  templateConfig,
  (): void => {
    if (templateConfig.value) {
      debouncedSave()
    }
  },
  { deep: true }
)

// 加载模板配置
const loadTemplateConfig = async (autoPreview: boolean = false): Promise<void> => {
  loading.value = true
  try {
    // 根据模板类型加载不同的配置
    const loadCommand = templateType.value === 'address'
      ? 'get_address_template_config'
      : 'get_template_config'
    const config = await invoke<TemplateConfig>(loadCommand)
    templateConfig.value = config

    // 配置加载成功后自动执行预览
    if (autoPreview) {
      // 使用 nextTick 确保配置已更新到 DOM
      await handleRefreshPreview(true)
    }
  } catch (error) {
    ElMessage.error(`加载模板配置失败: ${error}`)
    console.error('加载模板配置失败:', error)
  } finally {
    loading.value = false
  }
}

// 模板类型切换处理
const handleTemplateTypeChange = (): void => {
  // 清除预览
  previewImageUrl.value = ''
  // 重新加载配置并自动执行预览
  loadTemplateConfig(true)
}

// 刷新预览
// silent 参数：为 true 时不显示成功提示消息（用于自动预览）
const handleRefreshPreview = async (silent: boolean = false): Promise<void> => {
  previewLoading.value = true
  try {
    let response: PreviewResponse

    if (templateType.value === 'address') {
      // 地址模板预览
      response = await invoke<PreviewResponse>('preview_address', {
        request: {
          data: {
            name: '张三',
            callsign: 'BG7XXX',
            address: '广东省深圳市南山区科技园路1号'
          },
          output_config: {
            mode: 'text_bitmap_plus_native_barcode',
            threshold: 160
          }
        }
      })
    } else {
      // QSL 模板预览
      response = await invoke<PreviewResponse>('preview_qsl', {
        request: {
          template_path: null,
          data: {
            project_name: '预览测试',
            callsign: 'BG7XXX',
            sn: '001',
            qty: '100'
          },
          output_config: {
            mode: 'text_bitmap_plus_native_barcode',
            threshold: 160
          }
        } as PreviewRequest
      })
    }

    previewImageUrl.value = response.base64_data
    // 自动预览时不显示成功提示
    if (!silent) {
      ElMessage.success('预览生成成功')
    }
  } catch (error) {
    ElMessage.error(`预览生成失败: ${error}`)
    console.error('预览生成失败:', error)
  } finally {
    previewLoading.value = false
  }
}

// 组件挂载时加载配置并自动执行预览
onMounted((): void => {
  loadTemplateConfig(true)
})
</script>

<style scoped>
/* 页面布局 */
.page-content {
  padding: 30px;
}

/* 折叠面板样式优化 */
:deep(.el-collapse) {
  border-top: none;
  border-bottom: none;
}

:deep(.el-collapse-item__header) {
  font-weight: bold;
  font-size: 15px;
  background: #f5f7fa;
  padding-left: 15px;
  border-radius: 8px;
  margin-bottom: 10px;
}

:deep(.el-collapse-item__wrap) {
  border-bottom: none;
}

:deep(.el-collapse-item__content) {
  padding: 10px 15px 20px;
}

/* 表单项间距优化 */
:deep(.el-form-item) {
  margin-bottom: 18px;
}

/* 卡片样式 */
:deep(.el-card__body) {
  padding: 20px;
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

/* 滚动条样式 */
.el-form::-webkit-scrollbar {
  width: 6px;
}

.el-form::-webkit-scrollbar-thumb {
  background: #c0c4cc;
  border-radius: 3px;
}

.el-form::-webkit-scrollbar-thumb:hover {
  background: #909399;
}

.el-form::-webkit-scrollbar-track {
  background: transparent;
}
</style>
