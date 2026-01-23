# 设计文档：集成外部 SVG 图标

## 技术实现

### 1. unplugin-icons 配置

在 `vite.config.ts` 中添加配置：

```typescript
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import Icons from 'unplugin-icons/vite'
import { FileSystemIconLoader } from 'unplugin-icons/loaders'

export default defineConfig({
  plugins: [
    vue(),
    Icons({
      compiler: 'vue3',
      // 自定义图标集
      customCollections: {
        // 从 src/assets/icons/ 加载自定义图标
        'custom': FileSystemIconLoader(
          './src/assets/icons',
          svg => svg.replace(/^<svg /, '<svg fill="currentColor" ')
        ),
      },
    }),
  ],
})
```

### 2. 图标命名规则

使用 unplugin-icons 时，图标组件名遵循以下规则：

```
i-{collection}-{icon-name}
```

例如：
- `i-mdi-truck` → Material Design Icons 的 truck 图标
- `i-carbon-radio` → Carbon 的 radio 图标
- `i-custom-qsl-card` → 自定义的 qsl-card.svg 图标

### 3. 自定义图标目录结构

```
web/src/assets/icons/
├── qsl-card.svg
├── antenna.svg
├── radio-wave.svg
└── ...
```

### 4. SVG 图标规范

自定义 SVG 图标应遵循以下规范：

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
  <!-- 使用 currentColor 以支持颜色继承 -->
  <path fill="currentColor" d="..." />
</svg>
```

**要点：**
- 不设置固定的 `width` 和 `height`（由插件自动处理）
- 使用 `fill="currentColor"` 或 `stroke="currentColor"` 以支持颜色继承
- 使用 `viewBox` 定义视口

### 5. 类型支持（可选）

在 `env.d.ts` 或单独的声明文件中添加：

```typescript
declare module '~icons/*' {
  import type { FunctionalComponent, SVGAttributes } from 'vue'
  const component: FunctionalComponent<SVGAttributes>
  export default component
}
```

### 6. 与 Element Plus el-icon 配合

unplugin-icons 生成的组件可以与 `el-icon` 配合使用：

```vue
<template>
  <el-icon :size="20" :color="color">
    <i-mdi-truck />
  </el-icon>
</template>
```

或直接使用（推荐）：

```vue
<template>
  <i-mdi-truck style="font-size: 20px; color: #409eff" />
</template>
```

## 自动导入配置（可选增强）

如果项目后续引入 `unplugin-auto-import`，可以配置图标自动导入：

```typescript
import AutoImport from 'unplugin-auto-import/vite'
import IconsResolver from 'unplugin-icons/resolver'

AutoImport({
  resolvers: [
    IconsResolver({
      prefix: 'i',
      customCollections: ['custom'],
    }),
  ],
})
```

**注意：** 首版可以不实现自动导入，手动导入即可满足需求。

## 常用图标集推荐

| 图标集前缀 | 名称 | 图标数量 | 风格 |
|-----------|------|---------|------|
| `mdi` | Material Design Icons | 7000+ | 填充 |
| `mdi-light` | MDI Light | 260+ | 线条 |
| `carbon` | Carbon | 2000+ | IBM 风格 |
| `tabler` | Tabler | 4500+ | 线条 |
| `heroicons-outline` | Heroicons Outline | 290+ | 线条 |
| `heroicons-solid` | Heroicons Solid | 290+ | 填充 |
| `lucide` | Lucide | 1400+ | 线条 |
| `ph` | Phosphor | 6000+ | 多风格 |

## 按需安装

默认情况下，unplugin-icons 会从网络获取图标数据。如果需要离线使用，可以安装完整的图标数据包：

```bash
# 安装所有图标（较大，约 100MB）
npm install @iconify/json

# 或只安装需要的图标集
npm install @iconify-json/mdi @iconify-json/carbon
```

## 性能考虑

- **按需加载** - 只有在组件中使用的图标才会被打包
- **Tree-shaking** - 未使用的图标不会影响最终包体积
- **缓存** - Vite 会缓存图标数据，避免重复下载
