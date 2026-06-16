## 1. Worker：resolveTenant 只读参数

- [x] 1.1 `web_query_service/src/worker/index.js` `resolveTenant` 加可选 `readonly` 参数（默认 false）：为 true 时**跳过** `service_counters('auth_fallback')` 的 UPDATE（兜底命中仍解析出 `bh2ro`，只是不计数）
- [x] 1.2 `resolveTenant` 返回 `{tenant, viaFallback}`（告知调用方是否经 env.API_KEY 兜底命中，供 /ping 回显 `fallback`）；唯一调用方 `crossCheckTenant` 取 `.tenant`/`.viaFallback`

## 2. Worker：crossCheckTenant helper

- [x] 2.1 新增 `crossCheckTenant(env, key, declared, opts)`（置于 `resolveTenant` 之后）：`resolveTenant(opts)` → null 返 `{ok:false,status:401,code:'auth_failed'}`；`declared` 非空且 `!== tenant` 返 `{ok:false,status:403,code:'tenant_mismatch'}`；否则 `{ok:true,tenant}`（tenant 恒为 resolveTenant 值）
- [x] 2.2 **红线**：helper 返回的 `tenant` 永远是 resolveTenant 的解析值，`declared` 只参与比较、绝不出现在任何 SQL 的 `tenant_id` 绑定

## 3. Worker：/sync、/pull 接 crossCheckTenant + 读 X-Tenant-Id

- [x] 3.1 `/sync` handler：读 `(request.headers.get('X-Tenant-Id')||'').trim()`，`resolveTenant` 调用替换为 `crossCheckTenant`；403 返 `{success:false, code:'tenant_mismatch', ...}`（HTTP 403），**在任何 DELETE/INSERT 之前**；命中后入库逻辑零改动、仍用返回的 `tenant`
- [x] 3.2 `/pull` handler：同 3.1（403 在任何 SELECT 之前）
- [x] 3.3 校验：grep 确认 /sync、/pull 的入库/读取 `tenant_id` 仍来自 crossCheckTenant 返回的 `tenant`，无任何路径用 `X-Tenant-Id` 头值

## 4. Worker：/ping 升级

- [x] 4.1 `/ping` handler 重写：走 `crossCheckTenant(env, token, declared, {readonly:true})`（取代 `token===trim(env.API_KEY)` 直比）；401 返 `{success:false,code:'auth_failed',message:'API Key 无效'}`；403（带不一致 X-Tenant-Id）返 `tenant_mismatch`
- [x] 4.2 200 回显：`{ success:true, message:'pong', server_time, tenant, fallback }`（`tenant`/`fallback` 为 SHALL 必返）
- [x] 4.3 旧断言保持：env.API_KEY 含尾随空白仍测通（resolveTenant 兜底已 trim）；env.API_KEY 空且无表驱动命中 → 401（不 fail-open）

## 5. Worker：CORS

- [x] 5.1 `CORS_HEADERS` 的 `Access-Control-Allow-Headers` 加入 `X-Tenant-Id`

## 6. 本地验证

- [x] 6.1 新增 `web_query_service/verify/cross-check.test.js`（node:test）：crossCheckTenant/resolveTenant 纯逻辑——缺 declared 放行、declared==tenant 放行、declared!=tenant 403（返回 tenant 恒为解析值非声明值）、resolveTenant null 401、readonly 不计兜底、changes=0 抛错、表驱动优先于兜底
- [x] 6.2 扩展 `verify/run_worker_smoke.sh` 4.7 段：① 表驱动 Key（非 env.API_KEY）+ 正确 `X-Tenant-Id` → /ping 200 且回显 tenant=bh2ro fallback=false；② /ping X-Tenant-Id 一致→200/不一致→403；③ /sync 错 `X-Tenant-Id` → 403 且指纹不变 + 声明租户名下零行（红线）；④ /sync·/pull 缺/空白 `X-Tenant-Id` → 向后兼容 200、错头→403；⑤ /ping 兜底命中 `auth_fallback` 不变（只读）vs /sync 兜底计数（对照）；⑥ CORS 含 X-Tenant-Id
- [x] 6.3 跑通 `pnpm run build`、`node --test`（92/92）、`run_worker_smoke.sh`（108/0）、`node --check index.js`、shellcheck、`openspec-cn validate --strict`

## 7. 验收门

- [ ] 7.1 对抗性 review 通过（红线「声明值绝不当目标」、缺/空 header 向后兼容、/ping 只读不计兜底、resolveTenant 返回形态改动不破坏 /sync·/pull、spec↔impl 一致）

## 8. 部署与归档

- [x] 8.1 `pnpm run deploy` → worker **c6386089-d888-48f9-b810-2a0738b27830**（回滚目标 **facccf8a**，无 D1 迁移、回滚=退版本）
- [x] 8.2【验收】生产冒烟全绿：无凭据 /ping 401+auth_failed / /api/config 200 / /sync·/pull 401；有效 Key /ping 200 tenant=bh2ro fallback=false（表驱动命中、缺口修复）；+X-Tenant-Id 一致 200 / 不一致 403 tenant_mismatch；CORS 含 X-Tenant-Id
- [x] 8.3 `openspec-cn archive add-tenant-write-crosscheck`（增量并入 `cloud-backend-api`/`tenant-isolation` 主规范）
