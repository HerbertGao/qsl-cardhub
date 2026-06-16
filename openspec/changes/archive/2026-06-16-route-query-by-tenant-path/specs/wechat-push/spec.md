## MODIFIED Requirements

### 需求：按呼号查询结果页与「订阅收卡」绑定流程

若集成微信服务号推送，系统**必须**提供根据呼号查询收卡信息的单独页面；在该页的查询结果处提供「订阅收卡」入口，并按微信公众平台要求完成授权后将**呼号 + 租户**与用户（openid）绑定；后续顺丰推送根据该绑定关系向对应用户推送。绑定**必须**带租户维度（`callsign_openid_bindings` 含 `tenant_id`、主键 `(tenant_id, callsign, openid)`），授权回调的 `tenant_id` 由授权 `state` 解析得到（向前兼容），**禁止**跨租户写入或反查。

#### 场景：查询结果页展示订阅入口与提示

- **当** 用户访问根据呼号查询收卡信息的单独页面，并完成一次按呼号查询且返回了该呼号的收卡信息
- **那么** 结果区域必须提供「订阅收卡」按钮
- **并且** 必须提示用户：订阅后将收到该呼号相关的卡片分发/物流信息（如「订阅后，该呼号的卡片分发与物流动态将推送至您的微信」）
- **并且** 用户点击「订阅收卡」后，进入微信公众平台要求的授权流程（如跳转微信授权页）

#### 场景：授权 state 向前兼容携带并校验租户

- **当** 微信授权回调（`/api/wechat/auth-callback`）携带 `state` 到达服务端
- **那么** 服务端**必须**按 `tenant:callsign` 解析 `state`（以**首个**冒号分隔；callsign 与租户 slug 均不含冒号）
- **并且** 当 `state` **不含**冒号时**必须**向前兼容：`callsign` = 整串、`tenant_id` 回退为 `env.DEFAULT_TENANT` 指定的默认租户（本部署 `bh2ro`），**禁止**硬编码默认租户字面量（随 `tenant-isolation` 默认租户配置化）；本阶段（4-B `tenant-path-routing`）前端在 `/t/<slug>/` 页发 `tenant:callsign`（租户取自 URL）、bare 默认租户页发纯 callsign 走此无冒号兜底
- **并且** 解析出的 `tenant_id` **必须**校验为 `tenants` 表中 `status='active'` 的租户，否则**必须**拒绝（返回错误、**禁止**写入绑定），避免在不存在/非活跃租户名下落入垃圾绑定行
- **并且** 该 `tenant_id` **禁止**取自任何不经服务端校验的来源

#### 场景：微信授权后建立呼号–openid 绑定（含租户）

- **当** 用户在「订阅收卡」流程中按微信公众平台要求完成授权（提供用户信息/授权后带回 code）
- **那么** 服务端必须用授权得到的 code 换取 openid（及必要用户信息）
- **并且** 将「**租户 + 呼号**」与得到的 **openid** 建立绑定并写入 D1：`INSERT … INTO callsign_openid_bindings (tenant_id, callsign, openid, …)`，`tenant_id` 取自上一场景解析并校验后的值
- **并且** 同一租户下同一呼号可绑定多个用户（一呼号多 openid）；重复订阅同一 `(tenant_id, callsign, openid)` 时**必须**幂等处理（如 `INSERT OR IGNORE`）
- **并且** 向用户展示订阅成功提示

#### 场景：顺丰推送时按租户+呼号绑定关系向对应用户推送

- **当** 顺丰路由推送接口解析出呼号、并由匹配订单派生出 `tenant_id`（见 `tenant-isolation`/`sf-route-push-receiver` 规范），且微信推送功能已启用
- **那么** 服务端必须从 D1 查询「**该租户 + 该呼号**」下通过「订阅收卡」建立的绑定关系：`SELECT openid FROM callsign_openid_bindings WHERE tenant_id = ? AND callsign = ?`，得到对应的 openid 列表
- **并且** 对每个 openid 调用微信「发送模板消息」接口，模板内容包含运单号、路由描述（如 remark）、时间等卡片分发/物流信息
- **并且** **禁止**以无租户维度的 `WHERE callsign = ?` 反查 openid（否则同呼号跨租户推送、泄漏他租户订阅者 openid 与物流轨迹）
- **并且** 若该「租户 + 呼号」无绑定用户，则仅落库路由数据，不发送微信；发送失败时记录日志，不影响向顺丰返回 0000
