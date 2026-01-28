# 设计文档 - 更新下载优化

## 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                         应用端                               │
├─────────────────────────────────────────────────────────────┤
│  Frontend (Vue 3)                                           │
│    ├── AboutView.vue (检查更新 UI)                         │
│    └── updateStore.ts (更新状态管理)                       │
├─────────────────────────────────────────────────────────────┤
│  Tauri Updater Plugin                                       │
│    ├── check() - 检查更新（按 endpoints 顺序尝试）        │
│    └── downloadAndInstall() - 下载并安装                   │
└─────────────────────────────────────────────────────────────┘
                              ↓
                  优先尝试 CDN，失败降级
                              ↓
┌──────────────────────┬──────────────────────────────────────┐
│   阿里云 CDN (主源)   │      GitHub Releases (备用源)        │
│   cdn.qsl-cardhub.com│      github.com/.../releases         │
├──────────────────────┼──────────────────────────────────────┤
│ latest.json (CDN URLs)│  latest.json (GitHub URLs)          │
│ *.exe / *.dmg        │  *.exe / *.dmg                      │
│ *.sig                │  *.sig                              │
└──────────────────────┴──────────────────────────────────────┘
         ↑                           ↑
   CDN 回源访问                  直接访问
         ↓
┌─────────────────────────────────────────────────────────────┐
│              阿里云 OSS (私有 Bucket)                        │
│              qsl-cardhub-releases                           │
├─────────────────────────────────────────────────────────────┤
│  /latest.json                                               │
│  /releases/v0.6.2/                                          │
│    ├── qsl-cardhub-v0.6.2-windows-x64-setup.exe            │
│    ├── qsl-cardhub-v0.6.2-windows-x64-setup.exe.sig        │
│    └── ...                                                  │
└─────────────────────────────────────────────────────────────┘
                              ↑
                    CI/CD 自动上传
                              ↑
┌─────────────────────────────────────────────────────────────┐
│              GitHub Actions (release.yml)                   │
├─────────────────────────────────────────────────────────────┤
│  1. 构建多平台安装包                                        │
│  2. 上传到 GitHub Releases (备用源)                         │
│  3. 上传到阿里云 OSS (主源，并行)                           │
│  4. 生成 latest.json (CDN URLs) → 上传到 OSS               │
│  5. 生成 latest.json (GitHub URLs) → 上传到 GitHub         │
└─────────────────────────────────────────────────────────────┘
```

### 下载流程

```
┌─────────────────────────────────────────────────────────┐
│                    用户点击"检查更新"                    │
└────────────────────┬────────────────────────────────────┘
                     ↓
         ┌───────────────────────┐
         │ Tauri Updater Plugin  │
         │ check() 方法          │
         └───────────┬───────────┘
                     ↓
        ┌────────────────────────┐
        │ 尝试 Endpoint 1 (CDN)  │
        │ cdn.qsl-cardhub.com    │
        │ 超时 5 秒              │
        └────────┬───────────────┘
                 ↓
          ┌──────────┐
          │ 成功？    │
          └─────┬────┘
                │
        ┌───────┴────────┐
        ↓                ↓
    ┌──────┐        ┌──────┐
    │  是  │        │  否  │
    └──┬───┘        └───┬──┘
       ↓                ↓
  ┌─────────┐    ┌────────────────┐
  │使用 CDN  │    │尝试 Endpoint 2 │
  │ 清单    │    │ (GitHub)       │
  └────┬────┘    └──────┬─────────┘
       │                ↓
       │         ┌──────────┐
       │         │ 成功？    │
       │         └─────┬────┘
       │               │
       │       ┌───────┴────────┐
       │       ↓                ↓
       │   ┌──────┐        ┌──────┐
       │   │  是  │        │  否  │
       │   └──┬───┘        └───┬──┘
       │      ↓                ↓
       │  ┌─────────┐    ┌────────────┐
       │  │使用 GitHub│    │返回失败    │
       │  │ 清单    │    │提示用户    │
       │  └────┬────┘    └────────────┘
       │       │
       └───────┴────────┐
                        ↓
                 ┌─────────────┐
                 │比较版本号    │
                 └──────┬──────┘
                        ↓
                 ┌──────────┐
                 │有新版本？ │
                 └─────┬────┘
                       │
               ┌───────┴────────┐
               ↓                ↓
           ┌──────┐        ┌──────┐
           │  是  │        │  否  │
           └──┬───┘        └───┬──┘
              ↓                ↓
       ┌──────────────┐   ┌───────────┐
       │显示更新对话框 │   │显示已是   │
       │开始下载      │   │最新版本   │
       └──────┬───────┘   └───────────┘
              ↓
       ┌──────────────┐
       │从清单中的 URL │
       │下载更新包     │
       │(优先 CDN)     │
       └──────┬───────┘
              ↓
       ┌──────────┐
       │下载成功？ │
       └─────┬────┘
             │
     ┌───────┴────────┐
     ↓                ↓
 ┌──────┐        ┌──────┐
 │  是  │        │  否  │
 └──┬───┘        └───┬──┘
    ↓                ↓
