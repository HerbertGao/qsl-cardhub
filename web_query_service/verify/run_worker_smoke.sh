#!/usr/bin/env bash
# Worker 行为冒烟（6.3 / 6.4 / 6.5 / 6.6 / 6.7）—— 本地 wrangler dev --local（miniflare D1）。
# 已在组 D 验证中实测全 PASS（本机 wrangler 4.68.1 / node v22 / sqlite 3.51）。
# 用法：bash verify/run_worker_smoke.sh
#
# 重要前提与边界：
#   - 用【自选测试 Key】test-api-key-12345，经 --var API_KEY 注入，不触碰真实 secret；
#     实测 --var 优先级高于 .dev.vars，故测试隔离成立（/ping 用测试 Key 即 200）。
#   - 本地 miniflare D1 的 DB.batch() 原子性 / --file 全文件回滚为【强信号非等价】：
#     生产 --remote 行为以 Cloudflare 文档为准，6.2 / 6.4 的生产确认仍须 --remote（见 CHECKLIST）。
#   - 本地 D1 需先按 schema.sql 建表并 seed（本脚本自动做），因为 worker 已删除内联 CREATE TABLE。
set -uo pipefail
cd "$(dirname "$0")/.."
PORT=8799
B="http://127.0.0.1:$PORT"
TEST_KEY="test-api-key-12345"
HASH=$(node -e 'process.stdout.write(require("crypto").createHash("sha256").update((process.argv[1]||"").trim()).digest("hex"))' "$TEST_KEY")
PASS=0; FAIL=0
ok(){ printf 'PASS  %s\n' "$1"; PASS=$((PASS+1)); }
no(){ printf 'FAIL  %s\n' "$1"; FAIL=$((FAIL+1)); }
d1(){ npx wrangler d1 execute qsl-sync --local "$@" 2>/dev/null; }
d1json(){ npx wrangler d1 execute qsl-sync --local --json --command "$1" 2>/dev/null | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>process.stdout.write(JSON.stringify(JSON.parse(s)[0].results)))'; }
stop(){ pkill -f "wrangler dev --local --port $PORT" 2>/dev/null; sleep 2; }
start(){ # $1 = API_KEY value
  stop
  nohup npx wrangler dev --local --port "$PORT" --var API_KEY:"$1" --var CLIENT_SIGN_KEY:"" --var CAPTCHA_SECRET:"" >/tmp/qsl_smoke_dev.log 2>&1 &
  for i in $(seq 1 25); do curl -s -o /dev/null -m 2 "$B/api/config" 2>/dev/null && return 0; sleep 2; done
  echo "server failed to start; see /tmp/qsl_smoke_dev.log"; exit 1
}
code(){ curl -s -m 8 -o /dev/null -w '%{http_code}' "$@"; }

echo "== reset + seed local D1 =="
# 幂等清场：杀掉上次中断残留的 wrangler dev（避免与 seed 争用本地 D1 的 WAL），
# 并清空纯本地 miniflare D1 状态，使 d1 execute --local / wrangler dev --local 在干净库上重建。
# .wrangler/state 仅本地 miniflare 持久化（已 gitignore），删之不影响生产/远端。
pkill -f "wrangler dev" 2>/dev/null || true
sleep 1
rm -rf .wrangler/state/v3/d1 2>/dev/null || true
d1 --command "
DROP TABLE IF EXISTS projects; DROP TABLE IF EXISTS cards; DROP TABLE IF EXISTS sf_senders;
DROP TABLE IF EXISTS sf_orders; DROP TABLE IF EXISTS app_settings; DROP TABLE IF EXISTS sync_meta;
DROP TABLE IF EXISTS tenants; DROP TABLE IF EXISTS tenant_credentials; DROP TABLE IF EXISTS tenant_routes;
DROP TABLE IF EXISTS service_counters;" >/dev/null
d1 --file ./schema.sql >/dev/null
d1 --command "
INSERT INTO tenants (tenant_id,name,tier,status) VALUES ('bh2ro','BH2RO',NULL,'active');
INSERT INTO tenant_credentials (id,tenant_id,scope,key_hash,status) VALUES ('bh2ro-key','bh2ro','sync','$HASH','active');
INSERT INTO service_counters (name,count) VALUES ('auth_fallback',0);
INSERT INTO projects (tenant_id,id,name,created_at,updated_at) VALUES ('bh2ro','p1','项目一','t','t');
INSERT INTO cards (tenant_id,id,project_id,callsign,qty,status,created_at,updated_at) VALUES ('bh2ro','cq1','p1','BG1ABC',5,'pending','t','t');
INSERT INTO cards (tenant_id,id,project_id,callsign,qty,status,created_at,updated_at) VALUES ('other','cqOther','pO','BG1ABC',9,'pending','t','t');" >/dev/null

