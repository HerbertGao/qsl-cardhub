## 1. 版本脚本

- [x] 1.1 创建 `scripts/version.sh` 版本号升级脚本
  - 支持 major/minor/patch/自定义版本号
  - 从 Cargo.toml 读取当前版本
  - 同步更新 Cargo.toml、tauri.conf.json、web/package.json
  - 添加版本格式验证（semver）
  - 支持 macOS 和 Linux

- [x] 1.2 创建 `scripts/release.sh` 发布脚本
  - 检查 Git 分支和工作区状态
  - 调用 version.sh 升级版本
  - 执行 cargo check 验证
  - 创建版本提交和 Git 标签
  - 可选推送到远程

- [x] 1.3 更新 `scripts/README.md` 文档
  - 添加 version.sh 使用说明
  - 添加 release.sh 使用说明
  - 添加发布流程说明

## 2. 关于页面改造

- [x] 2.1 添加版本号动态获取
  - 使用 Tauri API 获取应用版本
  - 或通过 Vite 编译时注入版本号
  - 替换 AboutView.vue 中的硬编码版本

- [x] 2.2 添加检查更新 UI
  - 添加"检查更新"按钮
  - 添加更新对话框组件
  - 显示加载状态、版本信息、更新日志
  - 添加下载进度显示

- [x] 2.3 实现启动时自动检查更新
  - 应用启动后后台静默检查
  - 不阻塞启动流程
  - 检查失败时静默忽略

- [x] 2.4 实现菜单红点标记
  - 创建更新状态全局存储（Vue reactive 或 Pinia）
  - 在侧边栏"关于"菜单项添加红点组件
  - 发现新版本时显示红点
  - 用户访问关于页面后清除红点

## 3. Tauri Updater 集成

- [x] 3.1 添加 tauri-plugin-updater 依赖
  - 在 Cargo.toml 添加依赖
  - 在 web/package.json 添加 @tauri-apps/plugin-updater

- [x] 3.2 配置 tauri.conf.json
  - 添加 updater 插件配置
  - 设置更新端点 URL
  - 配置公钥（待用户生成后填入）

- [x] 3.3 生成签名密钥对
  - 创建 `scripts/generate-keys.sh` 脚本
  - 用户手动运行生成密钥
  - 将私钥添加到 GitHub Secrets
  - 将公钥添加到 tauri.conf.json

- [x] 3.4 实现前端更新逻辑
  - 调用 check() 检查更新
  - 调用 downloadAndInstall() 下载安装
  - 处理更新事件（进度、完成、错误）
  - 实现 relaunch() 重启应用

## 4. CI/CD 改造

- [x] 4.1 修改 tauri.conf.json 构建目标
  - 添加 NSIS 到 bundle.targets（Windows）
  - 设置 createUpdaterArtifacts: true

- [x] 4.2 修改 release.yml 工作流
  - 添加签名密钥环境变量
  - 生成更新包签名文件
  - 生成并上传 latest.json
  - 上传 NSIS 安装包到 Release

- [x] 4.3 配置 GitHub Secrets（用户手动完成）
  - 添加 TAURI_SIGNING_PRIVATE_KEY
  - 添加 TAURI_SIGNING_PRIVATE_KEY_PASSWORD（可选）

## 5. 测试验证

- [ ] 5.1 测试版本脚本
  - 测试 version.sh 各种参数
  - 测试 release.sh 完整流程
  - 验证多平台兼容性

- [ ] 5.2 测试更新功能
  - 测试检查更新 API 调用
  - 测试下载进度显示
  - 测试 Windows NSIS 静默安装
  - 测试 macOS DMG 下载提示

- [ ] 5.3 端到端测试
  - 发布测试版本验证完整流程
  - 从旧版本升级到新版本
  - 验证签名验证机制
