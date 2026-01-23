# 云端同步 API 规范

本文档描述了 QSL-CardHub 云端同步功能所需的 API 接口规范。您可以根据此规范自行实现云端接收服务。

## 概述

QSL-CardHub 支持将本地数据全量同步到用户自建的云端 API。同步采用推送模式，即客户端主动将数据推送到服务端。

- **同步方向**：单向（本地 → 云端）
- **同步模式**：全量同步（每次推送完整数据）
- **认证方式**：API Key (Bearer Token)

## 认证方式

所有接口都需要在请求头中携带 API Key：

```
Authorization: Bearer {your_api_key}
```

## 接口列表

### 1. 连接测试 (GET /ping)

用于测试 API 连接是否正常。

**请求**

```http
GET /ping
Authorization: Bearer {api_key}
```

**响应（成功）**

```json
{
  "success": true,
  "message": "pong",
  "server_time": "2026-01-23T14:30:00+08:00"
}
```

**响应（认证失败）**

HTTP 状态码：401

```json
{
  "success": false,
  "message": "API Key 无效"
}
```

### 2. 数据同步 (POST /sync)

接收客户端推送的全量数据。

**请求**

```http
POST /sync
Authorization: Bearer {api_key}
Content-Type: application/json
```

**请求体**

```json
{
  "client_id": "550e8400-e29b-41d4-a716-446655440000",
  "sync_time": "2026-01-23T14:30:00+08:00",
  "data": {
    "projects": [...],
    "cards": [...],
    "sf_senders": [...],
    "sf_orders": [...]
  }
}
```

**响应（成功）**

```json
{
  "success": true,
  "message": "同步成功",
  "received_at": "2026-01-23T14:30:01+08:00",
  "stats": {
    "projects": 10,
    "cards": 500,
    "sf_senders": 5,
    "sf_orders": 100
  }
}
```

**响应（失败）**

```json
{
  "success": false,
  "message": "错误描述"
}
```

## 数据结构定义

### Project（项目）

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "项目名称",
  "created_at": "2026-01-23T10:00:00+08:00",
  "updated_at": "2026-01-23T14:30:00+08:00"
}
```

### Card（卡片）

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "project_id": "550e8400-e29b-41d4-a716-446655440000",
  "creator_id": null,
  "callsign": "BV2AAA",
  "qty": 1,
  "status": "pending",
  "metadata": {
    "distribution": {
      "method": "邮寄",
      "address": "收件地址",
      "remarks": "备注",
      "distributed_at": "2026-01-23T14:00:00+08:00"
    },
    "return": null,
    "address_cache": [...]
  },
  "created_at": "2026-01-23T10:00:00+08:00",
  "updated_at": "2026-01-23T14:30:00+08:00"
}
```

**status 可选值**：
- `pending`：已录入（待分发）
- `distributed`：已分发
- `returned`：已退卡

### SFSender（顺丰寄件人）

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440002",
  "name": "寄件人姓名",
  "phone": "010-12345678",
  "mobile": "13800138000",
  "province": "北京市",
  "city": "北京市",
  "district": "朝阳区",
  "address": "详细地址",
  "is_default": true,
  "created_at": "2026-01-23T10:00:00+08:00",
  "updated_at": "2026-01-23T14:30:00+08:00"
}
```

### SFOrder（顺丰订单）

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440003",
  "order_id": "QSLHUB20260123143000001",
  "waybill_no": "SF1234567890123",
  "card_id": "550e8400-e29b-41d4-a716-446655440001",
  "status": "confirmed",
  "pay_method": 1,
  "cargo_name": "QSL 卡片",
  "sender_info": "{...}",
  "recipient_info": "{...}",
  "created_at": "2026-01-23T10:00:00+08:00",
  "updated_at": "2026-01-23T14:30:00+08:00"
}
```

**status 可选值**：
- `pending`：待确认
- `confirmed`：已确认
- `cancelled`：已取消
- `printed`：已打印

**pay_method 可选值**：
- `1`：寄方付
- `2`：收方付
- `3`：第三方付

## 错误码

| HTTP 状态码 | 说明 |
|------------|------|
| 200 | 请求成功 |
| 400 | 请求参数错误 |
| 401 | 认证失败（API Key 无效） |
| 500 | 服务器内部错误 |

## 实现建议

1. **数据隔离**：按 `client_id` 隔离不同客户端的数据，避免数据混淆。

2. **数据存储**：建议使用全量替换策略，每次同步时清空该 client_id 的旧数据，然后插入新数据。

3. **请求频率限制**：建议实现请求频率限制，防止滥用。

4. **HTTPS**：强烈建议使用 HTTPS 保护传输数据。

5. **日志记录**：记录每次同步请求的时间、client_id、数据量等信息，便于排查问题。

6. **数据验证**：验证接收到的数据格式是否正确，拒绝格式错误的请求。

## 示例实现

以下是使用 Node.js + Express 的简单实现示例：

```javascript
const express = require('express');
const app = express();

app.use(express.json({ limit: '50mb' }));

// API Key 验证中间件
function authMiddleware(req, res, next) {
  const authHeader = req.headers.authorization;
  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    return res.status(401).json({ success: false, message: 'API Key 无效' });
  }

  const apiKey = authHeader.slice(7);
  // 在这里验证 API Key
  if (apiKey !== process.env.API_KEY) {
    return res.status(401).json({ success: false, message: 'API Key 无效' });
  }

  next();
}

// 连接测试
app.get('/ping', authMiddleware, (req, res) => {
  res.json({
    success: true,
    message: 'pong',
    server_time: new Date().toISOString()
  });
});

// 数据同步
app.post('/sync', authMiddleware, async (req, res) => {
  try {
    const { client_id, sync_time, data } = req.body;

    // 在这里处理数据存储逻辑
    // ...

    res.json({
      success: true,
      message: '同步成功',
      received_at: new Date().toISOString(),
      stats: {
        projects: data.projects.length,
        cards: data.cards.length,
        sf_senders: data.sf_senders.length,
        sf_orders: data.sf_orders.length
      }
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

app.listen(3000, () => {
  console.log('Server running on port 3000');
});
```

## 更新历史

- 2026-01-23：初始版本
