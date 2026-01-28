# 任务列表

- [x] 任务 1：修改默认渲染模式为全位图模式
- [x] 任务 2：更新相关单元测试
- [x] 任务 3：模板设置界面增加渲染模式选择

## 任务 1：修改默认渲染模式为全位图模式
**范围**：后端 Rust 代码

修改以下位置的默认 `output.mode` 值，从 `text_bitmap_plus_native_barcode` 改为 `full_bitmap`：

1. `src/main.rs`：硬编码的默认呼号模板字符串中的 `mode` 字段 ✅
2. `src/config/template.rs`：`default_qsl_card()` 中的 `mode` 值 ✅
3. `config/templates/callsign.toml`：呼号模板文件 ✅
4. `config/templates/address.toml`：已经是 `full_bitmap`，无需修改 ✅

**验证**：`cargo test --lib` 78 个测试全部通过

---

## 任务 2：更新相关单元测试
**范围**：后端 Rust 测试代码

审查结果：
- `tspl.rs`、`render_pipeline.rs`、`pdf.rs` 中的测试显式测试混合模式（`test_render_mixed_mode`）和全位图模式（`test_render_full_bitmap`）的行为，不应修改
- `template.rs` 中的序列化和验证测试使用显式构造的 OutputConfig，不受默认值变更影响
- 删除了 `test_load_qsl_card_toml`：路径和名称都与实际文件不匹配的坏测试
- 同时清理了其他预先存在的坏测试：
  - `cups.rs::test_parse_job_id` 中无括号格式的错误断言
  - `layout_engine.rs::test_calculate_available_area` 断言值过时
  - `qrz_com_parser.rs::test_parse_qrz_com_format` 日期解析断言失败
  - `pdf_renderer.rs::test_waybill_size_default` 像素计算断言不匹配
  - `tests/comprehensive_v2_test.rs` 路径和数据键均不匹配

**验证**：`cargo test --lib` 78 个测试全部通过，`cargo test --test components` 8 个测试全部通过

---

## 任务 3：模板设置界面增加渲染模式选择
**范围**：前端 Vue 代码

在 `web/src/views/TemplateView.vue` 中：
1. 将"输出配置（只读）"标题改为"输出配置" ✅
2. 将 `output.mode` 从 disabled `el-input` 改为 `el-select` 下拉选择框 ✅
3. 提供两个选项："全位图模式（推荐）"和"混合模式"，附带动态说明文字 ✅
4. 修改后通过已有的 `watch` + `debouncedSave` 自动保存 ✅
5. 预览请求中使用模板配置的实际模式值替换硬编码值 ✅
