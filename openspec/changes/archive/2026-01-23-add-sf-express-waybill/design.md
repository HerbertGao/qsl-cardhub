# 顺丰速运面单打印 - 技术设计

## 1. 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                        前端 (Vue 3)                          │
├─────────────────────────────────────────────────────────────┤
│  SFExpressConfigView.vue    │    WaybillPrintDialog.vue     │
│  - 环境切换                  │    - 运单号输入                │
│  - 顾客编码配置              │    - 提交打印按钮（获取PDF）   │
│  - 校验码配置                │    - PDF预览显示               │
│                              │    - 打印按钮（发送到打印机）  │
└──────────────────┬──────────────────────┬───────────────────┘
                   │ Tauri Commands       │
┌──────────────────▼──────────────────────▼───────────────────┐
│                        后端 (Rust)                           │
├─────────────────────────────────────────────────────────────┤
│  commands/sf_express.rs                                      │
│  - sf_save_config      保存配置                              │
│  - sf_load_config      加载配置                              │
│  - sf_clear_config     清除配置                              │
│  - sf_fetch_waybill    获取面单PDF（返回Base64预览图）       │
│  - sf_print_waybill    打印面单（发送TSPL到打印机）          │
├─────────────────────────────────────────────────────────────┤
│  sf_express/                                                 │
│  ├─ client.rs          API 客户端（请求、签名、响应解析）     │
│  ├─ models.rs          数据模型（请求/响应结构）              │
│  └─ pdf_renderer.rs    PDF 转 1bpp 点阵                      │
└──────────────────┬──────────────────────────────────────────┘
                   │ HTTP POST
┌──────────────────▼──────────────────────────────────────────┐
│              顺丰开放平台 API                                 │
│  生产: https://bspgw.sf-express.com/std/service             │
│  沙箱: https://sfapi-sbox.sf-express.com/std/service        │
└─────────────────────────────────────────────────────────────┘
```

## 2. API 调用流程

### 2.1 数字签名计算

根据顺丰数字签名规范：

```
msgDigest = Base64(MD5(msgData + timestamp + checkWord))
```

**步骤：**
1. 构造 `msgData` JSON 字符串（不进行 URL 编码）
2. 获取当前时间戳（毫秒）
3. 拼接：`msgData + timestamp + checkWord`
4. 对拼接字符串进行 MD5 哈希（UTF-8 编码）
5. 将 MD5 结果进行 Base64 编码

**Rust 实现示例：**
```rust
use base64::{Engine as _, engine::general_purpose::STANDARD};
use md5::{Md5, Digest};

fn calculate_msg_digest(msg_data: &str, timestamp: i64, check_word: &str) -> String {
    let to_verify = format!("{}{}{}", msg_data, timestamp, check_word);
    let mut hasher = Md5::new();
    hasher.update(to_verify.as_bytes());
    let result = hasher.finalize();
    STANDARD.encode(result)
}
```

### 2.2 API 请求格式

**请求头：**
```
Content-Type: application/x-www-form-urlencoded;charset=UTF-8
```

**重要说明：**
- 通讯双方采用 HTTP 作为通讯协议
- 提交方式为 POST 方式
- 字符集编码统一使用 UTF-8
- 参数需要通过 HTTP URL 编码传送
- 业务数据统一以字符串格式放在 msgData 字段中传送

**请求体（form-urlencoded）：**
```
partnerID=<顾客编码>
&requestID=<UUID>
&serviceCode=COM_RECE_CLOUD_PRINT_WAYBILLS
&timestamp=<时间戳>
&msgDigest=<数字签名>
&msgData=<业务JSON>
```

**msgData 结构：**
```json
{
  "templateCode": "fm_76130_standard_{partnerID}",
  "version": "2.0",
  "fileType": "pdf",
  "sync": true,
  "documents": [
    {
      "masterWaybillNo": "SF1234567890123"
    }
  ]
}
```

**说明：**
- `templateCode`：模板编码需要动态替换 `{partnerID}` 为实际的顾客编码
- `version`：固定值 `2.0`（必填）
- `sync`：设置为 `true` 使用同步模式，直接返回 PDF 下载地址
- `documents`：一批不超过 20 个运单

### 2.3 响应处理

**成功响应：**
```json
{
  "apiResultCode": "A1000",
  "apiResultData": {
    "success": true,
    "obj": {
      "files": [
        {
          "url": "https://...",
          "token": "...",
          "waybillNo": "SF1234567890123"
        }
      ]
    }
  }
}
```

**PDF 下载：**
- 使用 GET 请求访问 `url`
- 请求头添加 `X-Auth-token: <token>`
- Token 有效期 24 小时

### 2.4 两步打印流程

```
用户界面                    后端                           顺丰API
   │                         │                               │
   │  1. 点击「提交打印」     │                               │
   │ ──────────────────────> │                               │
   │    sf_fetch_waybill     │   调用云打印API               │
   │                         │ ─────────────────────────────>│
   │                         │   返回PDF链接+Token           │
   │                         │ <─────────────────────────────│
   │                         │   下载PDF                     │
   │                         │ ─────────────────────────────>│
   │                         │   返回PDF数据                 │
   │                         │ <─────────────────────────────│
   │   返回预览图(Base64)    │                               │
   │ <────────────────────── │                               │
   │   显示PDF预览           │                               │
   │                         │                               │
   │  2. 点击「打印」        │                               │
   │ ──────────────────────> │                               │
   │    sf_print_waybill     │                               │
   │                         │   PDF→灰度→1bpp→TSPL         │
   │                         │   发送到打印机                 │
   │   返回打印结果          │                               │
   │ <────────────────────── │                               │
