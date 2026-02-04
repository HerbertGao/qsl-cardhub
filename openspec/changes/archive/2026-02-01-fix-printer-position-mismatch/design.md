# 技术设计：修复打印位置不一致问题

## 问题架构分析

### 当前架构

```
用户输入数据
     ↓
[TemplateEngine] 模板解析
     ↓
[LayoutEngine] 布局计算 → 计算每个元素的 (x, y) 坐标
     ↓
[RenderPipeline] 渲染
     ↓
     ├─→ [PDF 后端] → 所有元素渲染为位图 → PNG 预览
     │       ↑
     │   条码: BarcodeRenderer 渲染
     │
     └─→ [TSPL 生成器] → BITMAP + BARCODE 指令 → 打印机
             ↑
         条码: 原生 TSPL BARCODE 指令
```

### 问题根源

1. **条码渲染差异**: PDF 后端和 TSPL 生成器对条码的处理方式不同
2. **布局计算假设**: LayoutEngine 估算的条码宽度基于位图渲染，但 TSPL 原生条码宽度可能不同

### 详细差异分析

#### PDF 后端条码渲染 (pdf.rs:160-176)

```rust
let barcode_bitmap = self.barcode_renderer
    .render_barcode(&barcode.content, &barcode.barcode_type, barcode.height)?;
self.overlay(&mut canvas, &barcode_bitmap, barcode.x, barcode.y);
```

- 使用 `barcoders` crate 生成条码
- 条码宽度由 `barcoders` 计算确定

#### TSPL 生成器条码指令 (tspl.rs:289-292)

```rust
BARCODE {x},{y},"{type}",{height},{readable},0,2,2,"{content}"
```

- 使用打印机内置的条码生成
- 条码宽度由打印机固件计算，参数 `narrow=2, wide=2` 影响宽度

#### 布局引擎条码宽度估算 (layout_engine.rs:395-399)

```rust
let estimated_width = (char_count + 3) * 11 * narrow_width + quiet_zone_dots * 2;
```

- 基于 Code128 编码规则估算
- 可能与两种实际渲染结果都存在误差

## 解决方案设计

### 方案一：统一使用 full_bitmap 模式（推荐短期方案）

**原理**: 将所有元素（包括条码）都渲染为位图，然后发送到打印机

**优点**:
- 预览和打印完全一致
- 无需修改代码逻辑，只需修改模板配置

**缺点**:
- 位图数据量更大
- 条码清晰度可能略低于原生指令

**实现**:
```toml
# config/templates/callsign.toml
[output]
mode = "full_bitmap"  # 从 "text_bitmap_plus_native_barcode" 改为 "full_bitmap"
threshold = 160
```

### 方案二：添加 REFERENCE 支持

**原理**: 在 TSPL 指令中添加 REFERENCE 指令，允许用户调整打印起始位置

**TSPL 语法**:
```
REFERENCE x, y
```

**设计**:

```rust
// config/template.rs - PageConfig 扩展
pub struct PageConfig {
    // ... 现有字段
    pub reference_x_mm: Option<f32>,  // 默认 None
    pub reference_y_mm: Option<f32>,  // 默认 None
}

// tspl.rs - 添加 REFERENCE 指令
fn generate(...) -> Result<Vec<u8>> {
    // SIZE 指令后添加
    if let (Some(ref_x), Some(ref_y)) = (reference_x_mm, reference_y_mm) {
        let ref_x_dots = mm_to_dots(ref_x, dpi);
        let ref_y_dots = mm_to_dots(ref_y, dpi);
        tspl.extend_from_slice(
            format!("REFERENCE {},{}\r\n", ref_x_dots, ref_y_dots).as_bytes()
        );
    }
}
```

### 方案三：精确条码宽度计算（长期方案）

**原理**: 精确计算 TSPL BARCODE 指令的实际渲染宽度，使布局计算准确

**问题**: 需要了解具体打印机固件的条码渲染算法

**设计思路**:
1. 建立 TSPL 条码宽度查询表
2. 或者将条码渲染统一为位图模式

## 推荐实施顺序

1. **立即实施**: 方案一 - 修改模板配置为 `full_bitmap` 模式
2. **用户测试后**: 如仍有偏移，实施方案二添加 REFERENCE 支持
3. **长期优化**: 考虑方案三精确条码宽度计算

## 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| full_bitmap 模式条码扫描失败 | 高 | 测试多种扫描器，确保 203dpi 条码清晰度 |
| REFERENCE 参数设置复杂 | 中 | 提供打印校准页帮助用户确定偏移量 |
| 不同打印机行为差异 | 中 | 收集多种打印机测试数据 |
