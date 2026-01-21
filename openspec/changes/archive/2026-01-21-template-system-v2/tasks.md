# 实施任务清单

本文档列出打印模板系统v2改造的所有实施任务,按照依赖关系和优先级排序.

---

## 阶段1: 配置系统和基础设施 (1-2天)

### Task 1.1: 定义v2配置数据结构

**目标**: 在 `src/config/template_v2.rs` 中定义完整的v2配置结构

**验收标准**:
- [x] 定义 `TemplateV2Config` 结构体
- [x] 定义 `PageConfig`, `LayoutConfig`, `FontConfig`, `ElementConfig`, `OutputConfig` 子结构
- [x] 定义 `ElementSource` 枚举(Fixed/Input/Computed)
- [x] 实现 `Serialize` 和 `Deserialize` trait
- [x] 通过单元测试验证序列化/反序列化

**依赖**: 无

**规范引用**: `specs/template-config/spec.md`

---

### Task 1.2: 实现配置文件加载和保存

**目标**: 实现v2配置文件的加载、解析和保存功能

**验收标准**:
- [x] 实现 `TemplateV2Config::load_from_file(path)` 方法
- [x] 实现 `TemplateV2Config::save_to_file(path)` 方法
- [x] 实现 `TemplateV2Config::default_qsl_card_v2()` 生成默认配置
- [x] 添加配置验证(必填字段、合法性检查)
- [x] 通过单元测试验证加载和保存功能

**依赖**: Task 1.1

**规范引用**: `specs/template-config/spec.md`

---

### Task 1.3: 准备字体文件和字体加载器

**目标**: 准备中英文粗体字体文件,并实现字体加载逻辑

**验收标准**:
- [x] 在 `assets/fonts/` 目录放置字体文件:
  - `Arial-Bold.ttf` 或 `LiberationSans-Bold.ttf` (英文)
  - `SourceHanSansSC-Bold.otf` (中文,可考虑子集化)
- [x] 在 `src/printer/font_loader.rs` 中实现字体加载(使用 `include_bytes!`)
- [x] 测试字体加载成功(单元测试)

**依赖**: 无

**规范引用**: `specs/text-rendering/spec.md`

---

### Task 1.4: 创建默认v2配置文件

**目标**: 在 `config/templates/` 创建默认的v2配置文件

**验收标准**:
- [x] 创建 `config/templates/default.toml`
- [x] 配置应符合 `docs/template.v2.md` 规范
- [x] 配置应可被成功加载和解析

**依赖**: Task 1.2

**规范引用**: `specs/template-config/spec.md`

---

## 阶段2: 模板引擎 (1天)

### Task 2.1: 实现模板引擎核心逻辑

**目标**: 在 `src/printer/template_engine.rs` 中实现模板数据填充

**验收标准**:
- [x] 定义 `ResolvedElement` 结构体
- [x] 实现 `TemplateEngine::resolve(config, data)` 方法
- [x] 实现fixed元素处理(使用value)
- [x] 实现input元素处理(从data中取值)
- [x] 实现computed元素处理(简单模板引擎)
- [x] 通过单元测试验证三种来源类型

**依赖**: Task 1.1, Task 1.2

**规范引用**: `specs/template-engine/spec.md`

---

### Task 2.2: 实现简单模板引擎(占位符替换)

**目标**: 实现 `{field}` 占位符替换逻辑

**验收标准**:
- [x] 实现 `resolve_format(format, data)` 方法
- [x] 支持多个占位符替换
- [x] 支持数字和布尔类型自动转换
- [x] 缺少字段时返回明确错误
- [x] 通过单元测试验证

**依赖**: Task 2.1

**规范引用**: `specs/template-engine/spec.md`

---

### Task 2.3: 添加模板引擎日志和错误处理

**目标**: 增强模板引擎的可观测性

**验收标准**:
- [x] 添加DEBUG级别日志(记录每个元素的解析)
- [x] 添加INFO级别日志(解析完成摘要)
- [x] 完善错误信息(缺少字段、类型错误等)
- [x] 通过集成测试验证

**依赖**: Task 2.2

**规范引用**: `specs/template-engine/spec.md`

---

## 阶段3: 文本渲染增强 (2-3天)

### Task 3.1: 重构TextRenderer支持多字体