```

**步骤1：提交打印（sf_fetch_waybill）**
- 调用顺丰API获取PDF
- 将PDF渲染为预览图像（PNG格式，Base64编码）
- 缓存PDF数据供后续打印使用
- 返回预览图像给前端显示

**步骤2：打印（sf_print_waybill）**
- 使用缓存的PDF数据
- 渲染为1bpp点阵
- 生成TSPL指令
- 发送到打印机

## 3. PDF 转 TSPL 打印

### 3.1 渲染流程

```
PDF 文件
    ↓ pdfium/pdf-render
页面位图 (RGB/RGBA)
    ↓ 灰度转换
灰度图像 (8bit)
    ↓ 阈值二值化 (threshold=128)
1bpp 点阵图
    ↓ TSPL BITMAP 指令
打印机输出
```

### 3.2 TSPL BITMAP 指令

```
SIZE 76 mm, 130 mm
GAP 0 mm, 0 mm
DIRECTION 1,0
CLS
BITMAP 0,0,<width_bytes>,<height>,0,<data>
PRINT 1,1
```

其中：
- `width_bytes` = ceil(width_pixels / 8)
- `height` = 页面高度像素
- `data` = 1bpp 位图数据（MSB first）

### 3.3 DPI 适配

- 模板尺寸：76mm × 130mm
- 打印机 DPI：203 dpi
- 目标分辨率：608 × 1040 像素

如果 PDF 渲染分辨率不同，需要缩放到目标尺寸。

## 4. 数据模型

### 4.1 配置模型

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SFExpressConfig {
    /// 环境：production / sandbox
    pub environment: String,
    /// 顾客编码（合作伙伴编码）
    pub partner_id: String,
    /// 模板编码（固定值）
    pub template_code: String,
}
```

### 4.2 凭据存储键

| 键名 | 说明 |
|------|------|
| `qsl-cardhub:sf:partner_id` | 顾客编码 |
| `qsl-cardhub:sf:checkword_prod` | 生产环境校验码 |
| `qsl-cardhub:sf:checkword_sandbox` | 沙箱环境校验码 |
| `qsl-cardhub:sf:environment` | 当前环境 |

