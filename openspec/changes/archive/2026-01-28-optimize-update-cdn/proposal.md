# 优化更新下载策略 - 增加阿里云 CDN 加速

## 为什么

当前应用的更新下载完全依赖 GitHub Releases，在中国大陆地区访问速度不稳定，经常只有 100-500KB/s，下载 20-50MB 的安装包需要数分钟甚至超时失败。这导致国内用户放弃使用自动更新功能，只能手动访问 GitHub 下载，严重影响用户体验。

## 背景

当前应用的更新机制完全依赖 GitHub Release 进行存储和分发，具体流程如下：

1. **CI/CD 构建**：GitHub Actions 自动构建多平台安装包（Windows x64/ARM64、macOS Intel/ARM64）
2. **存储**：将构建产物（NSIS、DMG、签名文件等）上传到 GitHub Releases
3. **更新清单**：生成 `latest.json` 文件存储在 GitHub Releases
4. **应用端检查**：使用 Tauri Updater 插件从 GitHub 下载更新包

**存在的问题**：

- **国内访问速度慢**：GitHub Releases 在中国大陆访问速度不稳定，下载大型安装包（通常 20-50MB）时可能需要数分钟甚至失败
- **用户体验差**：自动更新功能因网络问题导致下载缓慢或失败，用户可能放弃更新
- **单点依赖**：只有 GitHub 一个下载源，没有备用方案

## 目标

1. **使用阿里云 OSS + CDN 作为主要下载源**，为所有用户（尤其是国内用户）提供高速下载
2. **保持 GitHub 作为备用源**，CDN 失败时自动降级到 GitHub
3. **私有 OSS + CDN 回源**，提升安全性，避免文件被公开访问
4. **自动化同步**，CI/CD 自动将构建产物同步到阿里云 OSS

## 方案概述

### 1. 存储架构

```
GitHub Releases (备用源)
├── qsl-cardhub-v0.6.2-windows-x64-setup.exe
├── qsl-cardhub-v0.6.2-windows-x64-setup.exe.sig
├── qsl-cardhub-v0.6.2-macos-arm64.dmg
├── qsl-cardhub-v0.6.2-macos-arm64.app.tar.gz
├── qsl-cardhub-v0.6.2-macos-arm64.app.tar.gz.sig
└── latest.json (包含 GitHub URLs，用于备用)

阿里云 OSS (私有 Bucket)
├── qsl-cardhub-v0.6.2-windows-x64-setup.exe
├── qsl-cardhub-v0.6.2-windows-x64-setup.exe.sig
├── qsl-cardhub-v0.6.2-macos-arm64.dmg
├── qsl-cardhub-v0.6.2-macos-arm64.app.tar.gz
├── qsl-cardhub-v0.6.2-macos-arm64.app.tar.gz.sig
└── latest.json (包含阿里云 CDN URLs，主更新清单)

阿里云 CDN (回源到 OSS)
└── 通过 CDN 域名访问 OSS 文件（自动回源）
```

### 2. 下载流程

```
1. 应用检查更新
   ↓
2. 从阿里云 CDN 获取 latest.json
   ↓
3. 从阿里云 CDN 下载更新包
   ├── 成功 → 验证签名 → 安装更新
   └── 失败 → 降级到 GitHub Releases 重试
```

### 3. CI/CD 自动同步

```
GitHub Actions (release.yml)
├── 构建多平台安装包
├── 上传到 GitHub Releases (备用源)
│   └── 生成 latest.json (GitHub URLs)
└── 上传到阿里云 OSS (主源)
    ├── 上传所有安装包和签名文件
    └── 上传 latest.json (阿里云 CDN URLs)
```

## 技术细节

### 阿里云 OSS 配置

- **Bucket 名称**：`qsl-cardhub-releases`（示例）
- **访问控制**：私有（Private），不允许公开访问
- **CDN 回源**：
  - 配置阿里云 CDN 绑定自定义域名（如 `cdn.qsl-cardhub.com`）
  - CDN 回源到 OSS Bucket（私有 Bucket 访问）
  - CDN 自动缓存文件，减少 OSS 请求次数
- **目录结构**：
  ```
  /releases/
    /v0.6.2/
      qsl-cardhub-v0.6.2-windows-x64-setup.exe
      qsl-cardhub-v0.6.2-windows-x64-setup.exe.sig
      ...
  /latest.json  (主更新清单，包含 CDN URLs)
  ```

### CDN 回源优势

1. **安全性**：OSS Bucket 保持私有，仅 CDN 可回源访问，防止未授权访问
2. **性能**：CDN 节点缓存文件，全国访问速度快（国内用户 5-20MB/s）
3. **成本优化**：CDN 流量费用低于 OSS 外网流量费用
4. **简化管理**：无需管理复杂的 OSS 签名 URL，CDN 域名直接访问

### 更新清单格式

**latest.json (存储在阿里云 OSS 和 GitHub Releases)**：

阿里云 OSS 版本（主更新清单）：
```json
{
  "version": "0.6.2",
  "notes": "更新日志...",
  "pub_date": "2026-01-28T00:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "...",
      "url": "https://cdn.qsl-cardhub.com/releases/v0.6.2/qsl-cardhub-v0.6.2-windows-x64-setup.exe"
    }
  }
}
```

