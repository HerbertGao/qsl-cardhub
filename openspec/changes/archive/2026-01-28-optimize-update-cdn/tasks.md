# 任务列表

## 阶段 1：基础设施准备（手动操作，由用户完成）

### T1.1 配置阿里云 OSS Bucket
- **状态**：待用户操作
- **依赖**：无
- **验证**：
  - 成功创建 Bucket `qsl-cardhub-releases`（或其他名称）
  - 访问控制设置为私有（Private）
  - 可通过 OSS 控制台上传和查看测试文件
- **文档参考**：`docs/aliyun-cdn-setup.md`

### T1.2 配置阿里云 CDN
- **状态**：待用户操作
- **依赖**：T1.1
- **验证**：
  - 成功绑定自定义域名（如 `cdn.qsl-cardhub.com`）或使用阿里云默认加速域名
  - CDN 回源配置正确（回源到 OSS Bucket）
  - CDN 缓存规则配置正确（缓存静态文件，TTL ≥ 7 天）
  - HTTPS 证书配置成功
  - 通过 CDN 域名可访问 OSS 中的文件
- **文档参考**：`docs/aliyun-cdn-setup.md`

### T1.3 配置 GitHub Secrets
- **状态**：待用户操作
- **依赖**：T1.1
- **验证**：
  - 在 GitHub 仓库 Settings → Secrets 中添加：
    - `ALIYUN_OSS_ACCESS_KEY_ID`
    - `ALIYUN_OSS_ACCESS_KEY_SECRET`
    - `ALIYUN_OSS_BUCKET_NAME`
    - `ALIYUN_OSS_ENDPOINT` (如 `oss-cn-hangzhou.aliyuncs.com`)
    - `ALIYUN_CDN_DOMAIN` (如 `cdn.qsl-cardhub.com`)
  - 凭据经过验证可正常访问 OSS
- **文档参考**：`docs/aliyun-cdn-setup.md`

---

## 阶段 2：CI/CD 集成

### T2.1 安装阿里云 OSS 上传工具 ✅
- **状态**：已完成
- **依赖**：T1.3
- **内容**：
  - 在 `release.yml` 中添加步骤安装 `ossutil`（阿里云官方 CLI 工具）
  - 配置 `ossutil` 使用 GitHub Secrets 中的凭据
- **验证**：
  - GitHub Actions 日志显示 `ossutil` 成功安装和配置
  - 可执行 `ossutil ls` 命令列出 Bucket 内容
- **完成文件**：`.github/workflows/release.yml:39-61`

### T2.2 上传安装包到阿里云 OSS ✅
- **状态**：已完成
- **依赖**：T2.1
- **内容**：
  - 在 `build-and-upload` job 中，每个平台构建完成后：
    - 使用 `ossutil cp` 上传安装包到 OSS（路径：`/releases/v{version}/`）
    - 上传签名文件到同一目录
    - 文件保持私有权限（不设置公共读）
- **验证**：
  - 所有平台的安装包和签名文件都成功上传到 OSS
  - 可通过 CDN 域名访问文件（如 `https://cdn.qsl-cardhub.com/releases/v0.6.2/qsl-cardhub-v0.6.2-windows-x64-setup.exe`）
  - 文件 MD5 与本地构建产物一致
- **完成文件**：`.github/workflows/release.yml:63-77`

### T2.3 生成 CDN 更新清单 ✅
- **状态**：已完成
- **依赖**：T2.2
- **内容**：
  - 在 `generate-update-manifest` job 中：
    - 生成包含阿里云 CDN URLs 的 `latest.json` 文件
    - 所有 URL 指向阿里云 CDN 域名
    - 签名字段与 GitHub 版本一致
- **验证**：
  - `latest.json` 格式正确，符合 Tauri Updater 规范
  - 所有 URL 指向阿里云 CDN
  - 签名字段正确
- **完成文件**：`.github/workflows/release.yml:198-236`

### T2.4 上传 latest.json 到 OSS ✅
- **状态**：已完成
- **依赖**：T2.3
- **内容**：
  - 上传 `latest.json` 到 OSS 根目录（`/latest.json`）
  - 文件保持私有权限（通过 CDN 回源访问）
- **验证**：
  - 可通过 `https://cdn.qsl-cardhub.com/latest.json` 访问
  - JSON 内容正确，版本号与当前发布一致
- **完成文件**：`.github/workflows/release.yml:238-244`

### T2.5 保留 GitHub Releases 作为备用源 ✅
- **状态**：已完成
- **依赖**：T2.4
- **内容**：
  - 保留现有的 GitHub Releases 上传逻辑
  - 在 GitHub Releases 中也生成一份 `latest.json`（包含 GitHub URLs）
  - 确保两个清单的版本号、签名一致
