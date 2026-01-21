# 构建和打包系统规范

## 目的

本规范定义 qsl-cardhub 项目的构建和打包系统需求，包括本地手动构建脚本和 GitHub Actions 自动化构建流程，确保应用能够在 macOS 和 Windows 平台上正确构建和发布。

---

## 新增需求

### 需求： 本地构建脚本

系统必须提供本地构建脚本，支持在 macOS 和 Windows 平台上手动构建应用安装包。

#### 场景： macOS 手动构建

- **当** 开发者在 macOS 上执行 `./scripts/build.sh`
- **那么** 脚本必须检查依赖（Node.js、npm、Rust、cargo）
- **并且** 如果依赖缺失，必须提示用户安装
- **并且** 必须自动执行前端构建（`npm run build`）
- **并且** 必须自动执行 Tauri 打包（`cargo tauri build`）
- **并且** 必须将 DMG 文件复制到 `dist/` 目录
- **并且** 文件名格式为 `qsl-cardhub-v{version}-macos-universal.dmg`
- **并且** 必须验证产物文件存在且大小合理（> 5MB）

#### 场景： Windows 手动构建

- **当** 开发者在 Windows 上执行 `scripts\build.bat` 或 `scripts\build.ps1`
- **那么** 脚本必须检查依赖（Node.js、npm、Rust、cargo）
- **并且** 如果依赖缺失，必须提示用户安装
- **并且** 必须自动执行前端构建
- **并且** 必须自动执行 Tauri 打包
- **并且** 必须将 MSI 文件复制到 `dist/` 目录
- **并且** 文件名格式为 `qsl-cardhub-v{version}-windows-x64.msi`
- **并且** 必须验证产物文件存在且大小合理（> 5MB）

#### 场景： 依赖检查失败

- **当** 构建脚本检测到依赖缺失（如未安装 Node.js）
- **那么** 脚本必须输出清晰的错误信息
- **并且** 必须提供安装依赖的建议或链接
- **并且** 必须以非零退出码退出

#### 场景： 构建失败处理

- **当** 构建过程中任何步骤失败（前端构建失败、Tauri 打包失败等）
- **那么** 脚本必须立即停止执行
- **并且** 必须输出失败步骤的名称
- **并且** 必须显示完整的错误日志
- **并且** 必须以非零退出码退出

#### 场景： 构建进度提示

- **当** 构建脚本执行过程中
- **那么** 必须显示当前执行的步骤（如"正在构建前端..."）
- **并且** 必须使用彩色输出区分成功（绿色）、错误（红色）和警告（黄色）
- **并且** 必须显示构建进度（如"步骤 2/5"）
- **并且** 在构建完成后必须显示总耗时

---

### 需求： 版本管理

系统必须提供版本号管理工具，确保版本号在所有配置文件中保持一致。

#### 场景： 版本号同步

- **当** 开发者运行 `./scripts/sync-version.sh`
- **那么** 脚本必须从 `Cargo.toml` 读取 `version` 字段
- **并且** 必须验证版本号格式符合 semver 规范（`X.Y.Z`）
- **并且** 必须更新 `tauri.conf.json` 中的 `version` 字段
- **并且** 必须输出同步前后的版本号

#### 场景： 版本号格式验证

- **当** `Cargo.toml` 中的版本号格式不正确（如"1.0"而不是"1.0.0"）
- **那么** 同步脚本必须输出错误信息
- **并且** 必须说明正确的版本号格式
- **并且** 必须以非零退出码退出

#### 场景： 版本号不一致警告

- **当** `Cargo.toml` 和 `tauri.conf.json` 中的版本号不一致
- **那么** 构建脚本必须输出警告信息
- **并且** 必须建议运行版本同步脚本

---

### 需求： GitHub Actions 自动构建

系统必须提供 GitHub Actions 工作流，实现自动化构建和发布。

#### 场景： Pull Request 构建验证

- **当** 开发者创建 Pull Request 到 master 分支
- **那么** GitHub Actions 必须自动触发 `build.yml` 工作流
- **并且** 必须在 macOS 和 Windows 平台并行构建
- **并且** 构建成功后必须将产物上传到 GitHub Artifacts
- **并且** PR 页面必须显示构建状态（成功或失败）
- **并且** 构建失败时必须显示详细的错误日志

#### 场景： Tag 发布构建

