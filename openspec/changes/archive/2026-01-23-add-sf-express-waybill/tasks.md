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

## 6. 下单 API 集成

- [x] 6.1 扩展数据模型 (`models.rs`)
  - `CreateOrderRequest` - 下单请求结构
  - `CreateOrderResponse` - 下单响应结构
  - `UpdateOrderRequest` - 订单确认/取消请求结构
  - `UpdateOrderResponse` - 订单确认/取消响应结构
  - `SearchOrderRequest` - 订单查询请求结构
  - `SearchOrderResponse` - 订单查询响应结构
  - `SenderInfo` - 寄件人信息结构
  - `RecipientInfo` - 收件人信息结构

- [x] 6.2 实现下单 API 客户端方法 (`client.rs`)
  - `create_order` - 调用 EXP_RECE_CREATE_ORDER 接口
  - `update_order` - 调用 EXP_RECE_UPDATE_ORDER 接口（确认/取消）
  - `search_order` - 调用 EXP_RECE_SEARCH_ORDER_RESP 接口
  - 错误处理和响应解析

- [x] 6.3 实现下单相关 Tauri 命令 (`commands/sf_express.rs`)
  - `sf_create_order` - 创建订单
  - `sf_confirm_order` - 确认订单
  - `sf_cancel_order` - 取消订单
  - `sf_search_order` - 查询订单
  - 错误处理和日志记录

## 7. 数据库扩展

- [x] 7.1 创建订单表迁移
  - 表名：`sf_orders`
  - 字段：id, order_id, waybill_no, card_id, status, sender_info, recipient_info, pay_method, cargo_name, created_at, updated_at
  - 索引：order_id, card_id, status
  - pay_method：付款方式（1=寄方付, 2=收方付, 3=第三方付）
  - cargo_name：托寄物名称（默认"QSL卡片"）

- [x] 7.2 创建寄件人表迁移
  - 表名：`sf_senders`
  - 字段：id, name, phone, mobile, province, city, district, address, is_default, created_at, updated_at
  - 索引：is_default

- [x] 7.3 实现数据库操作 (`db/sf_express.rs`)
  - `create_order` - 创建订单记录
  - `update_order` - 更新订单状态
  - `get_order` - 获取订单
  - `list_orders` - 查询订单列表
  - `list_orders_with_cards` - 查询订单列表（含关联卡片信息，使用 LEFT JOIN）
  - `create_sender` - 创建寄件人
  - `update_sender` - 更新寄件人
  - `delete_sender` - 删除寄件人
  - `get_senders` - 获取寄件人列表
  - `get_default_sender` - 获取默认寄件人

## 8. 地址数据集成

- [x] 8.1 集成地址数据源
  - 下载或引用 [data_location](https://github.com/mumuy/data_location) 数据
  - 将地址数据转换为应用可用的格式（JSON）
  - 将地址数据打包到应用中（或支持动态加载）

- [x] 8.2 实现地址选择器组件
  - 创建 `AddressSelector.vue` 组件
  - 实现省市区三级联动
  - 支持搜索功能
  - 支持手动输入地址

- [x] 8.3 实现地址数据 Tauri 命令（改为前端实现）
  - 使用前端 TypeScript 模块处理地址数据
  - `getProvinces()` - 获取省份列表
  - `getCities()` - 获取城市列表（根据省份）
  - `getDistricts()` - 获取区县列表（根据城市）

## 9. 寄件人管理功能

- [x] 9.1 扩展配置页面 (`SFExpressConfigView.vue`)
  - 添加「管理寄件人」按钮
  - 添加寄件人列表显示区域

- [x] 9.2 创建寄件人管理对话框 (`SenderDialog.vue`)
  - 寄件人表单（姓名、电话、手机、地址）
  - 地址选择器集成
  - 默认寄件人设置
  - 保存/编辑/删除功能

- [x] 9.3 实现寄件人管理 Tauri 命令
  - `sf_create_sender` - 创建寄件人
  - `sf_update_sender` - 更新寄件人
  - `sf_delete_sender` - 删除寄件人
  - `sf_list_senders` - 获取寄件人列表
  - `sf_set_default_sender` - 设置默认寄件人

## 10. 下单功能集成

- [x] 10.1 创建下单对话框 (`CreateOrderDialog.vue`)
  - 寄件人信息显示（自动预填默认寄件人）
  - 收件人信息表单
  - 地址选择器集成
  - 表单验证
  - 提交订单按钮

- [x] 10.2 创建订单确认对话框（集成到 CreateOrderDialog）
  - 订单信息预览
  - 「立即确认」和「稍后确认」选项
  - 确认后自动回填运单号到卡片备注

- [x] 10.3 修改分发对话框 (`DistributeDialog.vue`)
  - 添加「下顺丰订单」按钮（当选择"快递"方式时显示）
  - 集成下单流程
  - 订单确认后显示「打印面单」选项

## 11. 订单列表页面

- [x] 11.1 创建订单列表页面 (`SFOrderListView.vue`)
  - 订单列表表格（使用 SFOrderWithCard 结构，通过 LEFT JOIN 关联卡片和项目表）
  - 显示呼号列（callsign）、项目名（project_name）、数量（qty）
  - 订单状态显示（待确认、已确认、已取消、已打印）
  - 筛选功能（按状态、按时间）
  - 排序功能

- [x] 11.2 实现订单操作功能
  - 「确认订单」按钮（待确认订单）
  - 「取消订单」按钮（待确认订单和已确认订单，已确认订单取消时需传入运单号）
  - 「查询订单」按钮
  - 「打印面单」按钮（已确认订单）
  - 「查看详情」按钮

- [x] 11.3 创建订单详情对话框（集成到 SFOrderListView）
  - 显示订单完整信息
  - 显示关联卡片信息
  - 支持跳转到卡片详情

- [x] 11.4 添加菜单入口
  - 在 `App.vue` 添加「顺丰订单」菜单项（位于卡片管理下方）
  - 使用 Van 图标

## 12. 订单与卡片关联

- [x] 12.1 实现订单回填功能
  - 订单确认成功后，自动将运单号回填到分发对话框备注
  - 通过 `handleSFOrderSuccess` 回调实现
  - card_id 在创建订单时关联

- [x] 12.2 在分发对话框集成下单入口
  - 选择「快递」方式时显示「顺丰速运下单」按钮
  - 订单创建成功后自动填入运单号
  - 支持后续打印面单操作

## 13. 集成测试

- [ ] 13.1 沙箱环境测试
  - 配置沙箱凭据
  - 测试下单 API 调用
  - 测试订单确认/取消
  - 测试订单查询
  - 验证 PDF 下载

- [ ] 13.2 打印测试
  - PDF 渲染验证
  - TSPL 指令验证
  - 实际打印测试

- [ ] 13.3 错误场景测试
  - 网络超时
  - 签名错误
  - 运单不存在
  - 打印机离线
  - 下单数据验证失败
  - 订单确认失败
  - 地址数据加载失败

- [ ] 13.4 完整流程测试
  - 分发卡片 → 下单 → 确认 → 打印面单
  - 分发卡片 → 下单 → 稍后确认 → 订单列表确认 → 打印面单
  - 分发卡片 → 下单 → 取消订单
