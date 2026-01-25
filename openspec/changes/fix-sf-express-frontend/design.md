# 设计文档：默认API参数预配置方案

## 背景

用户需要在顺丰速运配置界面预定义一组默认API参数，但不能硬编码在前端代码中，需要一个灵活的配置方案。

## 设计目标

1. **安全性**：敏感凭据不硬编码在前端源码中
2. **灵活性**：支持默认参数和自定义参数两种模式
3. **可维护性**：默认参数可通过配置文件更新，无需重新编译
4. **用户友好**：默认模式下提供风险提示，自定义模式下提供清晰的申请指引

## 技术方案

### 方案一：环境变量（不采用）

**优点：** 简单直接
**缺点：**
- 用户安装后无法更新
- 打包时需要处理环境变量
- 不适合桌面应用分发

### 方案二：配置文件（采用）

**优点：**
- 灵活可更新
- 与现有配置系统一致
- 支持本地覆盖

**实现方式：**

1. 创建独立的配置文件 `config/sf_express_default.toml` 存储默认参数
2. 后端提供读取默认参数的API
3. 前端根据配置决定UI显示

## 详细设计

### 配置文件结构

采用「模板 + 实际配置」分离的方式，避免敏感参数被提交到公开Git仓库。

#### 1. 模板文件（提交到Git）

`config/sf_express_default.toml.example`：

```toml
# config/sf_express_default.toml.example
# 顺丰速运默认预配置参数模板
# 复制此文件为 sf_express_default.toml 并填入实际参数
# 注意：sf_express_default.toml 已被 .gitignore 忽略

# 是否启用默认参数
enabled = true

# 顾客编码
partner_id = ""

# 沙箱环境校验码
checkword_sandbox = ""

# 生产环境校验码
checkword_prod = ""
```

#### 2. 实际配置文件（不提交到Git）

`config/sf_express_default.toml`（添加到 `.gitignore`）：

```toml
# 实际配置文件，包含真实的API参数
# 此文件不会被提交到Git仓库

enabled = true
partner_id = "实际的顾客编码"
checkword_sandbox = "实际的沙箱校验码"
checkword_prod = "实际的生产校验码"
```

#### 3. Git忽略配置

在 `.gitignore` 中添加：

```gitignore
# 顺丰速运默认API配置（包含敏感信息）
config/sf_express_default.toml
```

**文件位置说明：**
- `config/sf_express_default.toml.example` - 模板文件，提交到Git，供开发者参考
- `config/sf_express_default.toml` - 实际配置文件，被gitignore，由开发者本地创建
- 打包时：CI/CD 流程中注入实际参数，或从安全的构建环境读取

### 数据流

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  配置文件       │────▶│  后端 Rust      │────▶│  前端 Vue       │
│  default.toml   │     │  sf_commands    │     │  SFExpressConfig│
└─────────────────┘     └─────────────────┘     └─────────────────┘
                              │
                              ▼
                        ┌─────────────────┐
                        │  用户配置存储   │
                        │  (钥匙串/加密)  │
                        └─────────────────┘
```

### 后端API

新增 Tauri 命令：

```rust
#[tauri::command]
pub async fn sf_get_default_api_config() -> Result<SFDefaultApiConfig, String> {
    // 从独立配置文件 sf_express_default.toml 读取默认参数
    // 返回结构（敏感字段只返回是否存在，不返回实际值）
}

#[derive(Serialize)]
pub struct SFDefaultApiConfig {
    pub enabled: bool,
    pub has_partner_id: bool,
    pub has_checkword_sandbox: bool,
    pub has_checkword_prod: bool,
}
```

修改现有命令：

```rust
#[tauri::command]
pub async fn sf_save_config(
    environment: String,
    partner_id: Option<String>,  // 自定义模式才传
    checkword_prod: Option<String>,
    checkword_sandbox: Option<String>,
    use_default: bool,  // 新增：是否使用默认参数
) -> Result<(), String> {
    if use_default {
        // 从 sf_express_default.toml 读取默认参数并保存到用户配置
    } else {
        // 使用用户传入的自定义参数
    }
}
```

### 前端UI设计

#### 配置模式选择

```vue
<el-form-item label="参数来源">
  <el-radio-group v-model="configMode">
    <el-radio value="default" :disabled="!defaultConfigAvailable">
      使用默认参数
    </el-radio>
    <el-radio value="custom">
      使用自定义参数
      <el-tag type="success" size="small">推荐</el-tag>
    </el-radio>
  </el-radio-group>
</el-form-item>

<!-- 默认模式风险提示 -->
<el-alert
  v-if="configMode === 'default'"
  type="warning"
  :closable="false"
>
  <template #title>
    该参数不可滥用，有随时停用或更换的风险，请尽量使用自定义参数。
  </template>
</el-alert>

<!-- 自定义模式提示 -->
<el-alert
  v-if="configMode === 'custom'"
  type="info"
  :closable="false"
>
  <template #title>
    前往
    <el-link type="primary" href="https://open.sf-express.com/" target="_blank">
      顺丰开放平台
    </el-link>
    申请您自己的 API 凭据
  </template>
</el-alert>
```

#### 状态管理

```typescript
// 配置模式
const configMode = ref<'default' | 'custom'>('custom')

// 默认配置是否可用
const defaultConfigAvailable = ref(false)

// 加载默认配置状态
const loadDefaultApiConfig = async () => {
  try {
    const config = await invoke<SFDefaultApiConfig>('sf_get_default_api_config')
    defaultConfigAvailable.value = config.enabled && config.has_partner_id

    // 如果默认配置不可用，自动切换到自定义模式
    if (!defaultConfigAvailable.value) {
      configMode.value = 'custom'
    }
  } catch (error) {
    defaultConfigAvailable.value = false
    configMode.value = 'custom'
  }
}
```

### 安全考虑

1. **源码安全**
   - 实际配置文件 `sf_express_default.toml` 被 `.gitignore` 忽略
   - 敏感参数不会被提交到公开Git仓库
   - 只有模板文件 `.toml.example` 被提交

2. **前端不传输敏感数据**
   - 默认模式下，前端只告知后端"使用默认参数"
   - 实际凭据在后端读取和处理

3. **构建安全**
   - 打包/发布时通过CI/CD环境变量或安全存储注入实际参数
   - 开发者本地需要手动创建配置文件

4. **用户配置加密**
   - 用户的自定义参数仍然通过钥匙串或加密文件存储

5. **风险提示**
   - 选择默认参数时显示明确的风险警告
   - 引导用户优先使用自定义参数

### 向后兼容

1. 现有用户配置不受影响
2. 如果配置文件中没有默认参数，自动使用自定义模式
3. 用户可以随时在两种模式间切换

## 替代方案

### 远程配置服务（暂不采用）

将官方参数存储在远程服务器，应用启动时拉取。

**优点：** 参数更新不需要发布新版本
**缺点：**
- 需要额外服务器
- 增加网络依赖
- 安全性更复杂

可作为未来版本的增强功能。
