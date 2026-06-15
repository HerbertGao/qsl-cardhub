/**
 * trusted-client-ip 纯函数单测。
 *
 * 覆盖黑盒（worker smoke）打不到的边界，与
 * openspec/changes/fix-cdn-real-ip/specs/trusted-client-ip/spec.md 的判定规则与全部场景对齐。
 *
 * 信任信号 = 密钥回源头 X-Origin-Auth（CDN 覆盖语义注入）+ 受信真实 IP 头 Ali-Cdn-Real-Ip。
 *
 * 运行：在 web_query_service 目录执行  node --test verify/client-ip.test.js  或  pnpm run test:unit
 * 仅用 node 内置 node:test + node:assert，无新增 devDependency。
 * resolveClientIP 第一个入参是 Fetch Headers 对象（node 18+ 全局有 Headers）；
 * 多值头用 h.append(name, v) 两次构造（Headers.get 会以 ", " 拼接 → 触发含逗号即多值的判定）。
 */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  isTrustedHeaderValueValid,
  resolveClientIP,
  getClientIP,
} from '../src/worker/client-ip.js';

const SECRET = 'a'.repeat(64); // 模拟 openssl rand -hex 32（64 hex 字符）
const REAL_HEADER = 'Ali-Cdn-Real-Ip';
const AUTH_HEADER = 'X-Origin-Auth';

// 用对象构造 Headers（单值用 set）。
function headers(obj) {
  const h = new Headers();
  for (const [k, v] of Object.entries(obj)) {
    if (v !== undefined && v !== null) h.set(k, v);
  }
  return h;
}
// 构造含多值受信头的 Headers（append 两次 → get 以 ", " 拼接）。
function headersMultiReal(base, realValues) {
  const h = headers(base);
  for (const v of realValues) h.append(REAL_HEADER, v);
  return h;
}

// ── isTrustedHeaderValueValid（受信头值运行时校验）──────────────────────────
test('isTrustedHeaderValueValid：合法 IPv4/IPv6 单值 → true', () => {
  for (const v of ['1.2.3.4', '203.0.113.45', '200.1.2.3', '255.255.255.255', '::1', '2001:db8::1', '::ffff:1.2.3.4', 'fe80::1']) {
    assert.equal(isTrustedHeaderValueValid(v), true, `应接受 ${v}`);
  }
});
test('isTrustedHeaderValueValid：带两侧空白的单值 → true（采信去空白）', () => {
  assert.equal(isTrustedHeaderValueValid('  1.2.3.4  '), true);
});
test('isTrustedHeaderValueValid：含逗号/多值/内部空白/空/非法 → false（禁 split 取段，F1 防护）', () => {
  for (const v of ['8.8.8.8,1.2.3.4', '8.8.8.8, 1.2.3.4', '8.8.8.8 1.2.3.4', '', '   ', 'not-an-ip', '256.1.2.3', '01.2.3.4', null, undefined]) {
    assert.equal(isTrustedHeaderValueValid(v), false, `应拒 ${JSON.stringify(v)}`);
  }
});
test('isTrustedHeaderValueValid：前导/尾随单冒号 IPv6 畸形串 → false', () => {
  for (const v of [':1:2:3:4:5:6:7:8', '1:2:3:4:5:6:7:8:', ':2001:db8::1', ':1::', '1::2:', ':::', '2001::db8::1', 'gggg::', 'fe80::1%eth0']) {
    assert.equal(isTrustedHeaderValueValid(v), false, `应拒 ${v}`);
  }
});

// ── resolveClientIP：分支语义 ──────────────────────────────────────────────
test('cf 为空/缺失 → unknown', () => {
  assert.equal(resolveClientIP(headers({}), SECRET, REAL_HEADER), 'unknown');
  assert.equal(resolveClientIP(headers({ 'CF-Connecting-IP': '' }), SECRET, REAL_HEADER), 'unknown');
});

