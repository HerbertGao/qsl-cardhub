# 规范：全局 Loading 状态管理

## 新增需求

### 需求：全局 Loading Store

系统**必须**提供全局 Loading 状态管理，支持在异步操作期间显示加载状态。

#### 场景：显示和隐藏 Loading

**给定** 用户在界面上
**当** 调用 `showLoading('加载中...')` 时
**则** 全局 Loading 遮罩层应该显示
**并且** 遮罩层应该显示文本"加载中..."
**并且** 遮罩层应该阻止用户点击其他元素

**当** 调用 `hideLoading()` 时
**则** 全局 Loading 遮罩层应该隐藏

#### 场景：嵌套调用

**给定** 已调用一次 `showLoading()`
**当** 再次调用 `showLoading()` 时
**则** Loading 遮罩层应该保持显示

**当** 调用一次 `hideLoading()` 时
**则** Loading 遮罩层应该保持显示（因为还有一个未关闭）

**当** 再调用一次 `hideLoading()` 时
**则** Loading 遮罩层应该隐藏

#### 场景：使用 withLoading 包装异步操作

**给定** 有一个异步函数 `asyncFn`
**当** 使用 `await withLoading(asyncFn, '处理中...')` 调用时
**则** Loading 遮罩层应该在调用开始时显示
**并且** 显示文本"处理中..."
**当** 异步函数完成时
**则** Loading 遮罩层应该自动隐藏

#### 场景：异步操作抛出异常

**给定** 有一个会抛出异常的异步函数 `asyncFn`
**当** 使用 `await withLoading(asyncFn)` 调用时
**并且** 异步函数抛出异常
**则** Loading 遮罩层应该自动隐藏
**并且** 异常应该继续向上抛出

---

### 需求：全局 Loading 组件

系统**必须**提供全局 Loading 组件，在 Loading 状态激活时显示遮罩层和加载动画。

#### 场景：视觉呈现

**给定** Loading 状态为显示
**则** 应该显示全屏半透明遮罩层
**并且** 遮罩层居中显示加载动画
**并且** 加载动画下方显示提示文本
**并且** 遮罩层应该在所有内容之上（z-index 最高）

#### 场景：过渡动画

**当** Loading 从隐藏变为显示时
**则** 应该有淡入动画效果

**当** Loading 从显示变为隐藏时
**则** 应该有淡出动画效果

#### 场景：阻止用户交互

**给定** Loading 状态为显示
**当** 用户点击遮罩层下方的任何元素时
**则** 点击事件应该被阻止
**并且** 不应触发任何操作

---

### 需求：Loading Composable

系统**必须**提供 `useLoading` composable，方便组件使用 Loading 功能。

#### 场景：使用 composable

**给定** 在 Vue 组件中
**当** 调用 `const { withLoading, showLoading, hideLoading, isLoading } = useLoading()`
**则** 应该返回 Loading 相关的方法和状态
**并且** `isLoading` 应该是一个响应式计算属性

#### 场景：检查 Loading 状态

**给定** 已调用 `showLoading()`
**当** 访问 `isLoading.value` 时
**则** 应该返回 `true`

**给定** 已调用 `hideLoading()`
**当** 访问 `isLoading.value` 时
**则** 应该返回 `false`
