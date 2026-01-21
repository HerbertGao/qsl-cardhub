# 配置管理改进 - 技术设计

## 上下文

当前配置管理系统基于 TOML 文件存储，每个配置（Profile）保存为独立的文件。用户可以通过前端界面创建、修改、删除配置。配置包含打印机信息、平台信息、模板选择等。

存在的问题：
1. `task_name` 字段只在前端内存中，未持久化
2. `template` 字段存储的是描述性名称，而非实际路径
3. `paper` 字段与模板文件中的纸张尺寸重复

## 目标 / 非目标

### 目标
- 持久化 `task_name` 字段，避免用户每次重启后重新输入
- 使 `template` 字段存储实际模板文件路径，便于追踪和切换
- 移除冗余的 `paper` 字段，简化配置结构
- 提供向后兼容的迁移机制

### 非目标
- **不支持**多模板选择界面（v1 暂时固定使用 default.toml）
- **不支持**自定义纸张尺寸（完全由模板定义）
- **不支持**批量迁移工具（自动迁移在加载时进行）

## 决策

### 决策 1：task_name 使用 Option<String> 类型

**理由：**
- task_name 是可选字段，用户可以不填
- 使用 `Option<String>` 明确表达"可能为空"的语义
- 便于区分"未设置"（None）和"空字符串"（Some("")）

**实现：**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    // ... 其他字段

    /// 任务名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_name: Option<String>,
}
```

**前端处理：**
```javascript
// 保存时
const profileData = {
  ...selectedConfig.value,
  task_name: selectedConfig.value.task_name || null
}

// 加载时
selectedConfig.value.task_name = config.task_name || ''
```

### 决策 2：template.path 存储相对路径

**理由：**
- 存储相对路径（相对于 `config/templates/`）而非绝对路径
- 便于跨平台迁移配置文件
- 默认值为 `"default.toml"`，与当前实际模板文件名一致

**实现：**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// 模板文件路径（相对于 config/templates/）
    pub path: String,
}

impl Profile {
    pub fn new(name: String, printer_name: String, platform: Platform) -> Self {
        // ...
        template: Template {
            path: "default.toml".to_string(),
        },
        // ...
    }
}
```

**加载模板时的路径解析：**
```rust
// 在 commands/printer.rs 或模板加载逻辑中
fn get_template_path_from_profile(profile: &Profile) -> PathBuf {
    let config_dir = get_config_dir();
    config_dir.join("templates").join(&profile.template.path)
}
```

### 决策 3：删除 paper 字段，完全依赖模板定义

**理由：**
- 纸张尺寸已在模板文件中定义（`page.width_mm`, `page.height_mm`）
- 配置文件中的 `paper` 字段与模板重复，且可能不一致
- 简化配置结构，减少维护成本

**实现：**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub platform: Platform,
    pub printer: PrinterConfig,
    // 删除: pub paper: PaperSpec,
    pub template: Template,
    pub task_name: Option<String>,  // 新增
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**前端调整：**
- 从 ConfigView.vue 中移除纸张规格显示部分
- 如需显示纸张信息，从模板文件读取

### 决策 4：时间戳使用东八区（UTC+8）

**理由：**
- 项目主要面向中国用户
- 使用本地时区更符合用户习惯
- 便于阅读配置文件中的时间戳

**实现：**
```rust
use chrono::{DateTime, FixedOffset};

// 东八区偏移量
const UTC_OFFSET: i32 = 8 * 3600;

impl Profile {
    pub fn new(name: String, printer_name: String, platform: Platform) -> Self {
        let tz = FixedOffset::east_opt(UTC_OFFSET).unwrap();
        let now = Utc::now().with_timezone(&tz);

        Self {
            // ...
            created_at: now,
            updated_at: now,
        }
    }

    pub fn touch(&mut self) {
        let tz = FixedOffset::east_opt(UTC_OFFSET).unwrap();
        self.updated_at = Utc::now().with_timezone(&tz);
    }
}
```

**配置文件格式：**
```toml
created_at = "2026-01-21T12:00:00+08:00"
updated_at = "2026-01-21T12:00:00+08:00"
```

### 决策 5：保持示例配置文件最新

**理由：**
- 示例配置是用户参考的模板
- 必须与代码实现的格式一致
- 便于新用户理解配置结构

**新格式示例：**
```toml
id = "example-profile-id"
name = "示例配置"
task_name = "CRSA 2区卡片"  # 可选字段
created_at = "2026-01-20T12:00:00Z"
updated_at = "2026-01-20T12:00:00Z"

[platform]
os = "macOS"
arch = "arm64"

[printer]
model = "Deli DL-888C"
name = "Deli DL-888C"

[template]
path = "default.toml"
```

## 风险 / 权衡

### 风险 1：template.path 路径不存在

**风险**：配置文件中的 `template.path` 指向不存在的模板文件

**缓解措施：**
- 在加载模板时检查文件是否存在
- 如果不存在，回退到 `default.toml`
- 记录警告日志

### 权衡 1：task_name 的默认值

**选择**：使用 `Option<String>`，未设置时为 `None`

**权衡**：
- ✅ 明确表达"可选"语义
- ✅ 序列化时可跳过 None 值，配置文件更简洁
- ❌ 前端需要处理 null 到空字符串的转换

### 权衡 2：是否在前端显示模板路径

**选择**：v1 不显示，固定使用 default.toml

**权衡**：
- ✅ 简化界面，用户无需关心模板路径
- ✅ 后续版本可扩展为模板选择下拉框
- ❌ 高级用户无法通过界面切换模板

## 实施计划

### 阶段 1：代码更新
1. 修改 `Profile` 和 `Template` 结构体
2. 更新 `Profile::new()` 方法
3. 删除 `PaperSpec` 结构体（如果未在其他地方使用）

### 阶段 2：前端更新
1. 修改 ConfigView.vue，移除 paper 显示
2. 确保 task_name 正常保存和加载

### 阶段 3：配置文件更新
1. 更新 `.example.toml` 为新格式
2. 手动更新现有配置文件（如有）

### 阶段 4：测试验证
1. 创建新配置，验证新格式
2. 验证 task_name 持久化
3. 验证打印时正确使用 template.path

## 待决问题

1. **是否需要在前端显示当前使用的模板名称？**
   - 建议：v1 不显示，v2 添加模板选择功能时再显示

2. **是否需要提供批量迁移工具？**
   - 建议：不需要，自动迁移已足够

3. **task_name 是否需要长度限制？**
   - 建议：保持现有的 50 字符限制
