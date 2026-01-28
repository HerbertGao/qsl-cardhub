# printer-backend Specification

## Purpose
TBD - created by archiving change refactor-printer-backend-abstraction. Update Purpose after archive.
## 需求
### 需求：打印机后端图像打印接口

系统必须提供统一的图像打印接口，屏蔽不同打印机后端的实现差异。

#### 场景：通过 PDF 测试打印机打印图像

- **当** 调用方请求打印图像到 "PDF 测试打印机"
- **那么** 系统将图像保存为 PNG 文件到下载目录
- **并且** 返回包含文件路径的成功消息

#### 场景：通过系统打印机打印图像

- **当** 调用方请求打印图像到系统打印机
- **那么** 系统将图像转换为 TSPL 指令
- **并且** 通过系统打印 API 发送到打印机
- **并且** 返回包含作业 ID 的成功消息

### 需求：打印机后端所有权声明

每个打印机后端必须能够声明它是否拥有（管理）特定的打印机。

#### 场景：PDF 后端声明所有权

- **当** 查询 PDF 后端是否拥有 "PDF 测试打印机"
- **那么** 返回 true
- **当** 查询 PDF 后端是否拥有其他打印机
- **那么** 返回 false

#### 场景：系统后端声明所有权

- **当** 查询系统后端是否拥有 "PDF 测试打印机"
- **那么** 返回 false
- **当** 查询系统后端是否拥有系统打印机
- **那么** 返回 true

### 需求：打印机名称常量定义

PDF 测试打印机的名称必须通过常量定义，避免魔法字符串。

#### 场景：使用常量引用打印机名称

- **当** 代码需要引用 PDF 测试打印机名称
- **那么** 必须使用 `PDF_TEST_PRINTER_NAME` 常量
- **并且** 该常量从 `printer::backend` 模块导出

### 需求：统一打印路由

`PrinterState` 必须提供统一的图像打印方法，自动路由到正确的后端。

#### 场景：自动路由到 PDF 后端

- **当** 调用 `print_image_to_printer()` 并指定 "PDF 测试打印机"
- **那么** 系统自动使用 PDF 后端处理请求

#### 场景：自动路由到系统后端

- **当** 调用 `print_image_to_printer()` 并指定系统打印机名称
- **那么** 系统自动使用系统后端处理请求

### 需求：TSPL 图像生成

TSPLGenerator 必须支持直接从灰度图像生成完整的 TSPL 打印指令。

#### 场景：从图像生成 TSPL

- **当** 调用 `generate_from_image()` 并提供灰度图像和纸张尺寸
- **那么** 生成包含 SIZE、GAP、DIRECTION、CLS、BITMAP、PRINT 的完整 TSPL 指令

