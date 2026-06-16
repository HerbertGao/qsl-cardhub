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
SESSION_SECRET_VAL="smoke-session-secret-$(printf 'b%.0s' $(seq 1 40))"  # 模拟 256bit 会话 HMAC 密钥
start(){ # $1 = API_KEY value
  stop
  nohup npx wrangler dev --local --port "$PORT" --var API_KEY:"$1" --var SESSION_SECRET:"$SESSION_SECRET_VAL" >/tmp/qsl_smoke_dev.log 2>&1 &
  for i in $(seq 1 25); do curl -s -o /dev/null -m 2 "$B/api/config" 2>/dev/null && return 0; sleep 2; done
  echo "server failed to start; see /tmp/qsl_smoke_dev.log"; exit 1
}
# ── CDN 密钥回源头专用启动（trusted-client-ip / fix-cdn-real-ip 验证）──────────────────
# 信任信号 = 密钥头 X-Origin-Auth（CDN 覆盖语义注入）。经【临时测试 toml 的 [vars]】注入
# CDN_ORIGIN_SECRET / CDN_REAL_IP_HEADER（用 toml 而非裸 --var，统一且避免任何转义坑）。
# 该临时 toml 用绝对路径引 main/assets，并复用项目 wrangler.toml 的 D1 + RATE_LIMIT KV 绑定
# （--config 会以 toml 所在目录解析相对 main，故用绝对路径；放 /tmp 防污染项目）。
CDN_TOML=/tmp/qsl_smoke_cdn.toml
CDN_SECRET="smoke-origin-secret-$(printf 'a%.0s' $(seq 1 48))"  # 模拟 256bit 密钥（定长、无逗号）
CDN_HDR="Ali-Cdn-Real-Ip"
write_cdn_toml(){
  local root; root="$(pwd)"
  cat > "$CDN_TOML" <<EOF
name = "qsl-web-query-service"
main = "$root/src/worker/index.js"
compatibility_date = "2024-01-01"
workers_dev = false
preview_urls = false

[assets]
directory = "$root/public"

[[d1_databases]]
binding = "DB"
database_name = "qsl-sync"
database_id = "9f16589f-12cb-4c81-9810-49027018f037"

[[kv_namespaces]]
binding = "RATE_LIMIT"
id = "b88cc03aa78945b0a07095cc147f8394"

[vars]
CDN_ORIGIN_SECRET = "$CDN_SECRET"
CDN_REAL_IP_HEADER = "$CDN_HDR"
EOF
}
# 段间清 KV + 重启（RATE_LIMIT KV 在 dev 启动时载入；要清空计数桶须删 state 后重启，
# 否则前段计数渗入后段，「不同真实 IP 不互挤」断言会假过）。
# .wrangler/state/v3/kv 仅本地 miniflare 持久化（已 gitignore），删之不影响生产/远端。
start_cdn(){
  stop
  rm -rf .wrangler/state/v3/kv 2>/dev/null || true
  write_cdn_toml
  # --persist-to 锁回项目 .wrangler/state：--config 在 /tmp 会让 miniflare D1/KV 持久化分叉到
  # /tmp/.wrangler（空库、无 seed 的 tenants 表），auth-callback 的租户校验查询会打空库 500。
  nohup npx wrangler dev --local --port "$PORT" --config "$CDN_TOML" --persist-to "$(pwd)/.wrangler/state" --var API_KEY:"$TEST_KEY" --var SESSION_SECRET:"$SESSION_SECRET_VAL" >/tmp/qsl_smoke_dev.log 2>&1 &
  for i in $(seq 1 25); do curl -s -o /dev/null -m 2 "$B/api/config" 2>/dev/null && return 0; sleep 2; done
  echo "CDN-IP server failed to start; see /tmp/qsl_smoke_dev.log"; exit 1
}
# 无 KV 绑定启动（验证会话端点在 KV 未绑时 fail-closed 503）：临时 toml 含 D1、含 SESSION_SECRET，
# 但**不含** [[kv_namespaces]]，使 sessionSubsystemReady 因缺 RATE_LIMIT 为假 → 503（且非因 SESSION_SECRET 缺失）。
NOKV_TOML=/tmp/qsl_smoke_nokv.toml
start_nokv(){
  stop
  local root; root="$(pwd)"
  cat > "$NOKV_TOML" <<EOF
name = "qsl-web-query-service"
main = "$root/src/worker/index.js"
compatibility_date = "2024-01-01"
workers_dev = false
preview_urls = false

[assets]
directory = "$root/public"

[[d1_databases]]
binding = "DB"
database_name = "qsl-sync"
database_id = "9f16589f-12cb-4c81-9810-49027018f037"

[vars]
SESSION_SECRET = "$SESSION_SECRET_VAL"
EOF
  nohup npx wrangler dev --local --port "$PORT" --config "$NOKV_TOML" --persist-to "$root/.wrangler/state" --var API_KEY:"$TEST_KEY" >/tmp/qsl_smoke_dev.log 2>&1 &
  for i in $(seq 1 25); do curl -s -o /dev/null -m 2 "$B/api/config" 2>/dev/null && return 0; sleep 2; done
  echo "no-KV server failed to start; see /tmp/qsl_smoke_dev.log"; exit 1
}
code(){ curl -s -m 8 -o /dev/null -w '%{http_code}' "$@"; }
body(){ curl -s -m 8 "$@"; }                       # 返回响应体（用于断言 server_version / data 形态）
# ── 防爬会话握手助手（challenge → node 解 PoW → POST /api/session）────────────────
solve_pow(){ # $1=seed $2=difficulty → 打印满足前导零的 nonce
  node -e '
    const {createHash}=require("crypto");
    const seed=process.argv[1], diff=+process.argv[2];
    const lz=(h)=>{let b=0;for(const c of h){const n=parseInt(c,16);if(n===0){b+=4;continue}if(n<2)b+=3;else if(n<4)b+=2;else if(n<8)b+=1;break}return b};
    for(let i=0;i<(1<<27);i++){if(lz(createHash("sha256").update(seed+":"+i).digest("hex"))>=diff){process.stdout.write(String(i));return}}
  ' "$1" "$2"
}
mk_query_url(){ # $1=token $2=sk $3=callsign [$4=额外业务参数 "k=v&k2=v2"]；复用共享 canonical 模块 + HMAC(sk)
  node --input-type=module -e '
    import { buildCanonicalPayload } from "./src/worker/canonical.js";
    import { createHmac, randomBytes } from "node:crypto";
    const [base, token, sk, callsign, extra] = process.argv.slice(1);
    const sid = token.slice(0, token.lastIndexOf("."));
    const u = new URL(base + "/api/callsigns/" + encodeURIComponent(callsign));
    if (extra) for (const kv of extra.split("&")) { const i=kv.indexOf("="); u.searchParams.set(kv.slice(0,i), kv.slice(i+1)); }
    const ts = String(Date.now()); const nonce = randomBytes(12).toString("hex");
    u.searchParams.set("token", token);
    const payload = buildCanonicalPayload({ sid, path: u.pathname, params: u.searchParams, ts, nonce });
    u.searchParams.set("_ts", ts); u.searchParams.set("_nonce", nonce);
    u.searchParams.set("_sig", createHmac("sha256", sk).update(payload).digest("hex"));
    process.stdout.write(u.toString());
  ' "$B" "$1" "$2" "$3" "${4:-}"
}
new_session(){ # $1=CF-Connecting-IP 值 → 打印 "TOKEN SK"（失败打印空）
  local iph="$1" ch seed diff nonce sess token sk
  ch=$(body -H "CF-Connecting-IP: $iph" "$B/api/session/challenge")
  seed=$(echo "$ch" | jget seed --raw); diff=$(echo "$ch" | jget difficulty --raw)
  [ -n "$seed" ] || return 1
  nonce=$(solve_pow "$seed" "$diff")
  sess=$(body -H "CF-Connecting-IP: $iph" -X POST -H 'Content-Type: application/json' -d "{\"seed\":\"$seed\",\"nonce\":\"$nonce\"}" "$B/api/session")
  token=$(echo "$sess" | jget token --raw); sk=$(echo "$sess" | jget sk --raw)
  [ -n "$token" ] && [ -n "$sk" ] && printf '%s %s' "$token" "$sk"
}
# 重建并 seed cards 表（6.5 DROP cards 后，供需要查询的段落复用；schema cards DDL 含 IF NOT EXISTS）
reseed_cards(){
  d1 --command "
  CREATE TABLE IF NOT EXISTS cards (tenant_id TEXT NOT NULL, id TEXT NOT NULL, project_id TEXT NOT NULL, creator_id TEXT, callsign TEXT NOT NULL, qty INTEGER NOT NULL CHECK(qty > 0 AND qty <= 9999), serial INTEGER, status TEXT NOT NULL CHECK(status IN ('pending','distributed','returned')) DEFAULT 'pending', metadata TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, PRIMARY KEY (tenant_id, id));
  INSERT OR REPLACE INTO cards (tenant_id,id,project_id,callsign,qty,status,created_at,updated_at) VALUES ('bh2ro','cq1','p1','BG1ABC',5,'pending','t','t');
  INSERT OR REPLACE INTO cards (tenant_id,id,project_id,callsign,qty,status,created_at,updated_at) VALUES ('other','cqOther','pO','BG1ABC',9,'pending','t','t');
  INSERT OR REPLACE INTO projects (tenant_id,id,name,created_at,updated_at) VALUES ('bh2ro','p1','项目一','t','t');" >/dev/null
}
# 从 JSON 字符串取一个点路径字段（缺失返回空串）；--raw 输出原始值（字符串不带引号），否则 JSON.stringify。
# 用法：echo "$RESP" | jget server_version   /   echo "$RESP" | jget data.cards.0.metadata.foo --raw
jget(){ node -e '
  let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{
    let o; try{o=JSON.parse(s)}catch{process.stdout.write("");return}
    let raw=process.argv[2]==="--raw";
    let cur=o; for(const k of (process.argv[1]||"").split(".")){ if(cur==null){cur=undefined;break} cur=cur[k]; }
    if(cur===undefined){process.stdout.write("");return}
    process.stdout.write(raw && (typeof cur!=="object") ? String(cur) : JSON.stringify(cur));
  });' "$1" "${2:-}"; }
