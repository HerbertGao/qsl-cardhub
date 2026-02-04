# 任务清单

## 阶段一：问题验证（短期方案）

- [x] **T1**: 将模板 `output.mode` 改为 `full_bitmap` 并测试
  - 修改 `config/templates/callsign.toml` ✅
  - 验证预览和打印位置是否一致（待用户测试）

- [ ] **T2**: 收集用户测试反馈
  - 确认 `full_bitmap` 模式是否解决位置偏移问题
  - 记录任何剩余的偏移量

## 已完成的修复

- [x] **修复模板配置保存后不生效的问题**
  - 修改 `src/commands/printer.rs` 中的 `print_qsl` 函数
  - 使用模板文件中保存的 `config.output` 配置

- [x] **统一所有打印功能使用配置**
  - 删除 `PrintRequest` 中的 `output_config` 字段
  - 删除 `AddressPreviewRequest` 中的 `output_config` 字段
  - 所有打印和预览功能均使用模板文件中的配置
  - 更新前端代码移除 `output_config` 参数

- [x] **打印机配置页面添加自动保存功能**
  - 修改 `web/src/views/ConfigView.vue`
  - 选择打印机后自动保存（500ms 防抖）
  - 显示保存状态提示

## 配置统一确认

### 打印机配置
所有打印功能都从 `get_printer_config` 读取打印机配置：
- **录入后打印** (CardInputDialog.vue) ✅
- **列表打印标签** (CardList.vue) ✅
- **打印地址标签** (DistributeDialog.vue) ✅
- **打印顺丰面单** (WaybillPrintDialog.vue) ✅

### 模板配置
所有功能都使用对应的模板配置文件：
- **QSL 标签打印/预览**: `callsign.toml` ✅
- **地址标签打印/预览**: `address.toml` ✅

## 阶段二：添加 REFERENCE 支持（如 T1 未完全解决）

- [ ] **T3**: 在模板配置中添加 REFERENCE 参数
- [ ] **T4**: 在 TSPL 生成器中支持 REFERENCE 指令
- [ ] **T5**: 更新模板配置 UI

## 阶段三：长期改进（可选）

- [ ] **T6**: 将 GAP 参数移到模板配置中
- [ ] **T7**: 改进 Code128 条码宽度计算精度
- [ ] **T8**: 添加打印机校准页功能

## 验证检查点

- [ ] 预览和打印位置完全一致（待用户测试）
- [ ] 条码可正常扫描（待用户测试）
- [ ] 文字不超出边框（待用户测试）