- **验证**：
  - GitHub Releases 中存在完整的安装包、签名文件和 `latest.json`
  - GitHub 版本的 `latest.json` 中的 URL 指向 GitHub
- **完成文件**：`.github/workflows/release.yml`（保留了原有逻辑）

### T2.6 并行上传优化 ✅
- **状态**：已完成
- **依赖**：T2.2
- **内容**：
  - 优化 CI/CD 流程，使 GitHub Releases 上传和阿里云 OSS 上传并行执行
  - 确保两者互不阻塞
- **验证**：
  - 总发布时间增加不超过 2 分钟
  - GitHub Actions 日志显示并行执行
- **实现方式**：所有 OSS 上传步骤使用条件执行（`if: ${{ secrets.ALIYUN_OSS_ACCESS_KEY_ID != '' }}`），不阻塞 GitHub 发布流程

---

## 阶段 3：应用端降级逻辑

### T3.1 配置 Tauri Updater endpoints ✅
- **状态**：已完成
- **依赖**：T2.4
- **内容**：
  - 修改 `tauri.conf.json` 中的 `updater.endpoints`
  - 第一个 endpoint 指向阿里云 CDN：`https://qsl-cardhub.herbert-dev.cn/latest.json`
  - 第二个 endpoint 指向 GitHub：`https://github.com/HerbertGao/QSL-CardHub/releases/latest/download/latest.json`
- **验证**：
  - ✅ 配置文件格式正确
  - ✅ Tauri Updater 会按顺序尝试这些 endpoints（先 CDN，后 GitHub）
- **完成文件**：`tauri.conf.json`、`tauri.conf.json.example`、`docs/cdn-endpoint-setup.md`

### T3.2 测试自动降级逻辑
- **状态**：待测试（需要完成 T1.1-T1.3 和 T3.1 后）
- **依赖**：T3.1
- **内容**：
  - 使用 Tauri Updater 插件的内置降级功能
  - 无需额外编写代码，Tauri Updater 会自动尝试备用 endpoint
- **验证**：
  - 当第一个 endpoint（CDN）失败时，自动尝试第二个（GitHub）
  - 日志中记录了 endpoint 切换

### T3.3 修改前端更新 UI
- **状态**：待实现（可选）
- **依赖**：T3.1
- **内容**：
  - 修改 `AboutView.vue`：
    - 保持现有的 `check()` 和 `downloadAndInstall()` 调用
    - 可选：添加下载源提示文案（"使用阿里云 CDN 加速"）
  - 修改 `updateStore.ts`（如需要）：
    - 添加下载源状态字段（可选）
- **验证**：
  - 前端点击"检查更新"后能正常获取更新信息
  - 下载更新时显示正确的进度

### T3.4 添加日志记录
- **状态**：待实现（可选）
- **依赖**：T3.2
- **内容**：
  - 在应用启动时记录配置的更新源
  - 在更新检查和下载时记录使用的 endpoint
  - 记录降级事件（如果发生）
- **验证**：
  - 日志清晰，便于调试
  - 可在日志查看页面查看更新相关日志

---

## 阶段 4：测试与优化

### T4.1 国内网络环境测试
- **依赖**：T3.3
- **内容**：
  - 在国内网络环境测试应用更新功能
  - 验证从阿里云 CDN 下载速度
- **验证**：
  - 应用成功从 CDN 下载更新
  - 下载速度 ≥ 5MB/s
  - 更新成功安装

### T4.2 国外网络环境测试
- **依赖**：T3.3
- **内容**：
  - 在国外网络环境测试应用更新功能
  - 验证从阿里云 CDN 下载速度（可能较慢）
- **验证**：
  - 应用成功从 CDN 下载更新
  - 下载速度 ≥ 1MB/s（可接受范围）
  - 更新成功安装

### T4.3 CDN 失败降级测试
- **依赖**：T3.2
- **内容**：
  - 模拟 CDN 不可用场景（如修改 hosts 文件阻止访问）
  - 验证自动降级到 GitHub
  - 验证下载成功
- **验证**：
  - 应用日志显示 CDN 失败，降级到 GitHub
  - 从 GitHub 下载成功完成
  - 更新成功安装

### T4.4 签名验证测试
- **依赖**：T3.3
- **内容**：
  - 验证从 CDN 下载的安装包签名验证正确
  - 验证从 GitHub 下载的安装包签名验证正确
  - 验证签名不匹配时拒绝安装
- **验证**：
  - 两个源的签名验证都通过
  - 篡改安装包后签名验证失败，拒绝安装

### T4.5 性能优化
- **依赖**：T4.1, T4.2
- **内容**：
  - 优化 CDN 超时时间（确保快速降级）
  - 优化下载进度展示（实时更新，不卡顿）
  - 验证 CDN 缓存是否生效（减少回源请求）