**目标**: 改造现有的 `src/printer/text_renderer.rs` 支持中英文字体自动切换

**验收标准**:
- [x] 在 `TextRenderer` 结构体中添加 `cn_font`, `en_font`, `fallback_font` 字段
- [x] 实现 `select_font_for_char(c: char) -> &Font` 方法
- [x] 实现CJK字符判断逻辑
- [x] 通过单元测试验证字体选择

**依赖**: Task 1.3

**规范引用**: `specs/text-rendering/spec.md`

---

### Task 3.2: 实现文本尺寸测量(支持混排)

**目标**: 实现准确的文本宽高度量,支持中英文混排

**验收标准**:
- [x] 实现 `measure_text(text, font_size) -> (u32, u32)` 方法
- [x] 支持纯英文、纯中文、混排三种情况
- [x] 累加宽度、取最大高度
- [x] 通过单元测试验证(不同文本、不同字号)

**依赖**: Task 3.1

**规范引用**: `specs/text-rendering/spec.md`

---

### Task 3.3: 实现文本渲染为1bpp位图

**目标**: 实现文本到1bpp黑白位图的渲染

**验收标准**:
- [x] 实现 `render_text(text, font_size) -> ImageBuffer` 方法
- [x] 支持混排文本的字形拼接
- [x] 实现灰度转1bpp阈值转换(threshold=160)
- [x] 通过组件测试验证(检查像素值为0或255)

**依赖**: Task 3.2

**规范引用**: `specs/text-rendering/spec.md`

---

### Task 3.4: 添加字体度量缓存

**目标**: 优化性能,缓存字符度量结果

**验收标准**:
- [x] 添加 `metrics_cache: Mutex<HashMap<(char, u32), CharMetrics>>` 字段
- [x] 在 `measure_text` 中使用缓存
- [x] 实现LRU淘汰策略(可选,初版可用固定大小)
- [x] 通过性能测试验证(缓存命中率、查询耗时)

**依赖**: Task 3.2

**规范引用**: `specs/text-rendering/spec.md`

---

### Task 3.5: 文本渲染单元测试和组件测试

**目标**: 完善文本渲染的测试覆盖

**验收标准**:
- [x] 单元测试: 字体选择、尺寸测量、位图渲染
- [x] 组件测试: 使用 `tests/components/text_rendering.rs`
- [x] 测试不同字号(8pt ~ 120pt)
- [x] 测试不同语言(英文、中文、混排)
- [x] 测试边界情况(空字符串、特殊字符)

**依赖**: Task 3.3

**规范引用**: `specs/text-rendering/spec.md`

---

## 阶段4: 布局引擎 (2-3天)

### Task 4.1: 实现画布和可用区域计算

**目标**: 在 `src/printer/layout_engine.rs` 中实现基础的尺寸转换

**验收标准**:
- [x] 创建 `LayoutEngine` 结构体
- [x] 实现 `calculate_canvas_size(page_config) -> (u32, u32)` (mm -> dots)
- [x] 实现 `calculate_available_area(page_config) -> (left, right, top, bottom)` (扣除边距)
- [x] 通过单元测试验证(203 DPI, 76×130mm)

**依赖**: Task 1.1

**规范引用**: `specs/layout-system/spec.md`

---

### Task 4.2: 实现二分搜索求最大字号算法

**目标**: 实现核心的字号计算算法

**验收标准**:
- [x] 实现 `calculate_max_font_size(content, max_height, available_width, font) -> f32` 方法
- [x] 使用二分搜索(范围8-120pt,精度0.5pt)
- [x] 依赖 `TextRenderer::measure_text` 进行字体度量
- [x] 通过单元测试验证(不同文本长度、不同高度预算)
- [x] 性能测试: 单次计算 < 10ms

**依赖**: Task 3.2, Task 4.1

**规范引用**: `specs/layout-system/spec.md`

---

### Task 4.3: 实现垂直居中和y坐标分配

**目标**: 实现整体内容块的垂直居中布局

**验收标准**:
- [x] 实现 `calculate_total_content_height(elements, line_gap) -> u32` 方法
- [x] 实现 `calculate_vertical_offset(available_height, total_height) -> u32` 方法
- [x] 实现 `assign_y_positions(elements, y_offset, line_gap)` 方法
- [x] 通过单元测试验证(累加高度、间距计算)

