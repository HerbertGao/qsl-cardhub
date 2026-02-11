## 为什么

当前 `web_query_service` 的 `/api/callsigns/:callsign` 接口返回了大量前端不需要的字段（`project_id`、`callsign`、`qty`、`serial`、`created_at`、`updated_at`、完整 `metadata` 含地址和退卡信息），这些敏感业务数据通过抓包即可获取，存在数据泄露风险。需要精简响应体，只返回前端实际使用的字段。

## 变更内容

- **BREAKING** 精简 `/api/callsigns/:callsign` 响应体，移除前端未使用的字段
- 后端 SQL 查询只选取必要字段，在 map 阶段只提取 metadata 中前端需要的部分
- 前端 `CardItem` 类型定义同步更新

## 功能 (Capabilities)

### 新增功能

（无）

### 修改功能

- `cloud-backend-api`: 按呼号查询接口的响应字段精简，从返回完整卡片数据改为只返回前端展示所需的最小字段集

## 影响

- `web_query_service/src/worker/index.js`: 修改 SQL 查询和响应 map 逻辑
- `web_query_service/src/client/App.vue`: 更新 `CardItem` 类型定义
- `web_query_service/src/client/components/ResultList.vue`: 更新 `CardItem` 类型定义
- 外部调用方（如有）需适配新的响应格式
