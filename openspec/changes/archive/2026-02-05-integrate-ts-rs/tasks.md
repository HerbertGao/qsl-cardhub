## 1. 依赖配置

- [x] 1.1 在 Cargo.toml 中添加 ts-rs 依赖（使用 feature flag）
- [x] 1.2 创建 `web/src/types/generated/` 目录
- [x] 1.3 在 `web/src/types/generated/` 添加 .gitkeep 文件

## 2. 核心类型导出（src/db/models.rs）

- [x] 2.1 为 `CardStatus` 枚举添加 `#[derive(TS)]` 和 `#[ts(export)]`
- [x] 2.2 为 `Card` 结构体添加 `#[derive(TS)]` 和 `#[ts(export)]`
- [x] 2.3 为 `CardWithProject` 结构体添加 `#[derive(TS)]` 和 `#[ts(export)]`
- [x] 2.4 为 `CardMetadata` 结构体添加 `#[derive(TS)]` 和 `#[ts(export)]`
- [x] 2.5 为 `DistributionInfo` 结构体添加 `#[derive(TS)]` 和 `#[ts(export)]`
- [x] 2.6 为 `ReturnInfo` 结构体添加 `#[derive(TS)]` 和 `#[ts(export)]`
- [x] 2.7 为 `AddressEntry` 结构体添加 `#[derive(TS)]` 和 `#[ts(export)]`
- [x] 2.8 为 `Project` 结构体添加 `#[derive(TS)]` 和 `#[ts(export)]`
- [x] 2.9 为 `ProjectWithStats` 结构体添加 `#[derive(TS)]` 和 `#[ts(export)]`
- [x] 2.10 为 `PagedCards` 结构体添加 `#[derive(TS)]` 和 `#[ts(export)]`

## 3. SF Express 类型导出（src/sf_express/models.rs）

- [x] 3.1 为 `SenderInfo` 结构体添加 `#[derive(TS)]` 和 `#[ts(export)]`
- [x] 3.2 为 `RecipientInfo` 结构体添加 `#[derive(TS)]` 和 `#[ts(export)]`
- [x] 3.3 为 `SFOrder` 结构体添加 `#[derive(TS)]` 和 `#[ts(export)]`
- [x] 3.4 为 `SFOrderWithCard` 结构体添加 `#[derive(TS)]` 和 `#[ts(export)]`
- [x] 3.5 为 `OrderStatus` 枚举添加 `#[derive(TS)]` 和 `#[ts(export)]`

## 4. 配置类型导出（src/config/）

- [x] 4.1 为 `Profile` 结构体添加 `#[derive(TS)]` 和 `#[ts(export)]`
- [x] 4.2 为 `Platform` 结构体添加 `#[derive(TS)]` 和 `#[ts(export)]`
- [x] 4.3 为 `PrinterConfig` 结构体添加 `#[derive(TS)]` 和 `#[ts(export)]`
- [x] 4.4 为模板相关类型添加 `#[derive(TS)]` 和 `#[ts(export)]`

## 5. 类型生成测试

- [x] 5.1 创建 `tests/export_bindings.rs` 测试文件，触发类型导出
- [x] 5.2 运行 `cargo test export_bindings` 验证类型生成成功
- [x] 5.3 检查生成的 TypeScript 文件格式是否正确

## 6. 前端集成

- [x] 6.1 在 `web/package.json` 添加 `generate:types` 脚本
- [x] 6.2 更新 `web/src/types/models.ts`，从 `generated/` 重导出类型
- [x] 6.3 移除 `models.ts` 中已被生成的手动类型定义
- [x] 6.4 验证前端构建无错误

## 7. 验证与清理

- [x] 7.1 对比生成的类型与原有手动类型，确认一致性
- [x] 7.2 运行前端测试，确保类型兼容（N/A：项目未配置前端测试）
- [x] 7.3 更新 `.gitignore`（可选：忽略 generated/ 目录）（跳过：生成的文件应提交版本控制）
- [x] 7.4 在生成的文件中添加 "do not edit" 注释头（ts-rs 自动添加）
