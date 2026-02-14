# 开发指南

本项目包含两个独立的子项目：

| 子项目 | 路径 | 说明 |
|-------|------|------|
| **桌面应用** | `/`（根目录）| Rust + Tauri 2 后端 + Vue 3 前端 |
| **云端查询服务** | `/web_query_service/` | Cloudflare Workers + D1 + Vue 3 前端 |

---

## 一、桌面应用

### 1.1 环境要求

| 依赖 | 最低版本 | 安装方式 |
|------|---------|---------|
| Rust | 1.85+ (Edition 2024) | [rustup.rs](https://rustup.rs/) |
| Node.js | 18+ | [nvm](https://github.com/nvm-sh/nvm) |
| pnpm | 9+ | `npm install -g pnpm` |
| Tauri CLI | 2.x | `cargo install tauri-cli` |

**平台特定依赖：**

- **macOS**：Xcode Command Line Tools（`xcode-select --install`）
- **Windows**：Visual Studio Build Tools（C++ 桌面开发工作负载）
- **Linux**：`libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`

### 1.2 快速开始

```bash
# 安装前端依赖
cd web && pnpm install && cd ..

# 生成顺丰占位配置（首次构建必需，CI 中由工作流自动生成）
cat > config/sf_express_default.toml << TOML
enabled = false
partner_id = ""
checkword_sandbox = ""
checkword_prod = ""
TOML

# 启动开发服务器
cargo tauri dev
```

`cargo tauri dev` 会自动启动前端 Vite 开发服务器（`http://localhost:5173`），Rust 后端和前端修改均支持热重载。

### 1.3 Rust 后端开发

#### 项目结构

```
src/
├── main.rs              # Tauri 应用入口
├── lib.rs               # 库入口
├── api.rs               # 公共 API（QslCardGenerator）
├── commands/            # Tauri Commands（前端调用的后端接口）
│   ├── cards.rs         # 卡片管理
│   ├── projects.rs      # 项目管理
│   ├── printer.rs       # 打印
│   ├── profile.rs       # Profile 配置
│   ├── tspl_config.rs   # TSPL 配置
│   ├── sf_express.rs    # 顺丰快递
│   ├── qrz_cn.rs        # QRZ.cn
│   ├── qrz_com.rs       # QRZ.com
│   ├── qrz_herbertgao.rs # QRZ.herbertgao.me
│   ├── security.rs      # 凭据管理
│   ├── data_transfer.rs # 数据导入导出
│   ├── export.rs        # Excel 导出
│   ├── factory_reset.rs # 恢复出厂设置
│   ├── sync.rs          # 云端同步
│   ├── logger.rs        # 日志
│   └── platform.rs      # 平台信息
├── db/                  # SQLite 数据库层
│   ├── sqlite.rs        # 连接管理与版本迁移
│   ├── models.rs        # 数据模型
│   ├── projects.rs      # 项目 CRUD
│   ├── cards.rs         # 卡片 CRUD + 地址缓存
│   ├── sf_express.rs    # 顺丰订单/寄件人
│   ├── export.rs / import.rs  # 数据导出导入
├── printer/             # 打印服务
│   ├── tspl.rs          # TSPL 指令生成
│   ├── template_engine.rs   # 模板引擎
│   ├── layout_engine.rs     # 布局引擎
│   ├── render_pipeline.rs   # 渲染管线
│   ├── text_renderer.rs     # 文本渲染（中英文混合）
│   ├── barcode_renderer.rs  # 条形码渲染（Code128）
│   ├── font_loader.rs       # 字体加载
│   └── backend/         # 打印后端抽象
│       ├── windows.rs   # Win32 Print API
│       ├── cups.rs      # macOS CUPS
│       └── pdf.rs       # PDF 虚拟打印（预览）
├── config/              # 配置管理
│   ├── profile_manager.rs   # Profile 管理
│   ├── template.rs      # 模板配置（TOML）
│   └── models.rs        # 配置数据模型
├── qrz/                 # QRZ 集成
├── sf_express/          # 顺丰快递集成
├── security/            # 凭据加密存储（AES-256-GCM + PBKDF2）
├── sync/                # 云端数据同步
├── logger/              # 日志系统（环形缓冲区）
└── error/               # 错误类型
```

#### 常用命令

```bash
cargo tauri dev          # 启动开发服务器
cargo test               # 运行测试
cargo clippy             # 代码检查
cargo fmt                # 格式化
```

#### 代码规范

- 使用 `rustfmt` 格式化，`clippy` 检查
- 命名：`snake_case`（函数/变量）、`PascalCase`（类型）、`SCREAMING_SNAKE_CASE`（常量）
- 错误处理：`Result<T, E>` + `anyhow`/`thiserror`
- 注释使用中文，`///` 文档注释，仅在复杂逻辑处添加行注释

#### 数据库迁移

SQLite 数据库路径：`~/.config/qsl-cardhub/cards.db`（macOS）/ `%APPDATA%\qsl-cardhub\cards.db`（Windows）。

数据库版本管理由 `src/db/sqlite.rs` 负责。修改表结构时：
1. 在迁移函数中添加新的 SQL 语句
2. 递增数据库版本号
3. 应用启动时自动检测并执行迁移

#### Tauri Commands 参数约定

Tauri 2 自动将 Rust `snake_case` 参数转换为前端 `camelCase`：

```rust
// Rust 后端：snake_case
#[tauri::command]
pub async fn distribute_card_cmd(
    id: String,
    proxy_callsign: Option<String>,
) -> Result<Card, String> { ... }
```

```typescript
// 前端调用：camelCase（Tauri 自动转换）
await invoke('distribute_card_cmd', {
  id: data.id,
  proxyCallsign: data.proxy_callsign || null
})
```

无需在 Rust 代码中添加 `rename_all` 配置。

### 1.4 桌面前端开发（`web/`）

#### 项目结构

```
web/src/
├── App.vue              # 根组件
├── main.ts              # 应用入口
├── views/               # 页面视图
│   ├── CardManagementView.vue   # 卡片管理（主页面）
│   ├── ConfigView.vue           # 配置管理
│   ├── TemplateView.vue         # 模板配置
│   ├── DataTransferView.vue     # 数据导入导出
│   ├── SFExpressConfigView.vue  # 顺丰配置
│   ├── SFOrderListView.vue      # 顺丰订单列表
│   ├── QRZConfigView.vue        # QRZ.cn 配置
│   ├── QRZComConfigView.vue     # QRZ.com 配置
│   ├── LogView.vue              # 日志查看
│   └── AboutView.vue            # 关于（含更新检查）
├── components/          # UI 组件
│   ├── cards/           # 卡片相关（录入、列表、详情、分发、退卡）
│   ├── projects/        # 项目管理
│   ├── sf-express/      # 顺丰快递
│   └── common/          # 通用组件（GlobalLoading 等）
├── types/               # TypeScript 类型
│   ├── tauri.ts         # Tauri 命令参数类型（snake_case，与 Rust 一致）
│   ├── models.ts        # 前端数据模型
│   ├── components.ts    # 组件类型
│   └── generated/       # ts-rs 自动生成的类型（勿手动修改）
├── stores/              # 状态管理（loadingStore, updateStore, navigationStore）
├── composables/         # 组合式函数（useLoading, useQtyDisplayMode）
├── services/            # 服务层（updateCheck）
└── utils/               # 工具函数（format, logger, markdown）
```

#### 技术栈

- **Vue 3** + Composition API（`<script setup>`）
- **Element Plus** UI 组件库
- **TypeScript** 严格模式
- **Vite 5** 构建工具
- **unplugin-icons** 图标（支持自定义 SVG 图标集 `web/src/assets/icons/`）
- **markdown-it + dompurify** Markdown 渲染

#### 常用命令

```bash
cd web
pnpm run dev             # 独立启动前端（不启动 Rust 后端）
pnpm run build           # 生产构建（含 vue-tsc 类型检查 + eslint）
pnpm run type-check      # TypeScript 类型检查
pnpm run lint            # ESLint 检查
pnpm run lint:fix        # 自动修复 Lint 问题
```

#### TypeScript 类型生成

项目使用 [ts-rs](https://github.com/Aleph-Alpha/ts-rs) 从 Rust 结构体自动生成 TypeScript 类型，输出到 `web/src/types/generated/`。

```bash
# 在项目根目录执行
cargo test export_bindings --features ts-rs
```

修改 Rust 数据模型（`src/db/models.rs`、`src/config/models.rs` 等）后必须重新生成并提交更新的类型文件。CI 会校验 `web/src/types/generated/` 是否与 Rust 定义同步。

#### 代码规范

- `camelCase`（函数/变量）、`PascalCase`（组件名、类型名）、`UPPER_SNAKE_CASE`（常量）
- ESLint 规则由根目录 `eslint.config.*` 定义
- 路径别名：`@` 指向 `web/src/`

### 1.5 构建与发布

#### 生产构建

```bash
cargo tauri build                # 直接构建

# 或使用构建脚本（含依赖检查、版本验证、产物整理）
./scripts/build.sh               # macOS/Linux
.\scripts\build.ps1              # Windows PowerShell
.\scripts\build.bat              # Windows CMD
```

构建产物：
- macOS: `target/release/bundle/dmg/*.dmg`
- Windows: `target/release/bundle/nsis/*.exe`、`target/release/bundle/msi/*.msi`

#### 版本管理

版本号同时维护在 `Cargo.toml`、`tauri.conf.json`、`web/package.json` 三个文件中：

```bash
./scripts/version.sh             # 查看当前版本
./scripts/version.sh patch       # 补丁升级 (0.6.12 → 0.6.13)
./scripts/version.sh minor       # 次版本升级 (0.6.12 → 0.7.0)
./scripts/version.sh major       # 主版本升级 (0.6.12 → 1.0.0)
./scripts/version.sh 1.0.0       # 设置指定版本
./scripts/version.sh check       # 检查版本一致性
./scripts/version.sh sync        # 从 Cargo.toml 同步到其他文件
```

#### 发布流程

```bash
# 一键发布（推荐）
./scripts/release.sh patch       # 自动：升级版本 → cargo check → 提交 → 打 Tag → 推送

# 手动发布
./scripts/version.sh patch
git add Cargo.toml tauri.conf.json web/package.json
git commit -m "chore: bump version to x.y.z"
git tag vx.y.z
git push origin master --tags
```

推送 `v*` Tag 后，GitHub Actions 自动触发多平台构建并创建 Release。

### 1.6 CI/CD

#### PR 构建（`build.yml`）

向 `master` 提交 PR 时自动触发：
1. **前端质量检查**：TypeScript 类型检查 + ESLint
2. **类型同步校验**：`cargo test export_bindings --features ts-rs` 后检查 `web/src/types/generated/` 是否有 diff
3. **多平台构建验证**

#### 发布构建（`release.yml`）

推送 `v*` Tag 时触发：
1. 代码质量检查
2. 多平台并行构建（macOS x64/ARM64、Windows x64/ARM64）
3. 创建 GitHub Release 并上传安装包
4. 上传到阿里云 OSS（CDN 加速更新下载）

---

## 二、云端查询服务（`web_query_service/`）

独立的 Cloudflare Workers 服务，提供云端数据同步、按呼号查询和顺丰路由推送接收功能。

### 2.1 环境要求

| 依赖 | 版本 | 安装方式 |
|------|------|---------|
| Node.js | 18+ | [nvm](https://github.com/nvm-sh/nvm) |
| pnpm | 9+ | `npm install -g pnpm` |
| Wrangler | 4+ | 项目 devDependency，无需全局安装 |

### 2.2 项目结构

```
web_query_service/
├── src/
│   ├── worker/
│   │   └── index.js           # Cloudflare Worker 入口（API 路由）
│   └── client/                # 查询页面前端（Vue 3）
│       ├── App.vue            # 根组件
│       ├── main.ts            # 入口
│       ├── style.css          # 样式
│       ├── components/
│       │   ├── SearchBox.vue      # 呼号搜索
│       │   ├── ResultList.vue     # 查询结果
│       │   ├── SubscribeCard.vue  # 订阅收卡
│       │   └── MathCaptcha.vue    # 数学验证码
│       └── utils/
│           └── sign.ts        # 请求签名
├── schema.sql                 # D1 数据库表结构
├── wrangler.toml.example      # Wrangler 配置示例
├── package.json
├── vite.config.ts
└── tsconfig.json
```

### 2.3 快速开始

```bash
cd web_query_service
pnpm install

# 复制并编辑 Wrangler 配置
cp wrangler.toml.example wrangler.toml
# 编辑 wrangler.toml，填入 D1 database_id 和 KV namespace id

# 创建 D1 数据库
npx wrangler d1 create qsl-sync
# 将返回的 database_id 填入 wrangler.toml

# 创建 KV 命名空间（限流用）
npx wrangler kv namespace create "RATE_LIMIT"
# 将返回的 id 填入 wrangler.toml

# 执行数据库迁移（本地）
pnpm run db:migrate:local

# 配置 API Key
npx wrangler secret put API_KEY

# 启动开发服务器
pnpm run dev
```

`pnpm run dev` 同时启动 Worker（`http://localhost:8787`）和前端 Vite 开发服务器（`http://localhost:5173`，API 请求代理到 Worker）。

### 2.4 API 端点

| 方法 | 路径 | 认证 | 说明 |
|------|------|------|------|
| GET | `/ping` | Bearer Token | 连接测试 |
| POST | `/sync` | Bearer Token | 全量数据同步（桌面端推送） |
| GET | `/api/callsigns/:callsign` | 无 | 按呼号查询收卡 |
| GET | `/api/query?callsign=` | 无 | 按呼号查询（查询参数） |
| POST | `/api/sf/route-push` | 无 | 顺丰路由推送（正式环境） |
| POST | `/api/sf/route-push/sandbox` | 无 | 顺丰路由推送（沙箱环境） |
| GET | `/query` | 无 | 呼号查询页面（Vue 前端） |

### 2.5 常用命令

```bash
cd web_query_service

pnpm run dev                 # 同时启动 Worker + 前端
pnpm run dev:worker          # 仅启动 Worker
pnpm run dev:client          # 仅启动前端
pnpm run build               # 构建前端（vue-tsc + vite build → public/）
pnpm run deploy              # 构建 + 部署到 Cloudflare Workers

# 数据库操作
pnpm run db:create           # 创建 D1 数据库
pnpm run db:migrate          # 远程数据库迁移
pnpm run db:migrate:local    # 本地数据库迁移
```

### 2.6 D1 数据库表结构

| 表 | 说明 |
|----|------|
| `sync_meta` | 同步元数据（client_id、同步时间） |
| `projects` | 项目（按 client_id 隔离） |
| `cards` | 卡片（按 client_id 隔离） |
| `sf_senders` | 顺丰寄件人 |
| `sf_orders` | 顺丰订单（路由推送时关联呼号） |
| `callsign_openid_bindings` | 呼号与微信 openid 绑定 |
| `sf_route_log` | 顺丰路由推送日志（去重） |

修改表结构时编辑 `schema.sql`，然后执行迁移命令。

### 2.7 环境变量与密钥

在 `wrangler.toml` 的 `[vars]` 中配置非敏感变量，通过 `wrangler secret put` 配置敏感信息：

| 变量 | 方式 | 必需 | 说明 |
|------|------|------|------|
| `API_KEY` | secret | 是 | `/ping`、`/sync` 的 Bearer Token |
| `CLIENT_SIGN_KEY` | vars | 否 | 前端请求签名密钥 |
| `CAPTCHA_SECRET` | secret | 否 | 验证码 token 签名密钥 |
| `WECHAT_APPID` | secret | 否 | 微信服务号 AppID |
| `WECHAT_SECRET` | secret | 否 | 微信服务号 AppSecret |
| `WECHAT_TEMPLATE_ID` | secret | 否 | 微信模板消息 ID |
| `SITE_FILING` | vars | 否 | 网站备案信息（JSON） |

### 2.8 部署

```bash
cd web_query_service
pnpm run deploy              # 构建前端 + 部署 Worker
```

首次部署前确保：
1. `wrangler.toml` 中 D1 和 KV 的 ID 已填写
2. 远程数据库已执行迁移（`pnpm run db:migrate`）
3. API Key 已通过 `wrangler secret put` 配置

---

## 三、通用约定

### Git 工作流

- **分支**：`master`（主）、`develop`（开发）、`feature/*`、`fix/*`
- **提交格式**：`类型: 中文描述`
  - `feat`: 新功能、`fix`: 修复、`docs`: 文档、`refactor`: 重构、`test`: 测试、`chore`: 构建/工具

### OpenSpec 变更管理

项目使用 OpenSpec 管理功能变更。规范文件位于 `openspec/` 目录：
- `openspec/config.yaml`：项目上下文
- `openspec/specs/`：已归档的功能规范
- `openspec/changes/`：变更工作流（提案 → 设计 → 规范 → 任务 → 归档）

### 相关文档

- [阿里云 CDN 配置指南](docs/aliyun-cdn-setup.md) — 更新下载加速配置
- [云端同步 API 规范](docs/cloud-sync-api-spec.md) — 同步接口请求/响应格式
- [云端查询服务部署](docs/web-query-service-deploy.md) — Cloudflare Workers 部署概要
- [构建脚本说明](scripts/README.md) — 版本管理、构建、发布脚本详细用法
