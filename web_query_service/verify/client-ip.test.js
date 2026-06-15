/**
 * trusted-client-ip 纯函数单测（tasks 4.1 / 4.2 / 4.3）。
 *
 * 覆盖黑盒（worker smoke）打不到的边界，与
 * openspec/changes/fix-cdn-real-ip/specs/trusted-client-ip/spec.md 的判定规则与全部场景对齐。
 *
 * 运行：在 web_query_service 目录执行  node --test verify/client-ip.test.js
 *      或  pnpm run test:unit
 *
 * 仅用 node 内置 node:test + node:assert，无新增 devDependency。
 * resolveClientIP 第一个入参是 Fetch Headers 对象（node 18+ 全局有 Headers）；
 * 多值头用 h.append(name, v) 两次构造（Headers.get 会以 ", " 拼接 → 触发含逗号即多值的判定）。
 */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  ipv4InCidr,
  parseTrustedCidrs,
  isTrustedHeaderValueValid,
  resolveClientIP,
} from '../src/worker/client-ip.js';

// 测试用受信白名单：两段全局可路由公网文档段（RFC 5737）。
const WL = ['203.0.113.0/24', '198.51.100.0/24'];
const REAL_HEADER = 'Ali-Cdn-Real-Ip';

// 用对象构造 Headers（单值用 set）。
function headers(obj) {
  const h = new Headers();
  for (const [k, v] of Object.entries(obj)) {
    if (v === undefined) continue;
    h.set(k, v);
  }
  return h;
}

// ============================================================
// 4.1 CIDR / 白名单纯函数边界
// ============================================================

test('4.1 ipv4InCidr：/32 精确匹配，相邻地址不命中', () => {
  assert.equal(ipv4InCidr('203.0.113.5', '203.0.113.5/32'), true);
  assert.equal(ipv4InCidr('203.0.113.6', '203.0.113.5/32'), false);
  assert.equal(ipv4InCidr('203.0.113.4', '203.0.113.5/32'), false);
});

test('4.1 ipv4InCidr：范围内 / 范围外', () => {
  assert.equal(ipv4InCidr('203.0.113.0', '203.0.113.0/24'), true);
  assert.equal(ipv4InCidr('203.0.113.255', '203.0.113.0/24'), true);
  assert.equal(ipv4InCidr('203.0.113.128', '203.0.113.0/24'), true);
  assert.equal(ipv4InCidr('203.0.114.0', '203.0.113.0/24'), false); // 越界一段
  assert.equal(ipv4InCidr('203.0.112.255', '203.0.113.0/24'), false);
});

test('4.1 ipv4InCidr：高位 IP（首字节 ≥128）符号位不误判（>>> 0 无符号化）', () => {
  // 200.x / 255.x：朴素有符号位运算会把这些值变负、致比较错；必须正确。
  assert.equal(ipv4InCidr('200.1.2.3', '200.1.2.0/24'), true);
  assert.equal(ipv4InCidr('200.1.3.3', '200.1.2.0/24'), false);
  assert.equal(ipv4InCidr('255.255.255.255', '255.255.255.255/32'), true);
  assert.equal(ipv4InCidr('128.0.0.1', '128.0.0.0/24'), true);
  // 跨「符号位边界」的大段：/1 把 128.0.0.0-255.255.255.255 划为一段。
  assert.equal(ipv4InCidr('200.0.0.1', '128.0.0.0/1'), true);
  assert.equal(ipv4InCidr('127.255.255.255', '128.0.0.0/1'), false);
});

test('4.1 ipv4InCidr：/0 在匹配函数层安全（mask=0 → 恒命中），但配置层会拒（见 parseTrustedCidrs）', () => {
  // 函数层 prefix===0 须按 mask=0 处理（防 JS `<<32` mod-32 no-op 陷阱），不抛错、行为确定。
  assert.equal(ipv4InCidr('8.8.8.8', '0.0.0.0/0'), true);
  assert.equal(ipv4InCidr('255.255.255.255', '0.0.0.0/0'), true);
});

