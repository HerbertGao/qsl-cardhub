## 上下文

本项目是一个 Tauri 2.0 应用，使用 Rust 后端和 TypeScript 前端。当前状态：

- **Rust 类型定义**：分布在 `src/db/models.rs`、`src/sf_express/models.rs`、`src/config/models.rs`、`src/config/template.rs` 等文件中，约 30+ 个结构体和枚举
- **TypeScript 类型定义**：手动维护在 `web/src/types/models.ts`，约 600+ 行代码
- **序列化**：所有 Rust 类型已使用 `#[derive(Serialize, Deserialize)]`，部分类型使用 serde 属性（如 `rename_all`）
- **构建工具**：使用 Cargo 构建 Rust，pnpm 构建前端

## 目标 / 非目标

**目标：**

1. 从 Rust 类型自动生成 TypeScript 定义，消除手动同步的维护负担
2. 确保生成的类型与 serde JSON 序列化结果完全一致
3. 提供简单的类型生成命令，支持开发和 CI 环境
4. 平滑迁移现有代码，不破坏现有功能

**非目标：**

1. 不生成 Tauri command 的类型签名（仅生成数据模型类型）
2. 不修改 Rust 类型的序列化行为
3. 不实现运行时类型验证（类型守卫为可选功能，初期不实现）
4. 不更改前端的 invoke 调用方式

## 决策

### 决策 1：使用 ts-rs 而非其他方案

**选择**：使用 [ts-rs](https://github.com/Aleph-Alpha/ts-rs) crate

**考虑的替代方案**：

| 方案 | 优点 | 缺点 |
|------|------|------|
| ts-rs | 成熟、活跃维护、支持 serde 属性 | 需要手动添加 derive 宏 |
| specta | 专为 Tauri 设计、可生成 command 类型 | 侵入性强、需要修改 command 定义方式 |
| typeshare | 支持多语言输出 | serde 兼容性较差 |
| 手动维护 | 无需额外依赖 | 易出错、维护负担重 |

**理由**：ts-rs 提供最佳的 serde 兼容性，且对现有代码侵入性最小，只需添加 derive 宏即可。

### 决策 2：类型导出策略

**选择**：使用 `#[ts(export)]` 属性配合测试模块触发导出

**实现方式**：
```rust
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../web/src/types/generated/")]
pub struct Card { ... }
```

通过运行 `cargo test export_bindings` 触发类型生成。

**理由**：这是 ts-rs 推荐的方式，不需要额外的构建脚本，且与 Cargo 生态无缝集成。

### 决策 3：生成文件组织

**选择**：单目录多文件结构

**目录结构**：
```
web/src/types/
├── generated/           # 自动生成的类型（不要手动编辑）
│   ├── Card.ts
│   ├── CardStatus.ts
│   ├── Project.ts
│   └── ...
├── models.ts            # 重导出 + 手动补充类型
└── tauri.ts             # Tauri command 参数类型（保留手动维护）
```

**理由**：
- 分离自动生成和手动维护的代码，便于 git 管理
- 保留 `models.ts` 作为统一入口，减少现有导入路径的变更
- `generated/` 目录可添加到 `.gitignore`（可选）或提交到仓库

### 决策 4：serde 属性处理

**选择**：启用 ts-rs 的 serde 兼容模式

**配置**：
```rust
#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]  // ts-rs 会自动识别
pub enum CardStatus { ... }
```

**理由**：确保生成的 TypeScript 类型与实际 JSON 序列化结果一致，这是使用 ts-rs 的核心价值。

### 决策 5：构建流程集成

**选择**：添加 pnpm script 触发类型生成

**package.json 配置**：
```json
{
  "scripts": {
    "generate:types": "cd .. && cargo test export_bindings --features ts-rs",
    "prebuild": "pnpm generate:types"
  }
}
```

**理由**：
- 开发者可手动运行 `pnpm generate:types`
- 构建前自动运行，确保类型始终最新
- CI 中可添加类型同步检查

## 风险 / 权衡

### 风险 1：ts-rs 版本兼容性

**风险**：ts-rs 可能与某些 Rust 类型或 serde 属性不兼容

**缓解措施**：
- 先在少量核心类型上验证，确认兼容后再全面铺开
- 对于不支持的类型，保留手动定义的 TypeScript 类型

### 风险 2：构建时间增加

**风险**：添加 `#[derive(TS)]` 可能增加编译时间

**缓解措施**：
- 使用 feature flag 控制 ts-rs 依赖，仅在需要生成类型时启用
- 类型生成不需要每次构建都运行，仅在类型变更时执行

### 风险 3：生成文件管理

**风险**：自动生成的文件与手动修改冲突

**缓解措施**：
- 在生成的文件顶部添加 `// This file is auto-generated. Do not edit.` 注释
- 考虑将 `generated/` 添加到 `.gitignore`，每次构建时重新生成

### 风险 4：迁移期间的类型断裂

**风险**：迁移过程中可能导致类型不一致

**缓解措施**：
- 分阶段迁移：先添加生成，再移除手动类型
- 保留 `models.ts` 作为统一入口，逐步切换导入源
