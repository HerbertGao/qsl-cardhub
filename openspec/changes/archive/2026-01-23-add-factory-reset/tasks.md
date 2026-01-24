# 任务清单：抹掉所有内容和设置功能

## 阶段 1：后端实现

### 1.1 新增钥匙串凭据清除函数
- [x] 在 `src/security/credentials.rs` 中添加 `clear_all_credentials()` 函数
- [x] 清除所有已知的凭据键：
  - `qsl-cardhub:qrz:username`
  - `qsl-cardhub:qrz:password`
  - `qsl-cardhub:qrz:session`
  - `qsl-cardhub:qrz.com:username`
  - `qsl-cardhub:qrz.com:password`
  - `qsl-cardhub:qrz.com:session`
  - `qsl-cardhub:sf:partner_id`
  - `qsl-cardhub:sf:checkword_prod`
  - `qsl-cardhub:sf:checkword_sandbox`
  - `qsl-cardhub:sf:environment`
  - `qsl-cardhub:sync:api_key`
- **验证**：单元测试

### 1.2 新增抹掉所有内容和设置命令
- [x] 在 `src/commands/` 下新建 `factory_reset.rs`
- [x] 实现 `factory_reset` Tauri 命令：
  1. 删除数据库文件 (`cards.db`)
  2. 删除配置文件 (`config.toml`, `template_config.toml`)
  3. 保留 `templates/default.toml`
  4. 调用 `clear_all_credentials()`
- [x] 在 `src/commands/mod.rs` 中导出
- [x] 在 `src/main.rs` 中注册命令
- **验证**：手动测试

## 阶段 2：前端实现

### 2.1 关于页面添加抹掉所有内容和设置按钮
- [x] 在 `AboutView.vue` 中添加危险操作区域卡片
- [x] 添加红色「抹掉所有内容和设置」按钮
- [x] 添加简短说明文字
- **验证**：UI 预览

### 2.2 实现确认对话框
- [x] 使用 `ElMessageBox.confirm` 实现二次确认
- [x] 对话框标题：「抹掉所有内容和设置」
- [x] 对话框内容：列出将被删除的数据类型
- [x] 确认按钮文字：「确认重置」
- [x] 取消按钮文字：「取消」
- **验证**：交互测试

### 2.3 调用后端命令并重启
- [x] 调用 `factory_reset` 命令
- [x] 成功后显示提示信息
- [x] 使用 `relaunch()` 重启应用
- [x] 处理错误情况
- **验证**：完整流程测试

## 阶段 3：文档和规范

### 3.1 新增规范文件
- [x] 创建 `openspec/specs/factory-reset/spec.md`
- [x] 定义需求和场景
- **验证**：`openspec-cn validate`

## 依赖关系

```
1.1 ──┐
      ├──> 1.2 ──> 2.3
2.1 ──┘
      └──> 2.2 ──┘
```

- 任务 1.1 和 2.1 可并行
- 任务 1.2 依赖 1.1
- 任务 2.2 依赖 2.1
- 任务 2.3 依赖 1.2 和 2.2
- 任务 3.1 可独立进行
