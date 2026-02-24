-- QSL CardHub 云端 D1 数据库 schema
-- 按 client_id 隔离多端同步数据；顺丰推送与微信绑定表可选使用

-- 同步元数据（可选：记录各 client 最近同步时间）
CREATE TABLE IF NOT EXISTS sync_meta (
    client_id TEXT PRIMARY KEY,
    sync_time TEXT,
    received_at TEXT
);

-- 项目表（含 client_id 隔离）
CREATE TABLE IF NOT EXISTS projects (
    client_id TEXT NOT NULL,
    id TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (client_id, id)
);
CREATE INDEX IF NOT EXISTS idx_projects_client ON projects(client_id);
CREATE INDEX IF NOT EXISTS idx_projects_created_at ON projects(created_at DESC);

-- 卡片表（含 client_id 隔离）
CREATE TABLE IF NOT EXISTS cards (
    client_id TEXT NOT NULL,
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
    PRIMARY KEY (client_id, id)
);
CREATE INDEX IF NOT EXISTS idx_cards_client ON cards(client_id);
CREATE INDEX IF NOT EXISTS idx_cards_callsign ON cards(callsign);
CREATE INDEX IF NOT EXISTS idx_cards_project ON cards(project_id);
CREATE INDEX IF NOT EXISTS idx_cards_created_at ON cards(created_at DESC);

-- 顺丰寄件人表（含 client_id 隔离）
CREATE TABLE IF NOT EXISTS sf_senders (
    client_id TEXT NOT NULL,
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
    PRIMARY KEY (client_id, id)
);
CREATE INDEX IF NOT EXISTS idx_sf_senders_client ON sf_senders(client_id);

-- 顺丰订单表（含 client_id 隔离；用于顺丰推送解析呼号：order_id/waybill_no -> card_id -> cards.callsign）
CREATE TABLE IF NOT EXISTS sf_orders (
    client_id TEXT NOT NULL,
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
    PRIMARY KEY (client_id, id)
);
CREATE INDEX IF NOT EXISTS idx_sf_orders_client ON sf_orders(client_id);
CREATE INDEX IF NOT EXISTS idx_sf_orders_order_id ON sf_orders(order_id);
CREATE INDEX IF NOT EXISTS idx_sf_orders_waybill_no ON sf_orders(waybill_no);
CREATE INDEX IF NOT EXISTS idx_sf_orders_card_id ON sf_orders(card_id);

-- 全局配置项表（含 client_id 隔离）
CREATE TABLE IF NOT EXISTS app_settings (
    client_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (client_id, key)
);
CREATE INDEX IF NOT EXISTS idx_app_settings_client ON app_settings(client_id);

-- 呼号–微信 openid 绑定表（订阅收卡后写入；顺丰推送时按呼号查 openid 发模板消息）
CREATE TABLE IF NOT EXISTS callsign_openid_bindings (
    callsign TEXT NOT NULL,
    openid TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (callsign, openid)
);
CREATE INDEX IF NOT EXISTS idx_bindings_callsign ON callsign_openid_bindings(callsign);

-- 顺丰路由推送去重/记录（可选：同一 mailno+opCode+id 不重复处理）
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
