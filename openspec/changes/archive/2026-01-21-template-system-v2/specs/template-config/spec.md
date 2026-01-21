# 功能: 模板配置系统 (template-config)

## 概述

定义和管理打印模板的配置格式(v2版本),支持灵活的元素来源配置、布局约束和渲染选项.

---

## 新增需求

### 需求: 支持v2版本的TOML配置格式

系统必须能够加载和解析v2版本的TOML模板配置文件,包含页面配置、布局配置、字体配置、元素列表和输出配置等完整部分.

#### 场景: 加载v2版本的模板配置文件

- **给定** 存在一个符合v2格式的TOML配置文件 `config/templates/default.toml`
- **当** 调用 `TemplateV2Config::load_from_file(path)` 加载配置
- **则** 应成功解析配置文件
- **并且** 返回的配置对象包含以下部分:
  - `metadata`: 模板元信息(name, version, description, template_version)
  - `page`: 纸张配置(尺寸、DPI、边距、边框)
  - `layout`: 布局配置(对齐方式、间距)
  - `fonts`: 字体配置(中文粗体、英文粗体、fallback)
  - `elements`: 元素列表(标题、副标题、呼号、条形码等)
  - `output`: 输出配置(渲染模式、阈值)

#### 场景: 解析页面配置

- **给定** v2配置文件包含以下页面配置:
  ```toml
  [page]
  dpi = 203
  width_mm = 76
  height_mm = 130
  margin_left_mm = 2
  margin_right_mm = 2
  margin_top_mm = 3
  margin_bottom_mm = 3
  border = true
  border_thickness_mm = 0.3
  ```
- **当** 解析配置
- **则** `config.page.dpi` 应为 203
- **并且** `config.page.width_mm` 应为 76
- **并且** `config.page.border` 应为 `true`

#### 场景: 解析布局配置

- **给定** v2配置文件包含以下布局配置:
  ```toml
  [layout]
  align_h = "center"
  align_v = "center"
  gap_mm = 2
  line_gap_mm = 2.0
  ```
- **当** 解析配置
- **则** `config.layout.align_h` 应为 `"center"`
- **并且** `config.layout.gap_mm` 应为 2
- **并且** `config.layout.line_gap_mm` 应为 2.0

#### 场景: 解析字体配置

- **给定** v2配置文件包含以下字体配置:
  ```toml
  [fonts]
  cn_bold = "SourceHanSansSC-Bold.otf"
  en_bold = "Arial-Bold.ttf"
  fallback_bold = "SourceHanSansSC-Bold.otf"
  ```
- **当** 解析配置
- **则** `config.fonts.cn_bold` 应为 `"SourceHanSansSC-Bold.otf"`
- **并且** `config.fonts.en_bold` 应为 `"Arial-Bold.ttf"`

---

### 需求: 支持三种元素来源类型(fixed/input/computed)

系统必须支持三种元素内容来源类型: fixed(固定值)、input(运行时输入)、computed(从其他字段计算),并能正确解析配置中的元素来源定义.

#### 场景: 解析固定值元素(fixed)

- **给定** v2配置文件包含以下元素:
  ```toml
  [[elements]]
  id = "title"
  type = "text"
  source = "fixed"
  value = "中国无线电协会业余分会-2区卡片局"
  max_height_mm = 10
  ```
- **当** 解析配置
- **则** 元素的 `source` 应为 `ElementSource::Fixed`
- **并且** `value` 应为 `"中国无线电协会业余分会-2区卡片局"`

#### 场景: 解析输入元素(input)

- **给定** v2配置文件包含以下元素:
  ```toml
  [[elements]]
  id = "callsign"
  type = "text"
  source = "input"
  key = "callsign"
  max_height_mm = 28
  ```
- **当** 解析配置
- **则** 元素的 `source` 应为 `ElementSource::Input`
- **并且** `key` 应为 `"callsign"`

#### 场景: 解析计算元素(computed)