test('4.1 ipv4InCidr：非法 IP / 非法 CIDR → 不命中（绝不误判命中）', () => {
  assert.equal(ipv4InCidr('not-an-ip', '203.0.113.0/24'), false);
  assert.equal(ipv4InCidr('203.0.113.5', 'garbage'), false);
  assert.equal(ipv4InCidr('256.1.1.1', '256.1.1.0/24'), false); // 段 >255
  assert.equal(ipv4InCidr('203.0.113.5', '203.0.113.0/33'), false); // 前缀 >32
  assert.equal(ipv4InCidr('1.2.3.4', '1.2.3.0'), false); // 缺 /prefix
});

test('4.1 parseTrustedCidrs：保留合法公网单播 IPv4 CIDR，逗号/空白分隔', () => {
  assert.deepEqual(parseTrustedCidrs('203.0.113.0/24, 198.51.100.0/24'), ['203.0.113.0/24', '198.51.100.0/24']);
  assert.deepEqual(parseTrustedCidrs('203.0.113.0/24 198.51.100.0/24'), ['203.0.113.0/24', '198.51.100.0/24']);
  assert.deepEqual(parseTrustedCidrs('  203.0.113.0/24 ,  198.51.100.0/24  '), ['203.0.113.0/24', '198.51.100.0/24']);
  assert.deepEqual(parseTrustedCidrs('200.0.0.0/8'), ['200.0.0.0/8']); // 高位首字节公网段保留
});

test('4.1 parseTrustedCidrs：未配置 / 空串 / 全被拒 → 空集', () => {
  assert.deepEqual(parseTrustedCidrs(undefined), []);
  assert.deepEqual(parseTrustedCidrs(null), []);
  assert.deepEqual(parseTrustedCidrs(''), []);
  assert.deepEqual(parseTrustedCidrs('   ,  '), []);
  assert.deepEqual(parseTrustedCidrs('10.0.0.0/8, 192.168.0.0/16'), []); // 全私网 → 全拒 → 空
});

test('4.1 parseTrustedCidrs：CGNAT / 私网 / 保留段一律被拒（默认拒）', () => {
  // 私网三段
  assert.deepEqual(parseTrustedCidrs('10.0.0.0/8'), []);
  assert.deepEqual(parseTrustedCidrs('172.16.0.0/12'), []);
  assert.deepEqual(parseTrustedCidrs('192.168.0.0/16'), []);
  // CGNAT 100.64.0.0/10
  assert.deepEqual(parseTrustedCidrs('100.64.0.0/10'), []);
  // loopback / link-local
  assert.deepEqual(parseTrustedCidrs('127.0.0.0/8'), []);
  assert.deepEqual(parseTrustedCidrs('169.254.0.0/16'), []);
  // 组播 / 保留 / 0.0.0.0/8
  assert.deepEqual(parseTrustedCidrs('224.0.0.0/4'), []);
  assert.deepEqual(parseTrustedCidrs('240.0.0.0/4'), []);
  assert.deepEqual(parseTrustedCidrs('0.0.0.0/8'), []);
});

test('4.1 parseTrustedCidrs：含 CGNAT 单条 CIDR 内的具体网段也被拒（不止整段写法）', () => {
  // 100.64.0.0/10 内的子段（如 100.100.0.0/16）也应落入 CGNAT 拒绝。
  assert.deepEqual(parseTrustedCidrs('100.100.0.0/16'), []);
});

