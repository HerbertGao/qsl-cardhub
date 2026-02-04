## 新增需求

### 需求：顺丰路由推送接收接口（沙箱与正式双 URL）

系统**必须**提供符合顺丰 RoutePushService 规范的 HTTP 接口，用于接收顺丰推送的运单路由信息；**必须**提供**两个**回调路径，供顺丰丰桥分别配置沙箱与正式环境：沙箱环境配置 `POST /api/sf/route-push/sandbox`，正式环境配置 `POST /api/sf/route-push`。若已集成微信服务号推送，则根据呼号与 openid 向用户发送物流状态通知；**沙箱**来源的推送在发给用户的内容中**必须**标记「【沙箱】」，**正式**来源不添加环境标记。

#### 场景：接收 JSON 格式路由推送

- **当** 顺丰系统向配置的 URL 发送 POST 请求，Content-Type 为 `application/json; charset=UTF-8`，请求体为 `{ "Body": { "WaybillRoute": [ { "mailno", "orderid", "acceptTime", "remark", "opCode", "id", ... } ] } }`（参见 `docs/sf-route-push-service.md`）
- **那么** 服务端必须解析 Body.WaybillRoute 数组
- **并且** 在顺丰要求的响应时间内返回 JSON：成功时 `{ "return_code": "0000", "return_msg": "成功" }`，失败时 `{ "return_code": "1000", "return_msg": "..." }`
- **并且** 对同一运单同一路由节点（如以 id 或 mailno+opCode+id 去重）不重复处理，避免重复写入与重复推送

#### 场景：路由数据与呼号关联

- **当** 服务端成功解析顺丰推送的一条或多条 WaybillRoute（含 `orderid`、`mailno` 等）
- **那么** 服务端必须根据同步得到的 `sf_orders` 与 `cards` 解析呼号：用 `orderid` 对应 `sf_orders.order_id` 或 `mailno` 对应 `sf_orders.waybill_no` 在 D1 中查询 `sf_orders`，再根据 `sf_orders.card_id` 关联 `cards` 表得到 `cards.callsign`
- **并且** 若已集成微信推送且该呼号存在绑定的 openid，则调用微信模板消息接口向该用户发送物流状态（运单号、路由描述、时间等）；若本次推送来自**沙箱**路径（/api/sf/route-push/sandbox），则推送内容中必须带「【沙箱】」标记，正式路径不添加该标记
- **并且** 路由数据可落库供查询或去重使用

#### 场景：未查到订单时呼号解析

- **当** 顺丰推送中的 `orderid` 或 `mailno` 在 D1 的 `sf_orders` 中无匹配记录（例如尚未同步或订单属其他 client）
- **那么** 服务端不得臆造呼号；仅落库路由数据（若有）、向顺丰返回 0000，不向微信用户推送
- **并且** 可记录日志便于排查

#### 场景：顺丰推送处理失败响应

- **当** 服务端处理推送时发生异常（如解析失败、数据库错误）
- **那么** 服务端必须仍在其要求的响应时间内返回 JSON：`{ "return_code": "1000", "return_msg": "系统异常" }`（或符合顺丰文档的失败描述）
- **并且** 记录错误日志便于排查；顺丰可能根据失败结果重试推送
