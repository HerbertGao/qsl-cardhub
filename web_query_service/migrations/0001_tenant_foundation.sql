-- ============================================================================
-- 0001_tenant_foundation.sql — 多租户地基（阶段 1）生产 D1 一次性迁移
-- ============================================================================
-- 目标 D1 绑定：wrangler.toml 中 [[d1_databases]] binding="DB"，database_name="qsl-sync"
-- 执行命令（由运维在自己终端跑，禁止自动化代理代跑生产迁移）：
--   wrangler d1 execute qsl-sync --file migrations/0001_tenant_foundation.sql --remote
--
-- 本文件把单租户旧 schema（业务表以 client_id 隔离）演进为行级 tenant_id 多租户，
-- 既有数据全部回填到内置 default 租户；对外可见行为不变（tenant_id 恒为 'default'）。
--
-- ----------------------------------------------------------------------------
-- 原子性（务必理解，否则会写错事务语句）
-- ----------------------------------------------------------------------------
-- Cloudflare D1 不支持在 SQL 文件里写显式 BEGIN TRANSACTION / COMMIT / SAVEPOINT，
-- 这些语句经 wrangler d1 execute --file 会以 not authorized: SQLITE_AUTH 失败。
-- 本文件因此【禁止】出现任何 BEGIN/COMMIT/SAVEPOINT。
-- 原子性来自 Cloudflare 的保证：整份迁移 SQL 由 --file --remote 单次执行，
-- 任一语句失败则 DB 返回原状、可安全重试（全文件回滚，不留半迁移）。
--
-- ----------------------------------------------------------------------------
-- 执行前置（均由用户在自己终端执行，代理不碰生产/secret 明文）
-- ----------------------------------------------------------------------------
-- 1) 全量备份作回滚点（默认含 schema+data）：
--      wrangler d1 export qsl-sync --remote --output backup-before-0001.sql
--    并记录当前生产 worker 部署版本号（Cloudflare Deployments）或对应 git commit
--    作为回滚步骤④的目标。
--
-- 2) 单一所有者校验（任务 1.1）——对每张业务表 + sync_meta 跑，确认 client_id 单一：
--      SELECT client_id, COUNT(*) FROM projects     GROUP BY client_id;
--      SELECT client_id, COUNT(*) FROM cards         GROUP BY client_id;
--      SELECT client_id, COUNT(*) FROM sf_senders    GROUP BY client_id;
--      SELECT client_id, COUNT(*) FROM sf_orders     GROUP BY client_id;
--      SELECT client_id, COUNT(*) FROM app_settings  GROUP BY client_id;
--      SELECT client_id, COUNT(*) FROM sync_meta     GROUP BY client_id;
--    sync_meta 因 worker 从不删旧行、换机/重装会累积多行，必须一并纳入。
--    若任一表出现多个 client_id，【中止迁移】，按以下步骤人工处置后重来：
--      a. 确认仅一台机器同步过业务数据（与使用者核对）；
--      b. 若确属同一逻辑所有者的历史多机残留，择 received_at 最新一条保留、其余删除，
--         例如 sync_meta：
--           DELETE FROM sync_meta
--           WHERE rowid NOT IN (
--             SELECT rowid FROM sync_meta
--             ORDER BY received_at DESC, client_id DESC LIMIT 1
--           );
--      c. 业务表若出现多 client_id（理论上「全量覆盖」模型不应发生），先排查异常来源，
--         不要盲删；确认后同法择最新所有者保留、余删。
--
-- 3) CHECK 越界史数据预扫——新表 cards / sf_orders 含旧表所无的 CHECK 约束，若旧库存在
--    违反新 CHECK 的脏行，INSERT…SELECT 回填会被新表 CHECK 拒、整文件回滚。须先预扫清洗：
--      SELECT count(*) FROM cards
--        WHERE NOT(qty > 0 AND qty <= 9999)
--           OR status NOT IN ('pending', 'distributed', 'returned');
--      SELECT count(*) FROM sf_orders
--        WHERE status NOT IN ('pending', 'confirmed', 'cancelled', 'printed');
--    两条均须返回 0。非 0 说明旧库有违反新 CHECK 的脏行，须先人工清洗后再迁，否则迁移必回滚。
--
-- 4) 冻结桌面端写入：迁移执行窗口内停止桌面端 /sync（避免「校验后、迁移前又同步引入
--    异常行」竞态）。
--
-- 5) 离线计算 default 写凭据 hash（与 worker JS String.prototype.trim() 逐字符同语义，
--    仅去首尾空白；输出 64 位小写 hex，无前缀）：
--    API_KEY 必须非空高熵值；空则本命令拒绝输出，不得把 `sha256('')` 当凭据 seed。
--      node -e 'const k=(process.env.API_KEY||"").trim(); if(!k){console.error("API_KEY 为空——拒绝 seed（避免 sha256(\"\") 成无鉴权凭据）");process.exit(1)} process.stdout.write(require("crypto").createHash("sha256").update(k).digest("hex"))'
--    【禁用】tr -d '[:space:]'（去全部空白含中间，与 JS trim 仅去首尾不等价，
--    Key 含中间空白时 hash 会不一致 → 表驱动永久 miss 全程兜底）。
--    把输出替换本文件下方 tenant_credentials seed 里的占位符
--    '<占位:离线 sha256(trim(API_KEY))>'（连同尖括号整段替换）。
--
-- ----------------------------------------------------------------------------
-- 执行后自检（务必看）
-- ----------------------------------------------------------------------------
-- 本文件【末尾】自带占位符残留自检：
--   SELECT key_hash FROM tenant_credentials WHERE key_hash LIKE '<占位%';
-- 该自检是【强制门】：返回任何行即视为迁移失败，【必须】回滚
-- （D1 顶层无 RAISE，故靠运维核对此输出 + 部署后 count===0 双重兜住）。
-- 风险更正：worker 比对的是 sha256(trim(key))，而占位行 key_hash 存的是占位串本身
-- （非其 sha256），故占位串【不可直接当 Bearer 用】；真实风险是占位行占用
-- idx_tenant_credentials_active_key_hash 的 active 唯一槽位 → 表驱动对真实 Key 永久 miss
-- → 全程走兜底，由部署后验收「auth_fallback count===0」兜住。
-- 注：此自检仅防占位残留，不证明 hash 算对；hash 正确性由部署后验收
-- 「兜底计数 service_counters('auth_fallback').count === 0」兜住（与本迁移属同一
-- 不可分交付单元）。
--
-- ----------------------------------------------------------------------------
-- 回滚剧本（有序 fail-closed，前滚优先）
-- ----------------------------------------------------------------------------
-- 迁移后 schema 与 worker 强耦合：
--   - 旧 worker 全是 client_id 列 SQL，读迁后的 tenant_id 表 → no such column 整站 500；
--   - 新 worker 依赖 tenant_credentials 等新表，恢复旧 dump 后这些表消失 → resolveTenant
--     查无表整站 500。
-- 故【既不能单独退 worker、也不能单独退数据】，回滚是配对动作，且运维上不可真原子。
-- 按以下 fail-closed 顺序执行（任一中间态皆为 500/503，非裸写）：
--   ① 新 worker 下线：部署一个对所有路由返 503 的临时 worker 版本，
--      或在 Cloudflare dashboard 解绑生产路由。
--   ② 清空迁移后 D1 的全部表。【推荐】建一个新空 D1 + 改 wrangler.toml 的
--      [[d1_databases]].database_id 绑定，天然规避「逐表 DROP 漏表」。
--      若坚持逐表 DROP，须 DROP【全部 12 张表】，漏清任何一张 → 旧 dump 的 INSERT
--      撞 PK 致 import 中止：
--        三租户表：tenants / tenant_credentials / tenant_routes
--        计数表：  service_counters
--        业务 5 表：projects / cards / sf_senders / sf_orders / app_settings
--        同步元：  sync_meta
--        全局表：  callsign_openid_bindings / sf_route_log
--                  （本期未迁，但旧 dump 含其数据；漏清 → import 撞 PK 失败）
--   ③ 导入步骤 1 的旧 dump（dump 不自动 DROP 已存在表，故须先清）：
--        wrangler d1 execute qsl-sync --file backup-before-0001.sql --remote
--   ④ 部署回迁移前的旧 worker（旧 dump 含 client_id 列、与旧 worker 匹配）。
--   ⑤ 撤下线 / 恢复路由。
-- 更稳的实践是【只前滚】：迁前用步骤 1 的 dump + 行数核对确保备份完整可重建，
-- 把回滚当最后手段。
-- ============================================================================


