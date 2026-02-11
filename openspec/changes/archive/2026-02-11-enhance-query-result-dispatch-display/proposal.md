## 为什么

当前查询页在“已分发”状态下只展示统一备注，用户无法快速区分“自取”与“代领”，也看不到代领人呼号，导致信息解释成本高、容易误解。该信息已存在于 `metadata.distribution` 中，应在结果页直接呈现。

## 变更内容

- 在查询页结果卡片中，针对 `status = distributed` 增加分发方式展示：
  - `method = 自取`：在状态下方显示“自取”，不提供复制按钮。
  - `method = 代领`：在状态下方显示“代领（<代领人呼号>）”，不提供复制按钮。
- 对“已分发 + 备注（如运单号）”保留现有备注展示与复制能力。
- 对“非已分发状态”不新增额外分发方式文案，保持现有状态徽章与可选备注展示逻辑，避免将分发元数据误用于不相关状态。
- 无 BREAKING 变更。

## 功能 (Capabilities)

### 新增功能

- （无）

### 修改功能

- `cloud-backend-api`: 调整“按呼号查询结果页”的展示要求，要求在已分发记录中按分发方式显示补充信息（自取/代领人）。

## 影响

- 前端查询页组件：`web_query_service/src/client/components/ResultList.vue`（展示逻辑与样式）。
- 前端查询页类型定义：`web_query_service/src/client/App.vue` 与 `web_query_service/src/client/components/ResultList.vue` 中 `CardItem.metadata.distribution` 字段补充。
- 云端查询接口返回结构不变；仅消费已有 `metadata.distribution.method/proxy_callsign/remarks`。
