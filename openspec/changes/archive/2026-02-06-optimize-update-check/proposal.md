## 为什么

当前的更新检查机制存在时序问题：当新版本的 Git Tag 推送后，GitHub Actions Release 工作流会立即创建 GitHub Release，但此时多平台构建和打包尚未完成（通常需要 10-20 分钟）。在这个时间窗口内，应用的更新检查会通过 GitHub Releases API 或 Tauri Updater 发现新版本已存在，但实际的安装包和 `latest.json` 更新清单尚未上传，导致用户看到"发现新版本"提示却无法下载更新。

## 变更内容

1. **修改更新检查逻辑**：在判断"有新版本可用"之前，增加对更新包实际可下载性的验证。不仅检查版本号是否更新，还要确认对应平台的安装包已就绪。
2. **修改 Tauri Updater 的行为处理**：当 Tauri Updater 报告有更新但下载端点尚不可用时，应视为"暂无更新"而非报错。
3. **修改 GitHub API fallback 逻辑**：在通过 GitHub Releases API 检查到新版本时，额外验证 Release 中是否已包含当前平台所需的安装包资产。

## 功能 (Capabilities)

### 新增功能

- `update-availability-check`: 更新可用性验证功能——在报告"有新版本"前，验证对应平台的安装包资产是否已上传就绪，避免在 CI/CD 构建期间误报更新可用。

### 修改功能

- `auto-updater`: 更新检查的判断条件从"存在更高版本号"变更为"存在更高版本号且对应平台的安装包已就绪"。

## 影响

- **前端代码**：`web/src/services/updateCheck.ts` — 需要修改 `checkForUpdate()` 函数，增加资产可用性验证
- **前端状态**：`web/src/stores/updateStore.ts` — 可能需要新增"构建中"状态
- **API 调用**：GitHub Releases API 调用需要额外检查 `assets` 字段
- **Tauri Updater**：需要处理 Tauri Updater 返回"有更新"但实际无法下载的边界情况
- **无 Breaking Change**：不影响 CI/CD 工作流、后端 Rust 代码或 Tauri 配置
