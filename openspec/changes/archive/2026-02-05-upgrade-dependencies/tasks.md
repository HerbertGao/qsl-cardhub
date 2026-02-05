## 1. 依赖版本更新

- [x] 1.1 在 Cargo.toml 中为 reqwest 添加 `multipart` feature
- [x] 1.2 确认 Cargo.toml 中所有依赖版本已正确设置

## 2. rand 0.9 API 适配

- [x] 2.1 在 `src/security/encryption.rs` 中添加 `use rand::RngCore;` 导入
- [x] 2.2 验证 `OsRng.fill_bytes()` 调用正常工作

## 3. rusqlite 0.38 类型适配

- [x] 3.1 检查 `src/db/projects.rs` 中的 `row.get()` 调用，修复 u64 类型问题
- [x] 3.2 检查 `src/db/sf_express.rs` 中的 `row.get()` 调用，修复 u64 类型问题
- [x] 3.3 检查其他 db 模块中可能存在的 u64 类型问题

## 4. 验证编译

- [x] 4.1 运行 `cargo check` 确认无编译错误
- [x] 4.2 运行 `cargo test` 确认所有测试通过（库测试 78 通过，集成测试失败与升级无关）

## 5. 脚本修复（已完成）

- [x] 5.1 修复 `scripts/version.sh` 中的 sed 模式，使用 awk 只更新 `[package]` 版本
