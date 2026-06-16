#!/usr/bin/env bash
# 阶段 4-A 离线验证（本机 sqlite3，非 D1）：
#   A) 迁移 0002 正确性——在「旧 callsign_openid_bindings(PK (callsign,openid)、无 tenant_id)」
#      fixture 上执行 migrations/0002_global_table_tenant.sql，断言无 already exists / 无报错、
#      新表结构正确、回填 bh2ro、行数不变、索引在、sf_route_log 未变。
#   B) route-push 跨租户隔离不变量（纯 SQL，强锚）——两租户同一呼号 + 各自订单/绑定，断言：
#      ① 呼号反查由匹配订单派生正确租户；② openid 反查按派生租户过滤只命中本租户 openid
#      （证伪：旧 WHERE callsign=? 无租户会同时命中两租户）；③ 同租户自洽 join 防跨租户错配卡片。
# 用法：bash verify/run_0002_migration.sh
set -uo pipefail
cd "$(dirname "$0")/.."
DIR="$(mktemp -d)"
MIG="migrations/0002_global_table_tenant.sql"
SCHEMA="schema.sql"
FAIL=0
pass() { printf 'PASS  %s\n' "$1"; }
fail() { printf 'FAIL  %s\n' "$1"; FAIL=1; }
# 断言「实际值」等于「期望值」
eq() { # $1=label $2=expected $3=actual
  if [ "$2" = "$3" ]; then pass "$1 (= $3)"; else fail "$1 期望[$2] 实得[$3]"; fi
}

echo "############################################################"
echo "# A) 迁移 0002 在旧 schema 上无碰撞 + 结构/回填正确"
echo "############################################################"
DBA="$DIR/a.db"
# 旧 fixture：迁移前的 callsign_openid_bindings（PK (callsign,openid)、无 tenant_id）+ 旧索引，
# 外加 sf_route_log（验证迁移不碰它）。模拟 0001 之后、0002 之前的生产现状。
sqlite3 "$DBA" <<'SQL'
CREATE TABLE callsign_openid_bindings (
    callsign TEXT NOT NULL,
    openid TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (callsign, openid)
);
CREATE INDEX idx_bindings_callsign ON callsign_openid_bindings(callsign);
INSERT INTO callsign_openid_bindings (callsign, openid, created_at) VALUES
  ('BD1ABC','openid-1','2026-01-01T00:00:00Z'),
  ('BD1ABC','openid-2','2026-01-02T00:00:00Z'),
  ('BG7XYZ','openid-3','2026-01-03T00:00:00Z');
