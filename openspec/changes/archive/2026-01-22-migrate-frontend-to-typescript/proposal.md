# 提案：将前端迁移到 TypeScript

## 概述

将 `/web` 前端项目从 JavaScript 迁移到 TypeScript，以提供更好的类型安全、开发体验和代码质量。

## 动机

### 当前状态
- 前端使用纯 JavaScript（`.js` 和 `.vue` 文件）
- 缺乏类型检查，容易出现运行时错误
- IDE 提示有限，开发效率受影响
- 与 Tauri 后端 API 交互时缺少类型定义

### 问题
1. **类型安全缺失**：调用 Tauri 命令时参数类型无法验证
2. **重构困难**：修改数据结构时容易遗漏相关代码
3. **开发体验差**：缺少自动补全和类型提示
4. **维护成本高**：随着项目增长，JavaScript 代码难以维护

### 预期收益
1. **编译时类型检查**：在构建时发现类型错误
2. **更好的 IDE 支持**：自动补全、重构、跳转定义
3. **代码质量提升**：强制类型约束，减少运行时错误
4. **文档价值**：类型定义即文档，易于理解代码意图
5. **团队协作**：统一的类型约束，降低沟通成本

## 技术方案

### 迁移策略
采用**渐进式迁移**策略：
1. 配置 TypeScript 环境，支持 `.ts` 和 `.js` 混用
2. 优先迁移核心类型定义和工具函数
3. 逐个迁移 Vue 组件（`.vue` 文件中的 `<script>` 改为 `<script setup lang="ts">`）
4. 最后迁移配置文件和入口文件

### 技术栈升级
- 添加 `typescript` 和 `vue-tsc` 依赖
- 配置 `tsconfig.json` 和 `vite.config.ts`
- 添加类型声明文件（`*.d.ts`）
- 引入 ESLint 和 TypeScript 相关插件（代码质量检查）
- 配置 Tauri 构建流程强制类型检查和 lint 检查

### 类型定义策略
1. **Tauri 命令类型**：为所有 `invoke` 调用定义参数和返回类型
2. **数据模型类型**：定义 Card、Project、Profile 等核心数据结构
3. **组件 Props/Emits**：使用 TypeScript 定义组件接口
4. **工具函数类型**：为日期格式化、状态转换等函数添加类型

## 范围

### 包含
- ✅ TypeScript 配置和构建环境
- ✅ Tauri 命令类型定义
- ✅ 核心数据模型类型定义
- ✅ 所有 `.vue` 组件迁移到 TypeScript
- ✅ 工具函数和辅助函数类型化
- ✅ 配置文件迁移（`vite.config.ts`、`main.ts`）
- ✅ ESLint 集成（TypeScript + Vue 规则）
- ✅ Tauri 构建前自动类型检查和 lint

### 不包含
- ❌ 修改业务逻辑（仅添加类型，不改变功能）
- ❌ 重构现有代码结构
- ❌ 添加新功能

## 影响

### 开发流程
- **构建时间**：TypeScript 编译会略微增加构建时间（可接受）
- **开发体验**：显著提升（类型提示、错误检查）
- **学习曲线**：团队需要熟悉 TypeScript 语法（文档充足）

### 现有代码
- **兼容性**：渐进式迁移，不影响现有功能
- **测试**：需要验证所有页面和功能正常工作
- **回滚**：可以保留 `.js` 版本作为备份

### 依赖
- 新增依赖：
  - `typescript`：TypeScript 编译器
  - `vue-tsc`：Vue 类型检查工具
  - `@types/node`：Node.js 类型定义
  - `eslint`：代码质量检查工具
  - `@eslint/js`：ESLint JavaScript 配置
  - `typescript-eslint`：TypeScript ESLint 工具链
  - `eslint-plugin-vue`：Vue ESLint 插件
- 构建工具：Vite 原生支持 TypeScript，无需额外配置
- Tauri 集成：构建前自动执行类型检查和 lint

## 风险

### 技术风险
- **类型定义错误**：可能定义过于宽松或过于严格的类型
  - 缓解：采用合理的类型粒度，必要时使用 `any` 或 `unknown`
