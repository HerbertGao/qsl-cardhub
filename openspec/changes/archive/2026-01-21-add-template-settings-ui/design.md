# 模板设置功能 - 技术设计

## 上下文

当前系统的模板配置存储在 `config/templates/default.toml`，用户需要手动编辑 TOML 文件来调整模板参数。这对普通用户不友好，且容易因格式错误导致系统无法加载模板。

本变更旨在提供一个可视化的模板设置界面，让用户通过表单编辑模板参数，并实时预览效果。

## 目标 / 非目标

### 目标
- 提供可视化的模板参数编辑界面
- 支持高频参数（边距、对齐、间距）的便捷修改
- 实现实时预览功能，所见即所得
- 自动保存修改到模板文件
- 保持与现有模板系统的兼容性

### 非目标
- **不支持**多模板管理（v1 仅支持修改默认模板）
- **不支持**添加/删除元素（元素列表固定）
- **不支持**修改元素的类型、来源等结构性字段
- **不支持**模板导入/导出功能

## 决策

### 决策 1：使用现有 TemplateConfig 结构，不引入新的数据模型

**理由：**
- 复用现有的 `TemplateConfig`、`PageConfig`、`LayoutConfig` 等结构
- 前后端使用相同的数据模型，减少映射和转换
- 利用现有的 TOML 序列化/反序列化逻辑

**实现：**
```rust
// 新增 Tauri 命令
#[tauri::command]
pub async fn get_template_config() -> Result<TemplateConfig, String> {
    // 读取 config/templates/default.toml
    TemplateConfig::load_from_file(get_default_template_path())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_template_config(config: TemplateConfig) -> Result<(), String> {
    // 序列化为 TOML 并写入文件
    let toml_str = toml::to_string_pretty(&config)
        .map_err(|e| format!("序列化失败: {}", e))?;
    std::fs::write(get_default_template_path(), toml_str)
        .map_err(|e| format!("写入文件失败: {}", e))
}
```

### 决策 2：字段级权限控制在前端实现

**理由：**
- 后端只负责读写完整配置，不关心哪些字段可编辑
- 前端根据业务规则控制字段的 `readonly` 或 `disabled` 属性
- 灵活性高，易于调整可编辑字段列表

**可编辑字段列表：**
- `page.margin_left_mm`, `page.margin_right_mm`, `page.margin_top_mm`, `page.margin_bottom_mm`
- `page.border`, `page.border_thickness_mm`
- `layout.align_h`, `layout.align_v`, `layout.gap_mm`, `layout.line_gap_mm`
- `elements[].max_height_mm`（仅 text 类型元素）

**只读字段列表：**
- `metadata.*`（所有元数据）
- `page.dpi`, `page.width_mm`, `page.height_mm`（纸张规格固定）
- `fonts.*`（字体配置固定）
- `elements[].id`, `elements[].type`, `elements[].source`（结构性字段）
- `elements[].value`, `elements[].key`, `elements[].format`（内容定义）
- `elements[barcode].height_mm`, `elements[barcode].quiet_zone_mm`, `elements[barcode].human_readable`
- `output.*`（输出配置固定）

### 决策 3：自动保存 + 防抖，避免手动保存按钮

**理由：**
- 用户体验更流畅，无需记得点击"保存"
- 使用防抖（debounce 500ms）避免频繁写文件
- 显示保存状态指示，让用户知道已保存

**考虑的替代方案：**
- ❌ 手动保存按钮：用户可能忘记保存，导致修改丢失
- ❌ 提交时验证：延迟反馈，用户体验差

**实现：**
```javascript
// 使用 Vue 的 watch + debounce
import { debounce } from 'lodash-es'

const debouncedSave = debounce(async () => {
  try {
    await invoke('save_template_config', { config: templateConfig.value })
    ElMessage.success('配置已保存')
  } catch (error) {
    ElMessage.error('保存失败: ' + error)
  }
}, 500)

watch(templateConfig, () => {
  debouncedSave()
}, { deep: true })
```

### 决策 4：预览按钮手动触发，不自动刷新

**理由：**
- 预览生成涉及复杂的渲染流程，性能开销较大
- 自动预览可能导致频繁渲染，影响用户编辑体验
- 手动触发给用户更多控制权