test('4.1 parseTrustedCidrs：过宽聚合段虽基址公网但整块覆盖保留地址 → 拒（整块相交判定）', () => {
  // 这些段的网络基址是公网，只查基址会误放行；按整块 ∩ 保留段相交须拒。
  assert.deepEqual(parseTrustedCidrs('8.0.0.0/6'), []);   // 含 10.0.0.0/8
  assert.deepEqual(parseTrustedCidrs('96.0.0.0/4'), []);  // 含 CGNAT 100.64.0.0/10
  assert.deepEqual(parseTrustedCidrs('168.0.0.0/7'), []); // 含 link-local 169.254/16
  assert.deepEqual(parseTrustedCidrs('128.0.0.0/1'), []); // 半个地址空间，含 224/4、240/4 等
  assert.deepEqual(parseTrustedCidrs('192.0.0.0/2'), []); // 含 192.168/16、224/4、240/4
  assert.deepEqual(parseTrustedCidrs('172.0.0.0/11'), []);// 含 172.16/12
  assert.deepEqual(parseTrustedCidrs('100.0.0.0/9'), []); // 含 CGNAT 100.64/10
  assert.deepEqual(parseTrustedCidrs('0.0.0.0/1'), []);   // 含 0/8、10/8、127/8 等
});

test('4.1 parseTrustedCidrs：基址与整块都不触保留段的公网块仍接受', () => {
  assert.deepEqual(parseTrustedCidrs('203.0.113.0/24'), ['203.0.113.0/24']);
  assert.deepEqual(parseTrustedCidrs('198.51.100.0/24'), ['198.51.100.0/24']);
  assert.deepEqual(parseTrustedCidrs('8.8.8.0/24'), ['8.8.8.0/24']);
  assert.deepEqual(parseTrustedCidrs('1.2.3.4/32'), ['1.2.3.4/32']);
  assert.deepEqual(parseTrustedCidrs('128.1.0.0/16'), ['128.1.0.0/16']);
});

test('4.1 parseTrustedCidrs：/0 被拒', () => {
  assert.deepEqual(parseTrustedCidrs('0.0.0.0/0'), []);
  assert.deepEqual(parseTrustedCidrs('203.0.113.0/24, 0.0.0.0/0'), ['203.0.113.0/24']); // /0 丢、其余留
});

test('4.1 parseTrustedCidrs：非法前缀（>32 / 负 / 前导零）→ 丢弃该条', () => {
  assert.deepEqual(parseTrustedCidrs('203.0.113.0/33'), []); // >32
  assert.deepEqual(parseTrustedCidrs('203.0.113.0/-1'), []); // 负
  assert.deepEqual(parseTrustedCidrs('203.0.113.0/024'), []); // 前导零前缀
  assert.deepEqual(parseTrustedCidrs('203.0.113.0/2x'), []); // 非整数
  // 合法条目仍保留、非法条目仅丢自己（不影响整张白名单）
  assert.deepEqual(parseTrustedCidrs('203.0.113.0/33, 198.51.100.0/24'), ['198.51.100.0/24']);
});

test('4.1 parseTrustedCidrs：非法 IP 字面量（含前导零段）→ 丢弃该条', () => {
  assert.deepEqual(parseTrustedCidrs('01.2.3.0/24'), []); // 段前导零
  assert.deepEqual(parseTrustedCidrs('256.0.0.0/8'), []); // 段 >255
  assert.deepEqual(parseTrustedCidrs('1.2.3/24'), []); // 段数不足
});

test('4.1 parseTrustedCidrs：白名单含 IPv6 条目时 IPv4 条目仍保留（仅丢 IPv6 条，不令整张失效）', () => {
  assert.deepEqual(
    parseTrustedCidrs('2400:cb00::/32, 203.0.113.0/24'),
    ['203.0.113.0/24']
  );
  assert.deepEqual(
    parseTrustedCidrs('203.0.113.0/24, 2001:db8::/32, 198.51.100.0/24'),
    ['203.0.113.0/24', '198.51.100.0/24']
  );
});

test('4.1 resolveClientIP：IPv6 来源（CF-Connecting-IP）不命中任何 IPv4 CIDR → 安全降级走 fail-safe（用 cf）', () => {
  // IPv6 cf 不会命中 IPv4 白名单 → 分支 3（未命中）→ 返回 cf，忽略注入头（即便已配头名）。
  const h = headers({ 'CF-Connecting-IP': '2400:cb00:0:1::abcd', 'Ali-Cdn-Real-Ip': '198.51.100.9' });
  assert.equal(resolveClientIP(h, WL, REAL_HEADER), '2400:cb00:0:1::abcd');
});

