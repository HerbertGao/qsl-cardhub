## 1. 可信真实客户端 IP 解析（client-ip.js 纯函数）

- [x] 1.1 IP 字面量校验纯函数：`ipv4ToUint32`（前导零/>255/段数拒）、`isValidIPv6`（::、内嵌 IPv4、边界单冒号、zone-id 拒等）、`isValidIPLiteral`、`isTrustedHeaderValueValid`（单值/合法 IP/不含逗号；**禁 split 取段**，含逗号即拒=透传防护）。
- [x] 1.2 常量时间密钥比对 `timingSafeEqualStr`（长度不同直接 false；逐字符 XOR 累积），防计时侧信道泄漏密钥。
- [x] 1.3 `resolveClientIP(headers, originSecret, realIpHeaderName)` 分支：CF-IP 空→unknown；密钥未配→cf；`X-Origin-Auth` 常量时间≠密钥→cf 忽略注入头；密钥通过+头名未配→cf；密钥通过+头合法→该头去空白；密钥通过+头缺失/多值/非法→unknown（不退 cf 节点桶）。**绝不读 X-Forwarded-For**。
- [x] 1.4 `getClientIP(request, env)` 包装：读 `env.CDN_ORIGIN_SECRET` 与 `env.CDN_REAL_IP_HEADER`，env 缺字段（undefined）安全 fail-safe。密钥头名常量 `X-Origin-Auth`。

## 2. 接线（index.js）

- [x] 2.1 删旧 getClientIP、`import { getClientIP } from './client-ip.js'`；查询端点（约 535）、`/api/captcha`（约 565）、`/api/wechat/auth-callback`（约 729）统一 `getClientIP(request, env)`。
- [x] 2.2 回归 grep：worker 内无 `getClientIP` 之外的 `CF-Connecting-IP` / `X-Forwarded-For` 直接读取。

## 3. 配置与文档

- [x] 3.1 `wrangler.toml.example` 新增 `CDN_ORIGIN_SECRET`（机密、wrangler secret、未配 fail-safe）与 `CDN_REAL_IP_HEADER`（无默认、证伪后填）占位 + 注释。
- [x] 3.2 `docs/web-query-service-deploy.md`：阿里云配置（修改出站请求头注入 X-Origin-Auth + 回源协议 HTTPS）、部署顺序、证伪式抓包门、维护/应急轮换、未配影响、回滚。

## 4. 验证

- [x] 4.1 `verify/client-ip.test.js`（node:test）：isTrustedHeaderValueValid（含逗号/多值/边界单冒号 IPv6 拒、合法 IPv4/IPv6 纳）；resolveClientIP 全分支（密钥未配/无密钥/错密钥/等长错密钥→cf；密钥正确+真实头合法/IPv6/带空白→采信；+伪造 XFF 不采信；头名未配→cf；头缺失/多值/非法→unknown；单值透传按字面采信=覆写由证伪门保证）；getClientIP env 缺字段安全。
- [x] 4.2 `verify/run_worker_smoke.sh` 4.4 段（密钥头模型）：①经 CDN 有效密钥不同真实用户独立计数+换 CDN 节点仍同桶；②满桶+伪造 XFF 仍 429；③直连无密钥伪造头→按 CF-IP；④错误密钥伪造头→按 CF-IP；⑤auth-callback 经 CDN 按真实 IP 独立桶。`start_cdn` 经测试 `[vars]` toml 注入 `CDN_ORIGIN_SECRET`/`CDN_REAL_IP_HEADER` + 段间清 KV。
- [x] 4.3 跑通：`node --test verify/client-ip.test.js` 全绿、`run_worker_smoke.sh` PASS/FAIL=0、`pnpm run build` 绿、回归既有断言不破。

## 5. 收尾（部署/阿里云/验收，用户或代理代执行）

- [x] 5.1 `openspec-cn validate fix-cdn-real-ip --strict` 通过。
- [x] 5.2 阿里云 CDN：「修改出站请求头」以覆盖语义（增加+不允许重复 或 替换）注入 `X-Origin-Auth: <密钥>` + 回源协议 HTTPS（用户已做出站头注入，确认操作类型与回源协议）。
- [x] 5.3 部署 worker（`pnpm run deploy`）→ `wrangler secret put CDN_ORIGIN_SECRET`（与阿里云注入值一致）+ 记录新版本与回滚目标。
- [x] 5.4 **证伪式抓包门**：经 `qsl.herbert-dev.cn` 带伪造 `X-Origin-Auth` + 伪造 `Ali-Cdn-Real-Ip`（含大小写变体）+ 伪造 XFF，临时 log 断言 worker 收到的 `X-Origin-Auth`=真实密钥（覆写成立）、`Ali-Cdn-Real-Ip`=真实出口 IP 且≠伪造值且单值。通过后才 `wrangler secret put CDN_REAL_IP_HEADER`（=`Ali-Cdn-Real-Ip`）；撤临时 log。
- [x] 5.5 生产验收：限流按真实用户 IP 生效、伪造 XFF/伪造头/错误密钥不绕过（用户确认）→ `openspec-cn archive fix-cdn-real-ip`。
