# 任务列表

## 1. 更新版本号解析逻辑

- [x] 1.1 扩展 `parse_version` 函数
  - 添加日期格式解析：`YYYY.M.D.NNN`
  - 保留旧格式支持：`NNN`
  - 使用正则表达式匹配两种格式

- [x] 1.2 实现版本号转整数
  - 新格式：`(YYYY - 2020) * 10000000 + MMDD * 1000 + NNN`
  - 旧格式：直接返回数字

- [x] 1.3 添加单元测试
  - 测试新格式解析
  - 测试旧格式解析
  - 测试混合格式排序
  - 测试边界情况

## 2. 整合现有迁移文件

- [x] 2.1 创建整合后的 SF Express 迁移文件
  - 合并 003、004、005 为单个文件
  - 在 CREATE TABLE 中直接包含 pay_method 和 cargo_name 列

- [x] 2.2 重命名迁移文件
  - `001_init.sql` → `2026.1.23.001_init.sql`
  - `002_add_cards.sql` → `2026.1.23.002_add_cards.sql`
  - 新整合文件命名为 `2026.1.23.003_add_sf_express.sql`

- [x] 2.3 删除旧的碎片化迁移文件
  - 删除 `003_add_sf_express.sql`
  - 删除 `004_add_pay_method.sql`
  - 删除 `005_add_cargo_name.sql`

## 3. 处理已有数据库兼容

- [x] 3.1 添加版本号迁移逻辑
  - 检测旧版本号（< 1000）
  - 将旧版本号映射到新版本号
  - 更新数据库的 user_version

- [x] 3.2 添加兼容性测试
  - 测试 `migrate_version_number` 函数
  - 验证旧版本号正确映射到新版本号

## 4. 验证和清理

- [x] 4.1 端到端测试
  - 运行单元测试验证迁移逻辑
  - 所有 7 个测试通过

- [x] 4.2 更新相关测试用例
  - 更新 `test_parse_version_valid` → `test_parse_version_old_format`
  - 新增 `test_parse_version_new_format`
  - 新增 `test_parse_version_sorting`
  - 新增 `test_migrate_version_number`
