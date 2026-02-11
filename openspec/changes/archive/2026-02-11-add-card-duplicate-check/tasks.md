## 1. 后端重复检查

- [x] 1.1 在 `src/db/cards.rs` 的 `create_card()` 函数中，INSERT 前增加 `SELECT EXISTS(SELECT 1 FROM cards WHERE project_id = ?1 AND callsign = ?2)` 检查，重复时返回 `AppError::InvalidParameter("该呼号已在此项目中录入")`

## 2. 前端即时校验

- [x] 2.1 在 `CardInputDialog.vue` 中新增响应式变量 `projectCallsigns`（`Set<string>` 类型），用于存储当前项目下所有已有呼号
- [x] 2.2 在项目切换（`handleProjectChange`）和弹窗打开（预选项目时）调用 `get_project_callsigns_cmd` 加载呼号集合
- [x] 2.3 为 callsign 表单字段添加自定义校验器，输入时在 `projectCallsigns` 中比对，命中则显示"该呼号已在此项目中录入"

## 3. 连续录入同步

- [x] 3.1 在 `CardManagementView.vue` 的 `handleCardInputConfirm` 中，录入成功后通知 `CardInputDialog` 将刚录入的呼号追加到 `projectCallsigns` 集合中