┌─────────┐   ┌──────────────┐
│验证签名  │   │降级到备用源   │
└────┬────┘   │(如果未用过)  │
     ↓        └──────┬───────┘
┌─────────┐          ↓
│安装更新  │   (重复下载流程)
└─────────┘
```

## 关键技术决策

### 1. 使用阿里云 CDN 作为主源

**选择理由**：
- 国内访问速度快（5-20MB/s），远超 GitHub（100-500KB/s）
- 全国 CDN 节点覆盖，任何地区都能快速访问
- 成本可控（CDN 流量费用约 0.24 元/GB）

**权衡**：
- 国外用户访问阿里云 CDN 可能稍慢，但仍可接受（≥1MB/s）
- 如果 CDN 对国外用户过慢，会自动降级到 GitHub

### 2. 私有 OSS + CDN 回源

**选择理由**：
- 提升安全性：OSS 文件不公开，仅 CDN 可回源访问
- 简化管理：无需生成 OSS 签名 URL，CDN 直接访问
- 降低成本：CDN 流量费用低于 OSS 外网流量费用
- 提升性能：CDN 缓存减少 OSS 请求

**权衡**：
- 需要配置 CDN 回源规则，稍复杂
- CDN 首次访问需要回源，后续命中缓存

### 3. 保留 GitHub 作为备用源

**选择理由**：
- 高可用性：CDN 失败时有备用方案
- 国际用户友好：GitHub 在国外访问速度快
- 零成本：GitHub Releases 免费

**权衡**：
- 需要维护两份更新清单
- CI/CD 需要上传到两个地方

### 4. 使用 Tauri Updater 插件的多 endpoints 功能

**选择理由**：
- 无需自己实现降级逻辑，Tauri Updater 内置支持
- 配置简单，仅需修改 `tauri.conf.json`
- 自动按顺序尝试 endpoints，失败自动切换

**权衡**：
- 依赖 Tauri Updater 插件的实现
- 无法精细控制降级逻辑（如速度慢时切换）

### 5. CDN 缓存 TTL 设置为 7 天

**选择理由**：
- 减少回源请求，降低 OSS 费用
- 更新包通常不频繁变更（版本号不同时 URL 不同）
- 7 天足够长，但不会导致过期问题

**权衡**：
- 如果需要紧急替换文件，需要手动刷新 CDN 缓存
- 更新清单 `latest.json` 缓存时间可以设短一些（如 1 小时）

## 数据流设计

### 更新清单格式

**CDN 版本（latest.json，存储在阿里云 OSS）**：
```json
{
  "version": "0.6.2",
  "notes": "## 新增功能\n- 支持阿里云 CDN 加速下载\n\n## Bug 修复\n- 修复下载速度慢的问题",
  "pub_date": "2026-01-28T12:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUldTU0tKNTQ2K0FwVmVoMXE4dmNNUnczKzgxOGpTR2pLUXFFczFWU1pRckVLUTJpYWhTYTlPWWIK...",
      "url": "https://cdn.qsl-cardhub.com/releases/v0.6.2/qsl-cardhub-v0.6.2-macos-arm64.app.tar.gz"
    },
    "darwin-x86_64": {
      "signature": "...",
      "url": "https://cdn.qsl-cardhub.com/releases/v0.6.2/qsl-cardhub-v0.6.2-macos-x64.app.tar.gz"
    },
    "windows-x86_64": {
      "signature": "...",
      "url": "https://cdn.qsl-cardhub.com/releases/v0.6.2/qsl-cardhub-v0.6.2-windows-x64-setup.exe"
    },
    "windows-aarch64": {
      "signature": "...",
      "url": "https://cdn.qsl-cardhub.com/releases/v0.6.2/qsl-cardhub-v0.6.2-windows-arm64-setup.exe"
    }
  }
}
```

**GitHub 版本（latest.json，存储在 GitHub Releases）**：
```json
{
  "version": "0.6.2",
  "notes": "## 新增功能\n- 支持阿里云 CDN 加速下载\n\n## Bug 修复\n- 修复下载速度慢的问题",
  "pub_date": "2026-01-28T12:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUldTU0tKNTQ2K0FwVmVoMXE4dmNNUnczKzgxOGpTR2pLUXFFczFWU1pRckVLUTJpYWhTYTlPWWIK...",
      "url": "https://github.com/HerbertGao/QSL-CardHub/releases/download/v0.6.2/qsl-cardhub-v0.6.2-macos-arm64.app.tar.gz"
    },
    ...
  }
}
```

**关键点**：两个清单的 `signature` 字段完全相同（因为文件内容相同）

### OSS 目录结构

```
qsl-cardhub-releases/  (Bucket 根目录，私有权限)
├── latest.json  (主更新清单，包含 CDN URLs)
└── releases/
    ├── v0.6.2/
    │   ├── qsl-cardhub-v0.6.2-windows-x64-setup.exe
    │   ├── qsl-cardhub-v0.6.2-windows-x64-setup.exe.sig
    │   ├── qsl-cardhub-v0.6.2-windows-arm64-setup.exe
    │   ├── qsl-cardhub-v0.6.2-windows-arm64-setup.exe.sig
    │   ├── qsl-cardhub-v0.6.2-windows-x64.msi (可选)
    │   ├── qsl-cardhub-v0.6.2-windows-arm64.msi (可选)
    │   ├── qsl-cardhub-v0.6.2-macos-arm64.dmg
    │   ├── qsl-cardhub-v0.6.2-macos-arm64.app.tar.gz
    │   ├── qsl-cardhub-v0.6.2-macos-arm64.app.tar.gz.sig
    │   ├── qsl-cardhub-v0.6.2-macos-x64.dmg
    │   ├── qsl-cardhub-v0.6.2-macos-x64.app.tar.gz
    │   └── qsl-cardhub-v0.6.2-macos-x64.app.tar.gz.sig
    ├── v0.6.3/
    │   └── ...
    └── ...
