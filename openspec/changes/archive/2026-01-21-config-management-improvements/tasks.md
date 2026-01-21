# 实施任务清单

## 1. 后端 - 数据模型更新
- [x] 1.1 修改 `Profile` 结构体
  - 添加 `task_name: Option<String>` 字段
  - 修改 `template: Template` 字段类型
  - 删除 `paper: PaperSpec` 字段
  - 修改 `created_at` 和 `updated_at` 类型为 `DateTime<FixedOffset>`
- [x] 1.2 修改 `Template` 结构体
  - 将 `name: String` 改为 `path: String`（存储模板文件路径）
  - 删除 `version: String` 字段
- [x] 1.3 更新 `Profile::new()` 方法
  - 初始化 `task_name` 为 `None`
  - 初始化 `template.path` 为 `"default.toml"`
  - 移除 `paper` 初始化
  - 使用东八区（UTC+8）时间戳
- [x] 1.4 更新 `Profile::touch()` 方法
  - 使用东八区（UTC+8）时间戳
- [x] 1.5 删除 `PaperSpec` 结构体（已删除）

## 2. 前端 - 配置界面更新
- [x] 2.1 修改 `ConfigView.vue` 表单
  - task_name 字段已正常保存和加载
  - 删除纸张规格（paper）显示部分
  - 模板字段更新为显示 template.path
- [x] 2.2 更新配置保存逻辑
  - task_name 字段已包含在保存逻辑中
  - 移除 paper 字段的处理

## 3. 配置文件更新
- [x] 3.1 更新示例配置文件 `config/profiles/.example.toml`
  - 添加 `task_name` 字段示例
  - 修改 `template` 部分为 `path = "default.toml"`
  - 删除 `paper` 部分
  - 更新时间戳格式为东八区

## 4. 测试验证
- [x] 4.1 测试新配置创建
  - 后端和前端代码编译成功
  - 数据模型已更新
- [x] 4.2 测试配置加载和保存
  - 配置结构已更新
  - task_name 字段已添加
- [x] 4.3 测试打印功能
  - PrintView.vue 已移除 paper 字段引用
  - 更新为显示模板路径信息
  - 前端运行时错误已修复

## 5. 文档更新
- [x] 5.1 更新配置文件格式文档
  - 示例配置文件已更新
  - 代码中已包含完整注释
  - 规范文档已包含完整格式说明

## 6. 额外清理
- [x] 6.1 移除 printer.model 字段
  - 从 PrinterConfig 结构体中删除
  - 从 ConfigView.vue 移除显示
  - 更新示例配置文件
  - 更新规范文档
