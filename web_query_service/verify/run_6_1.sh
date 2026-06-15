#!/usr/bin/env bash
# 6.1 离线迁移正确性验证（本机 sqlite3，非 D1）
# 用法：bash verify/run_6_1.sh
#
# 历史背景：早期 migrations/0001_tenant_foundation.sql 在真实旧 schema 上【必失败】——
#   part 2 在 *_new 表上 CREATE 了与旧表同名的索引（idx_projects_created_at /
#   idx_cards_project / idx_cards_created_at / idx_sf_orders_order_id /
#   idx_sf_orders_waybill_no / idx_sf_orders_card_id），而 SQLite 索引名是【库级全局唯一】，
#   旧同名索引此时尚未随 DROP TABLE 消失 → "index ... already exists"，迁移整文件无法完成。
#   已修复：把这些索引的 CREATE 移到对应 DROP 旧表 + ALTER ... RENAME 之后（绑最终表名），
#   旧同名索引随旧表 DROP 自动消失，不再碰撞。
# 本脚本对【真实迁移文件 migrations/0001_tenant_foundation.sql】做可复跑回归：
#   A) 断言迁移在真实旧 schema 上执行【无 already exists 碰撞、无任何报错】（修复落实证据）；
#   B) 跑全部 6.1 数据/PK/索引/EXPLAIN/seed/占位符/CHECK 断言，证明迁移逻辑正确。
set -uo pipefail
cd "$(dirname "$0")/.."
DIR="$(mktemp -d)"
MIG="migrations/0001_tenant_foundation.sql"
FIX="verify/old_schema_fixture.sql"
TESTHASH="0000000000000000000000000000000000000000000000000000000000000abc"
FAIL=0
pass() { printf 'PASS  %s\n' "$1"; }
fail() { printf 'FAIL  %s\n' "$1"; FAIL=1; }

# 真实迁移文件（替换占位符 hash），供 A/B 两段复用
sed "s/<占位:离线 sha256(trim(API_KEY))>/$TESTHASH/" "$MIG" > "$DIR/migration.sql"

echo "############################################################"
echo "# A) 真实迁移在旧 schema 上无索引碰撞 / 无报错"
echo "############################################################"
DBA="$DIR/a.db"
sqlite3 "$DBA" < "$FIX"
ERR=$(sqlite3 -bail "$DBA" < "$DIR/migration.sql" 2>&1 || true)
if echo "$ERR" | grep -qi "already exists"; then
  fail "真实迁移仍触发索引碰撞（index ... already exists）：$ERR"
elif [ -n "$ERR" ]; then
  fail "真实迁移执行报错：$ERR"
else
  pass "真实迁移在旧 schema 上执行无报错（索引碰撞 blocker 已修复）"
fi

echo
echo "############################################################"
echo "# B) 真实迁移跑全部 6.1 断言"
echo "############################################################"
DB="$DIR/b.db"
sqlite3 "$DB" < "$FIX"
PERR=$(sqlite3 -bail "$DB" < "$DIR/migration.sql" 2>&1 || true)
if [ -n "$PERR" ]; then fail "真实迁移执行报错：$PERR"; else pass "真实迁移无错误执行完成"; fi

echo
echo "== 断言 1：业务表无 client_id 列 =="
for t in projects cards sf_senders sf_orders app_settings; do
  if sqlite3 "$DB" "SELECT 1 FROM pragma_table_info('$t') WHERE name='client_id';" | grep -q 1; then
    fail "$t 仍含 client_id 列"
  else
    pass "$t 无 client_id 列"
  fi
done
if sqlite3 "$DB" "SELECT 1 FROM pragma_table_info('sync_meta') WHERE name='client_id';" | grep -q 1; then
  fail "sync_meta 含裸 client_id 列"
else
  pass "sync_meta 无裸 client_id 列（last_client_id 溯源列单独验）"
fi

echo
echo "== 断言 2：业务表 PK 为 (tenant_id, id)（app_settings 为 (tenant_id, key)）=="
chk_pk() {
  local t="$1" want="$2" got
  got=$(sqlite3 "$DB" "SELECT group_concat(name,',') FROM (SELECT name FROM pragma_table_info('$t') WHERE pk>0 ORDER BY pk);")
  [ "$got" = "$want" ] && pass "$t PK = ($got)" || fail "$t PK = ($got)，期望 ($want)"
}
chk_pk projects "tenant_id,id"
chk_pk cards "tenant_id,id"
chk_pk sf_senders "tenant_id,id"
chk_pk sf_orders "tenant_id,id"
chk_pk app_settings "tenant_id,key"
chk_pk sync_meta "tenant_id"

