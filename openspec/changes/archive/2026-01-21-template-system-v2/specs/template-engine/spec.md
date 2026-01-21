# 功能: 模板引擎 (template-engine)

## 概述

模板引擎负责解析模板配置文件,根据元素来源类型(fixed/input/computed)填充运行时数据,生成已解析的元素列表供布局引擎使用.

---

## 新增需求

### 需求: 填充固定值元素(fixed)

模板引擎必须能够处理source为fixed的元素,使用配置中的value字段作为元素内容,生成已解析的元素.

#### 场景: 解析fixed元素并使用配置中的固定值

- **给定** 模板配置包含一个fixed元素:
  ```toml
  [[elements]]
  id = "title"
  type = "text"
  source = "fixed"
  value = "中国无线电协会业余分会-2区卡片局"
  max_height_mm = 10
  ```
- **并且** 运行时数据为空 `{}`
- **当** 调用 `TemplateEngine::resolve(config, data)`
- **则** 应生成一个 `ResolvedElement`:
  - `id` 为 `"title"`
  - `content` 为 `"中国无线电协会业余分会-2区卡片局"`
  - `element_type` 为 `ElementType::Text`

---

### 需求: 填充输入元素(input)

模板引擎必须能够处理source为input的元素,从运行时数据中根据key获取值作为元素内容,并在缺少必需字段时返回错误.

#### 场景: 从运行时数据中获取值

- **给定** 模板配置包含一个input元素:
  ```toml
  [[elements]]
  id = "callsign"
  type = "text"
  source = "input"
  key = "callsign"
  max_height_mm = 28
  ```
- **并且** 运行时数据为:
  ```json
  {
    "callsign": "BG7XXX"
  }
  ```
- **当** 调用 `TemplateEngine::resolve(config, data)`
- **则** 应生成一个 `ResolvedElement`:
  - `id` 为 `"callsign"`
  - `content` 为 `"BG7XXX"`

#### 场景: 输入元素缺少必需的运行时数据

- **给定** 模板配置包含一个input元素,key为 `"callsign"`
- **并且** 运行时数据为空 `{}`
- **当** 调用 `TemplateEngine::resolve(config, data)`
- **则** 应返回错误: `"缺少必需的输入字段: callsign"`

#### 场景: 运行时数据包含额外字段

- **给定** 模板配置只引用了 `callsign` 字段
- **并且** 运行时数据为:
  ```json
  {
    "callsign": "BG7XXX",
    "extra_field": "ignored"
  }
  ```
- **当** 调用 `TemplateEngine::resolve(config, data)`
- **则** 应成功解析
- **并且** 额外的 `extra_field` 应被忽略

---

### 需求: 填充计算元素(computed)

模板引擎必须能够处理source为computed的元素,使用简单模板引擎替换format字符串中的{field}占位符,生成计算后的内容.

#### 场景: 使用简单模板引擎替换占位符

- **给定** 模板配置包含一个computed元素:
  ```toml
  [[elements]]
  id = "sn"
  type = "text"
  source = "computed"
  format = "SN: {sn}"
  max_height_mm = 22
  ```
- **并且** 运行时数据为:
  ```json
  {
    "sn": "123"
  }
  ```
- **当** 调用 `TemplateEngine::resolve(config, data)`
- **则** 应生成一个 `ResolvedElement`:
  - `id` 为 `"sn"`
  - `content` 为 `"SN: 123"`

#### 场景: 支持多个占位符

- **给定** 模板配置包含一个computed元素:
  ```toml
  [[elements]]
  id = "label"
  type = "text"
  source = "computed"
  format = "{callsign} - SN: {sn} - QTY: {qty}"
  max_height_mm = 20
  ```
- **并且** 运行时数据为:
  ```json
  {
    "callsign": "BG7XXX",
    "sn": "123",
    "qty": "50"
  }
  ```
- **当** 调用 `TemplateEngine::resolve(config, data)`
- **则** 生成的元素 `content` 应为 `"BG7XXX - SN: 123 - QTY: 50"`

