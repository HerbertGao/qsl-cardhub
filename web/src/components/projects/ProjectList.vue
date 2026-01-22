<template>
  <div class="project-list">
    <!-- 顶部新建按钮 -->
    <div class="project-list-header">
      <el-button
        type="primary"
        style="width: 100%"
        @click="$emit('create')"
      >
        <el-icon>
          <Plus />
        </el-icon>
        <span>新建转卡</span>
      </el-button>
    </div>

    <!-- 转卡列表 -->
    <div
      v-loading="loading"
      class="project-list-content"
    >
      <div
        v-if="projects.length === 0 && !loading"
        class="empty-tip"
      >
        <el-empty
          description="暂无转卡，请点击新建转卡"
          :image-size="80"
        />
      </div>

      <div
        v-for="project in projects"
        :key="project.id"
        class="project-item"
        :class="{ 'is-selected': project.id === selectedProjectId }"
        @click="$emit('select', project.id)"
        @contextmenu.prevent="showContextMenu($event, project)"
      >
        <div class="project-item-icon">
          <el-icon>
            <Box />
          </el-icon>
        </div>
        <div class="project-item-info">
          <div class="project-item-name">
            {{ project.name }}
          </div>
          <div class="project-item-count">
            {{ project.total_cards || 0 }} 张卡片
          </div>
        </div>
        <el-dropdown
          trigger="click"
          @command="handleCommand($event, project)"
        >
          <el-button
            type=""
            link
            class="project-item-more"
            @click.stop
          >
            <el-icon>
              <MoreFilled />
            </el-icon>
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="rename">
                <el-icon>
                  <Edit />
                </el-icon>
                重命名
              </el-dropdown-item>
              <el-dropdown-item
                command="delete"
                divided
              >
                <el-icon>
                  <Delete />
                </el-icon>
                删除
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>

    <!-- 底部统计 -->
    <div class="project-list-footer">
      <span>共 {{ projects.length }} 个转卡</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ProjectWithStats } from '@/types/models'

interface Props {
  projects: ProjectWithStats[]
  selectedProjectId: string | null
  loading: boolean
}

interface Emits {
  (e: 'select', projectId: string): void
  (e: 'create'): void
  (e: 'rename', project: ProjectWithStats): void
  (e: 'delete', project: ProjectWithStats): void
}

const props = withDefaults(defineProps<Props>(), {
  projects: () => [],
  selectedProjectId: null,
  loading: false
})

const emit = defineEmits<Emits>()

const handleCommand = (command: string, project: ProjectWithStats): void => {
  if (command === 'rename') {
    emit('rename', project)
  } else if (command === 'delete') {
    emit('delete', project)
  }
}

const showContextMenu = (_event: MouseEvent, _project: ProjectWithStats): void => {
  // 右键菜单可以后续扩展
}
</script>

<style scoped>
.project-list {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.project-list-header {
  padding: 16px;
  border-bottom: 1px solid #ebeef5;
}

.project-list-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.empty-tip {
  padding: 40px 16px;
}

.project-item {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.project-item:hover {
  background-color: #f5f7fa;
}

.project-item.is-selected {
  background-color: #ecf5ff;
}

.project-item-icon {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #e6f7ff;
  border-radius: 6px;
  margin-right: 12px;
  color: #409eff;
}

.project-item-info {
  flex: 1;
  min-width: 0;
}

.project-item-name {
  font-size: 14px;
  color: #303133;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.project-item-count {
  font-size: 12px;
  color: #909399;
  margin-top: 2px;
}

.project-item-more {
  opacity: 0;
  transition: opacity 0.2s;
}

.project-item:hover .project-item-more {
  opacity: 1;
}

.project-list-footer {
  padding: 12px 16px;
  border-top: 1px solid #ebeef5;
  font-size: 12px;
  color: #909399;
  text-align: center;
}
</style>
