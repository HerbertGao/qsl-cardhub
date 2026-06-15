## 1. 重构 db 层共享查询逻辑

- [x] 1.1 在 `src/db/cards.rs` 新增 `build_card_where(filter: &CardFilter) -> (String, Vec<Box<dyn rusqlite::ToSql>>)`，迁移现有 WHERE/参数构建逻辑。**crate-private（`fn`，不加 `pub`，避免 `pub use cards::*` 泄漏到 `db::*` 公共面）**；返回的 WHERE 串**不含前导/尾随空白**（空字符串或 `WHERE …`，沿用现状 `src/db/cards.rs:129-133`），片段间空白由调用方共享主体的尾部换行保证（见 1.3）
- [x] 1.2 在 `src/db/cards.rs` 新增 `map_card_row(row: &rusqlite::Row) -> rusqlite::Result<CardWithProject>`（**crate-private，不加 `pub`**），**逐字迁移**现有行映射闭包（`src/db/cards.rs:193-209`，列序 0..=9 不变，包括第 2 列 `project_name: row.get(2)?` 读为 `String`）。本次为**零行为变更**重构——不改任何列的读取语义（孤儿卡片 NULL `project_name` 的既有行为见 design「风险」，属 OUT-OF-SCOPE，不在本次处理）
- [x] 1.3 抽出共享的 SELECT 主体（SELECT 列 + FROM cards LEFT JOIN projects），供分页与全量查询拼接复用。**拼接空白契约**：共享主体与后续 `WHERE …`、`ORDER BY c.created_at DESC`、`LIMIT/OFFSET` 之间须各留至少一个换行或空格（沿用现状 `src/db/cards.rs:160-177` 的 raw string 形态），避免拼出 `projects pORDER BY` 之类语法错误
- [x] 1.4 将 `list_cards` 主体下沉为接收连接的 `list_cards_conn(conn, filter, pagination)`（**crate-private，不加 `pub`**），公开 `list_cards` 改为 `get_connection()` 后委托；**公开签名 `pub fn list_cards(filter: CardFilter, pagination: Pagination) -> Result<PagedCards, AppError>` 逐字不变**（另一调用方 `list_cards_cmd`（`src/commands/cards.rs:43`）零改动即编译通过）；行为（含 `page_size ≤ 100` 上限、`page/page_size` 下限 `.max(1)`、`total/page/page_size/total_pages` 计算）保持不变。**参数顺序契约**：先用 `build_card_where` 返回的 params 切片跑 COUNT，再向同一 `Vec` push `LIMIT`（`?{N+1}`）/`OFFSET`（`?{N+2}`）后跑数据查询——COUNT 不得消费 LIMIT/OFFSET 参数，占位符编号须随 `build_card_where` 的实际条件数（0~3）动态计算，禁止写死 `?1/?2`。**借用顺序**：`params_ref`（`Vec<&dyn ToSql>`）必须在向 `Vec` push 完 LIMIT/OFFSET **之后**、`params` 不再被修改时再构造（沿用现状 `src/db/cards.rs:190`），避免 push 与不可变借用冲突

## 2. 新增全量查询与导出改造

- [x] 2.1 在 `src/db/cards.rs` 新增 `list_all_cards_conn(conn, filter) -> Result<Vec<CardWithProject>, AppError>`（**crate-private，不加 `pub`**）：复用 `build_card_where` + 共享 SELECT 主体 + `ORDER BY c.created_at DESC`（限定 `c.` 别名，与 `list_cards` 现状一致、避免与 `projects.created_at` 歧义），**无 LIMIT/OFFSET**，用 `map_card_row` 映射
- [x] 2.2 在 `src/db/cards.rs` 新增公开 `list_all_cards(filter) -> Result<Vec<CardWithProject>, AppError>`，`get_connection()` 后委托 `list_all_cards_conn`（经 `pub use cards::*` 自动导出，无需改 `src/db/mod.rs`）
- [x] 2.3 修改 `src/commands/export.rs::export_cards_to_excel`：用 `db::list_all_cards(filter)` 取卡片（`CardFilter` 仍仅设 `project_id`，`callsign`/`status` 保持 `None` —— 全量导出语义不变），删除 `db::Pagination`/`page_size: 100000` 写法。`list_all_cards` 返回 `Result<_, AppError>`，而 `spawn_blocking` 闭包的错误类型是 `String`，故须 `.map_err(|e| format!("获取卡片列表失败: {}", e))?` 转换（与现状 `src/db/cards.rs` 调用点一致），再 `Ok::<(Project, Vec<CardWithProject>), String>((project, cards))`。保留其后 `cards.is_empty()` 空导出短路（`src/commands/export.rs:260`）