GitHub Releases 版本（备用）：
```json
{
  "version": "0.6.2",
  "notes": "更新日志...",
  "pub_date": "2026-01-28T00:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "...",
      "url": "https://github.com/HerbertGao/QSL-CardHub/releases/download/v0.6.2/qsl-cardhub-v0.6.2-windows-x64-setup.exe"
    }
  }
}
```

### 应用端逻辑

1. **检查更新**：
   - 主源：从 `https://cdn.qsl-cardhub.com/latest.json` 获取更新信息
   - 备用源：如果主源失败（超时 5 秒），降级到 `https://github.com/.../latest.json`

2. **下载更新包**：
   - 主源：从阿里云 CDN URL 下载
   - 备用源：如果主源下载失败，自动切换到 GitHub URL 重试

3. **签名验证**：
   - 使用相同的签名验证逻辑（无论从哪个源下载）
   - 签名不通过则拒绝安装

## 实现范围

### 阶段 1：基础设施准备
- [ ] 配置阿里云 OSS Bucket（私有权限）
- [ ] 配置阿里云 CDN，绑定自定义域名（如 `cdn.qsl-cardhub.com`）
- [ ] 配置 CDN 回源规则（回源到 OSS）
- [ ] 在 GitHub Secrets 中配置阿里云访问凭据（AccessKey ID/Secret）

### 阶段 2：CI/CD 集成
- [ ] 修改 `release.yml` 工作流，增加上传到阿里云 OSS 的步骤
- [ ] 上传所有平台的安装包和签名文件到 OSS
- [ ] 生成包含阿里云 CDN URLs 的 `latest.json` 并上传到 OSS
- [ ] 保留 GitHub Releases 上传逻辑（作为备用源）

### 阶段 3：应用端降级逻辑
- [ ] 修改前端更新检查逻辑，优先从阿里云 CDN 获取更新清单
- [ ] 实现 CDN 失败时自动降级到 GitHub 的逻辑
- [ ] 实现下载失败时自动切换到备用源
- [ ] 添加日志记录下载源和失败原因

### 阶段 4：测试与优化
- [ ] 在国内网络环境测试阿里云 CDN 下载速度
- [ ] 在国外网络环境测试阿里云 CDN 下载速度
- [ ] 测试 CDN 失败时降级到 GitHub 的逻辑
- [ ] 验证签名验证在两个源都正常工作
- [ ] 优化超时设置和重试逻辑

## 成功标准

1. **国内下载速度提升**：国内用户从阿里云 CDN 下载速度 ≥ 5MB/s（相比 GitHub 的 100-500KB/s）
2. **国际用户体验**：国外用户从阿里云 CDN 下载速度 ≥ 1MB/s（可接受范围）
3. **高可用性**：CDN 失败时能自动降级到 GitHub，成功率 ≥ 99%
4. **CI/CD 自动化**：每次发布时自动同步到两个源，无需手动操作
5. **安全性**：OSS 保持私有，仅通过 CDN 访问，防止未授权访问

## 潜在风险

1. **成本**：阿里云 OSS 和 CDN 会产生存储和流量费用
   - **缓解措施**：设置生命周期规则，只保留最近 3 个版本；预估每月成本约 50-100 元
2. **同步延迟**：CI/CD 上传到阿里云可能需要额外时间
   - **缓解措施**：并行上传 GitHub 和阿里云，延长总发布时间约 1-2 分钟
3. **域名管理**：需要配置和维护 CDN 域名
   - **缓解措施**：注册简单域名并配置 HTTPS 证书
4. **国外访问速度**：国外用户访问阿里云 CDN 可能比 GitHub 慢
   - **缓解措施**：保留 GitHub 作为备用源，如果 CDN 过慢可降级

## 替代方案

### 方案 A：公共读 OSS（不使用 CDN 回源）
- **优点**：配置���简单
- **缺点**：安全性较低，OSS 文件可被公开访问；流量费用更高

### 方案 B：使用 Cloudflare R2 或 AWS S3
- **优点**：全球 CDN 覆盖，国内外都快
- **缺点**：成本较高（Cloudflare R2 免费额度有限，AWS S3 流量费用高）

### 方案 C：保持现状，仅优化 GitHub 下载
- **优点**：零成本，零维护
- **缺点**：无法解决国内访问慢的根本问题

**选择当前方案（阿里云 OSS + CDN 回源）的理由**：
- 提供最佳的国内下载体验
- 私有 Bucket + CDN 回源提升安全性
- 成本可控（阿里云 OSS + CDN 价格低廉）
- 保留 GitHub 作为备用源，确保高可用性

## 未来扩展

1. **多地域 CDN**：使用阿里云全站加速（DCDN），进一步优化全球访问速度
2. **智能 DNS**：根据用户 IP 自动解析到最近的 CDN 节点
3. **增量更新**：仅下载差异文件，减少更新包大小
4. **断点续传**：支持大文件下载中断后继续，提升成功率
