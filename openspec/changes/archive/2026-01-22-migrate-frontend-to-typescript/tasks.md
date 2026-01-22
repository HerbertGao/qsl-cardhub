# 任务清单：迁移前端到 TypeScript

## Phase 1: 环境配置（2-3 小时）

### 1.1 安装 TypeScript 依赖
- [ ] 安装 `typescript`
- [ ] 安装 `vue-tsc`
- [ ] 安装 `@types/node`
- [ ] 更新 `package.json` 的 `devDependencies`

### 1.2 配置 TypeScript
- [ ] 创建 `tsconfig.json`
  - 配置 `compilerOptions`（target, module, lib, strict 等）
  - 配置 `include` 和 `exclude`
  - 配置 Vue 相关选项（`jsx`, `jsxImportSource`）
- [ ] 创建 `tsconfig.node.json`（用于 Vite 配置）

### 1.3 迁移构建配置
- [ ] 重命名 `vite.config.js` → `vite.config.ts`
- [ ] 添加类型导入和类型注解
- [ ] 验证构建配置正常工作

### 1.4 安装和配置 ESLint
- [x] 安装 ESLint 和相关插件
  - `eslint`
  - `@eslint/js`
  - `typescript-eslint`
  - `@typescript-eslint/eslint-plugin`
  - `@typescript-eslint/parser`
  - `eslint-plugin-vue`
- [x] 创建 `eslint.config.mjs` 配置文件
  - 配置 TypeScript 规则
  - 配置 Vue 规则
  - 配置忽略文件

### 1.5 更新构建脚本
- [x] 在 `package.json` 中添加 `type-check` 脚本
- [x] 在 `package.json` 中添加 `lint` 和 `lint:fix` 脚本
- [x] 更新 `build` 脚本，添加类型检查和 lint 检查
- [ ] 测试 `npm run type-check`、`npm run lint` 和 `npm run build`

---

## Phase 2: 类型定义（3-4 小时）

### 2.1 创建类型目录结构
- [ ] 创建 `src/types/` 目录
- [ ] 创建 `src/types/index.ts` 统一导出

### 2.2 定义 Tauri 命令类型
- [ ] 创建 `src/types/tauri.ts`
- [ ] 定义所有 Tauri 命令的参数和返回类型
  - 卡片管理命令（`create_card_cmd`, `list_cards_cmd` 等）
  - 项目管理命令（`create_project_cmd`, `list_projects_cmd` 等）
  - 配置管理命令（`get_profiles`, `create_profile` 等）
  - 打印命令（`preview_qsl`, `print_qsl` 等）
  - 安全命令（`save_credentials`, `qrz_save_and_login` 等）
- [ ] 定义 `invoke` 辅助函数类型

### 2.3 定义数据模型类型
- [ ] 创建 `src/types/models.ts`
- [ ] 定义核心数据结构
  - `Card`：卡片数据模型
  - `CardMetadata`：卡片元数据
  - `AddressHistory`：地址历史
  - `Project`：项目数据模型
  - `Profile`：配置文件数据模型
  - `TemplateConfig`：模板配置
  - `CardStatus`：卡片状态枚举
  - `CardFilter`：卡片过滤器
  - `Pagination`：分页参数
  - `PagedCards`：分页结果

### 2.4 定义组件类型
- [ ] 创建 `src/types/components.ts`
- [ ] 定义通用组件 Props 和 Emits 类型
- [ ] 定义表单数据类型
- [ ] 定义对话框数据类型

### 2.5 定义工具函数类型
- [ ] 创建 `src/types/utils.ts`
- [ ] 定义日期格式化函数类型
- [ ] 定义状态转换函数类型
- [ ] 定义验证函数类型

---

## Phase 3: 核心文件迁移（6-8 小时）

### 3.1 迁移入口文件
- [ ] 重命名 `src/main.js` → `src/main.ts`
- [ ] 添加类型导入
- [ ] 为 `createApp` 添加类型注解
- [ ] 验证应用启动正常

### 3.2 迁移 App 组件
- [ ] 修改 `src/App.vue`
  - 添加 `<script setup lang="ts">`
  - 为 reactive 变量添加类型
  - 为函数添加参数和返回类型
  - 导入类型定义
- [ ] 验证主应用正常渲染

### 3.3 迁移 Views 组件（7 个）
- [ ] `AboutView.vue`
  - 添加 `lang="ts"`
  - 定义 Props 类型（如有）
- [x] `CardManagementView.vue`
  - 添加 `lang="ts"`
  - 定义状态类型
  - 定义方法类型
- [x] `ConfigView.vue`
  - 添加 `lang="ts"`
  - 定义 Profile 相关类型
- [ ] `QRZConfigView.vue`
  - 添加 `lang="ts"`
  - 定义表单数据类型
  - 定义登录状态类型
- [ ] `CloudDatabaseConfigView.vue`
  - 添加 `lang="ts"`
  - 定义 Props 类型（如有）
- [x] `LogView.vue`
  - 添加 `lang="ts"`
  - 定义日志数据类型
