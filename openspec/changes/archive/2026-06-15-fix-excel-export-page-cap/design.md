## 上下文

`src/db/cards.rs::list_cards(filter, pagination)` 是卡片列表的唯一查询入口，内部第 154 行 `let page_size = pagination.page_size.min(100).max(1);` 把页大小钳到最大 100，以保护 UI 卡片列表（`list_cards_cmd`，前端默认每页 20）不会一次性拉取过多行。

`src/commands/export.rs::export_cards_to_excel` 复用了 `list_cards`，并传入 `Pagination { page: 1, page_size: 100000 }` 意图取全量。但该值被上述 `.min(100)` 静默压回 100，导致任何 >100 卡片的项目导出只得到第一页（Issue #43）。

完整数据备份路径 `src/db/export.rs::export_cards` 走的是另一条无 `LIMIT` 的查询，结果正确（415 条），但它返回 `Vec<Card>`（无 `project_name`、含 `creator_id`、全库无项目过滤、排序为升序），与导出所需的 `Vec<CardWithProject>`（JOIN projects、按 `created_at DESC`、按项目过滤）不兼容，不能直接复用。

约束：`get_connection()` 走全局 `DB_PATH` OnceCell，没有测试注入口——这也是 `projects.rs` 测试模块为空占位的原因；`import.rs` 能测是因为其逻辑接收 `&Connection`。

## 目标 / 非目标

**目标：**
- 导出 Excel 必须包含项目全部卡片，行数等于实际卡片数，不被分页上限截断。
- 不改变 `list_cards` 的分页上限行为，UI 卡片列表零回归。
- 复用既有 WHERE 构建与行映射逻辑，避免 SQL/映射重复。
- 为新查询补充可运行的单元测试（验证 >100 不截断、分页上限仍生效）。

**非目标：**
- 不调整 `list_cards` 的 `page_size ≤ 100` 上限或前端分页交互。
- 不改 Tauri command 签名或前端代码（前端无感知）。
- 不触碰完整数据备份路径（`src/db/export.rs`）。
- 不引入数据库 schema/迁移变更。

## 决策

**决策 1：新增专用的不分页查询 `db::list_all_cards(filter)`，而非放宽 `list_cards` 的上限。**
- 理由：放宽 `.min(100)` 会削弱 UI 列表的保护，且「传一个巨大 page_size 当全量」是含糊的 hack；用独立函数让「全量」成为一个显式、自解释的 API。这也消除了一个结构性陷阱——未来任何想取某项目全部卡片的功能若误用 `list_cards` 都会被静默截断。
- 备选：① 放宽/移除 clamp（被否：UI 回归风险 + 仍是魔法数）；② 先查 total 再传 `page_size = total`（被否：仍被 clamp 拦截，不解决问题）；③ 循环翻页合并（被否：啰嗦，且每页仍受 100 限制需多次往返）。

**决策 2：抽出共享辅助，重构 `list_cards` 与新函数共用。**
- `build_card_where(&CardFilter) -> (String, Vec<Box<dyn ToSql>>)`：统一 WHERE 子句与参数构建。
- `map_card_row(&Row) -> rusqlite::Result<CardWithProject>`：统一行映射（列序与 SELECT 一致）。**逐字迁移现有闭包、零行为变更**（含第 2 列 `project_name: row.get(2)?` 读为 `String`）；孤儿卡片 NULL `project_name` 的既有行为不在本次改动，见「风险」OUT-OF-SCOPE 条。
- 共享 SELECT 主体（FROM cards LEFT JOIN projects），分页路径追加 `ORDER BY c.created_at DESC LIMIT ? OFFSET ?`，全量路径仅追加 `ORDER BY c.created_at DESC`（统一带 `c.` 别名限定，与现状一致、避免与 `projects.created_at` 歧义）。
- 理由：两条路径的列、JOIN、映射完全一致，抽取后单点维护，降低未来分叉风险。

**决策 3：查询主体下沉为接收 `&Connection` 的内部函数 `list_cards_conn` / `list_all_cards_conn`，公开函数仅负责 `get_connection()` 后委托。**
- 理由：这是本仓库已验证可测的范式（`import.rs` 的测试即对着内存库注入连接）。`list_cards` 公开签名保持不变，新增 `list_all_cards` 公开签名稳定，同时使单元测试能对内存库断言「>100 全量」与「分页上限=100」两个关键行为。
- 备选：用 `#[cfg(test)]` 设置全局 `DB_PATH`（被否：OnceCell 只能设一次，多测试并发共享一个库会相互污染）。

**决策 4：`export_cards_to_excel` 改调 `list_all_cards`，删去 `Pagination`/`page_size: 100000`。**
- `db::list_all_cards` 经 `src/db/mod.rs` 既有的 `pub use cards::*` 自动导出，无需手改 mod.rs。

## 风险 / 权衡

- [全量加载内存占用] 极大项目一次性载入所有 `CardWithProject` 到内存 → 缓解：导出本就需要全部数据写入 Excel，备份路径同样全量加载，量级与现有备份一致；当前数据规模（数百到数千）无压力，不引入分块复杂度。
- [两路径未来再分叉 / 列序漂移] 共享 `map_card_row`/`build_card_where`/SELECT 主体后仍可能被单独改动；列序与映射不一致在类型兼容时（如 qty↔serial 均 INTEGER）编译期无法捕获 → 缓解：抽取为单一来源已最大化收敛 + 注释说明列序须与映射一致 + task 3.2 ④ 的逐列断言作运行期回归锚点。
- [新增内部 `_conn` 函数增加表面积] → 缓解：仅为可测性下沉，公开 API 不变，权衡值得（换来真实测试覆盖）。
- [`build_card_where` 抽取后占位符编号回归] 两条路径拼 `LIMIT/OFFSET` 时占位符须随条件数动态编号 → 缓解：占位符顺序契约写入 tasks 1.4，并由 task 3.5（N=0/1/3 多条件数）测试作为回归锚点。

**OUT-OF-SCOPE / 既有行为（本次不改，明确不在范围）：**
- [LEFT JOIN 孤儿卡片 NULL `project_name`] 现状 `list_cards` 第 2 列读为 `String`，若卡片 `project_id` 悬空则 `row.get::<_,String>(2)?` 报错。曾考虑顺手改 `Option<String>` 加固，但那会改变 `list_cards` 既有可观察行为（孤儿→报错 变为 →空串），违背本次「零行为变更重构」承诺且属需求外的健壮性扩张；且导出路径必按已存在 `project_id` 过滤、`get_connection()` 开 `PRAGMA foreign_keys=ON`、`projects.name` 为 `NOT NULL`，实际不可达。故**保持现状、列为既有隐患，OUT-OF-SCOPE**（若要修应另开变更并配测试）。
- [导出顺序在同 `created_at` 卡片间未定义] 生产 `created_at` 为秒级精度（`format_datetime`），同秒卡片的 `ORDER BY created_at DESC` 相对顺序由 SQLite 实现决定、不稳定。`card-export` spec 只约束导出**行数**=N、不约束行序，故可接受；测试用唯一递增 `created_at` 仅为断言可判定，该「有序」结论不外推到生产。
- [损坏 JSON `metadata` 静默降级为 `None`] 现状 `metadata_str.and_then(|s| serde_json::from_str(&s).ok())` 把解析失败静默置 `None`，导出该行地址缓存列留空且无告警。属既有行为，本次沿用，OUT-OF-SCOPE。
