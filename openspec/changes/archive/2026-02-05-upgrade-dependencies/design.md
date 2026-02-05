## 上下文

项目使用多个 Rust crate，其中 6 个需要升级到新的主版本。这些升级包含破坏性 API 变更，需要修改现有代码以适配新版本。同时发现 `scripts/version.sh` 存在 bug，会误修改依赖版本号。

当前编译错误分为三类：
1. `rand 0.9` - `OsRng::fill_bytes` trait bounds 变更
2. `reqwest 0.13` - `.form()` 方法需要启用 `multipart` feature
3. `rusqlite 0.38` - `u64` 类型的 `FromSql` trait 实现变更

## 目标 / 非目标

**目标：**
- 将所有 6 个依赖升级到最新稳定版本
- 修复所有因 API 变更导致的编译错误
- 修复 `version.sh` 只更新 `[package]` 版本的问题
- 保持现有功能行为不变

**非目标：**
- 不利用新版本的新功能
- 不重构现有代码架构
- 不修改用户可见的行为

## 决策

### 1. rand 0.9 - OsRng API 适配

**问题**：`rand 0.9` 移除了 `OsRng` 的 `RngCore` trait 自动实现，需要显式导入。

**解决方案**：在 `src/security/encryption.rs` 中添加 `use rand::RngCore;` 导入。

**替代方案考虑**：
- 使用 `rand::thread_rng()` 替代 `OsRng` - 不采用，因为 `OsRng` 更适合密码学场景
- 保持 rand 0.8 - 不采用，因为需要跟进安全更新

### 2. reqwest 0.13 - form() 方法适配

**问题**：`reqwest 0.13` 将 `.form()` 方法移到了 `multipart` feature 下。

**解决方案**：在 `Cargo.toml` 中为 `reqwest` 添加 `multipart` feature：
```toml
reqwest = { version = "0.13.1", features = ["blocking", "cookies", "json", "multipart"] }
```

**受影响文件**：
- `src/qrz/qrz_cn_client.rs` (2 处)
- `src/qrz/qrz_com_client.rs` (1 处)
- `src/sf_express/client.rs` (2 处)

### 3. rusqlite 0.38 - u64 类型适配

**问题**：`rusqlite 0.38` 改变了 `u64` 类型的 `FromSql` 实现。

**解决方案**：检查所有 `row.get()` 调用，将需要的 `u64` 改为 `i64` 后再转换，或使用明确的类型标注。

**受影响模块**：
- `src/db/projects.rs` - 卡片计数字段
- `src/db/sf_express.rs` - 快递记录

### 4. version.sh 修复

**问题**：sed 命令 `s/^version = /` 会匹配所有以 `version = ` 开头的行，包括 `[dependencies.ts-rs]` 下的版本号。

**解决方案**：使用 awk 替代 sed，只在 `[package]` section 内更新版本：
```bash
awk -v ver="$new_version" '
    /^\[package\]/ { in_package=1 }
    /^\[/ && !/^\[package\]/ { in_package=0 }
    in_package && /^version = / { $0="version = \"" ver "\"" }
    { print }
' Cargo.toml > Cargo.toml.tmp && mv Cargo.toml.tmp Cargo.toml
```

## 风险 / 权衡

| 风险 | 缓解措施 |
|------|----------|
| 升级后可能有未发现的行为变化 | 运行完整测试套件，手动测试关键路径 |
| rusqlite u64 变更可能影响数据读取 | 仔细检查所有 `row.get()` 调用的类型 |
| reqwest multipart feature 增加编译体积 | 可接受，功能必需 |
| awk 在某些系统上行为可能不同 | 使用 POSIX 兼容语法 |