CREATE TABLE sf_route_log (
    id TEXT PRIMARY KEY,
    mailno TEXT, orderid TEXT, op_code TEXT, accept_time TEXT, remark TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_sf_route_mailno_op ON sf_route_log(mailno, op_code, id);
INSERT INTO sf_route_log (id,mailno) VALUES ('rl1','WB-X');
SQL

PRE_ROUTELOG=$(sqlite3 "$DBA" "SELECT sql FROM sqlite_master WHERE type='table' AND name='sf_route_log';")
ERR=$(sqlite3 -bail "$DBA" < "$MIG" 2>&1 || true)
if echo "$ERR" | grep -qi "already exists"; then
  fail "迁移触发索引/表碰撞（already exists）：$ERR"
elif [ -n "$ERR" ]; then
  fail "迁移执行报错：$ERR"
else
  pass "迁移在旧 schema 上执行无报错（无 already exists）"
fi

# 新表含 tenant_id 列
HAS_TENANT=$(sqlite3 "$DBA" "SELECT COUNT(*) FROM pragma_table_info('callsign_openid_bindings') WHERE name='tenant_id';")
eq "新表含 tenant_id 列" "1" "$HAS_TENANT"
# 主键恰为 (tenant_id, callsign, openid)：pk 列按 pk 序号
PK_COLS=$(sqlite3 "$DBA" "SELECT group_concat(name,',') FROM (SELECT name FROM pragma_table_info('callsign_openid_bindings') WHERE pk>0 ORDER BY pk);")
eq "主键 = (tenant_id,callsign,openid)" "tenant_id,callsign,openid" "$PK_COLS"
# 全部存量回填 tenant_id='bh2ro'
NON_BH2RO=$(sqlite3 "$DBA" "SELECT COUNT(*) FROM callsign_openid_bindings WHERE tenant_id<>'bh2ro';")
eq "回填后无非 bh2ro 行" "0" "$NON_BH2RO"
# 行数不变（3）
ROWS=$(sqlite3 "$DBA" "SELECT COUNT(*) FROM callsign_openid_bindings;")
eq "行数不变(=3)" "3" "$ROWS"
# created_at 原样保留（抽一行）
CA=$(sqlite3 "$DBA" "SELECT created_at FROM callsign_openid_bindings WHERE callsign='BD1ABC' AND openid='openid-1';")
eq "created_at 原样保留" "2026-01-01T00:00:00Z" "$CA"
# 索引 idx_bindings_callsign 存在
HAS_IDX=$(sqlite3 "$DBA" "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_bindings_callsign';")
eq "idx_bindings_callsign 存在" "1" "$HAS_IDX"
# 索引为 (tenant_id, callsign COLLATE NOCASE)（带租户维度 + 大小写不敏感，服务 route-push 反查）
IDX_SQL=$(sqlite3 "$DBA" "SELECT lower(sql) FROM sqlite_master WHERE type='index' AND name='idx_bindings_callsign';")
case "$IDX_SQL" in *tenant_id*callsign*nocase*) pass "idx = (tenant_id, callsign COLLATE NOCASE)";; *) fail "idx 非 (tenant_id,callsign NOCASE): $IDX_SQL";; esac
# 回填不撞 PK：原 (callsign,openid) 唯一 → (bh2ro,callsign,openid) 仍唯一，distinct = 行数
DISTINCT_PK=$(sqlite3 "$DBA" "SELECT COUNT(*) FROM (SELECT DISTINCT tenant_id,callsign,openid FROM callsign_openid_bindings);")
eq "无 PK 冲突(distinct=3)" "3" "$DISTINCT_PK"
# sf_route_log 结构未被迁移触碰
POST_ROUTELOG=$(sqlite3 "$DBA" "SELECT sql FROM sqlite_master WHERE type='table' AND name='sf_route_log';")
eq "sf_route_log 结构未变" "$PRE_ROUTELOG" "$POST_ROUTELOG"
HAS_RL_TENANT=$(sqlite3 "$DBA" "SELECT COUNT(*) FROM pragma_table_info('sf_route_log') WHERE name='tenant_id';")
eq "sf_route_log 仍无 tenant_id 列" "0" "$HAS_RL_TENANT"

# 双写一致性：迁移后表结构与 schema.sql 中 callsign_openid_bindings 的列集 + 主键一致
DBSC="$DIR/schema.db"
sqlite3 "$DBSC" < "$SCHEMA"
SC_COLS=$(sqlite3 "$DBSC" "SELECT group_concat(name,',') FROM (SELECT name FROM pragma_table_info('callsign_openid_bindings') ORDER BY cid);")
MIG_COLS=$(sqlite3 "$DBA"  "SELECT group_concat(name,',') FROM (SELECT name FROM pragma_table_info('callsign_openid_bindings') ORDER BY cid);")
eq "schema.sql 双写列集与迁移一致" "$SC_COLS" "$MIG_COLS"
SC_PK=$(sqlite3 "$DBSC" "SELECT group_concat(name,',') FROM (SELECT name FROM pragma_table_info('callsign_openid_bindings') WHERE pk>0 ORDER BY pk);")
eq "schema.sql 双写主键与迁移一致" "$SC_PK" "$PK_COLS"
# 双写索引定义一致（schema 用 IF NOT EXISTS、迁移不用，归一后比较）
SC_IDX=$(sqlite3 "$DBSC" "SELECT lower(sql) FROM sqlite_master WHERE type='index' AND name='idx_bindings_callsign';" | sed 's/if not exists //')
MIG_IDX=$(sqlite3 "$DBA"  "SELECT lower(sql) FROM sqlite_master WHERE type='index' AND name='idx_bindings_callsign';")
eq "schema.sql 双写索引定义与迁移一致" "$SC_IDX" "$MIG_IDX"

echo ""
echo "############################################################"
echo "# B) route-push 跨租户隔离不变量（纯 SQL 强锚）"
echo "############################################################"
DBB="$DIR/b.db"
sqlite3 "$DBB" < "$SCHEMA"
# 两租户同一呼号 BD1ABC；各有一张同 id 'shared-card' 的卡（PK (tenant_id,id) 允许）以测同租户自洽 join；
# 各有订单（order_id/waybill_no 全局唯一）；各有 BD1ABC 的 openid 绑定（不同 openid）。
sqlite3 "$DBB" <<'SQL'
INSERT INTO tenants (tenant_id,name,tier,status) VALUES ('bh2ro','BH2RO',NULL,'active'),('t2','T2',NULL,'active');
INSERT INTO projects (tenant_id,id,name,created_at,updated_at) VALUES
  ('bh2ro','p','P','2026-01-01','2026-01-01'),('t2','p','P','2026-01-01','2026-01-01');
