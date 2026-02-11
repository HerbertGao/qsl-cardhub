## 上下文

`/api/callsigns/:callsign` 接口当前从 D1 查询完整的卡片记录并原样返回给前端。前端 `ResultList.vue` 实际只使用 5 个字段：`id`（列表 key）、`project_name`（项目名）、`status`（状态）、`metadata.distribution.method`（分发方式）、`metadata.distribution.proxy_callsign`（代领呼号）、`metadata.distribution.remarks`（备注）。其余字段（`project_id`、`callsign`、`qty`、`serial`、`created_at`、`updated_at`、`metadata.distribution.address`、`metadata.return_info`）均为多余暴露。

## 目标 / 非目标

**目标：**
- 精简 API 响应体，只返回前端渲染所需的最小字段集
- 减少网络传输量和敏感信息暴露面

**非目标：**
- 不改变 API 路由路径或认证机制
- 不修改数据库 schema 或同步逻辑
- 不改变前端 UI 行为

## 决策

### 决策 1：在应用层 map 阶段裁剪，不改 SQL 查询

**选择**：保留现有 SQL 查询（仍 SELECT 完整字段），在 JavaScript map 阶段只提取需要的字段。

**理由**：
- SQL 改动风险较高（D1 查询），保持查询不变更安全
- map 阶段裁剪简单直观，只需修改对象构造逻辑
- metadata 需要 JSON.parse 后再提取子字段，无论如何需要在应用层处理

**替代方案**：
- 精简 SQL 的 SELECT 列 → 能减少 D1→Worker 传输但 metadata 是整个 JSON 字段，无法在 SQL 层面裁剪内部字段
- 新建独立的精简查询接口 → 过度设计，改现有接口即可

### 决策 2：保留 id 字段用于列表 key

**选择**：响应中保留 `id` 字段（用作 Vue v-for key 和复制按钮状态跟踪），但可以考虑替换为序号。

**理由**：`id` 是 UUID，暴露风险较低（无法通过 id 反查其他数据），且改为序号需要前端改动更多。

## 风险 / 权衡

- **[外部调用方]** 这是 BREAKING CHANGE，如有外部系统依赖完整响应字段会受影响 → 当前无已知外部调用方，风险可控
- **[未来字段需求]** 如前端后续需要更多字段，需同步修改后端 → 可接受，按需添加比默认全返好
