## 1. CIDR 工具与配置解析（抽为纯函数，便于单测）

- [x] 1.1 实现 IPv4 `ip ∈ CIDR` 判定（纯函数）：IP 与网络地址转 32 位无符号整数 + 前缀掩码比较。**钉死陷阱**：比较前所有值 `>>> 0` 无符号化（防首字节 ≥128 的 IP 因符号位误判）；`prefix===32` 精确；`prefix===0` 安全处理（JS `<<32` 为 no-op，须特判或 `mask=0`）；前缀格式校验（非整数 / >32 / 负数 / 带前导零 → 视为非法）。
- [x] 1.2 白名单解析（纯函数）：从 `env.CDN_ORIGIN_CIDRS`（逗号/空白分隔）切分为 CIDR 数组，用**默认拒**口径——**仅接受全局可路由公网单播 IPv4 CIDR**，其余一律丢弃该条目：私网（`10/8`、`172.16/12`、`192.168/16`）、CGNAT（`100.64.0.0/10`）、loopback（`127/8`）、link-local（`169.254/16`）、组播（`224.0.0.0/4`）、保留（`240.0.0.0/4`、`0.0.0.0/8`）、`/0`、非法前缀/IP。未配置/空串/全被拒 → 空集。
- [x] 1.3 IPv6 安全降级：`CF-Connecting-IP` 为 IPv6 → 判「不命中」（除非本期确认需实现 IPv6 前缀匹配）；白名单含 IPv6 条目时**仅丢弃该 IPv6 条目**，IPv4 条目仍正常用于 IPv4 来源匹配（禁止因含一条 IPv6 即令整张白名单失效）；绝不误判命中。
- [x] 1.4 受信真实 IP 头名可配置（`env`），**无内置默认**——未配置即视为「未证实覆写」；推荐值 `Ali-Cdn-Real-Ip`，由运营者经抓包证实后显式设置。
- [x] 1.5 受信真实 IP 头值运行时校验（纯函数）：采信前校验为**单值、合法 IP 字面量、不含逗号**；多值/含逗号/非法（覆写假设失效信号）→ 返回需落 `'unknown'` 的信号。**实现禁止** `split(',')` 后取段再校验——必须对 `headers.get()` 原始返回值整体判定（含逗号即多值即 `'unknown'`），否则透传 `8.8.8.8,<真实>` 的首段会通过校验=F1 复现。

## 2. 可信真实客户端 IP 解析

- [x] 2.1 改 `getClientIP(request)` → `getClientIP(request, env)`（抽核心取值为纯函数 `resolveClientIP(headers, cidrs, realIpHeaderName)`）：CF-IP 命中白名单 **且**头名已配 → 采信该头（经 1.5 校验，去空白）、**禁止**采信 XFF；CF-IP 命中白名单 **但头名未配** → fail-safe 回退 `CF-Connecting-IP`（未证实覆写=安全态）；CF-IP 命中白名单 + 头名已配 **但本请求头缺失/为空/多值/非法** → `'unknown'`（不退回 CDN 节点 IP）；未命中 → 用 `CF-Connecting-IP`、忽略一切注入头；CF-IP 缺失 → `'unknown'`。
- [x] 2.2 更新所有调用点传 `env`：查询端点（约 572 行）、`/api/captcha`（约 542 行）。
- [x] 2.3 改 `/api/wechat/auth-callback` 限流（约 734-736 行）：把直接读 `request.headers.get('CF-Connecting-IP')`（约 736）改为 `getClientIP(request, env)`；更正约 734 行注释「IP 取自不可伪造的 CF-Connecting-IP」。
- [x] 2.4 回归 grep：改造后确认 worker 内无 `getClientIP` 之外的 `CF-Connecting-IP` / `X-Forwarded-For` 直接读取（auth-callback 旧读法已改/删）。

## 3. 配置与文档

- [x] 3.1 `web_query_service/wrangler.toml.example` 新增 `CDN_ORIGIN_CIDRS` 与受信真实 IP 头名占位 + 注释（来源、CIDR 格式、头名无默认需证实后填、未配=fail-safe 降级、宁缺勿宽）。
- [x] 3.2 部署文档 `docs/web-query-service-deploy.md`（已存在，直接补入勿新建散落文件）补：配置项含义、从阿里云获取回源 IP 段（`DescribeCdnBackSourceIp`/控制台）、定期维护提示、未配影响（CDN 路径限流按 CDN 节点 IP，失真但不被绕过）。
- [x] 3.3 文档写明**信任前置与顺序**：源站 `qsl.herbertgao.me` 须仅接受来自阿里云回源段 / Cloudflare 的回源（WAF/Firewall Rules），且**必须先于**配置 `CDN_ORIGIN_CIDRS`/启用采信头（否则进入「白名单生效但可直连源站伪造头」的危险中间态）；白名单与受信头名变更需人工 sign-off + 重跑抓包证实。

