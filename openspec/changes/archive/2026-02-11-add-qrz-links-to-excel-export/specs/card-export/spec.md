## ADDED Requirements

### 需求：导出地址缓存列
系统在导出 Excel 时必须增加三列地址缓存字段，并且必须仅使用卡片缓存数据填充。

#### 场景：新增三列并保持顺序
- **当** 用户导出卡片 Excel 文件
- **那么** 文件列顺序必须为：序号、呼号、数量、状态、QRZ.cn(中文)、QRZ.cn(English)、QRZ.com

#### 场景：按来源填充三列
- **当** 卡片 `metadata.address_cache` 同时包含 `qrz.cn` 和 `qrz.com` 缓存
- **那么** `QRZ.cn(中文)` 列必须写入 `qrz.cn` 的 `chinese_address`
- **并且** `QRZ.cn(English)` 列必须写入 `qrz.cn` 的 `english_address`
- **并且** `QRZ.com` 列必须写入 `qrz.com` 的 `english_address`

#### 场景：无缓存时留空
- **当** 卡片不存在 `address_cache` 或无对应来源记录
- **那么** `QRZ.cn(中文)`、`QRZ.cn(English)`、`QRZ.com` 三列必须写入空单元格

#### 场景：部分缓存时部分填充
- **当** 卡片仅有 `qrz.cn` 缓存而无 `qrz.com` 缓存
- **那么** `QRZ.cn(中文)` 与 `QRZ.cn(English)` 必须按缓存写入
- **并且** `QRZ.com` 列必须为空

## MODIFIED Requirements

### 需求：Excel 文件格式
导出的 Excel 文件必须符合标准 xlsx 格式，包含规定的列结构。

#### 场景：文件列结构
- **当** 导出完成
- **那么** Excel 文件必须包含以下列（按顺序）：
  - 第1列：序号
  - 第2列：呼号
  - 第3列：数量
  - 第4列：状态
  - 第5列：QRZ.cn(中文)
  - 第6列：QRZ.cn(English)
  - 第7列：QRZ.com

#### 场景：表头行
- **当** 打开导出的文件
- **那么** 第一行必须为表头行，内容为：序号、呼号、数量、状态、QRZ.cn(中文)、QRZ.cn(English)、QRZ.com

## REMOVED Requirements

- 无
