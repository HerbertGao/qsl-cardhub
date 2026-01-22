# 规范：前端 TypeScript 迁移

## 新增需求

### 需求：TypeScript 环境配置

作为开发者，**必须**配置完整的 TypeScript 开发环境，以便使用 TypeScript 开发前端代码。

#### 场景：安装 TypeScript 依赖

**假设** 项目使用 npm 管理依赖
**当** 执行 `npm install` 安装依赖
**那么** 应该安装以下 TypeScript 相关包：
- `typescript` ^5.0.0
- `vue-tsc` ^1.8.0
- `@types/node` ^20.0.0

#### 场景：配置 TypeScript 编译器

**假设** 项目根目录为 `/web`
**当** 查看 `/web/tsconfig.json`
**那么** 配置应包含以下选项：

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "preserve",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src/**/*.ts", "src/**/*.d.ts", "src/**/*.tsx", "src/**/*.vue"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

**并且** 应创建 `/web/tsconfig.node.json` 用于 Vite 配置：

```json
{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true
  },
  "include": ["vite.config.ts"]
}
```

#### 场景：迁移 Vite 配置到 TypeScript

**假设** 存在 `/web/vite.config.js`
**当** 将其重命名为 `/web/vite.config.ts`
**并且** 添加类型导入 `import { defineConfig } from 'vite'`
**那么** Vite 应能正常加载配置并启动开发服务器

#### 场景：配置类型检查脚本

**假设** 编辑 `/web/package.json`
**当** 在 `scripts` 中添加 `"type-check": "vue-tsc --noEmit"`
**那么** 执行 `npm run type-check` 应输出类型检查结果

---

### 需求：核心类型定义

作为开发者，**必须**为核心数据结构和 Tauri 命令定义类型，以便在编码时获得类型安全和自动补全。

#### 场景：定义 Tauri 命令类型

**假设** 创建 `/web/src/types/tauri.ts`
**当** 定义 Tauri 命令类型
**那么** 应包含所有命令的参数和返回类型定义

**例如**（卡片管理命令）：

```typescript
// 创建卡片
export interface CreateCardParams {
  project_id: string
  callsign: string
  qty: number
}

export interface CreateCardResponse {
  id: string
  project_id: string
  callsign: string
  qty: number
  status: CardStatus
  // ... 其他字段
}

// 查询卡片列表
export interface ListCardsParams {
  project_id?: string
  callsign?: string
  status?: string
  page?: number
  page_size?: number
}

export interface PagedCardsResponse {
  items: CardWithProject[]
  total: number
  page: number
  page_size: number
  total_pages: number
}
```

**并且** 应为 QRZ.cn 命令定义类型：

```typescript
export interface QRZSaveAndLoginParams {
  username: string
  password: string
}

export interface QRZQueryCallsignParams {
  callsign: string
}

export interface AddressInfo {
  callsign: string
  chinese_address: string
  source: string
}
```

#### 场景：定义数据模型类型

**假设** 创建 `/web/src/types/models.ts`
**当** 定义核心数据模型
**那么** 应包含以下接口定义：

```typescript
export type CardStatus = 'pending' | 'distributed' | 'returned'

export interface Card {
  id: string
  project_id: string
  creator_id?: string
  callsign: string
  qty: number
  status: CardStatus
  metadata?: CardMetadata
  created_at: string
  updated_at: string
}

export interface CardMetadata {
  distribution?: DistributionInfo
  return?: ReturnInfo
  address_history?: AddressHistory[]
}

export interface AddressHistory {
  source: string
  chinese_address: string
  updated_at: string
  cached_at: string
}

export interface Project {
  id: string
  name: string
  created_at: string
  updated_at: string
}

export interface Profile {
  id: string
  name: string
  task_name?: string
  printer: PrinterConfig
  platform: PlatformInfo
  // ... 其他字段
}
```

#### 场景：定义组件 Props 类型

**假设** 创建 `/web/src/types/components.ts`
**当** 定义通用组件类型
**那么** 应包含对话框和表单的 Props 定义

**例如**：

```typescript
export interface DialogProps {
  visible: boolean
  card?: Card | null
}

export interface DialogEmits {
  (e: 'update:visible', value: boolean): void
  (e: 'confirm', data: any): void
  (e: 'refresh'): void
}

export interface FormData {
  [key: string]: string | number | boolean | undefined
}
```

---

### 需求：组件 TypeScript 迁移

作为开发者，**必须**将所有 Vue 组件迁移到 TypeScript，以便享受类型安全和更好的开发体验。

#### 场景：迁移入口文件

**假设** 存在 `/web/src/main.js`
**当** 将其重命名为 `/web/src/main.ts`
**并且** 添加类型导入和注解
**那么** 代码应如下：

```typescript
import { createApp } from 'vue'
import ElementPlus from 'element-plus'
import zhCn from 'element-plus/dist/locale/zh-cn.mjs'
import 'element-plus/dist/index.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import App from '@/App.vue'

const app = createApp(App)

app.use(ElementPlus, {
  locale: zhCn
})

for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

app.mount('#app')
```

