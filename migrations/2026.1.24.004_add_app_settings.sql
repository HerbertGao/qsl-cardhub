-- 全局配置表
CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- 插入默认值
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('label_title', '中国无线电协会业余分会-2区卡片局');