INSERT INTO cards (tenant_id,id,project_id,callsign,qty,status,created_at,updated_at) VALUES
  ('bh2ro','shared-card','p','BD1ABC',1,'pending','2026-01-01','2026-01-01'),
  ('t2','shared-card','p','BD1ABC',1,'pending','2026-01-01','2026-01-01');
INSERT INTO sf_orders (tenant_id,id,order_id,waybill_no,card_id,status,sender_info,recipient_info,created_at,updated_at) VALUES
  ('bh2ro','o1','OID-BH','WB-BH','shared-card','pending','{}','{}','2026-01-01','2026-01-01'),
  ('t2','o2','OID-T2','WB-T2','shared-card','pending','{}','{}','2026-01-01','2026-01-01');
INSERT INTO callsign_openid_bindings (tenant_id,callsign,openid,created_at) VALUES
  ('bh2ro','BD1ABC','openid-bh','2026-01-01'),
  ('t2','BD1ABC','openid-t2','2026-01-01');
SQL

# ① 呼号反查由匹配订单派生租户（worker 实查 SQL 逐字一致：JOIN o.tenant_id=c.tenant_id AND o.card_id=c.id）
Q_ORDER='SELECT c.callsign||"|"||o.tenant_id FROM sf_orders o JOIN cards c ON o.tenant_id=c.tenant_id AND o.card_id=c.id WHERE o.order_id=? LIMIT 1'
D_BH=$(sqlite3 "$DBB" "$(printf '%s' "$Q_ORDER" | sed "s/?/'OID-BH'/")")
eq "order OID-BH 派生 callsign|tenant" "BD1ABC|bh2ro" "$D_BH"
D_T2=$(sqlite3 "$DBB" "$(printf '%s' "$Q_ORDER" | sed "s/?/'OID-T2'/")")
eq "order OID-T2 派生 callsign|tenant" "BD1ABC|t2" "$D_T2"
# waybill 分支
Q_WB='SELECT c.callsign||"|"||o.tenant_id FROM sf_orders o JOIN cards c ON o.tenant_id=c.tenant_id AND o.card_id=c.id WHERE o.waybill_no=? LIMIT 1'
W_BH=$(sqlite3 "$DBB" "$(printf '%s' "$Q_WB" | sed "s/?/'WB-BH'/")")
eq "waybill WB-BH 派生 callsign|tenant" "BD1ABC|bh2ro" "$W_BH"
W_T2=$(sqlite3 "$DBB" "$(printf '%s' "$Q_WB" | sed "s/?/'WB-T2'/")")
eq "waybill WB-T2 派生 callsign|tenant" "BD1ABC|t2" "$W_T2"

# ② openid 反查按派生租户过滤（worker 实查：WHERE tenant_id=? AND callsign=?）→ 各租户只命中自己 openid
OB=$(sqlite3 "$DBB" "SELECT group_concat(openid,',') FROM callsign_openid_bindings WHERE tenant_id='bh2ro' AND callsign='BD1ABC';")
eq "派生 bh2ro 只命中 bh2ro openid" "openid-bh" "$OB"
OT=$(sqlite3 "$DBB" "SELECT group_concat(openid,',') FROM callsign_openid_bindings WHERE tenant_id='t2' AND callsign='BD1ABC';")
eq "派生 t2 只命中 t2 openid" "openid-t2" "$OT"
# 证伪：旧 WHERE callsign=? 无租户维度会同时命中两租户（说明不修就跨租户泄漏）
OLD=$(sqlite3 "$DBB" "SELECT COUNT(*) FROM callsign_openid_bindings WHERE callsign='BD1ABC';")
eq "证伪:旧无租户反查会命中 2 条(跨租户泄漏)" "2" "$OLD"
# ②' callsign 大小写不敏感（worker 反查带 COLLATE NOCASE）：绑定存大写 BD1ABC，小写查仍命中
NOCASE=$(sqlite3 "$DBB" "SELECT group_concat(openid,',') FROM callsign_openid_bindings WHERE tenant_id='bh2ro' AND callsign='bd1abc' COLLATE NOCASE;")
eq "NOCASE 反查:小写 bd1abc 命中大写绑定" "openid-bh" "$NOCASE"
# 对照:不带 NOCASE 的 BINARY 比较小写查不命中（证 NOCASE 是必要的，否则大小写偏差静默漏推）
BINARY=$(sqlite3 "$DBB" "SELECT COUNT(*) FROM callsign_openid_bindings WHERE tenant_id='bh2ro' AND callsign='bd1abc';")
eq "证伪:BINARY 比较小写查 0 命中(漏推根因)" "0" "$BINARY"