test('密钥未配（originSecret 空）→ 返回 cf、忽略一切注入头', () => {
  const h = headers({ 'CF-Connecting-IP': '5.5.5.5', [AUTH_HEADER]: SECRET, [REAL_HEADER]: '1.2.3.4' });
  assert.equal(resolveClientIP(h, '', REAL_HEADER), '5.5.5.5');
  assert.equal(resolveClientIP(h, undefined, REAL_HEADER), '5.5.5.5');
});

test('密钥已配 + 无 X-Origin-Auth → 返回 cf（不采信注入头）', () => {
  const h = headers({ 'CF-Connecting-IP': '5.5.5.5', [REAL_HEADER]: '1.2.3.4' });
  assert.equal(resolveClientIP(h, SECRET, REAL_HEADER), '5.5.5.5');
});

test('密钥已配 + 错误 X-Origin-Auth（伪造）→ 返回 cf（伪造密钥无效）', () => {
  const h = headers({ 'CF-Connecting-IP': '5.5.5.5', [AUTH_HEADER]: 'b'.repeat(64), [REAL_HEADER]: '1.2.3.4' });
  assert.equal(resolveClientIP(h, SECRET, REAL_HEADER), '5.5.5.5');
});

test('密钥已配 + 等长但不同密钥 → 返回 cf（常量时间比对不放过）', () => {
  const wrong = 'a'.repeat(63) + 'b';
  const h = headers({ 'CF-Connecting-IP': '5.5.5.5', [AUTH_HEADER]: wrong, [REAL_HEADER]: '1.2.3.4' });
  assert.equal(resolveClientIP(h, SECRET, REAL_HEADER), '5.5.5.5');
});

test('密钥正确 + 真实头合法 → 采信受信头（去空白）', () => {
  const h = headers({ 'CF-Connecting-IP': '5.5.5.5', [AUTH_HEADER]: SECRET, [REAL_HEADER]: '  1.2.3.4 ' });
  assert.equal(resolveClientIP(h, SECRET, REAL_HEADER), '1.2.3.4');
});

test('密钥正确 + 真实头为合法 IPv6 → 采信', () => {
  const h = headers({ 'CF-Connecting-IP': '5.5.5.5', [AUTH_HEADER]: SECRET, [REAL_HEADER]: '2001:db8::1' });
  assert.equal(resolveClientIP(h, SECRET, REAL_HEADER), '2001:db8::1');
});

test('密钥正确 + 伪造 XFF → 绝不采信 XFF（取受信头/cf，不取 XFF）', () => {
  const h = headers({ 'CF-Connecting-IP': '5.5.5.5', [AUTH_HEADER]: SECRET, [REAL_HEADER]: '1.2.3.4', 'X-Forwarded-For': '6.6.6.6, 1.2.3.4' });
  assert.equal(resolveClientIP(h, SECRET, REAL_HEADER), '1.2.3.4');
});

test('密钥正确 + 真实头名未配 → fail-safe 到 cf', () => {
  const h = headers({ 'CF-Connecting-IP': '5.5.5.5', [AUTH_HEADER]: SECRET, [REAL_HEADER]: '1.2.3.4' });
  assert.equal(resolveClientIP(h, SECRET, ''), '5.5.5.5');
  assert.equal(resolveClientIP(h, SECRET, undefined), '5.5.5.5');
});

test('密钥正确 + 头名误配为 X-Forwarded-For（任意大小写）→ 强制拒绝、绝不采信 XFF、fail-safe 到 cf', () => {
  const h = headers({ 'CF-Connecting-IP': '5.5.5.5', [AUTH_HEADER]: SECRET, 'X-Forwarded-For': '6.6.6.6' });
  assert.equal(resolveClientIP(h, SECRET, 'X-Forwarded-For'), '5.5.5.5');
  assert.equal(resolveClientIP(h, SECRET, 'x-forwarded-for'), '5.5.5.5');
  assert.equal(resolveClientIP(h, SECRET, 'X-FORWARDED-FOR'), '5.5.5.5');
  assert.equal(resolveClientIP(h, SECRET, '  X-Forwarded-For  '), '5.5.5.5');
});