### 4.3 打印请求模型

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct WaybillPrintRequest {
    /// 运单号
    pub waybill_no: String,
}
```

## 5. 错误处理

### 5.1 API 平台错误码

| 错误码 | 说明 | 处理建议 |
|--------|------|----------|
| A1000 | 成功 | 继续处理业务结果（注意：不代表业务处理成功） |
| A1001 | 必传参数不可为空 | 检查必填字段、请求头 Content-type、参数编码 |
| A1002 | 请求时效已过期 | 检查系统时间是否正确 |
| A1003 | IP 无效 | 顾客编码配置了 IP 校验，请解除或绑定 IP |
| A1004 | 无对应服务权限 | 检查 API 是否已关联，环境是否正确 |
| A1005 | 流量受控 | 单接口限流：30次/秒，3000次/天 |
| A1006 | 数字签名无效 | 检查 checkword、签名算法、特殊字符处理 |
| A1007 | 重复请求 | 等待一分钟后重试 |
| A1009 | 目标服务异常或不可达 | 顺丰服务异常，稍后重试 |
| A1011 | OAuth2 认证失败 | accessToken 过期，需重新获取 |
| A1099 | 系统异常 | 顺丰服务异常，联系技术支持 |

### 5.2 业务错误码（下单相关）

| 错误码 | 说明 | 处理建议 |
|--------|------|----------|
| S0000 | 业务处理成功 | - |
| 1010 | 寄件地址不能为空 | 补充寄件人 address 字段 |
| 1011 | 寄件联系人不能为空 | 补充寄件人 contact 字段 |
| 1012 | 寄件电话不能为空 | 补充寄件人 tel 或 mobile 字段 |
| 1014 | 到件地址不能为空 | 补充收件人 address 字段 |
| 1015 | 到件联系人不能为空 | 补充收件人 contact 字段 |
| 1016 | 到件电话不能为空 | 补充收件人 tel 或 mobile 字段 |
| 1023 | 托寄物品名不能为空 | 补充 cargoDetails.name 字段 |
| 6126 | 月结卡号不合法 | 月结卡号必须为 10 位数字 |
| 8016 | 重复下单 | orderId 不能重复 |
| 8114 | 传入了不可发货的月结卡号 | 联系销售经理增加权限 |
| 8119 | 月结卡号不存在或已失效 | 检查 monthlyCard 字段 |
| 8196 | 信息异常 | 收件或寄件电话号码信息异常 |

### 5.3 业务错误码（订单确认/取消相关）

| 错误码 | 说明 | 处理建议 |
|--------|------|----------|
| 8017 | 订单号与运单号不匹配 | 检查 orderId 和 waybillNo 是否匹配 |
| 8018 | 未获取到订单信息 | 修改订单号 orderId |
| 8019 | 订单已确认或已消单 | 检查下单模板是否配置为自动确认 |
| 8037 | 已消单 | 订单已取消，不能重复取消 |
| 8252 | 订单已确认 | 订单已确认，不能重复确认 |
| 8253 | 订单已取消 | 订单已取消，不能重复取消 |
| 20052 | 月结卡号不匹配 | 月结卡号不匹配，不允许操作该订单 |

### 5.4 业务错误码（订单查询相关）

| 错误码 | 说明 | 处理建议 |
|--------|------|----------|
| 6135 | 未传入订单信息 | 检查请求格式 |
| 6150 | 找不到该订单 | 确认订单号是否正确 |
| 8018 | 未获取到订单信息 | 确认订单号是否正确 |

### 5.5 用户提示映射

所有错误应转换为用户友好的中文提示：

```rust
fn get_user_friendly_error(code: &str, msg: &str) -> String {
    match code {
        "A1001" => "请求参数不完整，请检查配置信息".to_string(),
        "A1003" => "IP 地址未授权，请联系顺丰开通".to_string(),
        "A1004" => "无服务权限，请检查 API 配置或联系顺丰".to_string(),
        "A1006" => "数字签名验证失败，请检查校验码是否正确".to_string(),
        "A1009" => "顺丰服务暂时不可用，请稍后重试".to_string(),
        "1010" | "1014" => "地址信息不完整，请补充详细地址".to_string(),
        "1011" | "1015" => "联系人姓名不能为空".to_string(),
        "1012" | "1016" => "联系电话不能为空".to_string(),
        "8016" => "订单号已存在，请勿重复下单".to_string(),
        "8018" | "6150" => "订单不存在，请检查订单号".to_string(),
        "8019" => "订单已确认或已取消".to_string(),
        "8114" | "8119" => "月结卡号无效或无权限".to_string(),
        _ => format!("操作失败：{}", msg),
    }
}
```

## 6. 依赖库

### 6.1 新增 Rust 依赖

```toml
[dependencies]
# MD5 哈希
md5 = "0.7"
# PDF 渲染（选择其一）
pdfium-render = "0.8"  # 推荐，跨平台
# 或 pdf = "0.9"

