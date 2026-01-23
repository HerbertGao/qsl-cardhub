<template>
  <Transition name="fade">
    <div
      v-if="loadingState.visible"
      class="global-loading-overlay"
    >
      <div class="global-loading-content">
        <div class="global-loading-spinner">
          <svg
            class="circular"
            viewBox="25 25 50 50"
          >
            <circle
              class="path"
              cx="50"
              cy="50"
              r="20"
              fill="none"
            />
          </svg>
        </div>
        <p class="global-loading-text">
          {{ loadingState.text }}
        </p>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { loadingState } from '@/stores/loadingStore'
</script>

<style scoped>
.global-loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  z-index: 9999;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: all;
}

.global-loading-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.global-loading-spinner {
  width: 42px;
  height: 42px;
}

.circular {
  animation: loading-rotate 2s linear infinite;
  width: 100%;
  height: 100%;
}

.path {
  stroke: #409eff;
  stroke-width: 4;
  stroke-linecap: round;
  animation: loading-dash 1.5s ease-in-out infinite;
}

.global-loading-text {
  color: #fff;
  font-size: 14px;
  margin: 0;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

/* 过渡动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* Element Plus 风格的加载动画 */
@keyframes loading-rotate {
  100% {
    transform: rotate(360deg);
  }
}

@keyframes loading-dash {
  0% {
    stroke-dasharray: 1, 200;
    stroke-dashoffset: 0;
  }
  50% {
    stroke-dasharray: 90, 150;
    stroke-dashoffset: -40px;
  }
  100% {
    stroke-dasharray: 90, 150;
    stroke-dashoffset: -120px;
  }
}
</style>