**依赖**: Task 4.1

**规范引用**: `specs/layout-system/spec.md`

---

### Task 4.4: 实现水平居中和x坐标计算

**目标**: 实现元素的水平居中

**验收标准**:
- [x] 实现 `calculate_horizontal_center(element_width, available_width, left_margin) -> u32` 方法
- [x] 支持文本元素(基于测量宽度)
- [x] 支持条形码元素(基于估算宽度+quiet_zone)
- [x] 通过单元测试验证

**依赖**: Task 3.2, Task 4.1

**规范引用**: `specs/layout-system/spec.md`

---

### Task 4.5: 实现全局防溢出校验和缩放

**目标**: 实现溢出检测和全局缩放

**验收标准**:
- [x] 实现 `apply_overflow_protection(elements, available_height, line_gap)` 方法
- [x] 检测内容总高度是否超出可用高度
- [x] 计算缩放比例并应用到所有元素
- [x] 缩放后重新计算y坐标
- [x] 通过单元测试验证(溢出和不溢出场景)

**依赖**: Task 4.3

**规范引用**: `specs/layout-system/spec.md`

---

### Task 4.6: 实现完整布局流程和结果输出

**目标**: 整合所有布局逻辑,输出 `LayoutResult`

**验收标准**:
- [x] 定义 `LayoutResult` 和 `LayoutedElement` 结构体
- [x] 实现 `layout(config, resolved_elements) -> LayoutResult` 方法
- [x] 依次调用: 字号计算、y坐标分配、x坐标计算、防溢出校验
- [x] 处理边框配置(如果启用)
- [x] 通过集成测试验证(完整流程)

**依赖**: Task 4.2, Task 4.3, Task 4.4, Task 4.5

**规范引用**: `specs/layout-system/spec.md`

---

### Task 4.7: 布局引擎性能测试

**目标**: 验证布局引擎性能

**验收标准**:
- [x] 布局计算总耗时 < 50ms
- [x] 二分搜索字号计算 < 10ms
- [x] 使用 `cargo bench` 或手动计时

**依赖**: Task 4.6

**规范引用**: `specs/layout-system/spec.md`

---

## 阶段5: 渲染管线 (2天)

### Task 5.1: 定义渲染结果数据结构

**目标**: 定义 `RenderResult` 枚举和相关结构

**验收标准**:
- [x] 定义 `RenderResult::MixedMode` 和 `RenderResult::FullBitmap` 枚举
- [x] 定义 `BarcodeElement` 结构体
- [x] 添加必要的字段(bitmaps, native_barcodes, canvas, canvas_size, border)

**依赖**: 无

**规范引用**: `specs/rendering-pipeline/spec.md`

---

### Task 5.2: 实现渲染管线核心逻辑

**目标**: 在 `src/printer/render_pipeline.rs` 中实现渲染协调

**验收标准**:
- [x] 创建 `RenderPipeline` 结构体
- [x] 实现 `render(layout_result, output_config) -> RenderResult` 方法
- [x] 根据 `output.mode` 分支到不同渲染模式
- [x] 通过单元测试验证分支逻辑

**依赖**: Task 5.1

**规范引用**: `specs/rendering-pipeline/spec.md`

---

### Task 5.3: 实现混合模式渲染(方案A)

**目标**: 实现文本位图+原生条码的渲染模式

**验收标准**:
- [x] 实现 `render_mixed_mode(layout) -> RenderResult` 方法
- [x] 遍历文本元素,调用 `TextRenderer::render_text` 生成位图
- [x] 保留条形码元素信息(不渲染为位图)
- [x] 返回 `RenderResult::MixedMode`
- [x] 通过单元测试验证

**依赖**: Task 3.3, Task 5.2

**规范引用**: `specs/rendering-pipeline/spec.md`

---

### Task 5.4: 实现全位图模式渲染(方案B)

**目标**: 实现全部渲染为位图的模式

**验收标准**:
- [x] 实现 `render_full_bitmap(layout) -> RenderResult` 方法
- [x] 创建白色背景画布
- [x] 叠加所有文本位图
- [x] 渲染条形码为位图并叠加
- [x] 绘制边框(如果启用)
- [x] 返回 `RenderResult::FullBitmap`
- [x] 通过单元测试验证

