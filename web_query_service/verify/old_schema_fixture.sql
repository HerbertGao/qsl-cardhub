-- old_schema_fixture.sql — 迁移前真实单租户旧结构（逐字取自 git HEAD:web_query_service/schema.sql）
-- + 样例数据。用于离线对 0001_tenant_foundation.sql 主体（第1–5部分）做迁移正确性验证。
-- 关键：旧业务表 PK 为 (client_id, id)；旧 sync_meta 无 server_version 列；含 idx_*_created_at /
-- idx_cards_project / idx_sf_orders_* 等与迁移新表重名的索引（验证全局索引命名空间碰撞）。

-- 同步元数据（旧：PK client_id，无 server_version 列，可能多行）
CREATE TABLE sync_meta (
    client_id TEXT PRIMARY KEY,
    sync_time TEXT,
    received_at TEXT
);

CREATE TABLE projects (
    client_id TEXT NOT NULL,
    id TEXT NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (client_id, id)
);
CREATE INDEX idx_projects_client ON projects(client_id);
CREATE INDEX idx_projects_created_at ON projects(created_at DESC);

CREATE TABLE cards (
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
CREATE INDEX idx_cards_client ON cards(client_id);
CREATE INDEX idx_cards_callsign ON cards(callsign);
CREATE INDEX idx_cards_project ON cards(project_id);
CREATE INDEX idx_cards_created_at ON cards(created_at DESC);

CREATE TABLE sf_senders (
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
CREATE INDEX idx_sf_senders_client ON sf_senders(client_id);

CREATE TABLE sf_orders (
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
CREATE INDEX idx_sf_orders_client ON sf_orders(client_id);
CREATE INDEX idx_sf_orders_order_id ON sf_orders(order_id);
CREATE INDEX idx_sf_orders_waybill_no ON sf_orders(waybill_no);
CREATE INDEX idx_sf_orders_card_id ON sf_orders(card_id);

CREATE TABLE app_settings (
    client_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (client_id, key)
);
CREATE INDEX idx_app_settings_client ON app_settings(client_id);

-- 全局表（本期不迁，但旧库含；迁移主体不碰它们）
CREATE TABLE callsign_openid_bindings (
    callsign TEXT NOT NULL,
    openid TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (callsign, openid)
);
CREATE INDEX idx_bindings_callsign ON callsign_openid_bindings(callsign);

CREATE TABLE sf_route_log (
    id TEXT PRIMARY KEY,
    mailno TEXT,
    orderid TEXT,
    op_code TEXT,
    accept_time TEXT,
    remark TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_sf_route_mailno_op ON sf_route_log(mailno, op_code, id);

-- ============================================================
-- 样例数据（单一逻辑所有者 cli-A；sync_meta 故意造多行模拟换机残留）
-- ============================================================
INSERT INTO projects (client_id, id, name, created_at, updated_at) VALUES
    ('cli-A', 'p1', '项目一', '2026-01-01T00:00:00+00:00', '2026-01-01T00:00:00+00:00'),
    ('cli-A', 'p2', '项目二', '2026-01-02T00:00:00+00:00', '2026-01-02T00:00:00+00:00');

INSERT INTO cards (client_id, id, project_id, creator_id, callsign, qty, serial, status, metadata, created_at, updated_at) VALUES
    ('cli-A', 'c1', 'p1', NULL, 'BG1ABC', 5, 1, 'pending', NULL, '2026-01-03T00:00:00+00:00', '2026-01-03T00:00:00+00:00'),
    ('cli-A', 'c2', 'p1', NULL, 'bg1abc', 3, 2, 'distributed', '{"distribution":{"method":"sf"}}', '2026-01-04T00:00:00+00:00', '2026-01-04T00:00:00+00:00'),
    ('cli-A', 'c3', 'p2', NULL, 'BD2XYZ', 1, 3, 'returned', NULL, '2026-01-05T00:00:00+00:00', '2026-01-05T00:00:00+00:00');

INSERT INTO sf_senders (client_id, id, name, phone, mobile, province, city, district, address, is_default, created_at, updated_at) VALUES
    ('cli-A', 's1', '张三', '12345678901', NULL, '广东', '深圳', '南山', '科技园', 1, '2026-01-06T00:00:00+00:00', '2026-01-06T00:00:00+00:00');

INSERT INTO sf_orders (client_id, id, order_id, waybill_no, card_id, status, pay_method, cargo_name, sender_info, recipient_info, created_at, updated_at) VALUES
    ('cli-A', 'o1', 'ORDER-001', 'SF1234567890', 'c1', 'confirmed', 1, 'QSL卡片', '{}', '{}', '2026-01-07T00:00:00+00:00', '2026-01-07T00:00:00+00:00');

INSERT INTO app_settings (client_id, key, value) VALUES
    ('cli-A', 'theme', 'dark'),
    ('cli-A', 'lang', 'zh');

-- sync_meta 多行（换机残留）：received_at 字典序最新者为 cli-A-new
INSERT INTO sync_meta (client_id, sync_time, received_at) VALUES
    ('cli-A-old', '2026-01-01T00:00:00+00:00', '2026-01-01T00:00:00+00:00'),
    ('cli-A-new', '2026-02-01T00:00:00+00:00', '2026-02-01T00:00:00+00:00');
