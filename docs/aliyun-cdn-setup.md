# 阿里云 OSS + CDN 配置指南

本文档说明如何配置阿里云 OSS 和 CDN，以实现应用更新的高速下载。

## 前置条件

- 阿里云账号
- 已实名认证
- 已开通 OSS 和 CDN 服务

---

## 第一步：创建 OSS Bucket

1. 登录阿里云控制台：https://oss.console.aliyun.com/
2. 点击"创建 Bucket"
3. 配置 Bucket：
   - **Bucket 名称**：`qsl-cardhub-releases`（或自定义名称）
   - **地域**：推荐选择 `华东1（杭州）` 或 `华北2（北京）`
   - **存储类型**：标准存储
   - **读写权限**：**私有**（重要！不要选择公共读）
   - **版本控制**：不启用
   - **服务端加密**：不启用（可选）
4. 点击"确定"创建

---

## 第二步：配置 CDN 加速

### 选项 A：使用阿里云默认加速域名（推荐快速测试）

1. 在 OSS Bucket 详情页，点击"域名管理" → "绑定用户域名"
2. 选择"自动添加 CNAME 记录"
3. 阿里云会自动分配一个加速域名，格式类似：
   ```
   qsl-cardhub-releases.oss-cn-hangzhou.aliyuncs.com
   ```
4. 记录这个域名，稍后在 GitHub Secrets 中使用

### 选项 B：使用自定义域名（推荐正式环境）

#### 2.1 准备域名

- 已注册域名（如 `qsl-cardhub.com`）
- 域名已完成备案（如果服务器在中国大陆）

#### 2.2 创建 CDN 加速域名

1. 登录 CDN 控制台：https://cdn.console.aliyun.com/
2. 点击"域名管理" → "添加域名"
3. 配置加速域名：
   - **加速域名**：`cdn.qsl-cardhub.com`（自定义）
   - **业务类型**：下载加速
   - **源站信息**：
     - 类型：OSS 域名
     - 域名：选择刚创建的 OSS Bucket（`qsl-cardhub-releases.oss-cn-hangzhou.aliyuncs.com`）
   - **加速区域**：全球加速（或仅中国大陆）
4. 点击"下一步"

#### 2.3 配置 HTTPS 证书

1. 在 CDN 域名详情页，点击"HTTPS 配置"
2. 点击"修改配置" → "免费证书"或上传自己的证书
3. 如果使用免费证书：
   - 选择"申请免费证书"
   - 等待证书签发（通常几分钟）
4. 启用"HTTPS 安全加速"和"强制跳转 HTTPS"
5. 保存配置

#### 2.4 配置 DNS 解析

1. 登录域名 DNS 管理（如阿里云 DNS）
2. 添加 CNAME 记录：
   - **记录类型**：CNAME
   - **主机记录**：`cdn`
   - **记录值**：阿里云 CDN 提供的 CNAME 值（如 `cdn.qsl-cardhub.com.w.kunlunsl.com`）
   - **TTL**：10 分钟
3. 保存记录
4. 等待 DNS 生效（通常 10 分钟内）

#### 2.5 配置缓存规则

1. 在 CDN 域名详情页，点击"缓存配置"
2. 添加缓存规则：
   - **路径**：`/releases/*`
   - **缓存时间**：7 天
   - **说明**：缓存安装包文件
3. 添加第二条规则：
   - **路径**：`/latest.json`
   - **缓存时间**：1 小时
   - **说明**：缓存更新清单（较短时间，便于快速更新）
4. 保存配置

---

## 第三步：创建 AccessKey

1. 登录阿里云控制台
2. 点击右上角头像 → "AccessKey 管理"
3. 点击"创建 AccessKey"
4. **重要**：记录 `AccessKey ID` 和 `AccessKey Secret`（仅显示一次）
5. 建议创建 RAM 用户，仅授予 OSS 上传权限（最小权限原则）：
   - 登录 RAM 控制台：https://ram.console.aliyun.com/
   - 创建 RAM 用户，勾选"编程访问"
   - 授予权限：`AliyunOSSFullAccess`（或自定义策略，仅允许上传到指定 Bucket）
   - 记录 RAM 用户的 AccessKey

---

## 第四步：配置 GitHub Secrets

1. 进入 GitHub 仓库：https://github.com/HerbertGao/QSL-CardHub
2. 点击"Settings" → "Secrets and variables" → "Actions"
3. 点击"New repository secret"，逐个添加以下密钥：