**并且** `/web/index.html` 中的 script 引用应更新为 `/src/main.ts`

#### 场景：迁移 Vue 组件到 TypeScript

**假设** 有 Vue 组件使用 `<script setup>`
**当** 添加 `lang="ts"` 属性
**那么** 组件脚本应使用 TypeScript 语法

**例如**（`App.vue`）：

```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Profile } from '@/types/models'

const activeMenu = ref<string>('cards')
const profiles = ref<Profile[]>([])

const handleMenuSelect = (index: string): void => {
  activeMenu.value = index
}

onMounted(async () => {
  try {
    profiles.value = await invoke<Profile[]>('get_profiles')
  } catch (error) {
    console.error('获取配置失败:', error)
  }
})
</script>
```

#### 场景：组件 Props 和 Emits 类型定义

**假设** 组件需要接收 Props 并发出事件
**当** 使用 TypeScript 定义 Props 和 Emits
**那么** 应使用 `defineProps` 和 `defineEmits` 的泛型版本

**例如**（`DistributeDialog.vue`）：

```vue
<script setup lang="ts">
import type { Card } from '@/types/models'

interface Props {
  visible: boolean
  card: Card | null
}

interface Emits {
  (e: 'update:visible', value: boolean): void
  (e: 'confirm', data: DistributeFormData): void
  (e: 'refresh'): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

interface DistributeFormData {
  id: string
  method: string
  address: string | null
  remarks: string | null
}

const form = ref<DistributeFormData>({
  method: '快递',
  remarks: ''
})
</script>
```

#### 场景：调用 Tauri 命令时使用类型

**假设** 需要调用 Tauri 后端命令
**当** 使用 `invoke` 调用命令
**那么** 应使用泛型指定返回类型

**例如**：

```typescript
import { invoke } from '@tauri-apps/api/core'
import type { PagedCardsResponse, ListCardsParams } from '@/types/tauri'

const loadCards = async (params: ListCardsParams): Promise<void> => {
  try {
    const result = await invoke<PagedCardsResponse>('list_cards_cmd', params)
    cards.value = result.items
    total.value = result.total
  } catch (error) {
    console.error('加载卡片失败:', error)
  }
}
```

---

### 需求：类型检查和验证

作为开发者，**必须**在构建前进行类型检查，以便在开发阶段发现类型错误。

#### 场景：运行类型检查

**假设** 已配置 TypeScript 环境
**当** 执行 `npm run type-check`
**那么** 应输出所有类型错误和警告
**并且** 如果没有错误，应输出 "Found 0 errors."

#### 场景：构建时自动类型检查

**假设** 已配置构建脚本
**当** 执行 `npm run build`
**那么** 应在构建前自动运行类型检查
**并且** 如果有类型错误，构建应失败

#### 场景：IDE 类型提示

**假设** 使用支持 TypeScript 的 IDE（如 VS Code）
**当** 在代码中输入变量或函数
**那么** IDE 应显示类型提示和自动补全
**并且** 鼠标悬停应显示完整的类型定义

#### 场景：类型覆盖率要求

**假设** 所有组件已迁移到 TypeScript
**当** 检查代码中的类型使用情况
**那么** 至少 80% 的代码应有明确的类型定义（非 `any`）
**并且** 核心数据模型、Tauri 命令、组件 Props 应有 100% 的类型定义

---

### 需求：构建流程集成 TypeScript

作为开发者，**必须**在构建流程中集成 TypeScript 类型检查，以便在构建前发现类型错误。

#### 场景：配置构建脚本

**假设** 编辑 `/web/package.json`
**当** 配置 `build` 脚本
**那么** 脚本应包含类型检查：`"build": "vue-tsc --noEmit && vite build"`
**并且** 如果类型检查失败，构建应中止

#### 场景：开发模式类型检查（可选）

**假设** 需要在开发模式下实时类型检查
**当** 配置 Vite 插件
**那么** 可以添加 `vite-plugin-checker` 插件实现实时类型检查

---

## 验收标准

### 环境配置验收
1. ✅ TypeScript 依赖已安装
2. ✅ `tsconfig.json` 配置正确
3. ✅ `vite.config.ts` 能正常加载
4. ✅ `npm run type-check` 能正常运行

### 类型定义验收
1. ✅ 所有 Tauri 命令有类型定义
2. ✅ 核心数据模型有接口定义
3. ✅ 组件 Props 和 Emits 有类型约束
4. ✅ 类型定义文件组织清晰

### 组件迁移验收
1. ✅ 所有 `.vue` 文件使用 `<script setup lang="ts">`
2. ✅ `main.ts` 使用 TypeScript
3. ✅ 所有组件正常渲染
4. ✅ 所有功能正常工作

### 类型质量验收
1. ✅ `npm run type-check` 通过，无错误
2. ✅ `npm run build` 成功
3. ✅ 至少 80% 的代码有明确类型
4. ✅ IDE 类型提示正常工作

### 开发体验验收
1. ✅ 调用 Tauri 命令时有参数提示
2. ✅ 组件 Props 有类型检查
3. ✅ 修改类型定义时能自动检测错误
4. ✅ 重构代码时有完整的类型支持
