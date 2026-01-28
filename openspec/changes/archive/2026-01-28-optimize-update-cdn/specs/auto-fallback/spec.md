# auto-fallback Specification

## Purpose
实现 CDN 下载失败时自动降级到 GitHub 的机制，确保更新功能的高可用性。默认使用阿里云 CDN 作为主要下载源，GitHub Releases 作为备用源。

## 新增需求

### 需求：默认使用 CDN 更新检查

应用必须默认从阿里云 CDN 获取更新信息，失败时自动降级到 GitHub。

#### 场景：从 CDN 检查更新

- **当** 调用 `check_update()` 函数时
- **那么** 必须首先从阿里云 CDN 获取更新清单（`https://{CDN_DOMAIN}/latest.json`）
- **并且** 必须设置超时时间为 5 秒
- **并且** 必须解析 JSON 内容，提取以下字段：
  - `version`：最新版本号
  - `notes`：更新日志
  - `pub_date`：发布日期
  - `platforms.{platform}.url`：当前平台的下载 URL
  - `platforms.{platform}.signature`：当前平台的签名
- **并且** 必须根据当前操作系统和架构选择正确的平台：
  - macOS ARM64 → `darwin-aarch64`
  - macOS x64 → `darwin-x86_64`
  - Windows x64 → `windows-x86_64`
  - Windows ARM64 → `windows-aarch64`

#### 场景：CDN 检查失败（降级到 GitHub）

- **当** 从阿里云 CDN 获取更新清单失败（超时、网络错误、HTTP 错误等）时
- **那么** 必须记录日志（WARN 级别）：
  ```
  从 CDN 检查更新失败: {error}，降级到 GitHub Releases
  ```
- **并且** 必须自动切换到 GitHub Releases 获取更新清单
- **并且** GitHub URL 必须为：`https://github.com/HerbertGao/QSL-CardHub/releases/latest/download/latest.json`

#### 场景：比较版本号

- **当** 获取更新清单成功后（无论从哪个源）
- **那么** 必须比较最新版本号与当前应用版本号
- **并且** 如果最新版本 > 当前版本，返回更新信息（`Some(UpdateInfo)`）
- **并且** 如果最新版本 ≤ 当前版本，返回无更新（`None`）
- **并且** 版本比较必须正确处理语义化版本（如 `0.6.2` vs `0.6.10`）

---

### 需求：带降级的下载

应用必须优先从 CDN 下载更新包，失败时自动降级到 GitHub。

#### 场景：从 CDN 下载更新包

- **当** 调用 `download_update()` 函数时
- **那么** 必须首先尝试从更新清单中的 URL 下载（优先为 CDN URL）
- **并且** 必须显示下载进度（通过回调函数或事件）
- **并且** 必须记录日志（INFO 级别）：
  ```
  开始从 CDN 下载更新包: {url}
  ```

#### 场景：CDN 下载成功

- **当** 从 CDN 下载成功时
- **那么** 必须验证文件签名
- **并且** 如果签名验证通过，返回下载的文件路径
- **并且** 必须记录日志（INFO 级别）：
  ```
  从 CDN 下载成功，文件: {file_path}
  ```

#### 场景：CDN 下载失败（降级到 GitHub）

- **当** 从 CDN 下载失败（网络错误、超时、HTTP 错误等）时
- **那么** 必须记录日志（WARN 级别）：
  ```
  从 CDN 下载失败: {error}，降级到 GitHub Releases
  ```
- **并且** 必须自动从 GitHub Releases 获取更新清单（如果之前未获取）
- **并且** 必须从 GitHub URL 下载更新包
- **并且** 必须重置下载进度为 0

#### 场景：GitHub 下载成功

- **当** 从 GitHub 下载成功时
- **那么** 必须验证文件签名
- **并且** 如果签名验证通过，返回下载的文件路径
- **并且** 必须记录日志（INFO 级别）：
  ```
  从 GitHub 下载成功（备用源），文件: {file_path}
  ```

#### 场景：两个源都失败

- **当** CDN 和 GitHub 都下载失败时
- **那么** 必须返回错误
- **并且** 错误信息必须包含两个源的失败原因
- **并且** 必须记录日志（ERROR 级别）：
  ```
  所有下载源都失败: CDN: {cdn_error}, GitHub: {github_error}
  ```

#### 场景：签名验证失败