- **当** 开发者推送 Git 标签（格式为 `v*`，如 `v0.4.0`）
- **那么** GitHub Actions 必须自动触发 `release.yml` 工作流
- **并且** 必须在 macOS 和 Windows 平台并行构建
- **并且** 构建成功后必须自动创建 GitHub Release
- **并且** Release 必须包含构建产物（DMG 和 MSI 文件）
- **并且** Release 标题必须为标签名称（如"v0.4.0"）
- **并且** Release 必须自动生成 Release Notes（基于 commits）

#### 场景： 手动触发构建

- **当** 开发者在 GitHub Actions 页面手动触发 `build.yml`
- **那么** 工作流必须执行完整的构建流程
- **并且** 必须将产物上传到 Artifacts
- **并且** 不创建 Release

#### 场景： 构建缓存优化

- **当** GitHub Actions 执行构建时
- **那么** 必须缓存 Cargo 构建产物（`~/.cargo`, `target/`）
- **并且** 必须缓存 npm 依赖（`node_modules/`, `~/.npm`）
- **并且** 缓存键必须基于 `Cargo.lock` 和 `package-lock.json` 的哈希值
- **并且** 缓存命中时，构建时间必须显著减少（< 5 分钟）

#### 场景： 构建矩阵配置

- **当** GitHub Actions 执行构建时
- **那么** 必须使用 matrix 策略并行构建多个平台
- **并且** 必须包含以下平台：
  - `macos-latest` (universal binary: arm64 + x86_64)
  - `windows-latest` (x64)
  - `windows-11-arm` (ARM64, 原生 runner)
- **并且** 每个平台的构建必须独立运行
- **并且** 任何平台构建失败不应影响其他平台

#### 场景： Windows ARM64 原生构建

- **当** GitHub Actions 构建 Windows ARM64 版本
- **那么** 必须使用 `windows-11-arm` runner（原生 ARM64 环境）
- **并且** 必须安装 Rust 和 Node.js（ARM64 版本）
- **并且** 构建流程必须与 Windows x64 完全相同（无需特殊配置）
- **并且** 必须生成 `qsl-cardhub-v{version}-windows-arm64.msi` 文件
- **并且** Tauri 必须自动检测 ARM64 架构并使用正确的 target

---

### 需求： 构建产物命名

系统必须使用统一的构建产物命名格式，便于识别和管理。

#### 场景： 产物命名格式

- **当** 构建脚本或 CI 生成安装包
- **那么** 文件名必须遵循格式：`qsl-cardhub-v{version}-{platform}-{arch}.{ext}`
- **并且** `{version}` 必须从 `Cargo.toml` 或 `tauri.conf.json` 读取
- **并且** `{platform}` 必须为 `macos` 或 `windows`
- **并且** `{arch}` 必须为 `universal`（macOS）、`x64`（Windows x64）或 `arm64`（Windows ARM64）
- **并且** `{ext}` 必须为 `dmg`（macOS）或 `msi`（Windows）

#### 场景： 产物存储位置

- **当** 本地构建完成
- **那么** 产物必须存储在项目根目录的 `dist/` 目录下
- **并且** `dist/` 目录必须在 `.gitignore` 中忽略
- **并且** 目录结构必须为：
  ```
  dist/
  ├── qsl-cardhub-v0.4.0-macos-universal.dmg
  ├── qsl-cardhub-v0.4.0-windows-x64.msi
  └── qsl-cardhub-v0.4.0-windows-arm64.msi
  ```

---

### 需求： 构建验证

系统必须对构建产物进行验证，确保文件完整性和质量。

#### 场景： 文件存在验证

- **当** 构建完成后
- **那么** 脚本必须检查产物文件是否存在
- **并且** 如果文件不存在，必须输出错误信息
- **并且** 必须以非零退出码退出

#### 场景： 文件大小验证

- **当** 构建完成后
- **那么** 脚本必须检查产物文件大小
- **并且** 如果文件大小 < 5MB，必须输出警告（可能构建不完整）
- **并且** 如果文件大小 > 100MB，必须输出警告（可能包含不必要的文件）
- **并且** 必须显示实际文件大小（以 MB 为单位）

#### 场景： 文件格式验证