echo
echo "== 断言 3：数据全部 tenant_id='default' 且行数保全 =="
for t in projects cards sf_senders sf_orders app_settings; do
  total=$(sqlite3 "$DB" "SELECT COUNT(*) FROM $t;")
  deflt=$(sqlite3 "$DB" "SELECT COUNT(*) FROM $t WHERE tenant_id='default';")
  if [ "$total" = "$deflt" ] && [ "$total" -gt 0 ]; then pass "$t: $total 行全为 default"; else fail "$t: total=$total default=$deflt"; fi
done

echo
echo "== 断言 4：id 不变 =="
ids=$(sqlite3 "$DB" "SELECT group_concat(id) FROM (SELECT id FROM cards ORDER BY id);")
[ "$ids" = "c1,c2,c3" ] && pass "cards id 保全: $ids" || fail "cards id = $ids，期望 c1,c2,c3"
pids=$(sqlite3 "$DB" "SELECT group_concat(id) FROM (SELECT id FROM projects ORDER BY id);")
[ "$pids" = "p1,p2" ] && pass "projects id 保全: $pids" || fail "projects id = $pids"
akeys=$(sqlite3 "$DB" "SELECT group_concat(key) FROM (SELECT key FROM app_settings ORDER BY key);")
[ "$akeys" = "lang,theme" ] && pass "app_settings key 保全: $akeys" || fail "app_settings key = $akeys"

echo
echo "== 断言 5：全部业务索引存在 / 旧 client 索引消失 =="
need_idx="idx_projects_created_at idx_cards_tenant_callsign idx_cards_project idx_cards_created_at idx_sf_orders_order_id idx_sf_orders_waybill_no idx_sf_orders_card_id idx_tenant_credentials_active_key_hash"
have_idx=$(sqlite3 "$DB" "SELECT name FROM sqlite_master WHERE type='index';")
for i in $need_idx; do
  echo "$have_idx" | grep -qx "$i" && pass "索引存在: $i" || fail "索引缺失: $i"
done
for i in idx_cards_client idx_projects_client idx_sf_orders_client idx_sf_senders_client idx_cards_callsign idx_app_settings_client; do
  echo "$have_idx" | grep -qx "$i" && fail "旧索引残留: $i" || pass "旧索引已消失: $i"
done

echo
echo "== 断言 6：EXPLAIN QUERY PLAN 命中目标索引 =="
qp_cards=$(sqlite3 "$DB" "EXPLAIN QUERY PLAN SELECT c.id FROM cards c WHERE c.tenant_id='default' AND c.callsign='BG1ABC' COLLATE NOCASE ORDER BY c.created_at DESC;")
echo "$qp_cards" | grep -q "idx_cards_tenant_callsign" && pass "cards 呼号查询命中 idx_cards_tenant_callsign" || { printf '  %s\n' "$qp_cards"; fail "cards 呼号查询未命中 idx_cards_tenant_callsign"; }
qp_sf=$(sqlite3 "$DB" "EXPLAIN QUERY PLAN SELECT c.callsign FROM sf_orders o JOIN cards c ON o.tenant_id=c.tenant_id AND o.card_id=c.id WHERE o.tenant_id='default' AND o.order_id='ORDER-001' LIMIT 1;")
echo "$qp_sf" | grep -q "idx_sf_orders_order_id" && pass "route-push 命中 idx_sf_orders_order_id" || { printf '  %s\n' "$qp_sf"; fail "route-push 未命中 idx_sf_orders_order_id"; }
qp_wb=$(sqlite3 "$DB" "EXPLAIN QUERY PLAN SELECT c.callsign FROM sf_orders o JOIN cards c ON o.tenant_id=c.tenant_id AND o.card_id=c.id WHERE o.tenant_id='default' AND o.waybill_no='SF0001' LIMIT 1;")
echo "$qp_wb" | grep -q "idx_sf_orders_waybill_no" && pass "route-push 命中 idx_sf_orders_waybill_no" || { printf '  %s\n' "$qp_wb"; fail "route-push 未命中 idx_sf_orders_waybill_no"; }

