# 提案：构建和打包系统

**状态**：📋 提案中
**最后更新**：2026-01-21

---

## 为什么

当前项目已完成核心功能开发（v0.1-v0.3），但缺少规范化的构建和打包流程，导致以下问题：

1. **手动打包流程不明确**：开发者需要手动执行多个命令（前端构建、Rust 编译、Tauri 打包），容易遗漏步骤或配置错误
2. **无自动化发布流程**：每次发布新版本需要在多个平台手动构建，耗时且容易出错
3. **缺少构建脚本**：没有统一的脚本简化本地构建和测试流程
4. **无 CI/CD 集成**：无法自动化测试、构建和发布流程，影响开发效率和版本质量

**解决方案价值：**

- ✅ **自动化发布**：通过 GitHub Actions 自动构建 macOS 和 Windows 版本
- ✅ **简化本地开发**：提供构建脚本，一键完成前端构建和应用打包
- ✅ **版本管理**：自动化版本号管理和 Release Notes 生成
- ✅ **质量保证**：集成测试和构建验证，确保发布质量

## 变更内容

实现完整的构建和打包系统，支持本地手动打包和 GitHub Actions 自动打包。

### 1. 手动打包脚本

**目标平台：**
- macOS (arm64 + x86_64 universal binary)
- Windows (x64)
- Windows (ARM64) - 原生构建（使用 GitHub Actions `windows-11-arm` runner）

**脚本功能：**
- 自动安装依赖（检测并安装 Node.js 依赖、Rust 依赖）
- 前端构建（执行 `npm run build`）
- Tauri 打包（执行 `cargo tauri build`）
- 输出产物整理（复制到 `dist/` 目录，统一命名格式）
- 构建验证（检查产物完整性、文件大小）

**脚本位置：**
- `scripts/build.sh` - macOS/Linux 构建脚本
- `scripts/build.ps1` - Windows 构建脚本（PowerShell）
- `scripts/build.bat` - Windows 构建脚本（CMD，调用 PowerShell）

### 2. GitHub Actions 自动打包

**触发条件：**
- 手动触发（`workflow_dispatch`）
- 标签推送（`tags: v*`）
- Pull Request 到 master（仅构建验证，不发布）

**构建矩阵：**
- macOS (arm64 + x86_64 universal) - `macos-latest`
- Windows x64 - `windows-latest`
- Windows ARM64 - `windows-11-arm` (原生 runner，public preview)

**工作流步骤：**
1. 环境准备（检出代码、安装 Rust、Node.js）
2. 依赖安装（cargo、npm）
3. 前端构建
4. Tauri 打包
5. 产物上传（GitHub Artifacts）
6. 创建 Release（仅在 tag 推送时）

**产物命名规范：**
- macOS: `qsl-cardhub-v{version}-macos-universal.dmg`
- Windows x64: `qsl-cardhub-v{version}-windows-x64.msi`
- Windows ARM64: `qsl-cardhub-v{version}-windows-arm64.msi`

### 3. 版本管理

**版本号来源：**
- 统一使用 `Cargo.toml` 中的 `version` 字段
- 自动同步到 `tauri.conf.json`

**版本更新流程：**
1. 手动更新 `Cargo.toml` 中的版本号
2. 运行 `scripts/sync-version.sh` 同步到其他文件
3. 提交版本更新
4. 创建 Git 标签：`git tag v{version}`
5. 推送标签触发 CI 构建：`git push origin v{version}`

### 4. 构建配置优化

**Cargo.toml 优化：**
- 已配置生产构建优化（`opt-level = "z"`, `lto = true`, `strip = true`）
- 保持当前配置不变

**Tauri 配置优化：**
- 配置 bundle identifier
- 配置应用图标（已有）
- 配置目标平台

## 范围

### 包含的功能

1. ✅ **手动打包脚本**
   - macOS 构建脚本（Shell）
   - Windows 构建脚本（PowerShell + Batch）
   - 依赖检查和安装
   - 构建产物验证

