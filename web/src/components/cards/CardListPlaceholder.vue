<template>
  <div class="card-list-placeholder">
    <el-empty
      :description="displayMessage"
      :image-size="120"
    >
      <template #image>
        <el-icon
          :size="80"
          color="#c0c4cc"
        >
          <Box />
        </el-icon>
      </template>
      <template
        v-if="projectName"
        #description
      >
        <p class="placeholder-project">
          项目：{{ projectName }}
        </p>
        <p class="placeholder-message">
          {{ message }}
        </p>
      </template>
    </el-empty>
  </div>
</template>

<script setup lang="ts">
import {computed} from 'vue'

const props = defineProps({
  message: {
    type: String,
    default: '卡片列表功能将在 Phase 2 实现'
  },
  projectName: {
    type: String,
    default: null
  }
})

const displayMessage = computed(() => {
  if (props.projectName) {
    return ''
  }
  return props.message
})
</script>

<style scoped>
.card-list-placeholder {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #fafafa;
}

.placeholder-project {
  font-size: 14px;
  color: #606266;
  margin-bottom: 8px;
}

.placeholder-message {
  font-size: 13px;
  color: #909399;
}
</style>
