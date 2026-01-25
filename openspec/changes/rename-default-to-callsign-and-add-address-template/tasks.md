# 实施任务清单

## 1. 重命名默认模板文件
- [ ] 1.1 将 `config/templates/default.toml` 重命名为 `config/templates/callsign.toml`
- [ ] 1.2 更新 `src/main.rs` 中的模板文件复制逻辑（`default.toml` → `callsign.toml`）
- [ ] 1.3 更新 `src/commands/printer.rs` 中的 `get_default_template_path()` 函数
- [ ] 1.4 更新 `src/config/profile_manager.rs` 中的 `get_default_template_name()` 函数
- [ ] 1.5 更新 `src/config/models.rs` 中的默认模板路径
- [ ] 1.6 更新所有测试文件中的模板路径引用
- [ ] 1.7 更新 `openspec/specs/configuration-management/spec.md` 中的默认模板路径引用

## 2. 创建地址模板文件
- [ ] 2.1 创建 `config/templates/address.toml` 模板文件
- [ ] 2.2 配置地址模板的页面参数（76mm × 130mm）
- [ ] 2.3 配置地址模板的元素（姓名、地址、邮寄方式等，不包含日期）
- [ ] 2.4 配置双份打印布局（上半部分和下半部分完全一致）

## 3. 后端 - 地址打印命令实现
- [ ] 3.1 在 `src/commands/printer.rs` 中实现 `print_address` 命令
  - 加载 `address.toml` 模板
  - 接收地址数据（姓名、中文地址、英文地址、邮寄方式等）
  - 生成双份打印布局（上半部分和下半部分）
  - 调用打印流程
- [ ] 3.2 在 `src/main.rs` 中注册 `print_address` 命令
- [ ] 3.3 实现地址模板加载函数 `get_address_template_path()`

## 4. 后端 - 地址模板配置接口
- [ ] 4.1 在 `src/commands/printer.rs` 中实现 `get_address_template_config` 命令
- [ ] 4.2 在 `src/commands/printer.rs` 中实现 `save_address_template_config` 命令
- [ ] 4.3 在 `src/main.rs` 中注册地址模板配置命令

## 5. 前端 - 分发对话框增强
- [ ] 5.1 在 `web/src/components/cards/DistributeDialog.vue` 中，在复制地址按钮后添加打印地址按钮
- [ ] 5.2 实现 `handlePrintAddress` 函数
  - 获取当前地址信息（从 `addressCache` 中获取）
  - 调用 `print_address` 命令
  - 显示打印成功/失败提示
- [ ] 5.3 处理不同地址源的地址数据格式（qrz.cn、qrz.com、QRZ卡片查询）

## 6. 前端 - 地址模板配置界面
- [ ] 6.1 在 `web/src/views/TemplateView.vue` 中添加模板类型选择（呼号模板/地址模板）
- [ ] 6.2 实现地址模板配置表单（参考现有标签模板配置）
- [ ] 6.3 实现地址模板预览功能
- [ ] 6.4 实现地址模板自动保存功能
- [x] 6.5 进入界面及切换标签类型时自动执行预览操作

## 7. 布局引擎 - 双份打印支持
- [ ] 7.1 在布局引擎中支持双份打印模式
  - 上半部分：正常布局
  - 下半部分：复制上半部分布局，垂直偏移到下半部分
- [ ] 7.2 确保上下两部分完全一致（元素位置、大小、内容）

## 8. 测试和验证
- [ ] 8.1 测试呼号模板重命名后的功能是否正常
- [ ] 8.2 测试地址模板打印功能
- [ ] 8.3 测试双份打印布局是否正确
- [ ] 8.4 测试不同地址源的数据格式处理
- [ ] 8.5 测试地址模板配置界面功能

## 9. 文档更新
- [ ] 9.1 更新相关文档中的模板路径引用
- [ ] 9.2 更新用户文档，说明地址模板的使用方法