2. ✅ **GitHub Actions 工作流**
   - macOS 自动构建
   - Windows 自动构建
   - 产物上传和 Release 创建
   - 构建缓存优化

3. ✅ **版本管理工具**
   - 版本同步脚本
   - 版本号验证

4. ✅ **文档更新**
   - README 中的构建说明
   - 脚本使用文档

### 不包含的功能

1. ❌ **Linux 打包**：暂不支持 Linux 打包（可在后续版本添加）
2. ❌ **自动更新功能**：暂不实现应用内自动更新（可在后续版本添加）
3. ❌ **代码签名**：暂不配置代码签名（需要开发者证书）
4. ❌ **公证（Notarization）**：暂不配置 macOS 公证（需要 Apple 开发者账号）

## 影响

### 新增文件

- `scripts/build.sh` - macOS/Linux 构建脚本
- `scripts/build.ps1` - Windows 构建脚本
- `scripts/build.bat` - Windows 构建脚本入口
- `scripts/sync-version.sh` - 版本同步脚本
- `.github/workflows/build.yml` - GitHub Actions 构建工作流
- `.github/workflows/release.yml` - GitHub Actions 发布工作流

### 修改文件

- `README.md` - 添加构建和发布说明
- `tauri.conf.json` - 优化 bundle 配置

### 受影响规范

- 需要新增规范：`build-and-packaging`（构建和打包）

## 验收标准

### 手动打包脚本

- [ ] macOS 脚本能成功构建 DMG 文件
- [ ] Windows 脚本能成功构建 MSI 文件
- [ ] 脚本能检测并安装缺失的依赖
- [ ] 构建产物能正常安装和运行
- [ ] 构建时间在合理范围内（< 10 分钟）

### GitHub Actions

- [ ] tag 推送触发自动构建
- [ ] macOS 和 Windows 构建并行执行
- [ ] 构建产物自动上传到 Artifacts
- [ ] 自动创建 GitHub Release
- [ ] Release 包含正确命名的安装包
- [ ] CI 构建时间在合理范围内（< 15 分钟）

### 版本管理

- [ ] 版本号在 Cargo.toml 和 tauri.conf.json 中保持一致
- [ ] 版本同步脚本能正确更新所有文件
- [ ] Git 标签格式正确（`v{major}.{minor}.{patch}`）

### 文档

- [ ] README 包含完整的构建说明
- [ ] 脚本包含使用说明和错误提示
- [ ] GitHub Actions 工作流包含清晰的注释

## 风险和缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 构建环境差异导致失败 | 中 | 使用 GitHub Actions 标准环境，本地脚本做环境检测 |
| 构建产物体积过大 | 低 | 已配置编译优化（`opt-level = "z"`, `strip = true`） |
| macOS 应用无法安装（未签名） | 中 | 文档说明用户需要允许"任何来源"的应用 |
| Windows 应用被 SmartScreen 拦截 | 中 | 文档说明用户需要点击"仍要运行" |
| GitHub Actions 配额不足 | 低 | 优化构建缓存，减少构建时间 |
| Windows ARM64 runner 不稳定（public preview） | 低 | 监控 GitHub 公告，必要时回退到交叉编译 |
| Windows ARM64 打印机驱动兼容性 | 中 | 文档说明需要 ARM64 兼容的驱动，提供测试指南 |

## 时间估算

- **手动打包脚本**：4-6 小时
- **GitHub Actions 配置**：4-6 小时
- **版本管理工具**：2-3 小时
- **文档更新**：2-3 小时
- **测试和验证**：3-4 小时

**总计**：15-22 小时（约 2-3 工作日）

## 依赖关系

- **前置依赖**：
  - ✅ Tauri 项目已配置完成
  - ✅ 前端构建流程已就绪
  - ✅ 应用图标已准备

- **并行任务**：
  - 手动脚本和 GitHub Actions 可以并行开发
  - 版本管理工具可以独立开发

- **后续任务**：
  - Linux 打包支持（可选）
  - 代码签名和公证（可选）
  - 自动更新功能（可选）