# 已有依赖（复用）
# reqwest, base64, serde_json, image
```

### 6.2 前端无新增依赖

复用现有的 Element Plus 组件和 Tauri API。

## 7. 下单流程设计

### 7.1 完整下单流程

```
用户界面                    后端                           顺丰API                    数据库
   │                         │                               │                          │
   │  1. 点击「下顺丰订单」  │                               │                          │
   │ ──────────────────────> │                               │                          │
   │   打开下单对话框        │                               │                          │
   │   自动预填寄件人信息    │   获取默认寄件人               │                          │
   │                         │ <─────────────────────────────│                          │
   │                         │                               │                          │
   │  2. 填写收件人信息      │                               │                          │
   │   提交订单              │                               │                          │
   │ ──────────────────────> │                               │                          │
   │    sf_create_order      │   调用下单API                 │                          │
   │                         │ ─────────────────────────────>│                          │
   │                         │   返回订单号(orderId)         │                          │
   │                         │ <─────────────────────────────│                          │
   │                         │   保存订单到数据库            │                          │
   │                         │ ──────────────────────────────┼────────────────────────>│
   │   返回订单号             │                               │                          │
   │ <────────────────────── │                               │                          │
   │   显示二次确认对话框    │                               │                          │
   │                         │                               │                          │
   │  3a. 选择「立即确认」   │                               │                          │
   │ ──────────────────────> │                               │                          │
   │    sf_confirm_order      │   调用确认订单API             │                          │
   │                         │ ─────────────────────────────>│                          │
   │                         │   返回运单号(waybillNo)        │                          │
   │                         │ <─────────────────────────────│                          │
   │                         │   更新订单状态                 │                          │
   │                         │ ──────────────────────────────┼────────────────────────>│
   │                         │   回填运单号到卡片备注         │                          │
   │                         │ ──────────────────────────────┼────────────────────────>│
   │   返回运单号             │                               │                          │
   │ <────────────────────── │                               │                          │
   │   显示「打印面单」选项  │                               │                          │
   │                         │                               │                          │
   │  3b. 选择「稍后确认」   │                               │                          │
   │ ──────────────────────> │                               │                          │
   │   关闭对话框             │                               │                          │
   │   订单保存到列表         │                               │                          │
```

### 7.2 订单状态流转

```
创建订单 → 待确认 → 已确认（获取运单号）→ 已打印
              ↓              ↓
           已取消         已取消（需传入运单号）
