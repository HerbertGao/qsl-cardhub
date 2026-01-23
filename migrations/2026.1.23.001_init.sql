-- 2026.1.23.001_init.sql
-- 数据库初始化脚本
-- 创建 projects 表

-- 转卡项目表
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 创建索引：按创建时间排序
CREATE INDEX IF NOT EXISTS idx_projects_created_at ON projects(created_at DESC);
