## 新增需求

### 需求：Rust 类型必须可导出为 TypeScript 定义

所有需要与前端通信的 Rust 结构体和枚举必须支持自动导出为 TypeScript 类型定义。导出的类型必须与 Rust 原始类型保持语义一致。

#### 场景：导出基础结构体

- **当** Rust 结构体添加了 `#[derive(TS)]` 宏
- **那么** 运行导出命令后生成对应的 TypeScript interface 定义

#### 场景：导出枚举类型

- **当** Rust 枚举添加了 `#[derive(TS)]` 宏
- **那么** 运行导出命令后生成对应的 TypeScript union type 定义

#### 场景：处理嵌套类型

- **当** 结构体字段引用了其他已标记的类型
- **那么** 生成的 TypeScript 定义必须正确引用被依赖的类型

### 需求：可选字段必须正确映射

Rust 中的 `Option<T>` 类型必须映射为 TypeScript 的可选字段（`field?: T | null`），确保空值语义一致。

#### 场景：Option 字段导出

- **当** Rust 结构体包含 `Option<String>` 类型的字段
- **那么** 生成的 TypeScript 定义中该字段必须标记为可选且允许 null

#### 场景：必填字段导出

- **当** Rust 结构体包含非 Option 的必填字段
- **那么** 生成的 TypeScript 定义中该字段必须为必填

### 需求：类型导出路径必须可配置

系统必须支持配置 TypeScript 类型文件的输出路径，默认导出到 `web/src/types/generated/` 目录。

#### 场景：使用默认导出路径

- **当** 未指定自定义导出路径
- **那么** 类型文件必须生成到 `web/src/types/generated/` 目录

#### 场景：导出路径不存在时自动创建

- **当** 配置的导出目录不存在
- **那么** 系统必须自动创建该目录

### 需求：必须提供类型生成命令

系统必须提供一个可执行的命令或脚本，用于触发 TypeScript 类型的生成。该命令必须可在开发和 CI 环境中运行。

#### 场景：手动触发类型生成

- **当** 开发者执行类型生成命令
- **那么** 系统必须扫描所有标记的 Rust 类型并生成对应的 TypeScript 文件

#### 场景：类型生成失败时报错

- **当** 类型生成过程中遇到不支持的类型或配置错误
- **那么** 系统必须输出明确的错误信息并以非零状态码退出

### 需求：生成的类型文件必须格式化

生成的 TypeScript 文件必须遵循项目的代码风格，输出格式化的代码以便于版本控制和代码审查。

#### 场景：生成格式化的 TypeScript 代码

- **当** 类型生成命令执行完成
- **那么** 输出的 `.ts` 文件必须是格式化的、可读的代码

### 需求：支持 serde 属性映射

ts-rs 必须正确处理 Rust 结构体上的 serde 属性（如 `#[serde(rename_all = "camelCase")]`），确保生成的 TypeScript 字段名与 JSON 序列化结果一致。

#### 场景：处理 camelCase 重命名

- **当** Rust 结构体标记了 `#[serde(rename_all = "camelCase")]`
- **那么** 生成的 TypeScript 字段名必须使用 camelCase 格式

#### 场景：处理字段重命名

- **当** Rust 字段标记了 `#[serde(rename = "customName")]`
- **那么** 生成的 TypeScript 定义必须使用 `customName` 作为字段名

### 需求：整数类型必须与 JSON 序列化结果一致

Rust 的 `i64`/`u64` 类型必须映射为 TypeScript 的 `number` 而非 `bigint`，以确保与 JSON 序列化结果一致。

**背景**：ts-rs 默认将 `i64`/`u64` 映射为 TypeScript 的 `bigint`，但 serde_json 将这些类型序列化为普通 JSON number，前端 JSON.parse 解析后得到的是 JavaScript `number` 而非 `bigint`。由于 `bigint` 和 `number` 在 TypeScript 中不兼容，会导致类型检查失败和运行时错误。

#### 场景：i64 字段导出

- **当** Rust 结构体包含 `i64` 类型的字段
- **那么** 必须使用 `#[ts(type = "number")]` 注解，使生成的 TypeScript 类型为 `number`

#### 场景：u64 字段导出

- **当** Rust 结构体包含 `u64` 类型的字段
- **那么** 必须使用 `#[ts(type = "number")]` 注解，使生成的 TypeScript 类型为 `number`

#### 场景：数值运算兼容性

- **当** 前端代码对导出的数值字段进行算术运算（如 `total || 0`）
- **那么** 类型检查必须通过，运行时行为必须正确

### 需求：导出的类型必须包含类型守卫（可选）

系统可以选择性地为导出的类型生成 TypeScript 类型守卫函数，用于运行时类型检查。

#### 场景：生成类型守卫函数

- **当** 配置启用了类型守卫生成
- **那么** 每个导出的类型必须附带一个 `isTypeName(value): value is TypeName` 函数

## 修改需求

（无）

## 移除需求

（无）