```

### 7.3 下单 API 接口

#### 7.3.1 EXP_RECE_CREATE_ORDER（下订单接口）

**请求参数（msgData）：**
```json
{
  "language": "zh-CN",
  "orderId": "客户订单号（唯一）",
  "cargoDetails": [
    {
      "name": "物品名称",
      "count": 1,
      "unit": "件",
      "weight": 0.5,
      "amount": 0,
      "currency": "CNY"
    }
  ],
  "contactInfoList": [
    {
      "contactType": 1,
      "contact": "寄件人姓名",
      "tel": "寄件人电话",
      "mobile": "寄件人手机",
      "country": "CN",
      "province": "广东省",
      "city": "深圳市",
      "county": "南山区",
      "address": "详细地址"
    },
    {
      "contactType": 2,
      "contact": "收件人姓名",
      "tel": "收件人电话",
      "mobile": "收件人手机",
      "country": "CN",
      "province": "广东省",
      "city": "广州市",
      "county": "天河区",
      "address": "详细地址"
    }
  ],
  "expressTypeId": 1,
  "payMethod": 1,
  "isGenWaybillNo": 1,
  "isReturnRoutelabel": 1
}
```

**字段说明：**
| 字段 | 必填 | 说明 |
|------|------|------|
| language | 是 | 响应报文语言，固定 `zh-CN` |
| orderId | 是 | 客户订单号，重复使用时返回第一次下单的运单信息 |
| cargoDetails | 是 | 托寄物信息 |
| contactInfoList | 是 | 收寄双方信息，contactType: 1=寄件人, 2=收件人 |
| country | 是 | 国家或地区代码，内地件填 `CN` |
| expressTypeId | 是 | 快件产品类别，2=顺丰标快 |
| payMethod | 否 | 付款方式，1=寄方付, 2=收方付, 3=第三方付 |
| isGenWaybillNo | 否 | 是否分配运单号，1=分配（默认） |
| isReturnRoutelabel | 是 | 是否返回路由标签，1=返回（建议传 1） |

**响应结构：**
```json
{
  "apiResultCode": "A1000",
  "apiResultData": {
    "success": true,
    "errorCode": "S0000",
    "errorMsg": null,
    "msgData": {
      "orderId": "客户订单号",
      "originCode": "719",
      "destCode": "020",
      "filterResult": 2,
      "waybillNoInfoList": [
        {
          "waybillType": 1,
          "waybillNo": "SF7444400043266"
        }
      ],
      "routeLabelInfo": [...]
    }
  }
}
```

**说明：**
- `filterResult`：筛单结果，1=人工确认, 2=可收派, 3=不可收派
- `waybillNoInfoList`：返回的运单号列表，waybillType: 1=母单, 2=子单, 3=签回单

#### 7.3.2 EXP_RECE_UPDATE_ORDER（订单确认/取消接口）

**请求参数（msgData）- 订单确认：**
```json
{
  "orderId": "客户订单号",
  "dealType": 1,
  "waybillNoInfoList": [
    {
      "waybillNo": "SF7444400043266",
      "waybillType": 1
    }
  ]
}
```

**请求参数（msgData）- 订单取消（待确认订单）：**
```json
{
  "orderId": "客户订单号",
  "dealType": 2,
  "waybillNoInfoList": []
}
```

**请求参数（msgData）- 订单取消（已确认订单）：**
```json
{
  "orderId": "客户订单号",
  "dealType": 2,
  "waybillNoInfoList": [
    {
      "waybillNo": "SF7444400043266",
      "waybillType": 1
    }
  ]
}
```

**字段说明：**
| 字段 | 必填 | 说明 |
|------|------|------|
| orderId | 是 | 客户订单号 |
| dealType | 否 | 操作类型，1=确认（默认），2=取消 |
| waybillNoInfoList | 条件 | 确认时必填；取消已确认订单时必填，取消待确认订单时传空数组 |

**说明：**
- 丰桥下订单接口默认自动确认，不需要客户重复确认
- 如需手工确认需要前往控制台修改配置
- 订单取消之后，订单号也不能重复利用
- 已确认订单取消时必须传入运单号信息

**响应结构（确认成功）：**
```json
{
  "apiResultCode": "A1000",
  "apiResultData": {
    "success": true,
    "errorCode": "S0000",
    "errorMsg": null,
    "msgData": {
      "orderId": "客户订单号",
      "waybillNoInfoList": [
        {
          "waybillType": 1,
          "waybillNo": "SF7444400048449"
        }
      ],
      "resStatus": 2
    }
  }
}
```

**响应结构（取消成功）：**
```json
{
  "apiResultCode": "A1000",
  "apiResultData": {
    "success": true,
    "errorCode": "S0000",
    "errorMsg": null,
    "msgData": {
      "orderId": "客户订单号",
      "waybillNoInfoList": [...],
      "resStatus": 2
    }
  }
}
```

**说明：**
- `resStatus`：操作结果，1=订单号与运单不匹配，2=操作成功

#### 7.3.3 EXP_RECE_SEARCH_ORDER_RESP（订单结果查询接口）

**请求参数（msgData）：**
```json
{
  "orderId": "客户订单号",
  "language": "zh-CN"
}
```

**字段说明：**
| 字段 | 必填 | 说明 |
|------|------|------|
| orderId | 是 | 客户订单号 |
| mainWaybillNo | 条件 | 运单号（15位或12位母单号），与 orderId 二选一 |
| searchType | 否 | 查询类型，1=正向单，2=退货单 |
| language | 否 | 响应语言，默认 `zh-CN` |

**响应结构：**
```json
{
  "apiResultCode": "A1000",
  "apiResultData": {
    "success": true,
    "errorCode": "S0000",
    "errorMsg": null,
    "msgData": {
      "orderId": "客户订单号",
      "origincode": "719",
      "destcode": "020",
      "filterResult": "2",
      "waybillNoInfoList": [
        {
          "waybillNo": "SF7444400034485",
          "waybillType": 1
        }
      ],
      "routeLabelInfo": [...]
    }
  }
}
```

**说明：**
- `filterResult`：筛单结果，1=人工确认，2=可收派，3=不可收派

## 8. 数据库设计

### 8.1 订单表（sf_orders）

```sql
CREATE TABLE sf_orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id TEXT NOT NULL UNIQUE,
    waybill_no TEXT,
    card_id INTEGER,
    status TEXT NOT NULL,  -- pending, confirmed, cancelled, printed
    sender_info TEXT NOT NULL,  -- JSON
    recipient_info TEXT NOT NULL,  -- JSON
    pay_method INTEGER NOT NULL DEFAULT 1,  -- 1=寄方付, 2=收方付, 3=第三方付
    cargo_name TEXT NOT NULL DEFAULT 'QSL卡片',  -- 托寄物名称
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (card_id) REFERENCES cards(id)
);

CREATE INDEX idx_sf_orders_order_id ON sf_orders(order_id);
CREATE INDEX idx_sf_orders_card_id ON sf_orders(card_id);
CREATE INDEX idx_sf_orders_status ON sf_orders(status);
```

### 8.2 寄件人表（sf_senders）

```sql
CREATE TABLE sf_senders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    phone TEXT NOT NULL,
    mobile TEXT,
    province TEXT NOT NULL,
    city TEXT NOT NULL,
    district TEXT NOT NULL,
    address TEXT NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_sf_senders_is_default ON sf_senders(is_default);
