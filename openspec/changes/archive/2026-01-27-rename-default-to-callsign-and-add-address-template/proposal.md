# 变更：重命名默认模板为呼号模板并新增地址模板功能

## 为什么

当前系统使用 `default.toml` 作为默认打印模板，但该模板主要用于打印呼号标签。为了更清晰地表达模板用途，需要将其重命名为 `callsign.toml`。

同时，在卡片分发场景中，用户需要打印地址信息用于邮寄。现有系统只提供复制地址功能，缺少直接打印地址的功能。需要新增地址模板 `address.toml`，支持打印完整的地址信息，并且在同一张纸上打印两遍（上半部分和下半部分完全一致），方便用户裁剪使用。

## 变更内容

- **重命名模板文件**：将 `config/templates/default.toml` 重命名为 `config/templates/callsign.toml`
- **更新所有引用**：更新代码中所有对 `default.toml` 的引用，改为 `callsign.toml`
- **新增地址模板**：创建 `config/templates/address.toml` 模板文件，用于打印地址信息
- **地址模板配置功能**：参考现有的标签模板配置，为地址模板提供配置界面
- **分发对话框增强**：在分发对话框的复制地址按钮后增加打印地址按钮
- **地址打印功能**：实现地址打印功能，支持打印全部地址信息（不打印日期信息）
- **双份打印布局**：地址模板在同一张纸上打印两遍，上半部分和下半部分完全一致

## 影响

- **受影响规范**：
  - `template-configuration`（模板配置系统）
  - `card-management`（卡片管理）
- **受影响代码**：
  - `config/templates/default.toml` → `config/templates/callsign.toml`（重命名）
  - `config/templates/address.toml`（新增）
  - `src/commands/printer.rs`（更新模板路径引用，新增地址打印命令）
  - `src/config/profile_manager.rs`（更新默认模板名称）
  - `src/main.rs`（更新模板文件复制逻辑）
  - `web/src/components/cards/DistributeDialog.vue`（新增打印地址按钮）
  - `web/src/views/TemplateView.vue`（支持地址模板配置）
  - 所有测试文件中的模板路径引用
- **向后兼容性**：需要迁移现有配置，将 `template.path` 从 `"default.toml"` 更新为 `"callsign.toml"`