test('4.1 resolveClientIP：白名单含 IPv6 条目但 IPv4 来源仍正常命中并采信受信头', () => {
  const cidrs = ['2400:cb00::/32', '203.0.113.0/24'];
  const h = headers({ 'CF-Connecting-IP': '203.0.113.5', 'Ali-Cdn-Real-Ip': '198.51.100.9' });
  assert.equal(resolveClientIP(h, cidrs, REAL_HEADER), '198.51.100.9');
});

// ============================================================
// 4.2 CF-IP 命中白名单（经 CDN 路径）的各分支
// ============================================================

test('4.2 命中 + 头名已配 + 真实头 → 取该头值（spec：采信 CDN 写入真实 IP 头）', () => {
  const h = headers({ 'CF-Connecting-IP': '203.0.113.5', 'Ali-Cdn-Real-Ip': '198.51.100.9' });
  assert.equal(resolveClientIP(h, WL, REAL_HEADER), '198.51.100.9');
});

test('4.2 命中 + 头名已配 + 真实头去空白（trim 后采信）', () => {
  const h = headers({ 'CF-Connecting-IP': '203.0.113.5', 'Ali-Cdn-Real-Ip': '  198.51.100.9  ' });
  assert.equal(resolveClientIP(h, WL, REAL_HEADER), '198.51.100.9');
});

test('4.2 命中 + 头名已配 + 受信头为合法 IPv6 字面量 → 采信（IPv6 值作计数键、不参与 CIDR）', () => {
  const h = headers({ 'CF-Connecting-IP': '203.0.113.5', 'Ali-Cdn-Real-Ip': '2001:db8::1234' });
  assert.equal(resolveClientIP(h, WL, REAL_HEADER), '2001:db8::1234');
});

test('4.2 命中 + 头名已配 + 真实头 + 伪造 XFF → 不采信 XFF，仍取受信头（spec：忽略客户端伪造 XFF）', () => {
  const h = headers({
    'CF-Connecting-IP': '203.0.113.5',
    'Ali-Cdn-Real-Ip': '198.51.100.9',
    // 经 CDN append 后 XFF 首段是攻击者可伪造值；绝不采用 XFF 任何分段。
    'X-Forwarded-For': '6.6.6.6, 198.51.100.9',
  });
  const got = resolveClientIP(h, WL, REAL_HEADER);
  assert.equal(got, '198.51.100.9');
  assert.notEqual(got, '6.6.6.6');
});

test('4.2 命中 + 头名已配 + 仅伪造 XFF（无受信头）→ unknown，绝不退回 cf 也绝不用 XFF', () => {
  const h = headers({
    'CF-Connecting-IP': '203.0.113.5',
    'X-Forwarded-For': '6.6.6.6, 7.7.7.7',
  });
  const got = resolveClientIP(h, WL, REAL_HEADER);
  assert.equal(got, 'unknown'); // 受信头缺失 → 惩罚桶
  assert.notEqual(got, '6.6.6.6');
  assert.notEqual(got, '203.0.113.5'); // 不退回 CDN 节点桶
});

test('4.2 命中 + 头名未配（空串）→ fail-safe 回退 cf，禁采信任何注入头', () => {
  const h = headers({ 'CF-Connecting-IP': '203.0.113.5', 'Ali-Cdn-Real-Ip': '198.51.100.9' });
  assert.equal(resolveClientIP(h, WL, ''), '203.0.113.5');
});

test('4.2 命中 + 头名未配（undefined / null / 纯空白）→ fail-safe 回退 cf', () => {
  const h = headers({ 'CF-Connecting-IP': '203.0.113.5', 'Ali-Cdn-Real-Ip': '198.51.100.9' });
  assert.equal(resolveClientIP(h, WL, undefined), '203.0.113.5');
  assert.equal(resolveClientIP(h, WL, null), '203.0.113.5');
  assert.equal(resolveClientIP(h, WL, '   '), '203.0.113.5');
});

