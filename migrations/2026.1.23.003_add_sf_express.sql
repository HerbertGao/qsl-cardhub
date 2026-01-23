-- 2026.1.23.003_add_sf_express.sql
-- 顺丰速运相关表

-- 寄件人表
CREATE TABLE IF NOT EXISTS sf_senders (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    phone TEXT NOT NULL,
    mobile TEXT,
    province TEXT NOT NULL,
    city TEXT NOT NULL,
    district TEXT NOT NULL,
    address TEXT NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_sf_senders_is_default ON sf_senders(is_default);

-- 顺丰订单表（包含 pay_method 和 cargo_name 字段）
CREATE TABLE IF NOT EXISTS sf_orders (
    id TEXT PRIMARY KEY,
    order_id TEXT NOT NULL UNIQUE,
    waybill_no TEXT,
    card_id TEXT,
    status TEXT NOT NULL CHECK(status IN ('pending', 'confirmed', 'cancelled', 'printed')) DEFAULT 'pending',
    pay_method INTEGER DEFAULT 1,
    cargo_name TEXT DEFAULT 'QSL卡片',
    sender_info TEXT NOT NULL,  -- JSON 字符串
    recipient_info TEXT NOT NULL,  -- JSON 字符串
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE SET NULL
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_sf_orders_order_id ON sf_orders(order_id);
CREATE INDEX IF NOT EXISTS idx_sf_orders_waybill_no ON sf_orders(waybill_no);
CREATE INDEX IF NOT EXISTS idx_sf_orders_card_id ON sf_orders(card_id);
CREATE INDEX IF NOT EXISTS idx_sf_orders_status ON sf_orders(status);
CREATE INDEX IF NOT EXISTS idx_sf_orders_created_at ON sf_orders(created_at DESC);