# 断言响应体某字段的 JS typeof（区分 object/boolean/string/number），用于 2.3a 形态契约。
jtypeof(){ node -e '
  let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{
    let o; try{o=JSON.parse(s)}catch{process.stdout.write("parse-error");return}
    let cur=o; for(const k of (process.argv[1]||"").split(".")){ if(cur==null){cur=undefined;break} cur=cur[k]; }
    process.stdout.write(cur===null?"null":Array.isArray(cur)?"array":typeof cur);
  });' "$1"; }
# 5 张业务表全行内容指纹：各表 group_concat(全部业务字段拼接 ORDER BY pk)，逐字节比对（不仅 COUNT/单列）。
# 拼接用 '|' 分隔、coalesce 兜空，ORDER BY 表主键非 tenant_id（同租户内 pk 唯一）。
fp(){ d1json "
  SELECT
   (SELECT coalesce(group_concat(coalesce(id,'')||'~'||coalesce(name,'')||'~'||coalesce(created_at,'')||'~'||coalesce(updated_at,''),'|'),'') FROM (SELECT * FROM projects WHERE tenant_id='$1' ORDER BY id)) AS projects,
   (SELECT coalesce(group_concat(coalesce(id,'')||'~'||coalesce(project_id,'')||'~'||coalesce(creator_id,'')||'~'||coalesce(callsign,'')||'~'||coalesce(qty,'')||'~'||coalesce(serial,'')||'~'||coalesce(status,'')||'~'||coalesce(metadata,'')||'~'||coalesce(created_at,'')||'~'||coalesce(updated_at,''),'|'),'') FROM (SELECT * FROM cards WHERE tenant_id='$1' ORDER BY id)) AS cards,
   (SELECT coalesce(group_concat(coalesce(id,'')||'~'||coalesce(name,'')||'~'||coalesce(phone,'')||'~'||coalesce(mobile,'')||'~'||coalesce(province,'')||'~'||coalesce(city,'')||'~'||coalesce(district,'')||'~'||coalesce(address,'')||'~'||coalesce(is_default,'')||'~'||coalesce(created_at,'')||'~'||coalesce(updated_at,''),'|'),'') FROM (SELECT * FROM sf_senders WHERE tenant_id='$1' ORDER BY id)) AS sf_senders,
   (SELECT coalesce(group_concat(coalesce(id,'')||'~'||coalesce(order_id,'')||'~'||coalesce(waybill_no,'')||'~'||coalesce(card_id,'')||'~'||coalesce(status,'')||'~'||coalesce(pay_method,'')||'~'||coalesce(cargo_name,'')||'~'||coalesce(sender_info,'')||'~'||coalesce(recipient_info,'')||'~'||coalesce(created_at,'')||'~'||coalesce(updated_at,''),'|'),'') FROM (SELECT * FROM sf_orders WHERE tenant_id='$1' ORDER BY id)) AS sf_orders,
   (SELECT coalesce(group_concat(coalesce(key,'')||'~'||coalesce(value,''),'|'),'') FROM (SELECT * FROM app_settings WHERE tenant_id='$1' ORDER BY key)) AS app_settings
  "; }
ver(){ d1json "SELECT server_version AS v FROM sync_meta WHERE tenant_id='$1'" | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{try{const r=JSON.parse(s);process.stdout.write(r.length?String(r[0].v):"")}catch{process.stdout.write("")}})'; }

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
INSERT INTO tenants (tenant_id,name,tier,status) VALUES ('gone','GONE',NULL,'revoked');
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

echo; echo "########## 6.6 读取注入 bh2ro + tenant_id 注入被忽略（经会话校验）##########"
# 前序 /sync 全量覆盖会清掉 bh2ro 卡片，6.6 前重新 seed 查询样例（test-only 状态复位）。
# 查询端点已改为需有效会话 → 先握手建会话，再带会话签名查询（同一 CF-IP 保证 binding 一致）。
reseed_cards
Q66_IP="203.0.113.66"
read Q66_TOKEN Q66_SK <<EOF
$(new_session "$Q66_IP")
EOF
[ -n "$Q66_TOKEN" ] && [ -n "$Q66_SK" ] && ok "6.6 会话握手建立成功" || no "6.6 会话握手失败"
R1=$(body -H "CF-Connecting-IP: $Q66_IP" "$(mk_query_url "$Q66_TOKEN" "$Q66_SK" "BG1ABC")")
# 带签名的 tenant_id=other 注入（tenant_id 作业务参数纳入签名）→ 服务端应忽略它、仍按 bh2ro 查
R2=$(body -H "CF-Connecting-IP: $Q66_IP" "$(mk_query_url "$Q66_TOKEN" "$Q66_SK" "BG1ABC" "tenant_id=other")")
echo "$R1" | grep -q '"id":"cq1"' && ! echo "$R1" | grep -q 'cqOther' && ok "6.6 query 仅返回 bh2ro cq1，无 other 泄漏" || no "6.6 query 结果异常: $R1"
echo "$R2" | grep -q '"id":"cq1"' && ! echo "$R2" | grep -q 'cqOther' && ok "6.6 ?tenant_id=other 被忽略（仍按 bh2ro 查，无 other 泄漏）" || no "6.6 tenant_id 注入改变了结果: $R2"

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

echo; echo "########## 6.1/6.2/6.3/2.3a OCC 乐观并发护栏 + /pull（add-sync-robustness）##########"
# 本段全部走 bh2ro 租户；前置：用一次「无条件路径」/sync 建立确定基线（含一条富样本以喂 2.3a 形态契约）。
# 富样本：cards 带嵌套 metadata（distribution/return 对象）、sf_orders 带嵌套 sender_info/recipient_info（对象）、sf_senders is_default=true。
RICH_DATA='{"projects":[{"id":"rp1","name":"项目甲"}],
  "cards":[{"id":"rc1","project_id":"rp1","callsign":"BG1OCC","qty":7,"serial":42,"status":"pending","metadata":{"distribution":{"method":"mail","proxy_callsign":"BA1XYZ","remarks":"r-dist"},"return":{"method":"self","remarks":"r-ret"}}}],
  "sf_senders":[{"id":"rs1","name":"张三","phone":"010-1","mobile":"139","province":"京","city":"京","district":"海淀","address":"中关村","is_default":true}],
  "sf_orders":[{"id":"ro1","order_id":"OID1","waybill_no":"WB1","card_id":"rc1","status":"pending","pay_method":1,"cargo_name":"QSL卡片","sender_info":{"name":"张三","phone":"010-1"},"recipient_info":{"name":"李四","phone":"020-2"}}],
  "app_settings":[{"key":"lang","value":"zh"}]}'
