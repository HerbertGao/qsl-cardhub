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
  "templateCode": "fm_76130_standard_HBTRJT0FNP6E",
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

### 5.1 API 错误码

| 错误码 | 说明 | 处理 |
|--------|------|------|
| A1000 | 成功 | 继续处理 |
| A1001 | 必传参数为空 | 提示用户检查配置 |
| A1002 | 请求时效已过期 | 检查系统时间 |
| A1003 | IP 无效 | 联系顺丰开通 IP |
| A1004 | 无对应服务权限 | 联系顺丰开通权限 |
| A1006 | 数字签名无效 | 检查校验码 |

### 5.2 用户提示

所有错误应转换为用户友好的中文提示，例如：
- "数字签名验证失败，请检查校验码是否正确"
- "无法连接顺丰服务器，请检查网络"
- "运单号不存在或无权限打印"

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
