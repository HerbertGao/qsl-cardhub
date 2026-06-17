<template>
  <!-- 首启模式选择网关：阻断首屏、不可空手关闭，必须二选一 -->
  <div class="auth-gate">
    <div class="auth-gate-card">
      <h1 class="auth-gate-title">
        QSL 分卡助手
      </h1>
      <p class="auth-gate-sub">
        请选择使用方式（之后可随时在「数据管理」里更改）
      </p>

      <div class="auth-gate-options">
        <button
          type="button"
          class="gate-option"
          @click="select('cloud')"
        >
          <el-icon class="gate-option-icon">
            <Connection />
          </el-icon>
          <span class="gate-option-title">云同步</span>
          <span class="gate-option-desc">
            配置云端 API 与租户代码，多台设备间同步卡片数据
          </span>
        </button>

        <button
          type="button"
          class="gate-option"
          @click="select('local')"
        >
          <el-icon class="gate-option-icon">
            <Monitor />
          </el-icon>
          <span class="gate-option-title">纯本地</span>
          <span class="gate-option-desc">
            仅本机使用，通过导出/导入文件备份迁移，零云端配置
          </span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// 网关只做「分诊」：发出选择，由 App.vue 处理导航/置标志/关闭/打印机检测。
// 不内嵌云配置表单（DataTransferView 是云配置单一事实源）。
const emit = defineEmits<{ select: [choice: 'cloud' | 'local'] }>()

function select(choice: 'cloud' | 'local'): void {
  emit('select', choice)
}
</script>

<style scoped>
.auth-gate {
  position: fixed;
  inset: 0;
  z-index: 3000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #409eff 0%, #2c7be5 100%);
}

.auth-gate-card {
  width: 680px;
  max-width: calc(100vw - 48px);
  padding: 48px 40px;
  background: #fff;
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.25);
  text-align: center;
}

.auth-gate-title {
  margin: 0 0 8px;
  font-size: 28px;
  color: #303133;
}

.auth-gate-sub {
  margin: 0 0 32px;
  font-size: 14px;
  color: #909399;
}

.auth-gate-options {
  display: flex;
  gap: 20px;
}

.gate-option {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 28px 20px;
  background: #f5f7fa;
  border: 2px solid transparent;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.gate-option:hover {
  border-color: #409eff;
  background: #ecf5ff;
  transform: translateY(-2px);
}

.gate-option-icon {
  font-size: 36px;
  color: #409eff;
}

.gate-option-title {
  font-size: 18px;
  font-weight: 600;
  color: #303133;
}

.gate-option-desc {
  font-size: 13px;
  line-height: 1.5;
  color: #606266;
}
</style>