# ④ 跨租户单号歧义 fail-closed：两租户共用同一 order_id 'OID-DUP'，派生命中 2 个 distinct 租户 → worker 跳过推送
sqlite3 "$DBB" <<'SQL'
INSERT INTO sf_orders (tenant_id,id,order_id,waybill_no,card_id,status,sender_info,recipient_info,created_at,updated_at) VALUES
  ('bh2ro','od1','OID-DUP','WB-DUP-A','shared-card','pending','{}','{}','2026-01-01','2026-01-01'),
  ('t2','od2','OID-DUP','WB-DUP-B','shared-card','pending','{}','{}','2026-01-01','2026-01-01');
SQL
DUP=$(sqlite3 "$DBB" "SELECT COUNT(DISTINCT o.tenant_id) FROM sf_orders o JOIN cards c ON o.tenant_id=c.tenant_id AND o.card_id=c.id WHERE o.order_id='OID-DUP';")
eq "跨租户同 order_id 派生命中 2 个租户(worker 应 fail-closed 跳过)" "2" "$DUP"
# 对照:正常唯一单号 OID-BH 仍只命中 1 个租户(单租户路径不受 fail-closed 影响)
UNIQ=$(sqlite3 "$DBB" "SELECT COUNT(DISTINCT o.tenant_id) FROM sf_orders o JOIN cards c ON o.tenant_id=c.tenant_id AND o.card_id=c.id WHERE o.order_id='OID-BH';")
eq "唯一单号 OID-BH 仍只命中 1 个租户(正常推送不受影响)" "1" "$UNIQ"
# ④' 两键互相矛盾 fail-closed：同一 push 的 order_id 仅在 bh2ro、waybill 仅在 t2 → worker 合并两键候选行
#    （UNION ALL）distinct 租户=2 → 跳过推送（堵 order_id 歧义后 fallthrough 到 waybill 推错租户）。
sqlite3 "$DBB" <<'SQL'
INSERT INTO sf_orders (tenant_id,id,order_id,waybill_no,card_id,status,sender_info,recipient_info,created_at,updated_at) VALUES
  ('bh2ro','ox1','OID-XKEY','WB-ONLYBH','shared-card','pending','{}','{}','2026-01-01','2026-01-01'),
  ('t2','ox2','OID-ONLYT2','WB-XKEY','shared-card','pending','{}','{}','2026-01-01','2026-01-01');
SQL
XKEY=$(sqlite3 "$DBB" "SELECT COUNT(DISTINCT tenant_id) FROM (
  SELECT o.tenant_id FROM sf_orders o JOIN cards c ON o.tenant_id=c.tenant_id AND o.card_id=c.id WHERE o.order_id='OID-XKEY'
  UNION ALL
  SELECT o.tenant_id FROM sf_orders o JOIN cards c ON o.tenant_id=c.tenant_id AND o.card_id=c.id WHERE o.waybill_no='WB-XKEY');")
eq "两键矛盾(order_id→bh2ro/waybill→t2)合并命中 2 租户(worker fail-closed)" "2" "$XKEY"

# ③ 同租户自洽 join 防跨租户错配卡片：bh2ro 订单引 'shared-card'，必须只 join 到 bh2ro 的卡、非 t2 的。
#    若漏 o.tenant_id=c.tenant_id，则 'shared-card' 在两租户都存在 → 笛卡尔取到错租户卡。这里断言只 1 行且 tenant=bh2ro。
JOINCNT=$(sqlite3 "$DBB" "SELECT COUNT(*) FROM sf_orders o JOIN cards c ON o.tenant_id=c.tenant_id AND o.card_id=c.id WHERE o.order_id='OID-BH';")
eq "同租户自洽 join 只命中 1 张卡" "1" "$JOINCNT"

echo ""
if [ "$FAIL" -eq 0 ]; then echo "ALL PASS"; else echo "SOME FAIL"; fi
rm -rf "$DIR"
exit "$FAIL"