echo
echo "== 断言 7：sync_meta 单行 + 确定性 last_client_id =="
sm_rows=$(sqlite3 "$DB" "SELECT COUNT(*) FROM sync_meta;")
[ "$sm_rows" = "1" ] && pass "sync_meta 单行" || fail "sync_meta 行数=$sm_rows，期望 1"
sm_tid=$(sqlite3 "$DB" "SELECT tenant_id FROM sync_meta;")
[ "$sm_tid" = "default" ] && pass "sync_meta tenant_id=default" || fail "sync_meta tenant_id=$sm_tid"
sm_lci=$(sqlite3 "$DB" "SELECT last_client_id FROM sync_meta;")
[ "$sm_lci" = "cli-A-new" ] && pass "sync_meta last_client_id=cli-A-new（确定性取最新 received_at）" || fail "sync_meta last_client_id=$sm_lci，期望 cli-A-new"
sm_sv=$(sqlite3 "$DB" "SELECT server_version FROM sync_meta;")
[ "$sm_sv" = "0" ] && pass "sync_meta server_version=0" || fail "sync_meta server_version=$sm_sv"

echo
echo "== 断言 8：seed —— default 租户 + 凭据 + auth_fallback 计数行 =="
t_def=$(sqlite3 "$DB" "SELECT COUNT(*) FROM tenants WHERE tenant_id='default' AND status='active';")
[ "$t_def" = "1" ] && pass "tenants 含 active default" || fail "tenants default 缺失"
cred=$(sqlite3 "$DB" "SELECT COUNT(*) FROM tenant_credentials WHERE tenant_id='default' AND status='active';")
[ "$cred" = "1" ] && pass "tenant_credentials 含 active default 凭据" || fail "default 凭据缺失"
afb=$(sqlite3 "$DB" "SELECT count FROM service_counters WHERE name='auth_fallback';")
[ "$afb" = "0" ] && pass "service_counters auth_fallback 行存在且 count=0" || fail "auth_fallback count=$afb（期望 0）"

echo
echo "== 断言 9：占位符自检（已替换 → 末尾 SELECT 0 行）=="
residue=$(sqlite3 "$DB" "SELECT COUNT(*) FROM tenant_credentials WHERE key_hash LIKE '<占位%';")
[ "$residue" = "0" ] && pass "占位符自检：无残留" || fail "占位符残留 $residue 行"

echo
echo "== 断言 10：tenant_id CHECK 拒非法 slug =="
if sqlite3 "$DB" "INSERT INTO tenants (tenant_id, status) VALUES ('abc!', 'active');" 2>/dev/null; then
  fail "tenant_id CHECK 误放行 'abc!'（否定字符类失效）"
else
  pass "tenant_id CHECK 正确拒绝 'abc!'"
fi
if sqlite3 "$DB" "INSERT INTO tenants (tenant_id, status) VALUES ('abc-123', 'active');" 2>/dev/null; then
  pass "tenant_id CHECK 放行合法 'abc-123'"
else
  fail "tenant_id CHECK 误拒合法 'abc-123'"
fi

echo
echo "== 断言 11：同 key_hash active 唯一（部分唯一索引）=="
sqlite3 "$DB" "INSERT INTO tenant_credentials (id, tenant_id, key_hash, status) VALUES ('k2','default','dup_hash','active');" 2>/dev/null
if sqlite3 "$DB" "INSERT INTO tenant_credentials (id, tenant_id, key_hash, status) VALUES ('k3','other','dup_hash','active');" 2>/dev/null; then
  fail "部分唯一索引误放行：同 active key_hash 登记到两个租户"
else
  pass "部分唯一索引拒绝：同 active key_hash 第二次登记被拒"
fi

echo
echo "== 断言 12（占位符残留场景）：未替换占位符 → 末尾自检命中 =="
# 跑【真实迁移、保留占位符字面量】（不替换 <占位...>），末尾自检 SELECT 应返回 1 行残留
DBC="$DIR/c.db"
sqlite3 "$DBC" < "$FIX"
sqlite3 "$DBC" < "$MIG" >/dev/null 2>&1 || true
RES=$(sqlite3 "$DBC" "SELECT COUNT(*) FROM tenant_credentials WHERE key_hash LIKE '<占位%';")
[ "$RES" = "1" ] && pass "未替换占位符时自检命中残留（1 行 → 视为迁移失败须中止排查）" || fail "占位符自检未命中残留（实际 $RES 行）"

echo
rm -rf "$DIR"
if [ "$FAIL" = "0" ]; then echo "==== 6.1 断言全 PASS（真实迁移 migrations/0001_tenant_foundation.sql 索引碰撞 blocker 已修复，逻辑正确）===="; exit 0; else echo "==== 6.1 存在 FAIL ===="; exit 1; fi