#### 场景: computed元素缺少必需的运行时数据

- **给定** 模板配置包含computed元素,format为 `"SN: {sn}"`
- **并且** 运行时数据缺少 `sn` 字段
- **当** 调用 `TemplateEngine::resolve(config, data)`
- **则** 应返回错误: `"computed元素引用了不存在的字段: sn"`

#### 场景: computed元素引用其他computed元素

- **给定** 模板配置包含两个元素:
  ```toml
  [[elements]]
  id = "base"
  type = "text"
  source = "input"
  key = "callsign"
  max_height_mm = 20

  [[elements]]
  id = "derived"
  type = "text"
  source = "computed"
  format = "Callsign: {callsign}"
  max_height_mm = 20
  ```
- **并且** 运行时数据为:
  ```json
  {
    "callsign": "BG7XXX"
  }
  ```
- **当** 调用 `TemplateEngine::resolve(config, data)`
- **则** 两个元素应都能正确解析
- **说明**: computed元素直接引用运行时数据,不引用其他已解析元素

---

### 需求: 填充条形码元素

模板引擎必须能够处理type为barcode的元素,支持固定值、输入和计算三种来源,正确解析条形码特有配置.

#### 场景: 条形码内容从呼号派生

- **给定** 模板配置包含一个条形码元素:
  ```toml
  [[elements]]
  id = "barcode"
  type = "barcode"
  barcode_type = "code128"
  source = "computed"
  format = "{callsign}"
  height_mm = 18
  quiet_zone_mm = 2
  human_readable = false
  ```
- **并且** 运行时数据为:
  ```json
  {
    "callsign": "BG7XXX"
  }
  ```
- **当** 调用 `TemplateEngine::resolve(config, data)`
- **则** 应生成一个 `ResolvedElement`:
  - `id` 为 `"barcode"`
  - `element_type` 为 `ElementType::Barcode`
  - `content` 为 `"BG7XXX"`
  - `config.barcode_type` 为 `"code128"`

#### 场景: 条形码内容为固定值

- **给定** 模板配置包含一个条形码元素:
  ```toml
  [[elements]]
  id = "barcode"
  type = "barcode"
  barcode_type = "code39"
  source = "fixed"
  value = "TEST123"
  height_mm = 18
  ```
- **当** 调用 `TemplateEngine::resolve(config, data)`
- **则** 生成的元素 `content` 应为 `"TEST123"`

---

### 需求: 保持元素顺序

模板引擎在解析元素时必须保持配置文件中定义的元素顺序,确保布局引擎按正确顺序进行垂直布局.

#### 场景: 解析后的元素顺序与配置文件一致

- **给定** 模板配置包含以下元素顺序:
  1. title (fixed)
  2. subtitle (input)
  3. callsign (input)
  4. barcode (computed)
  5. sn (computed)
  6. qty (computed)
- **当** 调用 `TemplateEngine::resolve(config, data)`
- **则** 返回的 `Vec<ResolvedElement>` 应保持相同顺序
- **理由**: 布局引擎依赖元素顺序进行垂直布局

---

### 需求: 数据类型转换和格式化

模板引擎必须支持自动转换运行时数据中的数字和布尔类型为字符串,以便在文本中显示.

#### 场景: 支持数字类型的运行时数据

- **给定** 模板配置包含computed元素,format为 `"SN: {sn}"`
- **并且** 运行时数据为:
  ```json
  {
    "sn": 123
  }
  ```
  (注意sn是数字类型,不是字符串)
- **当** 调用 `TemplateEngine::resolve(config, data)`
- **则** 应自动转换为字符串: `"SN: 123"`

#### 场景: 支持布尔类型的运行时数据

- **给定** 运行时数据包含布尔值:
  ```json
  {
    "is_special": true
  }
  ```
- **并且** computed元素format为 `"Special: {is_special}"`
- **当** 调用 `TemplateEngine::resolve(config, data)`
- **则** 应转换为 `"Special: true"`

