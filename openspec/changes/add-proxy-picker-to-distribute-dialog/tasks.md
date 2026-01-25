## 1. 后端实现

- [x] 1.1 在 `src/db/cards.rs` 中添加 `get_project_callsigns` 函数，查询指定项目下的所有呼号（去重）
- [x] 1.2 在 `src/commands/cards.rs` 中添加 `get_project_callsigns_cmd` Tauri 命令，供前端调用
- [x] 1.3 在 `src/db/models.rs` 中更新 `DistributionInfo` 结构体，添加 `proxy_callsign` 字段
- [x] 1.4 更新 `src/db/cards.rs` 中的 `distribute_card` 函数，接收 `proxy_callsign` 参数

## 2. 前端类型定义

- [x] 2.1 在 `web/src/types/components.ts` 中更新 `DistributeFormData` 接口，添加 `proxy_callsign` 字段（可选）
- [x] 2.2 在 `web/src/types/components.ts` 中更新 `DistributeConfirmData` 接口，添加 `proxy_callsign` 字段（可选）
- [x] 2.3 在 `web/src/types/models.ts` 中更新 `DistributionInfo` 接口，添加 `proxy_callsign` 字段

## 3. 前端组件实现

- [x] 3.1 在 `DistributeDialog.vue` 中添加代领人选择器组件（使用 Element Plus 的 `el-select`）
- [x] 3.2 实现当选择"代领"处理方式时显示代领人选择器，其他方式隐藏
- [x] 3.3 实现从后端获取本次转卡的所有呼号列表（去重）
- [x] 3.4 配置 `el-select` 的 `filterable` 属性，支持筛选功能
- [x] 3.5 配置 `el-select` 的 `allow-create` 属性，允许创建新选项
- [x] 3.6 代领人字段为可选（根据用户要求，不是必填项）

## 4. 数据提交

- [x] 4.1 更新 `DistributeDialog.vue` 的 `handleSubmit` 函数，将代领人呼号包含在提交数据中
- [x] 4.2 更新后端 `distribute_card_cmd` 函数，接收并处理 `proxy_callsign` 参数（可选）
- [x] 4.3 将代领人呼号保存到 `metadata.distribution` 中（如果提供了代领人）
- [x] 4.4 更新 `CardManagementView.vue` 的 `handleDistributeConfirm` 函数，传递 `proxy_callsign` 参数

## 5. 测试验证

- [ ] 5.1 验证选择"代领"时显示代领人选择器
- [ ] 5.2 验证代领人下拉列表显示本次转卡的所有呼号（去重）
- [ ] 5.3 验证筛选功能正常工作
- [ ] 5.4 验证可以输入不在列表中的呼号
- [ ] 5.5 验证临时输入的呼号不会保存到后台
- [ ] 5.6 验证选择其他处理方式时隐藏代领人选择器
- [ ] 5.7 代领人字段为可选，可以不填写
