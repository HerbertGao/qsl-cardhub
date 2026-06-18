# credential-minting 规范

## 目的

为运维提供离线签发租户写凭据的工具能力：把「租户 slug + 写凭据 Key」转换为可直接执行的 SQL，在**不连接数据库、不落明文 Key**的前提下，安全且可复用地为新租户/新设备签发 `tenant_credentials` 写凭据。归属真源恒为 `key→tenant`（worker `resolveTenant`），本能力只负责离线产出与服务端契约一致的凭据 SQL；注册纯线下、无公开自助端点。

## 需求
### 需求:离线签发租户写凭据

系统必须提供一个离线命令行脚本，把「租户 slug + 写凭据 Key」转换为可执行的 SQL，用于在 D1 中签发该租户的 active 写凭据。该脚本**禁止**连接任何数据库、**禁止**把明文 Key 写入其输出或任何文件——输出只含 `sha256(trim(key))` 的 hash。

#### 场景:产出签发 SQL
- **当** 运维以合法 slug 与合规 Key 运行脚本
- **那么** 脚本向 stdout 输出 `INSERT OR IGNORE INTO tenants (tenant_id, name, status)` 与一条 `INSERT INTO tenant_credentials (id, tenant_id, scope, key_hash, status)` 的 SQL（凭据表 NOT NULL 列为 `id`/`tenant_id`/`key_hash`/`status` 四列：前三者无默认值必须显式给值、`status` 有默认值仍显式写 `'active'`；`scope` 可空、恒写常量 `'sync'` 作元数据），以退出码 0 结束
- **那么** 输出绝不含明文 Key，仅含其 `sha256(trim(key))` hash

#### 场景:hash 与服务端逐字节一致
- **当** 脚本计算 `key_hash`
- **那么** 其值必须等于服务端 worker 对同一 Key 计算的 `sha256(key.trim())`（UTF-8 字节的 SHA-256，64 位小写 hex），二者对任一 Key 逐字节相等
- **那么** 脚本必须内置一个对**独立硬编码字面量**的自检断言（禁止用同一 `createHash` 调用自比自的恒真断言）

#### 场景:id 用完整 hash 派生
- **当** 脚本派生凭据行的 `id`
- **那么** `id` 必须由完整 64 位 `key_hash` 派生（如 `'<slug>-' || key_hash`），**禁止**截断为前 N 位——以免同租户两个合法 Key 的 hash 截断前缀相同而误撞主键

### 需求:重跑语义与跨租户复用安全

脚本产出的 SQL 必须有明确的重跑语义：`tenants` 行**幂等**（`INSERT OR IGNORE`，已存在则不动）；`tenant_credentials` 行**安全失败**——重复签发或跨租户复用同一 Key 时由约束拒绝并报明确错误，**禁止**静默覆写或静默吞掉。脚本**禁止**对凭据行使用 `INSERT OR IGNORE`（否则跨租户复用会被静默忽略、运维不自知）。

#### 场景:同一 Key 重复签发被拒
- **当** 同一租户的同一 Key 签发 SQL 被重复执行
- **那么** 凭据行因主键（同 `id`）/ `idx_tenant_credentials_active_key_hash`（同 active `key_hash`）约束被拒、报错，运维据此知道该 Key 已签发；`tenants` 行因 `OR IGNORE` 不重复创建

#### 场景:跨租户复用同一 Key 被拒
- **当** 同一 Key 被签发给第二个租户
- **那么** 该插入因 `idx_tenant_credentials_active_key_hash` 为**全局** active 唯一索引（无 tenant_id 列）被拒、报错——一把 Key 不能解析到两个租户

### 需求:输入校验（slug / 空 Key / 弱 Key）

脚本必须在产出 SQL 前校验输入。slug 必须匹配 `^[a-z0-9-]{1,32}$`（与服务端 `tenants.tenant_id` CHECK 同语义），非法 slug **必须拒绝且禁止静默转换**（不得自动小写化或截断）；Key 经 `trim()` 后为空时**必须**拒绝（绝不输出 `sha256('')` 凭据）；Key 经 `trim()` 后长度低于最小阈值时**必须**拒绝（unsalted sha256 短 Key 可离线爆破）。

#### 场景:非法 slug 拒绝
- **当** 运维传入含大写、空格、点号、超过 32 字符或为空的 slug
- **那么** 脚本以非 0 退出码结束并向 stderr 说明原因，不向 stdout 输出任何 SQL

#### 场景:空 Key 拒绝
- **当** 运维传入空白或仅含空白字符的 Key
- **那么** 脚本以非 0 退出码结束并向 stderr 说明原因，不输出任何凭据 SQL

#### 场景:弱 Key 拒绝
- **当** 运维传入 `trim()` 后长度低于最小阈值的 Key
- **那么** 脚本以非 0 退出码结束并向 stderr 提示使用高熵 Key（如 `openssl rand -hex 32`），不输出任何凭据 SQL

### 需求:Key 不进 shell history

脚本必须提供 `--key-stdin` 选项，从标准输入读取 Key，使 Key 不必出现在命令行参数（避免进入 shell 历史）。

#### 场景:从 stdin 读取 Key
- **当** 运维以 `--key-stdin` 运行并通过管道传入 Key
- **那么** 脚本从 stdin 读取 Key（而非位置参数），其余校验与产出行为与参数式一致（`trim()` 吸收尾随换行/CRLF）

