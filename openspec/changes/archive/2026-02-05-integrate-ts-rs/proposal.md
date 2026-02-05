## 为什么

当前项目在 Rust 后端 (`src/db/models.rs`, `src/sf_express/models.rs` 等) 和 TypeScript 前端 (`web/src/types/models.ts`) 中分别手动维护类型定义。这导致：
1. **维护负担**：每次修改 Rust 结构体后，必须手动同步 TypeScript 类型
2. **潜在不一致**：已发现 `return_card_cmd` 的 `method` vs `reason` 字段不一致、SF Express 订单信息的序列化处理差异等问题
3. **运行时错误风险**：类型不匹配只能在运行时发现，无法在编译期捕获

引入 [ts-rs](https://github.com/Aleph-Alpha/ts-rs) 可以从 Rust 类型自动生成 TypeScript 定义，确保类型始终同步。

## 变更内容

1. **添加 ts-rs 依赖**：在 `Cargo.toml` 中引入 `ts-rs` crate
2. **为 Rust 类型添加 derive 宏**：在需要导出的结构体和枚举上添加 `#[derive(TS)]` 和 `#[ts(...)]` 属性
3. **配置类型导出**：设置导出路径，生成 `.ts` 文件到 `web/src/types/generated/`
4. **创建构建脚本**：添加自动生成 TypeScript 类型的构建步骤
5. **迁移现有类型**：将 `web/src/types/models.ts` 中的手动类型定义替换为自动生成的类型
6. **移除冗余代码**：删除手动维护的重复类型定义

## 功能 (Capabilities)

### 新增功能

- `ts-rs-codegen`: 从 Rust 类型自动生成 TypeScript 定义的代码生成能力，包括 derive 宏配置、导出脚本和 CI 集成

### 修改功能

（无规范级行为变更，仅实现层面的类型定义迁移）

## 影响

- **Rust 代码** (`src/db/models.rs`, `src/sf_express/models.rs`, `src/config/models.rs`, `src/config/template.rs`)：需要为约 30+ 个结构体和枚举添加 `#[derive(TS)]`
- **TypeScript 代码** (`web/src/types/`)：`models.ts` 将被自动生成的类型替换
- **构建流程**：新增类型生成步骤，需要在前端构建前运行
- **依赖项**：新增 `ts-rs` crate（仅开发依赖，不影响运行时）
- **CI/CD**：可选添加类型同步检查，确保生成的类型与代码库同步
