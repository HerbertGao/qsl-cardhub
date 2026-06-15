## 为什么

「导出卡片到 Excel」对任何卡片数超过 100 的项目都只导出前 100 条（Issue #43：项目实有 415 张，导出文件仅 101 行 = 1 表头 + 100 数据）。根因是导出复用了带分页上限的 `list_cards`——该函数把 `page_size` 钳到最大 100（`src/db/cards.rs`），导出方虽显式传入 `page_size: 100000` 也被静默截断。这导致用户在毫无报错的情况下丢失大量导出数据，是数据完整性问题。

## 变更内容

- 在数据库层新增不分页的全量查询 `db::list_all_cards(filter)`，返回符合筛选条件的全部卡片（无 `LIMIT`），与分页的 `list_cards` 解耦。
- `export_cards_to_excel`（`src/commands/export.rs`）改用 `list_all_cards`，移除失效的 `page_size: 100000` 写法。
- `list_cards` 的 `page_size ≤ 100` 上限**保持不变**，继续保护 UI 卡片列表的分页行为（非破坏性）。
- 复用现有查询逻辑：抽出共享的 WHERE 子句构建与行映射，避免 SQL 重复；为新查询补充单元测试覆盖（>100 条不被截断、UI 分页上限仍生效）。

## 功能 (Capabilities)

### 新增功能
<!-- 无新增功能 -->

### 修改功能
- `card-export`: 新增「导出必须包含当前项目全部卡片」的行为需求——导出结果数据行数必须等于项目实际卡片数，不受任何分页上限限制。

## 影响

- 代码：`src/db/cards.rs`（新增 `list_all_cards` 及共享辅助函数、补测试）、`src/commands/export.rs`（改调用）。
- API：新增内部 DB 函数 `list_all_cards`，经 `pub use cards::*` 自动导出；不新增/不修改 Tauri command 签名，前端无感知。
- 行为：`list_cards` 与 UI 列表分页不受影响；其另一调用方 `list_cards_cmd`（`src/commands/cards.rs:43`）依赖 `list_cards` 公开签名不变，本次不改它、零改动即编译通过。完整数据备份路径（`src/db/export.rs`）本就正确，不动。
- 数据库 schema、迁移：无变更。
