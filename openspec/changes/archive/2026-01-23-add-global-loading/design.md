# 设计文档：全局 Loading 状态管理

## 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                        App.vue                               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                   GlobalLoading                        │  │
│  │  (全屏遮罩 + 加载动画 + 文本提示)                       │  │
│  └───────────────────────────────────────────────────────┘  │
│                            ▲                                 │
│                            │ 读取状态                        │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                   loadingStore                         │  │
│  │  - visible: boolean                                    │  │
│  │  - text: string                                        │  │
│  │  - count: number (支持嵌套)                            │  │
│  └───────────────────────────────────────────────────────┘  │
│                            ▲                                 │
│                            │ 调用                           │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                   useLoading                           │  │
│  │  - withLoading(fn, text?)                              │  │
│  │  - showLoading(text?)                                  │  │
│  │  - hideLoading()                                       │  │
│  └───────────────────────────────────────────────────────┘  │
│                            ▲                                 │
│                            │ 使用                           │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
│  │Component│  │Component│  │Component│  │Component│        │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘        │
└─────────────────────────────────────────────────────────────┘
```

## 核心模块设计

### 1. loadingStore.ts

```typescript
// 状态结构
interface LoadingState {
  visible: boolean      // 是否显示
  text: string         // 显示文本
  count: number        // 引用计数（支持嵌套调用）
}

// 导出方法
export function showLoading(text?: string): void
export function hideLoading(): void
export function setLoadingText(text: string): void
```

**设计决策：**
- 使用引用计数 `count` 支持嵌套调用场景
- 当 `count > 0` 时 `visible = true`，当 `count = 0` 时 `visible = false`
- 这样可以避免嵌套调用时提前关闭 loading

### 2. GlobalLoading.vue

**视觉设计：**
- 全屏固定定位，z-index 设为最高层级（如 9999）
- 半透明黑色背景（rgba(0, 0, 0, 0.5)）
- 居中显示 Element Plus 的 `el-loading` 样式
- 下方显示文本提示（默认"加载中..."）

**交互设计：**
- 遮罩层阻止所有点击事件（pointer-events: all）
- 支持 ESC 键不关闭（防止用户误操作）
- 使用 Vue Transition 添加淡入淡出效果

### 3. useLoading.ts

```typescript
// 主要 API
export function useLoading() {
  return {
    // 包装异步函数，自动显示/隐藏 loading
    withLoading: <T>(
      fn: () => Promise<T>,
      text?: string
    ) => Promise<T>

    // 手动控制
    showLoading: (text?: string) => void
    hideLoading: () => void
    isLoading: ComputedRef<boolean>
  }
}
```

**使用示例：**
```typescript
const { withLoading } = useLoading()

// 自动管理 loading
const result = await withLoading(
  () => invoke('sf_create_order', { params }),
  '正在创建订单...'
)

// 或手动控制
showLoading('正在加载...')
try {
  await someAsyncOperation()
} finally {
  hideLoading()
}
```

## 与现有代码的集成

### 方案 A：渐进式迁移（推荐）

保留现有的局部 loading 状态，新增全局 loading 作为补充：
- 简单操作：继续使用局部 loading（如按钮 loading）
- 复杂操作：使用全局 loading（如网络请求、批量操作）

**优点：** 无需大规模修改现有代码，风险低
**缺点：** 两套 loading 机制共存，需要开发者判断使用哪个

### 方案 B：统一替换

将所有 loading 状态统一迁移到全局 loading：
- 移除各组件的局部 loading 状态
- 全部使用 `withLoading` 包装

**优点：** 统一管理，代码简洁
**缺点：** 改动量大，可能影响现有功能

**选择方案 A**，因为它符合"渐进式采用"的设计原则。

## 推荐使用场景

| 场景 | 推荐方式 | 说明 |
|------|---------|------|
| 按钮点击后的简单操作 | 局部 loading | 使用 `:loading` 属性 |
| 网络 API 调用 | 全局 loading | 如 SF Express API、QRZ 查询 |
| 文件上传/下载 | 全局 loading | 可显示进度 |
| 批量操作 | 全局 loading | 如批量打印、批量删除 |
| 页面初始化加载 | 局部 loading | 使用 `v-loading` 指令 |

## 错误处理

`withLoading` 函数会自动处理异常：
1. 捕获异常后先调用 `hideLoading()`
2. 然后重新抛出异常，让调用者处理

```typescript
async function withLoading<T>(fn: () => Promise<T>, text?: string): Promise<T> {
  showLoading(text)
  try {
    return await fn()
  } finally {
    hideLoading()
  }
}
```

## 超时处理（可选增强）

考虑添加超时机制，防止 loading 永远不消失：
- 默认超时时间：30 秒
- 超时后自动隐藏 loading 并显示错误提示
- 可配置是否启用超时

**注意：** 此功能作为可选增强，首版可不实现。