test('密钥正确 + 头名为非法/非 ASCII（全角、含空格、零宽）→ fail-safe 到 cf（不抛、不绕过）', () => {
  const h = headers({ 'CF-Connecting-IP': '5.5.5.5', [AUTH_HEADER]: SECRET });
  for (const bad of ['ｘ-forwarded-for', 'X Forwarded For', 'real ip', 'Ali-Cdn-Real-Ip​', '真实IP头']) {
    assert.equal(resolveClientIP(h, SECRET, bad), '5.5.5.5', `非法头名 ${JSON.stringify(bad)} 应 fail-safe 到 cf`);
  }
});

test('密钥正确 + 真实头缺失/空/非法 → unknown（不退回 cf 节点桶）', () => {
  assert.equal(resolveClientIP(headers({ 'CF-Connecting-IP': '5.5.5.5', [AUTH_HEADER]: SECRET }), SECRET, REAL_HEADER), 'unknown');
  assert.equal(resolveClientIP(headers({ 'CF-Connecting-IP': '5.5.5.5', [AUTH_HEADER]: SECRET, [REAL_HEADER]: '' }), SECRET, REAL_HEADER), 'unknown');
  assert.equal(resolveClientIP(headers({ 'CF-Connecting-IP': '5.5.5.5', [AUTH_HEADER]: SECRET, [REAL_HEADER]: 'not-an-ip' }), SECRET, REAL_HEADER), 'unknown');
});

test('密钥正确 + 真实头多值（透传+追加）→ unknown（含逗号即拒，F1 防护）', () => {
  const h = headersMultiReal({ 'CF-Connecting-IP': '5.5.5.5', [AUTH_HEADER]: SECRET }, ['8.8.8.8', '1.2.3.4']);
  assert.equal(resolveClientIP(h, SECRET, REAL_HEADER), 'unknown');
});

test('密钥正确 + 单值透传伪造真实头（如 CDN 未覆写）→ 仍按字面采信（覆写由证伪门保证、非运行时）', () => {
  // 该 case 说明：运行时无法区分「CDN 覆写的单值」与「客户端透传的单值」；
  // 但前提是密钥已校验通过（=确来自 CDN），而密钥头同样由 CDN 覆写、客户端伪造不了。
  const h = headers({ 'CF-Connecting-IP': '5.5.5.5', [AUTH_HEADER]: SECRET, [REAL_HEADER]: '8.8.8.8' });
  assert.equal(resolveClientIP(h, SECRET, REAL_HEADER), '8.8.8.8');
});

// ── getClientIP 包装 + env 缺字段安全 ──────────────────────────────────────
test('getClientIP：env 缺字段（undefined）安全 → 直连按 cf', () => {
  const req = { headers: headers({ 'CF-Connecting-IP': '9.9.9.9', [AUTH_HEADER]: SECRET, [REAL_HEADER]: '1.2.3.4' }) };
  assert.equal(getClientIP(req, undefined), '9.9.9.9'); // env 无 → 密钥未配 → cf
  assert.equal(getClientIP(req, {}), '9.9.9.9');
});

test('getClientIP：env 配齐密钥+头名 → 采信真实头', () => {
  const req = { headers: headers({ 'CF-Connecting-IP': '5.5.5.5', [AUTH_HEADER]: SECRET, [REAL_HEADER]: '1.2.3.4' }) };
  assert.equal(getClientIP(req, { CDN_ORIGIN_SECRET: SECRET, CDN_REAL_IP_HEADER: REAL_HEADER }), '1.2.3.4');
});

test('getClientIP：env 配密钥但请求无密钥（直连伪造真实头）→ 按 cf，伪造头无效', () => {
  const req = { headers: headers({ 'CF-Connecting-IP': '9.9.9.9', [REAL_HEADER]: '1.2.3.4', 'X-Forwarded-For': '6.6.6.6' }) };
  assert.equal(getClientIP(req, { CDN_ORIGIN_SECRET: SECRET, CDN_REAL_IP_HEADER: REAL_HEADER }), '9.9.9.9');
});
