// 4-C1 单测：resolveTenant（{tenant,viaFallback} + readonly）与 crossCheckTenant（声明租户交叉校验）纯逻辑。
// 用 mock env.DB 隔离 D1，跑在 `node --test`（无需 wrangler dev）。HTTP 端到端见 run_worker_smoke.sh 的 4.7 段。
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { resolveTenant, crossCheckTenant } from '../src/worker/index.js';

// key_hash = sha256(trim(key)) 的 hex（与 worker 内 sha256 对 NIST 向量一致，见 sha256.test.js）
const keyHash = (k) => createHash('sha256').update((k || '').trim()).digest('hex');

// 构造 fake env：tenant_credentials 按 key_hash→tenant_id 命中；service_counters 计 auth_fallback UPDATE 次数
function makeEnv({ creds = {}, apiKey = '', counterChanges = 1 } = {}) {
  let fallbackUpdates = 0;
  return {
    API_KEY: apiKey,
    _fallbackUpdates: () => fallbackUpdates,
    DB: {
      prepare(sql) {
        const q = { args: [] };
        q.bind = (...a) => { q.args = a; return q; };
        q.first = async () => {
          if (sql.includes('tenant_credentials')) {
            const tid = creds[q.args[0]];
            return tid ? { tenant_id: tid } : null;
          }
          return null;
        };
        q.run = async () => {
          if (sql.includes('auth_fallback')) fallbackUpdates += 1;
          return { meta: { changes: counterChanges } };
        };
        return q;
      },
    },
  };
}

test('resolveTenant: 表驱动命中 → {tenant, viaFallback:false}', async () => {
  const env = makeEnv({ creds: { [keyHash('keyA')]: 'tenantA' } });
  assert.deepEqual(await resolveTenant(env, 'keyA'), { tenant: 'tenantA', viaFallback: false });
});

test('resolveTenant: 空/纯空白 key → tenant null（永不鉴权）', async () => {
  const env = makeEnv({ apiKey: 'x' });
  assert.deepEqual(await resolveTenant(env, ''), { tenant: null, viaFallback: false });
  assert.deepEqual(await resolveTenant(env, '   '), { tenant: null, viaFallback: false });
});

test('resolveTenant: env.API_KEY 兜底命中 → bh2ro + viaFallback:true + 计数+1', async () => {
  const env = makeEnv({ apiKey: 'envkey' });
  assert.deepEqual(await resolveTenant(env, 'envkey'), { tenant: 'bh2ro', viaFallback: true });
  assert.equal(env._fallbackUpdates(), 1);
});

test('resolveTenant: readonly 兜底命中不计兜底（核心：/ping 不污染撤兜底验收）', async () => {
  const env = makeEnv({ apiKey: 'envkey' });
  assert.deepEqual(await resolveTenant(env, 'envkey', { readonly: true }), { tenant: 'bh2ro', viaFallback: true });
  assert.equal(env._fallbackUpdates(), 0);
});

test('resolveTenant: 兜底计数行缺失（changes=0）→ 抛错（非 readonly fail-closed）', async () => {
  const env = makeEnv({ apiKey: 'envkey', counterChanges: 0 });
  await assert.rejects(() => resolveTenant(env, 'envkey'));
});

test('resolveTenant: 表驱动命中优先于兜底（不触发计数）', async () => {
  // key 既在表又等于 env.API_KEY：表命中先返回，不进兜底分支
  const env = makeEnv({ creds: { [keyHash('dual')]: 'tenantD' }, apiKey: 'dual' });
  assert.deepEqual(await resolveTenant(env, 'dual'), { tenant: 'tenantD', viaFallback: false });
  assert.equal(env._fallbackUpdates(), 0);
});

test('crossCheckTenant: 缺声明（空串/null）→ 放行（向后兼容）', async () => {
  const env = makeEnv({ creds: { [keyHash('keyA')]: 'tenantA' } });
  assert.deepEqual(await crossCheckTenant(env, 'keyA', ''), { ok: true, tenant: 'tenantA', viaFallback: false });
  assert.deepEqual(await crossCheckTenant(env, 'keyA', null), { ok: true, tenant: 'tenantA', viaFallback: false });
});

test('crossCheckTenant: 声明一致 → 放行', async () => {
  const env = makeEnv({ creds: { [keyHash('keyA')]: 'tenantA' } });
  assert.deepEqual(await crossCheckTenant(env, 'keyA', 'tenantA'), { ok: true, tenant: 'tenantA', viaFallback: false });
});

test('crossCheckTenant: 声明不一致 → 403，且返回 tenant 恒为解析值 A（红线，非声明值 B）', async () => {
  const env = makeEnv({ creds: { [keyHash('keyA')]: 'tenantA' } });
  const r = await crossCheckTenant(env, 'keyA', 'tenantB');
  assert.equal(r.ok, false);
  assert.equal(r.status, 403);
  assert.equal(r.code, 'tenant_mismatch');
  assert.equal(r.tenant, 'tenantA'); // ★红线：解析值 A，绝不是声明值 B
});

test('crossCheckTenant: key 无效 → 401', async () => {
  const env = makeEnv({});
  assert.deepEqual(await crossCheckTenant(env, 'badkey', 'tenantA'), { ok: false, status: 401, code: 'auth_failed' });
});

test('crossCheckTenant: opts.readonly 透传给 resolveTenant（/ping 路径不计兜底）', async () => {
  const env = makeEnv({ apiKey: 'envkey' });
  await crossCheckTenant(env, 'envkey', 'bh2ro', { readonly: true });
  assert.equal(env._fallbackUpdates(), 0);
});