```

## 9. 地址数据集成

### 9.1 地址数据源

使用 [data_location](https://github.com/mumuy/data_location) 项目提供的地址数据：
- 符合 GB/T 2260 标准
- 包含省、市、区三级数据
- 数据格式：JSON

### 9.2 地址数据存储

**方案一：打包到应用**
- 将地址数据 JSON 文件打包到应用资源目录
- 应用启动时加载到内存
- 优点：离线可用，响应快
- 缺点：应用体积增大（约几MB）

**方案二：动态加载**
- 地址数据存储在应用数据目录
- 首次使用时下载或从资源加载
- 支持定期更新
- 优点：应用体积小
- 缺点：首次使用需要加载时间

**推荐方案：方案一**（数据量不大，打包更方便）

### 9.3 地址选择器实现

**前端组件结构：**
```vue
<AddressSelector
  v-model:province="province"
  v-model:city="city"
  v-model:district="district"
/>
```

**数据流：**
1. 组件加载时调用 `sf_get_provinces` 获取省份列表
2. 用户选择省份后，调用 `sf_get_cities(province)` 获取城市列表
3. 用户选择城市后，调用 `sf_get_districts(city)` 获取区县列表
4. 选择完成后，组合为完整地址字符串

## 10. 数据模型扩展

### 10.1 下单请求模型

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub cargo_details: Vec<CargoDetail>,
    pub contact_info_list: Vec<ContactInfo>,
    pub express_type_id: i32,
    pub pay_method: i32,
    pub is_gen_bill_no: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CargoDetail {
    pub name: String,
    pub count: i32,
    pub unit: String,
    pub weight: f64,
    pub amount: f64,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactInfo {
    pub contact_type: i32,  // 1-寄件人, 2-收件人
    pub contact: String,
    pub tel: String,
    pub mobile: Option<String>,
    pub address: String,
}
```

### 10.2 订单响应模型

```rust
#[derive(Debug, Deserialize)]
pub struct CreateOrderResponse {
    pub order_id: String,
    pub waybill_no_list: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderResponse {
    pub order_id: String,
    pub waybill_no_list: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct SearchOrderResponse {
    pub order_id: String,
    pub waybill_no_list: Vec<String>,
    pub order_status: i32,
    pub order_status_desc: String,
}
```

### 10.3 寄件人模型

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderInfo {
    pub id: Option<i64>,
    pub name: String,
    pub phone: String,
    pub mobile: Option<String>,
    pub province: String,
    pub city: String,
    pub district: String,
    pub address: String,
    pub is_default: bool,
}
```

### 10.4 订单关联卡片模型

用于订单列表显示，通过 LEFT JOIN 关联卡片和项目表，获取呼号、项目名、数量等信息。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SFOrderWithCard {
    // 继承 SFOrder 所有字段
    pub id: i64,
    pub order_id: String,
    pub waybill_no: Option<String>,
    pub card_id: Option<i64>,
    pub status: String,
    pub sender_info: String,
    pub recipient_info: String,
    pub pay_method: i32,
    pub cargo_name: String,
    pub created_at: String,
    pub updated_at: String,
    // 关联卡片信息
    pub callsign: Option<String>,
    pub project_name: Option<String>,
    pub qty: Option<i32>,
}
```

## 11. 错误处理扩展

### 11.1 下单相关错误码

| 错误码 | 说明 | 处理 |
|--------|------|------|
| A1000 | 成功 | 继续处理 |
| A1001 | 必传参数为空 | 提示用户检查必填字段 |
| A1002 | 请求时效已过期 | 检查系统时间 |
| A1003 | IP 无效 | 联系顺丰开通 IP |
| A1004 | 无对应服务权限 | 联系顺丰开通权限 |
| A1006 | 数字签名无效 | 检查校验码 |
| A2001 | 订单创建失败 | 显示具体错误信息 |
| A2002 | 订单不存在 | 提示订单不存在 |
| A2003 | 订单状态不允许操作 | 提示当前状态不允许此操作 |

### 11.2 用户提示

所有错误应转换为用户友好的中文提示，例如：
- "订单创建失败：收件人地址不完整，请补充详细地址"
- "订单确认失败：订单不存在或已被取消"
- "地址数据加载失败，请检查网络连接或重新启动应用"
