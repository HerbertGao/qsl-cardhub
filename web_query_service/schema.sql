-- QSL CardHub 云端 D1 数据库 schema
-- 按 tenant_id 行级隔离多租户同步数据；顺丰推送与微信绑定表本期保持全局（无 tenant 列，阶段 4 才迁）
-- 租户身份由写入 Key 解析（见 tenant_credentials），禁止取请求体自报的 client_id 决定数据归属

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
CREATE TABLE IF NOT EXISTS service_counters (
    name TEXT PRIMARY KEY,
    count INTEGER NOT NULL DEFAULT 0
);

-- 同步元数据（PK tenant_id；last_client_id 溯源；server_version 为后续乐观并发护栏预留）
CREATE TABLE IF NOT EXISTS sync_meta (
    tenant_id TEXT PRIMARY KEY,
    last_client_id TEXT,
    server_version INTEGER NOT NULL DEFAULT 0,
    sync_time TEXT,
    received_at TEXT
);

-- 项目表（含 tenant_id 行级隔离）
CREATE TABLE IF NOT EXISTS projects (
    tenant_id TEXT NOT NULL,
    id TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (tenant_id, id)
);
CREATE INDEX IF NOT EXISTS idx_projects_created_at ON projects(created_at DESC);

-- 卡片表（含 tenant_id 行级隔离）
CREATE TABLE IF NOT EXISTS cards (
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
CREATE INDEX IF NOT EXISTS idx_cards_tenant_callsign ON cards(tenant_id, callsign COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_cards_project ON cards(project_id);
CREATE INDEX IF NOT EXISTS idx_cards_created_at ON cards(created_at DESC);

-- 顺丰寄件人表（含 tenant_id 行级隔离）
CREATE TABLE IF NOT EXISTS sf_senders (
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

-- 顺丰订单表（含 tenant_id 行级隔离；用于顺丰推送解析呼号：order_id/waybill_no -> card_id -> cards.callsign）
CREATE TABLE IF NOT EXISTS sf_orders (
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
CREATE INDEX IF NOT EXISTS idx_sf_orders_order_id ON sf_orders(order_id);
CREATE INDEX IF NOT EXISTS idx_sf_orders_waybill_no ON sf_orders(waybill_no);
CREATE INDEX IF NOT EXISTS idx_sf_orders_card_id ON sf_orders(card_id);

-- 全局配置项表（含 tenant_id 行级隔离）
CREATE TABLE IF NOT EXISTS app_settings (
    tenant_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (tenant_id, key)
);

-- 呼号–微信 openid 绑定表（订阅收卡后写入；顺丰推送时按租户+呼号查 openid 发模板消息）
-- 阶段 4-A 已租户化：加 tenant_id 行级隔离键，主键 (tenant_id, callsign, openid)。
-- auth-callback 由授权 state（tenant:callsign，无冒号回退 bh2ro）解析并校验租户后写入；
-- route-push 由匹配的 sf_orders 派生 tenant 后按 WHERE tenant_id=? AND callsign=? 反查 openid。
CREATE TABLE IF NOT EXISTS callsign_openid_bindings (
    tenant_id TEXT NOT NULL,
    callsign TEXT NOT NULL,
    openid TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (tenant_id, callsign, openid)
);
-- 索引服务 route-push 的 openid 反查 WHERE tenant_id=? AND callsign=? COLLATE NOCASE：
-- 以 (tenant_id, callsign COLLATE NOCASE) 复合且大小写不敏感，与全系统 callsign 比较的 NOCASE 约定一致。
CREATE INDEX IF NOT EXISTS idx_bindings_callsign ON callsign_openid_bindings(tenant_id, callsign COLLATE NOCASE);

-- 顺丰路由推送去重/记录（同一 mailno+opCode+id 不重复处理）
-- 全局表、不加 tenant_id：顺丰 waybill 全局唯一，去重维度全局；推送目标租户由匹配的
-- sf_orders join 派生（见 multi-tenant-design.md §6），不单独隔离去重维度。
CREATE TABLE IF NOT EXISTS sf_route_log (
    id TEXT PRIMARY KEY,
    mailno TEXT,
    orderid TEXT,
    op_code TEXT,
    accept_time TEXT,
    remark TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_sf_route_mailno_op ON sf_route_log(mailno, op_code, id);
