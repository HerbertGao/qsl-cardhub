# 提案：集成外部 SVG 图标

## 背景

当前项目使用 Element Plus 内置的图标库 `@element-plus/icons-vue`，该图标库提供了约 280 个常用图标。但在实际开发中，有些业务场景需要的图标（如快递物流图标、业余无线电相关图标等）并不在内置库中，需要使用外部 SVG 图标。

## 问题分析

1. **图标不足** - Element Plus 图标库有限，无法覆盖所有业务场景
2. **手动管理** - 目前没有统一的外部图标管理方案
3. **一致性** - 外部图标需要与 Element Plus 图标保持一致的使用方式

## 解决方案

使用 `unplugin-icons` 插件集成外部图标，支持：

1. **Iconify 图标库** - 访问 100+ 图标集，超过 10,000 个图标
2. **自定义本地图标** - 加载项目内的 SVG 文件
3. **按需加载** - 只打包实际使用的图标

## 推荐图标集

| 图标集 | 说明 | 适用场景 |
|--------|------|---------|
| `mdi` (Material Design Icons) | 7000+ 图标 | 通用 UI |
| `carbon` | IBM Carbon 设计系统 | 企业应用 |
| `tabler` | 4500+ 图标 | 通用 UI |
| `heroicons` | Tailwind 官方图标 | 现代 Web |
| `lucide` | Feather 图标的扩展 | 简洁风格 |

## 技术方案

### 方案 A：使用 unplugin-icons（推荐）

**优点：**
- 按需加载，不增加包体积
- 支持 Iconify 海量图标
- 支持自定义本地 SVG 图标
- 与 Vue 3 深度集成
- 可配置自动导入

**缺点：**
- 需要添加开发依赖

### 方案 B：手动导入 SVG 组件

**优点：**
- 无需额外依赖
- 完全控制

**缺点：**
- 需要手动管理每个图标
- 无法享受按需加载
- 维护成本高

**选择方案 A**

## 影响范围

- **新增依赖**
  - `unplugin-icons` - 图标插件
  - `@iconify/json` - Iconify 图标数据（可选，用于离线使用）

- **修改文件**
  - `web/vite.config.ts` - 添加插件配置
  - `web/tsconfig.json` - 添加类型声明（可选）

- **新增目录**
  - `web/src/assets/icons/` - 存放自定义 SVG 图标

## 使用示例

```vue
<template>
  <!-- 使用 Iconify 图标 -->
  <i-mdi-truck />
  <i-carbon-radio />

  <!-- 使用自定义图标 -->
  <i-custom-my-icon />

  <!-- 与 el-icon 组合 -->
  <el-icon :size="20">
    <i-mdi-truck />
  </el-icon>
</template>
```

## 预期效果

- 可使用海量 Iconify 图标
- 可加载自定义 SVG 图标
- 图标按需加载，不影响包体积
- 与现有 Element Plus 图标使用方式兼容
