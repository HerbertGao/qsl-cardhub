# 配置管理规范（Rust 版）

## 目的

本规范定义了 qsl-cardhub Rust 版本的配置管理功能需求。配置管理使用 Rust 的类型系统和 serde 进行数据序列化，提供类型安全的配置操作。本规范在 Python 版的基础上保持功能一致性，同时利用 Rust 的优势提升性能和可靠性。

## 修改需求

### 需求:配置文件（Profile）管理

**原需求**：系统必须支持多配置文件管理，允许用户为不同打印场景创建和管理独立的配置。（Python 版）

**修改原因**：迁移到 Rust + Tauri 架构，需要使用 Rust 数据结构和 serde 序列化。

**新需求**：系统必须使用 Rust struct 和 serde 实现类型安全的配置管理，保持与 Python 版的功能一致性。

#### 场景:创建配置文件

- **当** 用户创建新配置文件
- **那么** 系统应生成唯一的 UUID（使用 uuid crate）
- **并且** 配置应使用 `Profile` struct 表示
- **并且** 应包含平台信息、打印机设置、纸张规格、模板信息
- **并且** 系统应记录创建时间和更新时间（使用 chrono crate）
- **并且** 配置应序列化为 JSON 并持久化到 profiles.json

#### 场景:查看配置列表

- **当** 用户查看配置列表
- **那么** 系统应从 ProfileManager 获取所有配置
- **并且** 返回 `Vec<Profile>` 类型
- **并且** 默认配置应通过 default_profile_id 标识
- **并且** 每个配置应包含完整的元数据

#### 场景:编辑配置

- **当** 用户修改配置的打印机名称或其他设置
- **并且** 保存更改
- **那么** 系统应验证输入数据类型
- **并且** 更新 Profile struct 实例
- **并且** 更新 updated_at 字段（使用 `Utc::now()`）
- **并且** 持久化更改到 profiles.json

#### 场景:删除配置

- **当** 用户删除一个配置文件
- **那么** 系统应从 ProfileStore.profiles 中移除该配置
- **并且** 如果删除的是最后一个配置，应返回错误"至少保留一个配置"
- **并且** 如果删除的是默认配置，应自动选择第一个配置作为默认
- **并且** 持久化更改

#### 场景:设置默认配置

- **当** 用户将某个配置设为默认
- **那么** 系统应更新 ProfileStore.default_profile_id
- **并且** 验证 ID 存在于配置列表中
- **并且** 持久化更改

### 需求:配置文件结构

**原需求**：系统必须使用 JSON 格式存储配置，并包含完整的元数据。（Python 版）

**修改原因**：使用 TOML 格式替代 JSON，提升可读性和配置隔离性。

**新需求**：系统必须使用 Rust struct 定义配置数据模型，通过 serde 序列化为 TOML 格式。

#### 场景:配置数据模型

