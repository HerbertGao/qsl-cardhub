/**
 * 同步 SHA-256 正确性强锚：对 NIST 已知向量验证，并与服务端 crypto.subtle（session.js sha256Hex）
 * 逐字节比对——保证客户端 PoW 算出的 nonce 在服务端 verifyPow（crypto.subtle）下成立。
 */
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { sha256Hex, leadingZeroBits } from '../src/worker/sha256.js';
import { sha256Hex as subtleSha256Hex } from '../src/worker/session.js';

test('sha256: NIST 已知向量', () => {
  assert.equal(sha256Hex(''), 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855');
  assert.equal(sha256Hex('abc'), 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
  assert.equal(
    sha256Hex('abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq'),
    '248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1'
  );
});

test('sha256: 与 node crypto 在随机/边界输入上一致', () => {
  const samples = ['', 'a', 'seed:0', 'seed:123456', '中文测试🚀', 'x'.repeat(1000), 'A'.repeat(56), 'B'.repeat(64)];
  for (const s of samples) {
    const expected = createHash('sha256').update(s, 'utf8').digest('hex');
    assert.equal(sha256Hex(s), expected, `mismatch for ${JSON.stringify(s.slice(0, 16))}`);
  }
});

test('sha256: 与服务端 crypto.subtle（session.js）一致——客户端 nonce 必能服务端校验', async () => {
  for (let i = 0; i < 50; i++) {
    const msg = `abcd:${i}`;
    assert.equal(sha256Hex(msg), await subtleSha256Hex(msg), `subtle mismatch for ${msg}`);
  }
});

test('sha256: leadingZeroBits 与服务端口径一致', () => {
  assert.equal(leadingZeroBits('00ff'), 8);
  assert.equal(leadingZeroBits('0fff'), 4);
  assert.equal(leadingZeroBits('8000'), 0);
});