# ── 重置基线：无条件路径（不带 base_version）→ 200 且 server_version 单调推进 ──
SEED_RESP=$(body -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d "{\"client_id\":\"seed\",\"data\":$RICH_DATA}" "$B/sync")
V0=$(echo "$SEED_RESP" | jget server_version --raw)
[ -n "$V0" ] && [ "$(echo "$SEED_RESP" | jget success --raw)" = true ] && ok "OCC 基线：无条件 /sync 200 且回传 server_version=$V0" || no "OCC 基线 /sync 未回传 server_version ($SEED_RESP)"

echo; echo "## 6.1① 携带匹配 base_version -> 200 且 server_version 自增 ##"
R1=$(body -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d "{\"client_id\":\"occ1\",\"base_version\":$V0,\"data\":$RICH_DATA}" "$B/sync")
V1=$(echo "$R1" | jget server_version --raw)
[ "$(echo "$R1" | jget success --raw)" = true ] && [ "$V1" = "$((V0+1))" ] && ok "6.1① 匹配 base_version /sync 200 且 server_version ${V0}->${V1}（=base+1）" || no "6.1① 匹配 base 未 200/版本未自增 ($R1)"
[ "$(ver bh2ro)" = "$V1" ] && ok "6.1① 云端 sync_meta.server_version 落库 == $V1" || no "6.1① 云端版本未推进到 $V1"

echo; echo "## 6.1② 陈旧 base_version -> 409 且数据未变（全行内容指纹逐字节一致）##"
FP_BEFORE=$(fp bh2ro)
# 陈旧 base = V0（云端已到 V1）；payload 故意改 callsign/metadata，若误覆盖则指纹必变 → 证伪「同行数假绿」。
STALE_PAYLOAD='{"projects":[{"id":"rp1","name":"被篡改"}],"cards":[{"id":"rc1","project_id":"rp1","callsign":"HACKED","qty":1,"metadata":{"x":"y"}}]}'
R2C=$(code -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d "{\"client_id\":\"occ2\",\"base_version\":$V0,\"data\":$STALE_PAYLOAD}" "$B/sync")
R2=$(body -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d "{\"client_id\":\"occ2\",\"base_version\":$V0,\"data\":$STALE_PAYLOAD}" "$B/sync")
[ "$R2C" = 409 ] && ok "6.1② 陈旧 base_version /sync 409" || no "6.1② 陈旧 base 未返 409 ($R2C)"
[ "$(echo "$R2" | jget server_version --raw)" = "$V1" ] && ok "6.1② 409 体回传云端当前 server_version=$V1" || no "6.1② 409 体 server_version 非当前 ($R2)"
FP_AFTER=$(fp bh2ro)
[ "$FP_BEFORE" = "$FP_AFTER" ] && ok "6.1② 409 前后全行指纹逐字节一致（零改动，非仅 COUNT）" || no "6.1② 409 后数据被改动！before=$FP_BEFORE after=$FP_AFTER"
[ "$(ver bh2ro)" = "$V1" ] && ok "6.1② 409 后 server_version 未推进（仍 ${V1}）" || no "6.1② 409 后版本被推进"

echo; echo "## 6.1③ force=true 陈旧 base -> 200 覆盖且版本=当前+1 ##"
R3=$(body -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d "{\"client_id\":\"occ3\",\"base_version\":$V0,\"force\":true,\"data\":$RICH_DATA}" "$B/sync")
V3=$(echo "$R3" | jget server_version --raw)
[ "$(echo "$R3" | jget success --raw)" = true ] && [ "$V3" = "$((V1+1))" ] && ok "6.1③ force=true 陈旧 base /sync 200 且版本 ${V1}->${V3}（=当前+1）" || no "6.1③ force 陈旧 base 未覆盖/版本错 ($R3)"

echo; echo "## 6.1④ 不带 base_version（旧客户端）-> 200 无条件覆盖且版本+1 ##"
R4=$(body -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d "{\"client_id\":\"occ4\",\"data\":$RICH_DATA}" "$B/sync")
V4=$(echo "$R4" | jget server_version --raw)
[ "$(echo "$R4" | jget success --raw)" = true ] && [ "$V4" = "$((V3+1))" ] && ok "6.1④ 旧客户端无 base /sync 200 且版本 ${V3}->${V4}（+1）" || no "6.1④ 无 base 未 200/版本未+1 ($R4)"

echo; echo "## 6.1⑤ 命中时某业务表为空数组仍 200（防末条结果归属错位把空表 INSERT changes 误读为 409）##"
# 当前云端版本 V4；data 只给 cards，其余 4 表为空数组 → 守卫路径命中应 200。
R5=$(body -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d "{\"client_id\":\"occ5\",\"base_version\":$V4,\"data\":{\"projects\":[],\"cards\":[{\"id\":\"only1\",\"project_id\":\"rp1\",\"callsign\":\"BONLY\",\"qty\":1}],\"sf_senders\":[],\"sf_orders\":[],\"app_settings\":[]}}" "$B/sync")
V5=$(echo "$R5" | jget server_version --raw)
[ "$(echo "$R5" | jget success --raw)" = true ] && [ "$V5" = "$((V4+1))" ] && ok "6.1⑤ 空表数组守卫命中仍 200 且版本 $V4->$V5" || no "6.1⑤ 空表数组被误判 409/版本错 ($R5)"
[ "$(d1json "SELECT count(*) AS n FROM cards WHERE tenant_id='bh2ro'")" = '[{"n":1}]' ] && ok "6.1⑤ 全量替换后仅 only1 一行（空表 DELETE 生效）" || no "6.1⑤ 空表替换异常"

echo; echo "## 6.1⑥ sync_meta 行缺失 -> 409 且 server_version===null（非 undefined、非 500）##"
d1 --command "DELETE FROM sync_meta WHERE tenant_id='bh2ro';" >/dev/null
FP6_BEFORE=$(fp bh2ro)
R6C=$(code -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d "{\"client_id\":\"occ6\",\"base_version\":0,\"data\":$STALE_PAYLOAD}" "$B/sync")
R6=$(body -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d "{\"client_id\":\"occ6\",\"base_version\":0,\"data\":$STALE_PAYLOAD}" "$B/sync")
[ "$R6C" = 409 ] && ok "6.1⑥ sync_meta 缺失守卫路径 409（非 500）" || no "6.1⑥ sync_meta 缺失未返 409 ($R6C)"
# server_version 必须是 JSON null（typeof===object/null），禁 undefined（字段缺失 jget 返空）/NaN。
SV6=$(echo "$R6" | jget server_version)
[ "$SV6" = "null" ] && ok "6.1⑥ 409 体 server_version === null（JSON null，非 undefined/NaN）" || no "6.1⑥ 409 体 server_version 非 null: '$SV6' ($R6)"
[ "$FP6_BEFORE" = "$(fp bh2ro)" ] && ok "6.1⑥ sync_meta 缺失 409 后业务数据零改动" || no "6.1⑥ sync_meta 缺失 409 后数据被改动"
# 复位：用无条件路径重建 sync_meta 行并恢复富样本，供后续段落。
RESEED=$(body -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d "{\"client_id\":\"reseed\",\"data\":$RICH_DATA}" "$B/sync")
VR=$(echo "$RESEED" | jget server_version --raw)
[ -n "$VR" ] && ok "6.1⑥ 复位：无条件 /sync 重建 sync_meta 行（server_version=${VR}）" || no "6.1⑥ 复位失败 ($RESEED)"

echo; echo "## 6.1⑦ 无条件路径上传 N>=1 非空行 -> 200 且读回 server_version 正确（跨路径对照）##"
# 无条件路径读末条 SELECT、守卫路径读 CAS；此处验证无条件读回与云端落库一致。
R7=$(body -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d '{"client_id":"occ7","data":{"cards":[{"id":"u1","project_id":"rp1","callsign":"BU1","qty":2},{"id":"u2","project_id":"rp1","callsign":"BU2","qty":3}]}}' "$B/sync")
V7=$(echo "$R7" | jget server_version --raw)
[ "$(echo "$R7" | jget success --raw)" = true ] && [ -n "$V7" ] && [ "$V7" = "$(ver bh2ro)" ] && ok "6.1⑦ 无条件多行 /sync 200 且读回 server_version=$V7 == 云端落库" || no "6.1⑦ 无条件读回 server_version 与云端不符 ($R7 / db=$(ver bh2ro))"