test('4.2 命中 + 头名已配但本请求该头缺失 → unknown（不退回 cf 节点桶）', () => {
  const h = headers({ 'CF-Connecting-IP': '203.0.113.5' });
  assert.equal(resolveClientIP(h, WL, REAL_HEADER), 'unknown');
});

test('4.2 命中 + 头名已配但该头为空 → unknown', () => {
  const h = headers({ 'CF-Connecting-IP': '203.0.113.5', 'Ali-Cdn-Real-Ip': '   ' });
  assert.equal(resolveClientIP(h, WL, REAL_HEADER), 'unknown');
});

test('4.2 命中 + 头名已配但该头多值（含逗号）→ unknown（覆写假设失效信号，禁 split 取首段）', () => {
  // 两次 append → Headers.get 以 ", " 拼接 → 含逗号 → 多值。
  const h = new Headers();
  h.set('CF-Connecting-IP', '203.0.113.5');
  h.append('Ali-Cdn-Real-Ip', '8.8.8.8'); // 攻击者透传值（若被采为首段=F1 复现）
  h.append('Ali-Cdn-Real-Ip', '198.51.100.9'); // CDN 追加的真实值
  const got = resolveClientIP(h, WL, REAL_HEADER);
  assert.equal(got, 'unknown');
  assert.notEqual(got, '8.8.8.8'); // 禁取多值首段
});

test('4.2 命中 + 头名已配但单值含内嵌逗号串 → unknown（直接对原始值整体判定）', () => {
  const h = headers({ 'CF-Connecting-IP': '203.0.113.5', 'Ali-Cdn-Real-Ip': '8.8.8.8,198.51.100.9' });
  const got = resolveClientIP(h, WL, REAL_HEADER);
  assert.equal(got, 'unknown');
  assert.notEqual(got, '8.8.8.8');
});

test('4.2 命中 + 头名已配但该头非合法 IP 字面量 → unknown', () => {
  const h = headers({ 'CF-Connecting-IP': '203.0.113.5', 'Ali-Cdn-Real-Ip': 'not-an-ip' });
  assert.equal(resolveClientIP(h, WL, REAL_HEADER), 'unknown');
});

test('4.2 命中 + 配错（不存在的）头名 → unknown（该头名本请求读不到值）', () => {
  const h = headers({ 'CF-Connecting-IP': '203.0.113.5', 'Ali-Cdn-Real-Ip': '198.51.100.9' });
  // 运营者把头名配成一个 CDN 根本不注入的名字 → headers.get 返 null → unknown。
  assert.equal(resolveClientIP(h, WL, 'X-Does-Not-Exist'), 'unknown');
});

// ============================================================
// 4.3 直连（CF-IP ∉ 白名单）/ 白名单未配 → 用 CF-Connecting-IP，忽略一切注入头
// ============================================================

test('4.3 未命中（直连）+ 伪造 XFF + 伪造 Ali-Cdn-Real-Ip → 用 CF-Connecting-IP', () => {
  const h = headers({
    'CF-Connecting-IP': '8.8.8.8', // 不在白名单
    'Ali-Cdn-Real-Ip': '1.2.3.4', // 客户端伪造
    'X-Forwarded-For': '5.5.5.5, 6.6.6.6', // 客户端伪造
  });
  const got = resolveClientIP(h, WL, REAL_HEADER);
  assert.equal(got, '8.8.8.8');
  assert.notEqual(got, '1.2.3.4');
  assert.notEqual(got, '5.5.5.5');
});

test('4.3 白名单未配（空集）+ 伪造头 → fail-safe 用 CF-Connecting-IP', () => {
  const h = headers({
    'CF-Connecting-IP': '203.0.113.5', // 即便看起来像 CDN 段，但白名单空 → 不命中
    'Ali-Cdn-Real-Ip': '1.2.3.4',
    'X-Forwarded-For': '5.5.5.5',
  });
  // 已配头名也不该被采信，因为白名单为空 → 恒不命中 → 分支 3。
  assert.equal(resolveClientIP(h, [], REAL_HEADER), '203.0.113.5');
});