---

### 需求: 提供调试和日志功能

模板引擎必须提供调试日志记录元素解析过程,并输出已解析元素的摘要信息,便于问题排查.

#### 场景: 记录元素解析日志

- **给定** 启用了调试日志(log level = DEBUG)
- **当** 调用 `TemplateEngine::resolve(config, data)`
- **则** 应输出以下日志:
  - `[DEBUG] 解析元素: title (fixed)`
  - `[DEBUG] 解析元素: callsign (input) -> "BG7XXX"`
  - `[DEBUG] 解析元素: sn (computed) -> "SN: 123"`

#### 场景: 输出已解析元素摘要

- **给定** 解析完成
- **当** 调用 `TemplateEngine::resolve(config, data)`
- **则** 应输出日志: `[INFO] 成功解析 6 个元素`

---

## 修改需求

无

---

## 移除需求

无

---

## 重命名需求

无

---

## 技术说明

### 核心数据结构

```rust
/// 已解析的元素
#[derive(Debug, Clone)]
pub struct ResolvedElement {
    /// 元素ID
    pub id: String,
    /// 元素类型
    pub element_type: ElementType,
    /// 填充后的实际内容
    pub content: String,
    /// 元素配置(保留原始配置信息)
    pub config: ElementConfig,
}

/// 元素类型
#[derive(Debug, Clone, PartialEq)]
pub enum ElementType {
    Text,
    Barcode,
}
```

### TemplateEngine API

```rust
pub struct TemplateEngine;

impl TemplateEngine {
    /// 解析模板配置,填充运行时数据
    ///
    /// # 参数
    /// - `config`: 模板配置
    /// - `data`: 运行时数据(HashMap<String, Value>)
    ///
    /// # 返回
    /// 已解析的元素列表
    pub fn resolve(
        config: &TemplateV2Config,
        data: &HashMap<String, serde_json::Value>,
    ) -> Result<Vec<ResolvedElement>>;

    /// 解析单个元素
    fn resolve_element(
        element: &ElementConfig,
        data: &HashMap<String, serde_json::Value>,
    ) -> Result<ResolvedElement>;

    /// 简单模板引擎: 替换 {key} 占位符
    fn resolve_format(
        format: &str,
        data: &HashMap<String, serde_json::Value>,
    ) -> Result<String>;
}
```

### 简单模板引擎算法

```rust
fn resolve_format(
    format: &str,
    data: &HashMap<String, serde_json::Value>,
) -> Result<String> {
    let re = Regex::new(r"\{(\w+)\}").unwrap();
    let mut result = format.to_string();
    let mut missing_fields = Vec::new();

    for cap in re.captures_iter(format) {
        let key = &cap[1];
        if let Some(value) = data.get(key) {
            let value_str = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => return Err(anyhow!("不支持的数据类型: {}", key)),
            };
            result = result.replace(&format!("{{{}}}", key), &value_str);
        } else {
            missing_fields.push(key.to_string());
        }
    }

    if !missing_fields.is_empty() {
        return Err(anyhow!(
            "computed元素引用了不存在的字段: {}",
            missing_fields.join(", ")
        ));
    }

    Ok(result)
}
```

---

## 依赖关系

- **前置功能**: `template-config` (依赖配置结构定义)
- **后续功能**: `layout-system` (使用已解析的元素列表)

---

## 测试要求

### 单元测试

- 测试三种元素来源(fixed/input/computed)的解析
- 测试computed元素的多占位符替换
- 测试缺少必需字段时的错误处理
- 测试数字和布尔类型的自动转换
- 测试元素顺序保持

### 集成测试

- 测试完整配置文件的端到端解析
- 测试不同数据类型组合的运行时数据

### 测试数据

- 提供示例运行时数据: `tests/fixtures/runtime-data-example.json`
- 提供不完整运行时数据(用于错误测试): `tests/fixtures/runtime-data-incomplete.json`

---

**创建日期**: 2026-01-20