echo; echo "## 6.2 连续两次同步（中间无其他写）第二次用首次 V 作 base 仍 200（基线刷新闭环）##"
RA=$(body -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d '{"client_id":"seqA","data":{"cards":[{"id":"sq1","project_id":"rp1","callsign":"BSEQ","qty":1}]}}' "$B/sync")
VA=$(echo "$RA" | jget server_version --raw)
[ "$(echo "$RA" | jget success --raw)" = true ] && [ -n "$VA" ] && ok "6.2 第一次 /sync 200 拿到 server_version=$VA" || no "6.2 第一次 /sync 未回传版本 ($RA)"
RB=$(body -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d "{\"client_id\":\"seqA\",\"base_version\":$VA,\"data\":{\"cards\":[{\"id\":\"sq2\",\"project_id\":\"rp1\",\"callsign\":\"BSEQ2\",\"qty\":1}]}}" "$B/sync")
VB=$(echo "$RB" | jget server_version --raw)
[ "$(echo "$RB" | jget success --raw)" = true ] && [ "$VB" = "$((VA+1))" ] && ok "6.2 第二次用 V=${VA} 作 base 仍 200（版本 ${VA}->${VB}），非 409" || no "6.2 第二次同步 409 或版本错（基线刷新闭环断裂）($RB)"

echo; echo "## 6.3 两客户端持同一 base 依次上传：先到 200、后到 409 且零改动 ##"
VBASE=$(ver bh2ro)
# 客户端 X 先上传（base=VBASE）→ 200 推进版本。
RX=$(body -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d "{\"client_id\":\"cliX\",\"base_version\":$VBASE,\"data\":{\"cards\":[{\"id\":\"x1\",\"project_id\":\"rp1\",\"callsign\":\"BCLIX\",\"qty\":1}]}}" "$B/sync")
[ "$(echo "$RX" | jget success --raw)" = true ] && ok "6.3 先到客户端 X /sync 200（base=${VBASE}）" || no "6.3 先到 X 未 200 ($RX)"
FPY_BEFORE=$(fp bh2ro)
# 客户端 Y 持同一陈旧 base=VBASE 后上传 → 409 且零改动。
RYC=$(code -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d "{\"client_id\":\"cliY\",\"base_version\":$VBASE,\"data\":{\"cards\":[{\"id\":\"y1\",\"project_id\":\"rp1\",\"callsign\":\"BCLIY\",\"qty\":1}]}}" "$B/sync")
[ "$RYC" = 409 ] && ok "6.3 后到客户端 Y 持同一 base /sync 409" || no "6.3 后到 Y 未 409 ($RYC)"
[ "$FPY_BEFORE" = "$(fp bh2ro)" ] && ok "6.3 后到 Y 409 零改动（指纹一致，y1 未写入）" || no "6.3 后到 Y 409 却改动了数据"

echo; echo "## 6.3 /pull：有效写 Key -> 200 全量+server_version；错误/缺失 Key -> 401；查询参数注入被忽略 ##"
PULL=$(body -H "Authorization: Bearer $TEST_KEY" "$B/pull")
[ "$(echo "$PULL" | jget success --raw)" = true ] && ok "6.3 /pull 有效写 Key 200 success=true" || no "6.3 /pull 有效 Key 非 200 ($PULL)"
[ "$(echo "$PULL" | jget server_version --raw)" = "$(ver bh2ro)" ] && ok "6.3 /pull 返回 server_version == 云端落库($(ver bh2ro))" || no "6.3 /pull server_version 不符"
# 全量字段集存在
echo "$PULL" | grep -q '"projects"' && echo "$PULL" | grep -q '"cards"' && echo "$PULL" | grep -q '"sf_senders"' && echo "$PULL" | grep -q '"sf_orders"' && echo "$PULL" | grep -q '"app_settings"' && ok "6.3 /pull data 含全部 5 张业务表" || no "6.3 /pull data 缺表 ($PULL)"
[ "$(code -H "Authorization: Bearer WRONGKEY" "$B/pull")" = 401 ] && ok "6.3 /pull 错误 Key 401" || no "6.3 /pull 错误 Key 非 401"
[ "$(code "$B/pull")" = 401 ] && ok "6.3 /pull 缺失 Key 401" || no "6.3 /pull 缺失 Key 非 401"
# 查询参数注入 tenant_id：worker 由 Key 解析、忽略参数 → 与无参一致（不跨租户）。
PULL_INJ=$(body -H "Authorization: Bearer $TEST_KEY" "$B/pull?tenant_id=other&base_version=999")
[ "$PULL" = "$PULL_INJ" ] && ok "6.3 /pull ?tenant_id=other 注入被忽略（结果与无参一致，不跨租户）" || no "6.3 /pull 注入改变了结果（疑跨租户）"

echo; echo "## 2.3a 往返形态：/sync 写嵌套 metadata/sender_info -> /pull 读回为对象/布尔（非转义字符串/整数）##"
# 用一条已知富样本无条件覆盖，再 /pull 读回断言形态与逐字段值。
RT_DATA='{"projects":[{"id":"tp1","name":"往返项目"}],
  "cards":[{"id":"tc1","project_id":"tp1","callsign":"BG2RT","qty":9,"serial":7,"status":"distributed","metadata":{"distribution":{"method":"mail","proxy_callsign":"BA2RT","remarks":"dr"},"return":{"method":"self","remarks":"rr"}}}],
  "sf_senders":[{"id":"ts1","name":"寄件甲","phone":"P1","mobile":"M1","province":"PV","city":"CT","district":"DT","address":"AD","is_default":true}],
  "sf_orders":[{"id":"to1","order_id":"TOID","waybill_no":"TWB","card_id":"tc1","status":"confirmed","pay_method":2,"cargo_name":"卡片","sender_info":{"name":"寄件甲","phone":"P1"},"recipient_info":{"name":"收件乙","phone":"P2"}}],
  "app_settings":[{"key":"k1","value":"v1"}]}'
body -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d "{\"client_id\":\"rt\",\"data\":$RT_DATA}" "$B/sync" >/dev/null
RP=$(body -H "Authorization: Bearer $TEST_KEY" "$B/pull")
[ "$(echo "$RP" | jtypeof data.cards.0.metadata)" = object ] && ok "2.3a /pull cards[0].metadata 为 object（非转义字符串）" || no "2.3a metadata 形态非 object: $(echo "$RP" | jtypeof data.cards.0.metadata)"
[ "$(echo "$RP" | jget data.cards.0.metadata.distribution.proxy_callsign --raw)" = "BA2RT" ] && ok "2.3a /pull 嵌套 metadata.distribution.proxy_callsign 还原正确" || no "2.3a 嵌套 metadata 还原错"
[ "$(echo "$RP" | jtypeof data.sf_orders.0.sender_info)" = object ] && ok "2.3a /pull sf_orders[0].sender_info 为 object" || no "2.3a sender_info 形态非 object"
[ "$(echo "$RP" | jtypeof data.sf_orders.0.recipient_info)" = object ] && ok "2.3a /pull sf_orders[0].recipient_info 为 object" || no "2.3a recipient_info 形态非 object"
[ "$(echo "$RP" | jget data.sf_orders.0.recipient_info.name --raw)" = "收件乙" ] && ok "2.3a /pull recipient_info.name 还原正确" || no "2.3a recipient_info 内容错"
[ "$(echo "$RP" | jtypeof data.sf_senders.0.is_default)" = boolean ] && ok "2.3a /pull sf_senders[0].is_default 为 boolean（非整数 1/0）" || no "2.3a is_default 形态非 boolean: $(echo "$RP" | jtypeof data.sf_senders.0.is_default)"
[ "$(echo "$RP" | jget data.sf_senders.0.is_default --raw)" = "true" ] && ok "2.3a /pull is_default === true（整数 1 还原为布尔真）" || no "2.3a is_default 值非 true"