-- ============================================================================
-- 第 1 部分：全新表（生产 D1 无旧表，直接 CREATE，不走「建新→回填→DROP→RENAME」）
--   三租户表 + service_counters。列定义/CHECK/索引与 schema.sql 源逐字一致。
-- ============================================================================

-- 租户主表（tenant_id 为人类可读 slug）
CREATE TABLE IF NOT EXISTS tenants (
    tenant_id TEXT NOT NULL PRIMARY KEY CHECK (length(tenant_id) BETWEEN 1 AND 32 AND tenant_id NOT GLOB '*[^a-z0-9-]*'),
    name TEXT,
    tier TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 写入凭据表（key_hash = sha256(trim(key))，禁存明文 Key；命中得 tenant_id；支持多 Key → 同一租户）
CREATE TABLE IF NOT EXISTS tenant_credentials (
    id TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    scope TEXT,
    key_hash TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'revoked')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_used_at TEXT
);
-- 同一 key_hash 在 status='active' 下唯一（禁止一把 Key 解析到两个租户）
CREATE UNIQUE INDEX IF NOT EXISTS idx_tenant_credentials_active_key_hash
    ON tenant_credentials(key_hash) WHERE status='active';

-- host/path → 租户路由表（本期建表即可，路由解析逻辑属后续阶段）
CREATE TABLE IF NOT EXISTS tenant_routes (
    route_key TEXT NOT NULL PRIMARY KEY,
    tenant_id TEXT NOT NULL
);

