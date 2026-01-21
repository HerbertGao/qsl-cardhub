# 变更：配置管理改进

## 为什么

当前配置管理系统存在以下问题：

1. **task_name 字段未持久化**：用户在配置管理页面可以设置"任务名称"字段，该字段在打印时被使用（作为模板数据传入），但该字段没有保存到配置文件中，导致每次重启应用后丢失。

2. **template 字段使用硬编码值**：配置文件中的 `template` 字段存储的是硬编码的名称和版本（`name: "QSL Card v1"`, `version: "1.0"`），而不是实际的模板文件路径。这导致无法追踪配置使用的是哪个具体模板文件。

3. **paper 字段冗余**：`paper` 字段（纸张规格）在配置中是固定值（76mm × 130mm），与模板文件中的 `page.width_mm` 和 `page.height_mm` 重复。由于模板已经定义了纸张尺寸，配置文件中的 `paper` 字段变得冗余。

4. **printer.model 字段冗余**：`printer.model` 字段在所有配置中都是硬编码的 `"Deli DL-888C"`，没有实际意义。项目目前只支持一种打印机型号，这是项目级别的常量，不需要在每个配置中存储。

## 变更内容

### 1. 添加 task_name 字段到配置数据模型
- 在 `Profile` 结构体中添加 `task_name: Option<String>` 字段
- 在前端配置管理界面保存时，将 `task_name` 写入配置文件
- 在加载配置时，读取 `task_name` 字段

### 2. 将 template 字段改为存储模板文件路径
- 将 `template.name` 改为存储实际模板文件路径（相对于 `config/templates/`）
- 默认值从 `"QSL Card v1"` 改为 `"default.toml"`
- 移除 `template.version` 字段（版本信息在模板文件内部）

### 3. 移除 paper 字段
- 从 `Profile` 结构体中删除 `paper: PaperSpec` 字段
- 从前端配置界面移除纸张规格显示
- 纸张尺寸完全由模板文件定义

### 4. 移除 printer.model 字段
- 从 `PrinterConfig` 结构体中删除 `model: String` 字段
- 从前端配置界面移除打印机型号显示
- 只保留 `printer.name`（系统打印机名称）

## 影响

### 受影响规范
- 需要新增规范：`configuration-management`（配置管理）

### 受影响代码
- **修改**: `src/config/models.rs` - 更新 `Profile` 和 `Template` 结构
- **修改**: `src/config/profile_manager.rs` - 更新配置读写逻辑
- **修改**: `web/src/views/ConfigView.vue` - 更新配置表单，移除 paper 显示
- **修改**: `config/profiles/.example.toml` - 更新示例配置文件格式

### 向后兼容性
- **不完全兼容**：需要迁移现有配置文件
- **迁移策略**：
  1. 读取旧格式配置时，自动添加默认 `task_name: None`
  2. 读取旧格式 `template.name` 时，转换为 `"default.toml"`
  3. 忽略旧格式的 `paper` 字段
  4. 保存时使用新格式

### 用户体验
- **正面影响**：task_name 可以持久化，用户无需每次重新输入
- **正面影响**：配置更简洁，移除冗余字段
- **正面影响**：template 字段更准确，可追踪实际使用的模板文件