## 3. 测试

- [x] 3.1 在 `src/db/cards.rs` 新增 `#[cfg(test)] mod tests` + `setup_test_db() -> Connection`（`rusqlite::Connection::open_in_memory()`）：
  - **建表语句取迁移以保真**：`projects` 取 `2026.1.24.001_init.sql`（`id/name(UNIQUE)/created_at/updated_at`），`cards` 取 `2026.1.24.002_add_cards.sql`（含两条 CHECK，与生产 schema 一致）。注：本批断言均用合法数据，**不直接依赖 CHECK 触发**；CHECK 仅为测试 schema 与生产保真，不必为它单写「插非法值期望报错」的用例（那是测 SQLite 引擎、越界）。`project_id` FK 行可保留但测试不依赖其强制（不开 `PRAGMA foreign_keys`）。
  - **必须先插入对应的 `projects` 行（含 `name`），再插 `cards`**，确保 LEFT JOIN 命中、`project_name` 可判定（否则 3.2②/3.4 失败）。
  - **数据用直接 SQL `INSERT` 写入**（**禁止 `create_card()`**——它走系统时钟、秒级精度，批量插入会产生同值 `created_at` 使 `DESC` 顺序不确定、断言偶发失败）：手填唯一且固定宽度递增的 `created_at`（如 `2026-01-01 00:00:01`、`..:02` …，按 serial 递增），`status='pending'`，确保排序确定可判定。`qty` 与 `serial` **取不同值**（如 `qty=1`、`serial` 递增 1..N），以便 3.2④ 能捕获 qty↔serial 列错位。
- [x] 3.2 测试 `list_all_cards_conn` 全量 + 列映射：单项目插入 150 条，断言①返回 150 条（不被 100 截断）②`project_name` 已填充③按 `created_at DESC`（首条 serial 最大、末条最小）④**逐列映射正确**——对一行 `serial != qty` 的记录（如 `serial=150`、`qty=1`）断言 `serial`/`qty`/`status`/`callsign`/`metadata` 均与插入值一致（须取 serial≠qty 的行，否则 qty↔serial 互换因两值相同而不可见，类型兼容下假绿）
- [x] 3.3 测试 `list_cards_conn` 分页上限：同样 150 条数据传 `page_size: 100000`，断言 `items.len() == 100` 且 `page_size == 100`、`total == 150`（证明 UI 分页上限未退化）
- [x] 3.4 测试 `list_all_cards_conn` 项目过滤：两个项目分别 120/30 条，按某项目筛选断言只返回该项目卡片
- [x] 3.5 测试占位符编号等价性（重构最大回归面，覆盖多个条件数 N）：对 `list_cards_conn` + 超上限 `page_size` 分别验证——①N=3：`project_id + callsign + status` 三条件（`LIMIT/OFFSET` 落 `?4/?5`），断言返回行全满足三条件且 `len()==100`；②N=0：在**同一 150 行库**上无 filter 查询（`LIMIT/OFFSET` 落 `?1/?2`），断言 `items.len()==100` 且 `total==150`（不可用空库，否则 total=0 无法验上限）。（N=1 已由 3.3 覆盖）证明 `build_card_where` 抽取后占位符编号随条件数动态、未写死

## 4. 验证

- [x] 4.1 运行 `cargo test`（含新增测试）全部通过
- [x] 4.2 运行 `cargo build`，确认**零警告**（尤其 `export.rs` 移除 `Pagination` 后无 unused import）
- [x] 4.3 核对 `setup_test_db` 的 `cards` 建表语句取自迁移（含两条 CHECK），与生产 schema 保真、未退化为 `import.rs` 省略版
- [ ] 4.4 手动验证：构造 >100 条卡片的项目，导出 Excel 并核对数据行数 = 实际卡片数 + 1（表头）；与「完整数据备份」交叉核对卡片数一致