- **当** 系统定义配置数据模型
- **那么** 应使用以下 Rust structs：
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Profile {
      pub id: String,
      pub name: String,
      pub platform: Platform,
      pub printer: PrinterConfig,
      pub paper: PaperSpec,
      pub template: Template,
      pub created_at: DateTime<Utc>,
      pub updated_at: DateTime<Utc>,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Platform {
      pub os: String,
      pub arch: String,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct PrinterConfig {
      pub model: String,
      pub name: String,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct PaperSpec {
      pub width: u32,
      pub height: u32,
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Template {
      pub name: String,
      pub version: String,
  }

  #[derive(Debug, Serialize, Deserialize)]
  pub struct AppConfig {
      pub default_profile_id: Option<String>,
      pub window_state: Option<WindowState>,
  }
  ```
- **并且** 所有 struct 应派生 `Serialize` 和 `Deserialize` trait
- **并且** TOML 字段名应使用 snake_case 风格

#### 场景:配置验证

- **当** 系统加载配置文件
- **那么** serde 应自动验证字段类型
- **并且** 如果缺少必填字段，应返回 `toml::de::Error`
- **并且** 如果类型不匹配，应返回详细错误信息
- **并且** 系统应捕获错误并返回友好提示

### 需求:配置导入导出

**原需求**：系统必须支持配置文件的导入和导出，便于备份和迁移。（Python 版）

**新需求**：使用 Rust 实现配置导入导出，使用 TOML 格式。

#### 场景:导出配置

- **当** 用户导出一个配置文件
- **那么** 系统应使用 `toml::to_string_pretty()` 格式化 TOML
- **并且** 文件名应包含配置名称（如 "默认配置.toml"）
- **并且** TOML 应使用 UTF-8 编码
- **并且** 应包含注释说明各字段含义

#### 场景:导入配置

- **当** 用户导入一个配置 TOML 文件
- **那么** 系统应使用 `toml::from_str()` 解析 TOML
- **并且** 如果解析失败，应返回错误"配置文件格式错误"
- **并且** 如果配置名称已存在，应自动重命名（添加数字后缀）
- **并且** 应重新生成 UUID 和时间戳
- **并且** 导入成功后保存到 profiles/{uuid}.toml

### 需求:配置持久化

**原需求**：系统必须将配置持久化到本地文件系统。（Python 版）

**修改原因**：使用 TOML 格式，每个配置独立存储为单独的文件。

**新需求**：系统必须将每个配置保存为独立的 TOML 文件，全局配置保存为 config.toml。

#### 场景:保存配置到文件

- **当** 配置发生变更（创建、更新）
- **那么** 系统应使用 `toml::to_string_pretty()` 序列化 Profile
- **并且** 应使用 `std::fs::write()` 写入 `profiles/{id}.toml`
- **并且** 文件路径应使用 Tauri 的 `app_data_dir()` 获取
- **并且** TOML 应使用 UTF-8 编码
- **并且** 如果写入失败，应返回错误"配置保存失败"

#### 场景:删除配置文件

- **当** 配置被删除
- **那么** 系统应使用 `std::fs::remove_file()` 删除 `profiles/{id}.toml`
- **并且** 如果文件不存在，应忽略错误
- **并且** 应更新全局配置（如果删除的是默认配置）

#### 场景:加载配置列表

- **当** 系统启动时
- **那么** 应使用 `std::fs::read_dir()` 扫描 `profiles/` 目录
- **并且** 应读取所有 `.toml` 文件
- **并且** 应使用 `toml::from_str()` 反序列化为 Profile
- **并且** 如果目录不存在，应创建空目录
- **并且** 如果某个文件损坏，应跳过并记录警告

#### 场景:加载全局配置

- **当** 系统启动时
- **那么** 应读取 `config.toml`
- **并且** 应使用 `toml::from_str()` 反序列化为 AppConfig
- **并且** 如果文件不存在，应创建默认配置

#### 场景:文件位置

- **当** 系统访问配置文件
- **那么** 文件路径应为：
  - Windows: `%APPDATA%/qsl-cardhub/`
  - macOS: `~/Library/Application Support/qsl-cardhub/`
  - Linux: `~/.config/qsl-cardhub/`
- **并且** 目录结构应为：
  ```
  qsl-cardhub/
  ├── config.toml
  └── profiles/
      ├── uuid-1.toml
      └── uuid-2.toml
  ```
- **并且** 应使用 `tauri::api::path::app_data_dir()` 获取路径
- **并且** 如果目录不存在，应自动创建

## 新增需求

### 需求:ProfileManager 实现

系统必须提供 ProfileManager struct 管理所有配置操作。

#### 场景:初始化 ProfileManager

- **当** 系统启动时初始化 ProfileManager
- **那么** 应创建 ProfileManager 实例
- **并且** 应加载 profiles.json
- **并且** 如果加载失败，应创建空的 ProfileStore
- **并且** 应验证所有配置的数据完整性

#### 场景:线程安全访问

- **当** 多个 Tauri Command 并发访问 ProfileManager
- **那么** 应使用 `Arc<Mutex<ProfileManager>>` 包装
- **并且** 所有访问应获取锁
- **并且** 操作完成后应释放锁
- **并且** 避免死锁

#### 场景:错误处理

- **当** ProfileManager 操作失败
- **那么** 应返回 `Result<T, anyhow::Error>`
- **并且** 错误应包含详细的上下文信息
- **并且** Tauri Command 应将错误转换为 String 返回给前端

### 需求:配置格式说明

系统必须提供清晰的配置文件格式文档。

#### 场景:TOML 注释

- **当** 系统创建新配置文件
- **那么** 应在 TOML 文件顶部添加注释：
  ```toml
  # qsl-cardhub 打印配置
  # 创建时间: 2026-01-20
  ```
- **并且** 关键字段应有行内注释说明

#### 场景:配置示例

- **当** 用户需要手动编辑配置
- **那么** 应在文档中提供完整的 TOML 示例
- **并且** 说明每个字段的含义和有效值范围

#### 场景:Python 版本不兼容

- **当** 用户从 Python 版升级
- **那么** 应在文档中明确说明配置文件不兼容
- **并且** 说明需要在 Rust 版中重新创建配置
- **并且** 不提供自动迁移工具（简化实现）
