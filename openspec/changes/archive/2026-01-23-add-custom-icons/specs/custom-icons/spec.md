# 规范：外部 SVG 图标集成

## 新增需求

### 需求：Iconify 图标支持

系统**必须**支持使用 Iconify 图标库中的图标，通过 `unplugin-icons` 插件实现按需加载。

#### 场景：使用 Iconify 图标

**给定** 已配置 unplugin-icons 插件
**当** 在 Vue 组件中使用 `<i-mdi-truck />` 时
**则** 应该正确显示 Material Design Icons 的 truck 图标
**并且** 图标应该支持 CSS 样式（如 font-size、color）

#### 场景：图标按需加载

**给定** 组件中只使用了 `i-mdi-truck` 和 `i-mdi-home` 两个图标
**当** 构建生产版本时
**则** 只有这两个图标的数据被打包
**并且** 未使用的图标不会影响包体积

---

### 需求：自定义 SVG 图标支持

系统**必须**支持加载项目内的自定义 SVG 图标文件。

#### 场景：加载自定义图标

**给定** 在 `src/assets/icons/` 目录下有 `my-icon.svg` 文件
**当** 在 Vue 组件中使用 `<i-custom-my-icon />` 时
**则** 应该正确显示该 SVG 图标

#### 场景：自定义图标颜色继承

**给定** 自定义 SVG 图标使用了 `fill="currentColor"`
**当** 设置父元素的 `color` CSS 属性时
**则** 图标颜色应该随之改变

---

### 需求：与 Element Plus 兼容

系统**必须**保证 unplugin-icons 图标与 Element Plus 的 `el-icon` 组件兼容。

#### 场景：在 el-icon 中使用

**给定** 使用 `<el-icon :size="20"><i-mdi-truck /></el-icon>` 时
**则** 图标应该正确显示
**并且** el-icon 的 size 属性应该生效

#### 场景：独立使用

**给定** 直接使用 `<i-mdi-truck style="font-size: 24px" />` 时
**则** 图标应该正确显示
**并且** 图标大小应该为 24px