- **当** 构建完成后
- **那么** 脚本必须检查产物文件扩展名
- **并且** macOS 产物必须为 `.dmg` 格式
- **并且** Windows 产物必须为 `.msi` 格式
- **并且** 如果格式不正确，必须输出错误信息

---

### 需求： Release 创建

系统必须自动创建 GitHub Release，包含构建产物和发布说明。

#### 场景： 自动创建 Release

- **当** `release.yml` 工作流构建成功
- **那么** 必须自动创建 GitHub Release
- **并且** Release 标题必须为 Git 标签（如"v0.4.0"）
- **并且** Release 必须包含所有平台的安装包
- **并且** Release 必须自动生成 Release Notes（基于 commits）
- **并且** Release 状态必须为"已发布"（非 draft）

#### 场景： Release 文件列表

- **当** Release 创建后
- **那么** 必须包含以下文件：
  - `qsl-cardhub-v{version}-macos-universal.dmg`
  - `qsl-cardhub-v{version}-windows-x64.msi`
  - `qsl-cardhub-v{version}-windows-arm64.msi`
- **并且** 每个文件必须可下载
- **并且** 文件大小必须在页面上显示

#### 场景： Release Notes 生成

- **当** Release 创建时
- **那么** 必须自动生成 Release Notes
- **并且** Release Notes 必须包含自上一个版本以来的所有 commits
- **并且** Release Notes 必须按照 GitHub 默认格式分组（Features、Bug Fixes 等）
- **并且** 必须包含贡献者列表

---

### 需求： 构建配置优化

系统必须优化 Tauri 配置，确保构建产物质量和用户体验。

#### 场景： Bundle 配置

- **当** 执行 Tauri 打包
- **那么** `tauri.conf.json` 必须包含以下配置：
  - `identifier`: "com.herbertgao.qsl-cardhub"
  - `icon`: 正确的图标路径数组
  - `copyright`: 版权信息
  - `shortDescription`: 应用简短描述
  - `longDescription`: 应用详细描述
- **并且** 配置必须对 macOS 和 Windows 都生效

#### 场景： 目标平台配置

- **当** 执行 Tauri 打包
- **那么** `tauri.conf.json` 中的 `bundle.targets` 必须包含：
  - `dmg`（macOS）
  - `msi`（Windows）
- **并且** 必须排除不需要的格式（如 `deb`, `rpm`）

#### 场景： 应用元数据

- **当** 用户安装应用后
- **那么** 应用的"关于"信息必须显示正确的版本号
- **并且** 必须显示应用描述
- **并且** 必须显示版权信息
- **并且** 信息必须与 `tauri.conf.json` 一致

---

### 需求： 文档更新

系统必须提供完整的构建和发布文档，帮助开发者和用户理解流程。

#### 场景： README 构建说明

- **当** 开发者查看 README
- **那么** 必须包含"构建"章节
- **并且** 必须说明 macOS 和 Windows 的构建步骤
- **并且** 必须说明依赖要求
- **并且** 必须包含示例命令

#### 场景： README 发布说明

- **当** 开发者查看 README
- **那么** 必须包含"发布"章节
- **并且** 必须说明版本更新流程
- **并且** 必须说明 Git 标签创建和推送步骤
- **并且** 必须说明 CI 自动构建流程

#### 场景： 脚本使用文档

- **当** 开发者查看 `scripts/README.md`
- **那么** 必须说明每个脚本的功能
- **并且** 必须说明依赖要求
- **并且** 必须包含使用示例
- **并且** 必须包含常见问题（FAQ）

---

### 需求： 错误处理和日志

系统必须提供清晰的错误处理和日志输出，帮助开发者快速定位问题。

#### 场景： 脚本错误处理

- **当** 构建脚本执行过程中遇到错误
- **那么** 必须立即停止执行
- **并且** 必须输出清晰的错误信息（包含失败步骤）
- **并且** 必须显示完整的错误日志
- **并且** 必须以非零退出码退出

#### 场景： CI 构建日志

- **当** GitHub Actions 构建失败
- **那么** 必须在 Actions 页面显示完整的构建日志
- **并且** 必须高亮显示错误信息
- **并且** 必须保留日志至少 90 天

#### 场景： 构建成功日志

- **当** 构建成功完成
- **那么** 脚本必须输出成功信息（绿色）
- **并且** 必须显示构建产物路径
- **并且** 必须显示文件大小
- **并且** 必须显示总耗时
