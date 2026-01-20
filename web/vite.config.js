import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    // 自定义插件：在构建时注入 eel.js（必须在 Vue 应用之前加载）
    {
      name: 'inject-eel',
      transformIndexHtml(html) {
        // 在 </head> 之前注入 eel.js，这样它会在 Vue 模块执行前加载
        return html.replace(
          '</head>',
          '  <script type="text/javascript" src="/eel.js"></script>\n</head>'
        )
      }
    }
  ],
  base: './',
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    emptyOutDir: true,
    rollupOptions: {
      output: {
        manualChunks: undefined
      }
    }
  },
  server: {
    port: 5173,
    open: false
  }
})
