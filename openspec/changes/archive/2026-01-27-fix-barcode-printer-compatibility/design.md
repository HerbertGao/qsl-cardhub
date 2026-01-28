# 设计文档：修复条形码打印机兼容性问题

## 架构背景

当前打印系统的渲染管道：

```
模板配置 (TOML, output.mode)
    ↓
RenderPipeline.render()
    ├── "text_bitmap_plus_native_barcode" → render_mixed_mode()
    │     文本 → 位图, 条码 → BarcodeElement（原生指令参数）
    └── "full_bitmap" → render_full_bitmap_mode()
          文本 → 位图, 条码 → 位图, 全部合成到画布
    ↓
TSPLGenerator.generate()
    ├── MixedMode: BITMAP指令(文本) + BARCODE指令(条码)
    └── FullBitmap: 单个BITMAP指令(完整画布)
```

## 方案权衡

### 方案 A：将全位图设为默认模式（选定）

**优点：**
- 改动最小，仅修改默认值和 UI
- 全位图模式已在 `render_pipeline.rs` 和 `tspl.rs` 中完整实现并有测试覆盖
- 位图模式兼容所有支持 TSPL `BITMAP` 指令的打印机（这是最基础的 TSPL 指令）
- 条形码渲染质量由软件控制，不依赖打印机固件的条码渲染能力

**缺点：**
- 全位图数据量比混合模式稍大（条码也作为位图传输）
- 条码清晰度依赖软件渲染质量（但 `barcode_renderer.rs` 已实现高质量渲染）

### 方案 B：自动检测打印机能力并回退

**优点：**
- 用户无感知

**缺点：**
- TSPL 协议没有标准的能力查询机制
- 需要维护打印机型号兼容性数据库
- 实现复杂度极高，且检测结果不可靠

### 方案 C：将标签预览图整体转为 TSPL 位图（参考顺丰面单）

**分析：**
顺丰面单打印流程是：下载 PDF → `pdf_renderer.rs` 渲染为灰度图 → `generate_from_image()` 生成 TSPL。这与 `full_bitmap` 模式本质相同——都是将所有内容渲染为一张完整图像再通过 TSPL `BITMAP` 指令打印。

现有的 `full_bitmap` 模式已经实现了这个功能：`render_full_bitmap_mode()` 将所有元素（包括文本和条码）合成到一张画布上，再由 `TSPLGenerator` 生成单个 `BITMAP` 指令。

因此方案 C 与方案 A 等价，无需额外实现。

## 决策

选择**方案 A**：将全位图模式设为默认，并提供 UI 切换能力。

理由：
1. 改动最小、风险最低
2. 全位图模式已有完整实现和测试
3. 保留了混合模式作为可选项，满足需要原生条码指令的高级用户
4. 与顺丰面单打印方式一致（全图像 → TSPL BITMAP）