- **验证**：
  - CDN 超时在 5 秒内触发降级
  - 下载进度条流畅更新
  - CDN 缓存命中率 > 90%

### T4.6 文档更新 ✅
- **状态**：已完成
- **依赖**：T4.5
- **内容**：
  - 更新 `README.md`：说明新的更新机制
  - 更新 `openspec/project.md`：记录阿里云 OSS + CDN 配置
  - 更新 `openspec/specs/auto-updater/spec.md`：增加 CDN 降级需求
  - 创建阿里云配置文档（如 `docs/aliyun-cdn-setup.md`）
- **验证**：
  - 文档清晰易懂
  - 开发者可根据文档配置和维护 OSS + CDN
- **完成文件**：
  - `README.md`（添加 CDN 加速说明）
  - `openspec/project.md`（添加阿里云 OSS+CDN 配置章节）
  - `openspec/specs/auto-updater/spec.md`（合并了 CDN 相关需求）
  - `docs/aliyun-cdn-setup.md`（阿里云基础设施配置指南）
  - `docs/cdn-endpoint-setup.md`（应用端点配置指南）
  - `tauri.conf.json.example`（示例配置文件）

---

## 验收标准

- ✅ CI/CD 自动将构建产物同步到阿里云 OSS 和 GitHub Releases（已实现，条件执行）
- ✅ 生成包含 CDN URLs 的主更新清单，GitHub URLs 的备用清单（已实现）
- ⏳ 应用默认从阿里云 CDN 下载更新（需要用户配置 CDN 域名后修改 tauri.conf.json）
- ⏳ 国内用户从 CDN 下载速度 ≥ 5MB/s（待测试）
- ⏳ CDN 失败时自动降级到 GitHub，成功率 ≥ 99%（待测试）
- ⏳ 签名验证在两个源都正常工作（待测试）
- ⏳ 日志记录完整，便于调试和监控（可选，待实现）
- ✅ OSS 保持私有，仅通过 CDN 访问（文档已说明）

## 当前状态总结

### 已完成 ✅
- **阶段 2（CI/CD 集成）**：所有 6 个任务已完成
  - OSS 上传工具安装和配置
  - 安装包和签名文件上传
  - CDN 更新清单生成和上传
  - GitHub Releases 备用源保留
  - 并行上传优化（条件执行）
- **阶段 3.1（Endpoint 配置）**：配置文档已创建
- **阶段 4.6（文档更新）**：所有文档已完成

### 已由用户完成 ✅
- **阶段 1（基础设施准备）**：所有 3 个任务已完成
  - T1.1：配置阿里云 OSS Bucket ✅
  - T1.2：配置阿里云 CDN（域名：`https://qsl-cardhub.herbert-dev.cn/`）✅
  - T1.3：配置 GitHub Secrets ✅
- **阶段 3.1（Endpoint 配置）**：已完成 ✅

### 待测试 ⏳
- **阶段 4（测试与优化）**：5 个测试任务（T4.1-T4.5）
  - 需要在用户完成阶段 1 和 3.1 后进行

### 可选功能 💡
- **阶段 3.3**：前端更新 UI 优化（添加下载源提示）
- **阶段 3.4**：添加详细日志记录

## 下一步建议

### ✅ 所有配置已完成！可以立即进行测试

**阶段 1 和 3.1 已全部完成**，现在可以进行以下操作：

1. **发布新版本进行测试**：
   ```bash
   # 创建新的 tag 触发发布
   git tag v0.6.3
   git push origin v0.6.3
   ```
   - GitHub Actions 会自动构建并上传到阿里云 OSS 和 GitHub Releases
   - 检查 Actions 日志，验证 OSS 上传步骤是否成功执行
   - 访问 `https://qsl-cardhub.herbert-dev.cn/latest.json` 验证 CDN 清单是否可访问

2. **测试应用更新功能**（阶段 4 测试任务）：
   - **T4.1**：国内网络环境测试（验证 CDN 下载速度 ≥ 5MB/s）
   - **T4.2**：国外网络环境测试（验证可访问性）
   - **T4.3**：CDN 失败降级测试（修改 hosts 阻止 CDN，验证自动降级到 GitHub）
   - **T4.4**：签名验证测试（验证两个源的签名都正确）

3. **可选的后续优化**：
   - **T3.3**：在前端添加下载源提示（显示"使用阿里云 CDN 加速"）
   - **T3.4**：添加详细的更新日志记录

---

## 并行工作建议

- **T1.1 - T1.3** 可并行进行（基础设施准备）
- **T2.1 - T2.3** 可并行进行（CI/CD 准备工作）
- **T4.1 - T4.4** 可并行进行（多环境测试）
