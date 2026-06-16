-- QSL CardHub 云端 D1 迁移 0002：全局绑定表租户化（阶段 4-A）
-- ============================================================================
-- 作用：仅重建 callsign_openid_bindings——加 tenant_id 行级隔离键，主键演进为
--   (tenant_id, callsign, openid)，存量行回填创始租户 'bh2ro'。sf_route_log 保持
--   全局不动（顺丰 waybill 全局唯一，去重维度全局；其 tenant 由匹配的 sf_orders
--   join 派生，不单独隔离去重维度——见 multi-tenant-design.md §6）。
--
-- 前置：0001_tenant_foundation.sql 已执行（业务 5 表 + sync_meta 已租户化、bh2ro
--   租户已 seed）。callsign_openid_bindings 由更早的 wechat-push 迁移创建、0001 未碰，
--   生产现存结构为 PRIMARY KEY (callsign, openid)、无 tenant_id 列。
--
-- 原子性：整份文件由 `wrangler d1 execute --file --remote` 单次执行，任一语句失败即
--   回滚到原状。文件内【禁止】写 BEGIN/COMMIT/SAVEPOINT（D1 不支持用户侧事务控制，
--   写了会 SQLITE_AUTH 失败）。
--
-- 执行前置：
--   1) `wrangler d1 export` 全量备份生产 D1（回滚点）。
--   2) 与配对 worker 同批部署（迁移后 worker 写 INSERT(tenant_id,...)、route-push 反查
--      WHERE tenant_id=?；旧 worker 的 INSERT(callsign,openid,...) 会撞新表 NOT NULL
--      tenant_id → 不可单独回退 worker）。
--
-- 回滚：退 worker 版本 + 还原表（建新空 D1 + import 旧 dump，或逐表 DROP 后 import；
--   dump 不自动 DROP 已存在表）。
-- ============================================================================

-- 四步重建 callsign_openid_bindings：建新表 → INSERT…SELECT 回填 bh2ro → DROP 旧 → RENAME。
-- 索引 idx_bindings_callsign 在 DROP+RENAME 之后建（SQLite 索引名【库级全局唯一】，若在
-- *_new 上建同名索引、旧同名索引尚未随 DROP 消失 → "already exists" 整文件失败——沿用 0001 教训）。
-- 索引改为 (tenant_id, callsign COLLATE NOCASE)：服务 route-push 的 WHERE tenant_id=? AND callsign=?
-- COLLATE NOCASE 反查（旧单列 (callsign) 在租户化后既不带 tenant 维度、又被 PK 前缀覆盖而失效）。
CREATE TABLE callsign_openid_bindings_new (
    tenant_id TEXT NOT NULL,
    callsign TEXT NOT NULL,
    openid TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (tenant_id, callsign, openid)
);

-- 回填：存量行隔离键置创始租户 'bh2ro'，callsign/openid/created_at 原样拷贝。
-- 原主键 (callsign, openid) 唯一 → ('bh2ro', callsign, openid) 仍唯一，无 PK 冲突，
-- 无需单一所有者校验（绑定表无 client_id、回填值恒一）。
INSERT INTO callsign_openid_bindings_new (tenant_id, callsign, openid, created_at)
    SELECT 'bh2ro', callsign, openid, created_at FROM callsign_openid_bindings;

DROP TABLE callsign_openid_bindings;
ALTER TABLE callsign_openid_bindings_new RENAME TO callsign_openid_bindings;

-- 索引在 DROP 旧表 + RENAME 之后建（旧同名索引随旧表 DROP 消失，绑最终表名，避免全局索引名碰撞）
CREATE INDEX idx_bindings_callsign ON callsign_openid_bindings(tenant_id, callsign COLLATE NOCASE);
