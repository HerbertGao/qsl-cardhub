## 为什么

项目依赖的多个 crate 已发布新的主版本，包含安全修复、性能改进和新功能。当前版本已落后多个大版本，继续使用旧版本会增加安全风险和维护成本。现在升级可以确保项目保持在活跃维护的依赖版本上。

## 变更内容

升级以下依赖到最新稳定版本：

| 依赖 | 旧版本 | 新版本 | 破坏性变更 |
|------|--------|--------|------------|
| rusqlite | 0.32 | 0.38.0 | **BREAKING** - `u64` 类型处理变更 |
| rand | 0.8 | 0.9.2 | **BREAKING** - `OsRng` API 变更 |
| reqwest | 0.12 | 0.13.1 | **BREAKING** - `.form()` 需要 `multipart` feature |
| scraper | 0.22 | 0.25.0 | 可能有 API 变更 |
| rust_xlsxwriter | 0.79 | 0.93.0 | 可能有 API 变更 |
| ts-rs | 10 | 12.0.1 | 可能有 API 变更 |

同时修复 `scripts/version.sh` 中的 bug：sed 替换模式过于宽泛，会误修改依赖版本号。

## 功能 (Capabilities)

### 新增功能

无新增功能，此变更仅为依赖升级和 bug 修复。

### 修改功能

无规范级别的行为变更。所有变更都是实现层面的 API 适配，不影响用户可见的功能行为。

## 影响

### 受影响的代码

- **凭据存储模块** (`src/credentials/`): `rand` API 变更影响 `OsRng` 使用
- **数据库模块** (`src/db/`): `rusqlite` API 变更影响 `u64` 类型查询
- **HTTP 客户端模块**: `reqwest` API 变更影响表单提交功能
- **QRZ 集成**: 使用 `reqwest` 的表单提交
- **顺丰集成**: 使用 `reqwest` 的表单提交

### 受影响的依赖

- Cargo.toml 中的版本声明
- Cargo.lock 将完全更新

### 受影响的工具

- `scripts/version.sh`: 需要修复 sed 模式，仅更新 `[package]` 部分的版本号
