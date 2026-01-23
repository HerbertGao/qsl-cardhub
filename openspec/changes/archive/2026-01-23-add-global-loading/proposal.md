# 提案：添加全局 Loading 状态管理

## 背景

当前前端调用 Rust Tauri 命令时，由于网络操作（如顺丰 API 调用、QRZ 查询等）导致响应较慢时，系统会短暂失去响应。用户在等待期间可能会反复点击按钮，导致重复请求或不良用户体验。

## 问题分析

1. **用户体验问题**
   - 网络请求期间没有全局反馈，用户不确定系统是否在工作
   - 按钮虽有局部 loading 状态，但用户仍可能点击其他操作
   - 长时间等待时界面看起来像是卡住了

2. **技术问题**
   - 各组件独立管理 loading 状态，没有统一的加载状态管理
   - 缺少全局遮罩层阻止用户误操作
   - 没有请求防抖/节流机制

## 解决方案

### 1. 全局 Loading Store

创建 `loadingStore.ts`，提供统一的加载状态管理：
- 支持开启/关闭全局 loading
- 支持自定义 loading 文本
- 支持多重 loading 计数（嵌套调用场景）

### 2. 全局 Loading 组件

创建 `GlobalLoading.vue` 组件：
- 全屏半透明遮罩层
- 居中显示加载动画和文本
- 阻止用户点击其他元素

### 3. 异步操作包装器

创建 `useLoading` composable：
- 自动包装异步操作
- 自动显示/隐藏 loading
- 支持错误处理

## 影响范围

- **新增文件**
  - `web/src/stores/loadingStore.ts` - Loading 状态管理
  - `web/src/components/common/GlobalLoading.vue` - 全局 Loading 组件
  - `web/src/composables/useLoading.ts` - Loading composable

- **修改文件**
  - `web/src/App.vue` - 集成全局 Loading 组件
  - 可选：现有组件可逐步迁移使用新的 loading 机制

## 设计原则

1. **渐进式采用** - 新的全局 loading 机制不强制替换现有的局部 loading，可以共存
2. **简单易用** - 提供简洁的 API，一行代码即可启用
3. **可配置** - 支持自定义 loading 文本、超时时间等
4. **非侵入式** - 不需要大规模修改现有代码

## 预期效果

- 网络请求期间显示全局遮罩，明确告知用户系统正在处理
- 阻止用户在等待期间的误操作
- 提供统一的 loading 管理机制，方便后续维护