echo; echo "########## 6.3 表驱动命中 + 错误 Key + 兜底 ##########"
start "$TEST_KEY"
[ "$(code -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d '{"client_id":"cx","data":{"cards":[{"id":"A","project_id":"p1","callsign":"BH9","qty":2}]}}' "$B/sync")" = 200 ] && ok "表驱动命中 /sync 200" || no "表驱动 /sync 非 200"
[ "$(d1json "SELECT count FROM service_counters WHERE name='auth_fallback'" )" = '[{"count":0}]' ] && ok "表驱动命中：auth_fallback 仍 0" || no "表驱动命中后 auth_fallback != 0"
[ "$(code -H "Authorization: Bearer WRONG" -H 'Content-Type: application/json' -d '{"client_id":"c","data":{}}' "$B/sync")" = 401 ] && ok "错误 Key /sync 401" || no "错误 Key 非 401"

echo; echo "########## 6.3 env.API_KEY 空 -> 401（不放行） ##########"
d1 --command "UPDATE tenant_credentials SET status='revoked' WHERE id='bh2ro-key';" >/dev/null
start ""
[ "$(code -H "Authorization: Bearer anything" "$B/ping")" = 401 ] && ok "空 env /ping 401" || no "空 env /ping 非 401"
[ "$(code -H "Authorization: Bearer anything" -H 'Content-Type: application/json' -d '{"client_id":"c","data":{}}' "$B/sync")" = 401 ] && ok "空 env /sync 401（无裸写）" || no "空 env /sync 非 401"

echo; echo "########## 6.3 兜底命中：表 miss(revoked) + 非空 env 直比 -> 200 bh2ro + 计数 0->1 ##########"
# 凭据仍 revoked（接上段表驱动 miss），env.API_KEY 非空且 ==测试 Key → trim 相等走兜底；先把计数复位 0
d1 --command "UPDATE service_counters SET count=0 WHERE name='auth_fallback';" >/dev/null
start "$TEST_KEY"
[ "$(code -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d '{"client_id":"cf1","data":{"cards":[{"id":"fb1","project_id":"p1","callsign":"BG9FB","qty":1}]}}' "$B/sync")" = 200 ] && ok "兜底命中 /sync 200（数据落 bh2ro）" || no "兜底命中 /sync 非 200"
[ "$(d1json "SELECT count FROM service_counters WHERE name='auth_fallback'" )" = '[{"count":1}]' ] && ok "兜底命中：auth_fallback 0->1" || no "兜底命中后 auth_fallback != 1"

echo; echo "########## 6.3 兜底 fail-closed：计数行缺失 -> /sync 500（写失败不静默吞）##########"
d1 --command "DELETE FROM service_counters WHERE name='auth_fallback';" >/dev/null
[ "$(code -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d '{"client_id":"cf2","data":{}}' "$B/sync")" = 500 ] && ok "兜底计数缺失 fail-closed /sync 500" || no "兜底计数缺失未返 500"
# 复位：重建计数行 + 复活凭据，供后续段落使用
d1 --command "INSERT INTO service_counters (name,count) VALUES ('auth_fallback',0);" >/dev/null

