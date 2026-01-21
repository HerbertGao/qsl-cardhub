# qsl-cardhub 前端项目

基于 **Vite + Vue 3 + Element Plus** 的现代化前端项目。

## 技术栈

- **Vue 3** - 渐进式 JavaScript 框架
- **Element Plus** - Vue 3 UI 组件库
- **Vite** - 下一代前端构建工具
- **Eel** - Python 与前端的桥接

## 项目结构

```
web/
├── src/
│   ├── views/          # 页面组件
│   │   ├── PrintView.vue       # 打印页面
│   │   ├── ConfigView.vue      # 配置管理页面
│   │   └── AboutView.vue       # 关于页面
│   ├── components/     # 可复用组件（未来扩展）
│   ├── assets/         # 静态资源
│   ├── App.vue         # 根组件
│   └── main.js         # 入口文件
├── dist/               # 打包输出目录（生产环境）
├── index.html          # HTML 模板
├── vite.config.js      # Vite 配置
└── package.json        # 项目配置
```

## 开发流程

### 1. 安装依赖

```bash
cd web
npm install
```

### 2. 开发模式

启动开发服务器（支持热重载）：

```bash
npm run dev
```

访问 http://localhost:5173 查看效果。

**注意**：开发模式下 Eel API 调用可能会失败，因为需要 Python 后端。建议直接使用生产构建 + Eel 启动。

### 3. 生产构建

构建优化后的生产版本：

```bash
npm run build
```

构建产物会输出到 `dist/` 目录。

### 4. 启动完整应用

返回项目根目录，启动 Eel 应用（使用打包后的前端）：

```bash
cd ..
python3 eel_main.py
```

应用会自动打开浏览器访问 http://localhost:8000

## 开发建议

### 修改前端代码

1. 编辑 `src/` 目录下的 Vue 组件
2. 保存后运行 `npm run build` 重新打包
3. 重启 Python 应用查看效果

### 添加新页面

1. 在 `src/views/` 创建新的 `.vue` 文件
2. 在 `App.vue` 中引入并注册
3. 添加对应的菜单项

### 调用 Python API

使用 `window.eel` 调用 Python 函数：

```javascript
// 获取配置列表
const data = await window.eel.get_profiles()()

// 打印 QSL 卡片
const result = await window.eel.print_qsl(profileId, callsign, serial, qty)()
```

## 可用的 Eel API

- `get_profiles()` - 获取所有配置
- `get_platform_info()` - 获取平台信息
- `get_printers()` - 获取打印机列表
- `create_profile(name, printerName)` - 创建新配置
- `update_profile(profileDict)` - 更新配置
- `delete_profile(profileId)` - 删除配置
- `set_default_profile(profileId)` - 设置默认配置
- `export_profile(profileId)` - 导出配置
- `import_profile(jsonData)` - 导入配置
- `print_qsl(profileId, callsign, serial, qty)` - 打印 QSL 卡片
- `print_calibration(profileId)` - 打印校准页

## 常见问题

### Q: 为什么修改代码后没有生效？

A: 确保运行了 `npm run build` 重新打包，并重启了 Python 应用。

### Q: 如何添加新的依赖包？

A: 使用 `npm install package-name` 安装，然后在代码中 import。

### Q: 开发时如何调试？

A:
1. 使用浏览器开发者工具（F12）查看 Console
2. Vue Devtools 浏览器插件可以查看组件状态
3. Network 面板可以查看 API 请求

## 构建优化

当前构建大小：
- CSS: ~350 KB (gzip: 47 KB)
- JS: ~1.15 MB (gzip: 366 KB)

如需进一步优化，可以：
1. 使用动态导入（`import()`）按需加载
2. 配置 `manualChunks` 分离第三方库
3. 使用 Element Plus 按需引入（而非全量引入）

## 版本信息

- Vue: 3.4+
- Element Plus: 2.5+
- Vite: 5.0+
- Node.js: 20+
