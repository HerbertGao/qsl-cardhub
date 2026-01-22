## 1. 后端基础设施

- [x] 1.1 创建 `src/sf_express/` 模块目录结构
  - `mod.rs` - 模块入口
  - `models.rs` - 数据模型
  - `client.rs` - API 客户端
  - `pdf_renderer.rs` - PDF 渲染器

- [x] 1.2 实现数据模型 (`models.rs`)
  - `SFExpressConfig` - 配置结构
  - `WaybillPrintRequest` - 请求结构
  - `CloudPrintResponse` - API 响应结构
  - `PrintFile` - PDF 文件信息

- [x] 1.3 实现数字签名计算
  - MD5 哈希计算
  - Base64 编码
  - 签名验证测试

- [x] 1.4 实现 API 客户端 (`client.rs`)
  - 构造请求（form-urlencoded）
  - 发送 POST 请求
  - 解析响应
  - 下载 PDF 文件（带 X-Auth-token）

## 2. PDF 渲染与打印

- [x] 2.1 添加 PDF 渲染依赖
  - 选择 `pdfium-render` crate
  - 添加到 Cargo.toml

- [x] 2.2 实现 PDF 转位图 (`pdf_renderer.rs`)
  - 加载 PDF 文件
  - 渲染为 RGB 位图
  - 缩放到目标 DPI（203 dpi，608×1040）

- [x] 2.3 实现位图转 1bpp 点阵
  - 灰度转换
  - 阈值二值化
  - MSB-first 位打包

- [x] 2.4 实现 TSPL BITMAP 指令生成
  - 生成 SIZE/GAP/DIRECTION/CLS 前导指令
  - 生成 BITMAP 指令
  - 生成 PRINT 指令

## 3. Tauri 命令

- [x] 3.1 创建 `src/commands/sf_express.rs`
  - 注册模块到 `commands/mod.rs`
  - 注册命令到 `main.rs`

- [x] 3.2 实现配置管理命令
  - `sf_save_config` - 保存配置和凭据
  - `sf_load_config` - 加载配置
  - `sf_clear_config` - 清除配置

- [x] 3.3 实现打印命令（两步流程）
  - `sf_fetch_waybill` - 获取PDF并返回预览图像
  - `sf_print_waybill` - 将PDF转换为TSPL并发送到打印机
  - 集成打印机选择逻辑
  - 错误处理和日志记录

## 4. 前端配置页面

- [x] 4.1 创建 `SFExpressConfigView.vue`
  - 环境切换（生产/沙箱）
  - 顾客编码输入
  - 模板编码显示（只读）
  - 生产校验码输入
  - 沙箱校验码输入

- [x] 4.2 实现配置保存/加载逻辑
  - 调用 Tauri 命令
  - 保存状态显示
  - 存储方式提示

- [x] 4.3 添加菜单入口
  - 在 `App.vue` 数据配置菜单添加「顺丰速运」
  - 路由配置

## 5. 前端打印对话框

- [x] 5.1 创建 `WaybillPrintDialog.vue`（两步流程）
  - 运单号输入框
  - 打印机选择
  - 「提交打印」按钮 → 获取PDF并显示预览
  - PDF 预览区域
  - 「打印」按钮 → 发送到打印机（仅在获取成功后可用）
  - 打印状态显示（加载中/成功/失败）
  - 错误提示

- [x] 5.2 实现自动带入运单号
  - 接收来源参数（列表/分发）
  - 从分发备注带入运单号

- [x] 5.3 集成到卡片管理
  - 在 `CardManagementView.vue` 卡片列表添加操作按钮
  - 在 `DistributeDialog.vue` 添加打印面单入口

## 6. 集成测试

- [ ] 6.1 沙箱环境测试
  - 配置沙箱凭据
  - 测试 API 调用
  - 验证 PDF 下载

- [ ] 6.2 打印测试
  - PDF 渲染验证
  - TSPL 指令验证
  - 实际打印测试

- [ ] 6.3 错误场景测试
  - 网络超时
  - 签名错误
  - 运单不存在
  - 打印机离线