- [x] `PrintView.vue`
  - 添加 `lang="ts"`
  - 定义打印参数类型
- [x] `TemplateView.vue`
  - 添加 `lang="ts"`
  - 定义模板配置类型

### 3.4 迁移 Cards 组件（6 个）
- [x] `CardDetailDialog.vue`
  - 添加 `lang="ts"`
  - 定义 Props 和 Emits 类型
- [x] `CardInputDialog.vue`
  - 添加 `lang="ts"`
  - 定义表单数据类型
- [x] `CardList.vue`
  - 添加 `lang="ts"`
  - 定义卡片列表类型
- [x] `CardListPlaceholder.vue`
  - 添加 `lang="ts"`
- [x] `DistributeDialog.vue`
  - 添加 `lang="ts"`
  - 定义分发表单类型
  - 定义地址历史类型
- [x] `ReturnDialog.vue`
  - 添加 `lang="ts"`
  - 定义退卡表单类型

### 3.5 迁移 Projects 组件（2 个）
- [x] `ProjectDialog.vue`
  - 添加 `lang="ts"`
  - 定义项目表单类型
- [x] `ProjectList.vue`
  - 添加 `lang="ts"`
  - 定义项目列表类型

### 3.6 迁移 Data Config 组件（1 个）
- [ ] `QRZConfig.vue`
  - 添加 `lang="ts"`
  - 定义登录表单类型
  - 定义状态类型

---

## Phase 4: 验证和优化（2-3 小时）

### 4.1 类型检查
- [x] 运行 `npm run type-check`
- [x] 修复所有类型错误
- [ ] 修复所有类型警告（仅剩 22 个非关键警告）
- [x] 确保类型覆盖率 ≥ 80%

### 4.2 功能测试
- [ ] 测试卡片管理功能
  - 创建卡片
  - 列表查询
  - 分发卡片（包括地址查询）
  - 退卡
  - 删除卡片
- [ ] 测试项目管理功能
- [ ] 测试打印配置功能
- [ ] 测试数据配置功能
  - QRZ.cn 登录
  - 地址查询
  - 凭据清除
- [ ] 测试打印功能
- [ ] 测试模板编辑功能
- [ ] 测试日志查看功能

### 4.3 构建验证
- [ ] 运行 `npm run build`
- [ ] 确保构建成功，无类型错误
- [ ] 检查输出文件大小（与迁移前对比）
- [ ] 在开发模式下测试热更新

### 4.4 代码质量优化
- [ ] 移除不必要的 `any` 类型
- [ ] 为公共函数添加 JSDoc 注释
- [ ] 统一类型命名规范
- [ ] 检查类型导入是否合理

---

## Phase 5: 文档和清理（1-2 小时）

### 5.1 更新文档
- [ ] 更新 `web/README.md`
  - 添加 TypeScript 说明
  - 更新构建命令
  - 添加类型检查说明
- [ ] 创建 TypeScript 开发指南（可选）
  - 类型定义规范
  - 常见模式示例
  - 最佳实践

### 5.2 清理工作
- [ ] 删除未使用的 `.js` 文件（如有备份）
- [ ] 更新 `.gitignore`（忽略 TypeScript 编译产物）
- [ ] 检查并清理未使用的类型定义

---

## 验证检查清单

### 构建验证
- [ ] `npm run type-check` 通过，无错误
- [ ] `npm run build` 成功
- [ ] 构建输出文件正常
- [ ] 文件大小变化在合理范围内

### 功能验证
- [ ] 所有页面正常渲染
- [ ] 所有 Tauri 命令调用正常
- [ ] 表单验证正常工作
- [ ] 对话框打开/关闭正常
- [ ] 数据加载和显示正常

### 开发体验验证
- [ ] IDE 类型提示正常
- [ ] 自动补全功能正常
- [ ] 跳转到定义功能正常
- [ ] 重构功能正常

### 类型质量验证
- [ ] 至少 80% 的代码有明确类型
- [ ] 核心数据模型有详细接口定义
- [ ] Tauri 命令有完整类型定义
- [ ] 组件 Props/Emits 有类型约束

---

## 时间估算总结

| Phase | 预估时间 | 关键产出 |
|-------|---------|---------|
| Phase 1 | 2-3 小时 | TypeScript 环境配置完成 |
| Phase 2 | 3-4 小时 | 核心类型定义完成 |
| Phase 3 | 6-8 小时 | 所有组件迁移完成 |
| Phase 4 | 2-3 小时 | 类型检查通过，功能验证完成 |
| Phase 5 | 1-2 小时 | 文档更新，清理完成 |
| **总计** | **14-20 小时** | **完整 TypeScript 迁移** |

---

## 依赖和风险

### 依赖
- Vite 版本 ≥ 5.0（已满足）
- Vue 版本 ≥ 3.4（已满足）
- Element Plus 版本 ≥ 2.5（已满足）

### 风险缓解
- **类型错误过多**：分阶段迁移，逐步修复
- **第三方库类型缺失**：使用 `declare module` 补充声明
- **迁移中断风险**：保持 Git 分支，随时可回滚
