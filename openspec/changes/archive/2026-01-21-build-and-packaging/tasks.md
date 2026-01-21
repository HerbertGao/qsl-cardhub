# 实施任务清单

本文档列出了构建和打包系统的详细实施任务。任务按照依赖关系和优先级排序。

---

## 阶段 1：环境准备和基础脚本（第 1 天）

### 任务 1.1：创建脚本目录和基础结构

**描述**：创建 `scripts/` 目录和脚本模板。

**步骤**：
1. 创建 `scripts/` 目录
2. 创建 `.gitignore` 忽略临时文件

**验证**：
- `scripts/` 目录已创建

---

### 任务 1.2：macOS 构建脚本

**描述**：实现 macOS 平台的完整构建脚本。

**步骤**：
1. 创建 `scripts/build.sh`
2. 实现依赖检查（Node.js、npm、Rust、cargo）
3. 实现前端构建（`cd web && npm run build`）
4. 实现 Tauri 打包（`cargo tauri build`）
5. 实现产物复制到 `dist/` 目录
6. 实现构建验证（检查文件存在、大小合理）
7. 添加彩色输出和进度提示

**验证**：
- 脚本能检测缺失的依赖并提示安装
- 脚本能成功构建 DMG 文件
- 产物复制到 `dist/qsl-cardhub-{version}-macos-universal.dmg`
- 构建日志清晰易读

---

### 任务 1.3：Windows 构建脚本

**描述**：实现 Windows 平台的完整构建脚本。

**步骤**：
1. 创建 `scripts/build.ps1`（PowerShell）
2. 创建 `scripts/build.bat`（CMD 入口，调用 PowerShell）
3. 实现依赖检查（Node.js、npm、Rust、cargo）
4. 实现前端构建
5. 实现 Tauri 打包
6. 实现产物复制到 `dist/` 目录
7. 添加彩色输出和进度提示

**验证**：
- 脚本能检测缺失的依赖并提示安装
- 脚本能成功构建 MSI 文件
- 产物复制到 `dist/qsl-cardhub-{version}-windows-x64.msi`
- 构建日志清晰易读
- 支持通过 CMD 或 PowerShell 执行

---

## 阶段 2：版本管理（第 1 天）

### 任务 2.1：版本同步脚本

**描述**：实现版本号自动同步工具。

**步骤**：
1. 创建 `scripts/sync-version.sh`
2. 从 `Cargo.toml` 读取版本号
3. 更新 `tauri.conf.json` 中的版本号
4. 验证版本号格式（semver）

**验证**：
- 脚本能正确提取 `Cargo.toml` 版本号
- 脚本能更新 `tauri.conf.json` 版本号
- 版本号格式验证正确

---

## 阶段 3：GitHub Actions 工作流（第 2 天）

### 任务 3.1：创建 GitHub Actions 目录

**描述**：创建 `.github/workflows/` 目录结构。

**步骤**：
1. 创建 `.github/workflows/` 目录
2. 创建 `.github/workflows/README.md` 说明文档

**验证**：
- 目录结构正确

---

### 任务 3.2：构建工作流（build.yml）

**描述**：实现自动构建工作流，用于 Pull Request 验证。

**步骤**：
1. 创建 `.github/workflows/build.yml`
2. 配置触发条件（PR to master、手动触发）
3. 配置 macOS 构建 job
   - 安装 Rust
   - 安装 Node.js
   - 安装依赖
   - 前端构建
   - Tauri 打包
   - 上传产物到 Artifacts
4. 配置 Windows x64 构建 job（并行执行）
   - 相同步骤
5. 配置 Windows ARM64 构建 job（并行执行，使用 `windows-11-arm` runner）
   - 安装 Rust 和 Node.js
   - 安装依赖
   - 前端构建
   - Tauri 打包（自动使用 ARM64 架构）
   - 上传产物到 Artifacts
6. 配置构建缓存（cargo、npm）
7. 添加构建状态通知

**验证**：
- PR 触发自动构建
- macOS、Windows x64 和 Windows ARM64 并行构建
- 构建成功后产物上传到 Artifacts
- Windows ARM64 使用原生 runner 构建成功
- 构建失败时有清晰的错误信息

---

### 任务 3.3：发布工作流（release.yml）

**描述**：实现自动发布工作流，用于版本发布。

