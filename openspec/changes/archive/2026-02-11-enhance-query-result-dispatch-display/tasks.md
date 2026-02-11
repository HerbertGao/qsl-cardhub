## 1. 类型与展示数据准备

- [x] 1.1 在 `web_query_service/src/client/App.vue` 的 `CardItem.metadata.distribution` 类型中补充 `method` 与 `proxy_callsign` 字段。
- [x] 1.2 在 `web_query_service/src/client/components/ResultList.vue` 的 `CardItem.metadata.distribution` 类型中补充 `method` 与 `proxy_callsign` 字段。

## 2. 已分发分发方式展示

- [x] 2.1 在 `ResultList.vue` 中新增“分发方式补充信息”渲染逻辑，仅在 `item.status === 'distributed'` 时启用。
- [x] 2.2 实现 `method = 自取` 时显示“自取”，且不显示复制按钮。
- [x] 2.3 实现 `method = 代领` 时显示“代领”，并在 `proxy_callsign` 存在时显示代领人呼号，且不显示复制按钮。
- [x] 2.4 保留并验证 `remarks` 存在时的备注展示与复制按钮行为不变。

## 3. 非已分发状态与样式回归

- [x] 3.1 确认非 `distributed` 状态不显示“自取/代领/代领人”扩展文案。
- [x] 3.2 调整 `ResultList.vue` 样式以支持“分发方式行 + 备注行”并保持移动端可读性。
- [ ] 3.3 手工验证状态组合：`distributed+自取`、`distributed+代领`、`distributed+仅备注`、`pending/printed/completed`。
