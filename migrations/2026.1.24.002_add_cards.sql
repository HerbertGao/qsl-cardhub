-- 2026.1.24.002_add_cards.sql
-- 添加卡片表

-- 卡片表
CREATE TABLE IF NOT EXISTS cards (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    creator_id TEXT,  -- 预留字段，Phase 3 使用
    callsign TEXT NOT NULL,
    qty INTEGER NOT NULL CHECK(qty > 0 AND qty <= 9999),
    serial INTEGER,  -- 序列号，前端显示时格式化为三位数如 "001"
    status TEXT NOT NULL CHECK(status IN ('pending', 'distributed', 'returned')) DEFAULT 'pending',
    metadata TEXT,  -- JSON 字符串，存储分发/退卡信息
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- 为已存在的表添加 serial 列（如果不存在）
-- 使用 PRAGMA table_info 检查列是否存在的方式在 SQLite 中不支持条件 ALTER TABLE
-- 但是 ALTER TABLE ADD COLUMN 在列已存在时会失败，所以我们需要一个安全的方式
-- SQLite 允许我们通过检查错误来处理这种情况，但在迁移脚本中我们使用一个更安全的方法：
-- 先检查表是否存在且没有 serial 列，如果是则添加
ALTER TABLE cards ADD COLUMN serial INTEGER;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_cards_project ON cards(project_id);
CREATE INDEX IF NOT EXISTS idx_cards_callsign ON cards(callsign);
CREATE INDEX IF NOT EXISTS idx_cards_status ON cards(status);
CREATE INDEX IF NOT EXISTS idx_cards_created_at ON cards(created_at DESC);
