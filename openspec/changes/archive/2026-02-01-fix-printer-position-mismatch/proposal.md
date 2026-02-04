# 修复：HPRT N31D 打印机打印位置与预览不一致

> **类型**: Bug 修复 / 配置调整
> **影响范围**: 打印功能
> **需要规范增量**: 否（不涉及新功能，仅为现有行为的修复）

## 问题描述

用户报告使用 HPRT N31D 打印机打印标签时，实际打印位置与模板预览位置不一致。

## 日志分析

根据 `logs/test.qsl-logs-2026-02-01T10-07-09.txt` 日志文件分析：

### 基本信息
- **打印机**: HPRT N31D
- **模板**: 76mm × 130mm 标准模板
- **渲染模式**: `text_bitmap_plus_native_barcode`（混合模式）
- **画布尺寸**: 608x1039 dots

### 位图位置信息
```
位图[0]: 527x48 at (40, 35)   - 标题
位图[1]: 528x80 at (40, 123)  - 项目名
位图[2]: 369x120 at (119, 243) - 呼号
位图[3]: 390x120 at (109, 667) - SN
位图[4]: 342x120 at (133, 827) - QTY
```

### TSPL 指令参数
```
SIZE 76 mm, 130 mm
GAP 2 mm, 0 mm
DIRECTION 1
```

## 问题根因分析

经过代码审查，发现以下潜在问题：

### 1. 预览和打印使用不同的条码渲染方式

**问题**: 在混合模式下，预览（PDF 后端）和打印（TSPL 生成器）对条码的处理方式不同：

- **预览（pdf.rs:160-176）**: 条码使用 `BarcodeRenderer` 渲染为位图，然后叠加到画布
- **打印（tspl.rs:79-89）**: 条码使用原生 TSPL `BARCODE` 指令

**影响**: 原生 TSPL `BARCODE` 指令的条码宽度计算与位图渲染的条码宽度可能存在差异，导致整体布局偏移。

### 2. TSPL BARCODE 指令宽度估算不准确

**代码位置**: `layout_engine.rs:395-399`

```rust
let char_count = element.content.len() as u32;
let narrow_width = 2; // TSPL BARCODE 命令的窄条宽度参数
let estimated_width = (char_count + 3) * 11 * narrow_width + quiet_zone_dots * 2;
```

**问题**:
- Code128 条码宽度估算公式可能与 TSPL 打印机实际渲染的条码宽度不一致
- TSPL BARCODE 的 `narrow` 和 `wide` 参数（当前设置为 2,2）可能导致不同的实际宽度

### 3. TSPL BARCODE 指令参数

**代码位置**: `tspl.rs:289-292`

```rust
BARCODE {x},{y},"{type}",{height},{readable},0,2,2,"{content}"
```

**参数含义**:
- `narrow=2, wide=2`: 窄条和宽条的宽度

**问题**: 当 `narrow` 和 `wide` 都设置为 2 时，条码的实际渲染宽度可能与位图预览不一致。

### 4. GAP 参数设置

**代码位置**: `tspl.rs:49`

```rust
tspl.extend_from_slice(b"GAP 2 mm, 0 mm\r\n");
```

**问题**: GAP 参数固定为 2mm，但实际标签纸的间隙可能不同，导致垂直位置偏移。

### 5. REFERENCE 参数缺失

**问题**: TSPL 中没有设置 `REFERENCE` 指令来调整打印起始位置。某些打印机可能需要 `REFERENCE x,y` 来校正打印原点偏移。

## 建议解决方案

### 短期方案

1. **统一条码渲染模式**: 将默认模板的 `output.mode` 从 `text_bitmap_plus_native_barcode` 改为 `full_bitmap`，确保预览和打印使用完全相同的位图数据。

2. **添加 REFERENCE 支持**: 在模板配置中添加 `reference_x` 和 `reference_y` 参数，允许用户微调打印起始位置。

### 长期方案

1. **改进条码宽度计算**: 精确计算 TSPL BARCODE 指令的实际渲染宽度，使布局计算更准确。

2. **添加打印机校准功能**: 提供打印测试页功能，让用户测量并配置打印偏移量。

3. **GAP 参数可配置**: 将 GAP 参数移到模板配置中，支持不同标签纸规格。

## 验证步骤

1. 将模板的 `output.mode` 改为 `full_bitmap`
2. 重新打印测试，比较预览和实际打印结果
3. 如仍有偏移，添加 REFERENCE 参数进行微调

## 额外发现：模板配置保存后不生效的问题

### 问题描述

用户在模板配置页面将 `output.mode` 改为 `full_bitmap` 并保存成功，但打印时仍然使用 `text_bitmap_plus_native_barcode` 模式。

### 根因分析

前端在调用 `print_qsl` 时，`output_config` 是**硬编码**的，而不是从保存的模板配置中读取：

**CardManagementView.vue:364-367**:
```typescript
output_config: {
  mode: 'text_bitmap_plus_native_barcode',  // 硬编码！
  threshold: 160
}
```

**CardList.vue:341-344**:
```typescript
output_config: {
  mode: 'text_bitmap_plus_native_barcode',  // 硬编码！
  threshold: 160
}
```

后端 `print_qsl` 函数使用前端传入的 `request.output_config`，导致模板配置的修改不生效。

### 修复方案

修改 `src/commands/printer.rs` 中的 `print_qsl` 函数，使用模板文件中保存的 `config.output` 配置，而不是前端传入的参数。

**修改前**:
```rust
let output_config = OutputConfig {
    mode: request.output_config.mode.clone(),
    threshold: request.output_config.threshold,
};
```

**修改后**:
```rust
let output_config = OutputConfig {
    mode: config.output.mode.clone(),
    threshold: config.output.threshold,
};
```

## 相关文件

- `src/commands/printer.rs`: 打印命令（**已修复**）
- `src/printer/tspl.rs`: TSPL 指令生成器
- `src/printer/layout_engine.rs`: 布局引擎
- `src/printer/render_pipeline.rs`: 渲染管道
- `src/printer/backend/pdf.rs`: PDF 预览后端
- `config/templates/callsign.toml`: 呼号标签模板
- `web/src/views/CardManagementView.vue`: 卡片录入打印调用
- `web/src/components/cards/CardList.vue`: 卡片列表打印调用
- `web/src/views/ConfigView.vue`: 打印机配置页面（**已添加自动保存**）

## 额外改进：打印机配置自动保存

### 改进内容

为打印机配置页面添加了自动保存功能（类似模板配置页面）：

1. **自动保存**：选择打印机后自动保存，无需手动点击保存按钮
2. **状态提示**：显示"✓ 配置已自动保存"提示，3秒后自动消失
3. **防抖处理**：500ms 防抖，避免频繁保存
4. **统一配置**：添加提示"所有打印功能（QSL标签、地址标签、顺丰面单）都将使用此打印机"

### 打印功能统一配置确认

所有打印功能都从 `get_printer_config` 读取打印机配置：

- **录入后打印** (CardInputDialog.vue) ✅
- **列表打印标签** (CardList.vue) ✅
- **打印地址标签** (DistributeDialog.vue) ✅
- **打印顺丰面单** (WaybillPrintDialog.vue) ✅