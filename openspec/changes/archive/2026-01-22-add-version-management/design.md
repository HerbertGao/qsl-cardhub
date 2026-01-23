## 上下文

QSL-CardHub 是一个使用 Rust + Tauri 2 + Vue 3 构建的跨平台桌面应用。当前版本号分散在多个文件中（`Cargo.toml`、`tauri.conf.json`、`web/package.json`），需要手动同步。应用缺乏内置更新机制，用户需要手动下载新版本。

参考项目 [gaokao_bot](https://github.com/HerbertGao/gaokao_bot) 的 `version.sh` 和 `release.sh` 脚本，为本项目设计版本管理方案。

## 目标 / 非目标

### 目标
- 一键升级版本号并同步到所有配置文件
- 提供完整的发布流程脚本
- 应用内显示真实版本号
- 应用内检查更新并提示用户
- 支持下载并安装更新

### 非目标
- 完全静默后台自动更新（需要用户确认）
- Linux 平台的自动更新（Tauri Updater 对 Linux 支持有限）
- 应用内直接替换升级（受限于安装包格式）

## 决策

### 1. 版本号管理策略

**决策：以 `Cargo.toml` 为单一版本号来源**

- 所有版本号从 `Cargo.toml` 读取
- `version.sh` 修改 `Cargo.toml` 后自动同步到其他文件
- 已有的 `sync-version.js` 和 `sync-version.sh` 保留，供构建时使用

**考虑的替代方案**：
- 使用 `tauri.conf.json` 作为来源：但 Cargo.toml 是 Rust 项目的标准，且 Cargo 命令直接读取
- 使用专门的 VERSION 文件：增加了额外的管理复杂度

### 2. 版本脚本设计

**决策：创建独立的 `version.sh` 和 `release.sh`**

`version.sh` 职责：
```
version.sh [major|minor|patch|build|x.y.z]
```
- 从 Cargo.toml 读取当前版本
- 验证并更新版本号
- 同步到 tauri.conf.json 和 web/package.json
- 不执行 Git 操作

`release.sh` 职责：
```
release.sh [major|minor|patch|x.y.z]
```
- 调用 version.sh 更新版本
- 检查 Git 工作区状态
- 运行构建验证（可选：cargo check, npm run build）
- 创建版本提交
- 创建 Git 标签
- 推送到远程（可选）

### 3. 自动更新方案

**决策：使用 Tauri Updater 插件 + GitHub Releases**

架构：
```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  QSL-CardHub    │────▶│  GitHub Releases │────▶│  latest.json    │
│  (应用内检查)    │     │  (托管更新包)     │     │  (更新清单)      │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

更新流程：
1. 用户点击"检查更新"或启动时自动检查
2. 应用获取 `latest.json`，比较版本号
3. 如有更新，显示更新信息（版本、日志、大小）
4. 用户确认后，后台下载更新包
5. 下载完成后提示重启安装

**考虑的替代方案**：
- CrabNebula Cloud：商业服务，功能更丰富但增加依赖和成本
- 自建更新服务器：维护成本高，GitHub Releases 已足够

### 4. 平滑升级策略

**Windows 方案**：

| 格式 | 优点 | 缺点 | 决策 |
|------|------|------|------|
| MSI | 标准格式，企业支持好 | 不支持静默升级，需完全退出 | 保留 |
| NSIS | 支持静默安装/升级 | 非标准，部分企业受限 | 新增 |
| EXE | 便携版，无需安装 | 无自动更新支持 | 暂不支持 |

**决策**：同时生成 MSI 和 NSIS 安装包
- NSIS 用于自动更新（静默安装）
- MSI 用于企业部署场景
- Tauri Updater 默认使用 NSIS 包

**macOS 方案**：

DMG 格式不支持静默升级。采用"下载后提示"模式：
1. 下载新版 DMG 到用户 Downloads 目录
2. 打开 Finder 显示文件位置
3. 提示用户手动拖拽安装
4. 提供详细安装指南链接

**考虑的替代方案**：
- Sparkle 框架：需要额外依赖，与 Tauri 集成复杂
- pkg 格式：体验不如 DMG，用户不熟悉

### 5. 签名与安全

**决策：使用 Tauri 内置签名机制**

- 生成 Ed25519 密钥对
- 公钥写入 `tauri.conf.json`
- 私钥存储在 GitHub Secrets
- CI 构建时自动签名更新包
- 应用验证签名后才安装

### 6. 关于页面改造

**决策：动态读取版本信息 + 检查更新 UI**

```vue
<template>
  <el-descriptions-item label="版本">
    {{ version }}
    <el-button @click="checkUpdate">检查更新</el-button>
  </el-descriptions-item>

  <!-- 更新对话框 -->
  <el-dialog v-model="updateDialogVisible">
    <p>发现新版本: {{ updateInfo.version }}</p>
    <p>{{ updateInfo.notes }}</p>
    <el-progress :percentage="downloadProgress" />
    <el-button @click="downloadAndInstall">下载并安装</el-button>
  </el-dialog>
</template>
```

版本号获取方式：
- 前端通过 Tauri API 调用 `app.getVersion()`
- 或通过 Vite 编译时注入 `import.meta.env.PACKAGE_VERSION`

## 风险 / 权衡

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 签名密钥泄露 | 攻击者可发布恶意更新 | 密钥仅存 GitHub Secrets，定期轮换 |
| GitHub API 限流 | 用户无法检查更新 | 添加缓存，限制检查频率 |
| 网络环境问题 | 中国用户下载慢 | 提供镜像下载链接（后续考虑） |
| macOS 手动安装体验差 | 用户流失 | 提供详细指南，考虑后续 Sparkle 集成 |
| NSIS 被杀软误报 | 用户担忧 | 代码签名（需要证书，后续考虑） |

## 迁移计划

### 阶段 1：版本脚本（无破坏性）
1. 添加 `scripts/version.sh`
2. 添加 `scripts/release.sh`
3. 更新 `scripts/README.md`

### 阶段 2：关于页面改造（无破坏性）
1. 修改 AboutView.vue 动态显示版本
2. 添加检查更新按钮（仅 UI，无功能）

### 阶段 3：Updater 集成（需要测试）
1. 添加 tauri-plugin-updater 依赖
2. 配置 tauri.conf.json
3. 生成签名密钥
4. 实现前端更新逻辑

### 阶段 4：CI/CD 改造（需要测试）
1. 修改 release.yml 生成 NSIS 包
2. 自动生成并上传 latest.json
3. 添加签名步骤

### 回滚方案
- 各阶段独立，可单独回滚
- 如 Updater 出问题，可禁用检查更新按钮
- latest.json 格式错误时，应用不会崩溃，仅显示"无法检查更新"

## 已决问题

1. **更新检查时机**：启动时自动检查，如果有更新则在"关于"菜单增加红色标记提示，由用户手动触发更新流程。用户也可以随时在关于页面手动检查更新。
2. **镜像下载**：暂时不提供国内加速镜像，后续根据用户反馈考虑。
3. **代码签名**：暂不购买 Windows 代码签名证书，后续根据用户反馈考虑。
4. **版本号格式**：仅支持标准 semver 格式（`X.Y.Z`），不支持预发布版本（如 `1.0.0-beta.1`）。