- **给定** v2配置文件包含以下元素:
  ```toml
  [[elements]]
  id = "sn"
  type = "text"
  source = "computed"
  format = "SN: {sn}"
  max_height_mm = 22
  ```
- **当** 解析配置
- **则** 元素的 `source` 应为 `ElementSource::Computed`
- **并且** `format` 应为 `"SN: {sn}"`

---

### 需求: 支持文本和条形码两种元素类型

系统必须支持文本和条形码两种元素类型,并能正确解析每种类型特有的配置参数(如文本的max_height_mm、条形码的barcode_type和height_mm等).

#### 场景: 解析文本元素

- **给定** v2配置文件包含文本元素定义
- **当** 解析配置
- **则** 元素的 `type` 应为 `"text"`
- **并且** 应包含 `max_height_mm` 字段(高度预算)

#### 场景: 解析条形码元素

- **给定** v2配置文件包含以下条形码元素:
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
- **当** 解析配置
- **则** 元素的 `type` 应为 `"barcode"`
- **并且** `barcode_type` 应为 `"code128"`
- **并且** `height_mm` 应为 18
- **并且** `quiet_zone_mm` 应为 2
- **并且** `human_readable` 应为 `false`

---

### 需求: 支持输出模式配置

系统必须支持配置输出模式,包括"文本位图+原生条码"和"全位图"两种模式,并能正确解析输出配置中的mode和threshold参数.

#### 场景: 解析输出模式为"文本位图+原生条码"

- **给定** v2配置文件包含以下输出配置:
  ```toml
  [output]
  mode = "text_bitmap_plus_native_barcode"
  threshold = 160
  ```
- **当** 解析配置
- **则** `config.output.mode` 应为 `"text_bitmap_plus_native_barcode"`
- **并且** `config.output.threshold` 应为 160

#### 场景: 解析输出模式为"全位图"

- **给定** v2配置文件包含以下输出配置:
  ```toml
  [output]
  mode = "full_bitmap"
  threshold = 160
  ```
- **当** 解析配置
- **则** `config.output.mode` 应为 `"full_bitmap"`

---

### 需求: 提供默认v2模板配置

系统必须提供符合docs/template.v2.md规范的默认QSL Card v2模板配置,包含完整的元素定义(标题、副标题、呼号、条形码、序列号、数量)和合理的布局参数.

#### 场景: 获取默认的QSL Card v2模板

- **给定** 无现有配置文件
- **当** 调用 `TemplateV2Config::default_qsl_card_v2()`
- **则** 应返回一个完整的v2配置对象
- **并且** 配置应符合 `docs/template.v2.md` 规范
- **并且** 配置应包含以下元素顺序:
  1. 标题(Title)
  2. 副标题(Subtitle)
  3. 呼号(Callsign)
  4. 条形码(Barcode)
  5. 序列号(SN)
  6. 数量(QTY)

#### 场景: 保存默认模板到文件

- **给定** 默认的v2模板配置
- **当** 调用 `config.save_to_file(path)`
- **则** 应在指定路径生成TOML配置文件
- **并且** 文件内容应可被重新加载并解析

---

### 需求: 配置验证

系统必须在加载配置文件时验证必填字段的完整性、元素列表的合法性以及输出模式的有效性,并在验证失败时返回明确的错误信息.

#### 场景: 验证必填字段

- **给定** v2配置文件缺少必填字段(如 `page.width_mm`)
- **当** 尝试加载配置
- **则** 应返回错误,错误信息应指明缺少的字段

#### 场景: 验证元素顺序合法性

- **给定** v2配置文件的元素列表为空
- **当** 尝试加载配置
- **则** 应返回错误,错误信息应说明至少需要一个元素

#### 场景: 验证输出模式合法性

- **给定** v2配置文件的 `output.mode` 为无效值 `"invalid_mode"`
- **当** 尝试加载配置
- **则** 应返回错误,错误信息应列出支持的模式

