<template>
  <div class="card-management-view">
    <el-container>
      <!-- 左侧项目列表面板 -->
      <el-aside
        :width="sidebarCollapsed ? '0px' : '240px'"
        class="project-panel"
        :class="{ collapsed: sidebarCollapsed }"
      >
        <ProjectList
          v-show="!sidebarCollapsed"
          :projects="projects"
          :selected-project-id="selectedProjectId"
          :loading="projectLoading"
          @select="handleSelectProject"
          @create="handleCreateProject"
          @rename="handleRenameProject"
          @delete="handleDeleteProject"
        />
      </el-aside>

      <!-- 折叠/展开按钮 -->
      <div
        class="collapse-btn"
        :style="{ left: sidebarCollapsed ? '0px' : '240px' }"
        :title="sidebarCollapsed ? '展开项目列表' : '折叠项目列表'"
        @click="toggleSidebar"
      >
        <el-icon>
          <DArrowLeft v-if="!sidebarCollapsed" />
          <DArrowRight v-else />
        </el-icon>
      </div>

      <!-- 右侧卡片列表面板 -->
      <el-main class="card-panel">
        <CardListPlaceholder
          v-if="!selectedProjectId"
          message="请在左侧选择一个转卡"
        />
        <CardList
          v-else
          :cards="cards"
          :total="cardTotal"
          :page="cardPage"
          :page-size="cardPageSize"
          :loading="cardLoading"
          :project-id="selectedProjectId"
          @add="handleAddCard"
          @view="handleViewCard"
          @distribute="handleDistributeCard"
          @return="handleReturnCard"
          @delete="handleDeleteCard"
          @print-waybill="handlePrintWaybill"
          @search="handleSearchCard"
          @filter="handleFilterCard"
          @page-change="handlePageChange"
        />
      </el-main>
    </el-container>

    <!-- 项目弹窗 -->
    <ProjectDialog
      v-model:visible="projectDialogVisible"
      :mode="projectDialogMode"
      :project="editingProject"
      @confirm="handleProjectDialogConfirm"
    />

    <!-- 卡片录入弹窗 -->
    <CardInputDialog
      ref="cardInputDialogRef"
      v-model:visible="cardInputDialogVisible"
      :projects="projects"
      :preselected-project-id="selectedProjectId"
      @confirm="handleCardInputConfirm"
    />

    <!-- 分发弹窗 -->
    <DistributeDialog
      v-model:visible="distributeDialogVisible"
      :card="operatingCard"
      @confirm="handleDistributeConfirm"
      @refresh="loadCards"
    />

    <!-- 退卡弹窗 -->
    <ReturnDialog
      v-model:visible="returnDialogVisible"
      :card="operatingCard"
      @confirm="handleReturnConfirm"
    />

    <!-- 卡片详情弹窗 -->
    <CardDetailDialog
      v-model:visible="cardDetailDialogVisible"
      :card="operatingCard"
      @distribute="handleDistributeCard"
      @return="handleReturnCard"
    />

    <!-- 运单打印弹窗 -->
    <WaybillPrintDialog
      v-model:visible="waybillPrintDialogVisible"
      :default-waybill-no="waybillPrintDefaultNo"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import type { ProjectWithStats, CardWithProject, PagedCards } from '@/types/models'
import type {
  CardInputConfirmData,
  CardInputDialogInstance,
  DistributeConfirmData,
  ReturnConfirmData
} from '@/types/components'
import ProjectList from '@/components/projects/ProjectList.vue'
import ProjectDialog from '@/components/projects/ProjectDialog.vue'
import CardListPlaceholder from '@/components/cards/CardListPlaceholder.vue'
import CardList from '@/components/cards/CardList.vue'
import CardInputDialog from '@/components/cards/CardInputDialog.vue'
import DistributeDialog from '@/components/cards/DistributeDialog.vue'
import ReturnDialog from '@/components/cards/ReturnDialog.vue'
import CardDetailDialog from '@/components/cards/CardDetailDialog.vue'
import WaybillPrintDialog from '@/components/cards/WaybillPrintDialog.vue'
import { formatSerial } from '@/utils/format'
import { useQtyDisplayMode } from '@/composables/useQtyDisplayMode'

const { formatQty } = useQtyDisplayMode()

// ==================== 侧边栏状态 ====================
const sidebarCollapsed = ref<boolean>(false)

const toggleSidebar = (): void => {
  sidebarCollapsed.value = !sidebarCollapsed.value
}

// ==================== 项目相关状态 ====================
const projects = ref<ProjectWithStats[]>([])
const selectedProjectId = ref<string | null>(null)
const projectLoading = ref<boolean>(false)

