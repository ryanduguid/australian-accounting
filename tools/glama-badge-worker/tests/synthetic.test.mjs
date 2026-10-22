import assert from 'node:assert/strict';
import { writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { test } from 'node:test';
import { Miniflare, Response } from 'miniflare';

const origin = 'https://trial.example';
let outbound = 0;
const active = { SYNTHETIC_START_MS: String(Date.now() - 1_000), SYNTHETIC_END_MS: String(Date.now() + 120_000) };
function runtime(bindings) {
  return new Miniflare({
    modules: true,
    scriptPath: fileURLToPath(new URL('../build-synthetic/worker/shim.mjs', import.meta.url)),
    compatibilityDate: '2026-07-30',
    modulesRules: [{ type: 'ESModule', include: ['**/*.js'], fallthrough: true }, { type: 'CompiledWasm', include: ['**/*.wasm'], fallthrough: true }],
    bindings,
    outboundService: () => { outbound++; throw new Error('Synthetic trial must not make outbound requests'); },
  });
}
async function failure(response, status) {
  assert.equal(response.status, status);
  assert.equal(response.headers.get('cache-control'), 'no-store');
  assert.match(response.headers.get('content-type'), /^text\/plain/);
  await response.text();
}

await test('Synthetic-only compiled Worker', { timeout: 30_000 }, async t => {
  const mf = runtime(active);
  t.after(async () => { await mf.dispose(); assert.equal(outbound, 0); });
  await mf.ready;
  const fetch = (path = '/synthetic.svg', init) => mf.dispatchFetch(origin + path, init);
  let svg;
  await t.test('visible test label, centred fixture and security headers', async () => {
    const response = await fetch();
    assert.equal(response.status, 200);
    assert.equal(response.headers.get('x-synthetic-cache'), 'MISS');
    assert.equal(response.headers.get('cache-control'), 'public, max-age=60, must-revalidate');
    assert.equal(response.headers.get('content-type'), 'image/svg+xml; charset=utf-8');
    assert.equal(response.headers.get('content-security-policy'), "default-src 'none'; sandbox");
    assert.equal(response.headers.get('x-content-type-options'), 'nosniff');
    assert.equal(response.headers.get('referrer-policy'), 'no-referrer');
    svg = await response.text();
    assert.match(svg, /width="240" height="52" viewBox="0 0 240 52"/);
    assert.match(svg, /x="120" y="16" text-anchor="middle"/);
    assert.match(svg, /SYNTHETIC TEST DATA/);
    assert.match(svg, /Recorded fixture, not a live rating/);
    assert.match(svg, /<svg x="65" y="24"[^>]*width="110" height="20"/);
    assert.match(svg, /#FFD32A/);
    assert.doesNotMatch(svg, /linearGradient|<script|<foreignObject/);
    await writeFile(new URL('../build-synthetic/preview.svg', import.meta.url), svg);
  });
  await t.test('HEAD and warm GET reuse the Cache API without a body on HEAD', async () => {
    const head = await fetch('/synthetic.svg', { method: 'HEAD' });
    assert.equal(head.status, 200);
    assert.equal(head.headers.get('x-synthetic-cache'), 'HIT');
    assert.equal(await head.text(), '');
    const hit = await fetch();
    assert.equal(hit.headers.get('x-synthetic-cache'), 'HIT');
    assert.equal(await hit.text(), svg);
  });
  await t.test('live provider paths, queries and unknown routes are absent', async () => {
    for (const path of ['/', '/glama/servers/ryanduguid/australian-accounting.svg', '/synthetic.svg?', '/synthetic.svg?x=1', '/synthetic.svg/', '/%73ynthetic.svg']) {
      await failure(await fetch(path), 404);
    }
    const response = await fetch('/synthetic.svg', { method: 'POST', body: 'ignored' });
    assert.equal(response.headers.get('allow'), 'GET, HEAD');
    await failure(response, 405);
  });
  await t.test('deliberate failure stays uncached for GET and HEAD', async () => {
    await failure(await fetch('/failure.svg'), 503);
    const head = await fetch('/failure.svg', { method: 'HEAD' });
    assert.equal(head.status, 503);
    assert.equal(head.headers.get('cache-control'), 'no-store');
    assert.equal(await head.text(), '');
    await failure(await fetch('/failure.svg'), 503);
  });
  await t.test('invalid cached data is replaced and old timestamps do not extend freshness', async () => {
    const cache = (await mf.getCaches()).default;
    const key = `https://flat-glama-synthetic.invalid/v1/${active.SYNTHETIC_START_MS}/${active.SYNTHETIC_END_MS}.svg`;
    for (const entry of [{ body: svg, time: Date.now() - 301_000 }, { body: svg, time: Date.now() + 60_000 }, { body: new Uint8Array(65_537), time: Date.now() }, { body: new Uint8Array([255]), time: Date.now() }]) {
      await cache.put(key, new Response(entry.body, { headers: { 'cache-control': 'max-age=300', 'x-synthetic-rendered-at': String(entry.time) } }));
      const response = await fetch();
      assert.equal(response.headers.get('x-synthetic-cache'), 'MISS');
      assert.equal(await response.text(), svg);
    }
  });
  await t.test('32 simultaneous cold synthetic renders make no upstream calls', async () => {
    await mf.purgeCache();
    await Promise.all(Array.from({ length: 32 }, async () => {
      const response = await fetch();
      assert.equal(response.status, 200);
      assert.equal(await response.text(), svg);
    }));
    assert.equal(outbound, 0);
  });
  await t.test('missing, malformed, reversed, future, expired and oversized windows fail closed', async () => {
    const now = Date.now();
    for (const bindings of [{}, { SYNTHETIC_START_MS: 'bad', SYNTHETIC_END_MS: String(now + 10_000) }, { SYNTHETIC_START_MS: String(now), SYNTHETIC_END_MS: '18446744073709551616' }, { SYNTHETIC_START_MS: String(now + 60_000), SYNTHETIC_END_MS: String(now) }, { SYNTHETIC_START_MS: String(now + 60_000), SYNTHETIC_END_MS: String(now + 120_000) }, { SYNTHETIC_START_MS: '0', SYNTHETIC_END_MS: '0' }, { SYNTHETIC_START_MS: String(now - 60_000), SYNTHETIC_END_MS: String(now - 1) }, { SYNTHETIC_START_MS: String(now - 1_000), SYNTHETIC_END_MS: String(now + 3_600_000) }]) {
      const inactive = runtime(bindings);
      try { await failure(await inactive.dispatchFetch(origin + '/synthetic.svg'), 410); }
      finally { await inactive.dispose(); }
    }
  });
  await t.test('trial expiry bounds client freshness and overrides a warm cache', async () => {
    const now = Date.now();
    const expiring = runtime({ SYNTHETIC_START_MS: String(now - 1_000), SYNTHETIC_END_MS: String(now + 5_000) });
    try {
      const response = await expiring.dispatchFetch(origin + '/synthetic.svg');
      assert.equal(response.status, 200);
      assert.match(response.headers.get('cache-control'), /max-age=[0-4],/);
      await response.text();
      await new Promise(resolve => setTimeout(resolve, Math.max(0, now + 5_050 - Date.now())));
      await failure(await expiring.dispatchFetch(origin + '/synthetic.svg'), 410);
      const head = await expiring.dispatchFetch(origin + '/synthetic.svg', { method: 'HEAD' });
      assert.equal(head.status, 410);
      assert.equal(await head.text(), '');
    } finally { await expiring.dispose(); }
  });
});
