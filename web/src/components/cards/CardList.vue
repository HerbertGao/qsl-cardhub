<template>
  <div class="card-list">
    <!-- 工具栏 -->
    <div class="card-list-toolbar">
      <div class="toolbar-left">
        <el-button
          type="primary"
          @click="$emit('add')"
        >
          <el-icon>
            <Plus />
          </el-icon>
          <span>录入卡片</span>
        </el-button>
      </div>
      <div class="toolbar-right">
        <el-input
          v-model="searchKeyword"
          placeholder="搜索呼号"
          clearable
          style="width: 200px"
          @input="handleSearch"
        >
          <template #prefix>
            <el-icon>
              <Search />
            </el-icon>
          </template>
        </el-input>
        <el-select
          v-model="statusFilter"
          placeholder="状态筛选"
          clearable
          style="width: 120px; margin-left: 12px"
          @change="handleFilterChange"
        >
          <el-option
            label="全部"
            value=""
          />
          <el-option
            label="待分发"
            value="pending"
          />
          <el-option
            label="已分发"
            value="distributed"
          />
          <el-option
            label="已退回"
            value="returned"
          />
        </el-select>
      </div>
    </div>

    <!-- 表格 -->
    <el-table
      v-loading="loading"
      :data="cards"
      stripe
      style="width: 100%"
      height="calc(100% - 120px)"
    >
      <el-table-column
        type="index"
        label="序号"
        width="60"
        align="center"
        :index="indexMethod"
      />
      <el-table-column
        prop="callsign"
        label="呼号"
        width="120"
        align="center"
      />
      <el-table-column
        prop="qty"
        label="数量"
        width="80"
        align="center"
      />
      <el-table-column
        label="最终状态"
        width="100"
        align="center"
      >
        <template #default="{ row }">
          <el-tag
            :type="getStatusType(row.status)"
            size="small"
          >
            {{ getStatusLabel(row.status) }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column
        label="分发/退回时间"
        width="180"
        align="center"
      >
        <template #default="{ row }">
          {{ getProcessTime(row) }}
        </template>
      </el-table-column>
      <el-table-column
        prop="created_at"
        label="录入时间"
        width="180"
        align="center"
      >
        <template #default="{ row }">
          {{ formatDateTime(row.created_at) }}
        </template>
      </el-table-column>
      <el-table-column
        label="操作"
        width="120"
        align="center"
        fixed="right"
      >
        <template #default="{ row }">
          <div class="action-buttons">
            <el-button
              type="primary"
              link
              size="small"
              @click="$emit('view', row)"
            >
              查看
            </el-button>
            <el-dropdown
              trigger="click"
              @command="(cmd: string) => handleRowCommand(cmd, row)"
            >
              <el-button
                type="info"
                link
                size="small"
              >
                更多
                <el-icon class="el-icon--right">
                  <ArrowDown />
                </el-icon>
              </el-button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="distribute">
                    <el-icon>
                      <Promotion />
                    </el-icon>
                    分发
                  </el-dropdown-item>
                  <el-dropdown-item command="return">
                    <el-icon>
                      <RefreshLeft />
                    </el-icon>
                    退回
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
        </template>
      </el-table-column>
    </el-table>

    <!-- 分页 -->
    <div class="card-list-pagination">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :total="total"
        :page-sizes="[20, 50, 100]"
        layout="total, sizes, prev, pager, next, jumper"
        @size-change="handleSizeChange"
        @current-change="handleCurrentChange"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import type { CardWithProject, CardStatus } from '@/types/models'

interface Props {
  cards: CardWithProject[]
  total: number
  page: number
  pageSize: number
  loading: boolean
}

const props = withDefaults(defineProps<Props>(), {
  cards: () => [],
  total: 0,
  page: 1,
  pageSize: 20,
  loading: false
})

interface Emits {
  (e: 'add'): void
  (e: 'view', card: CardWithProject): void
  (e: 'distribute', card: CardWithProject): void
  (e: 'return', card: CardWithProject): void
  (e: 'delete', card: CardWithProject): void
  (e: 'search', keyword: string): void
  (e: 'filter', status: string): void
  (e: 'page-change', data: { page: number; pageSize: number }): void
}

const emit = defineEmits<Emits>()

// 搜索关键词
const searchKeyword = ref<string>('')
// 状态筛选
const statusFilter = ref<string>('')
// 当前页码
const currentPage = ref<number>(props.page)
// 每页条数
const pageSize = ref<number>(props.pageSize)

// 防抖计时器
let searchTimer: ReturnType<typeof setTimeout> | null = null

// 搜索处理（防抖）
const handleSearch = (): void => {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    emit('search', searchKeyword.value)
  }, 300)
}

// 状态筛选处理
const handleFilterChange = (): void => {
  emit('filter', statusFilter.value)
}

// 行操作命令处理
const handleRowCommand = (command: string, row: CardWithProject): void => {
  if (command === 'distribute') {
    emit('distribute', row)
  } else if (command === 'return') {
    emit('return', row)
  } else if (command === 'delete') {
    emit('delete', row)
  }
}

// 分页大小变化
const handleSizeChange = (size: number): void => {
  emit('page-change', { page: 1, pageSize: size })
}

// 页码变化
const handleCurrentChange = (page: number): void => {
  emit('page-change', { page, pageSize: pageSize.value })
}

// 序号计算
const indexMethod = (index: number): number => {
  return (currentPage.value - 1) * pageSize.value + index + 1
}

// 获取状态标签类型
const getStatusType = (status: CardStatus): 'info' | 'success' | 'warning' => {
  const types: Record<CardStatus, 'info' | 'success' | 'warning'> = {
    pending: 'info',
    distributed: 'success',
    returned: 'warning'
  }
  return types[status] || 'info'
}

// 获取状态标签文本
const getStatusLabel = (status: CardStatus): string => {
  const labels: Record<CardStatus, string> = {
    pending: '待分发',
    distributed: '已分发',
    returned: '已退回'
  }
  return labels[status] || status
}

// 格式化时间
const formatDateTime = (datetime: string | null | undefined): string => {
  if (!datetime) return '-'
  // 转换 ISO 8601 格式为本地时间显示
  const date = new Date(datetime)
  return date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

// 获取分发/退回时间（根据最终状态选择）
const getProcessTime = (row: CardWithProject): string => {
  if (row.status === 'distributed' && row.metadata?.distribution?.distributed_at) {
    return formatDateTime(row.metadata.distribution.distributed_at)
  }
  if (row.status === 'returned' && row.metadata?.return?.returned_at) {
    return formatDateTime(row.metadata.return.returned_at)
  }
  return '-'
}

// 监听 props 变化同步状态
watch(() => props.page, (val: number): void => {
  currentPage.value = val
})

watch(() => props.pageSize, (val: number): void => {
  pageSize.value = val
})
</script>

<style scoped>
.card-list {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 20px;
}

.card-list-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.toolbar-left {
  display: flex;
  align-items: center;
}

.toolbar-right {
  display: flex;
  align-items: center;
}

.card-list-pagination {
  display: flex;
  justify-content: flex-end;
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid #ebeef5;
}

.action-buttons {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
}
</style>