**依赖**: Task 3.3, Task 5.2, 现有的 `BarcodeRenderer`

**规范引用**: `specs/rendering-pipeline/spec.md`

---

### Task 5.5: 实现位图叠加和边框绘制

**目标**: 实现画布合成辅助函数

**验收标准**:
- [x] 实现 `create_canvas(width, height)` 创建白色背景
- [x] 实现 `overlay(canvas, bitmap, x, y)` 叠加位图(处理边界)
- [x] 实现 `draw_border(canvas, border_config)` 绘制边框
- [x] 通过单元测试验证(边界情况、透明度)

**依赖**: Task 5.4

**规范引用**: `specs/rendering-pipeline/spec.md`

---

## 阶段6: 后端适配 (2天)

### Task 6.1: 改造PDF后端接收RenderResult

**目标**: 重构 `src/printer/backend/pdf.rs` 为接收渲染结果

**验收标准**:
- [x] 移除TSPL解析逻辑(parse_tspl, parse_tspl_line, render_commands)
- [x] 新增 `render(result: RenderResult) -> Result<PathBuf>` 方法
- [x] 处理 `RenderResult::MixedMode`: 叠加位图+渲染条码
- [x] 处理 `RenderResult::FullBitmap`: 直接使用画布
- [x] 保存为PNG(PDF生成暂时禁用)
- [x] 通过集成测试验证

**依赖**: Task 5.5

**规范引用**: `specs/rendering-pipeline/spec.md`

---

### Task 6.2: 改造打印机后端生成TSPL指令

**目标**: 重构TSPL生成器从 `RenderResult` 生成指令

**验收标准**:
- [x] 在 `src/printer/backend/mod.rs` 或新建 `tspl_backend.rs`
- [x] 实现 `render(result: RenderResult) -> Result<String>` 方法
- [x] 处理 `RenderResult::MixedMode`: 生成BITMAP + BARCODE指令
- [x] 处理 `RenderResult::FullBitmap`: 生成整张BITMAP指令
- [x] 实现 `encode_bitmap_1bpp(bitmap)` 位图编码
- [x] 通过单元测试验证TSPL格式

**依赖**: Task 5.5

**规范引用**: `specs/rendering-pipeline/spec.md`

---

### Task 6.3: 实现TSPL位图编码

**目标**: 将1bpp位图编码为TSPL格式

**验收标准**:
- [x] 实现 `encode_bitmap_1bpp(bitmap) -> String` 方法
- [x] 每8个像素打包为1个字节(从高位到低位)
- [x] 转换为十六进制字符串
- [x] 处理行尾不足8位的情况
- [x] 通过单元测试验证(对比手工编码结果)

**依赖**: Task 6.2

**规范引用**: `specs/rendering-pipeline/spec.md`

---

## 阶段7: 命令层集成 (1天)

### Task 7.1: 更新打印命令使用新架构

**目标**: 改造 `src/commands/printer.rs` 中的打印命令

**验收标准**:
- [x] 更新 `print_qsl_card` Tauri命令
- [x] 调用流程: 加载配置 -> 模板引擎 -> 布局引擎 -> 渲染管线 -> 后端输出
- [x] 支持指定输出模式(PDF/Printer)
- [x] 处理错误并返回给前端
- [x] 通过端到端测试验证

**依赖**: Task 6.1, Task 6.2

**规范引用**: 所有规范

---

### Task 7.2: 更新前端调用代码

**目标**: 更新 `web/src/views/PrintView.vue` 使用新配置格式

**验收标准**:
- [x] 前端调用打印命令时传递运行时数据(callsign, sn, qty)
- [x] 适配新的错误格式
- [x] 更新UI提示(如"正在渲染..." vs "正在生成TSPL...")
- [x] 通过手动测试验证

**依赖**: Task 7.1

**规范引用**: 无(前端集成)

---

## 阶段8: 测试和优化 (2-3天)

### Task 8.1: 完整端到端测试

**目标**: 验证完整打印流程

**验收标准**:
- [x] 使用默认v2配置文件
- [x] 提供不同运行时数据(不同呼号、不同长度)
- [x] 测试PDF输出(检查PNG文件)
- [x] 测试打印机输出(检查TSPL指令格式)
- [x] 对比方案A和方案B的输出

**依赖**: Task 7.1

