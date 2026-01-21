# 实施任务清单

## 1. 修复模板路径配置
- [x] 1.1 修改 `src/commands/printer.rs` 中的 `get_default_template_path()` 函数
- [x] 1.2 将开发模式路径从 `../../config/templates/default.toml` 改为 `config/templates/default.toml`
- [x] 1.3 验证模板文件能正确加载

## 2. 修复边框绘制逻辑
- [x] 2.1 修改 `src/printer/layout_engine.rs` 中的边框配置计算
- [x] 2.2 确保边框绘制在 margin 边界（外框）
- [x] 2.3 确保内容区域在边框内侧（内框），向内缩进 border_thickness
- [x] 2.4 添加调试日志输出边框和内容区域信息

## 3. 增加文字安全边距
- [x] 3.1 在 `layout_text_element()` 中计算文字时减去安全边距
- [x] 3.2 设置安全边距为约 1mm（使用 `mm_to_dots(1.0, dpi)`）
- [x] 3.3 确保安全边距不会导致可用宽度为负

## 4. 修复文字宽度测量
- [x] 4.1 修改 `src/printer/text_renderer.rs` 中的 `measure_text()` 方法
- [x] 4.2 使用 `pixel_bounding_box()` 计算实际像素宽度，而非 `advance_width`
- [x] 4.3 处理字形向左延伸的情况（negative min_x）
- [x] 4.4 同步修改 `render_text()` 方法，确保渲染位置与测量一致
- [x] 4.5 移除旧的字符度量缓存逻辑（因为现在需要计算整个文本的边界框）

## 5. 测试验证
- [x] 5.1 创建对称性测试，验证元素左右边距相等
- [x] 5.2 创建实际像素分析测试，验证渲染后的像素分布
- [x] 5.3 运行综合测试验证所有功能正常
- [x] 5.4 清理临时测试文件

## 6. 文档更新
- [ ] 6.1 更新 README.md 说明模板路径配置
- [ ] 6.2 记录文字测量方法的变更（从排版宽度到像素宽度）
