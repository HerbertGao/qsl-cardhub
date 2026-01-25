# 顺丰速运默认配置说明

## 配置文件

`sf_express_default.toml` 包含顺丰速运 API 的默认凭据，用于为用户提供开箱即用的体验。

由于包含敏感信息，此文件已添加到 `.gitignore`，不会提交到公开仓库。

## GitHub Secrets 配置

在 GitHub 仓库中配置以下 Secrets，用于 CI/CD 构建时自动生成配置文件：

| Secret 名称 | 说明 |
|------------|------|
| `SF_PARTNER_ID` | 顺丰顾客编码（partnerID） |
| `SF_CHECKWORD_SANDBOX` | 沙箱环境校验码 |
| `SF_CHECKWORD_PROD` | 生产环境校验码 |

### 配置步骤

1. 进入 GitHub 仓库页面
2. 点击 **Settings** → **Secrets and variables** → **Actions**
3. 点击 **New repository secret**
4. 依次添加上述 Secrets

## GitHub Actions 使用示例

在构建工作流中添加以下步骤，生成配置文件：

```yaml
- name: Generate SF Express Config
  run: |
    cat > config/sf_express_default.toml << TOML
    enabled = true
    partner_id = "${{ secrets.SF_PARTNER_ID }}"
    checkword_sandbox = "${{ secrets.SF_CHECKWORD_SANDBOX }}"
    checkword_prod = "${{ secrets.SF_CHECKWORD_PROD }}"
    TOML
```

## 本地开发

本地开发时，手动编辑 `config/sf_express_default.toml` 文件：

```toml
enabled = true
partner_id = "你的顾客编码"
checkword_sandbox = "沙箱校验码"
checkword_prod = "生产校验码"
```

## 注意事项

- 切勿将包含真实凭据的配置文件提交到公开仓库
- 定期轮换 API 凭据以确保安全
- 如果凭据泄露，立即在顺丰开放平台重置