## 4. 验证（CIDR/IP 纯函数单测 + worker smoke）

- [x] 4.1 纯函数单测（node）覆盖黑盒打不到的边界：CIDR `/32`、`/0`（被拒）、范围内/外、非法条目跳过、非法前缀（>32/负/前导零）、高位 IP（首字节 ≥128）符号位、CGNAT/私网/保留段被拒、IPv6 来源不命中、白名单含 IPv6 条目时 IPv4 来源仍命中。
- [x] 4.2 纯函数单测：CF-IP 命中白名单——头名已配 + 真实头 → 取该头值；+ 伪造 XFF → **不**采信 XFF；头名**未配** → fail-safe 到 CF-IP；头名已配但头缺失/多值（含逗号）/非法 → `'unknown'`（不退回 cf）；配错（不存在的）头名 → `'unknown'`。
- [x] 4.3 纯函数单测：直连（CF-IP ∉ 白名单）+ 伪造 XFF/伪造 `Ali-Cdn-Real-Ip` → 用 `CF-Connecting-IP`；白名单未配 → 同（fail-safe）。
- [x] 4.4 `web_query_service/verify/run_worker_smoke.sh`：注入 `CDN_ORIGIN_CIDRS` + 受信头名、确认 `RATE_LIMIT` KV 绑定（dev --local 已验真计数到 429）、**段间清 KV**（否则前段计数渗入后段，「不同真实 IP 不互挤」断言会假过）；端到端断言——经 CDN 不同真实 IP 不互相挤占、伪造 XFF 不绕过、`/api/wechat/auth-callback` 在 CDN 路径按真实用户 IP 计数（独立桶不变）。**暗礁**：`CDN_ORIGIN_CIDRS` 是逗号分隔 CIDR，与 `wrangler dev --var KEY:VALUE` 解析冲突（值含逗号会被截断 → 白名单只剩子集 → 「命中」永不触发、全部 fail-safe 到 CF-IP → 测试因错误原因「通过」）；故须用**测试 `[vars]` 配置**（临时 toml），**不可**裸 `--var` 传逗号串（wrangler `--var` 解析按逗号切，shell 引号救不了）。若本地 miniflare 不透传客户端 `CF-Connecting-IP`，则 CDN-路径归桶以纯函数单测（4.1-4.3）为准、smoke 仅断言限流键端到端真计数到 429（与 verify/CHECKLIST.md 「本地非等价」口径一致）。
- [x] 4.5 跑通纯函数单测 + `run_worker_smoke.sh` 全绿；回归既有断言不破。

## 5. 收尾

- [x] 5.1 `openspec-cn validate fix-cdn-real-ip --strict` 通过。
- [ ] 5.2 部署（`pnpm run deploy`，受信头名先不配）+ 记录新 worker 版本与回滚目标版本（用户/代理代执行）。
- [ ] 5.3 **源站回源限制先行**（Cloudflare WAF/Firewall Rules 仅允许阿里云回源段 / CF）→ 再配 `CDN_ORIGIN_CIDRS`（用户执行，顺序硬约束）。
- [ ] 5.4 **证伪式抓包阻断门**（启用采信头的前置，针对最终要配置的那个头名）：经 `qsl.herbert-dev.cn` 带伪造目标头（含大小写变体 `Ali-Cdn-Real-Ip: 8.8.8.8` + `ali-cdn-real-ip: 7.7.7.7`）与伪造 `X-Forwarded-For: 9.9.9.9`，临时日志打印**两者**：(a) `headers.get('<配置头名>')` 原始值；(b) `resolveClientIP` 的最终返回值。**门唯一通过判据**：(b) = 真实出口 IP、**≠ 任一伪造值、且为单值**（证 CDN 对该头名做了覆写）。**其余皆失败**（=该头名被 CDN 透传或透传+追加、不可用，须排查 CDN 配置、保持头名未配）：若 (a) 含逗号/多值（CDN 把伪造值追加进来、未覆写）→ (b)=`'unknown'`，**这是门失败**（非通过、非「覆写成立」）；若 (b)=任一伪造值（CDN 透传单值未覆写）→ 门失败。再用一个 CDN 不注入的头名跑同样请求作对照，观察 (b) 取到攻击者值/落 unknown，实证「未被覆写的头名不可用」。**仅在该头名通过上述唯一判据后**才显式设受信头名。临时日志用**已知出口 IP 的测试机**、**禁止打印真实终端用户流量的 IP**，且**门通过后即移除**（与 captcha-protection 主规范「不记录用户完整 IP」一致）。
- [ ] 5.5 配齐后生产验收：限流按真实用户 IP 生效、伪造 XFF/伪造头不绕过（用户确认）。