// 项目弹窗状态
const projectDialogVisible = ref<boolean>(false)
const projectDialogMode = ref<'create' | 'edit'>('create')
const editingProject = ref<ProjectWithStats | null>(null)

// ==================== 卡片相关状态 ====================
const cards = ref<CardWithProject[]>([])
const cardTotal = ref<number>(0)
const cardPage = ref<number>(1)
const cardPageSize = ref<number>(20)
const cardLoading = ref<boolean>(false)
const cardSearchKeyword = ref<string>('')
const cardStatusFilter = ref<string>('')

// 卡片弹窗状态
const cardInputDialogRef = ref<CardInputDialogInstance | null>(null)
const cardInputDialogVisible = ref<boolean>(false)
const distributeDialogVisible = ref<boolean>(false)
const returnDialogVisible = ref<boolean>(false)
const cardDetailDialogVisible = ref<boolean>(false)
const waybillPrintDialogVisible = ref<boolean>(false)
const waybillPrintDefaultNo = ref<string>('')
const operatingCard = ref<CardWithProject | null>(null)

// ==================== 项目相关方法 ====================
const loadProjects = async (): Promise<void> => {
  projectLoading.value = true
  try {
    const result = await invoke<ProjectWithStats[]>('list_projects_cmd')
    projects.value = result
  } catch (error) {
    ElMessage.error('加载项目列表失败: ' + error)
  } finally {
    projectLoading.value = false
  }
}

const handleSelectProject = (projectId: string): void => {
  selectedProjectId.value = projectId
}

const handleCreateProject = (): void => {
  projectDialogMode.value = 'create'
  editingProject.value = null
  projectDialogVisible.value = true
}

const handleRenameProject = (project: ProjectWithStats): void => {
  projectDialogMode.value = 'edit'
  editingProject.value = project
  projectDialogVisible.value = true
}

const handleDeleteProject = async (project: ProjectWithStats): Promise<void> => {
  try {
    await ElMessageBox.confirm(
        `删除转卡将同时删除所有关联卡片，是否继续？`,
        '确认删除',
        {
          confirmButtonText: '确定',
          cancelButtonText: '取消',
          type: 'warning',
        }
    )

    await invoke('delete_project_cmd', {id: project.id})
    ElMessage.success('删除成功')

    if (selectedProjectId.value === project.id) {
      selectedProjectId.value = null
    }

    await loadProjects()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败: ' + error)
    }
  }
}

const handleProjectDialogConfirm = async (data: { name: string }): Promise<void> => {
  try {
    if (projectDialogMode.value === 'create') {
      await invoke('create_project_cmd', { name: data.name })
      ElMessage.success('创建成功')
    } else {
      await invoke('update_project_cmd', { id: editingProject.value!.id, name: data.name })
      ElMessage.success('更新成功')
    }

    projectDialogVisible.value = false
    await loadProjects()
  } catch (error) {
    ElMessage.error(String(error))
  }
}

// ==================== 卡片相关方法 ====================
const loadCards = async (): Promise<void> => {
  if (!selectedProjectId.value) return

  cardLoading.value = true
  try {
    const result = await invoke<PagedCards>('list_cards_cmd', {
      projectId: selectedProjectId.value,
      callsign: cardSearchKeyword.value || null,
      status: cardStatusFilter.value || null,
      page: cardPage.value,
      pageSize: cardPageSize.value
    })
    cards.value = result.items
    cardTotal.value = result.total
  } catch (error) {
    ElMessage.error('加载卡片列表失败: ' + error)
  } finally {
    cardLoading.value = false
  }
}

const handleAddCard = (): void => {
  cardInputDialogVisible.value = true
}

const handleViewCard = (card: CardWithProject): void => {
  operatingCard.value = card
  cardDetailDialogVisible.value = true
}

const handleDistributeCard = (card: CardWithProject): void => {
  operatingCard.value = card
  distributeDialogVisible.value = true
}

const handleReturnCard = (card: CardWithProject): void => {
  operatingCard.value = card
  returnDialogVisible.value = true
}

const handlePrintWaybill = (card: CardWithProject): void => {
  // 如果卡片已分发且有备注，使用备注作为默认运单号
  if (card.metadata?.distribution?.remarks) {
    waybillPrintDefaultNo.value = card.metadata.distribution.remarks
  } else {
    waybillPrintDefaultNo.value = ''
  }
  waybillPrintDialogVisible.value = true
}

