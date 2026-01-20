# API v2 使用指南

QSL-CardHub v2 提供了完整的 Rust API 和 Tauri 命令接口，用于生成和打印 QSL 卡片。

## 目录
- [Rust API](#rust-api)
- [Tauri 命令](#tauri-commands)
- [渲染模式](#rendering-modes)
- [示例代码](#examples)

---

## Rust API

### 快速开始

```rust
use QSL_CardHub::api_v2::quick_generate_png;
use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    // 准备数据
    let mut data = HashMap::new();
    data.insert("task_name".to_string(), "CQWW DX Contest".to_string());
    data.insert("callsign".to_string(), "BG7XXX".to_string());
    data.insert("sn".to_string(), "001".to_string());
    data.insert("qty".to_string(), "500".to_string());

    // 生成 PNG 预览
    let png_path = quick_generate_png(
        None,                          // 使用默认模板
        &data,
        PathBuf::from("output"),
        "full_bitmap",                 // 全位图模式
    ).unwrap();

    println!("✅ PNG 生成: {}", png_path.display());
}
```

### 高级用法

```rust
use QSL_CardHub::api_v2::QslCardGenerator;
use QSL_CardHub::config::template_v2::{TemplateV2Config, OutputConfig};
use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    // 创建生成器（可复用）
    let mut generator = QslCardGenerator::new().unwrap();

    // 加载自定义模板
    let config = TemplateV2Config::load_from_file(
        std::path::Path::new("config/templates/my-template.toml")
    ).unwrap();

    // 准备数据
    let mut data = HashMap::new();
    data.insert("callsign".to_string(), "BG7XXX".to_string());
    data.insert("sn".to_string(), "001".to_string());
    data.insert("qty".to_string(), "100".to_string());

    // 配置输出
    let output_config = OutputConfig {
        mode: "text_bitmap_plus_native_barcode".to_string(),
        threshold: 160,
    };

    // 生成 PNG
    let png_path = generator.generate_png(
        &config,
        &data,
        PathBuf::from("output"),
        &output_config,
    ).unwrap();

    // 生成 TSPL 指令
    let tspl = generator.generate_tspl(&config, &data, &output_config).unwrap();

    println!("✅ PNG: {}", png_path.display());
    println!("✅ TSPL: {} 字节", tspl.len());
}
```

### API 参考

#### `QslCardGenerator`

主要的卡片生成器类。

**方法：**

- `new() -> Result<Self>` - 创建新的生成器
- `generate_png(&mut self, config, data, output_dir, output_config) -> Result<PathBuf>` - 生成 PNG 文件
- `generate_tspl(&mut self, config, data, output_config) -> Result<String>` - 生成 TSPL 指令
- `render(&mut self, config, data, output_config) -> Result<RenderResult>` - 获取渲染结果（高级用法）

#### 便捷函数

- `quick_generate_png(template_path, data, output_dir, mode) -> Result<PathBuf>`
- `quick_generate_tspl(template_path, data, mode) -> Result<String>`

---

## Tauri 命令

Tauri 应用可通过以下命令与前端交互。

### `preview_qsl_v2`

生成 PNG 预览。

**参数：**
```typescript
interface PrintRequest {
  template_path?: string;  // 模板文件路径（可选）
  data: Record<string, string>;  // 运行时数据
  output_config: {
    mode: string;  // "text_bitmap_plus_native_barcode" | "full_bitmap"
    threshold: number;  // 0-255
  };
}
```

**返回：**
```typescript
interface PreviewResponse {
  png_path: string;  // PNG 文件路径
  width: number;     // 图像宽度
  height: number;    // 图像高度
}
```

**示例（前端）：**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const request = {
  data: {
    task_name: "CQWW DX Contest",
    callsign: "BG7XXX",
    sn: "001",
    qty: "500"
  },
  output_config: {
    mode: "full_bitmap",
    threshold: 160
  }
};

const result = await invoke('preview_qsl_v2', { request });
console.log(`预览生成: ${result.png_path} (${result.width}x${result.height})`);
```

### `print_qsl_v2`

打印 QSL 卡片到打印机。

**参数：**
- `printer_name: string` - 打印机名称
- `request: PrintRequest` - 打印请求（同上）

**示例（前端）：**
```typescript
await invoke('print_qsl_v2', {
  printerName: "TSC TTP-244",
  request: {
    data: {
      callsign: "BG7XXX",
      sn: "001",
      qty: "100"
    },
    output_config: {
      mode: "text_bitmap_plus_native_barcode",
      threshold: 160
    }
  }
});
```

### `generate_tspl_v2`

生成 TSPL 指令（调试用）。

**参数：**
- `request: PrintRequest`

**返回：** `string` - TSPL 指令

**示例（前端）：**
```typescript
const tspl = await invoke('generate_tspl_v2', { request });
console.log("TSPL 指令:", tspl);
```

### `load_template_v2`

加载模板配置。

**参数：**
- `path?: string` - 模板文件路径（可选，不提供则返回默认模板）

**返回：** `string` - 模板配置 JSON

### `save_template_v2`

保存模板配置。

**参数：**
- `path: string` - 保存路径
- `config_json: string` - 模板配置 JSON

---

## 渲染模式

### 1. text_bitmap_plus_native_barcode（混合模式）

**特点：**
- 文本渲染为位图
- 条形码使用打印机原生指令（BARCODE）
- 适用于支持 BARCODE 指令的热敏打印机
- TSPL 体积较小
- 打印速度快

**适用场景：**
- TSC、Zebra 等支持 Code128 的打印机
- 需要高质量条形码的场景

**TSPL 示例：**
```
SIZE 76 mm, 130 mm
GAP 2 mm, 0 mm
DIRECTION 0
CLS
BITMAP 21,141,71,52,1,00000000...  (文本位图)
BARCODE 200,300,"128",120,1,0,2,2,"BG7XXX"  (原生条码)
BOX 5,5,603,1034,3
PRINT 1
```

### 2. full_bitmap（全位图模式）

**特点：**
- 整个画布渲染为一张位图
- 条形码也渲染为位图
- 兼容所有支持 BITMAP 指令的打印机
- TSPL 体积较大
- 渲染质量完全可控

**适用场景：**
- 通用打印机
- PDF/PNG 预览
- 不支持 BARCODE 指令的打印机

**TSPL 示例：**
```
SIZE 76 mm, 130 mm
GAP 2 mm, 0 mm
DIRECTION 0
CLS
BITMAP 0,0,76,1039,1,FFFFFFFF...  (完整画布位图)
PRINT 1
```

---

## 示例代码

### 示例 1: 批量生成卡片

```rust
use QSL_CardHub::api_v2::QslCardGenerator;
use QSL_CardHub::config::template_v2::{TemplateV2Config, OutputConfig};
use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    let mut generator = QslCardGenerator::new().unwrap();
    let config = TemplateV2Config::default_qsl_card_v2();
    let output_config = OutputConfig {
        mode: "full_bitmap".to_string(),
        threshold: 160,
    };

    // 批量生成 100 张卡片
    for i in 1..=100 {
        let mut data = HashMap::new();
        data.insert("task_name".to_string(), "CQWW DX".to_string());
        data.insert("callsign".to_string(), "BG7XXX".to_string());
        data.insert("sn".to_string(), format!("{:03}", i));
        data.insert("qty".to_string(), "100".to_string());

        let png_path = generator.generate_png(
            &config,
            &data,
            PathBuf::from("output"),
            &output_config,
        ).unwrap();

        println!("✅ 生成 {}/100: {}", i, png_path.display());
    }
}
```

### 示例 2: 自定义模板

```rust
use QSL_CardHub::api_v2::QslCardGenerator;
use QSL_CardHub::config::template_v2::TemplateV2Config;
use std::path::Path;

fn main() {
    // 加载自定义模板
    let mut config = TemplateV2Config::load_from_file(
        Path::new("config/templates/qsl-card-v2.toml")
    ).unwrap();

    // 修改配置
    config.page.width_mm = 80.0;  // 修改纸张宽度
    config.output.threshold = 180;  // 调整二值化阈值

    // 保存修改后的模板
    config.save_to_file(Path::new("config/templates/my-custom.toml")).unwrap();

    println!("✅ 自定义模板已保存");
}
```

### 示例 3: 前端集成（Vue 3）

```vue
<template>
  <div class="qsl-generator">
    <form @submit.prevent="generatePreview">
      <input v-model="formData.callsign" placeholder="呼号" required />
      <input v-model="formData.sn" placeholder="序列号" required />
      <input v-model="formData.qty" placeholder="数量" required />
      <button type="submit">生成预览</button>
    </form>

    <img v-if="previewUrl" :src="previewUrl" alt="QSL 预览" />
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';
import { convertFileSrc } from '@tauri-apps/api/tauri';

const formData = ref({
  callsign: 'BG7XXX',
  sn: '001',
  qty: '100'
});

const previewUrl = ref(null);

async function generatePreview() {
  const request = {
    data: {
      task_name: "预览",
      callsign: formData.value.callsign,
      sn: formData.value.sn,
      qty: formData.value.qty
    },
    output_config: {
      mode: "full_bitmap",
      threshold: 160
    }
  };

  try {
    const result = await invoke('preview_qsl_v2', { request });
    previewUrl.value = convertFileSrc(result.png_path);
    console.log(`✅ 预览: ${result.width}x${result.height}`);
  } catch (error) {
    console.error('生成预览失败:', error);
  }
}
</script>
```

---

## 故障排除

### 问题 1: 中文显示为方框

**原因：** 字体未正确加载

**解决：** 确保 `assets/fonts/` 目录包含：
- `LiberationSans-Bold.ttf` (英文)
- `SourceHanSansSC-Bold.otf` (中文)

### 问题 2: 条形码无法扫描

**原因：** 打印机不支持原生 BARCODE 指令或条形码尺寸不合适

**解决：**
1. 切换到 `full_bitmap` 模式
2. 或调整条形码高度参数

### 问题 3: TSPL 指令过大

**原因：** 使用了 `full_bitmap` 模式

**解决：** 切换到 `text_bitmap_plus_native_barcode` 模式

---

## 更多信息

- [模板配置文档](./template.v2.md)
- [架构文档](../ARCHITECTURE.md)
- [GitHub Issues](https://github.com/your-repo/QSL-CardHub/issues)
