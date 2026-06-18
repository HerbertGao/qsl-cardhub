#!/usr/bin/env node
// 离线租户写凭据签发脚本：把「租户 slug + 写凭据 Key」转成可执行 SQL，供运维
// `wrangler d1 execute qsl-sync --remote --file=-` 自行执行。
//
// 不连任何数据库、不落明文 Key——输出只含 sha256(trim(key)) 的 hash。
// 数据走 stdout（纯 SQL），说明/错误走 stderr，退出码语义化（成功 0 / 校验失败非 0）。
//
// 用法：
//   node mint-credential.mjs <slug> <key>
//   printf %s "$KEY" | node mint-credential.mjs <slug> --key-stdin
//
// 与服务端 worker resolveTenant 的 sha256(trim(key)) 逐字节一致（UTF-8 SHA-256 → 小写 hex）。

import { createHash } from 'node:crypto';
import assert from 'node:assert';
import { readFileSync } from 'node:fs';

// 校验失败用异常流而非 process.exit()，避免 stdout 管道写入未刷新即终止导致 SQL 截断。
class MintError extends Error {}

// sha256(message) → 64 位小写 hex，与 worker 的
// crypto.subtle.digest('SHA-256', TextEncoder().encode(message)) → hex 同语义。
function sha256(message) {
  return createHash('sha256').update(message, 'utf8').digest('hex');
}

// 启动自检：对独立硬编码已知向量断言，证明 hash 实现没有相对 worker 漂移。
// （非用 createHash 现算自比自的恒真断言。）
assert(
  sha256('qsl-mint-selfcheck') ===
    '989e608711151a9484b398e3af86cb80f1449d3fa9824da303383c30f1d215fe',
  'self-check failed: sha256 implementation drifted from worker contract',
);

const SLUG_RE = /^[a-z0-9-]{1,32}$/; // 与 tenants.tenant_id CHECK 同语义；不加 m 标志
const MIN_KEY_LEN = 32;

function usage() {
  return [
    '用法：',
    '  printf %s "$KEY" | node mint-credential.mjs <slug> --key-stdin',
    '  node mint-credential.mjs <slug> <key>',
    '',
    'slug    租户标识，必须匹配 ^[a-z0-9-]{1,32}$（拒绝不转换）',
    'key     写凭据 Key（trim 后非空、长度 >= 32；建议 openssl rand -hex 32 生成）',
    '',
    '数据（SQL）走 stdout，可直接 | wrangler d1 execute qsl-sync --remote --file=-',
    '说明与错误走 stderr。',
  ].join('\n');
}

function fail(msg) {
  throw new MintError(msg);
}

function main() {
  const argv = process.argv.slice(2);

  // 无参：打印用法（stderr），退出码 0 视为请求帮助。
  if (argv.length === 0) {
    process.stderr.write(`${usage()}\n`);
    return;
  }

  // 极简 argv 解析：第一个非 --key-stdin 的位置参数为 slug，第二个为 key（可选）。
  const useStdin = argv.includes('--key-stdin');
  const positional = argv.filter((a) => a !== '--key-stdin');

  const slug = positional[0];
  if (slug === undefined) {
    fail(`缺少 slug。\n${usage()}`);
  }
  if (!SLUG_RE.test(slug)) {
    fail(
      `非法 slug：${JSON.stringify(slug)}。slug 必须匹配 ^[a-z0-9-]{1,32}$（小写字母/数字/连字符，1-32 字符），拒绝不转换。`,
    );
  }

  // Key 来源：--key-stdin 从 stdin 读，否则取第二个位置参数。两者不应混用。
  let rawKey;
  if (useStdin) {
    if (positional.length > 1) {
      fail('--key-stdin 与位置参数 Key 不能同时提供。');
    }
    if (process.stdin.isTTY) {
      fail('--key-stdin 需从管道读取 Key，例如：printf %s "$KEY" | node scripts/mint-credential.mjs <slug> --key-stdin');
    }
    rawKey = readFileSync(0, 'utf8'); // fd 0 = stdin
  } else {
    if (positional.length < 2) {
      fail(`缺少 Key（提供位置参数 Key 或使用 --key-stdin）。\n${usage()}`);
    }
    if (positional.length > 2) {
      fail('参数过多。');
    }
    rawKey = positional[1];
    process.stderr.write('警告：Key 经命令行参数传入，会出现在 ps / shell history；推荐改用 --key-stdin。\n');
  }

  const key = rawKey.trim(); // 吸收 stdin 尾随 \n/CRLF；与 worker trim 同语义
  if (key === '') {
    fail('Key 经 trim 后为空，拒绝（绝不输出 sha256("")）。');
  }
  if (key.length < MIN_KEY_LEN) {
    fail(
      `Key 经 trim 后长度 ${key.length} < ${MIN_KEY_LEN}，过短易被离线爆破。请用高熵 Key，例如：openssl rand -hex 32`,
    );
  }

  const keyHash = sha256(key); // 完整 64 位 hex
  const id = `${slug}-${keyHash}`; // id 用完整 hash 派生，不截断

  // slug 已经正则限制为 [a-z0-9-]、keyHash 为 hex、scope 为常量 'sync'——SQL 串内无任何自由文本插值（无注入面）。
  // tenants 行幂等（OR IGNORE）；tenant_credentials 行普通 INSERT（非 OR IGNORE），
  // 使重签 / 跨租户复用同一 Key 由约束安全失败报错（不静默覆写/吞掉）。
  const sql =
    `INSERT OR IGNORE INTO tenants (tenant_id, name, status) VALUES ('${slug}', '${slug}', 'active');\n` +
    `INSERT INTO tenant_credentials (id, tenant_id, scope, key_hash, status) VALUES ('${id}', '${slug}', 'sync', '${keyHash}', 'active');\n`;

  process.stdout.write(sql);
}

try {
  main();
} catch (e) {
  if (e instanceof MintError) {
    process.stderr.write(`错误：${e.message}\n`);
    process.exitCode = 1;
  } else {
    throw e;
  }
}