test('4.3 白名单未配（解析为空）+ 已配头名 → 仍 fail-safe 用 cf', () => {
  const cidrs = parseTrustedCidrs('10.0.0.0/8, 0.0.0.0/0'); // 全被拒 → 空
  assert.deepEqual(cidrs, []);
  const h = headers({ 'CF-Connecting-IP': '203.0.113.5', 'Ali-Cdn-Real-Ip': '198.51.100.9' });
  assert.equal(resolveClientIP(h, cidrs, REAL_HEADER), '203.0.113.5');
});

test('4.3 CF-Connecting-IP 缺失 → unknown（共享惩罚桶）', () => {
  const h = headers({ 'Ali-Cdn-Real-Ip': '198.51.100.9', 'X-Forwarded-For': '5.5.5.5' });
  assert.equal(resolveClientIP(h, WL, REAL_HEADER), 'unknown');
});

test('4.3 CF-Connecting-IP 为空串 → unknown', () => {
  const h = headers({ 'CF-Connecting-IP': '' });
  assert.equal(resolveClientIP(h, WL, REAL_HEADER), 'unknown');
});

// ============================================================
// 1.5 运行时受信头值校验（纯函数直测，补强 4.2 的边界）
// ============================================================

test('1.5 isTrustedHeaderValueValid：单值合法 IPv4 / IPv6 → true', () => {
  assert.equal(isTrustedHeaderValueValid('198.51.100.9'), true);
  assert.equal(isTrustedHeaderValueValid('2001:db8::1'), true);
  assert.equal(isTrustedHeaderValueValid('  198.51.100.9  '), true); // 去空白后单值合法
});

test('1.5 isTrustedHeaderValueValid：含逗号 / 多值 / 内部空白 / 空 / 非法 → false', () => {
  assert.equal(isTrustedHeaderValueValid('8.8.8.8,198.51.100.9'), false); // 含逗号即多值
  assert.equal(isTrustedHeaderValueValid('8.8.8.8 198.51.100.9'), false); // 内部空白
  assert.equal(isTrustedHeaderValueValid(''), false);
  assert.equal(isTrustedHeaderValueValid('   '), false);
  assert.equal(isTrustedHeaderValueValid('not-an-ip'), false);
  assert.equal(isTrustedHeaderValueValid(null), false);
  assert.equal(isTrustedHeaderValueValid(undefined), false);
});

test('1.5 isTrustedHeaderValueValid：前导/尾随单冒号畸形 IPv6 串 → false（落 unknown）', () => {
  // 边界孤立单冒号（非 '::'）的非法字面量必须被拒，否则被采信为计数键。
  assert.equal(isTrustedHeaderValueValid(':1:2:3:4:5:6:7:8'), false); // 前导单冒号、8 非空组
  assert.equal(isTrustedHeaderValueValid('1:2:3:4:5:6:7:8:'), false); // 尾随单冒号
  assert.equal(isTrustedHeaderValueValid(':2001:db8::1'), false);      // 前导单冒号 + ::
  assert.equal(isTrustedHeaderValueValid(':1::'), false);              // 前导单冒号
  assert.equal(isTrustedHeaderValueValid('1::2:'), false);             // 尾随单冒号
});

test('1.5 isTrustedHeaderValueValid：合法 IPv6（含压缩/边界 ::）仍接受', () => {
  assert.equal(isTrustedHeaderValueValid('::1'), true);
  assert.equal(isTrustedHeaderValueValid('1::'), true);
  assert.equal(isTrustedHeaderValueValid('::'), true);
  assert.equal(isTrustedHeaderValueValid('2001:db8::1'), true);
  assert.equal(isTrustedHeaderValueValid('::ffff:1.2.3.4'), true);
  assert.equal(isTrustedHeaderValueValid('1:2:3:4:5:6:7:8'), true);
  assert.equal(isTrustedHeaderValueValid('fe80::1'), true);
});
