## 1. 离线 mint CLI 脚本

- [x] 1.1 新建 `web_query_service/scripts/mint-credential.mjs`：位置参数 `<slug>` + Key（位置参数或 `--key-stdin`）；用 `node:crypto` 的 `createHash('sha256').update(key.trim(),'utf8').digest('hex')` 算 `key_hash`。**不暴露 `--scope`**（worker 不读 scope，恒写常量 `'sync'`）
- [x] 1.2 输入校验：slug 必须匹配 `^[a-z0-9-]{1,32}$`，非法即报错退出（**拒绝不转换**）；`key.trim()===''` 报错退出（绝不输出 `sha256('')`）；`key.trim().length` < 32 报错退出并提示 `openssl rand -hex 32`（floor 对齐建议、非熵保证）
- [x] 1.3 输出 SQL 到 stdout：`INSERT OR IGNORE INTO tenants(tenant_id,name,status)`（幂等）+ `INSERT INTO tenant_credentials(id,tenant_id,scope,key_hash,status) VALUES(...,'active')`（NOT NULL 列：`id`/`tenant_id`/`key_hash` 无默认必须显式给值、`status` 有默认仍显式写 `'active'`；`scope` 可空、恒写常量 `'sync'` 作元数据）；`id = '<slug>-' || key_hash`（**完整 64 位 hash**，不截断）；输出只含 hash，绝不含明文 Key；凭据行用普通 `INSERT`（**非** `OR IGNORE`）使重签/跨租户复用安全失败报错
- [x] 1.4 `--key-stdin` 从 stdin 读 Key（`trim()` 吸收尾随 `\n`/CRLF）；退出码语义化（数据走 stdout、说明走 stderr，可直接 `| wrangler d1 execute --file=-`）
- [x] 1.5 内置硬编码字面量自检：`assert(sha256('qsl-mint-selfcheck') === '989e608711151a9484b398e3af86cb80f1449d3fa9824da303383c30f1d215fe')`（**禁** `createHash` 自比自的恒真断言）；`node mint-credential.mjs` 无参时打印用法

## 2. 文档：cloud-sync-api-spec.md 重写到当前契约

- [x] 2.1 文首声明**契约真源 = `openspec/specs/cloud-backend-api` 主规范**，本文档为**非规范性**（non-normative）自托管实现指引、跟踪它、冲突以它为准——字段/端点以**带链接的示例**呈现而非平行的规范性断言（避免归档后双真源漂移）；删除过时单一 `API_KEY` / 无 `/pull` / 无版本护栏 / 把 `client_id` 当隔离键的描述与 Express 示例
- [x] 2.2 重写认证/租户模型节：`Authorization: Bearer <key>` 解析租户（表驱动 `key_hash`，不读 scope）+ 可选 `X-Tenant-Id` 交叉校验（不一致 **403 `tenant_mismatch`**、缺头向后兼容、Key 无效 **401 `auth_failed`**，用精确 code 串非散文）；默认单租户 slug `default`
- [x] 2.3 以**示例**（非规范性、注明「规范以 cloud-backend-api 为准」）覆盖端点契约（带类型与 null case 的请求/响应样例）：`GET /ping`→`{tenant, fallback}`；`POST /sync`→**必填 `client_id`（≤128、仅写 `last_client_id` 溯源、永不决定数据归属，缺则 400）**、`data` 含 `app_settings`、OCC `base_version`（**严格整数**）/ `force`（**`=== true` 布尔非字符串**）/ 409 / 返回 `server_version`；新增 `GET /pull`（JSON 串列/布尔列还原为对象/布尔，回 `server_version`/`last_client_id`/`sync_time` 及其类型与 null 情形）
- [x] 2.4 明示同步端点为**裸路径** `/ping`·`/sync`·`/pull`——`/t/<slug>/` 前缀在这三个端点上 worker 返 404；租户身份经 Bearer + 可选 `X-Tenant-Id` 头，**非** URL 路径；「更新历史」加本次条目（阶段 4-C4）

## 3. 文档：签发与自托管指引（docs/web-query-service-deploy.md 新增节）

- [x] 3.1 「新增租户」：用 `mint-credential.mjs` 签发凭据流程（推荐 `--key-stdin` + `wrangler d1 execute qsl-sync --remote --file=-`），明示 Key 不进任何文件/history、用 `openssl rand -hex 32` 生成（min-length 门仅是长度下限、非熵保证）；说明凭据已存在时 `wrangler` 抛 SQLite 约束错误（非脚本 bug）= 该 Key/租户已签发；一把 Key 不能跨租户复用、每租户签独立 Key；**密钥轮换**：active 唯一索引只覆盖 `status='active'`，re-mint 一个已 `revoked` 的旧 Key（若旧行 id 不同）不会被拦截——轮换时务必生成全新 Key
- [x] 3.2 「自托管」分两类：(a) **只实现同步 API 的自定义后端**——仅需 Bearer 鉴权，无需 PoW/会话/KV；(b) **自部署本仓库 worker**——公共查询面恒 PoW，需 `SESSION_SECRET` + `RATE_LIMIT` KV（参见防爬节）；`REQUIRE_POW`/`REQUIRE_TENANT_HEADER` 标「计划中」
- [x] 3.3 默认 slug `default` **必须**与 `env.DEFAULT_TENANT="default"` 一致 + seed 匹配租户行（worker 内置兜底是 `bh2ro`），**交叉引用** deploy 文档既有红线「seed slug 必须 == `DEFAULT_TENANT` 否则裸查询面静默空结果」；官方云创始租户仍 `bh2ro`

## 4. 验证与收尾

- [x] 4.1 实跑脚本：合法输入产出可执行 SQL（`id` 为完整 hash、`scope='sync'`）；非法 slug / 空 Key / 弱 Key（<32）非 0 退出且 stdout 无 SQL；`--key-stdin` 路径正确；重签 = 把同一 Key 的 SQL 在离线 SQLite **执行两次**、断言**第二次**报 `UNIQUE constraint failed`（只跑一次则该断言空过）
- [x] 4.2 差分验证：脚本对某 Key 的 `key_hash` == worker `sha256(trim(key))`——**绑定形式 = 独立比对**（对已上线 `bh2ro` 凭据核对 / 已知答案向量），**非**两路同 `createHash` 自比自；硬编码自检字面量断言通过
- [x] 4.3 `openspec-cn validate add-credential-mint-cli --strict` 通过
- [ ] 4.4 对抗 review-loop 收敛 → `openspec-cn archive add-credential-mint-cli`（新建 `credential-minting`；**无修改能力**）；归档前**人工评审门**（非纯机械 grep）：重写后的 `cloud-sync-api-spec.md` 文首声明从属 `cloud-backend-api`、且正文为非规范性示例、不复述与主规范并行的规范性断言（mirror 4-C1「0 残留」门，由评审者裁定）