- **第三方库类型缺失**：Element Plus 等库的类型定义可能不完整
  - 缓解：使用 `@types/*` 包或手动声明类型

### 开发风险
- **迁移工作量**：需要逐个文件迁移和测试
  - 缓解：采用渐进式迁移，优先级排序
- **团队适应**：开发者需要熟悉 TypeScript
  - 缓解：提供文档和示例，代码审查确保质量

## 时间估算

| 阶段 | 工作内容 | 预估时间 |
|------|---------|---------|
| 1. 环境配置 | TypeScript 配置、构建工具升级 | 2-3 小时 |
| 2. 类型定义 | Tauri 命令、数据模型类型定义 | 3-4 小时 |
| 3. 组件迁移 | 18 个 `.vue` 文件迁移 | 6-8 小时 |
| 4. 工具函数 | 工具函数类型化 | 2-3 小时 |
| 5. 测试验证 | 功能测试、类型检查 | 2-3 小时 |
| **总计** | | **15-21 小时** |

## 实施计划

### Phase 1: 环境配置（2-3 小时）
1. 安装 TypeScript 依赖
2. 创建 `tsconfig.json`
3. 迁移 `vite.config.js` → `vite.config.ts`
4. 配置构建脚本
5. 安装和配置 ESLint
6. 更新 Tauri 构建流程（已自动集成）

### Phase 2: 类型定义（3-4 小时）
1. 创建 `src/types/` 目录
2. 定义 Tauri 命令类型（`tauri.d.ts`）
3. 定义数据模型类型（`models.d.ts`）
4. 定义组件类型（`components.d.ts`）

### Phase 3: 核心文件迁移（6-8 小时）
1. 迁移 `main.js` → `main.ts`
2. 迁移 `App.vue`（添加 `lang="ts"`）
3. 迁移核心页面组件（7 个 views）
4. 迁移功能组件（13 个 components）

### Phase 4: 验证和优化（2-3 小时）
1. 运行 `vue-tsc --noEmit` 检查类型错误
2. 修复类型错误和警告
3. 功能测试验证
4. 优化类型定义

## 验收标准

### 必须满足
1. ✅ 所有 `.vue` 文件使用 `<script setup lang="ts">`
2. ✅ `main.ts` 和 `vite.config.ts` 使用 TypeScript
3. ✅ `npm run build` 成功，无类型错误和 lint 错误
4. ✅ `vue-tsc --noEmit` 通过，无类型错误
5. ✅ `eslint .` 通过，无严重错误
6. ✅ 所有现有功能正常工作
7. ✅ 至少 80% 的代码有明确类型定义（非 `any`）
8. ✅ Tauri 开发和构建流程自动执行检查

### 期望满足
1. ✅ Tauri 命令有完整的类型定义
2. ✅ 核心数据模型有详细的接口定义
3. ✅ 组件 Props 和 Emits 有类型约束
4. ✅ IDE 提供完整的类型提示和自动补全

## 替代方案

### 方案 A：保持 JavaScript + JSDoc
- **优点**：无需额外依赖，迁移成本低
- **缺点**：类型检查不严格，IDE 支持有限
- **结论**：不推荐，长期维护成本高

### 方案 B：完全重写为 TypeScript
- **优点**：类型定义最完善
- **缺点**：工作量巨大，风险高
- **结论**：不推荐，投入产出比低

### 方案 C：渐进式迁移到 TypeScript（推荐）
- **优点**：平衡迁移成本和收益，可控风险
- **缺点**：需要一定时间完成
- **结论**：推荐，符合实际需求

## 依赖关系

### 前置条件
- ✅ 前端项目已稳定运行
- ✅ 核心功能已实现

### 后续工作
- 建议：添加 ESLint TypeScript 规则
- 建议：添加类型覆盖率检查
- 建议：编写 TypeScript 开发规范文档

## 参考资料

- [Vue 3 + TypeScript 官方文档](https://vuejs.org/guide/typescript/overview.html)
- [Vite TypeScript 配置](https://vitejs.dev/guide/features.html#typescript)
- [Tauri TypeScript 指南](https://tauri.app/v1/guides/features/typescript/)
- [Element Plus TypeScript 支持](https://element-plus.org/en-US/guide/typescript.html)
