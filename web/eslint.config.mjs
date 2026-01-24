import eslint from '@eslint/js'
import tseslint from 'typescript-eslint'
import vuePlugin from 'eslint-plugin-vue'
import vueParser from 'vue-eslint-parser'

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  ...vuePlugin.configs['flat/recommended'],
  {
    files: ['**/*.vue', '**/*.ts'],
    languageOptions: {
      parser: vueParser,
      parserOptions: {
        parser: tseslint.parser,
        ecmaVersion: 2020,
        sourceType: 'module',
        extraFileExtensions: ['.vue']
      },
      globals: {
        // Browser globals
        console: 'readonly',
        document: 'readonly',
        window: 'readonly',
        navigator: 'readonly',
        setTimeout: 'readonly',
        clearTimeout: 'readonly',
        setInterval: 'readonly',
        clearInterval: 'readonly',
        Blob: 'readonly',
        URL: 'readonly',
        HTMLInputElement: 'readonly',
        MouseEvent: 'readonly',
        Event: 'readonly',
        EventTarget: 'readonly',
        fetch: 'readonly',
        FileReader: 'readonly'
      }
    }
  },
  {
    rules: {
      // ==================== TypeScript 规则 ====================

      // any 类型：类型定义文件中允许使用，业务代码中警告
      '@typescript-eslint/no-explicit-any': 'warn',

      // 未使用变量：以 _ 开头的变量忽略
      '@typescript-eslint/no-unused-vars': ['warn', {
        argsIgnorePattern: '^_',
        varsIgnorePattern: '^_',
        caughtErrorsIgnorePattern: '^_?error$'
      }],

      // 非空断言：在 Vue 组件中经常需要对 ref 进行断言，关闭此规则
      // 原因：Vue 组件模板引用（如 formRef.value!）在运行时保证存在
      '@typescript-eslint/no-non-null-assertion': 'off',

      // 空对象类型：允许继承时使用空接口
      '@typescript-eslint/no-empty-object-type': 'off',

      // 函数返回类型：不强制要求（TypeScript 可以自动推断）
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/explicit-module-boundary-types': 'off',

      // 禁止使用 @ts-ignore，推荐使用 @ts-expect-error
      '@typescript-eslint/ban-ts-comment': ['warn', {
        'ts-ignore': 'allow-with-description',
        'ts-expect-error': 'allow-with-description'
      }],

      // ==================== Vue 规则 ====================

      // 组件名称：允许单词组件名（如 App.vue）
      'vue/multi-word-component-names': 'off',

      // Props 默认值：不强制要求
      'vue/require-default-prop': 'off',
      'vue/no-required-prop-with-default': 'off',

      // 未使用变量
      'vue/no-unused-vars': 'warn',

      // Props 修改：警告直接修改 props
      'vue/no-mutating-props': 'warn',

      // 模板变量遮蔽：在 v-for 中使用同名变量是常见模式，关闭
      'vue/no-template-shadow': 'off',

      // v-html 指令：允许使用（需注意 XSS 风险）
      'vue/no-v-html': 'off',

      // 组件标签顺序：template -> script -> style
      'vue/block-order': ['warn', {
        order: ['template', 'script', 'style']
      }],

      // 属性顺序
      'vue/attributes-order': ['warn', {
        order: [
          'DEFINITION',      // is, v-is
          'LIST_RENDERING',  // v-for
          'CONDITIONALS',    // v-if, v-else-if, v-else, v-show, v-cloak
          'RENDER_MODIFIERS', // v-pre, v-once
          'GLOBAL',          // id
          'UNIQUE',          // ref, key
          'SLOT',            // v-slot, slot
          'TWO_WAY_BINDING', // v-model
          'OTHER_DIRECTIVES', // v-custom-directive
          'OTHER_ATTR',      // custom attributes
          'EVENTS',          // @click, v-on
          'CONTENT'          // v-text, v-html
        ],
        alphabetical: false
      }],

      // ==================== 通用规则 ====================

      // console：允许使用（开发和调试需要）
      'no-console': 'off',

      // debugger：生产环境警告
      'no-debugger': 'warn',

      // undefined 检查：TypeScript 处理
      'no-undef': 'off',

      // 优先使用 const
      'prefer-const': 'warn',

      // 禁止 var
      'no-var': 'error',

      // 对象简写
      'object-shorthand': ['warn', 'always'],

      // 箭头函数体
      'arrow-body-style': ['warn', 'as-needed'],

      // 相等比较使用 ===
      'eqeqeq': ['warn', 'always', { null: 'ignore' }]
    }
  },
  // 类型定义文件特殊规则
  {
    files: ['**/*.d.ts', '**/types/**/*.ts'],
    rules: {
      // 类型定义中允许使用 any
      '@typescript-eslint/no-explicit-any': 'off'
    }
  },
  {
    ignores: ['dist/**', 'node_modules/**', '*.config.js', '*.config.mjs']
  }
)