const handleDeleteCard = async (card: CardWithProject): Promise<void> => {
  try {
    await ElMessageBox.confirm(
        `确定要删除此卡片吗？`,
        '确认删除',
        {
          confirmButtonText: '确定',
          cancelButtonText: '取消',
          type: 'warning',
        }
    )

    await invoke('delete_card_cmd', {id: card.id})
    ElMessage.success('删除成功')

    await loadCards()
    await loadProjects() // 刷新项目卡片数量
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败: ' + error)
    }
  }
}

const handleSearchCard = (keyword: string): void => {
  cardSearchKeyword.value = keyword
  cardPage.value = 1
  loadCards()
}

const handleFilterCard = (status: string): void => {
  cardStatusFilter.value = status
  cardPage.value = 1
  loadCards()
}

const handlePageChange = ({ page, pageSize }: { page: number; pageSize: number }): void => {
  cardPage.value = page
  cardPageSize.value = pageSize
  loadCards()
}

const handleCardInputConfirm = async (data: CardInputConfirmData): Promise<void> => {
  try {
    // 获取项目名称（用于打印）
    const project = projects.value.find(p => p.id === data.projectId)
    const projectName = project?.name || ''

    // 创建卡片（serial 直接传数字）
    await invoke('create_card_cmd', {
      projectId: data.projectId,
      callsign: data.callsign,
      qty: data.qty,
      serial: data.serial || null
    })

    // 如果需要打印
    if (data.printAfterSave && data.printerName) {
      try {
        const serialStr = formatSerial(data.serial)
        await invoke('print_qsl', {
          printerName: data.printerName,
          request: {
            template_path: null,
            data: {
              project_name: projectName,
              callsign: data.callsign,
              sn: serialStr,
              qty: formatQty(data.qty)
            }
          }
        })
        ElMessage.success(`录入并打印成功: ${data.callsign} x ${formatQty(data.qty)}`)
      } catch (printError) {
        ElMessage.warning(`录入成功，但打印失败: ${printError}`)
      }
    } else {
      ElMessage.success(`录入成功: ${data.callsign} x ${formatQty(data.qty)}`)
    }

    if (data.continuousMode) {
      // 连续录入模式：重置表单，等待序列号加载完成避免竞态条件
      await cardInputDialogRef.value?.resetForContinuous()
    } else {
      cardInputDialogVisible.value = false
    }

    await loadCards()
    await loadProjects() // 刷新项目卡片数量
  } catch (error) {
    ElMessage.error('录入失败: ' + error)
  }
}

const handleDistributeConfirm = async (data: DistributeConfirmData): Promise<void> => {
  try {
    await invoke('distribute_card_cmd', {
      id: data.id,
      method: data.method,
      address: data.address,
      remarks: data.remarks,
      proxyCallsign: data.proxy_callsign || null
    })
    ElMessage.success('分发成功')

    distributeDialogVisible.value = false
    await loadCards()
  } catch (error) {
    ElMessage.error('分发失败: ' + error)
  }
}

const handleReturnConfirm = async (data: ReturnConfirmData): Promise<void> => {
  try {
    await invoke('return_card_cmd', {
      id: data.id,
      method: data.method,
      remarks: data.remarks
    })
    ElMessage.success('退卡成功')

    returnDialogVisible.value = false
    await loadCards()
  } catch (error) {
    ElMessage.error('退卡失败: ' + error)
  }
}

// 监听项目选择变化，加载卡片
watch(selectedProjectId, (newId: string | null): void => {
  if (newId) {
    cardPage.value = 1
    cardSearchKeyword.value = ''
    cardStatusFilter.value = ''
    loadCards()
  } else {
    cards.value = []
    cardTotal.value = 0
  }
})

// 组件挂载时加载数据
onMounted(async () => {
  await loadProjects()

  // 如果有项目，自动选中第一个；否则打开新建弹窗
  if (projects.value.length > 0) {
    selectedProjectId.value = projects.value[0].id
  } else {
    handleCreateProject()
  }
})
</script>

<style scoped>
.card-management-view {
  height: 100%;
}

.card-management-view .el-container {
  height: 100%;
  position: relative;
}

.project-panel {
  background: #fafafa;
  border-right: 1px solid #dcdfe6;
  overflow: hidden;
  transition: width 0.3s ease;
}

.project-panel.collapsed {
  border-right: none;
}

.collapse-btn {
  position: absolute;
  left: 240px;
  top: 50%;
  transform: translateY(-50%);
  width: 16px;
  height: 48px;
  background: #f0f0f0;
  border: 1px solid #dcdfe6;
  border-left: none;
  border-radius: 0 4px 4px 0;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  z-index: 10;
  transition: left 0.3s ease, background-color 0.2s;
}

.collapse-btn:hover {
  background: #e0e0e0;
}

.card-panel {
  padding: 0;
  overflow: hidden;
}
</style>
