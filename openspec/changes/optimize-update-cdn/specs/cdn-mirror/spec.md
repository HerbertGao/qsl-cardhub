# cdn-mirror Specification

## Purpose
使用阿里云 OSS + CDN 作为主要下载源，通过 CDN 回源方式访问私有 OSS Bucket，为所有用户（尤其是国内用户）提供高速、安全的更新下载体验。

## 新增需求

### 需求：阿里云 OSS 私有存储配置

CI/CD 必须将构建产物上传到阿里云私有 OSS Bucket，作为主要下载源。

#### 场景：创建私有 OSS Bucket

- **当** 首次配置阿里云存储时
- **那么** 必须创建一个专用的 OSS Bucket（如 `qsl-cardhub-releases`）
- **并且** 访问控制必须设置为私有（Private），禁止公开访问
- **并且** 仅允许阿里云 CDN 通过回源方式访问
- **并且** 启用 OSS 访问日志审计

#### 场景：配置 CDN 回源加速

- **当** OSS Bucket 创建完成后
- **那么** 必须配置阿里云 CDN，绑定自定义域名（如 `cdn.qsl-cardhub.com`）
- **并且** CDN 必须配置回源到私有 OSS Bucket
- **并且** CDN 必须启用 HTTPS（配置 SSL 证书）
- **并且** CDN 缓存规则必须缓存所有静态文件（.exe、.dmg、.sig、.json）
- **并且** CDN 缓存 TTL 必须设置为至少 7 天（减少回源请求）
- **并且** CDN 必须配置回源 Host 为 OSS Bucket 的内网域名

#### 场景：配置访问凭据

- **当** 需要在 CI/CD 中访问 OSS 时
- **那么** 必须在 GitHub Secrets 中配置以下凭据：
  - `ALIYUN_OSS_ACCESS_KEY_ID`：阿里云 AccessKey ID
  - `ALIYUN_OSS_ACCESS_KEY_SECRET`：阿里云 AccessKey Secret
  - `ALIYUN_OSS_BUCKET_NAME`：Bucket 名称
  - `ALIYUN_OSS_ENDPOINT`：OSS Endpoint（如 `oss-cn-hangzhou.aliyuncs.com`）
  - `ALIYUN_CDN_DOMAIN`：CDN 域名（如 `cdn.qsl-cardhub.com`）
- **并且** AccessKey 必须仅授予最小权限（OSS 上传权限）
- **并且** 凭据不得在代码中硬编码

---

### 需求：CI/CD 自动同步到阿里云

GitHub Actions 必须在发布新版本时自动将构建产物同步到阿里云 OSS。

#### 场景：安装 OSS 上传工具

- **当** CI/CD 开始构建时
- **那么** 必须安装阿里云官方 CLI 工具 `ossutil`
- **并且** 必须使用 GitHub Secrets 中的凭据配置 `ossutil`
- **并且** 必须验证配置是否成功（如执行 `ossutil ls` 列出 Bucket）

#### 场景：上传安装包到私有 OSS

- **当** 各平台构建完成后
- **那么** 必须将以下文件上传到 OSS：
  - Windows x64：`qsl-cardhub-v{version}-windows-x64-setup.exe` 和 `.sig` 文件
  - Windows ARM64：`qsl-cardhub-v{version}-windows-arm64-setup.exe` 和 `.sig` 文件
  - macOS ARM64：`qsl-cardhub-v{version}-macos-arm64.app.tar.gz` 和 `.sig` 文件、`.dmg` 文件
  - macOS x64：`qsl-cardhub-v{version}-macos-x64.app.tar.gz` 和 `.sig` 文件、`.dmg` 文件
- **并且** 文件必须上传到 `/releases/v{version}/` 目录
- **并且** 文件访问权限必须保持私有（不设置公共读）
- **并且** 上传失败时必须重试（最多 3 次）

#### 场景：并行上传优化

- **当** 上传到 GitHub Releases 和阿里云 OSS 时
- **那么** 两者必须并行执行，不互相阻塞
- **并且** 总发布时间增加不超过 2 分钟
- **并且** 如果阿里云上传失败，不影响 GitHub Releases 的发布（GitHub 作为备用源）

---

### 需求：生成阿里云 CDN 更新清单

CI/CD 必须生成包含阿里云 CDN URLs 的更新清单文件，作为主要更新清单。

#### 场景：生成 latest.json（CDN URLs）

- **当** 所有平台的安装包上传完成后
- **那么** 必须生成 `latest.json` 文件（包含阿里云 CDN URLs）
- **并且** 文件格式必须符合 Tauri Updater 规范
- **并且** 必须包含以下字段：
  - `version`：版本号
  - `notes`：更新日志
  - `pub_date`：发布日期
  - `platforms`：各平台的下载信息
- **并且** `platforms` 中每个平台必须包含：
  - `signature`：签名（与 GitHub 版本一致）
  - `url`：阿里云 CDN 下载 URL

#### 场景：阿里云 CDN URL 格式

- **当** 生成 `latest.json` 时
- **那么** URL 格式必须为：
  - `https://{CDN_DOMAIN}/releases/v{version}/{filename}`
  - 示例：`https://cdn.qsl-cardhub.com/releases/v0.6.2/qsl-cardhub-v0.6.2-windows-x64-setup.exe`
- **并且** 所有 URL 必须使用 HTTPS 协议
- **并且** 签名字段必须与 GitHub 版本的签名完全一致（因为文件内容相同）

#### 场景：上传 latest.json 到 OSS

- **当** `latest.json` 生成完成后
- **那么** 必须上传到 OSS 根目录（`/latest.json`）
- **并且** 文件访问权限必须保持私有（通过 CDN 回源访问）
- **并且** 可通过 `https://{CDN_DOMAIN}/latest.json` 访问（CDN 自动回源到 OSS）

---

### 需求：OSS 生命周期管理

阿里云 OSS 必须配置生命周期规则，自动清理旧版本文件以节省成本。

#### 场景：保留最近版本

- **当** 配置 OSS Bucket 时
- **那么** 必须设置生命周期规则：
  - 保留最近 3 个版本的所有文件
  - 自动删除超过 90 天的旧版本文件
- **并且** `latest.json` 永不过期（始终指向最新版本）
- **并且** 删除前必须有通知机制（如日志记录）

#### 场景：手动清理

- **当** 需要手动清理 OSS 存储时
- **那么** 必须提供清理脚本或文档说明
- **并且** 清理前必须确认不影响正在使用的版本
- **并且** 清理后必须验证 `latest.json` 仍然可用

---

## 修改需求

### 需求：更新 CI/CD 工作流（修改 auto-updater 规范）

必须修改现有的 `release.yml` 工作流，集成阿里云 OSS 上传步骤，生成包含 CDN URLs 的更新清单。

#### 场景：在 generate-update-manifest job 中生成 CDN 清单

- **当** 执行 `generate-update-manifest` job 时
- **那么** 必须生成包含阿里云 CDN URLs 的 `latest.json` 文件
- **并且** 必须上传到阿里云 OSS 根目录
- **并且** 必须保留 GitHub Releases 的 `latest.json`（包含 GitHub URLs，作为备用）
- **并且** 两个清单的版本号、更新日志、发布日期、签名必须一致，仅 URL 不同

#### 场景：验证上传完整性

- **当** 上传到阿里云 OSS 完成后
- **那么** 必须验证所有文件已成功上传
- **并且** 必须比对文件 MD5 或 SHA256，确保与本地构建产物一致
- **并且** 如果验证失败，必须标记 job 为失败