**步骤**：
1. 创建 `.github/workflows/release.yml`
2. 配置触发条件（tags: v*）
3. 复用 `build.yml` 的构建逻辑
4. 添加 Release 创建步骤
   - 使用 `softprops/action-gh-release`
   - 自动生成 Release Notes
   - 上传构建产物
5. 配置 Release 命名格式

**验证**：
- tag 推送触发自动构建
- 构建成功后自动创建 Release
- Release 包含正确的安装包
- Release Notes 自动生成

---

## 阶段 4：配置优化（第 2 天）

### 任务 4.1：优化 Tauri 配置

**描述**：优化 `tauri.conf.json` 中的 bundle 配置。

**步骤**：
1. 验证 `identifier` 正确（com.herbertgao.qsl-cardhub）
2. 验证 `icon` 路径正确
3. 配置 bundle 目标平台
4. 添加应用描述和版权信息

**验证**：
- 配置文件格式正确
- bundle 配置生效

---

### 任务 4.2：配置构建缓存

**描述**：优化 GitHub Actions 构建速度。

**步骤**：
1. 配置 Cargo 缓存（`actions/cache`）
2. 配置 npm 缓存
3. 测试缓存效果

**验证**：
- 第二次构建速度显著提升（< 5 分钟）
- 缓存命中率高（> 80%）

---

## 阶段 5：文档和测试（第 3 天）

### 任务 5.1：更新 README

**描述**：更新 README 中的构建和发布说明。

**步骤**：
1. 添加"构建"章节
   - 本地构建说明（macOS、Windows）
   - 脚本使用说明
2. 添加"发布"章节
   - 版本更新流程
   - Release 创建流程
3. 添加"下载"章节
   - GitHub Release 下载链接
   - 安装说明

**验证**：
- 文档清晰易懂
- 所有命令可执行

---

### 任务 5.2：脚本使用文档

**描述**：为构建脚本添加详细的使用说明。

**步骤**：
1. 创建 `scripts/README.md`
2. 说明每个脚本的功能
3. 说明依赖要求
4. 添加常见问题（FAQ）

**验证**：
- 文档完整

---

### 任务 5.3：本地构建测试

**描述**：在实际环境中测试构建脚本。

**步骤**：
1. macOS 测试
   - 清理 `dist/` 目录
   - 执行 `scripts/build.sh`
   - 验证 DMG 文件生成
   - 安装并运行应用
2. Windows 测试
   - 清理 `dist/` 目录
   - 执行 `scripts/build.bat`
   - 验证 MSI 文件生成
   - 安装并运行应用

**验证**：
- 两个平台的构建都成功
- 安装包能正常安装和运行
- 应用功能正常

---

### 任务 5.4：GitHub Actions 测试

**描述**：测试 GitHub Actions 工作流。

**步骤**：
1. 创建测试分支
2. 提交代码并创建 PR
3. 验证 `build.yml` 触发并成功
4. 创建测试 tag（如 `v0.1.0-test`）
5. 验证 `release.yml` 触发并成功
6. 检查 Release 是否正确创建

**验证**：
- 所有工作流执行成功
- 产物正确上传
- Release 正确创建

---

### 任务 5.5：版本发布演练

**描述**：完整演练版本发布流程。

**步骤**：
1. 更新 `Cargo.toml` 版本号（如 v0.4.0）
2. 执行 `scripts/sync-version.sh`
3. 提交版本更新
4. 创建 Git 标签：`git tag v0.4.0`
5. 推送标签：`git push origin v0.4.0`
6. 观察 GitHub Actions 执行
7. 验证 Release 创建成功
8. 下载并测试发布的安装包

**验证**：
- 版本号正确同步
- CI 自动触发
- Release 正确创建
- 安装包可用

---

## 总计

**总估算时间**：15-22 小时（2-3 工作日）

**关键里程碑**：
1. ✅ 第 1 天：手动构建脚本和版本管理完成
2. ✅ 第 2 天：GitHub Actions 工作流完成
3. ✅ 第 3 天：文档、测试和发布演练完成

**并行任务建议**：
- 任务 1.2 和 1.3 可以并行（macOS 和 Windows 脚本）
- 任务 3.2 和 3.3 可以并行（构建和发布工作流）
- 任务 5.3 和 5.4 可以并行（本地和 CI 测试）