---

## 修改需求

### 需求: 保持v1配置格式向后兼容

系统必须能够自动检测并加载v1版本的模板配置文件,提供统一的配置加载API同时支持v1和v2格式,并在加载v1配置时输出升级建议.

#### 场景: 加载v1版本的模板配置文件

- **给定** 存在一个v1格式的TOML配置文件 `config/templates/qsl-card-v1.toml`
- **当** 调用 `TemplateConfig::load_from_file(path)`(统一入口)
- **则** 应自动检测为v1版本
- **并且** 应成功解析为 `TemplateV1Config`
- **并且** 应输出警告: "检测到v1配置,建议迁移到v2格式"

#### 场景: 同时支持v1和v2配置的API

- **给定** 系统需要同时支持v1和v2配置
- **当** 设计配置加载API
- **则** 应提供统一的 `TemplateConfig` 枚举类型:
  ```rust
  pub enum TemplateConfig {
      V1(TemplateV1Config),
      V2(TemplateV2Config),
  }
  ```
- **并且** 应提供 `load_from_file` 自动检测版本

---

## 移除需求

无

---

## 重命名需求

### 需求: 将v1的PaperConfig重命名为PageConfig

- **旧名称**: `PaperConfig`
- **新名称**: `PageConfig`
- **理由**: v2配置使用 `[page]` 节,名称应保持一致

---

## 技术说明

### 配置文件结构(v2)

完整的v2配置文件示例参见 `docs/template.v2.md` 第7节.

### 数据结构定义

```rust
/// v2版本模板配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateV2Config {
    pub metadata: TemplateMetadata,
    pub page: PageConfig,
    pub layout: LayoutConfig,
    pub fonts: FontConfig,
    pub elements: Vec<ElementConfig>,
    pub output: OutputConfig,
}

/// 元素配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementConfig {
    pub id: String,
    #[serde(rename = "type")]
    pub element_type: String,  // "text" | "barcode"
    pub source: String,         // "fixed" | "input" | "computed"

    // 根据source不同,以下字段互斥:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,  // source = "fixed"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,    // source = "input"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>, // source = "computed"

    // 文本元素特有:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_height_mm: Option<f32>,

    // 条形码元素特有:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub barcode_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height_mm: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quiet_zone_mm: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub human_readable: Option<bool>,
}

/// 输出配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// 渲染模式: "text_bitmap_plus_native_barcode" | "full_bitmap"
    pub mode: String,
    /// 灰度转黑白阈值(0-255)
    pub threshold: u8,
}
```

### 版本检测算法

```rust
impl TemplateConfig {
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;

        // 先尝试解析为v2(检查是否有[page]节或template_version字段)
        if content.contains("[page]") || content.contains("template_version") {
            let v2 = toml::from_str::<TemplateV2Config>(&content)?;
            return Ok(TemplateConfig::V2(v2));
        }

        // 尝试解析为v1
        let v1 = toml::from_str::<TemplateV1Config>(&content)?;
        log::warn!("检测到v1配置,建议使用迁移工具转换为v2格式");
        Ok(TemplateConfig::V1(v1))
    }
}
```

---

## 依赖关系

- **前置功能**: 无
- **后续功能**:
  - `template-engine` (使用本配置结构)
  - `layout-system` (使用本配置中的布局参数)

---

## 测试要求

### 单元测试

- 测试v2配置文件加载和解析
- 测试三种元素来源(fixed/input/computed)的解析
- 测试配置验证(必填字段、合法性检查)
- 测试默认模板配置的生成和保存

### 集成测试

- 测试v1和v2配置的自动检测
- 测试完整配置文件的端到端加载

### 测试数据

- 提供示例v2配置文件: `tests/fixtures/template-v2-example.toml`
- 提供不完整配置文件(用于验证测试): `tests/fixtures/template-v2-invalid.toml`

---

**创建日期**: 2026-01-20