echo; echo "## 2.3a 异构多列样本逐字段相等（证伪绑定列错序：callsign/serial/qty 绑反则此处必挂）##"
# 写入多行、各列各不相同；/pull 读回逐字段比对。绑定错序时行数/版本仍对、内容错位 → 这里抓。
MULTI_DATA='{"cards":[
  {"id":"m1","project_id":"rp1","callsign":"AAA1","qty":11,"serial":101,"status":"pending","metadata":{"tag":"one"}},
  {"id":"m2","project_id":"rp1","callsign":"BBB2","qty":22,"serial":202,"status":"distributed","metadata":{"tag":"two"}},
  {"id":"m3","project_id":"rp1","callsign":"CCC3","qty":33,"serial":303,"status":"returned","metadata":{"tag":"three"}}]}'
body -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d "{\"client_id\":\"multi\",\"data\":$MULTI_DATA}" "$B/sync" >/dev/null
RM=$(body -H "Authorization: Bearer $TEST_KEY" "$B/pull")
# /pull 不保证顺序，用 node 按 id 索引逐字段比对期望值。
MOK=$(echo "$RM" | node -e '
  let s="";process.stdin.on("data",d=>s+=d).on("end",()=>{
    let o; try{o=JSON.parse(s)}catch{console.log("PARSE_ERR");return}
    const exp={m1:["AAA1",11,101,"pending","one"],m2:["BBB2",22,202,"distributed","two"],m3:["CCC3",33,303,"returned","three"]};
    const by={}; for(const c of (o.data&&o.data.cards||[])) by[c.id]=c;
    for(const id of Object.keys(exp)){
      const c=by[id]; if(!c){console.log("MISS:"+id);return}
      const e=exp[id];
      if(c.callsign!==e[0]||c.qty!==e[1]||c.serial!==e[2]||c.status!==e[3]||(c.metadata&&c.metadata.tag)!==e[4]){
        console.log("MISMATCH:"+id+" got="+JSON.stringify([c.callsign,c.qty,c.serial,c.status,c.metadata&&c.metadata.tag]));return;
      }
    }
    console.log("OK");
  });')
[ "$MOK" = OK ] && ok "2.3a 异构多列 3 行逐字段相等（callsign/qty/serial/status/metadata 未绑定错序）" || no "2.3a 异构多列逐字段不等: $MOK"

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

echo; echo "########## 4.4 可信真实客户端 IP 限流归桶（fix-cdn-real-ip / trusted-client-ip）##########"
# 端到端验证 getClientIP 在「密钥回源头」模型下的限流计数键归桶；KV 已绑（dev --local 真计数到 429）。
# 信任信号 = 带有效 X-Origin-Auth 密钥（=来自 CDN）→ 采信 Ali-Cdn-Real-Ip 真实用户 IP；
# 无/错密钥（直连）→ 按 CF-Connecting-IP 计数、忽略一切注入头。绝不采信 X-Forwarded-For。
# miniflare 透传 curl 设的 CF-Connecting-IP / X-Origin-Auth / Ali-Cdn-Real-Ip（与 design / RC 实测一致）；
# 若未来本地不再透传，断言会整体偏移即暴露环境非等价（对齐 CHECKLIST.md），届时归桶以纯函数单测为准。
RL_MAX=20  # = index.js RATE_LIMIT_MAX；查询桶 ratelimit:<bkey>（归一键），authcb 独立桶 ratelimit:authcb:<bkey>，握手桶 ratelimit:session:<bkey> 另算。
hammer(){ local n="$1"; shift; local last=""; for _i in $(seq 1 "$n"); do last=$(code "$@"); done; printf '%s' "$last"; }
# checkRateLimit 用固定窗口 now-(now%60)；若 RL_MAX+1 连发跨窗口边界，计数会中途重置→末次到不了上限。
# 故每个「打满到 429」前对齐到新窗口起点：仅当处于当前窗口末段(>45s)时才睡入下一窗口（最多 ~15s，平时不睡）。
align_window(){ local sec; sec=$(( $(date +%s) % 60 )); if [ "$sec" -gt 45 ]; then sleep $((61 - sec)); fi; }

# 段间清 KV + 重启（用 CDN 测试 toml；密钥经 [vars] 注入）。
start_cdn
AUTH="X-Origin-Auth: $CDN_SECRET"                 # 有效密钥（=经 CDN 覆写注入）
AUTH_BAD="X-Origin-Auth: smoke-WRONG-secret-value" # 伪造/错误密钥（攻击者猜不到真值）
CF_NODE="203.0.113.50"    # 经 CDN 时 CF-Connecting-IP = CDN 回源节点 IP（密钥有效则忽略它、按真实头归桶）
CF_NODE2="198.51.100.77"  # 另一 CDN 回源节点 IP（同一真实用户应仍同桶）
CF_DIRECT="8.8.8.8"       # 直连 Cloudflare 源站时的真实出口 IP
# 注：查询端点 Layer0 限流先于会话校验——未限流但无会话 → 401（=401 即「未被限流」哨兵）；auth-callback 未配微信 → 503 哨兵。429 一律来自 checkRateLimit（限流先行）。

echo "## 4.4① 经 CDN（有效密钥）不同真实用户互不挤占（按 Ali-Cdn-Real-Ip 归桶，非 CDN 节点 IP）##"
# 真实用户 A（198.51.100.10）：带有效密钥连发 RL_MAX+1 → 末次必 429（按真实 IP 计数到上限）。
align_window
A_LAST=$(hammer $((RL_MAX+1)) -H "CF-Connecting-IP: $CF_NODE" -H "$AUTH" -H "Ali-Cdn-Real-Ip: 198.51.100.10" "$B/api/callsigns/SMOKE")
[ "$A_LAST" = 429 ] && ok "4.4① 真实用户 A 经 CDN 计数到上限 → 429（按 Ali-Cdn-Real-Ip 真实 IP）" || no "4.4① 真实用户 A 未到 429 ($A_LAST)"
# 真实用户 B（198.51.100.20）：同一 CDN 节点 IP、有效密钥、不同真实头 → 独立桶，1 次未限流 → 401（限流通过、无会话哨兵；=401 而非 !=429，服务挂(000)时不假过）。
B_ONE=$(code -H "CF-Connecting-IP: $CF_NODE" -H "$AUTH" -H "Ali-Cdn-Real-Ip: 198.51.100.20" "$B/api/callsigns/SMOKE")
[ "$B_ONE" = 401 ] && ok "4.4① 真实用户 B 独立桶不受 A 挤占 → 401 未限流（真实 IP 各自计数）" || no "4.4① 真实用户 B 被 A 挤占或服务异常 ($B_ONE)"
# 同一真实用户经【不同 CDN 节点 IP】(CF_NODE2) 仍归同一真实 IP 桶：A 已满 → 仍 429（证按真实 IP 而非节点 IP）。
A_VIA2=$(code -H "CF-Connecting-IP: $CF_NODE2" -H "$AUTH" -H "Ali-Cdn-Real-Ip: 198.51.100.10" "$B/api/callsigns/SMOKE")
[ "$A_VIA2" = 429 ] && ok "4.4① 真实用户 A 经另一 CDN 节点($CF_NODE2)仍同桶 429（按真实 IP 非节点 IP 归桶）" || no "4.4① 真实用户 A 换 CDN 节点后逃逸限流 ($A_VIA2)（误按节点 IP 归桶）"

echo "## 4.4② 经 CDN 伪造 XFF 不绕过限流（绝不采信 XFF 任何分段）##"
# 真实用户 A 已满桶；带有效密钥 + 每次不同伪造 XFF 仍 429 → XFF 未被用作自选桶键。
XFF1=$(code -H "CF-Connecting-IP: $CF_NODE" -H "$AUTH" -H "Ali-Cdn-Real-Ip: 198.51.100.10" -H "X-Forwarded-For: 6.6.6.6, 198.51.100.10" "$B/api/callsigns/SMOKE")
XFF2=$(code -H "CF-Connecting-IP: $CF_NODE" -H "$AUTH" -H "Ali-Cdn-Real-Ip: 198.51.100.10" -H "X-Forwarded-For: 7.7.7.7, 198.51.100.10" "$B/api/callsigns/SMOKE")
[ "$XFF1" = 429 ] && [ "$XFF2" = 429 ] && ok "4.4② 满桶真实用户带不同伪造 XFF 仍 429（XFF 不制造自选桶）" || no "4.4② 伪造 XFF 绕过限流 (XFF1=$XFF1 XFF2=$XFF2)"

echo "## 4.4③ 直连(无密钥) + 伪造 Ali-Cdn-Real-Ip/XFF → 按 CF-Connecting-IP 计数 ##"
# 直连真实出口 8.8.8.8（不带 X-Origin-Auth）：每次伪造不同 Ali-Cdn-Real-Ip + XFF；若误采信伪造头则每次新桶永不 429。
align_window
D_LAST=""
for i in $(seq 1 $((RL_MAX+1))); do
  D_LAST=$(code -H "CF-Connecting-IP: $CF_DIRECT" -H "Ali-Cdn-Real-Ip: 1.2.3.$i" -H "X-Forwarded-For: 5.5.5.$i" "$B/api/callsigns/SMOKE")
done
[ "$D_LAST" = 429 ] && ok "4.4③ 直连无密钥伪造 Ali-Cdn-Real-Ip/XFF 均无效，按 CF-Connecting-IP 计数到 429" || no "4.4③ 直连伪造头绕过限流（永不 429, last=${D_LAST}）"

echo "## 4.4④ 错误密钥（伪造 X-Origin-Auth）→ 按 CF-Connecting-IP 计数（不采信注入头）##"
# 同一直连出口 7.7.7.7 带【错误密钥】+ 每次不同伪造 Ali-Cdn-Real-Ip：若误信则每次新桶永不 429；正确应按 CF-IP 到 429。
CF_BADKEY="7.7.7.7"
align_window
BK_LAST=""
for i in $(seq 1 $((RL_MAX+1))); do
  BK_LAST=$(code -H "CF-Connecting-IP: $CF_BADKEY" -H "$AUTH_BAD" -H "Ali-Cdn-Real-Ip: 4.4.4.$i" "$B/api/callsigns/SMOKE")
done
[ "$BK_LAST" = 429 ] && ok "4.4④ 错误密钥下伪造 Ali-Cdn-Real-Ip 无效，按 CF-Connecting-IP 计数到 429" || no "4.4④ 错误密钥仍采信注入头绕过限流（永不 429, last=${BK_LAST}）"

echo "## 4.4⑤ auth-callback 经 CDN（有效密钥）按真实用户 IP 计数（独立桶 authcb，不与查询互挤）##"
# 清 KV 重启隔离本段（authcb 桶独立、且与上文 captcha 桶证伪互不渗透）。
start_cdn
AUTH="X-Origin-Auth: $CDN_SECRET"
CB="$B/api/wechat/auth-callback?code=x&state=BG1ABC"
# 真实用户 U（198.51.100.30）带有效密钥 + 每次不同伪造 XFF：连发 RL_MAX+1 → 末次 429（按真实 IP，XFF 不绕过）。
align_window
U_LAST=""
for i in $(seq 1 $((RL_MAX+1))); do
  U_LAST=$(code -H "CF-Connecting-IP: $CF_NODE" -H "$AUTH" -H "Ali-Cdn-Real-Ip: 198.51.100.30" -H "X-Forwarded-For: 9.9.9.$i" "$CB")
done
[ "$U_LAST" = 429 ] && ok "4.4⑤ auth-callback 真实用户经 CDN 计数到 429（按真实 IP，伪造 XFF 不绕过）" || no "4.4⑤ auth-callback 真实用户未到 429 ($U_LAST)"
# 同一真实用户 U 在【查询桶】仍新鲜（authcb 与查询桶独立）→ 1 次未限流 → 401 哨兵（=401 而非 !=429，服务挂(000)不假过）。
U_QUERY=$(code -H "CF-Connecting-IP: $CF_NODE" -H "$AUTH" -H "Ali-Cdn-Real-Ip: 198.51.100.30" "$B/api/callsigns/SMOKE")
[ "$U_QUERY" = 401 ] && ok "4.4⑤ 同真实用户查询桶不受 authcb 挤占 → 401 未限流（独立桶不变）" || no "4.4⑤ authcb 桶渗入查询桶或服务异常 ($U_QUERY)"
# 另一真实用户 V 在 auth-callback 独立桶 → 1 次未限流 → 503 哨兵（=503 而非 !=429，服务挂(000)不假过）。
V_ONE=$(code -H "CF-Connecting-IP: $CF_NODE" -H "$AUTH" -H "Ali-Cdn-Real-Ip: 198.51.100.40" "$CB")
[ "$V_ONE" = 503 ] && ok "4.4⑤ auth-callback 另一真实用户 V 独立桶 → 503 未限流" || no "4.4⑤ auth-callback 真实用户 V 被 U 挤占或服务异常 ($V_ONE)"

echo; echo "########## 4.5 防爬会话握手端到端（query-antibot-session）##########"
# 用项目 toml 启动（worker 与 d1()/reseed_cards 共用同一本地 D1 持久化目录，避免 --config /tmp 分叉）；
# 清 KV 使首个 challenge 难度=base（18，node 可在 ~0.3s 内解）。三步用同一 CF-Connecting-IP（无
# X-Origin-Auth → getClientIP 取该 CF 值）保证 binding_key 一致。
stop
rm -rf .wrangler/state/v3/kv 2>/dev/null || true
start "$TEST_KEY"
reseed_cards   # 6.5 DROP 了 cards 表，4.5④ 查询前重建并 seed
SESS_IP="203.0.113.99"
SH=(-H "CF-Connecting-IP: $SESS_IP")   # 一致来源头（无密钥 → 按 CF-IP 归一键绑定）

# 端点退役检查：/api/captcha 404、/api/config 无 sign_key/captcha
[ "$(code "$B/api/captcha")" = 404 ] && ok "4.5 /api/captcha 已移除 → 404" || no "4.5 /api/captcha 未 404（$(code "$B/api/captcha")）"
CFG=$(body "$B/api/config")
[ -z "$(echo "$CFG" | jget sign_key --raw)" ] && ! echo "$CFG" | grep -q '"captcha"' && ok "4.5 /api/config 不再下发 sign_key / captcha" || no "4.5 /api/config 仍含 sign_key/captcha: $CFG"

# ① challenge → {seed, difficulty}
CH=$(body "${SH[@]}" "$B/api/session/challenge")
SEED=$(echo "$CH" | jget seed --raw); DIFF=$(echo "$CH" | jget difficulty --raw)
[ -n "$SEED" ] && [ -n "$DIFF" ] && ok "4.5① challenge 下发 seed+difficulty, diff=$DIFF" || no "4.5① challenge 异常: $CH"

# ② node 解 PoW（sha256(seed:nonce) 前导零 ≥ difficulty；与服务端 crypto.subtle 一致）
NONCE=$(node -e '
  const {createHash}=require("crypto");
  const seed=process.argv[1], diff=+process.argv[2];
  const lz=(hex)=>{let b=0;for(const c of hex){const n=parseInt(c,16);if(n===0){b+=4;continue}if(n<2)b+=3;else if(n<4)b+=2;else if(n<8)b+=1;break}return b};
  for(let i=0;i<(1<<27);i++){if(lz(createHash("sha256").update(seed+":"+i).digest("hex"))>=diff){process.stdout.write(String(i));return}}
  process.stderr.write("pow-fail");
' "$SEED" "$DIFF")
[ -n "$NONCE" ] && ok "4.5② node 解出 PoW nonce, diff=$DIFF" || no "4.5② PoW 求解失败"

# ③ POST /api/session 验 PoW → 签发会话
SESS=$(body "${SH[@]}" -X POST -H 'Content-Type: application/json' -d "{\"seed\":\"$SEED\",\"nonce\":\"$NONCE\"}" "$B/api/session")
TOKEN=$(echo "$SESS" | jget token --raw); SK=$(echo "$SESS" | jget sk --raw); QUOTA=$(echo "$SESS" | jget quota --raw)
[ -n "$TOKEN" ] && [ -n "$SK" ] && ok "4.5③ POST /api/session 验 PoW 签发会话 quota=$QUOTA" || no "4.5③ 会话签发失败: $SESS"

# 构造带会话签名的查询 URL（复用共享 canonical 模块 + HMAC(sk)）
mk_url(){ node --input-type=module -e '
  import { buildCanonicalPayload } from "./src/worker/canonical.js";
  import { createHmac, randomBytes } from "node:crypto";
  const [base, token, sk, callsign] = process.argv.slice(1);
  const sid = token.slice(0, token.lastIndexOf("."));
  const u = new URL(base + "/api/callsigns/" + encodeURIComponent(callsign));
  const ts = String(Date.now()); const nonce = randomBytes(12).toString("hex");
  u.searchParams.set("token", token);
  const payload = buildCanonicalPayload({ sid, path: u.pathname, params: u.searchParams, ts, nonce });
  u.searchParams.set("_ts", ts); u.searchParams.set("_nonce", nonce);
  u.searchParams.set("_sig", createHmac("sha256", sk).update(payload).digest("hex"));
  process.stdout.write(u.toString());
' "$B" "$1" "$2" "$3"; }

# ④ 带有效会话 + 会话签名查询 → 200 且返回 bh2ro 卡片
QURL=$(mk_url "$TOKEN" "$SK" "BG1ABC")
QBODY=$(body "${SH[@]}" "$QURL")
[ "$(echo "$QBODY" | jget success --raw)" = true ] && echo "$QBODY" | grep -q '"cq1"' && ok "4.5④ 有效会话+签名查询 200 且返回 bh2ro 卡片 cq1" || no "4.5④ 有效会话查询失败: $QBODY"

# ⑤ 无会话查询 → 401
[ "$(code "${SH[@]}" "$B/api/callsigns/BG1ABC")" = 401 ] && ok "4.5⑤ 无会话查询 → 401" || no "4.5⑤ 无会话查询未 401"

# ⑥ 伪造 token（错 HMAC）→ 401
FURL=$(mk_url "deadsid.deadbeef" "$SK" "BG1ABC")
[ "$(code "${SH[@]}" "$FURL")" = 401 ] && ok "4.5⑥ 伪造 token 查询 → 401" || no "4.5⑥ 伪造 token 未 401"

# ⑦ 篡改签名（正确 token、错 sk 算签名）→ 401
WURL=$(mk_url "$TOKEN" "wrong-sk-0000000000000000000000000000" "BG1ABC")
[ "$(code "${SH[@]}" "$WURL")" = 401 ] && ok "4.5⑦ 错 sk 签名查询 → 401" || no "4.5⑦ 错 sk 签名未 401"

# ⑧ 换网（不同 binding_key）携带原会话 → 401（IP 绑定生效，防会话搬移）
MOVED=$(mk_url "$TOKEN" "$SK" "BG1ABC")
[ "$(code -H "CF-Connecting-IP: 198.51.100.250" "$MOVED")" = 401 ] && ok "4.5⑧ 会话搬移到不同真实 IP → 401（IP 绑定）" || no "4.5⑧ 会话搬移未被拒"

echo; echo "########## 4.6 auth-callback state 解析 + 租户校验（阶段 4-A）##########"
# worker 仍由 4.5 的 start "$TEST_KEY"（项目 toml：DB+tenants(bh2ro active / gone revoked)+RATE_LIMIT KV）运行。
# 校验在微信换 openid 之前（无凭据公开回调）：有效租户+有效呼号→503(未配微信，证解析+校验通过)；
# 不存在/非活跃租户→400(无效租户)；空呼号→400(缺少呼号)。每断言用不同 CF-IP 避免 authcb 桶串扰。
# 有效租户 tenant:callsign → 503（解析 + 租户校验通过，止于未配微信）
[ "$(code -H "CF-Connecting-IP: 203.0.113.41" "$B/api/wechat/auth-callback?code=x&state=bh2ro:BG1ABC")" = 503 ] && ok "4.6 state=bh2ro:CALL 有效租户 → 503（解析+校验通过）" || no "4.6 state=bh2ro:CALL 未 503"
# 无冒号回退创始租户 bh2ro（有效）→ 503（向前兼容）
[ "$(code -H "CF-Connecting-IP: 203.0.113.42" "$B/api/wechat/auth-callback?code=x&state=BG1ABC")" = 503 ] && ok "4.6 state=CALL 无冒号回退 bh2ro → 503（向前兼容）" || no "4.6 无冒号回退未 503"
# 不存在租户 → 400「无效租户」（拒绝、不落垃圾绑定）
[ "$(code -H "CF-Connecting-IP: 203.0.113.43" "$B/api/wechat/auth-callback?code=x&state=nope:BG1ABC")" = 400 ] && ok "4.6 state=nope:CALL 不存在租户 → 400 拒绝" || no "4.6 不存在租户未 400"
# 非活跃租户(revoked) → 400（证 status='active' 过滤，非仅存在性）
[ "$(code -H "CF-Connecting-IP: 203.0.113.44" "$B/api/wechat/auth-callback?code=x&state=gone:BG1ABC")" = 400 ] && ok "4.6 state=gone:CALL 非活跃租户 → 400 拒绝" || no "4.6 非活跃租户未 400"
# 空呼号(state=bh2ro:) → 400「缺少呼号」
[ "$(code -H "CF-Connecting-IP: 203.0.113.45" "$B/api/wechat/auth-callback?code=x&state=bh2ro:")" = 400 ] && ok "4.6 state=bh2ro: 空呼号 → 400" || no "4.6 空呼号未 400"
# 文案区分：不存在租户响应含「无效租户」（确保非与「缺少呼号」同分支误判）
echo "$(body -H "CF-Connecting-IP: 203.0.113.46" "$B/api/wechat/auth-callback?code=x&state=nope:BG1ABC")" | grep -q "无效租户" && ok "4.6 不存在租户响应含「无效租户」文案" || no "4.6 无效租户文案缺失"
# 畸形 state：%25 经 URLSearchParams 解出裸 '%'，第二次 decodeURIComponent 抛错 → try/catch 兜 400（非顶层 500）
[ "$(code -H "CF-Connecting-IP: 203.0.113.47" "$B/api/wechat/auth-callback?code=x&state=%25")" = 400 ] && ok "4.6 畸形 state(%25→裸%) → 400（非顶层 500）" || no "4.6 畸形 state 未 400（$(code -H "CF-Connecting-IP: 203.0.113.47" "$B/api/wechat/auth-callback?code=x&state=%25")）"
# 超长 state(>256) → 400 参数过长（防超长构造）
LONGSTATE=$(printf 'a%.0s' $(seq 1 300))
[ "$(code -H "CF-Connecting-IP: 203.0.113.48" "$B/api/wechat/auth-callback?code=x&state=$LONGSTATE")" = 400 ] && ok "4.6 超长 state(>256) → 400" || no "4.6 超长 state 未 400"
# callsign 字符白名单：含 HTML 元字符的 callsign（%3Cscript%3E→<SCRIPT>）→ 400「无效呼号」（堵成功页反射 XSS 源）
[ "$(code -H "CF-Connecting-IP: 203.0.113.49" "$B/api/wechat/auth-callback?code=x&state=bh2ro:%3Cscript%3E")" = 400 ] && ok "4.6 callsign 含 HTML 元字符 → 400（XSS 源被白名单拒）" || no "4.6 XSS 构造 callsign 未 400"

# ⑨ KV 未绑（SESSION_SECRET 已配，仅缺 RATE_LIMIT KV）→ 会话端点 503 fail-closed
start_nokv
[ "$(code "$B/api/session/challenge")" = 503 ] && ok "4.5⑨ KV 未绑 → /api/session/challenge 503（fail-closed）" || no "4.5⑨ KV 未绑未 503（$(code "$B/api/session/challenge")）"
[ "$(code "$B/api/callsigns/BG1ABC")" = 503 ] && ok "4.5⑨ KV 未绑 → 无会话查询 503（fail-closed，不静默放行）" || no "4.5⑨ KV 未绑查询未 503"

rm -f "$CDN_TOML" "$NOKV_TOML" 2>/dev/null || true

echo; echo "########## 4.7 声明租户交叉校验 X-Tenant-Id (4-C1) ##########"
# 复位 bh2ro 活跃凭据 + 计数；种一把【表驱动】非 env.API_KEY 的 Key（证 /ping 走 resolveTenant、表驱动也能测通）
TD_KEY="table-driven-key-zzz"
TD_HASH=$(node -e 'process.stdout.write(require("crypto").createHash("sha256").update((process.argv[1]||"").trim()).digest("hex"))' "$TD_KEY")
d1 --command "UPDATE tenant_credentials SET status='active' WHERE id='bh2ro-key'; UPDATE service_counters SET count=0 WHERE name='auth_fallback'; INSERT OR IGNORE INTO tenant_credentials (id,tenant_id,scope,key_hash,status) VALUES ('td-key','bh2ro','sync','$TD_HASH','active');" >/dev/null
start "$TEST_KEY"

# ① 表驱动非 env Key 在 /ping 测通（旧实现只 env.API_KEY 那把通；本次应 200 + 回显 tenant=bh2ro fallback=false）
PING_TD=$(body -H "Authorization: Bearer $TD_KEY" "$B/ping")
[ "$(echo "$PING_TD" | jget success --raw)" = true ] && ok "4.7① 表驱动非 env Key 在 /ping 测通(200,修复阶段1缺口)" || no "4.7① 表驱动 Key /ping 未通: $PING_TD"
[ "$(echo "$PING_TD" | jget tenant --raw)" = bh2ro ] && [ "$(echo "$PING_TD" | jget fallback --raw)" = false ] && ok "4.7① /ping 回显 tenant=bh2ro fallback=false" || no "4.7① /ping 回显异常: $PING_TD"

# ② /ping X-Tenant-Id 一致→200 / 不一致→403
[ "$(code -H "Authorization: Bearer $TD_KEY" -H "X-Tenant-Id: bh2ro" "$B/ping")" = 200 ] && ok "4.7② /ping X-Tenant-Id=bh2ro 一致 → 200" || no "4.7② /ping 一致未 200"
[ "$(code -H "Authorization: Bearer $TD_KEY" -H "X-Tenant-Id: other" "$B/ping")" = 403 ] && ok "4.7② /ping X-Tenant-Id=other 不一致 → 403" || no "4.7② /ping 不一致未 403"

# ③ /sync 声明不一致 → 403 且零数据改动（红线：绝不据声明值写入）。先立已知基线指纹。
d1 --command "DELETE FROM cards WHERE tenant_id='bh2ro'; DELETE FROM cards WHERE tenant_id='other'; INSERT INTO cards (tenant_id,id,project_id,callsign,qty,status,created_at,updated_at) VALUES ('bh2ro','baseline','p1','BG1ABC',1,'pending','t','t');" >/dev/null
FP_BEFORE=$(fp bh2ro)
[ "$(code -H "Authorization: Bearer $TEST_KEY" -H "X-Tenant-Id: other" -H 'Content-Type: application/json' -d '{"client_id":"cmm","data":{"cards":[{"id":"X","project_id":"p1","callsign":"EVIL","qty":9}]}}' "$B/sync")" = 403 ] && ok "4.7③ /sync X-Tenant-Id=other 不一致 → 403" || no "4.7③ /sync 不一致未 403"
[ "$(fp bh2ro)" = "$FP_BEFORE" ] && ok "4.7③ 红线:403 零数据改动(bh2ro 指纹不变,未进 DELETE/INSERT)" || no "4.7③ 403 后 bh2ro 数据被改动"
[ "$(d1json "SELECT count(*) AS n FROM cards WHERE tenant_id='other'")" = '[{"n":0}]' ] && ok "4.7③ 红线:声明 other 名下零行(声明值绝不当写入目标)" || no "4.7③ 声明值被当写入目标"

# ④ 一致 bh2ro → 200；缺头 → 向后兼容 200；/pull 同
[ "$(code -H "Authorization: Bearer $TEST_KEY" -H "X-Tenant-Id: bh2ro" -H 'Content-Type: application/json' -d '{"client_id":"cok","data":{"cards":[]}}' "$B/sync")" = 200 ] && ok "4.7④ /sync X-Tenant-Id=bh2ro 一致 → 200" || no "4.7④ /sync 一致未 200"
[ "$(code -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d '{"client_id":"cbc","data":{"cards":[]}}' "$B/sync")" = 200 ] && ok "4.7④ /sync 缺 X-Tenant-Id 向后兼容 → 200" || no "4.7④ 缺头未向后兼容放行"
# 纯空白 X-Tenant-Id → 200。注：curl 多丢弃纯空白值头、workerd 也剥 OWS，故此条实际等价「缺头放行」、
# 不穿 handler .trim()（不对 trim 删除变异敏感）；仅作「空白头不致 403」的契约确认。
[ "$(code -H "Authorization: Bearer $TEST_KEY" -H "X-Tenant-Id:    " -H 'Content-Type: application/json' -d '{"client_id":"cws","data":{"cards":[]}}' "$B/sync")" = 200 ] && ok "4.7④ /sync 纯空白 X-Tenant-Id 不致 403 → 200（等价缺头）" || no "4.7④ 空白头致 403"
# 有效 slug + 尾随空白（本地 miniflare 保留 OWS、达 worker）→ handler .trim() 后 ==bh2ro 一致放行 200；
# 删 handler .trim() 则 "bh2ro  "!==bh2ro → 403。【此断言对 trim 删除变异可检，非假绿，真覆盖 handler .trim()】
[ "$(code -H "Authorization: Bearer $TEST_KEY" -H "X-Tenant-Id: bh2ro  " -H 'Content-Type: application/json' -d '{"client_id":"cwsv","data":{"cards":[]}}' "$B/sync")" = 200 ] && ok "4.7④ /sync 有效 slug+尾随空白 经 handler trim 一致 → 200（变异可检 trim）" || no "4.7④ 尾随空白 slug 未经 trim 放行（trim 契约破/被删）"
[ "$(code -H "Authorization: Bearer $TEST_KEY" "$B/pull")" = 200 ] && ok "4.7④ /pull 缺 X-Tenant-Id 向后兼容 → 200" || no "4.7④ /pull 缺头未 200"
[ "$(code -H "Authorization: Bearer $TEST_KEY" -H "X-Tenant-Id: other" "$B/pull")" = 403 ] && ok "4.7④ /pull X-Tenant-Id=other 不一致 → 403" || no "4.7④ /pull 不一致未 403"

# ⑤ /ping 经兜底命中只读不计兜底；/sync 兜底命中计数（对照）。撤 bh2ro-key+td-key 使表 miss、TEST_KEY 走 env 兜底。
d1 --command "UPDATE tenant_credentials SET status='revoked' WHERE id IN ('bh2ro-key','td-key'); UPDATE service_counters SET count=0 WHERE name='auth_fallback';" >/dev/null
PING_FB=$(body -H "Authorization: Bearer $TEST_KEY" "$B/ping")
[ "$(echo "$PING_FB" | jget fallback --raw)" = true ] && ok "4.7⑤ /ping 经 env 兜底命中 → fallback=true" || no "4.7⑤ /ping 兜底回显异常: $PING_FB"
[ "$(d1json "SELECT count FROM service_counters WHERE name='auth_fallback'")" = '[{"count":0}]' ] && ok "4.7⑤ /ping 只读:兜底命中 auth_fallback 不变(0)" || no "4.7⑤ /ping 污染了兜底计数"
d1 --command "UPDATE service_counters SET count=0 WHERE name='auth_fallback';" >/dev/null
code -H "Authorization: Bearer $TEST_KEY" -H 'Content-Type: application/json' -d '{"client_id":"cfb","data":{"cards":[]}}' "$B/sync" >/dev/null
[ "$(d1json "SELECT count FROM service_counters WHERE name='auth_fallback'")" = '[{"count":1}]' ] && ok "4.7⑤ 对照:/sync 兜底命中 auth_fallback 0->1(仍计数)" || no "4.7⑤ /sync 兜底未计数"
d1 --command "UPDATE tenant_credentials SET status='active' WHERE id='bh2ro-key'; DELETE FROM tenant_credentials WHERE id='td-key'; UPDATE service_counters SET count=0 WHERE name='auth_fallback';" >/dev/null

# ⑥ CORS 放行 X-Tenant-Id
[ "$(curl -s -m 8 -D - -o /dev/null "$B/api/config" | grep -i 'access-control-allow-headers' | grep -ic 'x-tenant-id')" -ge 1 ] && ok "4.7⑥ CORS Allow-Headers 含 X-Tenant-Id" || no "4.7⑥ CORS 未放行 X-Tenant-Id"

stop
echo
echo "==== worker 冒烟：PASS=$PASS FAIL=$FAIL ===="
[ "$FAIL" = 0 ] && exit 0 || exit 1