echo; echo "########## 6.7 + 6.3 对抗：尾随空白 env.API_KEY ##########"
d1 --command "UPDATE tenant_credentials SET status='active' WHERE id='bh2ro-key'; UPDATE service_counters SET count=0 WHERE name='auth_fallback';" >/dev/null
start "$TEST_KEY  "   # 尾随两个空格
[ "$(code -H "Authorization: Bearer $TEST_KEY" "$B/ping")" = 200 ] && ok "6.7 尾随空白 env /ping 200（trim 生效）" || no "6.7 /ping 非 200"
[ "$(code -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d '{"client_id":"c","data":{}}' "$B/sync")" = 200 ] && ok "6.3 对抗：尾随空白 env 表驱动仍命中 200" || no "6.3 对抗 /sync 非 200"
[ "$(d1json "SELECT count FROM service_counters WHERE name='auth_fallback'" )" = '[{"count":0}]' ] && ok "6.3 对抗：兜底计数仍 0" || no "6.3 对抗：兜底计数 != 0"

echo; echo "########## 鉴权绕过回归：空 Bearer + 空hash凭据 -> 401（worker 空key守卫）##########"
# 模拟「误把 sha256('') seed 成 active 凭据」：插入空串 hash 作 bh2ro 的 active 凭据。
# 不带 Authorization header 发 /sync：getBearerToken 返 null -> resolveTenant trimmedKey='' 早返 null -> 必须 401。
EMPTY_HASH="e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
d1 --command "INSERT INTO tenant_credentials (id,tenant_id,scope,key_hash,status) VALUES ('empty-hash-cred','bh2ro','sync','$EMPTY_HASH','active');" >/dev/null
[ "$(code -H 'Content-Type: application/json' -d '{"client_id":"cbypass","data":{}}' "$B/sync")" = 401 ] && ok "空 Bearer + 空hash凭据 /sync 401（空key守卫生效）" || no "空 Bearer 命中空hash凭据未返 401"
# 清理该条凭据，避免污染后续段落
d1 --command "DELETE FROM tenant_credentials WHERE id='empty-hash-cred';" >/dev/null

echo; echo "########## 6.6 读取注入 bh2ro + tenant_id 注入被忽略 ##########"
# 前序 /sync 全量覆盖会清掉 bh2ro 卡片，6.6 前重新 seed 查询样例（test-only 状态复位）
d1 --command "INSERT OR REPLACE INTO cards (tenant_id,id,project_id,callsign,qty,status,created_at,updated_at) VALUES ('bh2ro','cq1','p1','BG1ABC',5,'pending','t','t'); INSERT OR REPLACE INTO cards (tenant_id,id,project_id,callsign,qty,status,created_at,updated_at) VALUES ('other','cqOther','pO','BG1ABC',9,'pending','t','t'); INSERT OR REPLACE INTO projects (tenant_id,id,name,created_at,updated_at) VALUES ('bh2ro','p1','项目一','t','t');" >/dev/null
R1=$(curl -s -m 8 "$B/api/query?callsign=BG1ABC")
R2=$(curl -s -m 8 "$B/api/query?callsign=BG1ABC&tenant_id=other")
echo "$R1" | grep -q '"id":"cq1"' && ! echo "$R1" | grep -q 'cqOther' && ok "6.6 query 仅返回 bh2ro（cq1），无 other 泄漏" || no "6.6 query 结果异常: $R1"
[ "$R1" = "$R2" ] && ok "6.6 ?tenant_id=other 被忽略（结果与无参一致）" || no "6.6 tenant_id 注入改变了结果"

echo; echo "########## 6.3 client_id 超长截断 ≤128（index.js:304 slice(0,128)）##########"
LONGID=$(printf 'x%.0s' {1..200})
CCLID=$(code -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d "{\"client_id\":\"$LONGID\",\"data\":{\"cards\":[{\"id\":\"clid1\",\"project_id\":\"p1\",\"callsign\":\"BCLID\",\"qty\":1}]}}" "$B/sync")
[ "$CCLID" = 200 ] && ok "client_id 超长 /sync 200" || no "client_id 超长 /sync 非 200 ($CCLID)"
[ "$(d1json "SELECT length(last_client_id) AS n FROM sync_meta WHERE tenant_id='bh2ro'")" = '[{"n":128}]' ] && ok "client_id 截断：last_client_id 长度 == 128" || no "client_id 截断：last_client_id 长度 != 128"