**规范引用**: 所有规范

---

### Task 8.2: 性能测试和优化

**目标**: 测试和优化整体性能

**验收标准**:
- [x] 测试完整渲染流程耗时
- [x] 目标: 配置加载 < 10ms, 模板引擎 < 20ms, 布局 < 50ms, 渲染 < 200ms
- [x] 识别性能瓶颈并优化
- [x] 验证字体度量缓存效果

**依赖**: Task 8.1

**规范引用**: 各规范的性能要求

---

### Task 8.3: 边界情况和错误处理测试

**目标**: 测试各种边界情况

**验收标准**:
- [x] 超长文本(自动缩小字号)
- [x] 超高内容块(触发全局缩放)
- [x] 空字符串、特殊字符
- [x] 缺少运行时字段
- [x] 损坏的配置文件
- [x] 所有错误应有清晰的错误信息

**依赖**: Task 8.1

**规范引用**: 各规范的错误处理要求

---

### Task 8.4: 条形码扫描测试

**目标**: 验证条形码可扫描性

**验收标准**:
- [x] 生成PDF输出
- [x] 打印或使用手机扫描条形码
- [x] 验证扫描结果与输入一致
- [x] 测试不同呼号长度的条形码

**依赖**: Task 8.1

**规范引用**: `specs/rendering-pipeline/spec.md`

---

## 阶段9: 文档和发布 (1天)

### Task 9.1: 更新用户文档

**目标**: 编写v2配置格式的使用文档

**验收标准**:
- [x] 更新 `README.md` 说明新特性
- [x] 编写 `docs/template-v2-guide.md` 配置指南
- [x] 提供配置示例和最佳实践
- [x] 说明两种渲染模式的差异

**依赖**: Task 8.1

**规范引用**: 无

---

### Task 9.2: 更新开发者文档

**目标**: 更新架构文档和API文档

**验收标准**:
- [x] 更新 `ARCHITECTURE.md` 反映新架构
- [x] 添加模块级文档注释(Rust doc comments)
- [x] 生成API文档: `cargo doc --no-deps --open`

**依赖**: Task 9.1

**规范引用**: 无

---

### Task 9.3: 清理代码和注释

**目标**: 移除废弃代码,完善注释

**验收标准**:
- [x] 移除旧的v1相关代码(如果不再需要)
- [x] 添加模块级和函数级文档注释(中文)
- [x] 运行 `cargo clippy` 修复警告
- [x] 运行 `cargo fmt` 格式化代码

**依赖**: Task 9.2

**规范引用**: 无

---

### Task 9.4: 发布v0.2版本

**目标**: 发布包含v2模板系统的新版本

**验收标准**:
- [x] 更新 `Cargo.toml` 版本号为 0.2.0
- [x] 更新 `tauri.conf.json` 版本号
- [x] 编写 CHANGELOG.md
- [x] 创建Git标签: `git tag v0.2.0`
- [x] 构建发布包: `cargo tauri build`

**依赖**: Task 9.3

**规范引用**: 无

---

## 并行任务说明

以下任务可以并行执行(在满足依赖的前提下):

**阶段2和阶段3可部分并行**:
- Task 2.1-2.3 (模板引擎) 和 Task 3.1-3.4 (文本渲染) 可同时进行

**阶段4的子任务可部分并行**:
- Task 4.1 (画布计算) 完成后
- Task 4.2 (字号计算) 和 Task 4.3 (垂直布局) 可同时进行

**阶段6的子任务可并行**:
- Task 6.1 (PDF后端) 和 Task 6.2-6.3 (打印机后端) 可同时进行

---

## 任务统计

- **总任务数**: 40+
- **预计工期**: 12-15天
- **关键路径**: 阶段1 → 阶段2 → 阶段3 → 阶段4 → 阶段5 → 阶段6 → 阶段7 → 阶段8
- **可并行阶段**: 阶段2+3, 阶段6

---

## 风险提示

1. **字体子集化**: 如果完整中文字体过大,可能需要使用fonttools提取子集(额外1-2天)
2. **TSPL位图兼容性**: 不同打印机对BITMAP指令支持不同,可能需要调试(额外1天)
3. **性能优化**: 如果初版性能不达标,可能需要额外优化(额外1-2天)

---

**任务清单创建日期**: 2026-01-20