**考虑的替代方案：**
- ❌ 自动预览：性能影响大，用户体验差
- ❌ 保存后预览：耦合保存和预览逻辑

**实现：**
```javascript
const handleRefreshPreview = async () => {
  loading.value = true
  try {
    const response = await invoke('preview_qsl', {
      request: {
        template_path: null,  // 使用当前模板
        data: {
          task_name: '预览测试',
          callsign: 'BG7XXX',
          sn: '001',
          qty: '100'
        },
        output_config: {
          mode: 'text_bitmap_plus_native_barcode',
          threshold: 160
        }
      }
    })
    previewImageUrl.value = response.png_path
  } catch (error) {
    ElMessage.error('预览失败: ' + error)
  } finally {
    loading.value = false
  }
}
```

### 决策 5：左右分栏布局，左侧表单，右侧预览

**理由：**
- 符合用户从左到右的阅读习惯
- 方便用户边编辑边查看效果
- 预览图垂直放置，利用屏幕高度

**布局比例：**
- 左侧表单：60% 宽度（固定或可调整）
- 右侧预览：40% 宽度
- 使用 Element Plus 的 `el-row` 和 `el-col` 实现响应式布局

**实现：**
```vue
<template>
  <div class="page-content">
    <h1>模板设置</h1>
    <el-row :gutter="20" style="margin-top: 30px">
      <!-- 左侧表单 -->
      <el-col :span="14">
        <el-form :model="templateConfig" label-width="150px">
          <!-- 表单字段 -->
        </el-form>
      </el-col>

      <!-- 右侧预览 -->
      <el-col :span="10">
        <el-card shadow="hover">
          <template #header>
            <div style="display: flex; justify-content: space-between">
              <span>预览</span>
              <el-button size="small" @click="handleRefreshPreview">
                刷新预览
              </el-button>
            </div>
          </template>
          <div v-loading="loading">
            <img :src="previewImageUrl" style="width: 100%" />
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>
```

## 风险 / 权衡

### 风险 1：文件写入失败
- **风险**：用户没有文件写入权限，或文件被占用
- **缓解措施**：
  - 捕获文件写入错误并显示友好提示
  - 建议用户检查文件权限或重启应用
  - 记录详细错误日志供调试

### 风险 2：无效配置导致打印失败
- **风险**：用户设置了不合理的参数（如负数边距），导致布局异常
- **缓解措施**：
  - 前端添加表单验证，限制输入范围
  - 边距 >= 0，间距 >= 0
  - 提供参数说明和推荐值

### 风险 3：预览与实际打印不一致
- **风险**：预览使用的是 PDF 后端，实际打印使用 TSPL，可能存在细微差异
- **缓解措施**：
  - 在预览区域添加说明："预览仅供参考，实际打印可能有细微差异"
  - 鼓励用户打印测试页验证效果

### 权衡 1：自动保存 vs 手动保存
- **选择**：自动保存 + 防抖
- **权衡**：可能导致意外覆盖，但提升了用户体验
- **建议**：后续版本可考虑添加"撤销"功能或版本历史

### 权衡 2：所有字段展示 vs 只展示可编辑字段
- **选择**：所有字段展示（部分只读）
- **权衡**：界面可能较复杂，但用户能全面了解模板配置
- **建议**：使用折叠面板组织，默认展开高频配置组

## 迁移计划

本功能为纯新增，无需数据迁移。

**部署步骤：**
1. 部署后端代码（新增 Tauri 命令）
2. 部署前端代码（新增页面和路由）
3. 验证功能正常
4. 通知用户新功能可用

**回滚策略：**
- 如果新功能有问题，可直接删除路由和菜单入口
- 用户仍可手动编辑 `default.toml` 文件
- 不影响现有打印功能

## 待决问题

1. **是否需要"重置为默认"功能？**
   - 让用户一键恢复到系统默认配置
   - 建议：后续版本添加

2. **是否需要导出/导入模板？**
   - 方便用户备份和分享配置
   - 建议：后续版本添加

3. **是否需要配置预设（Presets）？**
   - 提供几套预设配置（紧凑型、宽松型等）
   - 建议：v2.0 考虑
