# add-tenant-foundation 本地验证产物与用户须执行清单（组 D）

本目录是组 D（本地验证，tasks 第 6 节）的验证产物。绝对路径：
`/Users/herbertgao/RustroverProjects/QSL-CardHub/web_query_service/verify/`

## 文件
- `old_schema_fixture.sql` —— 迁移前真实单租户旧结构（逐字取自 `git HEAD:web_query_service/schema.sql`）+ 样例数据。
- `run_6_1.sh` —— 6.1 离线迁移正确性验证（本机 sqlite3），直接对真实迁移 `migrations/0001_tenant_foundation.sql` 跑回归：A 段断言无索引碰撞 / 无报错，B 段跑全部数据/PK/索引/EXPLAIN/seed/占位符/CHECK 断言。
- `run_worker_smoke.sh` —— 6.3/6.4/6.5/6.6/6.7 worker 行为冒烟（`wrangler dev --local` + miniflare D1，自选测试 Key，不触碰真实 secret）。本机已实测 14/14 PASS。

## 组 D 本地已验（实测 PASS，对应复选框已在 tasks.md 勾选）
- 6.3 表驱动命中 200 + auth_fallback 计数恒 0、错误 Key 401、env.API_KEY 置空 401（不放行）、尾随空白 env 表驱动仍命中且兜底 0、兜底路径本身（撤表凭据后 token==trim(env) → 200 default + D1 计数 0→1）。
- 6.4 含违例（qty=0 触发 CHECK）的 /sync：单 `DB.batch` 整体回滚——预存 keep1/keep2 存活、新行 n1/bad 未写，非 200。**注**：本地 miniflare D1 的 batch 原子性为强信号；生产 `--remote` 行为同 Cloudflare 文档（DB.batch 包事务），如需生产级确认按下「6.2/6.4 remote 确认」段在 `--remote` 临时库重跑。
- 6.5 DROP cards 后 /sync → 500 显式报错、worker 不静默重建（确认 4.3 内联 CREATE TABLE 已删）。
- 6.6 `/api/query?callsign=BG1ABC` 仅返回 default（cq1），不返回同呼号的 other 租户卡；`?tenant_id=other` 注入结果与无参完全一致（参数被忽略，无跨租户）。
- 6.7 尾随空白 env.API_KEY 下 /ping 200（trim 生效）。

## 须用户/环境执行（本地验不了或属生产前硬门）

### 6.1 索引命名 blocker —— 已修复（保留以记录历史）
历史问题：`migrations/0001_tenant_foundation.sql` 第 2 部分曾在 `*_new` 表上 `CREATE INDEX` 了与旧表【同名】的索引，
而 SQLite/D1 索引名是**库级全局唯一**；旧同名索引此刻尚未随 `DROP TABLE` 消失 →
`index ... already exists`，整文件在索引创建处即失败、迁移无法完成。碰撞的 6 个索引名：
`idx_projects_created_at` / `idx_cards_project` / `idx_cards_created_at` /
`idx_sf_orders_order_id` / `idx_sf_orders_waybill_no` / `idx_sf_orders_card_id`。
（`idx_cards_tenant_callsign` 是新名、不碰撞。）

**已落地修复**：这些 `CREATE INDEX` 已从「建在 `*_new` 上、DROP 旧表前」改为
「在对应 `DROP TABLE 旧表` + `ALTER TABLE *_new RENAME TO *` 之后、绑最终表名创建」——
旧同名索引随旧表 DROP 自动消失，不再碰撞。

验证：`bash verify/run_6_1.sh` 直接对真实迁移跑回归——A 段断言无 `already exists` / 无报错，
B 段跑通全部 6.1 断言（数据全 default、id 不变、PK (tenant_id,id)、
全索引在、EXPLAIN 命中 idx_cards_tenant_callsign / idx_sf_orders_order_id、占位符自检、CHECK 拒 abc!、
active key_hash 部分唯一）。本机已实测全 PASS。

上生产前建议在一次性 `--remote` 临时库再验收（见下）。

### 6.2 + 6.4 remote 确认（上生产硬门，代理禁连 --remote）
确认当前 wrangler 4.68.1 的 `--file` 全文件失败回滚、`DB.batch` 事务原子在生产 D1 与文档一致。
本机已对 `--local` 跑出强信号（`--file` 中段失败 → 全文件回滚、无半迁移表；`DB.batch` 含违例 → 整体回滚），
但 `--local`(miniflare) ≠ `--remote`(生产 D1)，故生产前由用户在一次性 `--remote` 临时库确认：

```
# 建临时库
wrangler d1 create qsl-mig-verify
# 灌旧结构 + 样例（用本目录 fixture）
wrangler d1 execute qsl-mig-verify --remote --file verify/old_schema_fixture.sql
# 6.2：先用「中段注入失败」的迁移副本跑一次，断言整文件回滚、无 *_new 残留表：
#   （把迁移中段插一条必失败语句，如 INSERT 引用不存在列）
wrangler d1 execute qsl-mig-verify --remote --file <注入失败的迁移副本>
wrangler d1 execute qsl-mig-verify --remote --command "SELECT name FROM sqlite_master WHERE name LIKE '%_new';"  # 须 0 行
# 6.1：跑【已修索引时机】的正式迁移（占位符替换为 sha256(trim(真实 API_KEY)))，断言 run_6_1.sh B 段那组结果
# 6.2 若实测为逐语句提交（非全文件回滚）→ 改影子表（cards_v2 校验后 RENAME 切换），见 tasks 6.2
# 验毕删库
wrangler d1 delete qsl-mig-verify
```

### 部署顺序（钉死）
**迁移必须先于新 worker 部署**：先 `wrangler d1 execute ... --file 0001_... --remote` 完成建表/回填，再部署新 worker 版本。
反序（worker 先上、表未迁）后果是 `resolveTenant` 查无 `tenant_credentials`/`service_counters` 表 → 整站 500
（可用性事故、非安全事故、非裸写）。

### 第 7 节生产迁移与验收（用户在自己终端执行，代理不代跑生产）
7.1 备份(`d1 export`) → 单一所有者校验(含 sync_meta) → 冻结写入 → 离线算
`sha256(trim(API_KEY))` 替换占位符 → `wrangler d1 execute qsl-sync --file 0001_... --remote` →
占位符自检 0 行；7.2 部署 worker 新版本；7.3 验收：现有桌面端 /sync 200 落 default、
**auth_fallback 计数行存在且 count===0**（严格相等+存在性双检，>0 或缺失/不可读判 inconclusive）、
/ping 200、移动端裸域查询字段集不变；7.4 量化撤兜底判据 + 记 callsign_openid_bindings/sf_route_log 二次迁表点。

## 复跑
```
bash verify/run_6_1.sh           # 离线迁移正确性（直接对真实迁移：A 无碰撞 / B 全 PASS）
bash verify/run_worker_smoke.sh  # 6.3-6.7 worker 行为（wrangler dev --local）
```
