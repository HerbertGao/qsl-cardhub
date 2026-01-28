# barcode-image-fallback Specification

## Purpose
TBD - created by archiving change fix-barcode-printer-compatibility. Update Purpose after archive.
## 需求
### 需求：模板配置读取接口
默认模板的 `output.mode` 必须从 `text_bitmap_plus_native_barcode` 改为 `full_bitmap`，以确保兼容不支持 TSPL 原生 BARCODE 指令的打印机。

#### 场景：新安装默认模板使用全位图模式
- **当** 用户首次安装应用或重置后初始化默认模板
- **那么** 呼号标签模板 (`callsign.toml`) 的 `output.mode` 必须为 `full_bitmap`
- **并且** 地址标签模板 (`address.toml`) 的 `output.mode` 必须为 `full_bitmap`
- **并且** 硬编码的默认 `OutputConfig` 中 `mode` 必须为 `full_bitmap`

#### 场景：已有用户模板不受影响
- **当** 用户已有模板文件存在
- **那么** 系统不得修改已有模板的 `output.mode` 值
- **并且** 用户可通过模板设置界面手动切换渲染模式

### 需求：模板设置用户界面
模板设置界面的"输出"配置组中，`output.mode` 字段必须从只读改为可编辑的下拉选择框。

#### 场景：显示渲染模式选择器
- **当** 用户打开模板设置页面的"输出"配置组
- **那么** `output.mode` 字段必须显示为下拉选择框
- **并且** 选项包含：
  - `full_bitmap`：标签"全位图模式"，说明"所有元素渲染为图片打印，兼容性最好（推荐）"
  - `text_bitmap_plus_native_barcode`：标签"混合模式"，说明"条形码使用打印机原生指令，需要打印机支持 TSPL BARCODE 指令"
- **并且** 修改后触发自动保存和预览刷新

#### 场景：渲染模式切换立即生效
- **当** 用户切换渲染模式
- **那么** 配置变更必须通过现有的自动保存机制持久化
- **并且** 下次打印或预览必须使用新的渲染模式