-- 服务级计数器（兜底命中等内部计数；resolveTenant 兜底命中递增 name='auth_fallback' 行）
-- 生产 D1 本无此表，须在 seed（第 5 部分）之前建好，否则 seed INSERT 会 no such table 致整文件回滚。
CREATE TABLE IF NOT EXISTS service_counters (
    name TEXT PRIMARY KEY,
    count INTEGER NOT NULL DEFAULT 0
);


-- ============================================================================
-- 第 2 部分：业务表重建（projects / cards / sf_senders / sf_orders / app_settings）
--   逐表「建新表(临时名，含 tenant_id/新 PK/全部业务索引) → INSERT…SELECT 回填
--   tenant_id='default' → DROP 旧表 → RENAME」。索引在建新表步一并建好（随表名走）。
--   隔离列 client_id 映射为 tenant_id='default'，其余业务列原样拷贝。
-- ============================================================================

-- ---- projects ----
CREATE TABLE projects_new (
    tenant_id TEXT NOT NULL,
    id TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (tenant_id, id)
);
INSERT INTO projects_new (tenant_id, id, name, created_at, updated_at)
    SELECT 'default', id, name, created_at, updated_at FROM projects;
DROP TABLE projects;
ALTER TABLE projects_new RENAME TO projects;
-- 索引在 DROP 旧表 + RENAME 之后建（旧同名索引随旧表 DROP 消失，绑最终表名，避免全局索引名碰撞）
CREATE INDEX idx_projects_created_at ON projects(created_at DESC);

-- ---- cards ----
CREATE TABLE cards_new (
    tenant_id TEXT NOT NULL,
    id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    creator_id TEXT,
    callsign TEXT NOT NULL,
    qty INTEGER NOT NULL CHECK(qty > 0 AND qty <= 9999),
    serial INTEGER,
    status TEXT NOT NULL CHECK(status IN ('pending', 'distributed', 'returned')) DEFAULT 'pending',
    metadata TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (tenant_id, id)
);
INSERT INTO cards_new (tenant_id, id, project_id, creator_id, callsign, qty, serial, status, metadata, created_at, updated_at)
    SELECT 'default', id, project_id, creator_id, callsign, qty, serial, status, metadata, created_at, updated_at FROM cards;
DROP TABLE cards;
ALTER TABLE cards_new RENAME TO cards;
-- 索引在 DROP 旧表 + RENAME 之后建（旧同名索引随旧表 DROP 消失，绑最终表名，避免全局索引名碰撞）
CREATE INDEX idx_cards_tenant_callsign ON cards(tenant_id, callsign COLLATE NOCASE);
CREATE INDEX idx_cards_project ON cards(project_id);
CREATE INDEX idx_cards_created_at ON cards(created_at DESC);

-- ---- sf_senders ----
CREATE TABLE sf_senders_new (
    tenant_id TEXT NOT NULL,
    id TEXT NOT NULL,
    name TEXT NOT NULL,
    phone TEXT NOT NULL,
    mobile TEXT,
    province TEXT NOT NULL,
    city TEXT NOT NULL,
    district TEXT NOT NULL,
    address TEXT NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (tenant_id, id)
);
INSERT INTO sf_senders_new (tenant_id, id, name, phone, mobile, province, city, district, address, is_default, created_at, updated_at)
    SELECT 'default', id, name, phone, mobile, province, city, district, address, is_default, created_at, updated_at FROM sf_senders;
DROP TABLE sf_senders;
ALTER TABLE sf_senders_new RENAME TO sf_senders;

-- ---- sf_orders ----
CREATE TABLE sf_orders_new (
    tenant_id TEXT NOT NULL,
    id TEXT NOT NULL,
    order_id TEXT NOT NULL,
    waybill_no TEXT,
    card_id TEXT,
    status TEXT NOT NULL CHECK(status IN ('pending', 'confirmed', 'cancelled', 'printed')) DEFAULT 'pending',
    pay_method INTEGER DEFAULT 1,
    cargo_name TEXT DEFAULT 'QSL卡片',
    sender_info TEXT NOT NULL,
    recipient_info TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (tenant_id, id)
);
INSERT INTO sf_orders_new (tenant_id, id, order_id, waybill_no, card_id, status, pay_method, cargo_name, sender_info, recipient_info, created_at, updated_at)
    SELECT 'default', id, order_id, waybill_no, card_id, status, pay_method, cargo_name, sender_info, recipient_info, created_at, updated_at FROM sf_orders;