| 密钥名称 | 值 | 说明 |
|---------|-----|------|
| `ALIYUN_OSS_ACCESS_KEY_ID` | 你的 AccessKey ID | 阿里云访问凭证 |
| `ALIYUN_OSS_ACCESS_KEY_SECRET` | 你的 AccessKey Secret | 阿里云访问密钥 |
| `ALIYUN_OSS_BUCKET_NAME` | `qsl-cardhub-releases` | OSS Bucket 名称 |
| `ALIYUN_OSS_ENDPOINT` | `oss-cn-hangzhou.aliyuncs.com` | OSS 访问域名（根据地域调整） |
| `ALIYUN_CDN_DOMAIN` | `cdn.qsl-cardhub.com` | CDN 加速域名（或 OSS 默认域名） |

---

## 第五步：验证配置

### 5.1 测试 OSS 上传

在本地安装 `ossutil` 工具：

```bash
# macOS
brew install ossutil

# 或下载二进制文件
# https://github.com/aliyun/ossutil/releases
```

配置 `ossutil`：

```bash
ossutil config \
  -e oss-cn-hangzhou.aliyuncs.com \
  -i <AccessKey ID> \
  -k <AccessKey Secret>
```

测试上传文件：

```bash
# 创建测试文件
echo "Hello OSS" > test.txt

# 上传到 OSS
ossutil cp test.txt oss://qsl-cardhub-releases/test.txt

# 验证上传成功
ossutil ls oss://qsl-cardhub-releases/
```

### 5.2 测试 CDN 访问

在浏览器访问：

```
https://cdn.qsl-cardhub.com/test.txt
```

或使用 `curl`：

```bash
curl https://cdn.qsl-cardhub.com/test.txt
```

如果返回 "Hello OSS"，说明 CDN 配置成功！

### 5.3 验证私有权限

尝试直接访问 OSS URL（不通过 CDN）：

```bash
curl https://qsl-cardhub-releases.oss-cn-hangzhou.aliyuncs.com/test.txt
```

应该返回 403 错误（Access Denied），说明 Bucket 是私有的，仅 CDN 可回源访问。

---

## 第六步：配置生命周期规则（可选）

为了节省存储成本，可以配置生命周期规则自动删除旧版本：

1. 在 OSS Bucket 详情页，点击"基础设置" → "生命周期"
2. 点击"创建规则"
3. 配置规则：
   - **规则名称**：`删除旧版本`
   - **策略**：整个 Bucket
   - **文件过期规则**：
     - 勾选"指定前缀"：`releases/`
     - 勾选"过期天数"：90 天
   - **说明**：自动删除 90 天前的旧版本文件
4. 保存规则

**注意**：`latest.json` 不会被删除（因为它不在 `releases/` 目录下）

---

## 常见问题

### Q1: CDN 访问返回 403 错误

**原因**：CDN 回源配置错误，无法访问私有 OSS Bucket

**解决方案**：
1. 检查 CDN 域名的源站配置，确认指向正确的 OSS Bucket
2. 检查 OSS Bucket 的 CDN 回源设置，确认已允许 CDN 访问

### Q2: 上传文件后，CDN 访问仍是旧版本

**原因**：CDN 缓存未刷新

**解决方案**：
1. 登录 CDN 控制台
2. 点击"刷新预热" → "刷新"
3. 输入需要刷新的 URL（如 `https://cdn.qsl-cardhub.com/latest.json`）
4. 点击"提交"
5. 等待几分钟后重新访问

### Q3: GitHub Actions 上传失败，提示权限错误

**原因**：AccessKey 权限不足或配置错误

**解决方案**：
1. 检查 GitHub Secrets 中的 `ALIYUN_OSS_ACCESS_KEY_ID` 和 `ALIYUN_OSS_ACCESS_KEY_SECRET` 是否正确
2. 检查 RAM 用户权限，确保有 OSS 上传权限
3. 检查 OSS Bucket 名称和 Endpoint 是否匹配

### Q4: CDN 费用过高

**原因**：下载量大或缓存命中率低

**解决方案**：
1. 优化缓存规则，延长 TTL（如 7 天）
2. 启用智能压缩（CDN 控制台 → 性能优化）
3. 设置费用告警（费用中心 → 费用预警）
4. 考虑购买阿里云资源包降低单价

---

## 下一步

配置完成后，继续执行以下步骤：

1. ✅ 配置完成阿里云 OSS 和 CDN
2. ✅ 配置完成 GitHub Secrets
3. ⏭️ CI/CD 会在下次发布时自动上传到 OSS
4. ⏭️ 应用会自动从 CDN 下载更新

如有问题，请参考阿里云官方文档：
- OSS 文档：https://help.aliyun.com/product/31815.html
- CDN 文档：https://help.aliyun.com/product/27099.html
