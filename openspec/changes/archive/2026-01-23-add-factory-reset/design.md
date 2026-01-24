# 设计文档：抹掉所有内容和设置功能

## 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                     AboutView.vue                            │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              危险操作区域                              │    │
│  │  ┌──────────────────────┐                           │    │
│  │  │  抹掉所有内容和设置按钮      │ ────> 确认对话框         │    │
│  │  └──────────────────────┘                           │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                   Tauri Command                              │
│                   factory_reset                              │
└─────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│   删除数据库     │ │   删除配置文件   │ │   清除钥匙串     │
│   cards.db      │ │   config.toml   │ │   凭据           │
│                 │ │   template_*.toml│ │                 │
└─────────────────┘ └─────────────────┘ └─────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │   重启应用       │
                    │   relaunch()    │
                    └─────────────────┘
```

## 数据清除策略

### 需要清除的文件

| 文件/目录 | 位置（生产环境） | 说明 |
|-----------|-----------------|------|
| `cards.db` | `{config_dir}/cards.db` | SQLite 数据库 |
| `config.toml` | `{config_dir}/config.toml` | 打印机配置 |
| `template_config.toml` | `{config_dir}/template_config.toml` | 模板配置 |
| `logs/` | `{config_dir}/logs/` | 可选删除 |

### 需要保留的文件

| 文件 | 位置 | 保留原因 |
|------|------|----------|
| `templates/default.toml` | `{config_dir}/templates/default.toml` | 默认模板，重启后需要使用 |

### 钥匙串凭据键

```rust
// QRZ.cn
"qsl-cardhub:qrz:username"
"qsl-cardhub:qrz:password"
"qsl-cardhub:qrz:session"

// QRZ.com
"qsl-cardhub:qrz.com:username"
"qsl-cardhub:qrz.com:password"
"qsl-cardhub:qrz.com:session"

// 顺丰速运
"qsl-cardhub:sf:partner_id"
"qsl-cardhub:sf:checkword_prod"
"qsl-cardhub:sf:checkword_sandbox"
"qsl-cardhub:sf:environment"

// 云同步
"qsl-cardhub:sync:api_key"
```

## 配置目录路径

根据操作系统不同，配置目录位于：

- **Windows**: `%APPDATA%/qsl-cardhub/`
- **macOS**: `~/Library/Application Support/qsl-cardhub/`
- **Linux**: `~/.config/qsl-cardhub/`

## API 设计

### Tauri 命令

```rust
#[tauri::command]
pub async fn factory_reset() -> Result<(), String>
```

**返回值**：
- `Ok(())` - 重置成功
- `Err(String)` - 错误信息

### 前端调用

```typescript
import { invoke } from '@tauri-apps/api/core'
import { relaunch } from '@tauri-apps/plugin-process'

async function handleFactoryReset() {
  await invoke('factory_reset')
  await relaunch()
}
```

## 用户界面设计

### 确认对话框内容

```
标题：抹掉所有内容和设置

此操作将删除以下数据，且无法恢复：

• 所有项目和卡片数据
• 打印机配置
• QRZ.cn / QRZ.com 登录凭据
• 顺丰速运配置
• 云同步配置

默认打印模板将被保留。

确定要继续吗？

[取消]  [确认重置]
```

### 按钮样式

- 使用 Element Plus 的 `el-button` 组件
- 类型：`type="danger"`
- 文字：「抹掉所有内容和设置」

## 错误处理

1. **文件删除失败**：记录错误但继续尝试删除其他文件
2. **钥匙串清除失败**：记录错误但不阻断流程
3. **重启失败**：提示用户手动重启

## 安全考虑

1. **二次确认**：必须通过确认对话框才能执行
2. **不可逆操作**：明确告知用户数据将被永久删除
3. **凭据清除**：确保敏感信息从系统钥匙串中完全移除
