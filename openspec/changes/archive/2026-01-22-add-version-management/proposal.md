# 变更：版本管理与自动更新

## 为什么

当前项目缺乏完善的版本管理脚本和应用内更新检查功能。用户需要手动访问 GitHub Releases 下载新版本，体验不佳。同时，开发者升级版本号时需要手动修改多个文件，容易遗漏。

## 变更内容

### 1. 一键升版本号脚本
- 添加 `scripts/version.sh` 脚本，支持 `major`/`minor`/`patch` 和自定义版本号
- 添加 `scripts/release.sh` 脚本，整合版本升级、构建验证和发布流程
- 自动同步 `Cargo.toml`、`tauri.conf.json`、`web/package.json` 的版本号
- 支持 Git 标签创建和推送

### 2. 应用内版本检查
- 在关于页面显示当前版本号（动态读取而非硬编码）
- 添加"检查更新"按钮，调用 GitHub API 检查最新版本
- 显示更新信息（版本号、更新日志、发布日期）

### 3. 自动更新支持
- 集成 Tauri Updater 插件
- 支持从 GitHub Releases 下载更新
- 实现后台下载和进度显示
- **平滑升级方案**：
  - Windows：使用 NSIS 安装包（支持静默安装，优于 MSI）
  - macOS：保持 DMG 格式，但提供应用内下载和提示安装

### 4. GitHub Release 配置
- 自动生成更新清单（`latest.json`）
- 配置签名密钥进行安全验证
- 修改 release.yml 生成更新包和签名文件

## 影响

- 受影响规范：`build-and-packaging`（扩展版本管理需求）
- 新增规范：`version-management`、`auto-updater`
- 受影响代码：
  - `scripts/`：新增版本和发布脚本
  - `web/src/views/AboutView.vue`：添加更新检查 UI
  - `tauri.conf.json`：配置更新插件
  - `Cargo.toml`：添加 tauri-plugin-updater 依赖
  - `.github/workflows/release.yml`：生成更新清单和签名
