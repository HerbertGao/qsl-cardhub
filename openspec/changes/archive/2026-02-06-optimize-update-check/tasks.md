## 1. 平台检测工具函数

- [x] 1.1 在 `web/src/services/updateCheck.ts` 中新增 `getPlatformAssetKeyword()` 函数，使用 Tauri OS API（`@tauri-apps/plugin-os` 的 `platform()` 和 `arch()`）获取当前操作系统和架构，返回对应的资产文件名匹配关键词（如 `macos-arm64` + `.dmg`、`windows-x64` + `-setup.exe`）。无法识别平台时返回 `null`。

## 2. GitHub API 路径增加资产验证

- [x] 2.1 在 `web/src/services/updateCheck.ts` 中新增 `hasAssetForPlatform(assets, platformKeyword, extensionKeyword)` 函数，接收 GitHub Release 的 `assets` 数组，检查是否存在匹配当前平台关键词和文件扩展名的资产文件。
- [x] 2.2 修改 `checkForUpdate()` 中 Tauri Updater 成功返回 `null`（无更新）后的 GitHub API fallback 路径（第 56-84 行区域）：在 `compareVersions(latestVersion, currentVersion) > 0` 判定为 true 后，额外调用 `hasAssetForPlatform()` 验证资产可用性。如果资产不存在，跳过 `setUpdateAvailable()` 调用（视为无更新）。
- [x] 2.3 修改 `checkForUpdate()` 中 Tauri Updater 抛出异常后的 GitHub API fallback 路径（第 89-114 行的 catch 块）：同样在版本号比较后增加 `hasAssetForPlatform()` 验证。

## 3. Tauri Updater 下载失败优雅处理

- [x] 3.1 修改 `web/src/views/AboutView.vue` 中的 `handleDownloadUpdate()` 函数：在 `pendingUpdate.downloadAndInstall()` 的 catch 块中，增加判断逻辑——如果错误为 404 或网络超时类错误，记录 `console.warn` 日志，清除更新状态（调用 `clearUpdate()`），并向用户显示"更新暂时不可用，请稍后重试"提示，而非当前的通用下载失败错误。

## 4. 验证

- [ ] 4.1 手动测试：在有新 Release 但无安装包资产的情况下，确认手动检查更新显示"已是最新版本"。（需用户手动验证）
- [ ] 4.2 手动测试：在有新 Release 且安装包资产已上传的情况下，确认更新检查正常提示新版本并可下载。（需用户手动验证）
- [ ] 4.3 手动测试：在无法识别平台时（模拟 `getPlatformAssetKeyword()` 返回 `null`），确认回退到仅版本号判断逻辑。（需用户手动验证）