- **当** 下载成功但签名验证失败时
- **那么** 必须删除已下载的文件
- **并且** 必须返回错误（"签名验证失败，文件可能被篡改"）
- **并且** 必须记录日志（ERROR 级别）：
  ```
  下载的文件签名验证失败，拒绝安装: {source}
  ```
- **并且** 不尝试切换到备用源（因为签名相同，备用源也会失败）

---

### 需求：下载进度回调

下载过程必须支持进度回调，便于前端展示下载进度。

#### 场景：注册进度回调

- **当** 调用下载函数时
- **那么** 必须支持传入进度回调函数：
  ```rust
  pub fn download_update<F>(
      update_info: UpdateInfo,
      progress_callback: F,
  ) -> Result<PathBuf, Error>
  where
      F: Fn(u64, u64) + Send + 'static,
      // 参数: (已下载字节数, 总字节数)
  ```

#### 场景：实时更新进度

- **当** 下载过程中时
- **那么** 必须定期调用进度回调（如每下载 1MB 或每 500ms）
- **并且** 回调参数必须包含：
  - 已下载字节数（`downloaded_bytes: u64`）
  - 总字节数（`total_bytes: u64`）
- **并且** 前端可根据此信息计算进度百分比

#### 场景：切换源时重置进度

- **当** 从 CDN 切换到 GitHub 时
- **那么** 必须重置进度为 0
- **并且** 必须通过回调通知前端（`progress_callback(0, total_bytes)`）

---

### 需求：下载源枚举

应用必须定义下载源枚举类型，用于日志记录和调试。

#### 场景：定义 DownloadSource 枚举

- **当** 实现下载功能时
- **那么** 必须定义 Rust 枚举：
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum DownloadSource {
      CDN,      // 阿里云 CDN（主源）
      GitHub,   // GitHub Releases（备用源）
  }
  ```
- **并且** 枚举必须实现 `Display` trait，输出人类可读的名称

---

### 需求：与现有 Tauri Updater 插件兼容

新的更新逻辑必须与现有 Tauri Updater 插件兼容。

#### 场景：保留 Tauri Updater 配置

- **当** 实现自定义更新逻辑时
- **那么** 必须保留 `tauri.conf.json` 中的 `updater` 配置
- **并且** 必须更新 `updater.endpoints` 指向阿里云 CDN：
  ```json
  "endpoints": [
    "https://cdn.qsl-cardhub.com/latest.json",
    "https://github.com/HerbertGao/QSL-CardHub/releases/latest/download/latest.json"
  ]
  ```
- **并且** Tauri Updater 插件会自动按顺序尝试这些 endpoints

#### 场景：签名验证使用相同公钥

- **当** 验证签名时
- **那么** 必须使用与 Tauri Updater 插件相同的公钥
- **并且** 公钥必须从 `tauri.conf.json` 的 `updater.pubkey` 字段读取
- **并且** 确保从 CDN 和 GitHub 下载的文件都可通过验证

---

## 修改需求

### 需求：修改前端更新逻辑（修改 auto-updater 规范）

必须修改 `AboutView.vue` 和 `updateStore.ts`，使用阿里云 CDN 作为主要更新源。

#### 场景：调用更新检查

- **当** 用户点击"检查更新"按钮时
- **那么** 前端必须调用 Tauri Updater 插件的 `check()` 方法
- **并且** Tauri Updater 会自动按 `endpoints` 顺序尝试（先 CDN，后 GitHub）
- **并且** 必须显示加载状态（"正在检查更新..."）

#### 场景：显示下载源（可选）

- **当** 检查更新成功且有新版本时
- **那么** 可在更新对话框中显示说明文案：
  ```
  更新源: 阿里云 CDN（国内加速）
  ```
- **并且** 如果从 CDN 下载失败，显示提示：
  ```
  CDN 下载失败，已自动切换到 GitHub
  ```

#### 场景：调用下载

- **当** 用户点击"下载更新"按钮时
- **那么** 前端必须调用 Tauri Updater 插件的 `downloadAndInstall()` 方法
- **并且** 必须监听下载进度事件，更新进度条
- **并且** Tauri Updater 会自动处理降级逻辑

#### 场景：失败时提供手动下载链接

- **当** 下载失败（CDN 和 GitHub 都失败）时
- **那么** 必须显示友好的错误提示
- **并且** 必须提供 GitHub Releases 手动下载链接
- **并且** 错误信息必须包含失败原因（网络错误、签名验证失败等）

---

## 移除需求

无（不移除任何现有需求，仅增加和修改）