echo; echo "########## app_settings 按租户全量替换 round-trip（cloud-backend-api「app_settings 表结构」）##########"
[ "$(code -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d '{"client_id":"as1","data":{"app_settings":[{"key":"lang","value":"zh"},{"key":"theme","value":"dark"}]}}' "$B/sync")" = 200 ] && ok "app_settings 首轮 /sync 200" || no "app_settings 首轮 /sync 非 200"
[ "$(d1json "SELECT count(*) AS n FROM app_settings WHERE tenant_id='bh2ro' AND key IN ('lang','theme')")" = '[{"n":2}]' ] && ok "app_settings 首轮：lang+theme 落库（tenant_id=bh2ro）" || no "app_settings 首轮：键值未按租户落库"
[ "$(code -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d '{"client_id":"as2","data":{"app_settings":[{"key":"lang","value":"en"}]}}' "$B/sync")" = 200 ] && ok "app_settings 次轮 /sync 200" || no "app_settings 次轮 /sync 非 200"
[ "$(d1json "SELECT count(*) AS n FROM app_settings WHERE tenant_id='bh2ro'")" = '[{"n":1}]' ] && ok "app_settings 全量替换：旧 theme 被清、仅剩 1 行" || no "app_settings 次轮残留旧键"
[ "$(d1json "SELECT value AS v FROM app_settings WHERE tenant_id='bh2ro' AND key='lang'")" = '[{"v":"en"}]' ] && ok "app_settings 全量替换：lang=en（DELETE+INSERT 生效）" || no "app_settings lang 未更新为 en"

echo; echo "########## 6.4 单 batch 回滚 ##########"
d1 --command "DELETE FROM cards WHERE tenant_id='bh2ro'; INSERT INTO cards (tenant_id,id,project_id,callsign,qty,status,created_at,updated_at) VALUES ('bh2ro','keep1','p1','BK1',1,'pending','t','t'),('bh2ro','keep2','p1','BK2',2,'pending','t','t');" >/dev/null
C64=$(code -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d '{"client_id":"cf","data":{"cards":[{"id":"n1","project_id":"p1","callsign":"BN1","qty":3},{"id":"bad","project_id":"p1","callsign":"BBAD","qty":0}]}}' "$B/sync")
[ "$C64" = 500 ] && ok "6.4 含违例 batch /sync 500 ($C64)" || no "6.4 违例 batch 未返 500 ($C64)"
IDS=$(d1json "SELECT id FROM cards WHERE tenant_id='bh2ro' ORDER BY id")
echo "$IDS" | grep -q keep1 && echo "$IDS" | grep -q keep2 && ! echo "$IDS" | grep -q '"n1"' && ! echo "$IDS" | grep -q '"bad"' && ok "6.4 回滚完好：keep1/keep2 存活、n1/bad 未写" || no "6.4 回滚异常: $IDS"

echo; echo "########## 6.5 DROP 表 -> /sync 显式报错、不静默重建 ##########"
d1 --command "DROP TABLE cards;" >/dev/null
C65=$(code -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d '{"client_id":"cd","data":{"cards":[{"id":"x","project_id":"p1","callsign":"BZ","qty":1}]}}' "$B/sync")
[ "$C65" = 500 ] && ok "6.5 缺表 /sync 500 ($C65)" || no "6.5 缺表未返 500 ($C65)"
PRES=$(d1json "SELECT name FROM sqlite_master WHERE type='table' AND name='cards'")
[ "$PRES" = "[]" ] && ok "6.5 worker 未静默重建 cards" || no "6.5 cards 被静默重建: $PRES"

stop
echo
echo "==== worker 冒烟：PASS=$PASS FAIL=$FAIL ===="
[ "$FAIL" = 0 ] && exit 0 || exit 1
