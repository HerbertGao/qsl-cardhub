## 上下文

当前更新检查流程（`updateCheck.ts`）采用双重策略：先通过 Tauri Updater 插件的 `check()` 检查，失败后回退到 GitHub Releases API。两条路径均仅基于版本号比较来判断更新可用性。

问题根源在于 CI/CD 时序：Release 工作流由 Tag 推送触发，`create-release` job 会立即创建 Release（draft: false），但 `build-and-upload`（多平台构建 + 上传）和 `generate-update-manifest`（生成 latest.json）需要 10-20 分钟才能完成。在此窗口期内：

- **Tauri Updater 路径**：`check()` 访问 `latest.json` 端点。由于 latest.json 尚未更新（仍指向上一版本），Tauri Updater 通常不会误报。但存在边界情况：CDN 缓存过期 + 新 latest.json 已上传但构建产物尚未全部上传。
- **GitHub API 路径**：`/releases/latest` 已返回新版本的 Release 信息，但 `assets` 列表为空或不完整，导致误报更新可用。

变更仅涉及前端 TypeScript 代码，不需要修改 Rust 后端、Tauri 配置或 CI/CD 工作流。

## 目标 / 非目标

**目标：**
- 消除 CI/CD 构建期间的更新误报：用户不会看到无法下载的"新版本"提示
- 保持现有双重检查策略（Tauri Updater → GitHub API）的架构不变
- 变更对用户透明：构建完成后的正常更新流程不受影响

**非目标：**
- 不修改 CI/CD 工作流（如改为 draft release 再发布）
- 不修改 Tauri Updater 端点配置
- 不增加后端 Rust 代码
- 不实现"版本正在构建中"的特殊 UI 状态（过度工程化）

## 决策

### 决策 1：在 GitHub API 路径中增加资产存在性检查

**选择**：在 GitHub Releases API 返回新版本后，检查 `release.assets` 数组中是否包含当前平台所需的安装包文件。

**理由**：GitHub Releases API 的响应天然包含 `assets` 字段（文件名、下载 URL、大小等），无需额外 API 调用。这是最轻量的验证方式。

**替代方案考虑**：
- ~~HEAD 请求验证下载 URL~~：需要额外的网络请求，增加延迟，且可能受 CDN 缓存影响
- ~~检查 latest.json 是否已更新~~：latest.json 的版本号更新不等于安装包已就绪（latest.json 在构建完成后才生成，但理论上存在部分平台构建完成的情况）
- ~~修改 CI/CD 改为 draft release~~：需要修改工作流，且会影响 Release 页面的用户体验（用户无法提前看到 Release Notes）

### 决策 2：Tauri Updater 路径保持现有行为，增加错误捕获

**选择**：当 Tauri Updater 的 `check()` 返回有更新，但后续 `downloadAndInstall()` 调用失败（如 404）时，在下载阶段优雅降级为"暂无更新"。不在 check 阶段做额外验证。

**理由**：
- Tauri Updater 的 `check()` 依赖 latest.json，而 latest.json 是在所有构建完成后才生成的，因此 Tauri Updater 路径误报的概率极低
- `check()` 返回的 Update 对象没有暴露 assets 信息，无法在 check 阶段做资产验证
- 真正的问题出现在下载阶段，此时捕获错误并提示用户即可

**替代方案考虑**：
- ~~在 check() 后立即用 fetch 验证下载 URL~~：引入额外网络请求，增加复杂度，且 Tauri Updater 路径几乎不会误报

### 决策 3：平台检测使用 Tauri API

**选择**：使用 `@tauri-apps/api/os`（或 `@tauri-apps/plugin-os`）获取当前操作系统和架构，映射到资产文件名中的平台标识符。

**理由**：Tauri 提供的 OS API 能准确返回 `platform`（`darwin`/`windows`）和 `arch`（`x86_64`/`aarch64`），与 CI/CD 生成的文件名命名规则一致。

**平台到文件名的映射规则**：
| OS + Arch | 匹配的资产关键词 |
|---|---|
| darwin + aarch64 | `macos-arm64` + `.dmg` |
| darwin + x86_64 | `macos-x64` + `.dmg` |
| windows + x86_64 | `windows-x64` + `-setup.exe` |
| windows + aarch64 | `windows-arm64` + `-setup.exe` |

### 决策 4：不引入新的 UI 状态

**选择**：当版本号更高但安装包未就绪时，表现为"已是最新版本"（与无新版本时一致），而不是显示"新版本正在构建中"。

**理由**：
- 对用户而言，无法下载的版本等同于不存在
- 避免引入额外 UI 状态增加复杂度
- 构建窗口期通常仅 10-20 分钟，用户几乎不会注意到

## 风险 / 权衡

**[风险] GitHub API Rate Limit**
→ 现有逻辑已使用 GitHub API，assets 检查不增加额外请求（同一 response 中的字段），无新增风险。

**[风险] 资产文件名格式变更**
→ 如果 CI/CD 修改了产物命名规则（如从 `macos-arm64` 改为 `darwin-aarch64`），资产匹配会失败。缓解措施：匹配逻辑失败时回退到仅版本号判断（向后兼容），不阻塞更新。

**[权衡] 部分平台构建完成的情况**
→ 如果 Windows 构建已完成但 macOS 尚未完成，macOS 用户看到"已是最新版本"而 Windows 用户看到"有更新"。这是预期行为：每个平台只关心自己的安装包是否可用。

**[权衡] Tauri Updater 路径未增加提前检查**
→ 极低概率下 Tauri Updater 可能误报（CDN latest.json 已更新但安装包 URL 404），此时用户点击"下载更新"会报错。现有的下载错误处理已覆盖此场景，用户体验可接受。
