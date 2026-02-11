## 1. 后端响应裁剪

- [x] 1.1 在 `web_query_service/src/worker/index.js` 中修改 `/api/callsigns/` 处理器的 map 逻辑，只返回 `id`、`project_name`、`status`、`distribution`（从 metadata 中提取 method、proxy_callsign、remarks），移除 `project_id`、`callsign`、`qty`、`serial`、`metadata`、`created_at`、`updated_at`

## 2. 前端类型同步

- [x] 2.1 更新 `web_query_service/src/client/App.vue` 中的 `CardItem` 接口定义，匹配精简后的响应结构
- [x] 2.2 更新 `web_query_service/src/client/components/ResultList.vue` 中的 `CardItem` 接口定义，匹配精简后的响应结构，调整模板中的字段访问路径（`item.metadata?.distribution` → `item.distribution`）