DROP TABLE sf_orders;
ALTER TABLE sf_orders_new RENAME TO sf_orders;
-- 索引在 DROP 旧表 + RENAME 之后建（旧同名索引随旧表 DROP 消失，绑最终表名，避免全局索引名碰撞）
CREATE INDEX idx_sf_orders_order_id ON sf_orders(order_id);
CREATE INDEX idx_sf_orders_waybill_no ON sf_orders(waybill_no);
CREATE INDEX idx_sf_orders_card_id ON sf_orders(card_id);

-- ---- app_settings ----（PK 为 (tenant_id, key)）
CREATE TABLE app_settings_new (
    tenant_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (tenant_id, key)
);
INSERT INTO app_settings_new (tenant_id, key, value)
    SELECT 'default', key, value FROM app_settings;
DROP TABLE app_settings;
ALTER TABLE app_settings_new RENAME TO app_settings;


-- ============================================================================
-- 第 3 部分：重建 sync_meta 为终态（PK tenant_id）并确定性回填单行
--   现状 PK 为 client_id、可能多行（worker 从不 DELETE 旧行）。
--   只回填一行 'default'：last_client_id 取最新一行 client_id
--   （ORDER BY received_at DESC, client_id DESC LIMIT 1，并列由 client_id 兜底；
--    received_at 可字典序==时序由 serverTime() 定宽零填充 ISO 保证，改其格式须同步校验此 tiebreaker）。
--   空表时子查询返回 NULL（last_client_id=NULL，交首个 /sync 写）。server_version 置 0。
--   禁止照搬多行致 PK 冲突。
-- ============================================================================
CREATE TABLE sync_meta_new (
    tenant_id TEXT PRIMARY KEY,
    last_client_id TEXT,
    server_version INTEGER NOT NULL DEFAULT 0,
    sync_time TEXT,
    received_at TEXT
);
INSERT INTO sync_meta_new (tenant_id, last_client_id, server_version, sync_time, received_at)
SELECT
    'default',
    (SELECT client_id  FROM sync_meta ORDER BY received_at DESC, client_id DESC LIMIT 1),
    0,
    (SELECT sync_time   FROM sync_meta ORDER BY received_at DESC, client_id DESC LIMIT 1),
    (SELECT received_at FROM sync_meta ORDER BY received_at DESC, client_id DESC LIMIT 1);
DROP TABLE sync_meta;
ALTER TABLE sync_meta_new RENAME TO sync_meta;


-- ============================================================================
-- 第 4 部分：seed 内置 default 租户与默认写凭据 + 兜底计数初始行
-- ============================================================================

-- 内置 default 活跃租户
INSERT INTO tenants (tenant_id, name, tier, status)
    VALUES ('default', 'Default Tenant', NULL, 'active');

-- default 活跃写凭据。
-- key_hash 必须为离线计算的 sha256(trim(API_KEY))（64 位小写 hex，无前缀）。
-- 执行前由用户用文件头「执行前置 5)」的 node 命令算出，替换下方占位符整段（连同尖括号）。
-- 占位符必须保留 '<占位' 前缀字面量，使文件末尾自检 LIKE '<占位%' 能命中残留。
-- 风险更正：占位串不可直接当 Bearer 用（worker 比对 sha256(trim(key))，占位行存的是占位串本身、
-- 非其 sha256）；真实风险是占位行占用 active key_hash 唯一槽位 → 表驱动对真实 Key 永久 miss
-- → 全程走兜底，由部署后「auth_fallback count===0」验收兜住。残留即强制门：必须回滚。
INSERT INTO tenant_credentials (id, tenant_id, scope, key_hash, status)
    VALUES ('default-key', 'default', 'sync', '<占位:离线 sha256(trim(API_KEY))>', 'active');

-- 兜底计数初始行（count=0，避免验收读到「行不存在」误判）
INSERT INTO service_counters (name, count) VALUES ('auth_fallback', 0);


-- ============================================================================
-- 第 5 部分：执行后自检——占位符必须已被替换（下行须返回 0 行）
--   若返回任何行 = 占位符残留 = 一条已知公开串成了 active 凭据，视为迁移失败须中止排查。
-- ============================================================================
SELECT key_hash FROM tenant_credentials WHERE key_hash LIKE '<占位%';
