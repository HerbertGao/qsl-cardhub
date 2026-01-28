# CDN Endpoint 配置说明

当您完成阿里云 OSS + CDN 配置后，需要修改 `tauri.conf.json` 中的 `updater.endpoints` 配置，启用 CDN 加速下载。

## 配置步骤

### 1. 确认 CDN 域名

首先确认您的阿里云 CDN 域名，例如：
- `cdn.qsl-cardhub.com`（自定义域名）
- 或 `qsl-cardhub-releases.oss-cn-hangzhou.aliyuncs.com`（OSS 默认域名）

### 2. 修改 tauri.conf.json

打开 `tauri.conf.json` 文件，找到 `plugins.updater.endpoints` 配置：

**修改前**（仅使用 GitHub）：
```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/HerbertGao/QSL-CardHub/releases/latest/download/latest.json"
      ]
    }
  }
}
```

**修改后**（CDN 优先，GitHub 备用）：
```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://cdn.qsl-cardhub.com/latest.json",
        "https://github.com/HerbertGao/QSL-CardHub/releases/latest/download/latest.json"
      ]
    }
  }
}
```

### 3. 工作原理

Tauri Updater 会按顺序尝试这些 endpoints：

1. **第一个 endpoint（CDN）**：优先从阿里云 CDN 获取更新信息
   - 国内用户：高速访问（5-20MB/s）
   - 国外用户：可接受速度（1-5MB/s）
   
2. **第二个 endpoint（GitHub）**：CDN 失败时自动降级
   - 超时或网络错误时自动切换
   - 确保全球用户都能正常更新

### 4. 验证配置

配置完成后，您可以通过以下方式验证：

#### 4.1 测试 CDN 访问

在浏览器或命令行访问您的 CDN 域名：

```bash
curl https://cdn.qsl-cardhub.com/latest.json
```

应该返回类似以下内容的 JSON：

```json
{
  "version": "0.6.2",
  "notes": "更新日志...",
  "pub_date": "2026-01-28T00:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "...",
      "url": "https://cdn.qsl-cardhub.com/releases/v0.6.2/qsl-cardhub-v0.6.2-macos-arm64.app.tar.gz"
    },
    ...
  }
}
```

#### 4.2 测试应用更新

1. 构建应用：`cargo tauri build`
2. 运行应用
3. 进入"关于"页面，点击"检查更新"
4. 查看日志，确认使用了 CDN endpoint

### 5. 常见问题

#### Q1: CDN endpoint 一直超时

**原因**：CDN 域名配置错误或 DNS 未生效

**解决方案**：
1. 检查 CDN 域名是否正确
2. 使用 `ping` 或 `nslookup` 检查 DNS 解析
3. 等待 DNS 生效（通常 10 分钟内）

#### Q2: 下载速度仍然很慢

**原因**：CDN 缓存未命中，正在回源

**解决方案**：
1. 检查 CDN 缓存配置（TTL 应 ≥ 7 天）
2. 手动预热 CDN（在 CDN 控制台进行）
3. 等待 CDN 缓存生效（首次访问会回源）

#### Q3: 想暂时禁用 CDN

**方法1**：仅保留 GitHub endpoint
```json
{
  "endpoints": [
    "https://github.com/HerbertGao/QSL-CardHub/releases/latest/download/latest.json"
  ]
}
```

**方法2**：调换顺序，GitHub 优先
```json
{
  "endpoints": [
    "https://github.com/HerbertGao/QSL-CardHub/releases/latest/download/latest.json",
    "https://cdn.qsl-cardhub.com/latest.json"
  ]
}
```

### 6. 下一步

配置完成后，下次发布新版本时，CI/CD 会自动：
1. 构建安装包
2. 上传到 GitHub Releases 和阿里云 OSS
3. 生成两个更新清单（GitHub + CDN）
4. 应用会优先从 CDN 下载更新

---

更多信息请参考：[阿里云 OSS + CDN 配置指南](./aliyun-cdn-setup.md)
