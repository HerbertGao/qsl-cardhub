## 为什么

导出 Excel 时如果还需要手动去系统里查地址，会增加重复操作。既然卡片 `metadata.address_cache` 已保存了 QRZ 查询结果，导出应直接复用缓存；同时当没有缓存时保持空值，避免导出过程触发额外查询。

## 变更内容

- 修改 Excel 导出列结构，在现有 4 列后新增 3 列：
  - `QRZ.cn(中文)`
  - `QRZ.cn(English)`
  - `QRZ.com`
- 三列值均来自卡片 `metadata.address_cache`：
  - `qrz.cn` 记录的 `chinese_address` -> `QRZ.cn(中文)`
  - `qrz.cn` 记录的 `english_address` -> `QRZ.cn(English)`
  - `qrz.com` 记录的 `english_address` -> `QRZ.com`
- 若对应来源缓存不存在（或字段为空），导出单元格必须留空。
- 不引入破坏性变更（无 BREAKING）。

## 功能 (Capabilities)

### 新增功能

- （无）

### 修改功能

- `card-export`: 扩展 Excel 文件列结构，新增三列并改为读取地址缓存填充内容。

## 影响

- 后端导出实现：`src/commands/export.rs`（表头、数据写入、列宽、测试）。
- 使用现有数据模型：`Card.metadata.address_cache`（无需新增存储结构）。
- 前端导出调用接口不变（参数与返回结构不变）。