```

## 安全考量

### 1. 签名验证

- **签名生成**：CI/CD 使用私钥（GitHub Secret）对安装包签名
- **签名验证**：应用端使用内嵌公钥验证签名
- **关键点**：无论从哪个源下载，签名都相同（因为文件内容相同）
- **安全保证**：即使 OSS 或 CDN 被攻击，篡改的文件也无法通过签名验证

### 2. HTTPS 强制

- 所有下载 URL 必须使用 HTTPS
- 阿里云 CDN 配置 HTTPS 证书
- 防止中间人攻击

### 3. 私有 OSS

- OSS Bucket 设置为私有，禁止公开访问
- 仅 CDN 可通过回源访问
- 防止未授权直接访问 OSS 文件

### 4. 凭据管理

- 阿里云 AccessKey 存储在 GitHub Secrets
- 不在代码中硬编码
- 使用最小权限原则（仅授予 OSS 上传权限）

### 5. CDN 访问控制

- CDN 配置 Referer 白名单（可选，限制访问来源）
- CDN 配置 IP 黑名单（可选，防止恶意访问）
- 启用 CDN 日志审计

## 成本估算

### 阿里云 OSS + CDN 成本（按月）

**假设**：
- 每个版本安装包总大小：200MB（4 个平台 × 50MB）
- 每月发布 2 个版本
- 每月下载次数：1000 次
- CDN 流量：200MB × 1000 = 200GB
- CDN 缓存命中率：90%

**费用明细**：
- **存储费用**：200MB × 2 = 400MB ≈ 0.12 元/GB/月 = 0.05 元
- **CDN 流量费用**：200GB × 0.24 元/GB = 48 元（国内 CDN 流量）
- **OSS 回源流量**：200GB × 10% = 20GB × 0.50 元/GB = 10 元
- **请求费用**：可忽略不计（PUT/GET 请求费用极低）

**总计**：约 **50-60 元/月**

**优化措施**：
- 设置 OSS 生命周期规则，只保留最近 3 个版本（节省存储费用）
- 提高 CDN 缓存命中率（延长 TTL）
- 使用阿里云资源包（降低单价）

## 监控与日志

### 应用端日志

记录以下事件（INFO 级别）：
- 配置的更新源（启动时）
- 检查更新开始（使用的 endpoint）
- 检查更新成功（版本号、下载 URL）
- 下载开始、进度、完成/失败
- 源降级事件（从 CDN 降级到 GitHub）
- 签名验证结果

### CI/CD 日志

记录以下事件：
- OSS 上传开始和结果
- 生成的更新清单内容
- 上传失败重试次数

### 阿里云 OSS/CDN 监控

- 启用 OSS 访问日志
- 启用 CDN 访问日志
- 监控流量和请求次数
- 设置费用告警（如月费用超过 100 元）

## 回滚计划

如果新功能出现严重问题，回滚步骤：

1. **应用端**：
   - 修改 `tauri.conf.json` 的 `updater.endpoints`
   - 仅保留 GitHub endpoint，删除 CDN endpoint
   - 发布热修复版本

2. **CI/CD**：
   - 停止上传到阿里云 OSS
   - 仅保留 GitHub Releases 上传

3. **数据清理**：
   - 保留 OSS Bucket 和数据（不删除，避免破坏已发布的版本）
   - 停止 CDN 加速（可选，节省成本）

**恢复时间**：< 1 小时（发布热修复版本）
